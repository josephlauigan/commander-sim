# Tier 6: Rograkh/Thrasios's real win lines (2026-10-26)

Rograkh/Thrasios won 17.9% of the decisive Tier 6 games it sat in, below the 18-32% band every other deck reaches. Its
ready round was 8.

## Lines in the list

The list (`decklists/pool/t6-cedh/rograkh-thrasios-creature-combo.md`) does not hold This Town Ain't Big Enough, the
Breach / Brain Freeze / Lion's Eye Diamond line or Izzet Guildmage: the handoff's guess was wrong. Every line ends with
Thrasios drawing the library, then Finale of Devastation (X 10 or more) or Faerie Mastermind with Green Sun's Zenith.

| Line | Before | After |
|---|---|---|
| A. Oboro + Gaea's Cradle + Talon Gates | Modelled | Chord of Calling, Nature's Rhythm or Invasion of Ikoria can now put Oboro in. With Training Grounds or Biomancer's Familiar, Oboro's ability costs {1}, so Cradle at 5 is enough. |
| B. Eternal Witness + Snap + Cradle | Both cards had to be in hand | Also starts with Witness on the battlefield and Snap in hand, with Witness in hand and Snap in the graveyard, or with a creature tutor fetching Witness. |
| C, D. Badgermole Cub + clone lines | Modelled | Unchanged |
| Finish | Thrasios had to be on the battlefield | The loop's mana can cast Thrasios from the command zone, and that cast can be countered. With Finale in hand, the deck wins without Thrasios. |
| G. Springheart Nantuko + Vibrance | The code comment said "finite" | It is unbounded by Oracle; only the comment is fixed. It runs only alongside line A, which already wins. |

## Why it was losing

The deck spends Snap as a tempo bounce, so line B's two cards were seldom in hand together. A measurement bug also
hid the gap: no quiet Tier 6 line was ever counted as "one tutor away". Once that count was fixed, it showed the deck
passing while it held Green Sun's Zenith, Finale, Chord, Nature's Rhythm or Invasion. The tutor wish list now asks for
the closer line's missing piece first.

## Numbers (4,000 games, seeds 500000-503999, loose)

| Measure | Before | After |
|---|---|---|
| Rograkh/Thrasios share of its decisive games | 17.9% | 21.3% |
| Calibrate wins by line A / B / C / D / damage | 103 / 16 / 11 / 2 / 34 | 100 / 69 / 5 / 4 / 34 |
| Ready (median round, share of games) | 8 (17%) | 8 (23%) |
| Median win round | 10 | 9 |
| Tier 6 deck win spread | 16.7 pts | 11.6 pts |

The seat targets still fail, as before.

## Leads left alone

- `combos::deck_combos` drops every quiet line, so `combo_progress` (how the lookahead AI judges a combo in progress)
  ignores every stepped Tier 6 line. Fixing it changes every Tier 6 deck's AI.
- One tutor away counts only tagged tutors. Crop Rotation, Gamble, Expedition Map, Sylvan Scrying and Spellseeker
  are untagged.
- Chrome Mox and Mox Diamond are often held on turns 1-3 with the mana to cast them.

Checks: `cargo test --release` passes (197 binaries, 3,309 tests). The build has no warnings.
