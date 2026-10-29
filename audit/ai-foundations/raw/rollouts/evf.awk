# the fallback per kind of the heuristic's top play (EVF lines, ai-rollouts columns 16-18): decisions; the share the
# margin handed back to the heuristic in the end and on the first six rollouts; the look-ahead's final departures
# from the top play; the paired SE of top minus its best alternative (over the rollouts used); how often the bare and
# the final pick change on a second set of seeds (decided by the same rule); the rollouts per option a decision used
# (all, and those handed back at six) and the share that ran to the cap (CAP, default 24)
BEGIN { FS = "\t"; if (CAP == "") CAP = 24 }
$1 == "EVF" {
  for (pass = 0; pass < 2; pass++) {
    g = pass ? "ALL" : $6
    grp[g] = 1; N[g]++; H[g] += $9; H6[g] += $17; O[g] += 1 - $15; SE[g] += $11; FB[g] += $12; FF[g] += $13
    U[g] += $16; if ($17) { N6[g]++; U6[g] += $16; if ($16 >= CAP) C[g]++ }
  }
}
END {
  printf "%-10s %7s | %7s %7s %8s | %6s | %6s %6s | %6s %6s %6s\n", "top kind", "dec", "handed", "hand@6", "finaloff", "SE", "flipB", "flipF", "roll", "roll@6", "atcap"
  for (g in grp) printf "%-10s %7d | %6.1f%% %6.1f%% %7.1f%% | %6.2f | %5.1f%% %5.1f%% | %6.2f %6.2f %5.1f%%\n", g, N[g], 100*H[g]/N[g], 100*H6[g]/N[g], 100*O[g]/N[g], SE[g]/N[g], 100*FB[g]/N[g], 100*FF[g]/N[g], U[g]/N[g], (N6[g] ? U6[g]/N6[g] : 0), (N6[g] ? 100*C[g]/N6[g] : 0)
}
