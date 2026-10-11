# Report: zada-cards (branch models-zada-cards, two batches)

All cards of the brief are **Full** in `cardgen audit --mine` (the 12 remaining zada rows are zada-core's cards).
Tests: `rust/crates/sim-core/tests/cards_zada_cards.rs`, 35 tests (one or more per card). Code:
`rust/crates/sim-core/src/impls/zada_cards.rs`; status and notes per card: `data/card_overrides.toml`, section "zada-cards".

## Checks (final commit)
- `cargo test --release`: all pass (two full runs). `crates/practice/tests/http_play.rs::test_review_and_try_it` is flaky
  (1 failure in about 6 runs: the first stream event is "images" instead of "reset"), unrelated to this work.
- `cargo build --release --all-targets`: no warnings.
- `all_decks adaptive 3` (135 games) and `all_decks lookahead 1` (45 games): no panic.
- `commander-sim --deck zada --pool t3 --profile loose --games 48 --ai lookahead --jobs 8 --brief`: finishes, 0% wins
  (Zada's own ability is zada-core's, not merged here).
- I ran `python -` once by mistake (empty stdin, no effect); nothing else of Python.

## Mechanics built
- **prepared** (batch 1): rows in `mine.rs` PREPARED + `prepared_effect`/`prepared_value`, tag `prepare`; the spell halves
  (Craft with Pride, Awaken the Ages) are rows in `data/carddb.txt`. Blazing Firesinger, Goblin Glasswright, Strife Scholar.
- **intimidate** (batch 1): `rules::evasion_blocked`. Ward-pay-2-life: `rules::removal_taxes`.
- **plot** (Irascible Wolverine): `hand_options` offers "plot" in the second main phase when {2}{R} is spare and "cast free"
  on a later turn. A plotted card stays in the hand, marked in `impulse_long` as `u32::MAX - 1 - turn`; its `prio` is 0.
- **Sagas** (Burn, Burn, Tree and Fern; Ral and the Implicit Maze): `etb` sets lore 1 and runs chapter I, a `main1` hook adds
  a lore counter and runs the next chapter, sacrifice after the last (the Fable pattern).
- **Gy upkeep trigger** (Command the Stage): new slot `gy_upkeep` fired for every graveyard at each upkeep.

## Cards (batch 2; batch 1 is in the handoff and the override notes)
Burn Burn (6 damage to the best killable creature, destroy an artifact permanent, {R} twice; Treasure/Food/Clue are
counters and can't be targeted). Ral (2 damage kills what it kills; creatures it doesn't kill are not marked; planeswalkers
lose loyalty; Weird token grows through `on_cast`). Command the Stage (Cadet name not tracked). Glimpse the Impossible
(`Player.glimpse_exiled`, resolved at the end step). Living History (`Player.gy_left`). Lightning (cards kept in hand, marked
`u32::MAX`). Spawn-Gang Commander (devoid via `colors_of`; {C} from a colourless source, sacrifices a Spawn). Relic of
Legends (`extra_mana`; legend offered only when summoning sick, after combat or on other turns, so attackers aren't tapped).
Bonders' Enclave (end of turn only). Generator Servant (`dyn_mana_perm` 2, `on_tap_perm` sacrifices, `haste_next`).
Reiterating Bolt: replicate never paid (no energy in the deck). Violent Echoes: Jace empower only with the Alela Jace.

## Shared-file edits (all small, additive)
`state.rs` (DataKey::WeirdToken; Player fields gy_left, nc_dmg_this/last, glimpse_exiled, haste_next), `hooks.rs`
(Event::GyUpkeep, slot gy_upkeep, EVENTS length), `cardcode.rs` (gy_upkeep), `engine/turn.rs` (nc_dmg rotation + gy_upkeep in
upkeep; Glimpse and haste_next cleanup in end_step), `engine/life.rs` (nc_dmg_this), `engine/cast.rs` (gy_left on casting from
the graveyard; Weird counters in on_cast), `engine/zones.rs` (haste_next in enter), `engine/values.rs` (devoid in colors_of),
`impls/mine.rs`, `impls/rules.rs`, `impls/mod.rs` (batch 1), `sim-cli/src/decks.rs` + `args.rs` (zada added to FILES/MINE/DECK_KEYS,
test count 7 to 8: `--deck zada` didn't exist; the Kefka agent will touch the same lines, expect a trivial merge conflict),
`data/carddb.txt`, `data/card_overrides.toml`, `data/cards.json`.

## Open
- Zada's deck AI may override my `prio` values through `ai/decks.rs` (zada-core's).
- `--all-decks` now includes zada (MINE has 8 entries).
