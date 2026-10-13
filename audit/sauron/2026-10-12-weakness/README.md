# Sauron weakness audit and rebuild (2026-10-12/13)

Quick reference of every swap, buy and bulk move: [`SWAPS.md`](SWAPS.md).

## Recommendation: twelve swaps, +8.3 points

Confirmed at 1,000 paired games per tier (look-ahead AI, loose profile, Tiers 1-5; `final/`):

| | T1 | T2 | T3 | T4 | T5 | All tiers |
|---|---|---|---|---|---|---|
| Current list | 26.9% | 30.7% | 22.1% | 20.8% | 21.2% | 24.3% |
| Proposed list | 33.7% | 39.5% | 32.1% | 30.7% | 27.0% | **32.6%** |
| Change | +6.8 | +8.8 | +10.0 | +9.9 | +5.8 | **+8.3** |

Every tier is up by more than its noise (±3.4 to ±4.0). 32.6% would put Sauron third of the ten decks, between
Sephiroth (36.9%) and Sheoldred (30.8%). The Army's first hit comes 0.5-0.9 turns earlier and in 8-13 percentage points
more games; Sauron himself lands about 0.1-0.5 turns earlier.

| Out (to bulk) | In | Package |
|---|---|---|
| Toxic Deluge | Ral and the Implicit Maze (in bulk) | one-sided control |
| Blasphemous Act | Massacre Wurm | one-sided control |
| Chaos Warp | Chandra's Ignition | one-sided control |
| Spell Pierce | Inferno Titan | one-sided control |
| Deepglow Skate | Talisman of Indulgence | speed |
| Conqueror's Flail | Rakdos Signet | speed |
| Sword of Fire and Ice | Dreadhorde Invasion | speed |
| Flux Channeler | Lazotep Reaver | speed |
| Underworld Breach | Exsanguinate | Breach slots |
| Brain Freeze | Torment of Hailfire | Breach slots |
| Dark Ritual | Fellwar Stone | Breach slots |
| Lotus Petal | Dimir Signet | Breach slots |

**Comet Storm for Torment of Hailfire** ($4 vs $36; 1,000 paired games per tier on the proposed list; `comet/`): -0.6
(-0.1, -3.7, +1.8, -0.1, -1.1), within noise (about ±1.4): no measurable difference, so the cheaper card is fine.

**To buy (11):** Massacre Wurm, Chandra's Ignition, Inferno Titan, Talisman of Indulgence, Rakdos Signet, Dreadhorde
Invasion, Lazotep Reaver, Exsanguinate, Torment of Hailfire, Fellwar Stone, Dimir Signet. (Massacre Wurm, Exsanguinate
and Fellwar Stone are in other decks already; Sauron needs his own copies.) Ral and the Implicit Maze is in bulk.

Bracket: Underworld Breach leaves, so Sauron drops to two Game Changers (Cyclonic Rift, Orcish Bowmasters), still
Bracket 3. No infinite combo is added (JD's rule); Sword of Feast and Famine + Aggravated Assault stays.

## Why (the audit, 960 games per tier, `analyze-t*.txt`)

- **Slow:** the old "online by turn 12" figure was the Sword + Assault combo (about 1% of games). The real engine, the
  Army's first hit, came around turn 8. Dreadhorde Invasion and Lazotep Reaver start the Army before Sauron lands.
- **Symmetric wipes killed the Army:** Toxic Deluge and Blasphemous Act were among the worst cards at every tier.
- **The Breach line won when cast but was rarely cast** (11-14% of games); its four slots did better as other cards.
- **Tier 5 combo decks** remain the hardest matchup (Kinnan, Urza, Winota and Zur still eliminate Sauron in most Tier 5
  games), but Tier 5 is up 5.8.

## The screen (500 paired games per tier, about ±2 over all tiers; `swaps/`, `breach-on-1d3/`)

| Test | Change | All tiers |
|---|---|---|
| 1D+3 | control (4) + speed (4), the first eight swaps above | +6.4 |
| 3 | speed package alone | +4.2 |
| 1A | Ral Maze + Massacre Wurm for the two wipes | +3.2 |
| 2C | Breach slots to three rocks + Dreadhorde Invasion | +2.8 |
| 1C | Chandra's Ignition + Inferno Titan for the two wipes | +2.6 |
| 2A | Breach slots to Exsanguinate, Torment + two rocks | +1.8 |
| 1B | Widespread Brutality + Crippling Fear | +0.5 |
| 2B | Breach slots to small creatures | -1.6 |
| B1 (on 1D+3) | Breach slots to Exsanguinate, Torment, Fellwar Stone, Dimir Signet | +2.8 more |
| B2 (on 1D+3) | Breach slots to Fellwar Stone, Dimir and Izzet Signet, Exsanguinate | +1.6 more |

## Kefka and the Breach package

- 2K, Brain Freeze, Dark Ritual and Lotus Petal for Duress, Dark Inquiry and Potioner's Trove: -0.8.
- Why (new Breach-line diagnostic in `--analyze`): the line wins whenever it runs, but Breach and a storm card are
  together in only about 18% of games, and half the attempts fall short on storm count or mana.
- K2, the same plus Grapeshot (bulk), Gamble and Jeska's Will for Ichor Slick, Thought Vessel and Chaos Warp: **+2.0**
  (+1.2, +3.2, +3.0, +2.6, +0.2), at the edge of the screen's noise. Brain Freeze, Dark Ritual and Lotus Petal would
  come from Sauron's cuts; Gamble and Jeska's Will would be bought.
- **K2 confirmation (1,000 paired games per tier, code 649adf3 with the hold fix; `kefka-k2/`): +0.8** (+1.5, +1.7,
  +0.3, +1.7, -1.0; about ±1.4 over all tiers). Not a real gain: the screen's +2.0 was mostly noise. Not recommended;
  nothing to buy for Kefka. (Gamble and Jeska's Will are both Game Changers: K2 would have put Kefka at four, Bracket 4.)
- **K3 (1,000 paired games per tier; `kefka-k3/`):** the Breach package + Grapeshot + Grim Tutor + Diabolic Intent +
  Jeska's Will (for Duress, Dark Inquiry, Potioner's Trove, Ichor Slick, Thought Vessel, Chaos Warp, Postmortem
  Professor): **+0.5** (+3.3, +0.6, -0.5, -0.7, -0.3), within noise. Better tutors don't rescue the line in Kefka.

## Simulator changes on this branch (`sauron-weakness`)

Sauron speed metrics (Army-hit plan milestone, commander timing, Army power), the Breach line for any deck plus Kefka's
AI for it and the line diagnostic, and new cards modeled: Crippling Fear, Widespread Brutality, Chandra's Ignition,
Inferno Titan, Lazotep Reaver, Grim Initiate, Torment of Hailfire; Ral and the Implicit Maze restored; Dualcaster Mage,
Dreadhorde Invasion and Grishnákh finished. 1,448 tests pass. No decklist is changed here; branch `sauron-1d3` holds a
test-only list and is not for merging.

## Applied 2026-10-13 (JD's choices)

The decklists on this branch now have the swaps: Sauron's twelve (Comet Storm in Torment of Hailfire's slot), Kefka's
seven (K3), and Veyran's Jeska's Will -> Flux Channeler. `bulk-cards.txt` has the sixteen cuts; Ral and the Implicit
Maze and Grapeshot left it.

**Data fix found while applying:** `pool_add_tags` in `data/card_overrides.toml` only apply to cards outside JD's
decks, so once Rakdos Signet and Dimir Signet were in Sauron's list the autotag made them two-mana rocks (`rock=2`) and
Talisman of Indulgence a colourless-only rock. Their tags are now `add_tags` (applied everywhere); every card's tags
are identical to before the list change. The screens, the +8.3 confirmation, K2/K3 and the Veyran test were on the old
lists with the correct tags. The B1/B2 and Comet Storm tests (test-only list on `sauron-1d3`) had the wrong tags in both
arms, so their differences should hold but their absolute win rates are a little off.
