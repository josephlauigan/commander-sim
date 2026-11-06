# Report model-gaps-a4 (branch model-gaps-a4)

Status: items 1 and 3 done; item 2 partly (a person's choices for Marvin's Kiki-Jiki, Chatterfang, the five new abilities and 17
other creatures with a person path; the rest in rust/agents/handoff-model-gaps-a4.md); item 4 not done (Marvin stays
Approximate until item 2 is). Tests: `rust/crates/sim-core/tests/model_gaps_a4.rs` (17 pass) plus 36 related files (model_gaps_a3,
karn, others, the practice_*_probe files, the pool decks' card tests, Godo/Sauron tests): all pass. `cargo build --release
--all-targets`: no warnings. `cardgen export --offline`, `audit --mine`: only Marvin and Hexing Squelcher below Full.
Not run: the full suite and any sim.

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

## 2. Marvin: a person chooses (part)
`human::model_gaps_a4::BORROWED` names the creatures whose person path Marvin uses (`marvin_abilities`, listed by `abilities_of`;
the AI-chosen hook options of those creatures are dropped for a person: `replaced_label`). New person abilities: Kiki-Jiki (the
person picks the nonlegendary creature to copy, Marvin included) and Chatterfang (the person picks the target and X), also for the
creatures themselves. In BORROWED: those two, the five above, Azorius Guildmage, Blightspeaker, the Ramosian searchers, Defiant
Vanguard, Lin Sivvi, Ramosian Revivalist, Errant Doomsayers, Lawbringer, Lightbringer, Urborg Syphon-Mage, Sakura-Tribe Elder,
Jace's Archivist. `dsl::pay_ability_cost`: a person chooses the permanent a compiled sacrifice cost sacrifices (asked after the
mana is known payable; the AI still gives up the least valuable). Marvin's Krenko needs no choice (it stays a hook).
Not done: the hook-only creatures listed in the handoff file (a person still gets the AI's target for those through Marvin).
Files: human/human.rs (`abilities_of`), human/model_gaps_a4.rs, dsl/mod.rs.

## 3. Godo Helm {5} equip
`StepKind::Equip` (impls/combos.rs): `run_steps` pays the cost, then `ability_run_w` puts the equip on the stack (copyable, answerable)
and runs the step's effect once per resolved copy and once for the ability; the step table is otherwise unchanged. Used by
`S_EQUIP` (t6_combos_godo.rs) and `S_EQUIP_LOOP` (t6_combos_extra.rs). The person's equip was already on the stack (a3). Worth 0:
the AI's Camera does not copy it (no gain). Test: an equip step with a person's Camera copying it.

## 4. Marvin's status
Not set to Full (item 2 is not finished). The Marvin note (data/card_overrides.toml, data/audit.toml) names exactly what is done and
what is left. Camera's note is untouched. Professional Face-Breaker got a card_notes entry; the notes of the other four were updated.

## Shared-file changes
impls/{mod,t1,rules,rules2,t5,combos,t6_combos_godo,t6_combos_extra}.rs, human/{mod,cards,human}.rs, dsl/mod.rs; data/card_overrides.toml,
data/audit.toml. New: impls/model_gaps_a4.rs, human/model_gaps_a4.rs, tests/model_gaps_a4.rs.
