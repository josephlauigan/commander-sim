# Report E: engine batch (2026-11-04)

Branch engine-batch. Tests: whole workspace `cargo test --release --workspace --no-fail-fast`: 3991 passed, 0 failed,
no warnings (both probes, the counter probe and play_bot_games included). No sims run.

1. Ragavan (impls/t4.rs): the top card of the damaged player's library is exiled; a land stays exiled, anything else is
   held in the hand as an impulse card (`impulse` + `stolen`, not `agent_ids`, so ordinary mana). The AI's usual cast
   logic and a person's hand list reach it; `cast_card` now gives the card back to its owner when `stolen` alone is set;
   the end step (engine/turn.rs) sends an uncast card to its OWNER's exile (also corrects Grenzo's cards). The old
   cast-now-if-permanent code is gone. Tests: tests/engine_batch_e.rs (3), tests/practice_ragavan.rs.
2. Goldspan Dragon (impls/rules.rs `goldspan_targeted`, called from `card_models3::spell_targeted` in `cast_card`): a
   Treasure when any player's spell targets it as the spell is cast. Not covered: spells whose target is chosen only on
   resolution, and abilities (the card says spell). Test in engine_batch_e.rs.
3. MDFC back faces. Only two are in the owned decks: Bloodsoaked Insight // Sanguine Morass (Sauron) already worked both
   ways (land, and the sorcery via hand_options); a person test added. Birgi // Harnfel (Kefka): Harnfel is now an
   engine-made card (cards.rs `engine_made`, no data change, no export needed); a person can cast it from the hand
   (Birgi's `hand_options`, mine.rs), the permanent's `phys` is Birgi's card so Birgi goes to the graveyard when it
   leaves; its discard ability is in human/gaps_c.rs. The AI never casts Harnfel (no clear use over a 3-mana Birgi),
   so no AI behaviour change. Tests: tests/practice_harnfel.rs (2).
