# Report: t6-ai `report` (the Tier 6 measuring tool)

Branch `t6-ai-report`. No AI decision changed: counting only (no RNG use). The t3 lock is byte-identical
(`agents/lock.sh` runs it: it diffs against `agents/perf-lock-t3.txt` as the brief says).

## What was built
1. **Deck names.** The three partner pool decks print both names: `Kraum/Tymna`, `Rograkh/Silas`, `Rograkh/Thrasios`
   (`cardgen::decks::display_name`, written to `data/decks.json` by the exporter; `Sim::name_of` now reads the exported
   name for pool decks, so calibration, traces and the game log agree). Tier 0-5 names are unchanged (no other pool
   deck has partners). Practice lists use their own "A and B" text (`practice/catalog.rs`) and were left alone. No
   test pinned the old pool names.
2. **Counters** (`sim-core/src/t6stats.rs`, new): plain additions to a Tier 6 player's `stats` (flag `Player.t6`, set from
   the deck key by `is_t6_key`: the six pool keys and `godo`). Key formats are documented at the top of the file.
   Hooks (all additive): `resolve_counter` (+ Ertai and Louisoix, which mark `countered_by` themselves), `apply_removal`
   (spell or "ability" + owner's deck), `apply_wipe`, `on_cast` (stax pieces, a name list in `t6stats::STAX`),
   `combos::attempt` / `run_steps` (go, win, stop; the stopping card comes from `g.last_counter`, which nothing else
   reads, now also set by every resolved counter and by `interrupted`'s removal / Tidebinder), `mulligan` (`mull_n`: 0-3;
   the old `mulls` only knew 2 and 3).
3. **Report** (`sim-cli/src/t6report.rs`, new): printed after each Tier 6 balance table by
   `--calibrate within --pool t6`. Per tier: win share by seat (turn-order seat 1-4 of decisive games, i.e. a winner that
   was not the round-cap tiebreak) against the TARGETS-t6 values, `OUT` when outside the sim range, round-cap draw
   rate. Per deck: seated / won (round-cap wins apart), wins by route (combo line name, damage, commander damage,
   poison, other: from the winner's `win|` counter, or for last-player-standing wins from how the last opponent out
   died), average and median win round, times it went for each combo line, how many were stopped and by what card, mulligans
   (hands with 1, 2, 3), and the interaction it cast: counters resolved, removal resolved, stax cast, wipes, each with the
   top cards and whose spell / permanent was hit.
   Only Tier 6 prints it: adding lines to other tiers would break the t3 lock. (The collection runs for every tier
   and costs nothing; widening the print is one `if` in `poolmode::run_calibration`.)
4. **Tests:** `sim-core/tests/t6_stats.rs` (10: the t6 flag, a resolved counter and whose spell, a non-t6 player keeps
   none, removal with spell/ability/own-permanent, stax cast, Ertai as the stopper, removal spell as the stopper, an
   unanswered line is a win, `go` once per turn, mulligan count) and three in `sim-cli/src/t6report.rs` (seat shares
   over decisive games only, a combo win named by its line with counters summed and the printed text, how a damage win
   is told apart).

## Notes for whoever measures with it
- "removal resolved" counts every effect that removes a permanent, ETB triggers included (Dack's Fiend Hunter /
  All-Fates Stalker loops give ~2.7 per game).
- "stopped by (no card named)" is a step that failed without a counter or a removal spell (a cost, ward, lock, or a
  counter path that does not go through `resolve_counter`). Rograkh/Silas has 11 of 18.
- Mulligans are high (35-48% of hands) because the engine's keep rule is only "2-5 lands".

## Tier 6 now (300 games per profile, lookahead, seed default; same as the "first look" in the brief)
No AI change, so this is also the "before":

| Deck | conservative | loose |
| --- | --- | --- |
| Dack Fayden | 27.6% | 37.4% HIGH |
| Kinnan | 26.4% | 23.9% |
| Kraum/Tymna | 32.3% | 33.3% |
| Rograkh/Silas | 29.1% | 27.0% |
| Rograkh/Thrasios | 11.8% LOW | 9.5% LOW |
| Sisay | 23.7% | 19.7% |

Average game 9.4 rounds (conservative) / 9.3 (loose); timeouts 2.0% / 2.7%.

Seats (decisive games): conservative 28.2 / 22.4 / 26.2 / 23.1 (targets 34 / 27 / 23 / 17), loose 26.7 / 25.7 / 25.0 / 22.6:
the seat edge is far too flat (seat 1 low by 6-7 points, seat 4 high by 5-6): games are decided by who assembles
the line, not by tempo. That is a Stage 6 / table-rules matter. (Note Rograkh/Silas was 10-12% for "Rograkh" in the
first look only because the two Rograkh decks shared a name; they are 27-29% and 10-12%.)

## What the report already shows (for the deck agents)
- Dack: 74-85% of wins are the first flip (spec's main line); stopped only 3-5 times in ~50 tries.
- Kraum/Tymna and Rograkh/Silas win 92-95% on Thassa's Oracle + Consultation/Pact; Silas goes 74 times and is stopped 24%.
- Rograkh/Thrasios: 79% of wins are plain damage in round 14, only 6 combo tries in 211 games; its win round is
  the latest of the six. Sisay: 70% damage, 13 tries in 198 games, 0 stopped. Both are the decks that do not play
  their combos (spec: main line should be the most common win).
- Kinnan: 71% Basalt Monolith line, 6% stopped.

Sample output: see `/tmp/claude-1000/t6ai-report/t6-run1.txt` (not kept in the repo); the shape is
`  -- Tier 6 report (300 games, 294 decisive; round-cap draws 2.0%) --` then the seat line, then one block per deck.

## Checks
`cargo test --release --no-fail-fast`: 2534 passed, 0 failed (one earlier run had `practice` `http_play`
`test_undo_...` fail once while the machine was loaded; it passes alone and in the rerun: a timing flake, not this change).
`cargo build --release --all-targets`: no warnings. t3 lock (`agents/lock.sh`): IDENTICAL.
