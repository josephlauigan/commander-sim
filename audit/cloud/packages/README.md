# Cloud: pinned upgrade packages (not applied, not tested)

JD, 2026-10-13: keep the Cloud deck as it is (the C4 list) for now. These two packages are parked for when JD wants to
push Cloud toward parity (about 25%) with Tiers 1 and 2. Both are built against the C4 list and use only fully
modeled cards (checked with the scratch audit on 2026-10-13). Prices are Scryfall's cheapest printing that day.

## Why
The C4 analysis (`audit/cloud/2026-10-13/c4/`, 480 games per tier) shows Cloud at 14.2% (T1) and 9.6% (T2), dead by
round 9-11, almost always to **combat**: Light-Paws (commander damage), Isshin, Tatyova, Lathril at T1; Brago, Kaalia,
Lord Windgrace, Sythis, Meren at T2. At T2 Cloud is also removed 1.3 times a game. Two rounds of AI fixes moved the
win rate by under a point (`audit/cloud/2026-10-13/aifix2/`), so the remaining gap is the list, not the AI.

## Package D: defense against combat races (`d-defense.swaps`, 8 swaps, about $23)
| Out | In | Price |
|---|---|---|
| Stormfront Riders | Ghostly Prison | $6.62 |
| Beatrix, Loyal General | Norn's Annex | $5.69 |
| Unyielding Gatekeeper | Archangel of Tithes | $4.53 |
| Ennis, Debate Moderator | Windborn Muse | $0.81 |
| Explorer's Scope | Baird, Steward of Argive | $0.32 |
| White Mage's Staff | Wrath of God | $3.97 |
| Unfinished Business | Day of Judgment | $0.92 |
| Rootborn Defenses | Fumigate | $0.25 |

Attack taxes slow the creature decks; the wipes leave Cloud's Equipment on the battlefield and Cloud comes back from
the command zone. (The earlier Cloud, Planet's Champion deck measured Ghostly Prison +1.2 and Windborn Muse +1.0.)

## Package R: removal and Cloud protection (`r-removal.swaps`, 6 swaps, about $24)
| Out | In | Price |
|---|---|---|
| Adelbert Steiner | Swords to Plowshares | $0.98 |
| Jocasta, Automaton Avenger | Path to Exile | $0.84 |
| Secure the Scene | Council's Judgment | $1.09 |
| Recommission | Generous Gift | $0.97 |
| Nim Deathmantle | Mother of Runes | $11.76 |
| Helitrooper | Flawless Maneuver | $8.86 |

Cheaper options: Oblivion Ring ($0.29) or Banishing Light ($0.66) for Flawless Maneuver; drop Mother of Runes to save
$12.

## Test plan when resumed
Three paired runs at Tiers 1 and 2 only, 1,000 games per tier: D alone, R alone, D + R (concatenate the two files).
About 15 minutes each at 7 threads. Command per tier (run from the repo root, one `--swap` per line of the file):

```
commander-sim --deck cloud --pool t1 --profile loose --games 1000 --ai lookahead --jobs 7 --swap "OUT=>IN" ...
```

Neither package adds a Game Changer, so Cloud stays Bracket 3. If D + R together still fall well short of 25%, the
likely conclusion is that a 2/1 voltron commander can't reach T1/T2 parity in this pool.
