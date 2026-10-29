# unbiased: every decision, top option minus the stop option (both always played out on the same seeds)
BEGIN { FS = "\t"; if (by == "") by = "class" }
$1 == "EVD" {
  id = $2 ":" $3 ":" $4
  if (!(id in seen)) { seen[id] = 1; ids[++nid] = id; cls[id] = $9; stp[id] = $8; dk[id] = $4; trn[id] = $7; chg[id] = $12; chs[id] = ""; ko[id] = 0 }
  ko[id]++
  if ($14 == "top") for (i = 16; i <= 34; i++) T[id, i] = $i
  if ($14 == "stop") { for (i = 16; i <= 34; i++) P[id, i] = $i; hasstop[id] = 1 }
  if ($15 == 1) chs[id] = $14
}
END {
  split("22 18 19 23 24 25 26 27 28 29 30 31 32 33 34", cols, " ")
  for (k = 1; k <= nid; k++) {
    id = ids[k]
    if (!hasstop[id] || !((id, 16) in T)) continue
    h = (P[id, 34] > 0) ? "hold" : "nohold"
    if (by == "class") g = cls[id]
    else if (by == "hold") g = cls[id] "/" h
    else if (by == "step") g = cls[id] "/" stp[id] "/" h
    else if (by == "deck") g = dk[id] "/" cls[id]
    else if (by == "turn") g = cls[id] "/t" (trn[id] > 4 ? "5+" : trn[id])
    N[g]++; grp[g] = 1
    if (chs[id] == "stop") S2[g]++
    if (chs[id] == "top") TT[g]++
    nopt[g] += ko[id]
    d = T[id, 22] - P[id, 22]; sd[g] += d; sd2[g] += d * d
    db = d - (T[id, 34] - P[id, 34]) * P[id, 20] / 6   # without the hold bonus
    if (db > 0) beatnob[g]++
    if (d > 0) beat[g]++
    for (j = 1; j <= 15; j++) D[g, j] += T[id, cols[j]] - P[id, cols[j]]
  }
  printf "%-44s %5s %5s %5s %4s | top - stop, mean over all decisions (raw terms over open playouts)\n", "group", "dec", "top%", "stop%", "opts"
  printf "%-44s %5s %5s %5s %4s | %6s %5s %6s %6s %6s %6s %6s %6s %6s %6s %6s %6s %6s %6s %6s %6s\n", "", "", "", "", "", "score", "sd", "win", "loss", "oeval", "life", "board", "power", "hand", "lands", "treas", "combo", "opp", "hold", "t>s", "t>s-h"
  for (g in grp) {
    n = N[g]; m = sd[g] / n
    printf "%-44s %5d %5.1f %5.1f %4.1f | %6.2f %5.1f %6.3f %6.3f %6.2f %6.2f %6.2f %6.2f %6.2f %6.2f %6.2f %6.2f %6.2f %6.2f %6.2f %6.2f\n", g, n, 100*TT[g]/n, 100*S2[g]/n, nopt[g]/n, m, sqrt(sd2[g]/n - m*m), D[g,2]/n/6, D[g,3]/n/6, D[g,4]/n, D[g,5]/n, D[g,6]/n, D[g,7]/n, D[g,8]/n, D[g,9]/n, D[g,10]/n, D[g,11]/n, D[g,12]/n, -D[g,15]/n, beat[g]/n, beatnob[g]/n
  }
}
