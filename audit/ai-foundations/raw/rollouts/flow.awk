# Tier 6 flow: per deck, cards seen by T4 / T6, mana spent T2 / T3 / T4, Rhystic Study and Tymna seen/cast lines
/^--- Card and mana flow:/ { d = $6; if ($7 !~ /^\(/) d = d " " $7 }
/cards seen by the turn's end/ { s[d] = $11 " / " $13 }
/mana spent, own turn/ { m[d] = $6 " / " $7 " / " $8; ord[++n] = d }
/^    Rhystic Study  / { r[d] = $7 " " $8 }
/^    Tymna the Weaver / { t[d] = $8 " " $9 }
END { for (i = 1; i <= n; i++) { d = ord[i]; printf "%-18s seen T4/T6 %s | mana T2/T3/T4 %s | Rhystic %s | Tymna %s\n", d, s[d], m[d], r[d], t[d] } }
