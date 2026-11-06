# Report model-gaps-a3 (branch model-gaps-a3)

Status: items 1-6 done (Marvin stays Approximate: see NEEDS JD in the reply). `cardgen audit --mine`: Marvin 1, thanos 1 (Hexing Squelcher).
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
## Shared-file changes
engine/turn.rs (remove_land, land_arrives, enter_token_land_copy), engine/zones.rs (leave, Enter.token, Aura Shards ward),
engine/control.rs (lands lists, steal), engine/mana.rs (sick creature lands), engine/removal.rs (ward moved out), engine/ward.rs
(new), engine/stack.rs (counter_on_top, pick_counter, cast_counter_spell, gates), engine/cast.rs (Choice::Land), engine/life.rs
(legend rule land), dsl/mod.rs (target_ok), human/{abilities,legal,human,karn,triggers_t1,cards,cloud}.rs, ai/decks.rs, ~25 impls
files (remove_land call sites, ward sites), tests (cards_green, cards_cloud, engine_t6_combo2, cards_legends, cards_t6_dack:
signature changes only). Data: data/audit.toml, data/card_overrides.toml, data/cards.json (re-exported).
## 5. Peter Parker's Camera (Full)
Cost and effect of every hand-written activated ability in the Sauron deck are split: `stack::ability_run_w(g, p, src, cd, name, imp, target, worth, eff)`
puts the ability (cost already paid) on the stack with `Ctx::copy_worth = Some(worth)` (`copyable` is true for it), then runs
`eff(g, is_copy)` once for each copy that resolved (Camera or Rings), then once for the ability if it was not countered. New
`ability_window_card_ex` (source is a card: lands, abilities from hand). AI: `ai_respond` copies an own ability whose worth is
`CAMERA_WORTH` (3) or more (Expedition Map, Urza's Cave 3.0; Barad-dur worth X; the rest 0 = never). Person: the existing Camera
ability ("copy which ability?") now lists them, and `camera_window` gives priority on any copyable hand-written ability.
Converted (AI and person code): Barad-dur, Expedition Map, Mind Stone, Jace's Archivist, Rogue's Passage (a person may pick a new
target for the copy), Urza's Cave, Scavenger Grounds, Sokenzan's channel (it now uses the stack; it did not before), and every equip
(`stack::equip_ability`, used by `equip_to`, `dsl::equip`, the pool protection equips and the person's Equip). Mana abilities are not
on the stack (rule 605.3). Files: engine/stack.rs, state.rs (Ctx.copy_worth), dsl/mod.rs (equip), ai/pool.rs (equip_protection),
impls/{common,mine,karn_engines,generic_paths,lands}.rs, human/{cards,karn,abilities,human,urdragon}.rs.
Tests (model_gaps_a3.rs): effect runs again without the cost (worth 0 vs 3), the AI copies its Expedition Map, a person copies
Expedition Map, Mind Stone and Rogue's Passage (new target for the copy).

## 6. Marvin, Murderous Mimic (Approximate, see NEEDS JD)
Marvin borrows (AI, and a person through the same hook options): every other creature's `options` hook run with Marvin as the source
(`newcards_b::marvin_options`): Chatterfang, Stoneforge Mystic, Priest of Forgotten Gods, Sakura-Tribe Elder and Cankerbloom (the AI
weighs Marvin as the thing sacrificed), Yawgmoth, Allosaurus Shepherd, Eladamri, Elvish Warmaster, Timberwatch Elf, Skrelv ...;
Kefka's {8} (`mine::kefka_borrowed`: Marvin pays, Kefka is the one that transforms); Tekuthal's (`partials::tekuthal_borrowed`: the
indestructible counter goes on Tekuthal); Bloodline Keeper's transform is not offered (it does nothing to Marvin). Sacrifice outlets
(`common::outlet_name`: Carrion Feeder, Viscera Seer; person: `cards::marvin_outlets`). Protectors (`pool::protector_options`: Mother
of Runes ... with Marvin as the source). Mana (`mana::marvin_donor`): Marvin taps for the best mana creature's ability (restriction, rider
and pain from that creature) and has Quirion Ranger's / Wirewood Symbiote's untaps. Tests: Kefka, Priest, Timberwatch, Carrion Feeder,
mana, Quirion. Files: impls/{newcards_b,mine,partials,common}.rs, engine/mana.rs, ai/pool.rs, human/{cards,human}.rs.

## Shared-file changes (items 5, 6)
engine/stack.rs (ability_run_w, ability_window_card_ex, equip_ability, copyable, ai_respond), state.rs (Ctx.copy_worth), engine/mana.rs
(Marvin's mana, Quirion/Symbiote units, mana_card), dsl/mod.rs (equip), ai/pool.rs (protector_options, equip_protection), human/*.
Data: data/card_overrides.toml, data/audit.toml (Camera Full; Marvin note), data/cards.json (re-exported).
