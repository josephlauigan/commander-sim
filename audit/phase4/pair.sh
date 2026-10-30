#!/bin/sh
# pair.sh HAND.txt PLAN.txt: each deck's share both ways with 95% CIs (Wald) and the difference's CI
for f in "$1" "$2"; do grep "Each deck's share" "$f" | sed 's/.*sat in: //' | tr ';' '\n' | sed 's/^ *//'; echo "--"; done |
awk 'BEGIN { arm=0 }
/^--$/ { arm++; next }
{ n=split($0,a," "); d=a[n]; pct=a[n-2]; sub("%","",pct); name=""; for(i=1;i<=n-3;i++) name=name (i>1?" ":"") a[i];
  p=pct/100; k=arm; P[k,name]=p; D[k,name]=d; if (k==0) order[++m]=name }
END { printf "%-22s %-22s %-22s %s\n","deck","hand (95% CI)","plan (95% CI)","plan-hand (95% CI)";
  for(j=1;j<=m;j++){ nm=order[j]; p0=P[0,nm]; d0=D[0,nm]; p1=P[1,nm]; d1=D[1,nm];
    e0=1.96*sqrt(p0*(1-p0)/d0); e1=1.96*sqrt(p1*(1-p1)/d1); e=1.96*sqrt(p0*(1-p0)/d0+p1*(1-p1)/d1);
    printf "%-22s %5.1f%% [%4.1f,%4.1f] n=%d  %5.1f%% [%4.1f,%4.1f] n=%d  %+5.1f [%+5.1f,%+5.1f]\n", nm, 100*p0,100*(p0-e0),100*(p0+e0),d0, 100*p1,100*(p1-e1),100*(p1+e1),d1, 100*(p1-p0),100*(p1-p0-e),100*(p1-p0+e) } }'
