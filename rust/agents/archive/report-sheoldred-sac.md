# Report: sheoldred-sac (claude-sonnet-5-5)

Commit: see `git log` on branch sheoldred-sac (code commit de8972a plus the report/stat commit after it).

## Done
- **Phyrexian Tower** (bug fixed): data override `c=C` (taps for {C}, was {B}{B} free); the `{T}, sacrifice a creature: {B}{B}`
  mode is a land option (`tower_options`/`tower_go`, `impls/sheoldred.rs`). Offered when a death payoff makes the
  sacrifice worth it (same test as the outlets) or on the AI's own turn when the extra mana puts a spell in hand in
  reach. Never the commander, a payoff creature, Sephiroth, or anything with `sac_worth` >= 2.2. Full.
  **Five pool decks run it (Teysa, Meren, Korvold, Tergrid, Yawgmoth): their mana drops by one per Tower (it was a free
  {B}{B} land; now {C}, and the sacrifice mode is rarely used there).** Pool decks go through `pool::special_options`,
  which calls `land_options`, so they can use the sacrifice mode too.
- **Outlets for main decks** (`ai/brain.rs`): `aristocrat_options` now runs for any main deck holding a free outlet
  (Phyrexian Altar, Carrion Feeder, ...), except the seph deck (own logic). Improvements in `common.rs`:
  fodder excludes the commander, `no_sac` creatures and death-payoff creatures (`is_payoff`); the outlet is chosen
  by `outlet_gain`; a mana outlet's mana counts when it is the AI's turn and a hand spell is one or two mana out of
  reach (`mana_needed_now`); `death_value` includes Sephiroth's drain and a flip bonus (+3 when the next death is the
  fourth drain of the turn, +0.5 at two) via `sheoldred::seph_flip_bonus`.
- **About to die anyway**: `sac_in_response` now also handles "destroy"/"wipe"/damage kinds, but only through an outlet
  that gains something (scry, ping, counter, or mana on own turn with a spell waiting); `protect_response` (ai/decks.rs)
  falls back to it for every main deck except seph. Wipes: `wipe_response` sacrifices everything the wipe will kill
  (by `WipeRule::hits`) through free outlets first (not for seph, not for bounce wipes, not with Marchesa out, whose
  return counts simultaneous deaths).
- **Carnage Altar / Common Crook / Callous Inspector**: checked against Oracle, marked Full (Carnage Altar, Common Crook
  overrides; Callous Inspector was already Full). The compiled Carnage Altar sacrifice fodder (`dsl/mod.rs`
  `pay_ability_cost`) no longer picks the commander, a `no_sac` creature or a payoff creature.
- Sephiroth's own ETB/attack sacrifice no longer eats a free outlet (Carrion Feeder).
- **Stats**: `creatures_sacrificed` (zones::die, cause "sac") and `seph_flips`; the analysis report prints
  "Sacrifices: N creatures per game; Sephiroth flipped in X% of games" when the deck holds Sephiroth.
- Tests: `tests/cards_sheoldred_sac.rs`, 19 tests (Tower mana + sacrifice rules, Altar, Feeder, four sacrifices flip
  Sephiroth, AI loop flips Sephiroth, in response, wipe, Carnage Altar, Crook, Inspector).

## Not done
- A creature dying in combat is not sacrificed in response: the engine has no "about to die in combat" hook.

## Checks
- `cargo test --release -j 4 --no-fail-fast`: all pass except `practice::http_lan::test_a_saved_two_player_game_waits_for_the_friend`,
  a timing flake under load (passes alone).
- Build with `--all-targets`: no warnings. `all_decks adaptive 3` and `all_decks lookahead 1`: no panic.
- Smoke (96 games, t3, loose, lookahead, the four swaps): variant win rate 34.4%; 1.98 creatures sacrificed per game;
  Sephiroth flipped in 5.2% of games.

## Shared-file changes (all additive)
`ai/brain.rs` (outlet condition), `ai/decks.rs` (protect_response wrapper, wipe sacrifice), `cardcode.rs` (two
wrappers), `dsl/mod.rs` (fodder filter), `engine/zones.rs` (stat), `impls/common.rs` (above), `sim-cli/src/report.rs`
(two counters + one line), `data/card_overrides.toml` and `data/cards.json` (Tower, Carnage Altar, Common Crook).

## Lead review fix (pool decks unchanged)
All new aristocrat/dsl/sac_in_response behaviour is gated by `ai::is_main(key)`; pool decks take the old code path
(outs[0], old fodder filter, per = death_value). Verified against the fdbe491 binary on --pool t4: veyran, sauron and
kefka `--games 200 --seed 500000 --ai adaptive --brief` identical; veyran traces 0..40 identical.
