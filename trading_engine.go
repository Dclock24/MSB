package main

import (
	"encoding/json"
	"fmt"
	"log"
	"crypto/hmac"
	"crypto/sha256"
	"crypto/sha512"
	"encoding/base64"
	"math/rand"
	"os/exec"
	"net/http"
	"net/url"
	"os"
	"strconv"
	"strings"
	"sync/atomic"
	"time"
)

// StrikeType represents different types of macro strikes
type StrikeType int

const (
	MacroArbitrage StrikeType = iota
	MacroMomentum
	MacroVolatility
	MacroLiquidity
	MacroFunding
	MacroFlash
)

// StrikeStatus represents the status of a strike
type StrikeStatus int

const (
	Targeting StrikeStatus = iota
	Striking
	Hit
	Miss
	Aborted
)

// MarketAnalysis represents comprehensive market analysis data
type MarketAnalysis struct {
	Symbol         string  `json:"symbol"`
	StrikeType     string  `json:"strike_type"`
	Price          float64 `json:"price"`
	Confidence     float64 `json:"confidence"`
	ExpectedReturn float64 `json:"expected_return"`
	Volatility     float64 `json:"volatility"`
	Momentum       float64 `json:"momentum"`
	Liquidity      float64 `json:"liquidity"`
	PrecisionScore float64 `json:"precision_score"`
	Recommendation string  `json:"recommendation"`
	Timestamp      int64   `json:"timestamp"`
}

// MacroStrike represents a trading strike
type MacroStrike struct {
	ID                uint64      `json:"id"`
	Symbol            string      `json:"symbol"`
	StrikeType        StrikeType  `json:"strike_type"`
	EntryPrice        float64     `json:"entry_price"`
	TargetPrice       float64     `json:"target_price"`
	StopLoss          float64     `json:"stop_loss"`
	Confidence        float64     `json:"confidence"`
	ExpectedReturn    float64     `json:"expected_return"`
	MaxExposureTimeMs uint64      `json:"max_exposure_time_ms"`
	StrikeForce       float64     `json:"strike_force"`
	Timestamp         int64       `json:"timestamp"`
	Status            StrikeStatus `json:"status"`
	HitTime           *int64      `json:"hit_time,omitempty"`
	ExitPrice         *float64    `json:"exit_price,omitempty"`
	PnL               *float64    `json:"pnl,omitempty"`
	Leverage          uint32      `json:"leverage"`
}

// TradingEngine handles the core trading logic
type TradingEngine struct {
	Capital            int64
	TargetCapital      int64
	PeakCapital        int64
	NextStrikeID       uint64
	ConsecutiveMisses  int64
	MaxConsecutiveMisses int64
	TotalStrikes       int64
	SuccessfulStrikes  int64
	FailedStrikes      int64
	TotalPnL           int64
	TradesCompleted    int64

	// Live trading config
	LiveTrading        bool
	KrakenAPIKey       string
	KrakenAPISecret    string
	OrderUSDSize       float64

	// Risk & campaign
	OrderRiskPct       float64
	CampaignStart      time.Time
	CampaignDays       int
	MaxDrawdownPct     float64
}

// Constants
const (
	TotalTrades           = 2500
	InitialCapital        = 10000000  // $100k in cents
	TargetCapital         = 11850000  // $118.5k in cents (18.5% in window)
	StrikeForce           = 0.15      // 15% of capital per strike
	PrecisionThreshold    = 0.85      // 85% confidence required
	MaxExposureTimeMs     = 30000     // 30 seconds max exposure
	StrikeCooldownMs      = 1         // 1ms cooldown
	MaxConsecutiveMisses  = 20        // Max consecutive misses before emergency stop
)

// Leverage policy (applies in simulation/PNL model). Live spot orders are unlevered but log intended leverage.
const (
    MinLeverage = 3
    MaxLeverage = 5
)

// Simulation PnL parameters
const (
    RoundTripFeePct = 0.0016 // 0.16% total fees
    SimTakeProfitPct = 0.003 // 0.30% TP
    SimStopLossPct   = 0.0025 // 0.25% SL
)

var symbols = []string{
	"WETH/USDC", "WBTC/USDC", "LINK/USDC", "UNI/USDC",
	"AAVE/USDC", "CRV/USDC", "USDC/USDT", "DAI/USDC",
}

var basePrices = []float64{
	3000.0, 45000.0, 15.50, 8.50, 120.0, 0.85, 1.00, 1.00,
}

// NewTradingEngine creates a new trading engine
func NewTradingEngine() *TradingEngine {
	live := os.Getenv("LIVE_TRADING") == "1"
	orderSize := 25.0
	if v := os.Getenv("ORDER_USD_SIZE"); v != "" {
		if f, err := strconv.ParseFloat(v, 64); err == nil && f > 0 {
			orderSize = f
		}
	}
	orderRisk := 0.01
	if v := os.Getenv("ORDER_RISK_PCT"); v != "" {
		if f, err := strconv.ParseFloat(v, 64); err == nil && f > 0 {
			orderRisk = f / 100.0
		}
	}
	campaignDays := 5
	if v := os.Getenv("CAMPAIGN_DAYS"); v != "" {
		if n, err := strconv.Atoi(v); err == nil && n > 0 {
			campaignDays = n
		}
	}
	maxDD := 10.0
	if v := os.Getenv("MAX_DRAWDOWN_PCT"); v != "" {
		if f, err := strconv.ParseFloat(v, 64); err == nil && f > 0 {
			maxDD = f
		}
	}
	te := &TradingEngine{
		Capital:             InitialCapital,
		TargetCapital:       TargetCapital,
		PeakCapital:         InitialCapital,
		NextStrikeID:        1,
		ConsecutiveMisses:   0,
		MaxConsecutiveMisses: MaxConsecutiveMisses,
		LiveTrading:         live,
		KrakenAPIKey:        os.Getenv("KRAKEN_API_KEY"),
		KrakenAPISecret:     os.Getenv("KRAKEN_API_SECRET"),
		OrderUSDSize:        orderSize,
		OrderRiskPct:        orderRisk,
		CampaignStart:       time.Now(),
		CampaignDays:        campaignDays,
		MaxDrawdownPct:      maxDD,
	}
	// In simulation mode, raise target capital to avoid early stop
	if os.Getenv("SIM_MODE") == "1" {
		te.TargetCapital = te.Capital * 100 // allow growth without early stop
	}
	return te
}

// krakenPair maps our symbol to Kraken's pair code
func (te *TradingEngine) krakenPair(symbol string) string {
	switch symbol {
	case "WETH/USDC":
		return "ETHUSD"
	case "WBTC/USDC":
		return "XBTUSD"
	case "LINK/USDC":
		return "LINKUSD"
	case "UNI/USDC":
		return "UNIUSD"
	case "AAVE/USDC":
		return "AAVEUSD"
	case "CRV/USDC":
		return "CRVUSD"
	case "USDC/USDT":
		return "USDCUSD"
	case "DAI/USDC":
		return "DAIUSD"
	default:
		return ""
	}
}

// krakenPrivate performs a signed private API request
func (te *TradingEngine) krakenPrivate(path string, data url.Values) (map[string]interface{}, error) {
	if te.KrakenAPIKey == "" || te.KrakenAPISecret == "" {
		return nil, fmt.Errorf("kraken credentials not set")
	}

	nonce := fmt.Sprintf("%d", time.Now().UnixNano()/int64(time.Millisecond))
	data.Set("nonce", nonce)
	postData := data.Encode()

	sha := sha256.Sum256([]byte(nonce + postData))
	msg := append([]byte(path), sha[:]...)

	secret, err := base64.StdEncoding.DecodeString(te.KrakenAPISecret)
	if err != nil {
		return nil, fmt.Errorf("invalid kraken secret: %v")
	}

	mac := hmac.New(sha512.New, secret)
	mac.Write(msg)
	signature := base64.StdEncoding.EncodeToString(mac.Sum(nil))

	req, err := http.NewRequest("POST", "https://api.kraken.com"+path, strings.NewReader(postData))
	if err != nil {
		return nil, err
	}
	req.Header.Set("API-Key", te.KrakenAPIKey)
	req.Header.Set("API-Sign", signature)
	req.Header.Set("Content-Type", "application/x-www-form-urlencoded; charset=utf-8")

	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	var out map[string]interface{}
	if err := json.NewDecoder(resp.Body).Decode(&out); err != nil {
		return nil, err
	}
	if errs, ok := out["error"].([]interface{}); ok && len(errs) > 0 {
		return nil, fmt.Errorf("kraken error: %v", errs)
	}
	return out, nil
}
// krakenPrivateWithRetry wraps krakenPrivate with simple retry/backoff
func (te *TradingEngine) krakenPrivateWithRetry(path string, data url.Values) (map[string]interface{}, error) {
    var lastErr error
    for i := 0; i < 3; i++ {
        res, err := te.krakenPrivate(path, data)
        if err == nil {
            return res, nil
        }
        lastErr = err
        time.Sleep(time.Duration(500*(i+1)) * time.Millisecond)
    }
    return nil, lastErr
}

// placeMarketOrder places a market buy order sized by USD
func (te *TradingEngine) placeMarketOrder(pair string, side string, usdSize float64, price float64) (string, error) {
	if usdSize <= 0 || price <= 0 {
		return "", fmt.Errorf("invalid size/price")
	}
	volume := usdSize / price
	vals := url.Values{}
	vals.Set("pair", pair)
	vals.Set("type", side)
	vals.Set("ordertype", "market")
	vals.Set("volume", fmt.Sprintf("%.8f", volume))

	res, err := te.krakenPrivateWithRetry("/0/private/AddOrder", vals)
	if err != nil {
		return "", err
	}
	if result, ok := res["result"].(map[string]interface{}); ok {
		if txids, ok := result["txid"].([]interface{}); ok && len(txids) > 0 {
			return fmt.Sprintf("%v", txids[0]), nil
		}
	}
	return "", fmt.Errorf("unexpected kraken response")
}

// getOrder retrieves order info
func (te *TradingEngine) getOrder(txid string) (map[string]interface{}, error) {
    vals := url.Values{}
    vals.Set("txid", txid)
    return te.krakenPrivateWithRetry("/0/private/QueryOrders", vals)
}

// placeMarketExit sells the filled quantity at market
func (te *TradingEngine) placeMarketExit(pair string, volume float64) (string, error) {
    vals := url.Values{}
    vals.Set("pair", pair)
    vals.Set("type", "sell")
    vals.Set("ordertype", "market")
    vals.Set("volume", fmt.Sprintf("%.8f", volume))
    res, err := te.krakenPrivateWithRetry("/0/private/AddOrder", vals)
    if err != nil { return "", err }
    if result, ok := res["result"].(map[string]interface{}); ok {
        if txids, ok := result["txid"].([]interface{}); ok && len(txids) > 0 {
            return fmt.Sprintf("%v", txids[0]), nil
        }
    }
    return "", fmt.Errorf("unexpected kraken response")
}

// GetMarketAnalysis fetches market analysis using Julia script
func (te *TradingEngine) GetMarketAnalysis(symbol string, strikeType string) (*MarketAnalysis, error) {
	cmd := exec.Command("julia", "market_analysis.jl", symbol, strikeType)
	output, err := cmd.Output()
	if err != nil {
		return nil, fmt.Errorf("failed to get market analysis: %v", err)
	}

	var analysis MarketAnalysis
	if err := json.Unmarshal(output, &analysis); err != nil {
		return nil, fmt.Errorf("failed to parse market analysis: %v", err)
	}

	return &analysis, nil
}

// GenerateStrike creates a new trading strike
func (te *TradingEngine) GenerateStrike() (*MacroStrike, error) {
	strikeID := atomic.AddUint64(&te.NextStrikeID, 1)
	symbolID := int(strikeID) % len(symbols)
	symbol := symbols[symbolID]

	// Generate strike type
	strikeType := StrikeType(int(strikeID) % 6)
	strikeTypeName := te.getStrikeTypeName(strikeType)

	// Simulation mode: bypass Julia, generate high-confidence strikes
	if os.Getenv("SIM_MODE") == "1" {
		basePrice := basePrices[symbolID]
		expectedReturn := te.getExpectedReturn(strikeType)
		conf := 0.80 + rand.Float64()*0.15 // 0.80 - 0.95
		return &MacroStrike{
			ID:                strikeID,
			Symbol:            symbol,
			StrikeType:        strikeType,
			EntryPrice:        basePrice,
			TargetPrice:       basePrice * (1.0 + expectedReturn),
			StopLoss:          basePrice * 0.98,
			Confidence:        conf,
			ExpectedReturn:    expectedReturn,
			MaxExposureTimeMs: MaxExposureTimeMs,
			StrikeForce:       0.0,
			Timestamp:         time.Now().Unix(),
			Status:            Targeting,
			Leverage:          1,
		}, nil
	}

	// Get market analysis from Julia
	analysis, err := te.GetMarketAnalysis(symbol, strikeTypeName)
	if err != nil {
		// For accuracy: skip when analysis is unavailable
		return nil, fmt.Errorf("skip: analysis unavailable")
	}

	// Use Julia analysis for strike parameters
	entryPrice := analysis.Price
	confidence := analysis.Confidence
	expectedReturn := analysis.ExpectedReturn

	// Use Julia's precision score to adjust confidence
	precisionAdjustedConfidence := confidence * analysis.PrecisionScore

	// Disable soft TA gate for accuracy-only mode
	allowSoft := false

	// Proceed if EXECUTE with high confidence or soft gate approves
	if !(analysis.Recommendation == "EXECUTE" && precisionAdjustedConfidence >= 0.80) && !allowSoft {
		// Skip low-quality setups; caller will try next without counting a trade
		return nil, fmt.Errorf("skip: %s conf=%.2f", analysis.Recommendation, precisionAdjustedConfidence)
	}

	return &MacroStrike{
		ID:                strikeID,
		Symbol:            symbol,
		StrikeType:        strikeType,
		EntryPrice:        entryPrice,
		TargetPrice:       entryPrice * (1.0 + expectedReturn),
		StopLoss:          entryPrice * 0.98, // 2% stop loss
		Confidence:        precisionAdjustedConfidence,
		ExpectedReturn:    expectedReturn,
		MaxExposureTimeMs: MaxExposureTimeMs,
		StrikeForce:       0.0, // Will be calculated
		Timestamp:         time.Now().Unix(),
		Status:            Targeting,
		Leverage:          1,
	}, nil
}

// ExecuteStrike executes a trading strike
func (te *TradingEngine) ExecuteStrike(strike *MacroStrike) (float64, error) {
	// Calculate strike size
	currentCapital := float64(atomic.LoadInt64(&te.Capital)) / 100.0
	strikeSize := currentCapital * StrikeForce * strike.Confidence

	// Enforce leverage policy 3x-5x in PnL model
	intendedLeverage := float64(MinLeverage)
	if strike.StrikeType == MacroMomentum || strike.StrikeType == MacroVolatility {
		intendedLeverage = float64(MaxLeverage)
	}
	strike.Leverage = uint32(intendedLeverage)
	strikeSize *= intendedLeverage

	// In simulation, cap position by risk percent of equity
	if os.Getenv("SIM_MODE") == "1" && te.OrderRiskPct > 0 {
		// risk per trade in USD
		riskUSD := currentCapital * te.OrderRiskPct
		// size so that loss at stop equals riskUSD
		stopPct := SimStopLossPct
		maxSizeByRisk := riskUSD / (stopPct * intendedLeverage)
		if maxSizeByRisk < strikeSize {
			strikeSize = maxSizeByRisk
		}
	}

	strike.StrikeForce = strikeSize
	strike.Status = Striking

	if te.LiveTrading {
		// LIVE: place a market buy of OrderUSDSize on Kraken for the pair at current entry price
		pair := te.krakenPair(strike.Symbol)
		if pair == "" {
			return 0, fmt.Errorf("no kraken pair for %s", strike.Symbol)
		}
		// Use entry price as indicative; Kraken market order uses book
		txid, err := te.placeMarketOrder(pair, "buy", te.OrderUSDSize, strike.EntryPrice)
		if err != nil {
			return 0, err
		}
		log.Printf("LIVE ORDER: %s buy $%.2f @ ~%.2f (txid=%s)", pair, te.OrderUSDSize, strike.EntryPrice, txid)

		// Poll fills briefly (up to 30s)
		var filledVolume float64
		buyPrice := strike.EntryPrice
		start := time.Now()
		for time.Since(start) < 30*time.Second {
			ord, err := te.getOrder(txid)
			if err == nil {
				if result, ok := ord["result"].(map[string]interface{}); ok {
					if info, ok := result[txid].(map[string]interface{}); ok {
						if volExec, ok := info["vol_exec"].(string); ok {
							if v, err := strconv.ParseFloat(volExec, 64); err == nil && v > 0 {
								filledVolume = v
							}
						}
						if priceStr, ok := info["price"].(string); ok {
							if p, err := strconv.ParseFloat(priceStr, 64); err == nil && p > 0 {
								buyPrice = p
							}
						}
						if filledVolume > 0 {
							break
						}
					}
				}
			}
			time.Sleep(2 * time.Second)
		}
		if filledVolume == 0 {
			return 0, fmt.Errorf("no fill for %s in 30s", txid)
		}

		// Exit after short hold (e.g., 20s) at market
		time.Sleep(20 * time.Second)
		exitTx, err := te.placeMarketExit(pair, filledVolume)
		if err != nil {
			return 0, fmt.Errorf("exit failed: %v", err)
		}

		// Poll exit to get price
		sellPrice := buyPrice
		start = time.Now()
		for time.Since(start) < 30*time.Second {
			ord, err := te.getOrder(exitTx)
			if err == nil {
				if result, ok := ord["result"].(map[string]interface{}); ok {
					if info, ok := result[exitTx].(map[string]interface{}); ok {
						if priceStr, ok := info["price"].(string); ok {
							if p, err := strconv.ParseFloat(priceStr, 64); err == nil && p > 0 {
								sellPrice = p
							}
						}
						break
					}
				}
			}
			time.Sleep(2 * time.Second)
		}

		// Compute PnL in USD
		pnl := (sellPrice - buyPrice) * filledVolume
		pnlCents := int64(pnl * 100)
		atomic.AddInt64(&te.Capital, pnlCents)
		atomic.AddInt64(&te.TotalPnL, pnlCents)
		atomic.AddInt64(&te.TotalStrikes, 1)
		// Update peak capital in live mode
		currentCapitalInt := atomic.LoadInt64(&te.Capital)
		peakCapital := atomic.LoadInt64(&te.PeakCapital)
		if currentCapitalInt > peakCapital {
			atomic.StoreInt64(&te.PeakCapital, currentCapitalInt)
		}
		if pnl >= 0 {
			atomic.AddInt64(&te.SuccessfulStrikes, 1)
			atomic.StoreInt64(&te.ConsecutiveMisses, 0)
			strike.Status = Hit
		} else {
			atomic.AddInt64(&te.FailedStrikes, 1)
			atomic.AddInt64(&te.ConsecutiveMisses, 1)
			strike.Status = Miss
		}
		strike.PnL = &pnl
		log.Printf("LIVE EXIT: %s filled=%.8f buy=%.2f sell=%.2f PnL=$%.2f (buyTx=%s, sellTx=%s)", pair, filledVolume, buyPrice, sellPrice, pnl, txid, exitTx)
		return pnl, nil
	}

	// Simulated backtest mode retained for offline runs
	priceMovement := (rand.Float64() - 0.5) * 0.04 // ±2% movement (noise only)
	finalPrice := strike.EntryPrice * (1.0 + priceMovement)

	// Determine hit/miss based on confidence
	hitProbability := strike.Confidence
	isHit := rand.Float64() < hitProbability

	// Calculate PnL with TP/SL and fees
	var pnl float64
	fees := strikeSize * RoundTripFeePct
	if isHit {
		// Use realistic TP in SIM_MODE, else strategy expectedReturn
		tp := strike.ExpectedReturn
		if os.Getenv("SIM_MODE") == "1" { tp = SimTakeProfitPct }
		gross := strikeSize * tp * float64(strike.Leverage)
		pnl = gross - fees
		if finalPrice > strike.EntryPrice {
			pnl += strikeSize * 0.0002 * float64(strike.Leverage) // tiny bonus
		}
	} else {
		// Use realistic SL in SIM_MODE
		sl := SimStopLossPct
		grossLoss := strikeSize * sl * float64(strike.Leverage)
		pnl = -grossLoss - fees
	}

	// Update metrics
	atomic.AddInt64(&te.TotalStrikes, 1)
	if isHit {
		atomic.AddInt64(&te.SuccessfulStrikes, 1)
		atomic.StoreInt64(&te.ConsecutiveMisses, 0)
		strike.Status = Hit
	} else {
		atomic.AddInt64(&te.FailedStrikes, 1)
		atomic.AddInt64(&te.ConsecutiveMisses, 1)
		strike.Status = Miss
	}

	// Update capital
	pnlCents := int64(pnl * 100)
	atomic.AddInt64(&te.Capital, pnlCents)
	atomic.AddInt64(&te.TotalPnL, pnlCents)

	// Update peak capital
	currentCapitalInt := atomic.LoadInt64(&te.Capital)
	peakCapital := atomic.LoadInt64(&te.PeakCapital)
	if currentCapitalInt > peakCapital {
		atomic.StoreInt64(&te.PeakCapital, currentCapitalInt)
	}

	// Set exit price and PnL
	strike.ExitPrice = &finalPrice
	strike.PnL = &pnl
	now := time.Now().Unix()
	strike.HitTime = &now

	return pnl, nil
}

// CheckEmergencyStops checks if emergency stops should be triggered
func (te *TradingEngine) CheckEmergencyStops() bool {
	currentCapital := atomic.LoadInt64(&te.Capital)
	peakCapital := atomic.LoadInt64(&te.PeakCapital)
	consecutiveMisses := atomic.LoadInt64(&te.ConsecutiveMisses)

	// Check emergency stop (15% drawdown from peak)
	if currentCapital < peakCapital*85/100 {
		log.Printf("🚨 EMERGENCY STOP: Capital dropped 15%% from peak")
		return true
	}
	// Configurable max drawdown
	if te.MaxDrawdownPct > 0 {
		threshold := int64(float64(peakCapital) * (1.0 - te.MaxDrawdownPct/100.0))
		if currentCapital < threshold {
			log.Printf("🚨 EMERGENCY STOP: Configured drawdown hit: %.2f%%", te.MaxDrawdownPct)
			return true
		}
	}

	// Check consecutive misses
	if consecutiveMisses >= te.MaxConsecutiveMisses {
		log.Printf("🚨 EMERGENCY STOP: Too many consecutive misses: %d", consecutiveMisses)
		return true
	}

	return false
}

// ExecuteCampaign runs the full trading campaign
func (te *TradingEngine) ExecuteCampaign() error {
	log.Printf("🎯 MACRO STRIKE CAMPAIGN INITIATED - %d TRADES", TotalTrades)
	log.Printf("Target: $%.2f in 5 days", float64(te.TargetCapital)/100.0)
	log.Printf("Total Trades: %d", TotalTrades)
	log.Printf("Strike Force: %.1f%% per strike", StrikeForce*100.0)

	startTime := time.Now()
	isSim := os.Getenv("SIM_MODE") == "1"

	for atomic.LoadInt64(&te.TradesCompleted) < TotalTrades {
		// Campaign stop: time window (skip in simulation)
		if !isSim && time.Since(te.CampaignStart) > time.Duration(te.CampaignDays)*24*time.Hour {
			log.Printf("⏱️ Campaign window ended: %d days", te.CampaignDays)
			break
		}
		// Campaign stop: target capital reached (skip in simulation)
		if !isSim && atomic.LoadInt64(&te.Capital) >= te.TargetCapital {
			log.Printf("🎉 Target capital reached: $%.2f", float64(te.TargetCapital)/100.0)
			break
		}

		// Generate and execute strike (skip low-quality setups quietly)
		strike, err := te.GenerateStrike()
		if err != nil {
			if strings.HasPrefix(err.Error(), "skip:") {
				// Try next setup without logging noise
				time.Sleep(time.Duration(StrikeCooldownMs) * time.Millisecond)
				continue
			}
			log.Printf("Error generating strike: %v", err)
			continue
		}

		pnl, err := te.ExecuteStrike(strike)
		if err != nil {
			log.Printf("Error executing strike: %v", err)
			continue
		}

		atomic.AddInt64(&te.TradesCompleted, 1)

		// Log strike result
		currentCapital := float64(atomic.LoadInt64(&te.Capital)) / 100.0
		if strike.Status == Hit {
			log.Printf("✅ HIT: %s | PnL=$%.2f | Capital=$%.2f | Trades: %d/%d",
				strike.Symbol, pnl, currentCapital, atomic.LoadInt64(&te.TradesCompleted), TotalTrades)
		} else {
			log.Printf("❌ MISS: %s | PnL=$%.2f | Capital=$%.2f | Trades: %d/%d",
				strike.Symbol, pnl, currentCapital, atomic.LoadInt64(&te.TradesCompleted), TotalTrades)
		}

		// Check emergency stops
		if te.CheckEmergencyStops() {
			break
		}

		// Progress logging every 100 trades
		if atomic.LoadInt64(&te.TradesCompleted)%100 == 0 {
			progress := (currentCapital - float64(InitialCapital)/100.0) / (float64(InitialCapital) / 100.0)
			elapsed := time.Since(startTime).Seconds()
			tradesPerSecond := float64(atomic.LoadInt64(&te.TradesCompleted)) / elapsed

			log.Printf("Progress: %d/%d trades | Capital: $%.2f | Progress: %.1f%% | Rate: %.1f trades/sec",
				atomic.LoadInt64(&te.TradesCompleted), TotalTrades, currentCapital, progress*100.0, tradesPerSecond)
		}

		// Minimal cooldown
		time.Sleep(time.Duration(StrikeCooldownMs) * time.Millisecond)
	}

	// Campaign complete
	finalCapital := float64(atomic.LoadInt64(&te.Capital)) / 100.0
	finalReturn := (finalCapital - float64(InitialCapital)/100.0) / (float64(InitialCapital) / 100.0)
	totalTime := time.Since(startTime)
	tradesCompleted := atomic.LoadInt64(&te.TradesCompleted)

	log.Printf("🏁 CAMPAIGN COMPLETE: %.1f%% return | Trades: %d/%d | Time: %.2fs",
		finalReturn*100.0, tradesCompleted, TotalTrades, totalTime.Seconds())

	return nil
}

// getStrikeTypeName returns the string name for a strike type
func (te *TradingEngine) getStrikeTypeName(strikeType StrikeType) string {
	switch strikeType {
	case MacroArbitrage:
		return "MacroArbitrage"
	case MacroMomentum:
		return "MacroMomentum"
	case MacroVolatility:
		return "MacroVolatility"
	case MacroLiquidity:
		return "MacroLiquidity"
	case MacroFunding:
		return "MacroFunding"
	case MacroFlash:
		return "MacroFlash"
	default:
		return "MacroArbitrage"
	}
}

// getExpectedReturn returns the expected return for a strike type
func (te *TradingEngine) getExpectedReturn(strikeType StrikeType) float64 {
	switch strikeType {
	case MacroArbitrage:
		return 0.005
	case MacroMomentum:
		return 0.022
	case MacroVolatility:
		return 0.032
	case MacroLiquidity:
		return 0.035
	case MacroFunding:
		return 0.042
	case MacroFlash:
		return 0.059
	default:
		return 0.01
	}
}

func main() {
	// Initialize random seed
	rand.Seed(time.Now().UnixNano())

	// Create and run trading engine
	engine := NewTradingEngine()
	if err := engine.ExecuteCampaign(); err != nil {
		log.Fatalf("Campaign failed: %v", err)
	}
}
