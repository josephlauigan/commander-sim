# Tier 6 seat targets from TopDeck.gg Swiss rounds (2026-10-24)

Data courtesy of [TopDeck.gg](https://topdeck.gg) (API v2, read 2026-10-24 with JD's key).

## Why

The sim's seat 1 share (27.6%) misses the EDHTop16 target (32.5%). EDHTop16's report says "All Rounds" and gives no
method. Two worries:
- top cuts, where seats can follow standings, might inflate seat 1;
- a small random-seat study (TopDeck.gg article, 648 Swiss pods) showed 31.5 / 24.2 / 24.1 / 20.2.

So the target was rebuilt from raw Swiss pods.

## Method

1. `POST /v2/tournaments` with `game` = "Magic: The Gathering", `format` = "EDH", `rounds` = true, month by month
   from April to September 2026: 7159 events.
2. **Swiss rounds** are the rounds with a numeric label; top-cut rounds have labels such as "Top 8" and are left out.
   Kept: tables with `status` = "Completed" and four players.
3. **Draws** are `winner_id` = "Draw". Tables with no winner recorded are dropped.
4. **cEDH events** are those whose name contains "cEDH" or "competitive": 3071 events, 40464 pods, 24.1% draws.
5. **Seat** = the winner's position in `players`. The API doesn't document that order. The data shows it is the seat
   order:
   - the shares fall steadily from first to last;
   - round 1 (seated at random, before any standings) shows the same edge;
   - a random order would give 25% each.

## Results (decisive pods, 95% intervals)

| Subset | Pods | Seat 1 | Seat 2 | Seat 3 | Seat 4 |
| --- | --- | --- | --- | --- | --- |
| cEDH events, Swiss (**the new target**) | 29864 | 31.8 ± 0.5 | 27.3 ± 0.5 | 22.1 ± 0.5 | 18.8 ± 0.4 |
| cEDH events, round 1 only | 8502 | 31.8 ± 1.0 | 27.5 ± 0.9 | 22.5 ± 0.9 | 18.1 ± 0.8 |
| other EDH events, Swiss | 23345 | 31.3 | 26.4 | 22.8 | 19.4 |
| EDHTop16, all rounds (old target) | - | 32.5 | 26.9 | 22.7 | 17.9 |

April alone, all EDH events, round 1 against rounds 2+:

| Rounds | Seat 1 | Seat 2 | Seat 3 | Seat 4 |
| --- | --- | --- | --- | --- |
| round 1 | 34.9 | 26.8 | 21.5 | 16.8 |
| rounds 2+ | 32.3 | 27.0 | 22.2 | 18.5 |

## Conclusions

- **The seat edge is real and is not a seating artefact.** It holds in round 1 (random seats), in Swiss rounds
  without top cuts, and in non-cEDH EDH events.
- **EDHTop16's numbers are confirmed** within about a point. The 648-pod study was a small sample.
- **The new targets** (`data/realism/t6.toml`) are 31.8 / 27.3 / 22.1 / 18.8, each held to ± 1 as before.
- **The sim's 27.6% for seat 1 is a real gap**, about 4 points below. That is the readiness problem in
  `t6-seat-edge-2026-10-24.md`: too few players reach a combo line, too late, for turn order to decide games.

The raw downloads (about 515 MB) are not kept in the repository. The method above reproduces them with a TopDeck.gg
key.
