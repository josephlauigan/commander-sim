# Report: practice-rings (Rings of Brighthearth and the Basalt loop for the person)

## What it does now
- **Rings of Brighthearth, by the rules, for the person** (`engine/stack.rs`). A human seat that controls Rings (on the
  battlefield, not phased out, not neutered; one ask per Rings) and activates a non-mana ability is asked
  "Rings of Brighthearth: pay {2} to copy <ability>?" between the ability going on the stack and the round of priority.
  Yes and payable (pool first, then tapping sources: `human::mana::pay_tax_ask`, the same payer as taxes): the Rings
  trigger goes on the stack (`trigger_window`, so a Stifle-type answer can stop it), then the copy goes on the stack
  above the ability (`copy_item`, the Camera's machinery with the `copyable` test lifted for human Rings), players
  respond, and a copy that resolves is counted (`trig_copies`). The caller applies the copy's effect first, choosing
  anew (the effect fn picks its target as it runs), then the original. A countered original still lets its copy resolve.
  After a copy the person gets a priority prompt ("The copy of X has resolved; the original is still to resolve",
  `human::copies_priority`) so they can tap Basalt again before the original untaps it: that is what makes each loop
  net {C} (test `rings_copies_basalts_untap_and_each_loop_nets_one`: 5 in, 6 out).
- Everything is gated on `human::is_human`; the AI paths are unchanged (the new `ability_window_rc` / `ability_run`
  fall straight through to `ability_window` for a bot, `window_item(.., rings=false)` for everything else).
- **The loop shortcut** (`human/abilities.rs` `basalt_loop`): Basalt Monolith and Rings each list "start the loop:
  infinite colourless mana" when both are out and {5} is available in all (same test as `combos::power_artifact`:
  `mana::available >= 5`). It pays {5} (pool / artifact mana first, then taps sources), puts an ability on the stack
  (importance 10: Stifle-type answers), then `ai::combo_interrupted` (instant removal on Basalt or Rings in response),
  then asks how much {C} (999 default, 500, 200, 100, 50, 20, 10), adds it to the pool as plain {C} (what Basalt's own
  tap does: not `ManaPool::art`) and leaves Basalt tapped. Logged.
- **Outlets**: Walking Ballista cast X: the X menu is now `choices::amount_choices` (every number to 24, then round
  numbers and the whole pool), used by all Tokx/Xtutor/Xdrain casts; X = 999 makes 499 counters. Ballista ping asks
  the target, then how many counters (N separate activations, 1 damage each, Rings asked once for the whole run:
  `stack::with_rings_auto`). Staff of Domination gets "draw N cards" (untap {1}, draw {5}+{T} through `dsl::activate`,
  Rings asked once). Thrasios (now in `human/karn.rs`, no longer limited to main 2 / other turns and with no
  3-card library floor) has "{4}" and "{4} N times" (the engine's effect, `tymna_cards::thrasios_resolve`, split out
  of `thrasios_go` unchanged for the AI).

## Rings hooked into (human ability paths)
- Compiled abilities: `dsl::activate` (Basalt's untap, Staff, Walking Ballista if compiled, anything with DSL) and
  `dsl::use_loyalty`, automatically via `ability_window_ex` (human only).
- Card-code planeswalkers on the walker framework: `impls/common.rs` `walker_go` (the same path the AI uses; AI unchanged).
- `human/karn.rs`: Grim Monolith untap (now a stack object; it had no window), Steel Hellkite pump, Walking Ballista
  +1/+1 and ping (single and N), Thrasios.
- Staff "draw N" and the Ballista N-ping run `dsl::activate` / `ping_once` per activation.

## Not covered (Rings is not offered; no fake)
- `human/cards.rs` `walker` (Vraska, Ral Zarek, Ral Storm Conduit ... ): effects are `FnOnce` closures chosen before the
  loyalty is paid; copying needs a rework to repeatable effects.
- The other `human/karn.rs` abilities that use plain `ability_window` / `_card` or the AI's `Action::Ability`
  (Tome, Baubles, Disk, Stones, Ratchet Bomb, Lantern, Forge, Overseer, Obelisk, Forgemaster, Batterskull, Atlas,
  Factory/Foundry pumps, the lands: Enclave, Ruin, Fair, War Room, Vault, Urza's Factory) and `human/abilities.rs`
  (Archivist, Assault, Tortured, Passage, Grounds), `human/cards.rs` abilities (Mind Stone, Triskelion, Aetherflux,
  Guildmage ...), equip. Each is mechanical to convert with `stack::ability_run` (the pattern is in `karn.rs`
  `hellkite_pump`), but the sacrifice-cost ones need their effect closure to tolerate the source being gone.
- Fetch lands (cracked by the engine, not a listed ability); Isochron Scepter-like card-code options.
- Many-copy runs (Ballista N, Staff N, Thrasios N) skip the per-activation "original is still to resolve" priority.

## Shared files touched
`engine/stack.rs` (Rings offer, `ability_run`, `ability_window_rc`, `with_rings_auto`), `human/human.rs` (X menu,
`copies_priority`), `human/mana.rs` (`pay_tax_ask`), `human/choices.rs` (`amount_choices`), `dsl/mod.rs` (priority after
copies in `activate` / `use_loyalty`), `impls/common.rs` (`walker_go` through `ability_run`), `impls/tymna_cards.rs`
(`thrasios_resolve` split out). Practice-only: `human/abilities.rs`, `human/karn.rs`. Tests: `tests/play_rings.rs`.

## Python bugs / notes
None (no Python). Note: `thrasios_options` for the person only fired in main 2 or on other turns; replaced by the
`karn.rs` entry (Thrasios is in `karn::PERMS`, so the AI hook option no longer shows twice).
