# Report t6-ai-rthras (deck:rograkh-thrasios-creature-combo)

Files: `impls/t6_plan_rograkh_thrasios.rs` (all five slots), `tests/ai_t6_rograkh_thrasios.rs` (7 tests: spec scenarios 10, 11, 12,
tutor order, Tier 6 gate, line-A go, keep rate over 300 opening hands).
Shared-file hook (Tier-6-gated, small): `engine/stack.rs` `pact_affordable` - see "Why" below.

## What the slots do
- `prio`: Rograkh 97 (turn 1, {0}), Thrasios 93; Oboro, Finale, GSZ, Snap held at 0 (cast only by the combo, spec "do not cast
  Oboro on a turn the deck is not going for the win"); Eternal Witness held unless a counter / protection is in the graveyard;
  Mastermind held while GSZ is in hand; cheap bodies for Cradle 82; Gamble / Crop Rotation 88 while Cradle is still in the
  library, 0 once Cradle is in hand or in play (Gamble discards at random).
- `wish`: Cradle first, fast mana when the board has 2 or fewer mana sources, then Talon Gates, Oboro, Witness, Snap, Finale,
  Spellseeker, Mastermind, GSZ, Veil.
- `keep`: 2+ lands and an accelerant; or 1+ land with land+accelerants >= 3; 2+ lands with an engine card or 3 other plays and
  fewer than two reactive cards; 3+ lands with fewer than 3 reactive; ship 6+ lands, 0-1 landers without fast mana, and
  reactive-only hands. Keeps 73% of 7-card hands (the deck averages 1.9 lands in 7).
- `go_now`: lines A and B go when a cover card (Force of Will, Fierce, Pact, Veil, Swat, Squelcher) is held, or late (round 5 / 7);
  otherwise the shared check decides. `hold_up`: no opinion (not done).

## Why it did not assemble (traces, `--deck godo --pool t6 --trace N`)
1. Line A needs Cradle + Thrasios in play + 6 creatures + Oboro in hand + Gates in reach; Cradle is one card in 100. It was
   in play in only 5 of 19 traced Rthras games, and Godo/others often win before. The generic AI also cast Oboro/Witness as plain
   creatures and tutored Sol Ring / Mana Vault with Gamble.
2. **Pact of Negation lost games**: the counter code treated "5 lands" as able to pay {3}{U}{U} at the next upkeep. 4 of ~21 traced
   Rthras games (and Kraum, Silas, Sisay games) ended "can't pay for Pact of Negation and loses". Fixed for Tier 6 only
   (`pact_affordable`: 5 lands per pact owed and 2 blue lands; Tiers 0-5 keep the old 5-land test). Not visible in the numbers
   below because all six decks benefit.
3. Mulligans were 35% (49% with my first, too strict rule); the deck has 27 lands of 100 so 0-1 landers are common.

## Before / after (`--calibrate within --pool t6 --games 300`, conservative profile, Rograkh/Thrasios block; before = audit/t6/stage5-baseline.txt)
| | before | after |
|---|---|---|
| win % | 17.1 (12.6-22.7) | 16.2 (11.8-21.8) |
| wins by | damage 94%, Oboro line 6% | Oboro line 55% (18 wins), damage 45% |
| win round avg / median | 14.8 / 15 | 13.1 / 13 |
| combo tries (per game) | 5 (0.02) | 28 (0.13) |
| tries stopped | 3 (60%) | 10 (36%) |
| mulligans | 75 of 211 (35.5%) | 52 of 210 (24.8%) |
| loose profile win % | 13.3 | 12.9 |
Noise at 300 games is about +-5 points: win rate is flat, the route moved to the spec's main line (A).

## Still differs from the spec
- Wins come at round 13, not turns 3-6: the deck rarely has Cradle and six creatures early. Plan A needs 6 creatures
  (spec scenario 20: even 5 phases the table out - that is the combo definition in `t6_combos_t6.rs`, not this slot).
- Lines B-G are almost never tried (B 2 of 28); Spellseeker -> Snap chains, Mastermind + GSZ finish and Vibrance loop are not
  planned by the slots. Scenarios 2, 4, 13-22 (bait first, Totem/Agent/Rule of Law reactions, Thrasios digging in response)
  are not implemented or tested; `hold_up` is no opinion.
- Rograkh is recast again and again after dying (attacks into blockers); Thrasios is often exiled/bounced on sight.
- Stage 6 candidates: the 6-creature requirement, Cradle count (Crop Rotation / Gamble / Chord only), stax (Trinisphere) that
  leaves 7 cards stuck in hand.
