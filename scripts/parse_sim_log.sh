#!/usr/bin/env bash
set -euo pipefail
LOG_FILE=${1:-sim2500.log}
OUT_DIR=${2:-data}
mkdir -p "$OUT_DIR"
PER_TRADE="$OUT_DIR/sim_per_trade.csv"
SUMMARY="$OUT_DIR/sim_summary_by_symbol.csv"

printf "timestamp,status,symbol,pnl_usd,capital_usd,trade_index,total_trades\n" > "$PER_TRADE"
awk '
  /HIT:|MISS:/ {
    ts = $1" "$2
    status = ($0 ~ /HIT:/) ? "HIT" : "MISS"
    line = $0
    sub(/.*(HIT:|MISS: )[ ]*/, "", line)
    n = split(line, parts, " \| ")
    sym = parts[1]
    match(parts[2], /\$(-?[0-9.]+)/, p); pnl = (p[1] ? p[1] : 0)
    match(parts[3], /\$([0-9.]+)/, c); cap = (c[1] ? c[1] : 0)
    match(parts[4], /Trades: ([0-9]+)\/[0-9]+/, t); ti = (t[1] ? t[1] : 0)
    match(parts[4], /Trades: [0-9]+\/([0-9]+)/, tt); tot = (tt[1] ? tt[1] : 0)
    printf "%s,%s,%s,%.2f,%.2f,%d,%d\n", ts, status, sym, pnl+0, cap+0, ti, tot
  }
' "$LOG_FILE" >> "$PER_TRADE"

awk -F, 'NR>1 {sym=$3; pnl=$4+0; status=$2; cnt[sym]++; sum[sym]+=pnl; if(!(sym in max) || pnl>max[sym]) max[sym]=pnl; if(!(sym in min) || pnl<min[sym]) min[sym]=pnl; if(status=="HIT") hits[sym]++; else misses[sym]++} \
  END { \
    print "symbol,hits,misses,total,win_rate_pct,avg_pnl_usd,sum_pnl_usd,max_pnl_usd,min_pnl_usd"; \
    for(sym in cnt){ \
      h=(sym in hits)?hits[sym]:0; m=(sym in misses)?misses[sym]:0; tot=cnt[sym]; wr=(tot>0)?(100.0*h/tot):0; avg=(tot>0)?(sum[sym]/tot):0; \
      printf "%s,%d,%d,%d,%.2f,%.2f,%.2f,%.2f,%.2f\n", sym,h,m,tot,wr,avg,sum[sym],max[sym],min[sym]; \
    } \
  }' "$PER_TRADE" | sort > "$SUMMARY"

echo "Wrote: $PER_TRADE"
echo "Wrote: $SUMMARY"
