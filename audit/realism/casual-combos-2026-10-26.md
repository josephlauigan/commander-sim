# Casual tiers: combo and alternate-win lines (2026-10-26)

In Tiers 0–2 the sim almost never won by combo or alternate win, while Playgroup's games often do (Tier 0 13.8%,
Tier 1 11.3%, Tier 2 21.2%). In 200 traced games, five decks won by combo close to never: Stella Lee 0 of 13,
Brago 0 of 22, Teysa 3 of 30, Sythis 0 of 70 and Tatyova 0 of 13. Meren was fine at 11 of 22. One agent per deck read
the real list's non-combat wins line by line (Oracle text), checked each against the combo catalogue and the AI, and
added only the lines the list really has. No deck list changed.

Playgroup data is CC BY 4.0, from playgroup.gg.

## What each deck has

| Deck | Tier | Real non-combat lines | Change |
|---|---|---|---|
| Stella Lee (precon) | 0 | Spell-chain burn only (Guttersnipe, Electrostatic Field, Niv-Mizzet, Veyran). No combo, alternate win or mill. | The pool AI never cast Mizzix's Mastery and seldom cast Galvanic Iteration; it now casts both (`t0_stella.rs`). |
| Tatyova | 1 | None: it wins by combat. | Retreat to Coralhelm untaps Scaled Herbalist while a land waits in hand (`partials.rs`). |
| Teysa Karlov | 1 | Pitiless Plunderer + Reassembling Skeleton + Altar was catalogued already. Missing: Teysa or Drivnod doubling the Plunderer's Treasures to pay for the Skeleton, and Phyrexian Altar + Skeleton + Pawn of Ulamog or Sifter of Skulls. | Both lines added (`cc_teysa.rs`). Pool Teysa no longer sacrifices her own payoffs and Plunderer as fodder (`common.rs`, this deck only). |
| Brago | 2 | Peregrine Drake + Archaeomancer + Ghostly Flicker was catalogued but never fired; Displacer Kitten + Archaeomancer + Drake or Cloud of Faeries + a flicker spell was missing. Both finish with Agent of Treachery. | Both lines fixed or added (`cc_brago.rs`); Cloudshift is no longer cast with no target (Brago only). |
| Sythis | 2 | Blind Obedience's extort only, already modelled. No alternate win, mill or loop. | None. |

None of these lists holds a two-card infinite.

## Scorecard, before and after (4,000 games, loose)

| Statistic | Tier 0 before | Tier 0 after | Tier 1 before | Tier 1 after | Tier 2 before | Tier 2 after | Targets (t0 / t1 / t2) |
|---|---|---|---|---|---|---|---|
| Wins by combo or alternate win | 0.6% | 0.4% | 1.2% | 2.2% | 8.2% | 9.8% | 13.8% / 11.3% / 21.2% |
| Targets passing | 3 of 9 | 3 of 9 | 4 of 9 | 4 of 9 | 1 of 9 | 1 of 9 | |
| Stella / Teysa / Brago share of their decisive games | 14.4% | 16.9% | 18.9% | 19.2% | 10.7% | 11.3% | |

The "after" runs are on the merged branch, seeds 500000–503999.

## Why the gap stays

The combo-share targets average each commander's Playgroup page, and those pages pool every list players log under
that commander, upgraded builds included. These lists, Tier 0's precon and Tiers 1–2's EDHREC bracket 2–3 lists, hold
few or no combo lines. Where they do (Teysa, Brago), the lines need three or four specific pieces and the decks have
few tutors. So the remaining gap is in the lists, not the sim. Closing it would mean playing combo-heavier lists, or
making the win-type rows references (`scored = false`) as in Tiers 4–5. JD decided to keep them scored in Tiers 0–2
(2026-10-25), so they stay scored and fail.

Checks: `cargo test --release` passes (206 binaries, 3,463 tests). The build has no warnings.
