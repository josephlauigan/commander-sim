# noise per decision group (EVN lines): the paired top-minus-stop difference and its standard error at 6 rollouts, how
# much the shared seeds (common random numbers) cut the variance, and how often the pick changes on a second seed
BEGIN { FS = "\t"; if (by == "") by = "class" }
$1 == "EVN" {
  g = (by == "step") ? $5 "/" $6 "/" ($7 ? "hold" : "nohold") : (by == "all" ? "all" : $5)
  grp[g] = 1; N[g]++
  amd[g] += ($8 < 0 ? -$8 : $8); se[g] += $9; md[g] += $8
  if (($8 < 0 ? -$8 : $8) < $9) inside[g]++
  vt[g] += $10 * $10 + $11 * $11; vd[g] += $12 * $12
  if ($13 != $16) fb[g]++
  if ($14 != $17) fm[g]++
  if ($13 == $18) tb[g]++
  if ($14 == $18) tm[g]++
  if ($15 == $18) ta[g]++
}
END {
  printf "%-28s %6s | %7s %7s %7s %6s %6s | %6s %6s | %6s %6s %6s\n", "group", "dec", "mean d", "|d|", "SE(d)", "|d|<SE", "CRNcut", "flipB", "flipM", "topB", "topM", "chosen"
  for (g in grp) printf "%-28s %6d | %7.2f %7.2f %7.2f %5.0f%% %5.0f%% | %5.0f%% %5.0f%% | %5.0f%% %5.0f%% %5.0f%%\n", g, N[g], md[g]/N[g], amd[g]/N[g], se[g]/N[g], 100*inside[g]/N[g], 100*(1 - vd[g]/vt[g]), 100*fb[g]/N[g], 100*fm[g]/N[g], 100*tb[g]/N[g], 100*tm[g]/N[g], 100*ta[g]/N[g]
}
