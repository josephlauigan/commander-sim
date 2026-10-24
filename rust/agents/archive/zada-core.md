# Task: zada-core — Zada's copy ability, the Zada spells, the spell payoffs and Zada's AI plan

Deck: `decklists/JD/zada-mono-red-go-wide.md` (key `zada`). Read its Strategy and Key lines first.

Your files: `rust/crates/sim-core/src/impls/zada.rs` (new; register it in `impls/mod.rs`), its tests in
`rust/crates/sim-core/tests/cards_zada_core.rs`, card data rows for these cards. Another agent (zada-cards) models the
rest of the deck's cards in `impls/zada_cards.rs`; don't touch those cards.

## Zada, Hedron Grinder (the core)
*Whenever you cast an instant or sorcery spell that targets only Zada, copy that spell for each other creature you
control that the spell could target. Each copy targets a different one of those creatures.* Copies are not cast
(no cast triggers, no storm count). Each copy resolves with its own target, including card draw, Role tokens and
"exile the top card, you may play it" effects. Build it on the engine's existing spell-copy machinery if there is
one; a test with three creatures for at least Ancestral Anger (three extra cards) and Monstrous Rage (three Roles).

## Cards to model (until `cardgen audit --mine` shows each as Full)
The Zada spells: Ancestral Anger (+X/+0, X counts Ancestral Angers in your graveyard; draw), Monstrous Rage (Monster
Role: +1/+1 and trample; a new Role replaces an older one on the creature), Bulk Up (double power; flashback), Act of
Treason (control, untap, haste until end of turn), Into the Fray (attacks this turn if able; splice onto Arcane can be
a no-op if the deck has no other Arcane spell: say so). Also check that Blazing Crescendo, Sure Strike, Furious
Bellow, Smaug's Fury, Crowd's Favor (convoke), Twinflame (token copies with haste, exiled at end step) and Return the
Favor (copy a triggered ability, used to copy Zada's trigger) really work with Zada's copies; fix what doesn't.

Spell payoffs and spell recursion: Speed, Young Avenger, Expressive Firedancer (opus), Burning Prophet (scry),
Dreadhorde Arcanist (on attack, cast an instant or sorcery with mana value up to its power from your graveyard for
free, then exile it), Empty the Warrens (**storm**), Firebird, Blazing Ranger.

## AI plan for zada (in `impls/zada.rs`, wired into `ai/decks.rs` / `ai/brain.rs` like the other decks)
- Cast a Zada spell **targeting only Zada** when he's on the battlefield with at least two other creatures you
  control; otherwise hold it (the cantrips may still be cast normally if hand is empty of better plays).
- Draw and card spells (Ancestral Anger, Blazing Crescendo) in the main phase before combat; pump spells when
  attacking, after blockers if they're instants; Twinflame before the pump on a turn you attack; Act of Treason on
  Zada for haste when fresh tokens could attack.
- Go wide first: creatures and token makers before Zada spells while Zada isn't out.

Smoke run for this deck: `--deck zada`.
