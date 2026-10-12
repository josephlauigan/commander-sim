#!/bin/bash
# The pool-results.md table for the complete rounds of this run: each cell "win% (95% interval)", bold when the
# interval is above an even share (25%); rows sorted by the all-tier average.
cd "$(dirname "$0")"
DECKS="seph veyran sauron galadriel yshtola alela jodah kefka titania sheoldred"
rounds=""
for R in r*/; do
  R=${R%/}; ok=1
  for d in $DECKS; do for t in t1 t2 t3 t4 t5; do grep -qs "^Win rate" $R/$d-$t.txt || ok=0; done; done
  [ $ok = 1 ] && rounds="$rounds $R"
done
n=$(echo $rounds | wc -w)
echo "| Deck | T1 High B2/Low B3 | T2 Mid B3 | T3 High B3 | T4 Low B4 | T5 High B4 | All tiers |"
echo "|---|---|---|---|---|---|---|"
for d in $DECKS; do
  for t in t1 t2 t3 t4 t5; do
    for R in $rounds; do echo "$d $t $(grep -m1 '^Win rate' $R/$d-$t.txt | awk '{print $3}' | tr -d %)"; done
  done
done | awk -v n=$n '
  BEGIN { split("seph Sephiroth veyran Veyran sauron Sauron galadriel Galadriel yshtola Y'"'"'shtola alela Alela jodah Jodah kefka Kefka titania Titania sheoldred Sheoldred", a, " ")
          for (i = 1; i < 20; i += 2) name[a[i]] = a[i+1] }
  { s[$1" "$2] += $3; seen[$1] = 1 }
  END {
    g = 960 * n
    for (d in seen) {
      row = "| " name[d]; avg = 0; v = 0
      for (j = 1; j <= 5; j++) {
        p = s[d" t"j] / n; avg += p / 5; q = p / 100; h = 196 * sqrt(q * (1 - q) / g); v += q * (1 - q) / g
        cell = sprintf("%.1f%%", p); if (p - h > 25) cell = "**" cell "**"
        row = row " | " cell sprintf(" (%.1f-%.1f)", p - h, p + h)
      }
      ha = 196 * sqrt(v) / 5; cell = sprintf("%.1f%%", avg); if (avg - ha > 25) cell = "**" cell "**"
      printf "%08.3f %s | %s (±%.1f) |\n", avg, row, cell, ha
    }
  }' | sort -r | cut -d' ' -f2-
