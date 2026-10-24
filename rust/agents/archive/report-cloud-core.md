# Report: cloud-core

All 22 cards of the brief are Full in `cardgen audit --mine` (the rest of the cloud list is white-rest's). 51 tests in
`rust/crates/sim-core/tests/cards_cloud.rs`; full `cargo test --release -j 6` passes; `--all-targets` builds with no
warnings; `all_decks adaptive 3` and `all_decks lookahead 1` finish; 48-game lookahead smoke of `--deck cloud` finishes.

## How each card is modeled (impls/cloud.rs, verdicts in data/card_overrides.toml)
- **Cloud**: etb hook searches the Equipment `cloud_pick` wants (none out: Wrecking Ball Arm, then Hero's Blade; one
  out: Sword of Kaldra / Mask of Memory; two Kaldra pieces out: the third; Cloud under threat: Shield of Kaldra).
  The doubling is `stack::queue_triggers`: a queued trigger whose source is an equipped Cloud, or an Equipment attached to
  him, is queued twice (`cloud::doubles`). Granted abilities (Conformer Shuriken, Medic's Kitesail, White Mage's Staff)
  are queued with the Equipment as their source, so they double while it is on Cloud. His own enter trigger is not
  doubled (he enters bare).
- **Equip cost** (`cloud::equip_cost`, read by `dsl::equip_options`): Wrecking Ball Arm legend {3}, Hulkbuster Armor
  Hero {3}, Veteran's Powerblade Soldier {W} (paid as {1}: the deck is mono-white), Puresteel Paladin's metalcraft {0},
  Helitrooper -{2}. Cid: `cost` hook (-1 for Equipment/Vehicle spells), `grant_kw` flying on his turn, `options` regrow.
- **Size/evasion**: Wrecking Ball Arm and Hulkbuster Armor set base P/T (`cloud::base_pt` in `dsl::pt`); Colossus Hammer
  +10/+10 and loses flying (`dsl::has_kw`, `combat::can_block`); Wrecking Ball Arm / Trailblazer's Boots in
  `cloud::evasion_blocked`; Magitek Scythe's "must be blocked" is the `must be blocked` eot keyword plus
  `cloud::force_blocks` (the AI's blocks only; the practice-mode human is not forced).
- **Attach cards**: Hero's Blade, Armory Automaton (enter and attack), Beatrix (beginning of combat), Magitek Scythe,
  Squire's Lightblade, White Mage's Staff (Hero token), Magnetic Snuffler (+ counters when you sacrifice an artifact),
  Puresteel Paladin (draw). Attach decisions: Cloud first, then `holder_score`; Equipment already on a better wearer is
  left alone.
- **Others**: Conformer Shuriken, Explorer's Scope (attack hooks on the Equipment), Helitrooper, Jocasta (counter hook;
  the graveyard return is `cloud::jocasta_attack`, called from `combat::attack_triggers_once`), Zack Fair (as-enters
  counter; `zack_sacrifice` is used by `cloud_protect`), Unfinished Business (resolve slot; Equipment and own-creature
  Auras via `g.attach_to`).
- White Mage's Staff's "is a Cleric" is not tracked (no Cleric payoff in the deck; said in the verdict note).

## AI plan
`cloud_prio` (Cloud 130 on turn 2+, so the sampled play is Cloud; Cid and Puresteel first when Equipment is in hand;
Hero's Blade before Cloud only with mana for both; the attach-on-enter Equipment wait for a creature via their `prio`
slot), `cloud_protect` (Shardmage's Rescue, Take Up the Shield, Zack Fair, Cho-Manno's Blessing, then the board-wide
instants; edicts are not answered), `cloud::attack_filter` (generic outside-deck filter, and a bare Cloud stays home when
anything can block him), `holder_bonus` / `equip_utility_bonus` (equip options favour Cloud), `reenter_value`
(blink/etb choices: `common::blink_value` and `mine::etb_value` ask it for Cloud).

## Shared-file edits (all additive)
- `engine/stack.rs`: queue_triggers doubling. `cardcode.rs`: evasion_blocked calls cloud's. `engine/combat.rs`: can_block
  uses `loses_flying`; `force_blocks` after the AI's blocks; `jocasta_attack` in attack_triggers_once.
- `dsl/mod.rs`: `equip_options` (per-target equip cost, holder/utility bonus, re-equip onto Cloud), `pt` (base P/T), `has_kw` (loses flying).
- `ai/mod.rs`: attack_filter routes key cloud to `cloud::attack_filter`. `impls/common.rs` (`blink_value`) and
  `impls/mine.rs` (`etb_value`): Cloud returns `cloud::reenter_value`.

## For the lead
- Cho-Manno's Blessing and Shardmage's Rescue (white-rest's) are cast by `cloud_protect` with `g.attach_to = Cloud`; a
  save counts only if the result is verified (hexproof / protection), so with the Auras unmodeled nothing is falsely saved.
  After merging white-rest, add a test of those two through `cloud_protect`.
- For the Ennis / Forum Familiar / Stormfront Riders / Unyielding Gatekeeper targets: white-rest's blink choices should
  call `cloud::reenter_value(g, p, m)` (> 0 only for Cloud); the generic `blink_value` / `etb_value` already do.
- Win rate of the deck right now is about 2-3% (adaptive, t1/t3, 480 games), partly because the white-rest cards are bodies
  only until their branch merges.
