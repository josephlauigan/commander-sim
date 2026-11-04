# Report: cloud-gaps (Judgment of Alexander, Forge Anew)

## Modelled
**Judgment of Alexander** (`impls/cloud.rs`, "Judgment of Alexander"). A per-turn flag on the caster (`flag_turn`, as
Ultimate Magic: Holy's shield). `judgment_shield(g, p, src)` is true for damage from another player's source; it is read by
`life::prevents_damage` (combat, burn and every damage to the player, so reuse of the existing prevention path) and
`values::shielded` (the attack and burn AIs look elsewhere). Strike back: `combat::resolve_blocks` records each attacker
whose damage the shield prevented (twice for an unblocked double striker) and, after the damage loop, calls
`cloud::judgment_strike`: per creature a `trigger_window` (stack), then every commander creature's power summed into one
damage to it (kills when it reaches the toughness, like `partials::deal_noncombat`). Nothing without a commander creature.
AI: `hand_defend` after blockers casts it when the unblocked damage is lethal (or commander-damage lethal), heavy
(>= max(6, 40% of life)), or the commanders' power kills attackers worth >= 5 `pval` with >= 3 damage coming;
`judgment_respond` (from `stack::ai_respond`) casts it in response to a burn spell aimed at the face of 6+ damage, lethal,
or 4+ and half their life. `prio` is 0 (never cast in the main phase).

**Forge Anew.** The enter trigger was NOT right: the compiled `reanimate` had an empty filter and `card_matches` ignores
`subtype`, so it returned the best card of any type from the graveyard. Replaced by card code (`forge_etb`: the best
Equipment, `trigger_window`, Soulless Jailer check), override `dsl = []`. Free first equip: `cloud::equip_cost` returns 0 while
`forge_free` (Forge out, own turn, no equip activated yet); `equip_cost_full` is the old function. `note_equip` marks the
first activation (called where equips pay: `dsl::equip`, `stack::equip_to`, practice mode). The main-phase AI's equip option
adds `0.3 * full cost` while the free equip is up, so it goes to the dearest equip. Instant speed: `forge_blocks`
(`blocks` event, after blockers) moves Equipment from the battlefield's loose/non-attacking holders onto the best unblocked
attacker: free equips if worth anything, paid ones only if cost <= 2 and large gain. Practice mode: `check_equip` skips the
sorcery-timing check under `forge_instant`. `forge_prio`: 68 with an Equipment in the graveyard, 45 with 2+ Equipment and a
creature out, else 0.

## Shared files touched (all small, additive)
`engine/life.rs` (prevents_damage: one check), `engine/values.rs` (shielded), `engine/combat.rs` (`judged` list, strike back
call), `engine/stack.rs` (ai_respond Judgment hook; `equip_to` note_equip), `engine/cast.rs` (cast_card: a face-targeted spell
gets a priority round when the target holds Judgment, `judgment_watch`; otherwise a zero-importance burn spell never opens
one), `dsl/mod.rs` (`equip` note_equip, `equip_options` free-equip bonus), `human/human.rs` (note_equip), `human/legal.rs`
(`check_equip` timing), `data/card_overrides.toml`, `data/cards.json` (exported).

## Simplifications
- Strike back only for combat damage: noncombat damage to a player carries no source creature (a creature's ping is
  prevented, no strike back). Several commanders' damage is summed into one hit. Damage is not kept marked (kill at toughness).
- A ping aimed at the player is not answered with Judgment (only burn spells, which carry `ctx.face`).
- If Chasm and Judgment both apply, the strike back still fires.
- The free equip is also consumed by the main-phase AI's equips, so the combat-time moves usually pay (cost <= 2) or find it spent.
- Practice mode: equip timing relaxed and cost shown; no new prompts.

## Tests
`tests/cards_cloud_gaps.rs`: 15 tests (shield, caster only, own sources, end of turn, strike back, no commander, AI lethal /
strike-back / trivial / no mana / burn response, Forge cost, turn, combat equip, enter trigger).
