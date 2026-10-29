# Aggregates EVD lines: for each decision, the top option against the chosen one (changed) or the stop/next-best (kept)
# usage: awk -f evd.awk [-v by=class|step|deck|classstep|chosenkind] file
BEGIN { FS = "\t"; if (by == "") by = "class" }
$1 == "EVD" {
  id = $2 ":" $3 ":" $4
  if (!(id in seen)) { seen[id] = 1; ids[++nid] = id; ch[id] = $12; cls[id] = $9; stp[id] = $8; dk[id] = $4; top[id] = $10; chl[id] = $11; trn[id] = $7 }
  if ($14 == "top") for (i = 16; i <= 34; i++) T[id, i] = $i
  if ($15 == 1) { for (i = 16; i <= 34; i++) C[id, i] = $i; ckind[id] = ($14 == "stop" ? "stop" : ($14 == "top" ? "top" : "other")) }
}
END {
  split("16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34", cols, " ")
  split("util n win loss open fail score openeval life board power hand lands treas combo opp dead ramp hold", nm, " ")
  for (k = 1; k <= nid; k++) {
    id = ids[k]
    if (by == "class") g = cls[id]
    else if (by == "step") g = cls[id] "/" stp[id]
    else if (by == "deck") g = dk[id] "/" cls[id]
    else if (by == "turn") g = cls[id] "/t" (trn[id] > 4 ? "5+" : trn[id])
    else g = cls[id]
    N[g]++
    if (ch[id] == 1 && ((id, 16) in T)) {
      M[g]++; K[g, ckind[id]]++
      for (j = 1; j <= 19; j++) D[g, j] += C[id, cols[j]] - T[id, cols[j]]
      # wins/losses as shares
    }
    grp[g] = 1
  }
  printf "%-34s %6s %6s %5s %5s | mean (chosen - top) over the changed decisions:\n", "group", "decis", "chang", "stop", "other"
  printf "%-34s %6s %6s %5s %5s | %6s %6s %6s %6s %6s %6s %6s %6s %6s %6s %6s %6s %6s %6s\n", "", "", "", "", "", "score", "win", "loss", "oeval", "life", "board", "power", "hand", "lands", "treas", "combo", "opp", "ramp", "hold"
  for (g in grp) {
    m = M[g] ? M[g] : 1
    printf "%-34s %6d %6d %5d %5d | %6.2f %6.2f %6.2f %6.2f %6.2f %6.2f %6.2f %6.2f %6.2f %6.2f %6.2f %6.2f %6.2f %6.2f\n", g, N[g], M[g], K[g, "stop"], K[g, "other"], D[g,7]/m, D[g,3]/m/6, D[g,4]/m/6, D[g,8]/m, D[g,9]/m, D[g,10]/m, D[g,11]/m, D[g,12]/m, D[g,13]/m, D[g,14]/m, D[g,15]/m, D[g,16]/m, D[g,18]/m, D[g,19]/m
  }
}
