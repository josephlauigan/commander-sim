# t6-ai-b report

# t6-ai-b (2026-11-06): follow-ups decided by JD

Branch `t6-ai-b` (from t6-ai bfe3cc3c). Brief: `rust/agents/t6-ai-b.md`. Tests: `rust/crates/sim-core/tests/t6_ai_b.rs`
(9 tests; against the old source 6 fail, the other 3 are a control, a negative and the Dack finding). No sims run.

## NEEDS JD
1. **Dack's and JD's `thrasios` Lion's Eye Diamond have no fuel in their lists.** Both play LED but none of Wheel of Fortune, Windfall, Timetwister
   or Underworld Breach, so no cast-priority row was added for it (the brief's rule would never fire). LED stays a card
   with no use there except `led_uses` (recast the commander / flashback with a lands-only hand). Say if either should lose
   LED or get another rule (`thrasios` is a deck of JD's).
2. **Windfall and LED**: Windfall draws as many cards as the most any player discarded *to it*. The diamond's discard does
   not count, so cracking it first makes Windfall draw only the biggest opposing hand. The crack is therefore offered for
   Windfall only when no opponent holds fewer cards than the rest of our hand (otherwise it would cost draws). Wheel of
   Fortune and Timetwister draw seven whatever happens.
3. **The deck's choice among options is a sample** (softmax), so "crack for the wheel" is offered 0.25 above the plain cast
   of the wheel and is usually, not always, chosen. g32 R2 shows the same effect for a different case (below).
4. **Deafening Silence and the AI's order (new, not fixed)**: in g32 R2 Rograkh/Thrasios held Mana Vault and Wild Growth
   under Dack's Deafening Silence (one noncreature spell each turn) and sampled Wild Growth (19%) over Mana Vault (77%).
   Once the one spell is cast, Mana Vault is "not castable". The AI does not choose the best single noncreature spell under
   that lock; not in this brief, no change made.

## 1. Lion's Eye Diamond (fixed)
- `data/plans/{kraum-tymna-blue-breach-midrange,rograkh-silas-turbo-breach}.toml`: a `led_fuel` set (Kraum: Underworld
  Breach; Rograkh/Silas: Wheel of Fortune, Windfall, Timetwister, Underworld Breach) and a `[[prio]]` row for LED: 90 with
  `hand:led_fuel>=1`, else 0 (held). Dack: see NEEDS JD 1.
- `impls/mine.rs` `led_options`: with LED on the battlefield and a wheel in hand, an option "<wheel> (Lion's Eye Diamond)"
  (`led_wheel_go`): the rest of the hand is discarded, the diamond sacrificed, three mana of the wheel's colour made and the
  wheel cast for real with it (real play: the wheel is on the stack when the diamond is cracked to pay for it; the hand it
  discards would go to the wheel anyway). Offered at the wheel's utility + 0.25, only when the deck wants to cast that wheel
  (`deck_prio_f` > 0: a small or stuck hand, no combo piece in it), the wheel is castable, the cost is {2}{colour} payable
  with the three mana, and (Windfall) see NEEDS JD 2. So the crack never goes into an empty plan. The old uses (lands-only hand)
  are unchanged (`led_uses`). The payment may take lands rather than the pool; the three mana stay available this phase either way.
- `t6stats::makes_mana_now`: LED counts only with a use (`mine::led_use_open`: such a wheel, or a lands-only hand with a use).
**Decks:** LED is in four Tier 6 lists (jq on `data/decks.json`): Kraum/Tymna and Rograkh/Silas (changed), Dack and JD's `thrasios` (no wheel or Breach in either list, so no priority row and no crack option; for `thrasios` only the held statistic changes: LED is no longer counted as mana there). No other deck plays it.

## 2. Instant removal for a combo-piece commander (fixed)
`ai/brain.rs` `hold_value`: for `is_t6` decks the instant removal hold reads `combo_near` alone (as `near` does); others read
`cmdr_combo` as before. **Decks:** `thrasios` (JD, Tier 6) only; its instant removal is no longer held because Kinnan, Thrasios
or Tymna sits on a table. The other eight of JD's decks are unchanged (Sheoldred tested as the control).

## 3. Urza's Saga, Tezzeret, Mana Vault (investigated; no code change)
- **Saga / Tezzeret**: only Dack's Tier 6 list plays them (JD's retired `thanos` list plays the Saga; Urza and Kinnan/Simic are
  not Tier 6 lists). `planner::tutor_rule` returns nothing for permanents on purpose: the planner searches one turn and the Saga
  fetches by itself, in its chapter III trigger two turns after it enters (the AI takes the deck's wish list first,
  `tutor_pick(.., "art")`, restricted to cmc <= 1, non-X, matching "mana cost {0} or {1}"), while Tezzeret is a 4-mana walker
  whose -3 (`t6_godo::tez_three`, `t1::tutor_named`) already takes the planner's card. Neither can fetch a piece of a Dack
  line: the test `dack_has_no_line_piece_that_the_saga_or_tezzeret_could_fetch` lists the artifact pieces (mana value <= 1)
  in Dack's catalogued lines (`combos::audit_lines`): none (the lines are the Preston / Felidar / Solar creatures and the Dack
  flip). As mana tutors they cost more than the artifact gives back within the turn (Tezzeret 4 mana for Sol Ring / Vault),
  so `plan_mana_targets` would never choose them. Nothing to change.
- **Mana Vault in g32** (`--calibrate within --pool t6 --ai adaptive --games 4000 --seed 900000 --trace 32`, R2,
  Rograkh/Thrasios): a legitimate lock. Dack cast Deafening Silence on R1 ("each player can't cast more than one noncreature
  spell each turn"); Rograkh/Thrasios had cast Wild Growth that turn, so Mana Vault and Nature's Rhythm are "not castable"
  (`engine::cast::castable`). Not a bug; the sampled order is NEEDS JD 4.

## Files changed
`data/plans/kraum-tymna-blue-breach-midrange.toml`, `data/plans/rograkh-silas-turbo-breach.toml`, `ai/brain.rs` (hold_value),
`impls/mine.rs` (led_options, led_uses, led_wheel_*, led_use_open), `t6stats.rs` (makes_mana_now), `tests/t6_ai_b.rs`.
rustfmt re-wrapped some unrelated lines in `impls/mine.rs` and `t6stats.rs`.

## Checks run (t6-ai-b)
- `cargo build --release --all-targets`: no warnings.
- `cargo test --release --no-fail-fast` over 75 test files (the brief's list plus ai_eval, ai_xtutor, t6_stats, t6_stops,
  engine_needs*): 1230 passed, 0 failed (1221 before + the 9 new).
- `t6_ai_b.rs` against the t6-ai source and data (bfe3cc3c): 6 fail (plan priority, the offer on turn one, the crack, the
  crack with lands, the statistic, the removal hold); the control, the "no empty plan" negative and the Dack finding pass.
