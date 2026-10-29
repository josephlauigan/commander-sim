# per decision: shares in the top and stop lines of open playouts where the top card was cast by the end, and where the
# held card's mana was up at the end of the decider's turn
BEGIN { FS = "\t"; if (by == "") by = "class" }
$1 == "EVD" {
  id = $2 ":" $3 ":" $4
  if (!(id in seen)) { seen[id] = 1; ids[++nid] = id; cls[id] = $9; stp[id] = $8; dk[id] = $4; trn[id] = $7 }
  if ($14 == "top") { To[id] = $20; Tc[id] = $35; Th[id] = $36; Tw[id] = $18; Tl[id] = $19 }
  if ($14 == "stop") { So[id] = $20; Sc[id] = $35; Sh[id] = $36; Hb[id] = $34; hs[id] = 1 }
  if ($15 == 1) chs[id] = $14
}
END {
  for (k = 1; k <= nid; k++) {
    id = ids[k]; if (!hs[id] || !(id in To)) continue
    h = Hb[id] > 0 ? "hold" : "nohold"
    g = (by == "step") ? cls[id] "/" stp[id] "/" h : (by == "deck" ? dk[id] "/" cls[id] : cls[id] "/" h)
    grp[g] = 1; N[g]++
    a[g] += To[id]; b[g] += Tc[id]; c[g] += So[id]; d[g] += Sc[id]; e[g] += Th[id]; f[g] += Sh[id]
    if (chs[id] == "top") nt[g]++
    if (chs[id] == "stop") ns[g]++
  }
  printf "%-40s %6s %6s %6s | %8s %8s | %8s %8s\n", "group", "dec", "top%", "stop%", "cast:top", "cast:stp", "held:top", "held:stp"
  for (g in grp) printf "%-40s %6d %6.1f %6.1f | %8.2f %8.2f | %8.2f %8.2f\n", g, N[g], 100*nt[g]/N[g], 100*ns[g]/N[g], b[g]/(a[g]?a[g]:1), d[g]/(c[g]?c[g]:1), e[g]/(a[g]?a[g]:1), f[g]/(c[g]?c[g]:1)
}
