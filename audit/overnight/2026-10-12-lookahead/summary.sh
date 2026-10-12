#!/bin/bash
# Pools the complete rounds (all 50 cells present) of the 2026-10-12 run.
# `summary.sh brief`: one line of deck averages. `summary.sh`: markdown table, per tier and average, with 95% half-widths.
cd "$(dirname "$0")"
DECKS="seph veyran sauron galadriel yshtola alela jodah kefka titania sheoldred"
rounds=""
for R in r*/; do
  R=${R%/}; ok=1
  for d in $DECKS; do for t in t1 t2 t3 t4 t5; do grep -qs "^Win rate" $R/$d-$t.txt || ok=0; done; done
  [ $ok = 1 ] && rounds="$rounds $R"
done
n=$(echo $rounds | wc -w)
[ $n = 0 ] && { echo "no complete round yet"; exit 0; }
for d in $DECKS; do
  for t in t1 t2 t3 t4 t5; do
    for R in $rounds; do echo "$d $t $(grep -m1 '^Win rate' $R/$d-$t.txt | awk '{print $3}' | tr -d %)"; done
  done
done | awk -v n=$n -v mode="$1" '
  { s[$1" "$2] += $3; if (!($1 in seen)) { seen[$1]=1; order[++k]=$1 } }
  END {
    g = 960 * n
    if (mode == "brief") {
      line = n " rounds (" g "/cell):"
      for (i = 1; i <= k; i++) { d = order[i]; a = 0; for (j = 1; j <= 5; j++) a += s[d" t"j] / n; line = line " " d " " sprintf("%.1f", a / 5) }
      print line; exit
    }
    print "| Deck | T1 | T2 | T3 | T4 | T5 | Average | ±95% (avg) |"
    print "|---|---|---|---|---|---|---|---|"
    for (i = 1; i <= k; i++) {
      d = order[i]; row = "| " d; a = 0; v = 0
      for (j = 1; j <= 5; j++) { p = s[d" t"j] / n; a += p; v += (p/100) * (1 - p/100) / g; row = row " | " sprintf("%.1f", p) }
      row = row " | **" sprintf("%.1f", a / 5) "** | " sprintf("%.2f", 196 * sqrt(v) / 5) " |"
      print row
    }
    print ""; print n " complete rounds, " g " games per cell (seeds 500000-" 500000 + g - 1 ")."
  }'
