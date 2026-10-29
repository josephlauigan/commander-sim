# AI foundations step 3: Tier 6 card and mana flow (2026-10-28)

Branch `ai-foundations`. Brief: `rust/agents/ai-t6-flow.md`. Raw output: `raw/step3/`:
- `base/`: the step 2 play with the flow counters (identical games: checked against `raw/step2/goldfish-on.txt` and
  the step 2 Tier 6 scorecard);
- `A/`, `B/`: after each fix; `C-wheels-reverted/`: a third fix tried and reverted;
- `full-base/`, `full-final/`: the full set, t0-t5 realism and all-decks.

## Summary

- **Flow is measured now** (`fl|...` counters, stats only; the report prints after `--realism` for Tier 6 and in
  each goldfish deck report). By the end of own turn 2/3/4 the Tier 6 decks have seen 8.8-9.2 / 10.0-10.8 /
  11.2-12.8 cards, against 9/10/11 from the draw step alone.
- **The engines work once they are out.** Rhystic Study draws 2.8-3.2 cards a turn cycle, with 4.7-5.1 tax triggers
  and about 40% paid. Mystic Remora draws 2.6-2.9 (about 7% paid), Esper Sentinel 1.5-2 and Necropotence about 4.
  These match the hand reference. The card models aren't the gap.
- **They come out too seldom and too late.** Tymna (Kraum/Tymna's engine commander) was out by the end of turn 4
  in 38% of games, against about 90% for a pilot. 25-65% of the Rhystic Studies a deck had seen were still uncast
  at turn 4. The cause is the AI: the look-ahead chose "stop" over the heuristic's 95-100% choice of the engine,
  turn after turn.
- **Two fixes, both AI (Tier 6 only):**
  - A: a card engine or commander the heuristic rates best is cast before the look-ahead;
  - B: the same for mana permanents.

  Kraum/Tymna sees 1.0 more card by turn 6 and 1.0 more by turn 8, and its win share rose 26.3% → 29.6%. A third
  fix (wheels by the cards they net) changed nothing and was reverted.
- **The seat edge and the spread did not move.**

  | | seats | spread | median round |
  | --- | --- | --- | --- |
  | before | 29.4 / 26.6 / 22.5 / 21.5% | 11.3 | 8 |
  | after A + B | 29.7 / 26.3 / 21.9 / 22.1% | 11.6 | 7 |
  | target | 31.8 / 27.3 / 22.1 / 18.8% | 0-8 | 6-8 |

  The median round reached the target's 7.
- **What is left isn't flow** (see "What is left"). With the counters, the remaining gaps per deck are 0.5-2 cards by
  turn 4: the hand-size and colour limits of the lists, and opponents who cast about one spell a turn. The seat edge
  needs most of the table to threaten a win by turns 3-5. A card or two by turn 4 doesn't make that race. The
  limits are the decks' kill speed (colours and draws, step 2) and the table's answers to the first attempt.

## Part 1: the flow, measured

### How it is counted (`sim_core::t6stats`, flow section; `sim-cli/src/flow.rs`)

Per Tier 6 player and own turn 1-8:
- **Cards drawn, by source.** Each player keeps the source of the effect resolving for it (`Player::flow_src`): a
  spell (`cast::resolve`), a trigger (`trigger_window`) or an activated ability (`ability_window`). The draw step is
  its own class.
  - Classes: engine (Rhystic Study, Mystic Remora, Esper Sentinel, Sylvan Library, Necropotence, The One Ring, Ad
    Nauseam); wheel; commander (and Dack); cantrip (any other spell); other permanents.
  - A draw counts toward the own turn on which the card can first be used.
  - Necropotence's and Ad Nauseam's cards count as draws of their source. Sylvan Library's put-back cards don't count.
- **Cards seen:** the distinct cards that reached the hand (the kept hand, draws, tutors and engines), by the end of
  each own turn. "Tutors and others" is the new cards seen less the new draws.
- **Casts:** spells cast on the own turn, on the other turns of its cycle, and those countered.
- **Mana:** available at the main phase after the land drop; spent, own turn and others' turns.
- **Life paid** by card; **land drops** made and missed, with or without a land in hand.
- **Engines:** turn cycles out, cards per cycle out, and tax triggers with the share paid. A funnel gives, by turn,
  the share of games where each engine has been seen, cast and is out. For an engine seen but never cast, it
  records where the card is at the end of turn 4.

### The Tier 6 table before (`base/realism-t6.txt`, 4,000 games)

| deck | seen by T2 / T3 / T4 | drawn T3 / T4 (engines + commanders) | spells cast own turn T2-T4 | mana spent / available T3 | land drop missed T3 (no land in hand) |
| --- | --- | --- | --- | --- | --- |
| Dack | 8.8 / 10.0 / 11.2 | 1.19 / 1.24 (0.02 / 0.07) | 1.1-1.2 | 2.95 / 4.10 | 22% |
| Kinnan | 8.8 / 10.2 / 11.8 | 1.22 / 1.58 (0.14 / 0.36) | 1.1-1.3 | 3.50 / 4.25 | 33% |
| Kraum/Tymna | 9.1 / 10.8 / 12.8 | 1.61 / 2.00 (0.54 / 0.91) | 0.9-1.1 | 2.30 / 3.49 | 28% |
| Rograkh/Silas | 9.2 / 10.7 / 12.4 | 1.43 / 1.59 (0.36 / 0.53, wheels 0.1) | 1.1-1.5 | 2.02 / 3.40 | 36% |
| Rograkh/Thrasios | 9.0 / 10.3 / 11.9 | 1.22 / 1.45 (0.12 / 0.35) | 1.1 | 2.63 / 3.59 | 22% |
| Sisay | 9.0 / 10.3 / 11.6 | 1.23 / 1.34 (0.15 / 0.26) | 1.0-1.3 | 3.27 / 3.75 | 24% |

- Cards in hand at the main phase stay at about 5 from turn 3 on (Dack about 4).
- Spells cast on the other players' turns: 0.1-0.4 a cycle. Countered: about 0.1 a cycle.
- Engines while out (turns 2-8):
  - Rhystic Study: 2.8-3.2 cards a cycle; 4.7-5.1 tax triggers, 39-46% paid.
  - Mystic Remora: 2.6-2.9 cards; 2.7-3.1 triggers, 6-9% paid.
  - Esper Sentinel: 1.5-2.0 cards; 2.1-2.9 triggers, 28-43% paid.
  - Necropotence: about 4; The One Ring: about 2.
  - Kraum: 1.2; Tymna: 0.95; Thrasios: 0.2-0.4 (the {4} a dig is spare only late).
- Engines out few cycles: Rhystic Study 0.14-0.30 cycles a game over turns 2-8, Tymna 1.46 of about 5.9 turn starts.
- The goldfish (`base/goldfish.txt`) shows the same shape with the tax engines blank, since the inert seats never
  cast.

Engine funnel, share of games by the end of turn 4 (seen / cast / out), 1,000-game diagnostic
(`base/diag-t6-1000.txt`):

| engine | Kinnan | Kraum/Tymna | Rograkh/Silas | Rograkh/Thrasios | Sisay |
| --- | --- | --- | --- | --- | --- |
| Rhystic Study | 11 / 6 / 6 | 12 / 7 / 5 | 14 / 9 / 8 | 11 / 6 / 4 | 10 / 4 / 3 |
| Mystic Remora | 14 / 10 / 7 | 15 / 11 / 8 | 12 / 9 / 7 | 11 / 8 / 5 | 10 / 8 / 6 |
| Esper Sentinel | | 16 / 12 / 10 | | | 11 / 7 / 7 |
| Necropotence | | 12 / 2 / 1 | 11 / 3 / 2 | | |
| commander engine | Thrasios (card) 18 / 9 / 14 | Tymna 100 / 47 / 38 | | Thrasios 100 / 92 / 78 | |
| wheels | | | Timetwister 10 / 4, Wheel of Fortune 12 / 0, Windfall 11 / 1 | | |

Most seen, never-cast engines at turn 4 are still in hand with the mana to cast them (`ez|`: Rhystic Study "hand,
payable" 15-29 games of each 1,000-game deck sample). They aren't pitched or discarded (exile or graveyard 0-6).

### The reference, by hand

The arithmetic is the reference: there is no public dataset.

**The draws:** 7 cards kept (6.9 with mulligans) plus one draw a turn, from turn 1 in multiplayer: 9 / 10 / 11 by
the end of turns 2 / 3 / 4.

**When an engine is out.** A given singleton card has been seen by the end of turn T with probability
(7 + T) / 99: 0.081, 0.091, 0.101, 0.111 for T = 1-4. A pilot casts it the turn it is seen when the mana allows:
- one-mana engines (Remora, Sentinel) at once (90%);
- three mana: 10% on turn 1, 50% on turn 2, 90% on turn 3;
- BBB (Necropotence) 30% on turn 2, 60% on turn 3.

Its cards then count from the next cycle.

**What real opponents do.** Each opponent casts about 2 spells on its own turns 1-4 and about 0.5 on the others'
turns, so 2.5 spells a cycle, 7.5 at a three-opponent table; about 80% are noncreature. On the cards per cycle:
- **Rhystic Study:** opponents pay {1} for about 60% of triggers when they can spare it: 7.5 × 0.4 = 3.0 cards a
  cycle (range 1.5-4.5).
- **Mystic Remora:** {4} is almost never paid early, and opponents hold about half their noncreature spells
  around it: 6 × 0.5 = 3.0.
- **Esper Sentinel:** one trigger per opponent's turn with a noncreature spell, about 3.6 a cycle, about 60% unpaid:
  about 1.5.
- **The rest:**
  - Necropotence: about 4 a cycle (8 in hand at the end step, life above a floor of 10-15);
  - Tymna: cast on turn 2 (50%) or 3 (90%), then 1.2 cards a turn from the next (1 to 1.5 opponents hit);
  - Kraum: 1.5 a cycle (cast turn 4-5);
  - Faerie Mastermind: 0.7; Thrasios: 0.5 once the {4} is spare;
  - a wheel: about 5 net cards, cast in half the games where it is seen by turn 3 in a turbo list.

**The reference by deck** (cards seen by the end of turn 2 / 3 / 4; engines by turn 4 in brackets):

| deck | engines in the list | reference | sim before | sim after A+B | gap after (T4) |
| --- | --- | --- | --- | --- | --- |
| Kraum/Tymna | Remora 0.25 cycles × 3.0 = 0.74; Sentinel 0.25 × 1.5 = 0.37; Rhystic 0.15 × 3.0 = 0.44; Necro 0.09 × 4 = 0.35; Mastermind 0.16; Tymna 1.5; Kraum 0.2; tutors 0.4 | 9.4 / 11.8 / 15.2 [3.8] | 9.1 / 10.8 / 12.8 | 9.1 / 10.8 / 13.1 | 2.1 |
| Kinnan | Remora 0.74, Rhystic 0.44, Sylvan 0.06, One Ring 0.05, Mastermind 0.16; tutors 0.3 | 9.3 / 10.9 / 12.8 [1.5] | 8.8 / 10.2 / 11.8 | 8.8 / 10.2 / 11.9 | 0.9 |
| Rograkh/Silas | Remora 0.74, Rhystic 0.44, Necro (rituals) 0.68, wheels 0.75, Gitaxian Probe 0.1; tutors 0.4 | 9.5 / 11.6 / 14.1 [2.7] | 9.2 / 10.7 / 12.4 | 9.1 / 10.5 / 12.2 | 1.9 |
| Rograkh/Thrasios | Remora 0.74, Rhystic 0.44, Mastermind 0.16, Thrasios 0.8; tutors 0.3 | 9.3 / 11.1 / 13.4 [2.1] | 9.0 / 10.3 / 11.9 | 9.0 / 10.3 / 11.9 | 1.5 |
| Sisay | Remora 0.74, Rhystic 0.44, Sentinel 0.37; tutors 0.1 | 9.3 / 10.8 / 12.6 [1.6] | 9.0 / 10.3 / 11.6 | 9.0 / 10.2 / 11.6 | 1.0 |
| Dack | The One Ring 0.05 (Scroll Rack is selection) | 9.0 / 10.0 / 11.1 | 8.8 / 10.0 / 11.2 | 8.8 / 10.0 / 11.2 | 0 |

For the mana, real cEDH openers cast their fast mana as it comes, so a pilot has 2.5-3.5 mana on turn 2 and spends
most of it. The sim's turn 2 mana (2.5-2.7 available at the main phase, 1.7-2.5 spent) is at the low end. The low
spend is the same look-ahead "stop" (fix B). Land drops: the sim misses the turn 3 drop in 22-36% of games, nearly
always with no land in hand. With 23-28 lands in 99 that is what the lists give, and the reference is the same.

### Under-delivering engines and card classes (before the fixes; gap by turn 4)

| engine or class | sim | reference | gap (cards by T4) | cause (Part 2) |
| --- | --- | --- | --- | --- |
| Tymna (Kraum/Tymna) | out by T4 in 38% of games | ~90% | ~0.8 | (b) the look-ahead's "stop" |
| Rhystic Study (all but Dack) | cast by T4 in 36-73% of the games that saw it | ~90% | 0.1-0.3 | (b) the same |
| Necropotence (Kraum/Tymna, Rograkh/Silas) | cast by T4 in 17-27% of the games that saw it | 60-90% | 0.3-0.5 | the list: BBB in a four-colour deck without the ritual in hand |
| wheels (Rograkh/Silas) | 0-4% of the games that saw one cast it by T4 | ~50% | ~0.7 | (b) the empty-hand rule; reverted fix C: not the binding limit |
| Kraum's trigger | 1.2 a cycle | 1.5 | small | (c) opponents cast about 1.1 spells a turn |
| spells cast own turn | 0.9-1.3 | about 2 | (no card gap) | (b) look-ahead "stop", and the hand of held counters |

## Part 2: why, per engine

- **The card models are right.** Oracle text checked in `data/scryfall_cache.json`:
  - Rhystic Study: the {1} tax on every opponent's spell (`cast::on_cast`, `rhystic_unpaid`: pay when it can spare
    the mana for its best spell);
  - Mystic Remora: cumulative upkeep {1}, the noncreature trigger, the {4} tax (`rules::remora`, `remora_age`);
  - Esper Sentinel: the first noncreature spell each turn, X = its power (`common::sentinel`);
  - Necropotence: skip the draw step, pay 1 life a card (`decks::necro_pay`; the cards come to hand in the end step,
    as Oracle's "next end step" does when it is paid on its own turn);
  - Sylvan Library, The One Ring, Kraum, Tymna, Thrasios, Timetwister, Windfall, Wheel of Fortune: as Oracle.

  Their rates while out are the reference's. Kraum/Tymna's and the others' cards are all Full or Full-auto in
  `cardgen audit`. No (a) or (d) causes.
- **(b) the AI: the look-ahead votes development down.** Traced Tier 6 games
  (`--calibrate within --pool t6 --trace 0..11`):
  - "Tymna the Weaver 100%, stop (hold mana) 0% -> stop (hold mana) [search]" on Kraum/Tymna's turns 2, 3 and 4;
  - "Rhystic Study 100%, stop 0% -> stop [search]";
  - "Chrome Mox 97% -> stop (hold mana)";
  - Mox Diamond, Rograkh, Silas and Remora the same.

  The look-ahead plays to the end of the next turn, so an engine's cards come at most one cycle into its playouts,
  and every cycle after is outside it. Its position score also values a card in hand at 0.9 and a rock near 0
  (REPORT defect F). The step 1 `must_option` already forces plays at priority 90+, but Tymna (86) and Rhystic (88)
  sit just below, and it is off whenever the plan asks to hold mana up. The T5 Tymna decks had the same problem
  fixed for their commanders (`tymna_partners::commander_first`). The Tier 6 keys were never included.
- **(c) the opponents:**
  - They feed Rhystic Study as the reference assumes (about 5 triggers a cycle, about 40% paid).
  - They cast about 1.1 spells on their own turns against a real 2, which starves Kraum's second-spell trigger. That
    is the same look-ahead "stop", plus hands of held counters (Kraum/Tymna's plan holds every counter at priority 0
    from turn 3). Fixes A and B lift it a little.
- **Necropotence:** BBB in Kraum/Tymna's and Rograkh/Silas's four-colour mana. The AI casts it when it can (its
  priority is 45-80 once it buys 2+ cards). The "payable" flag in the turn-4 fate counts mana, not colours. This is
  the list, not the AI.
- **Wheels:** the empty-hand rule holds them (`wheel_prio`: two cards in hand at most, and for Wheel of Fortune no
  opponent above five). Fix C replaced it at Tier 6 by "nets 3+ cards and 2 more than the opponent it helps most". It
  changed nothing (Wheel of Fortune cast in 1% of games by turn 8, Timetwister 13% from 9%, Rograkh/Silas 20.0%
  from 20.3%): the hand stays full of counters and pieces, and a wheel throws those away. Reverted.

## Part 3: the fixes, each measured

`--realism --pool t6 --games 4000` and `--goldfish t6,scion --games 2000 --seed 500000` after each.

| run | seat 1 / 2 / 3 / 4 | spread | median round | Dack / Kinnan / Kraum / Rog-Silas / Rog-Thrasios / Sisay |
| --- | --- | --- | --- | --- |
| before (base) | 29.4 / 26.6 / 22.5 / 21.5 | 11.3 | 8 | 23.1 / 24.9 / 26.3 / 22.2 / 21.2 / 32.5 |
| A: engines and commanders first (eefe4e34) | 29.1 / 25.5 / 23.4 / 22.0 | 11.8 | 7 | 24.0 / 24.6 / 29.7 / 20.3 / 19.9 / 31.7 |
| B: mana permanents too (40bc2a8e) | 29.7 / 26.3 / 21.9 / 22.1 | 11.6 | 7 | 24.2 / 24.9 / 29.6 / 20.3 / 19.8 / 31.4 |
| C: wheels by net cards (reverted) | 29.6 / 25.3 / 23.4 / 21.6 | 12.9 | 7 | 25.0 / 24.0 / 29.5 / 20.0 / 19.4 / 32.3 |

At 4,000 games a seat share's 95% interval is about ±1.4 points. No seat moved beyond it.

Flow, before → after B:
- **Kraum/Tymna:**
  - cards seen by turns 4 / 6 / 8: 12.8 / 17.8 / 23.3 → 13.1 / 18.7 / 24.3;
  - Tymna cast by turn 3 / 4: 33 / 47% → 60 / 75%; out at turn 4: 38 → 63%;
  - Rhystic Study cast by turn 4 (of the games that saw it): 7 of 12 → 10 of 14.
- **Kinnan:** Rhystic Study cast by turn 4: 6 of 11 → 8 of 11; seen by turn 6: 15.7 → 16.1.
- **Mana spent on own turn 4** (before → B): Kinnan 4.42 → 4.84, Kraum/Tymna 2.90 → 3.13, Rograkh/Silas 2.38 → 2.56,
  Sisay 4.44 → 4.55.
- **The other decks' cards seen moved by 0.2 or less.** Their engines were already cast once the heuristic offered
  them, or are held by their plans (Sisay's Sisay fetches at priority 91 come before Rhystic Study at 72).

**Goldfish** (2,000 games), median / mean kill turn and kills by turn 3, before → after B:

| deck | before | after B |
| --- | --- | --- |
| scion | 9 / 9.6, 5.2% | 9 / 9.8, 5.0% |
| Kraum/Tymna | 8 / 8.8, 3.9% | 8 / 8.8, 2.9% |
| Kinnan | 8 / 8.4, 2.5% | 8 / 8.4, 2.4% |
| Rograkh/Silas | 9 / 9.1, 6.8% | 9 / 9.1, 5.8% |
| Rograkh/Thrasios | 9 / 9.1, 0.1% | 9 / 9.2, 0.1% |
| Sisay | 7 / 7.3, 0.1% | 7 / 7.4, 0.1% |
| Dack | 10 / 9.6, 9.7% | 10 / 9.5, 9.7% |

The goldfish's inert seats never cast, so Rhystic Study, Remora and Sentinel draw nothing there. Casting them first
costs a turn-3 kill in about 1% of Kraum/Tymna's and Rograkh/Silas's goldfish games, and scion 0.2 of a turn of
mean. At a table those engines draw about 3 cards a cycle.

Tests: `tests/ai_flow.rs` (5): engine first, passing rated higher, a non-engine best play, the commander and other
tiers, the mana permanent but not a one-shot, and the draw credited to its source. Also the `sim-cli` flow report
unit test.

## The full set (t0-t6 realism, all-decks), before → after

(filled in from `full-base/` and `full-final/` below)

## What is left, and why it isn't flow

- **After A and B the flow gaps against the reference are 0-2 cards by turn 4**, and they have causes outside the AI's
  card and mana use:
  - Necropotence's BBB in four-colour mana;
  - wheels that would throw away a hand of held counters;
  - opponents who hold their counters, so they cast about one spell a turn (the Tier 6 plans' "counters held from
    turn 3").
- **More cards didn't make a race.** Kraum/Tymna gained a card by turn 6 and 3.4 points of win share. The table's
  seat shares didn't move. Step 2 showed why: the decks' kill turns are set by draws and colours (a second blue
  source for Thassa's Oracle), not by card volume. The seat-edge audit showed that first-attempt success is
  76-86% whatever the seat. The seat edge needs most of the table to threaten a win by turns 3-5. Here 72% of
  player-games never have a line ready.
- **So the remaining seat-edge gap is the decks' kill speed and the table's answers, not flow.** In order:
  1. **Line readiness by turn 3-5:** colours, the lists' land counts (23-28 lands), pieces in the library. The
     goldfish's legal best case is turn 8 for scion.
  2. **The tournament clock:** about 1 point of seat 1. Real decisive games leave out the long ones that hit time.
  3. **The table's threat assessment:** who gets countered or removed first (Kinnan drew about 1.75 removals a
     game before its threat value was lowered).
- **The spread (11.6) is Sisay (31-32%) against the two Rograkh decks (about 20%).** Fix A widened it a little by
  lifting Kraum/Tymna, which is the one deck whose engine was being held. Rograkh/Thrasios's and Rograkh/Silas's
  shortfall isn't in their flow: they see as many cards as Kinnan and Sisay.

## Decisions (made without JD; the conservative call, with the alternative)

1. **Fix A and B skip the look-ahead** when the heuristic's best play is an engine, a commander or a mana permanent
   (Tier 6 only), as `commander_first` does for the T5 Tymna decks. The alternative was to give the look-ahead's
   position score an engine's cards past the horizon (a valuation term with a cards-a-cycle constant). That is the
   bonus-constant kind the brief rules out, and it would reach every tier's search.
2. **Fix C (wheels) was reverted:** it changed nothing measurable. A rule that does nothing adds nothing. The
   alternative was to keep it as "more correct". It is in `raw/step3/C-wheels-reverted/`.
3. **No change to opponents' Rhystic tax or spell counts.** The tax rule pays when the payer can spare the mana for
   its best spell, and its 40% paid sits inside the reference's range. Opponents casting fewer spells comes from
   their plans' held counters, which are deck-plan choices for JD and the deck agents. The alternative was a
   "cast more" rule for the table, which would be a style knob, and the brief rules those out.
4. **The goldfish stays inert** (step 1's decision). Fix A costs scion 0.2 of a turn of goldfish mean, because the
   engines it casts are blanks there.
5. **The baseline for the full set is the instrumented binary** (step 2 play plus stats). The step 2 binary was lost
   with a session restart, and the instrumented one plays identical games (checked on the goldfish and the Tier 6
   scorecard).
6. **The reference's assumptions are stated as ranges** where they matter: Rhystic paid 40-75%, opponents' spells
   1.8-3.2 a cycle. The comparison holds across them: the engine rates are inside the range, and the gaps are in
   when the engines come out.

## Decisions for JD

1. **Accept that the seat edge isn't a flow problem.** The next lever would be the decks' kill speed (colours, land
   counts, Thassa's Oracle's second blue source), the tournament clock in the realism report (count timeouts and
   long games as draws), or the table's threat assessment. Which, if any?
2. **The spread:** the two Rograkh decks are at about 20%. Their plans hold their cards, not their flow. Is it worth
   a deck-agent pass on their plans, or is 0-8 too tight a band for six lists?

## Status (main session, 2026-10-28 15:10)

- **A and B are provisional workarounds.** They make Tier 6 cast an engine, commander or mana permanent *before* the
  look-ahead, because the look-ahead kept choosing "stop" over them. Why it did so was not diagnosed. The likely
  causes, from `architecture-review.md`:
  1. `search::strength` values a permanent's future output at about 0 (a card in hand is 0.9);
  2. the horizon ends at the deck's next turn;
  3. the hold value and hold bonus favour passing with mana up.

  JD (2026-10-28): fix the evaluation at the cause instead, for every deck. A and B are to be removed once that fix
  covers them.
- **Full set:** realism t0-t5 is byte-identical between `full-base` and `full-final` (A and B are Tier 6 only). The
  all-decks comparison was stopped before it finished and was not run.
