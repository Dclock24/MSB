import csv, re, os
log_file = 'sim2500.log'
out_dir = 'data'
os.makedirs(out_dir, exist_ok=True)
per_trade_path = os.path.join(out_dir, 'sim_per_trade.csv')
summary_path = os.path.join(out_dir, 'sim_summary_by_symbol.csv')
line_re = re.compile(r'^(\d{4}/\d{2}/\d{2} \d{2}:\d{2}:\d{2}) .*?(HIT:|MISS:)\s+([^|]+) \| PnL=\$(-?[0-9.]+) \| Capital=\$([0-9.]+) \| Trades: (\d+)/(\d+)')
rows=[]
with open(log_file,'r') as f:
  for line in f:
    m=line_re.search(line)
    if not m: continue
    ts, status_tag, symbol, pnl, cap, idx, total = m.groups()
    status='HIT' if 'HIT' in status_tag else 'MISS'
    rows.append([ts,status,symbol.strip(),float(pnl),float(cap),int(idx),int(total)])
with open(per_trade_path,'w',newline='') as f:
  w=csv.writer(f); w.writerow(['timestamp','status','symbol','pnl_usd','capital_usd','trade_index','total_trades']); w.writerows(rows)
from collections import defaultdict
cnt=defaultdict(int); sum_p=defaultdict(float); hits=defaultdict(int); miss=defaultdict(int); max_p={}; min_p={}
for _,status,sym,pnl,*_ in rows:
  cnt[sym]+=1; sum_p[sym]+=pnl
  if sym not in max_p or pnl>max_p[sym]: max_p[sym]=pnl
  if sym not in min_p or pnl<min_p[sym]: min_p[sym]=pnl
  if status=='HIT': hits[sym]+=1
  else: miss[sym]+=1
with open(summary_path,'w',newline='') as f:
  w=csv.writer(f); w.writerow(['symbol','hits','misses','total','win_rate_pct','avg_pnl_usd','sum_pnl_usd','max_pnl_usd','min_pnl_usd'])
  for sym in sorted(cnt.keys()):
    tot=cnt[sym]; wr=100.0*hits[sym]/tot if tot else 0.0; avg=sum_p[sym]/tot if tot else 0.0
    w.writerow([sym,hits[sym],miss[sym],tot,round(wr,2),round(avg,2),round(sum_p[sym],2),round(max_p[sym],2),round(min_p[sym],2)])
print(per_trade_path); print(summary_path)
