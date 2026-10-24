# Report: t6-ai-pieces (Tier 6 tutors, free casts, self-damage)

Branch t6-ai-pieces. All AI choices are gated to Tier 6 AI players (`t6_plans::is_t6`, not human). No rules fix for
other tiers. Shared helper: new `impls/t6_pieces.rs` (piece groups of the deck's modeled lines, quiet lines included;
what the player holds: hand, battlefield, lands, command zone; which library cards complete a line).

## What each card does now
- **Gamble** (`fixes.rs` `gamble_allowed`, as the card's `can_cast`): with a line piece (not a land) in hand, Gamble is
  castable only if the library holds a card that completes a line and costs no more than the mana left after the {R}
  (so the find is cast at once). Without a piece in hand: as before.
- **Beseech the Mirror** (`rules2.rs` `beseech_free_ok`): the bargained free cast of a found line piece (Thassa's
  Oracle) happens only if the line's other groups are held; otherwise the piece stays in hand and nothing is sacrificed.
- **Chrome Mox** (`engine/tutors.rs` `chrome_imprint`): never imprints a piece of the deck's lines (Demonic
  Consultation, Underworld Breach, ...); with no other candidate it imprints nothing.
- **Wishclaw Talisman** (`rules2.rs` `wishclaw_ok`, `wishclaw_search`): offered only when a library card completes a
  line and is castable with the mana left after the {1}; the search then takes that card (cheapest) directly.
- **Invasion of Ikoria** (`t6_green2.rs` `ikoria_pick`): a candidate that completes a line (Oboro with Cradle, Gates and
  Thrasios held) comes before the wish list. Only creature X-tutor in the code.
- **The One Ring** (`partials.rs` `ring_life_ok`): not tapped when life <= 3 * (burden + 1) + 10.
- **Sylvan Library** (`topdeck.rs` `sylvan_pays`): pays 4 life for at most one card a turn, only at life >= 30, life
  margin >= 12 and desire >= 65 (other tiers: life >= 24, desire >= 55 as before).

## Tests
`tests/ai_t6_pieces.rs`, 8 tests (one per card, with the other-tier control). Full `cargo test --release`: all pass.
`cargo build --release --all-targets`: no warnings. t3 lock (300 games, seed 500000): identical.
No "internal simulator error" in any run.

## Tier 6, 300 games per profile (`--calibrate within --pool t6`), conservative profile
Before = `audit/t6/stage5-baseline.txt` (commit 423c135); after = this branch. At 300 games a deck's win % moves by
+-5 points from noise alone, so the win columns are not evidence either way.

| Deck | win % before | after | median win round before | after |
| --- | --- | --- | --- | --- |
| Dack | 27.1 | 25.1 | 9.0 | 7.5 |
| Kinnan | 32.5 | 33.5 | 7.0 | 7.5 |
| Kraum/Tymna | 24.1 | 28.7 | 8.0 | 8.0 |
| Rograkh/Silas | 19.9 | 19.4 | 7.0 | 7.5 |
| Rograkh/Thrasios | 17.1 | 14.7 | 15.0 | 12.0 |
| Sisay | 29.8 | 29.3 | 12.0 | 12.0 |
| table: avg rounds / timeouts | 10.2 / 2.7% | 9.9 / 2.7% | | |

Godo vs Tier 6 (300 games): win 52.3% (CI 46.7-57.9), wins median round 6.0, avg 6.9. The pieces fixes do not touch
Godo's wins (its combos are Helm / copy lines); Godo's baseline was 50-55% per the brief.

## Not done / notes
- Wishclaw Talisman has no way to see the tutored card win "that turn" beyond "castable now with the mana left"; it
  does not check the line's other steps' mana.
- Gamble's random discard can still hit the other half of the line when the find completes it (card text, no choice).
- The effect of these fixes is small in 300-game runs (the cases are rare: one to two games in twenty); the unit
  tests pin the behaviour.
