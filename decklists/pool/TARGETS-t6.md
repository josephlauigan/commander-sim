# Tier 6 (cEDH) — Calibration Targets

What real cEDH tournaments look like, as numbers the simulator must match before Godo is run against Tier 6
(Stage 6 of the Tier 6 plan). Every number says where it came from. The folder is `staging-t6-cedh/` until the
tier is wired into the simulator (Stage 3); `cardgen` skips it until then.

## Data window

- **Window:** 2026-04-18 to 2026-10-17 (EDHTop16 "six months"), pulled 2026-10-17.
- **Why six months:** the agreed rule is six months or since the last Commander banned-list change, whichever is
  shorter. The last change was 2026-02-09 (Biorhythm unbanned, Lutri unbanned except as a companion, Game Changers
  list updated; [Wargamer](https://www.wargamer.com/magic-the-gathering/commander-banlist)), so six months is shorter.
- **Source:** the [EDHTop16](https://edhtop16.com) GraphQL API: commander stats and per-tournament seat win and
  draw rates, for tournaments of 32+ players. 638 tournaments, of which 596 (31,518 entries) report game results.

## Pod-level targets

| Target | Real value | Sim must land in | Notes |
| --- | --- | --- | --- |
| Seat 1 share of decisive games | 34.1% | 30–38% | Size-weighted over 596 events; per-event middle half 29.2–39.0%. Events of 100+ players: 33.9%. |
| Seat 2 share | 27.3% | 24–31% | |
| Seat 3 share | 22.5% | 19–26% | |
| Seat 4 share | 17.4% | 14–21% | Per-event middle half 12.4–22.1%. Events of 100+ players: 17.0%. |

"Decisive" means games with a winner: each seat's win rate divided by (1 − draw rate), per event. The seat edge is
the best public measure of speed: the faster the format, the more an extra turn is worth. The sim ranges are about
the size-weighted values plus or minus 4 points, which covers the run-to-run noise of a 2,000-game calibration.

**Recorded, not calibrated:**

- **Draw rate: 24.4%** size-weighted, per-event middle half 18.0–30.2%. Tournament draws are mostly intentional
  draws and round time limits, which the sim doesn't have. The sim's equivalent, games with no winner by the turn
  cap, should stay rare. It is reported but not tuned to 24%.
- **Win turn:** no public dataset gives the turn cEDH games end on. Speed is calibrated through the seat edge above
  and checked against each deck's primer (the turn it aims to win on; Stage 2).
- **Win routes:** come from the primers in Stage 2 (which lines each deck wins with, and how often a primer
  expects each). Added here when the specs are done.

## Deck-level targets

Six-month EDHTop16 numbers for the six Tier 6 commanders. "Decisive share" is the game win rate divided by
(1 − 24.4%), the field's draw rate.

| Deck | Entries | Meta share | Top-cut conversion | Game win rate | Decisive share |
| --- | --- | --- | --- | --- | --- |
| Kraum & Tymna | 1,273 | 7.5% | 23.8% | 21.1% | 27.9% |
| Kinnan | 1,168 | 6.9% | 19.2% | 20.7% | 27.4% |
| Rograkh & Thrasios | 861 | 5.1% | 20.6% | 19.4% | 25.7% |
| Rograkh & Silas | 751 | 4.4% | 16.1% | 18.1% | 23.9% |
| Sisay | 647 | 3.8% | 24.3% | 20.8% | 27.5% |
| Dack Fayden | 32 | 0.3% | 53.1% | 37.2% | 49.2% |

In real events these decks face the whole field, not only each other. Within Tier 6 the target is therefore
balance rather than the exact values:

- each deck wins **18–32%** of the decisive games it plays in, in Tier 6 pods (an even share is 25%). Dack Fayden is held to the same range: its 49% comes from only 32 entries in its first two weeks;
- the gap between the best and worst deck is **8 points or less** (the real gap is 4 points);
- **Speed (JD, 2026-10-19):** median win round about 7 for the table (real cEDH game length is thin data, about turn 7). Each deck's median win round is in the Tier 6 report.
- Kinnan is held to the same band and gap as the others (JD, 2026-10-18, late): its earlier lead came from a modeling error (Kinnan + Basalt counted as a win without a mana sink, fixed in #100), so the earlier "Kinnan may stay the strongest" exception is withdrawn;
- no deck wins more than half its games through one line the primer calls a backup.

## Godo reference

Godo, Bandit Warlord in the same window: **78 entries, 21.4% game win rate (about 28% decisive), 28.2% top-cut
conversion**; over one year: 144 entries, 21.2%, 26.4%. The sample is small. Godo faces the whole field in real
events and only top decks in Tier 6, so a lower sim result is expected. A sim result far outside **15–35% of
decisive games** points to a modeling problem to find before trusting any Godo number.

## The six decks

| Deck | File | Way of winning | Source list |
| --- | --- | --- | --- |
| Kraum & Tymna | `kraum-tymna-blue-breach-midrange.md` | Blue midrange: counters first, Oracle or Breach | Breach the Bay 2, #2 of 429, 2026-08-22 |
| Kinnan | `kinnan-bonder-mana-combo.md` | Mana combo with creatures | SIEGE cEDH 10K, #1 of 308, 2026-06-13 |
| Rograkh & Silas | `rograkh-silas-turbo-breach.md` | Turbo Oracle and Breach | CCS Invitational Qualifier #2, #1 of 125, 2026-06-13 |
| Rograkh & Thrasios | `rograkh-thrasios-creature-combo.md` | Green creature-engine combo | Pittsburgh 2026, #1 of 195, 2026-07-11 |
| Sisay | `sisay-legends-toolbox.md` | Taxing legends toolbox | The Cookout 2026, #2 of 238, 2026-08-29 |
| Dack Fayden | `dack-fayden-white-stax.md` | Stax (added at JD's request 2026-10-17) | Baked Ziti 4, #1 of 119, 2026-10-03 |

Among the established decks no dedicated stax deck reached the top of the meta in the window (Derevi had 4 top-4
lists, Tivit 1). Dack Fayden, released 2026-10-02, is a true stax deck and fills that slot. Its numbers come from its
first two weeks only, and a new deck's early results often fall once the field adapts, so its deck-level target is
the same balance range as the others.

All six lists were checked against Scryfall on 2026-10-17: 100 cards, singleton, within colour identity, all
Commander-legal. 158 of their 329 unique cards are not yet in the card database (185 with Godo's).
