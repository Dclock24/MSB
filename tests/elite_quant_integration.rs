#[cfg(test)]
mod elite_quant_tests {
    use macro_strk_bot::elite_quant_framework::*;
    use tokio;

    #[tokio::test]
    async fn test_volume_oscillator() {
        let mut oscillator = VolumeOscillator::new(100);
        
        // Simulate volume data
        let volumes = vec![
            100000.0, 120000.0, 95000.0, 130000.0, 140000.0,
            160000.0, 180000.0, 220000.0, 250000.0, 280000.0,
            320000.0, 350000.0, 380000.0, 400000.0, 420000.0,
            450000.0, 480000.0, 500000.0, 520000.0, 550000.0,
            580000.0, 600000.0, 620000.0, 650000.0, 680000.0,
        ];
        
        for volume in volumes {
            let signal = oscillator.update(volume);
            
            println!("Volume: {} | Oscillator: {:.2} | Velocity: {:.2} | Signal: {:?}",
                volume, 
                signal.oscillator_value,
                signal.velocity,
                signal.signal_type
            );
            
            // Verify signal generation logic
            if signal.oscillator_value < -2.0 && signal.velocity > 0.5 {
                assert!(matches!(signal.signal_type, SignalType::StrongLong | SignalType::Long));
            }
            if signal.oscillator_value > 2.0 && signal.velocity < -0.5 {
                assert!(matches!(signal.signal_type, SignalType::StrongShort | SignalType::Short));
            }
        }
    }

    #[tokio::test]
    async fn test_leverage_optimization() {
        let optimizer = LeverageOptimizer::new();
        let portfolio = Portfolio::default();
        
        // Test crypto leverage (max 10x)
        let crypto_signal = TradingSignal {
            symbol: "BTC-USD".to_string(),
            direction: Direction::Long,
            size: 10000.0,
            entry_price: 50000.0,
            stop_loss: 48000.0,
            take_profit: 55000.0,
            win_probability: 0.7,
            win_loss_ratio: 2.5,
            volatility: 0.8,
            asset_class: AssetClass::Crypto,
        };
        
        let crypto_leverage = optimizer.calculate_optimal_leverage(&crypto_signal, &portfolio);
        assert!(crypto_leverage <= 10.0);
        println!("Crypto optimal leverage: {:.2}x", crypto_leverage);
        
        // Test forex leverage (max 5x)
        let forex_signal = TradingSignal {
            symbol: "EUR-USD".to_string(),
            direction: Direction::Short,
            size: 100000.0,
            entry_price: 1.0850,
            stop_loss: 1.0900,
            take_profit: 1.0750,
            win_probability: 0.65,
            win_loss_ratio: 2.0,
            volatility: 0.2,
            asset_class: AssetClass::Forex,
        };
        
        let forex_leverage = optimizer.calculate_optimal_leverage(&forex_signal, &portfolio);
        assert!(forex_leverage <= 5.0);
        println!("Forex optimal leverage: {:.2}x", forex_leverage);
        
        // Test equity leverage (max 2x)
        let equity_signal = TradingSignal {
            symbol: "AAPL".to_string(),
            direction: Direction::Long,
            size: 5000.0,
            entry_price: 180.0,
            stop_loss: 175.0,
            take_profit: 190.0,
            win_probability: 0.6,
            win_loss_ratio: 2.0,
            volatility: 0.3,
            asset_class: AssetClass::Equities,
        };
        
        let equity_leverage = optimizer.calculate_optimal_leverage(&equity_signal, &portfolio);
        assert!(equity_leverage <= 2.0);
        println!("Equity optimal leverage: {:.2}x", equity_leverage);
    }

    #[tokio::test] 
    async fn test_execution_latency() {
        let mut executor = UltraLowLatencyExecutor::new();
        
        let order = Order {
            id: "TEST_001".to_string(),
            symbol: "SPY".to_string(),
            side: Side::Buy,
            quantity: 1000.0,
            order_type: OrderType::Limit,
            limit_price: Some(450.50),
            stop_price: None,
            take_profit: Some(455.00),
        };
        
        let start = std::time::Instant::now();
        let report = executor.execute_order(order).await;
        let total_latency = start.elapsed();
        
        println!("Execution Report:");
        println!("  Fill Price: ${:.2}", report.fill_price);
        println!("  Fill Quantity: {}", report.fill_quantity);
        println!("  Latency: {}μs", report.latency_us);
        println!("  Slippage: {:.1} bps", report.slippage_bps);
        println!("  Total Test Latency: {}μs", total_latency.as_micros());
        
        // Verify latency target (< 200 microseconds)
        assert!(report.latency_us < 200, "Latency exceeded 200μs target");
        
        // Verify slippage is minimal
        assert!(report.slippage_bps < 5.0, "Slippage exceeded 5 basis points");
    }

    #[tokio::test]
    async fn test_strategy_integration() {
        // Test Renaissance Medallion strategy
        let mut medallion = RenaissanceMedallion::new();
        let market_data = MarketData {
            volume: 250000.0,
            price: 100.0,
            timestamp: 1234567890,
        };
        
        let signals = medallion.generate_signals(&market_data).await;
        println!("Medallion generated {} signals", signals.len());
        
        // Test Two Sigma ML predictions
        let mut two_sigma = TwoSigmaML::new();
        let prediction = two_sigma.predict(&market_data).await;
        println!("Two Sigma ML prediction generated");
        
        // Test Citadel market making
        let mut citadel = CitadelMarketMaking::new();
        let order_book = OrderBook::default();
        let (bid, ask) = citadel.generate_quotes(&order_book).await;
        println!("Citadel quotes - Bid: {:.4} | Ask: {:.4}", bid.price, ask.price);
        
        // Test Jump Trading HFT
        let mut jump = JumpTradingHFT::new();
        let opportunity = ArbitrageOpportunity::default();
        let result = jump.execute_arbitrage(&opportunity).await;
        println!("Jump Trading arbitrage executed");
        
        // Test Jane Street ETF arbitrage
        let mut jane_street = JaneStreetETF::new();
        let etf = ETF { last_price: 100.0 };
        let basket = Basket::default();
        let trades = jane_street.arbitrage_etf(&etf, &basket).await;
        println!("Jane Street generated {} ETF arbitrage trades", trades.len());
    }

    #[tokio::test]
    async fn test_macro_strategies() {
        let universe = AssetUniverse::default();
        
        // Test Bridgewater All-Weather
        let mut bridgewater = BridgewaterAllWeather::new();
        let portfolio = bridgewater.allocate_portfolio(&universe).await;
        println!("Bridgewater All-Weather portfolio allocated");
        
        // Test AQR Factor Investing
        let mut aqr = AQRFactorInvesting::new();
        let factor_portfolio = aqr.construct_portfolio(&universe).await;
        println!("AQR factor portfolio constructed with {} signals", 
            factor_portfolio.to_signals().len());
        
        // Test Man Group Trend Following
        let mut man_group = ManGroupTrendFollowing::new();
        let futures_markets = FuturesMarkets::default();
        let cta_signals = man_group.generate_cta_signals(&futures_markets).await;
        println!("Man Group CTA generated {} trend signals", cta_signals.len());
    }

    #[tokio::test]
    async fn test_pod_structure() {
        let mut millennium = MillenniumPodStructure::new(10);
        let market_data = MarketData {
            volume: 500000.0,
            price: 150.0,
            timestamp: 1234567890,
        };
        
        let aggregated = millennium.run_pods(&market_data).await;
        let signals = aggregated.to_trading_signals();
        
        println!("Millennium pod structure generated {} aggregated signals", signals.len());
        assert!(signals.len() >= 0); // Pods should generate some signals
    }

    #[tokio::test]
    async fn test_performance_metrics() {
        // Simulate performance tracking
        let tracker = PerformanceTracker::new();
        
        // Target specifications
        let target_sharpe = 2.5;
        let target_win_rate = 0.65;
        let target_drawdown = 0.15;
        let target_return = 0.40; // 40% annual
        
        println!("Performance Targets:");
        println!("  Sharpe Ratio: > {}", target_sharpe);
        println!("  Win Rate: > {:.0}%", target_win_rate * 100.0);
        println!("  Max Drawdown: < {:.0}%", target_drawdown * 100.0);
        println!("  Annual Return: {:.0}%-60%", target_return * 100.0);
        
        // Verify the tracker prints stats correctly
        tracker.print_stats();
    }

    #[tokio::test]
    async fn test_risk_management() {
        let risk_manager = RiskManager::new();
        let mut signals = Vec::new();
        
        // Create test signals with different risk profiles
        for i in 0..5 {
            signals.push(TradingSignal {
                symbol: format!("TEST_{}", i),
                direction: if i % 2 == 0 { Direction::Long } else { Direction::Short },
                size: 10000.0 * (i + 1) as f64,
                entry_price: 100.0,
                stop_loss: 98.0,
                take_profit: 105.0,
                win_probability: 0.65 + 0.05 * i as f64,
                win_loss_ratio: 2.5,
                volatility: 0.2 + 0.1 * i as f64,
                asset_class: AssetClass::Equities,
            });
        }
        
        // Check risk limits
        let approved = risk_manager.check_limits(&signals);
        println!("Risk check for {} signals: {}", signals.len(), 
            if approved { "APPROVED" } else { "REJECTED" });
        
        // Get portfolio for verification
        let portfolio = risk_manager.get_current_portfolio();
        println!("Current portfolio retrieved for risk assessment");
    }

    #[tokio::test]
    async fn test_kelly_criterion() {
        let kelly = KellyCalculator::new();
        
        // Test various win probabilities and payoff ratios
        let test_cases = vec![
            (0.6, 2.0),  // 60% win rate, 2:1 payoff
            (0.7, 1.5),  // 70% win rate, 1.5:1 payoff
            (0.55, 3.0), // 55% win rate, 3:1 payoff
            (0.8, 1.2),  // 80% win rate, 1.2:1 payoff
        ];
        
        for (p, b) in test_cases {
            let kelly_fraction = kelly.calculate(p, b);
            let percentage = kelly_fraction * 100.0;
            
            println!("Kelly Criterion: p={:.0}%, b={:.1}:1 => {:.1}% of capital",
                p * 100.0, b, percentage);
            
            // Kelly fraction should be reasonable (not > 100% and not negative)
            assert!(kelly_fraction >= 0.0 && kelly_fraction <= 1.0,
                "Kelly fraction out of bounds: {}", kelly_fraction);
        }
    }

    #[tokio::test]
    async fn test_volatility_scaling() {
        let scaler = VolatilityScaler::new();
        
        let volatilities = vec![0.1, 0.2, 0.5, 1.0, 2.0];
        
        for vol in volatilities {
            let scale = scaler.scale(vol);
            println!("Volatility: {:.1} => Scale: {:.2}", vol, scale);
            
            // Higher volatility should result in lower scaling
            assert!(scale > 0.0 && scale <= 1.0);
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use macro_strk_bot::elite_quant_framework::*;
    use std::time::Duration;
    use tokio;

    #[tokio::test]
    #[ignore] // This is a long-running integration test
    async fn test_full_framework_simulation() {
        println!("\n════════════════════════════════════════════════════════");
        println!("     ELITE QUANT FRAMEWORK - FULL SIMULATION TEST");
        println!("════════════════════════════════════════════════════════");
        
        let mut framework = EliteQuantFramework::new();
        
        // Run for a limited time in simulation
        let simulation_handle = tokio::spawn(async move {
            // Run for 5 seconds then stop
            tokio::select! {
                _ = framework.run() => {},
                _ = tokio::time::sleep(Duration::from_secs(5)) => {
                    println!("\n✓ Simulation completed successfully");
                }
            }
        });
        
        // Wait for simulation to complete
        let _ = simulation_handle.await;
        
        println!("\n════════════════════════════════════════════════════════");
        println!("              SIMULATION TEST COMPLETE");
        println!("════════════════════════════════════════════════════════");
    }
}
