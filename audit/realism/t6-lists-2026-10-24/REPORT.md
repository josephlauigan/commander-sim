# Why the Tier 6 table is slow: what it is not, 2026-10-24

The sim's Tier 6 table wins at a median round of 9-10 and gives seat 1 about 26% of decisive games; real cEDH wins
around round 7 and gives seat 1 32.5% (`data/realism/t6.toml`). Real games with these six commanders last 6-7 rounds
on average (Playgroup: Kinnan 6.0, Rograkh 6.6, Tymna 6.7, Thrasios 6.6, Kraum 7.0, Sisay 7.2); the sim's last 10.3.
This report tests five explanations. **None of them is the cause.** All runs: loose profile, adaptive AI, seeds from
500000.

| Explanation tested | How | Seat 1 | Median win round | Files |
| --- | --- | --- | --- | --- |
| (baseline, lists as on main) | `--realism --pool t6 --games 8200` | 27.0% | 10 | `before-adaptive-8200.txt` |
| The AI's knobs | each knob at both ends of its range, 2,000 games each | 25.3-28.0% | 9-10 | `../knobs/README.md` |
| The deck lists | aligned with 48 real winning lists (below), 8,200 games | 26.0% | 10 | `after-adaptive-8200.txt` |
| Dack Fayden's stax | the table without Dack (a throwaway worktree), 8,200 games | 26.4% | 10 | `no-dack-adaptive-8200.txt` |
| Fast mana held | Mox Diamond's Tier 6 land rule 3 -> 2 -> 1 (knob `t6_moxd_lands`), 8,200 games | 26.2% | 10 | `moxd*-8200.txt` |
| Mulligans | real players of these commanders take 0.69-0.96 a game (Playgroup); the sim 0.63-0.76 | - | - | `calibrate-within-adaptive-2000.txt` |

## The deck lists are already real

Each Tier 6 commander's 8 most recent EDHTop16 tournament wins (April-October 2026; Dack Fayden's best are 2nd-7th
places, it has 4 top-4 finishes in a year) were compared with the pool list, using EDHTop16's per-commander play
rates. The "real core" is the cards in 6 or more of the 8 lists.

| Pool deck | Real core | In the pool list | Pool cards in none of the 8 lists |
| --- | --- | --- | --- |
| Kraum/Tymna | 87 | 86 | 2 |
| Rograkh/Thrasios | 82 | 79 | 0 |
| Rograkh/Silas | 82 | 78 | 1 |
| Sisay | 75 | 71 | 0 |
| Dack Fayden | 55 | 50 | 0 |
| Kinnan | 51 | 49 | 1 |

The 15 missing core cards went in (decklists/pool/CHANGELOG.md, 2026-10-24) and the table did not change.

## Where the time goes (calibrate-within report)

- Each player goes for a combo about 0.25 times a game, and 30-47% of tries are stopped. Lines are "one tutor away" at
  a median round of 6-7 but become ready in only 18-45% of games. The decks go as soon as a line is ready (Kraum/Tymna
  is the exception: it held a ready line in 153 of 588 ready games).
- So the slowness is in how fast the decks turn tutors and draws into a finished line. That is the AI and the decks'
  plans (Phases 1 and 4), not the knobs, the lists, one stax deck, or mulligans.
- Dack Fayden is the sim's strongest deck (35-36%), the weakest in real play. Its removal count is high: 3,686 removals
  in 1,345 games, mostly Fiend Hunter, All-Fates Stalker and an activated ability, 1,001 of them on Kinnan. Both cards
  return what they exiled when they leave (`impls/t6_dack.rs`, `oring_return`), so this is probably the flicker plan
  re-exiling the same creature each blink rather than a removal bug; why the deck is so strong is still open. It
  doesn't explain the speed (the table without Dack is as slow).
- "Stopped by no card named" (95-99 tries for Kraum/Tymna and Rograkh/Silas) is mostly a combo whose step returned
  false (Ad Nauseam that resolves but doesn't reach a win that turn), not an opponent's answer.

## Next leads

1. Tutor and draw conversion (the main lead): trace games where a deck is one tutor away for several rounds and see
   what it does instead (which tutor it holds, what it fetches).
2. Why Dack Fayden wins 35% in the sim and rarely in real play (its first-flip win, 76% of its wins, and its flicker
   plan), before any Dack-specific fitting.
3. Kraum/Tymna's held lines (the "stop (hold mana)" choice with a ready line).

## Refused plays (added later the same night)

A log line now records each play the AI chooses that `act::perform` can't make (`ai/brain.rs`). In 8 traced Tier 6
games there are 132, all in the six games with Dack Fayden seated (none in the two without; 63 in the game where Dack
resolved two tax pieces). They are Dack's taxes (Damping Sphere, Thorn of Amethyst, Defense Grid) that the option
list doesn't price in. The engine is right to refuse them, and the table without Dack is as slow, so they are not
the speed gap.
