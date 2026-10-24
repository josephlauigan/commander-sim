# Report: x-fallback (compiled "X" that silently became 3)

Commit: see `git log` on branch x-fallback (base calib-1014). Tests: `cargo test --release -j 3` all pass (new file
`rust/crates/sim-core/tests/x_fallback.rs`, 10 tests); `cargo build --release -j 3 --all-targets` has no warnings;
`all_decks adaptive 3` and `all_decks lookahead 1` finish.

## Root cause
`cardgen`'s `num_expr` (and `n_` for "x"/"that many") returned the bare amount `"X"` for any counting phrase it did not
know. At run time `dsl::num("X")` reads `ctx.x`; triggers/activations/statics run with `DslCtx::base()` (x = 3) and
`resolve_spell` turns an unchosen X (0) into 3. So every such card used 3.

## The 16 cards
| Card | What ran | Now |
|---|---|---|
| Adelbert Steiner | static +1 (power only) per "X" = +3/+0 | `+1/+1 per Equipment you control` (`type_you:equipment`); the compiler also dropped the toughness half of "+a/+b for each", now `tgh` |
| Ant-Man | hand-coded (t0_captain), `dsl = []` | already right; test exists (cards_t0_captain) |
| Avenge | hand-coded (t0_captain), `dsl = []` | already right; test exists |
| Bone Devourer | **wrong, Zurgo lane, not edited**: enters with no counters (note only); dies: draw X / lose X with X = 3 instead of its +1/+1 counters | report only; still bare X, so the audit now flags it Partial |
| Crypt Rats | **wrong, in no deck**: activated with an empty cost (free) dealing 3 to every creature and player (up to 8 times a turn) | not fixed (no deck holds it; needs an {X}-with-black-mana cost); `dsl = []` would be the stop-gap |
| Deadly Embrace (kefka) | drew 3 | draws per creature that died this turn (`creatures_died_this_turn`, from `g.died_count`) |
| Demon of Fate's Design | +3/+0 | +X/+0 with X = the sacrificed enchantment's mana value (`sacrificed_mv`); the pump parser ignored `where X is` for `pow`/`tgh` |
| Finale of Revelation | hand-coded (t0_stella) spell hook; X chosen by the AI | legitimately X (cost {X}{U}{U}); existing tests |
| Fumigate | gained 3 | gains per creature destroyed (`destroyed_this_way`, `DslCtx.destroyed`) |
| Gandalf's Sanction | 3 damage | damage = instants/sorceries in your graveyard (`gy_instants_sorceries`); "excess damage to the controller" is still not modeled |
| Grand Crescendo | cost is {X}{W}{W}: legitimately X | fine in the engine; X must be chosen by the AI (Zurgo agent). If cast with X = 0 it falls back to 3 |
| Haughty Djinn | power = 3 + 1 (carddb base `*` read as 1) and toughness also +3 (`base0` for "power is equal") | power = instants/sorceries in graveyard; toughness stays 4; override tags `pow=0 tgh=4 fly` |
| Kor Spiritdancer | compiled static (+6 power) on top of the hand-coded `self_pt` (+2/+2 per Aura): was a 6/2 alone | `dsl = []` override; 0/2 alone, +2/+2 per Aura |
| Lock and Load | hand-coded (t0_stella) spell hook, test proves 1 + one per other spell | fine; DSL not used |
| Shimmercreep (kefka) | drained 3 | drains per colour among your permanents (`colors_among_permanents`) |
| Thalisse | **wrong, Zurgo lane, not edited**: X = 3 spirits each end step (should be tokens created this turn) | report only; needs a "tokens created this turn" count; audit now flags it Partial |

## Closing the hole
1. cardgen `num_expr`: names for "creature(s) that died this turn", "destroyed this way", "instant and sorcery cards in
   your graveyard", "colors among permanents you control", "sacrificed ...'s mana value", and Equipment you control.
   `where X is ...` is now substituted into `pow`/`tgh` as well as `n`.
2. cardgen audit: a card whose compiled abilities still contain a bare `"X"` but whose cost/text has no `{X}` is
   `Partial` ("an amount the compiler could not name runs as 3") unless a hand override sets its status. Verdict
   changes in `audit --md` over the pool decks: only Zurgo (Thalisse and Bone Devourer: Full-auto -> Partial).
3. Run time: `dsl::x_fallbacks()` (thread-local count) says how many times a compiled "X" was read with no X chosen
   (`DslCtx::x_default`). `tests/x_fallback.rs` plays whole games of the JD decks and asserts it stays 0, and a second
   test shows the counter fires (Grand Crescendo cast with no X).

## Shared-file changes
`rust/crates/sim-core/src/dsl/mod.rs` (new named amounts, `DslCtx` fields `destroyed`, `sacrificed_mv`, `x_default`, a
thread-local counter and `LAST_SAC_MV`, `tgh` in self_scaling), `rust/crates/cardgen/src/{dsl_parse,audit}.rs`,
`data/card_overrides.toml` (Kor `dsl = []`, Haughty Djinn tags), regenerated `data/cards.json`.

## Same-seed checks
- `--deck sephfs --pool t1 --games 200 --ai adaptive --jobs 3 --seed 7 --brief`: NOT identical (36.5% -> 36.0%), because
  t1 holds Light-Paws, which plays Kor Spiritdancer (the only changed card in t1; it was a 6/2). Proof: with only Kor's
  compiled static restored in `data/cards.json`, the new binary reproduces the old output exactly.
- Kefka (`--pool all` = t2-t4, `--profile loose`, lookahead, seed 7, **250 per cell, not 400** - 400 did not fit the
  time; loose profile only): t2 41.6 -> 40.8, t3 24.8 -> 29.2, t4 34.8 -> 35.6, average 33.7 -> 35.2 (CI about +-6 per
  cell, so within noise; Deadly Embrace now draws only for real deaths, Shimmercreep drains 1-3 not always 3).
- Cloud (t1-t2, same settings): t1 11.6 -> 14.0, t2 12.4 -> 11.2, average 12.0 -> 12.6 (Adelbert is now 2/1 plus
  1/1 per Equipment instead of 5/1; within noise).
- The "before" runs used the old binary with the old `data/` (git HEAD), the "after" runs the new binary with new data.
