# one line per deck from a realism / goldfish report: cards seen T2/T3/T4/T6, mana spent T2-T4, and the engine funnel at T4
/^--- Card and mana flow: / { d = $0; sub(/^--- Card and mana flow: /, "", d); sub(/ \(.*/, "", d); deck = d; order[++n] = d }
deck != "" && /cards seen by the turn's end/ { seen[deck] = $(NF-6) "/" $(NF-5) "/" $(NF-4) "/" $(NF-2) }
deck != "" && /mana spent, own turn/ { spent[deck] = $(NF-6) "/" $(NF-5) "/" $(NF-4) }
deck != "" && /^    (Tymna the Weaver|Rhystic Study|Mystic Remora|Kraum, Ludevic's Opus|Thrasios, Triton Hero|Esper Sentinel) +T2/ {
  name = $0; sub(/^ +/, "", name); sub(/ +T2 .*/, "", name)
  for (i = 1; i <= NF; i++) if ($i == "T4") f = $(i+1)
  sub(/^Tymna the Weaver$/, "Tymna", name); sub(/^Rhystic Study$/, "Rhystic", name); sub(/^Mystic Remora$/, "Remora", name)
  sub(/^Kraum, Ludevic's Opus$/, "Kraum", name); sub(/^Thrasios, Triton Hero$/, "Thrasios", name); sub(/^Esper Sentinel$/, "Sentinel", name)
  fun[deck] = fun[deck] " " name " " f
}
END { for (i = 1; i <= n; i++) { d = order[i]; printf "%-18s seen T2/3/4/6 %-22s spent T2/3/4 %-16s T4 seen/cast/out:%s\n", d, seen[d], spent[d], fun[d] } }
