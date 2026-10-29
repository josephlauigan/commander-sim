# goldfish summary: deck, median / mean kill turn, won by T3, no kill
/^=== / { d = $2 }
/^Kill turn/ { gsub(",", ""); med[d] = $6; mean[d] = $8; order[++n] = d }
/^  won by turn:/ && !(d in t3) { t3[d] = $9 }
/^  no kill by round/ { nk[d] = $6 }
END { for (i = 1; i <= n; i++) { d = order[i]; printf "%-12s %s / %s, %s, %s\n", d, med[d], mean[d], t3[d], nk[d] } }
