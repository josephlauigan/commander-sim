# Report model-gaps-a4 (branch model-gaps-a4)

Status: items 1-4 done. Tests: `rust/crates/sim-core/tests/model_gaps_a4.rs` (27 pass) plus the related files (model_gaps_a3, karn, others, the practice_* files, the pool decks' card tests, Godo/Sauron tests, about 60 files): all pass. `cargo build --release --all-targets`: no warnings. `cardgen export --offline`, `audit --mine`: only Hexing Squelcher below Full (Marvin is Full). Not run: the full suite and any sim.

## 1. Five abilities (impls/model_gaps_a4.rs, human/model_gaps_a4.rs)
Each is an `options` hook (the AI, and Marvin borrows it with Marvin as the source: marvin_options runs every other creature's hook)
and a person ability (`human::cards::abilities` asks `model_gaps_a4::abilities` first; the person pays from the pool). Each puts
the ability on the stack with `stack::ability_run_w` (cost already paid), so Peter Parker's Camera and Rings can copy it.
- Siege-Gang Commander {1}{R}, sacrifice a Goblin: 2 damage to any target (`human::choices::apply_damage`). AI: finishes the
  player with the least life when the mana and the Goblins reach it (activations = min(Goblins that can go, mana/2), utility 8.5),
  a planeswalker at 2 loyalty or less (3 + value/2), a creature that 2 damage kills and is worth 3.5 or more (value - 2); the Goblin
  it gives up is the least valuable token or creature under 3 (never a commander). A person chooses the target and the Goblin; a
  Camera copy of a person's chooses a new target.
- Syr Konrad {1}{B}: each player mills a card (`zones::mill`: Konrad's own trigger does the damage). AI: with spare mana (end-step
  window first) when its own library has 8+ cards and the creature cards expected from the libraries are worth it
  (0.8 + 0.3 x creature cards x opponents, at least 1.0).
- Professional Face-Breaker: sacrifice a Treasure (Sacrifice triggers fire; a tapped one first): exile the top card, playable this
  turn (hand + `impulse`, as Urabrask's). AI: your turn, main phase, when a land (drop open) or a card the mana left pays for is at
  least 55% likely (utility 1 + 4 x chance), from the library's make-up (the top card is not read).
- Tinybones {4}{B}{B}: each opponent with no cards in hand (as it resolves) loses 10 life. AI: 8.5+ when one such opponent is at
  10 or less, 3.0 when at 20 or less.
- Ezuri {2}{G}{G}{G}: Elf creatures you control (as it resolves) get +3/+3 and trample until end of turn. The AI's overrun was
  t1's, dead code: the Python port replaced it by an empty hook (rules.py `_ezuri_none`), probably because it fired whenever four
  Elves were ready (utility 1 + 0.8 x Elves, no thought of the attack, `player.trample` for all attackers). Both removed. The new AI
  use: before combat only, when the pump makes the all-out attack lethal through the blockers (9.0) or the pumped damage is 70% of
  a life total and 4+ more than without (2.5-4). Trample is per Elf (`eot_kw`).
Shared files: impls/mod.rs (registration), impls/t1.rs (dead Ezuri code removed), impls/rules.rs (`ezuri_none` removed),
impls/rules2.rs (`squirrels`, `fang_go` visible), impls/t5.rs (`kiki_go` visible), human/cards.rs (dispatch), human/mod.rs.

## 2. Marvin: a person chooses
A person using any ability Marvin borrows is asked what the creature's own ability asks, with Marvin as the source and the one that pays. `human/model_gaps_a4.rs` `marvin_abilities` lists, per creature in `BORROWED` (46) or `marvin_hooks::NAMES` (52), the union of `abilities::permanent_abilities`, `karn::`, `cloud::perm_abilities` and `cards::abilities` of the donor (labels deduplicated; Kefka and Tekuthal are built with the donor captured, as the text names it); the AI hook options of those creatures are dropped for a person (`replaced_label`). New file `human/marvin_hooks.rs`: a person path (cost from the pool, every choice asked, the ability on the stack with `ability_run_w`/`aimed`, so a Camera copy picks a new target) for the 52 hook-only creatures with a choice: Amphin Pathmage, Chittering Witch, Cankerbloom, Captivating Vampire, Goblin Cratermaker, Goblin Trashmaster, Grim Hireling, Hawkeye, Hope of Ghirapur, Jagged-Scar Archers, Loran, Mogg Fanatic, Pashalik Mons, Pia Nalaar (both), Sai, Temur Sabertooth, Timberwatch Elf, Vilis, Weathered Wayfarer, Destiny Spinner, Eldrazi Displacer, Formidable Speaker, Patriot, Deathrite Shaman (both), Ioreth (both), Goblin Engineer, Goblin Welder, High Perfect Morcant, Kinnan, Loot, Magda, Magus of the Candelabra, Mina and Denn, Nullmage Shepherd, Nyx Weaver, Oboro Breezecaller, Preston, Priest of Forgotten Gods, Recruitment Officer, Ruthless Technomancer, Sisay, Skrelv, Tavern Scoundrel, Teysa, Urza (Powerstone Prodigy), Eladamri, Drivnod, Elvish Reclaimer, Brutal Hordechief, Scourge of Valkas, Lathril, Tekuthal. These are also the creatures' own person abilities (a person had only the AI's pick before). The 24 creatures with a person path elsewhere (Balan, Cid, Stoneforge, Steel Hellkite/Overseer, Forgemaster, Thrasios, Triskelion, Yawgmoth, Thanos, Scion, Kefka, Garth, Gix, Jaxis ...) were added to `BORROWED`. Creatures whose hook has nothing to choose stay the AI's option for a person (`NO_CHOICE` in the test). Also: a person chooses what a compiled sacrifice cost sacrifices (`dsl::pay_ability_cost`, a4 part 1).
Other changes: `newcards_b::NOT_ACTIVATED`: creatures whose `options` hook is a casting permission, a prepared spell or a paired grant (Muldrotha, Realmwalker, Korlessa, Rivaz, Valgavoth, Snoop, Navigator, the prepared creatures, Restless Bloodseeker) are no longer borrowed by Marvin (they were offered as if activated abilities). Marvin carries `blocks` (Brutal Hordechief: `hordechief_blocks`, which asks a person which attacker each blocker blocks), `combat_damage` (`hope_hit`: Hope of Ghirapur's ability) and `grant_kw` (`tekuthal_kw`: Drivnod's counter) hooks, so these abilities do something when Marvin uses them (before, the data went onto Marvin and nothing read it). Shared files: impls/{newcards_b,partials,mine,rules2,t45_edgar,tymna_cards}.rs (pub(crate)/hooks), human/{mod,model_gaps_a4}.rs. Tests: `every_creature_with_an_options_hook_is_a_person_path_a_no_choice_hook_or_not_an_ability` (classifies every creature with a hook), `a_person_using_each_borrowed_ability_is_asked_and_never_gets_the_ais_choice` (walks all 98 creatures in a rich table: each is listed as a person ability, never as a hook, and a person is asked unless QUIET says why), spot tests for Mogg Fanatic, Cankerbloom, Chittering Witch, Tekuthal, Drivnod, Hordechief blocks, NOT_ACTIVATED.

## 3. Godo Helm {5} equip
`StepKind::Equip` (impls/combos.rs): `run_steps` pays the cost, then `ability_run_w` puts the equip on the stack (copyable, answerable)
and runs the step's effect once per resolved copy and once for the ability; the step table is otherwise unchanged. Used by
`S_EQUIP` (t6_combos_godo.rs) and `S_EQUIP_LOOP` (t6_combos_extra.rs). The person's equip was already on the stack (a3). Worth 0:
the AI's Camera does not copy it (no gain). Test: an equip step with a person's Camera copying it.

## 4. Marvin's status
Full in data/card_overrides.toml and data/audit.toml (the note names what is borrowed, how a person is asked, what is not an activated ability, and the named-creature reading). Camera's note untouched. `cardgen export --offline`; `audit --mine`: only Hexing Squelcher below Full.

## Shared-file changes
impls/{mod,t1,rules,rules2,t5,combos,t6_combos_godo,t6_combos_extra}.rs, human/{mod,cards,human}.rs, dsl/mod.rs; data/card_overrides.toml,
data/audit.toml. New: impls/model_gaps_a4.rs, human/model_gaps_a4.rs, tests/model_gaps_a4.rs.
