# Sauron audit: status (work in progress)

> **Delete this folder (`audit/sauron/`) once the audit is picked up and understood.** It is a hand-off note between
> computers, not part of the project. Nothing in the simulator depends on it.

Started 2026-10-04 on branch `sauron-audit-wip`, from `main` at `ea54a55` (after PR #30: the deck folders split into
`decklists/JD/`, `Avery/`, `Other/`).

## The question

Why isn't Sauron (`decklists/JD/sauron-grixis-amass.md`, key `sauron`) stronger? Ranked against JD's other decks it
is third of four:

| Deck | Fast AI, loose, 1,000 games/tier (5-tier average) | Recorded look-ahead average (`pool-results.md` 0d) |
| --- | --- | --- |
| Sephiroth | 47.0% | 38.1% |
| Y'shtola | 46.9% | 34.0% |
| Sauron | 28.5% | 22.9% |
| Veyran | 24.3% | 17.3% |

Sauron by tier (fast AI, loose): T1 29.6%, T2 33.4%, T3 23.2%, T4 26.9%, T5 29.2%. Roughly an even share
everywhere: steady, but never strong.

## What's been measured so far

All with the heuristic ("adaptive") AI, loose profile. Full report in `analyze_all_tiers_adaptive_loose.txt`
(`python3 -m commander_sim --deck sauron --pool all --games 1000 --ai adaptive --profile loose --analyze`).

1. **It dies early.** Eliminated in 65-76% of games, on average around round 10-12 (round 7.5 against Tier 5). At
   Tiers 1-3 it dies to creature decks' combat (Aurelia alone kills it in 23.5% of Tier 3 games). At Tiers 4-5 it
   dies to combo (Heliod, Kinnan, Urza, Winota).
2. **The combos almost never happen.** Sword + Aggravated Assault is attempted in 0.7-2.0% of games. The plan
   milestone ("Sword + Assault combo attempted") is reached in 3-8% of games. The Breach line wins well when it's
   cast (Underworld Breach: 63.5% win rate when cast at Tier 1), but Breach is cast in only 43% of the games it's
   drawn, and Brain Freeze in 16%.
3. **It's an Army deck that's only as good as its Army.** `army_stats.py`, 300 games each at T1/T3/T5:
   - Sauron lands at a median turn 6 (by turn 6 in about half of games; at T5 he's cast in only 57% of games).
   - The Army's peak power: median 10 (T1), 8 (T3), 3 (T5). Peak in wins is about 16-18, in losses 6-7.
   - Lands on turn 6: median 5. The six-mana commander is late.
   - Sauron leaves the battlefield about 1.2-1.3 times a game at T1/T3; Armies are lost 1.2-1.4 times a game.
4. **Who removes Sauron and the Army** (`removal_causes.py`, 600 games), per game:
   - Sauron: opponent spot removal 0.43, opponent wipe 0.18, combat 0.08, sacrificed 0.07, own wipe 0.06, other 0.12.
   - Army: opponent wipe 0.23, opponent spot removal 0.18, combat 0.17, sacrificed 0.13, own wipe 0.08, other 0.14.
   - So opponents' interaction is the main cause. The deck's own wipes (Blasphemous Act, Toxic Deluge, Nibelheim
     Aflame) kill its own Sauron or Army about 0.14 times a game. Small, but worth checking whether the AI's wipe
     timing accounts for its own commander and Army.
5. **The "sacrificed" and "other" removals were being traced when the audit paused** (`removal_chains.py`). The
   call chains show most of them come from opponents' ability-language effects (resolve_spell / flush_triggers →
   execute → die: edicts and the like), plus some from inside Sauron's own main phase (`main → go → die` and
   `main → <lambda> → go → die`). **The own-main-phase ones are the open lead:** check whether Sauron's AI is
   sacrificing its own Army or commander as a cost (Diabolic Intent's sacrifice, Deadly Dispute-style costs,
   `brain.spare_creature`), which the deck document says should never happen to the Army.

## Cards that sit in hand (from the analysis report, Tier 1)

Cast in under half of the games they're drawn: Brain Freeze 16%, Deepglow Skate 28%, Grave Titan 32%, Noxious
Gearhulk 35%, Kaervek 42%, Underworld Breach 43%, Sauron, the Necromancer 45% (plus counterspells, which are
reactive by design). Several are 5-6 mana in a deck whose commander also costs 6 and lands about turn 6. That points
at a curve / mana problem, a cast-priority problem, or both.

## Next steps (planned, not started)

1. Finish the own-main-phase sacrifice lead (item 5): find the cards behind `main → go → die` for the Army and
   Sauron, and fix the AI if it's feeding them to a cost.
2. Check the AI's own-wipe decisions: does `consider_wipe` / `wipe_options` weigh losing Sauron and the Army?
3. Check the priority of the stuck cards (`ais.sauron_prio`): Deepglow Skate, Underworld Breach, Brain Freeze,
   Kaervek. Is the AI never casting them, or never able to?
4. Mana: 35 lands plus seven rocks, Sauron at six mana, median five lands on turn 6. Measure how often Sauron is
   castable by turn 5 or 6, and compare with a paired `--swap` of a high-cost card for a land or rock.
5. Defence: it dies to combat at Tiers 1-3. Check blocking (does the Army block?) and whether the deck has enough
   cheap blockers or fog effects. Compare with how Y'shtola's pillowfort cards change her losses.
6. Confirm any change with a paired comparison, e.g.
   `python3 -m commander_sim --deck sauron --pool t3 --swap "Card Out=>Card In" --ai adaptive --profile loose`, then
   with the look-ahead AI before recommending it.

## Running the scripts

From the repository root (each takes a few minutes with the fast AI):

```
PYTHONPATH=. python3 audit/sauron/army_stats.py 300      # when Sauron lands, Army size, losses (T1, T3, T5)
PYTHONPATH=. python3 audit/sauron/removal_causes.py 200  # who removes Sauron and the Army
PYTHONPATH=. python3 audit/sauron/removal_chains.py      # call chains for the unexplained removals
```

The scripts monkeypatch `engine.enter` / `engine.leave` / `engine.tick` in-process; they don't change the
simulator.
