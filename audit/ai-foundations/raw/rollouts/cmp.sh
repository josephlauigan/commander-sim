#!/usr/bin/env bash
# cmp.sh A B : the scorecard rows of two realism outputs side by side (sim value and result), and the deck shares
sc() { grep -E '  (PASS|FAIL|reference|no target)$' "$1" | sed -E 's/^  //; s/ {2,}/\t/g'; }
paste <(sc "$1" | cut -f1,2,6) <(sc "$2" | cut -f2,6) | awk -F'\t' '{ f = ($3 != $5) ? "  <-- " $3 " -> " $5 : ""; printf "%-55s %8s %8s  %s%s\n", $1, $2, $4, $5, f }'
grep -E 'targets pass' "$1" "$2" | sed 's/.*\///'
for f in "$1" "$2"; do grep "^Each deck" "$f" | sed -E 's/Each deck.s share of the decisive games it sat in: //; s/ of [0-9]+//g'; done
