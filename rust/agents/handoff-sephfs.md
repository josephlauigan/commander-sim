# Handoff: sephfs (agent 1 stopped after ~70 tool calls)

Branch `sephfs`, worktree .claude/worktrees/sephfs. Commits: c4fc267 (registration), 654219c (card code + plan, WIP).
Baseline binary for same-seed checks (registration only, no card code): /tmp/claude-1000/-home-joseph-projects-mtg-sims/
291e1f60-a736-4dbc-bd9b-964e999afb88/scratchpad/base-sim (if gone: `git worktree`/checkout c4fc267 into a scratch dir and build).

## Done
- Deck registered (decks.rs FILES/MINE 14, args.rs, my_decks.toml with the full DFC commander name, fixture, game.rs 84,
  practice catalog tests); `cardgen export` + `validate` OK.
- Card code `impls/sephfs_cards.rs`: Extinction Event, Necromantic Selection, Blood on the Snow, Living Death (resolve hooks),
  Grave Betrayal, Dread Slaver, Rabid Attack, Kaya's Ghostform (aura, host_dies/host_exiled), Pestilence (options + end step),
  Grave Researcher // Reanimate (upkeep + prepared copy). Overrides added to data/card_overrides.toml (all Full).
  `cardgen audit --cards rust/agents/sephfs-cards.list`: every card is Full or Full-auto.
- Plan `impls/sephfs.rs`: sephfs_prio (edicts held until the flip is reachable, outlets/payoffs, Ghostform, Betrayal,
  Massacre Girl), wipe_utility (drain per death, parity, Living Death), rean_targets (any graveyard).
- Shared-file changes so far (all additive, gated to the deck/cards): state.rs (DataKey Ghostform/Rabid/Slaver/Betrayed,
  Game.exiling), common.rs (AuraSpec.host_exiled + aura_fall calls host_dies on exile), zones.rs (exile_perm and die_rest
  return a Ghostform host; die_triggers draws for Rabid Attack), combat.rs (slaver_mark before the two die() calls),
  ai/brain.rs (wipe_options uses sephfs::wipe_utility for key sephfs), ai/decks.rs (deck_prio arm, rean_targets gate),
  impls/mod.rs, practice tests (deck lists), sim-cli decks.rs/args.rs.
- Smoke: 48 games seed 7 t3 lookahead: 37.5% win (noise +-14), runs clean.

## Next, in order
1. tests/cards_sephfs.rs (nothing written yet): one test per new card (the 11 above) + Sephiroth as commander (tests exist in
   cards_seph_triad.rs for: ETB sacrifice+draw, fourth drain flips, emblem outlives him, own death no drain, back-face attack
   sacrifices all; add: flip needs Sephiroth alive, emblem drains once per death, Sephiroth + wipe) + hand-checks of Altar,
   Tower, Viscera Seer, Yahenni, Woe Strider, Massacre Girl, Pestilence, Butcher, Blood Artist, Zulaport, Plaguecrafter,
   Fleshbag, Accursed Marauder, Reanimate / Animate Dead on an opponent's graveyard. Likely bugs to look for: Ghostform on
   the commander (cmd zone flag), Pestilence end-step sacrifice, Grave Betrayal timing (EndStep hook runs per active player).
2. Rabid Attack has no cast path yet (sephfs_prio should return 0 for it; add a special option in
   `brain::special_options` arm "sephfs" -> sephfs::options: cast before a wipe/edict when mana allows).
3. Plan gaps from the brief: stolen/token creatures to Altar/Tower for mana only when the mana is used (check common outlets
   do it), Grave Betrayal before a wipe (prio), Exsanguinate timing, hold Living Death until graveyards favour us (done in
   wipe_utility, verify by test).
4. Stats + report: add `sf_*` counters (sf_wipes, sf_steals, sf_betrayals, sf_slaver, sf_reanimate, sf_ghostform,
   sf_pestilence, sf_rabid_draws exist; still missing: flips per game and turn of first flip (`seph_flips` stat and the
   milestone exist in sheoldred.rs), emblem drains per game, sacrifices (`creatures_sacrificed`), cards drawn by Sephiroth) and a
   "Sephiroth" section in sim-cli/src/report.rs (copy the garland section).
5. Checks: cargo test --release (all), cargo build --release --all-targets no warnings, all_decks adaptive 3 / lookahead 1,
   same-seed (seed 7, 48 games, lookahead, loose) vs base-sim for sauron t3, cloud t5, thrasios t3, garland t3, seph t3 (if one
   differs, find the cause and gate it); 1,000-game `--deck sephfs --pool t3 --analyze --seed 1013` with no card "never cast
   by the sim" (--jobs 16, nice 19, MemoryMax=16G).
6. Write rust/agents/report-sephfs.md, commit, final 15-line report.

## Gotchas
- Never run Python (not even --version): the hook blocks it.
- `cardgen deck` only records the fixture when the list changes; it was done once (fixture has sephfs).
- Damage that does not kill is not tracked by the engine: Pestilence applies its activations in one action.
