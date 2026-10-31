# Thanos, the Mad Titan (deck key `thanos`, Tier 6)

The list is Scion v1 minus Scion, Ancient Copper Dragon, Korvold and Prismari, plus Fierce Guardianship, Deflecting
Swat and Deadly Rollick (`decklists/JD/thanos-five-colour-turbo.md`). Thanos took Scion's seat, plan file and tests on
2026-10-30. Scion's history is in `audit/scion/`.

## Result (loose, 2000 games, seeds 500000-501999)
- **t6: 29.6% (CI 27.6-31.6).** Scion's last result was 30.1% (CI 28.1-32.1) on the same seeds; the two are within
  noise.
- **After the power-up fix (once per Thanos object, 2026-10-30): t6 30.8% (CI 28.8-32.8)**, same seeds; within noise of
  29.6%. Thassa's Oracle lines 79% of wins.
- **Goldfish:** median kill turn 11 (mean 11.1); 35.2% of kills come by turn 8; 82% of kills held protection.

## How it wins
- Thassa's Oracle + Demonic Consultation / Tainted Pact: 81% of wins (464 of 592). Damage: 15%. A few wins come from
  Underworld Breach and other combos. 20 wins came at the round cap.
- Median win round 7. The plan is ready in 36% of games and goes for a line in 35%. In the goldfish, the Oracle line
  converts 99.4% of tries.
- The free spells are live: Fierce Guardianship resolved 251 counters and Deadly Rollick 212 removals.
- Thanos's power-up resolved 415 times (about one game in five).

## How it dies
- Finishing blows: Kraum/Tymna 14.8% of games, Sisay 13.7%, Kinnan 11.2%, Dack Fayden 11.1%.
- Its lines are stopped mostly with no card named (a cost, a ward or a lock: 80 times). After that come about 20 each
  from Force of Will, Swan Song, Mental Misstep, Fierce Guardianship and Force of Negation.
- It mulligans 43.5% of hands. The keep rule is provisional, carried over from Scion; JD's mulligan decision is
  still open (D11).

## Strengths
- A cheap commander: in the goldfish he is cast in 80.5% of games, at a median own turn of 3. That turns on the three
  free spells.
- The same combo core and speed as Scion.

## Weaknesses
- No third plan once the Oracle and Breach lines are stopped. Scion's Prismari and Copper Dragon backups are gone.
- Casting Thanos when the deck is one tutor away competes with holding mana for the win turn (73 decisions).

## Model status
- **Thanos: Approximate.**
  - Power-up costs {C}{W}{U}{B}{R}{G}, or {C}{U}{G} on the turn he entered. {C} must be paid with colourless mana.
  - He takes two +1/+1 counters, then each other creature with mana value of the chosen parity is destroyed (0 is
    even).
  - The AI picks the parity from the board: the opponents' value destroyed against 1.2 times its own.
  - Once per Thanos object: a blinked or recast Thanos is a new object (rule 113.7) and may power up again, at the
    reduced cost on the turn he returns (fixed 2026-10-30).
  - Approximation: a token's mana value is 0.
- **Fierce Guardianship, Deflecting Swat, Deadly Rollick: Full.** Each is free only with a commander on the
  battlefield.
- **Partial:** Orim's Chant, Redirect Lightning, Tinder Wall, Gemstone Caverns, Shifting Woodland.
- **Approximate:** Crop Rotation, Hexing Squelcher, Lotho.

## Tested
No swaps yet.
