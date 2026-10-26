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
9. **Rograkh/Thrasios: PR #145 (against main).** It went from 17.9% to 21.3% of its decisive games, inside the
   band. I checked line B's start states against the rules, matched the scorecards to the commits by time, and ran
   the full suite. The lines the handoff guessed at (This Town Ain't Big Enough, Breach) are not in this list.
   **For you:** the agent ran `cd … && python3 --version` once. The no-Python hook missed it because the command
   didn't *start* with python. Nothing else ran, but the hook has a gap. I did not edit the hook, since that's your
   config.
10. **Next two phases, run in parallel (10 threads each):**
    - `t6-quiet-progress`, stacked on #145: let the lookahead AI see Tier 6's stepped combo lines (the
      `deck_combos` lead). It changes every Tier 6 deck's AI, so it's measured paired and kept on its own branch for
      you to judge.
    - `t3-misses`, based on main: the same line audit for Tier 3 (combo 8.9% vs 16.5%, poison 19% vs 9%).
11. **Tier 6 quiet lines: PR #146 (stacked on #145).** The fix is correct (the lookahead ignored every stepped Tier 6
    line) but neutral: seat 1 went 28.1→28.4%, within noise. I opened it anyway because it's a correctness fix. Merge
    it or drop it, as you prefer. Next lead started: fast mana (Chrome Mox, Mox Diamond) held on turns 1-3, on
    `t6-fast-mana`, stacked on #146.
12. **Tier 3 lines: PR #147 (against main).** Combo/alt went from 11.5% to 16.5%, on target; 3 of 9 targets now
    pass, up from 1. I checked the three new lines against the rules and ran the full suite. Calls I made:
    - I kept Helm + Aurelia uncatalogued (it plays out as combat).
    - I kept Marwyn's Staff line even though it overshoots Marwyn's own Playgroup share: the list really holds it.
    - I accepted widening one practice test from 24 to 48 seeds (the Tymna recast). It's seed-sensitive, not broken.
    
    For you: Tergrid rose to 39.1%, and Atraxa and Tergrid still have no combo wins against Playgroup's 13-25%.
13. **Ertai/Tataru bug: PR #148 (against main), fixed myself.** The cause was general: the trigger probe, which runs
    when anyone holds an ability answer, used up `once_per_turn`. I fixed `once_per_turn` itself rather than Tataru
    alone, so all 47 callers are covered. The full suite passes. I didn't run a scorecard for it, since it's a rules
    fix.
14. **Korvold Skeleton loop: added to #147.** The old catalogued loop ignored Korvold's forced draws; it now
    checks them like the Chatterfang line (two draws a pass). The full suite passes. The Tier 3 scorecard is unchanged
    (combo/alt 16.4%, passes).
15. **Sword of the Animist: added to #147.** Aurelia now casts and equips it. On the rescore, combo/alt still passes.
    The pass count shows 1 of 9, down from 3, because seat 1 and seat 3 moved under a point (inside noise) across
    their ±1-point band edges. **For you:** the seat bands (±1 point) are tighter than 4,000 games can resolve; the
    95% CI is about ±1.4. You may want to widen them or run more games for seat rows. I didn't change the targets.
16. **Tier 6 fast mana: PR #149 (stacked on #146).** Chrome Mox is now cast (held 185→28 card-turns), and the
    held-mana counter is corrected. Neutral on the gate: seat 1 went 28.4→28.2%. So two levers for the Tier 6 seat
    gap (combo reading, fast mana) are ruled out. The gap is probably elsewhere, in interaction or in how real games
    snowball, and I stopped chasing it overnight.
17. **Next: casual-tier speed (`casual-speed`, stacked on #144).** The median win round is 12 vs 9 in t0/t2, and
    11-17% of games pass round 15. One agent traces why and fixes bugs or clearly wrong AI choices, without touching
    JD's decks' play.
