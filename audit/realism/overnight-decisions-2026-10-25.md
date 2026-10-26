# Overnight decisions, 2026-10-25/26

JD went to bed and said to keep going and note every decision I would otherwise have asked about.
Newest at the bottom.

1. **Tatyova agent's full-thread run.** It started a 4,000-game realism run without `--jobs`, so I told it to kill
   that run and redo it at 2,000 games with `--jobs 4`, which is the cap rule.
2. **Tatyova: no combo line added, merged.** Its real list (EDHREC) has no non-combat win, so Playgroup's 18.7%
   (only 63 games, every build) can't be reached without inventing cards. I accepted the one card fix (Retreat to
   Coralhelm untaps Scaled Herbalist, matching the Oracle text) and merged it into casual-combos. I left Tatyova's
   combo/alt share below its target rather than changing the list, since the briefs said not to change lists.
3. **Sythis: nothing added.** Its list holds no alt-win, mill or infinite (I checked: no Felidar Sovereign,
   Sanctum of All, Walking Ballista or Spike Feeder; Heliod has no loop partner). Its only drain, Blind Obedience's
   extort, is already modelled. It already wins 45.5% of decisive t2 games, so adding wins would make the tier worse.
   I accepted "no change". The t2 combo/alt row (8.2% vs 21.2%) stays failing; the target averages five commanders'
   Playgroup pages, which include other builds.
4. **Brago: merged, with one fix of mine.** The agent fixed the catalogued Drake + Archaeomancer + Ghostly Flicker
   line (it never fired) and added Displacer Kitten + Archaeomancer + Drake/Cloud of Faeries + a flicker spell, both
   finishing with Agent of Treachery. I checked them against the rules. One error: it counted Ghostly Flicker as
   blinking the mana creature twice, but Kitten's trigger resolves first and makes it a new object, so Flicker's
   second target is gone. Cloud of Faeries + Flicker was counted as a loop when it loses mana. I fixed it and added a
   test assertion. It also holds Cloudshift when there's no target (Brago only). t2 combo/alt went from 8.2% to 9.9%.
5. **Teysa: merged.** Two new lines, both checked: Teysa/Drivnod doubling Pitiless Plunderer's Treasures to pay
   for Reassembling Skeleton, and Phyrexian Altar + Skeleton + Pawn of Ulamog/Sifter of Skulls. Pool Teysa also now
   keeps her payoffs out of the sacrifice fodder, as JD's decks already do. I accepted that this rule change
   applies to her deck only. t1 combo/alt went from 1.2% to 2.5%.
6. **Stella: merged.** The precon has no combo or alt win; every non-combat win is a spell-chain burn turn, which is
   correctly counted as damage. The agent fixed two AI gaps: the pool AI never cast Mizzix's Mastery and almost never
   cast Galvanic Iteration. Stella's win share went from 14.4% to 16.8%, still the weakest t0 deck. I left the t0
   combo/alt target alone (Playgroup's commander page pools upgraded Stella lists).
7. **Merged result and PR.** The full suite passes on the merged branch, and the rescore is in
   `casual-combos-2026-10-26.md`: combo/alt went t0 0.6→0.4%, t1 1.2→2.2%, t2 8.2→9.8%, with no target newly
   passing. I kept the win-type rows scored, as you decided. I pushed `casual-combos` and opened a PR stacked on
   #142 (base `tier012`), as with earlier phases; it's yours to merge.
8. **Next phase: Rograkh/Thrasios (handoff item 2).** I started one agent on branch `t6-rograkh-thrasios`, based on
   main, not stacked on #142/#144, since Tier 6 is independent. It models the deck's missing real win lines. It
   runs alone on the machine, so its cap is 16 threads (`--jobs 16`, `-j 16`), one command at a time.
