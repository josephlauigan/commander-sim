# Handoff: AI foundations step 3, Tier 6 card and mana flow (brief `ai-t6-flow.md`)

Branch `ai-foundations`, worktree `.claude/worktrees/ai-foundations`. Report (not written yet):
`audit/ai-foundations/step3-flow.md`; raw output: `audit/ai-foundations/raw/step3/`.

**Paused 2026-10-27 ~20:25 at JD's request** (WIP commit "WIP: step 3 flow paused"). Part 1 is mostly measured;
Parts 2-3 not started. No play change has been made: every change so far is stats only.

## Done

- Part 1 instrumentation (stats only, `// --- ai-flow`), built and working:
  - `t6stats.rs` flow section: `fl|...` counters per own turn 1-8 for Tier 6 players: draws by source class
    (step / cantrip / engine / wheel / cmdr / other) and by card, cards seen (cumulative and per turn), tutored
    cards (`tut`), hand size and mana at the main phase, spells cast own / others' turns, spells countered, mana
    spent own / others' turns, life paid by card, land drops made / missed (with or without a land in hand);
    engine cycles out (`out|`), tax triggers and paid share (`tax|`), the engine funnel seen/cast/out by turn
    (`ef|`, now also commanders and wheels), and the turn-4 fate of a seen-never-cast engine (`ez|`).
  - `Player::flow_src` (state.rs): the source a draw is credited to; set in `stack.rs` (`trigger_window`,
    `trigger_window_card`, `ability_window`, `ability_window_card`, restored after a queued trigger) and
    `cast.rs::resolve` (restored after). Hooks in `zones.rs` (draw), `mana.rs` (spend_unit), `turn.rs` (turn
    start, main, end), `cast.rs` (Rhystic tax, countered spell), `rules.rs` (Remora tax), `common.rs` (Sentinel
    tax), `tutors.rs` (Ad Nauseam), `decks.rs` (Necropotence), `topdeck.rs` (Sylvan Library), `legends.rs` (Tymna).
  - `sim-cli/src/flow.rs` (new, with a unit test): the report, printed after `--realism` (Tier 6 players only keep
    the counters) and in each `--goldfish` deck report (`GfGame::flow`).
- **Checked stats-only:** the goldfish with the counters equals step 2's `raw/step2/goldfish-on.txt` except the flow
  sections and the time line; the Tier 6 scorecard equals step 2's (seats 29.4/26.6/22.5/21.5, spread 11.3,
  median round 8).
- Raw: `raw/step3/base/goldfish.txt` (2000 games, seed 500000, flow tables for all 7 decks; built before the
  funnel/hand/countered rows). The Tier 6 table runs a-c were overwritten in turn; their numbers are below.
- The step 2 end binary is saved in the session scratchpad as `bin/step2-commander-sim` (may be gone after the
  session: rebuild from fcaf6605 if needed). Run helper: scratchpad `run.sh OUTDIR NAME BIN ARGS` (lost with the
  session too; it just runs the binary with `--jobs 16 --quiet` into NAME.txt/.err and logs to progress.log).

## Found so far (Tier 6 table, 4000 games, loose; per player-turn averages)

- Cards seen by the end of own turn 2/3/4: Dack 8.8/10.0/11.2, Kinnan 8.8/10.2/11.8, Kraum/Tymna 9.1/10.8/12.8,
  Rog/Silas 9.2/10.7/12.4, Rog/Thrasios 9.0/10.3/11.9, Sisay 9.0/10.3/11.6 (base 7 + T = 9/10/11).
- Spells cast on own turn: about 1.0-1.3 on turns 1-8 for every deck (Rog/Thrasios 2.0 on turn 1, Rog/Silas 1.5
  on turn 2); 0.1-0.4 on others' turns. Hand at main phase stays ~5 cards. Mana spent own turn is 55-85% of the
  mana available (e.g. Kraum T3 2.3 of 3.5, Rog/Silas T4 2.4 of 4.0). Countered: ~0.1 spells a cycle.
- **Engine rates while out look right:** Rhystic Study 4.7-5.1 tax triggers a cycle, ~40% paid, 2.75-3.2 cards a
  cycle; Mystic Remora 2.7-3.1 triggers, ~7% paid, 2.6-2.9 cards; Esper Sentinel 2.1-2.9 triggers, ~33% paid,
  1.5-2 cards; Necropotence ~4 cards a cycle; The One Ring ~2. Kraum 1.2 cards a cycle out; Tymna 0.93.
- **Engines are out few cycles:** Rhystic 0.14-0.30 cycles a game over turns 2-8 (my rough hand estimate for a
  pilot who casts it when able: ~0.55). Funnel, % of turns ended, seen/cast/out: Rhystic at T4 Kinnan 11/7/6,
  Kraum 14/8/7, Rog/Silas 11/8/7, Rog/Thrasios 11/6/5, **Sisay 11/4/3**; at T8 25-40% of seen Rhystics were never
  cast. Remora T4 ~12/9/6 (prio 0 after turn 4 by design). Countering engines is rare (~0.02-0.05 a game).
- **Tymna out only 1.46 turn-starts a game (of ~5.9 reached)** for Kraum/Tymna, Kraum 0.92; Thrasios 3.09 for
  Rog/Thrasios (0.42 cards a cycle). Why Tymna is out so little (cast late or removed) is the open question the
  stopped run d (commander funnel) was to answer.
- Goldfish (raw/step3/base/goldfish.txt): about 1 spell a turn too; Rhystic/Remora/Sentinel blank (inert seats).
- Land drops: T3 missed in 22-37% of turns, nearly always with no land in hand (lists run 23-28 lands).

## In the middle of

- Run d of `--realism --pool t6 --games 4000` with the commander/wheel funnel; **stopped**, no output kept. The
  code after it (the `ez|` turn-4 fate and its report line) is written but **not yet built**.

## Next steps

1. Build (`cargo build --release -j 16`, capped, background); then a 1000-game `--realism --pool t6` for the
   diagnostics (funnel incl. Tymna/Kraum/Thrasios/wheels, `ez|` fates), then the full 4000-game base run and
   `--goldfish t6,scion --games 2000 --seed 500000`, into `raw/step3/base/`.
2. Write the hand reference (Part 1): 7 + T base draws, plus per engine P(out by turn) x cards a cycle, with the
   assumptions stated (opponents ~2.5 spells a cycle each, ~40-60% Rhystic tax paid, Remora tax ~never paid early,
   engines cast turn 1-3 when in hand, Tymna cast turn 2-3 and hitting 1-1.5 opponents a turn). First estimate:
   Kraum/Tymna ~14.5 cards by T4 vs 12.8 sim; Kinnan ~12.5 vs 11.8.
3. Part 2 candidates, largest first (to confirm with traces: `--calibrate within --pool t6 --trace N`):
   (b) Tymna out few turns; (b) seen engines not cast (Sisay worst; check the `ez|` fates: pitched to Force,
   discarded, held behind other plays or hold_up); (c) opponents cast ~1.1 spells a turn, which starves Kraum's
   second-spell trigger and the tax engines. Check every card's Oracle text in `data/scryfall_cache.json` first.
4. Part 3 as the brief says: one commit per fix, each followed by t6 realism 4000 + goldfish; then the full set
   (t0-t6 realism, all-decks 1000) against the step 2 binary.

## Gotchas

- Perl one-liners with `|` delimiters and `\|` in the pattern silently misbehave (twice this session); sed with `|`
  delimiters breaks on `fl|` keys. Use the Edit tool.
- A new run overwrites NAME.txt: give each run its own name.
- One sim or cargo command at a time, `--jobs 16` / `-j 16`; Tier 6 4000 games take ~15 min (detached unit).
