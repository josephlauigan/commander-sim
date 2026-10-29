# the fallback per kind of the heuristic's top play (EVF lines): decisions, the share the margin handed back to the
# heuristic, the look-ahead's final departures from the top play, the paired SE of top minus its best alternative,
# and how often the bare and the final pick change on a second set of seeds
BEGIN { FS = "\t" }
$1 == "EVF" {
  for (pass = 0; pass < 2; pass++) {
    g = pass ? "ALL" : $6
    grp[g] = 1; N[g]++; H[g] += $9; O[g] += 1 - $15; B[g] += 1 - $14; SE[g] += $11; FB[g] += $12; FF[g] += $13
    if ($11 < 1) lo[g]++
  }
}
END {
  printf "%-10s %7s | %8s %8s %8s | %7s %7s | %7s %7s\n", "top kind", "dec", "handed", "bareoff", "finaloff", "SE", "SE<1", "flipB", "flipF"
  for (g in grp) printf "%-10s %7d | %7.1f%% %7.1f%% %7.1f%% | %7.2f %6.1f%% | %6.1f%% %6.1f%%\n", g, N[g], 100*H[g]/N[g], 100*B[g]/N[g], 100*O[g]/N[g], SE[g]/N[g], 100*lo[g]/N[g], 100*FB[g]/N[g], 100*FF[g]/N[g]
}
