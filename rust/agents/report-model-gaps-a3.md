# Report model-gaps-a3 (branch model-gaps-a3)

Status: items 1-4 done; items 5 (Peter Parker's Camera) and 6 (Marvin) NOT started beyond reading the code (context ran out,
see HANDOFF). `cardgen audit --mine`: sauron 1 not Full (Camera), kefka 1 (Marvin), thanos 1 (Hexing Squelcher, see NEEDS JD).
Tests: `rust/crates/sim-core/tests/model_gaps_a3.rs` (13 pass) plus ~90 existing files that touch the changed code (cards_*,
engine_t6_*, model_gaps_karn/others, practice_*_probe ...): all pass. `cargo build --release --all-targets`: no warnings.
Not run: the full suite and any sim.

## 1. Token lands
`engine::turn::remove_land(g, p, l, to: Option<Zone>) -> bool` is now the one way a land leaves: it removes the land and sends
its card to `to` through `zones::to_zone_card` (commander zone, owner, Blightsteel ... handled there); a token land (Mirrorworks
or Astral Dragon copy) goes nowhere and the result is false (so the "land card put into the graveyard" triggers fire only when
true). All 38 callers changed (None = parked, e.g. animate / Druid Class). `lands::sac_land`, `destroy_land` x2, Braids,
Liliana's ultimate, Kefka paths, Strip Mine, Urza's Saga, fetch crack, Glacial Chasm, bouncelands, Tragic Lesson, Daze ... all use it;
`cast.rs` Choice::Land too (it bypassed it). `animate`/`revert_animated` no longer push a token land's card to the graveyard.
Tests: destroy, sac, bounce/exile/tuck/graveyard/top and Liliana's mass sacrifice of token Vault/Citadel copies.

## 2. Astral Dragon
A person may pick any noncreature permanent (a land, an Aura); the AI still copies only nonland, non-Aura ones. Land copy:
`turn::enter_token_land_copy` (token land in `Player::lands` AND in `perms` as a 3/3 flying Dragon creature, summoning sick:
`mana_units` skips a sick creature land; land abilities see it as tapped until it is not sick; `land_arrives` = the enter effects
of a land, factored out of `play_land_card`; landfall, Etb triggers, Chasm/bounce/fetch effects run). `zones::leave`,
`remove_land` and `control` keep the two lists in step. `lands::animate` on an already-creature land only sets size and
keywords for the turn. Aura copy: Aura creature (rule 303.4d): both tokens enter and are put into the graveyard at once.
`Enter.token` (new field): a token copy is a token from the start, so an Aura's own set-up that removes it leaves no card.
Tests: land copy tapped/sick/mana/attack/die/destroy, Aura copy, AI still copies a nonland.

## 3. Ward
New `engine/ward.rs` `pay_ward(g, actor, m) -> Res<bool>`: ward {N} (card + Sheltered by Ghosts), Sauron, Squelcher/Prismari
life, Valgavoth sacrifice; a person is asked (mana ward through `pay_tax_ask`), the AI pays when it can. Called from
`removal::apply_removal_rest` for EVERY removal kind and for abilities too (the old `spell.is_some()` condition is gone),
`control::steal` (new; used by Marchesa's, Garland, Edgar, Brago, Stormdrake exchange, Spell Swap, Zealous-style steals),
dsl `tap/counters/pump/grant/phase_out` on targeted selections, and the card-code taps and -X/-X effects: Frost Titan, Rapier
Wit, The Wasp, Conformer Shuriken, Derevi, Azorius Guildmage, Errant Doomsayers/Whipcorder, Repel, Hidden Strings, Acrobatics,
Hireling, Fang, Sisters of Stone Death, Aura Shards. AI change: abilities that destroy/bounce/etc. an opponent's warded
permanent now pay (or are countered); `marchesa::steal`, `take_as_vampire`, `frost_tap` now return Res.
Tests: bounce spell, destroy ability, Squelcher life on an ability, steal, a person asked to pay.

## 4. Mystic Forge
`stack::counter_on_top` (Forge top card with the Ctr tag; Pact of Negation is the colourless one); `pick_counter`, the
response gates (`ai_respond`, the counter count) and `cast_counter_spell` (stages the top card into the hand, puts it back on top
if not cast) see it. Person: `legal::check_counter` accepts it, `human::can_respond` opens the window, and the Forge has a new
ability "cast X from the top of your library" (`human/karn.rs forge_cast`; any Forge-legal spell, a counterspell in response).
Tests: the AI casts the Pact from the top (40 seeds), no Forge no option, person legality/ability/window.

## 5, 6: not done (HANDOFF). Findings for the next agent
- Camera copies stack items: `stack::copyable` is true for triggers with a `Trigger` and for items with `ctx.dsl_ab`;
  `ability_window_ex` returns `copies` and the caller applies its effect once per copy. A hand-written ability needs: its
  effect split into `fn effect(g, src, p)`, its window changed to `ability_window_ex` with a new Ctx marker (say `hand_ab`) that
  makes `copyable` true, and `for _ in 0..out.copies { effect() }` before the original. Sauron's hand-written non-mana
  abilities: Barad-dûr, Expedition Map, Helm of the Host (equip), Jace's Archivist, Lightning Greaves (equip), Mind Stone,
  Rogue's Passage, Urza's Cave (cycling), plus any equip of Hammer/Cloak if hand-written.
- The person has NO Camera ability at all (nothing in human/ mentions it): needs "{2},{T}, remove a counter: copy target
  ability" offered while their own ability/trigger is on the stack, with a "copy which ability?" choice over `copyable` items.
- Marvin: same split for creatures' card-code abilities (kefka deck: Kefka, Blazing Firesinger; and t3 pool creatures).

## Shared-file changes
engine/turn.rs (remove_land, land_arrives, enter_token_land_copy), engine/zones.rs (leave, Enter.token, Aura Shards ward),
engine/control.rs (lands lists, steal), engine/mana.rs (sick creature lands), engine/removal.rs (ward moved out), engine/ward.rs
(new), engine/stack.rs (counter_on_top, pick_counter, cast_counter_spell, gates), engine/cast.rs (Choice::Land), engine/life.rs
(legend rule land), dsl/mod.rs (target_ok), human/{abilities,legal,human,karn,triggers_t1,cards,cloud}.rs, ai/decks.rs, ~25 impls
files (remove_land call sites, ward sites), tests (cards_green, cards_cloud, engine_t6_combo2, cards_legends, cards_t6_dack:
signature changes only). Data: data/audit.toml, data/card_overrides.toml, data/cards.json (re-exported).
