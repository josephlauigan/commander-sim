# Sheoldred, the Apocalypse — findings (living file)

What the simulator has taught us about this deck: strengths, weaknesses, and what was tried and why it did or didn't
work. Updated after every run. Newest evidence first; numbers are look-ahead, loose profile, vs Sheoldred's matched
tiers (T3-T5) unless noted. Older tests: README.md in this folder.

## 2026-11-07: Screeching Soulbreaker => Reanimate (JD's choice, not simulated)
Reanimate ({B}, owned; it left the list for Animate Dead earlier today) comes back next to Animate Dead, so the
graveyard start has two reanimation spells: Entomb into either is a six-drop on turn 2. It takes a creature card from
any graveyard; you lose life equal to its mana value. Model status: Full. Screeching Soulbreaker ({2}{B} 1/4 flier,
drain 1 on attack) was the cut: win rate when cast -3.0 in the 11/07 analyze run, never tested.
Not measured, for the reason given for Animate Dead below: the AI has no Entomb-into-reanimation line and barely casts
Entomb (6-10% of the games drawn) or Yawgmoth's Will (4-5%), so a paired run would measure Reanimate only as a late
answer to a dead creature (it was cast in about 5.5% of games when it was in the list, winning 48% of those at t4,
+20). The deck loses a flying blocker and a drain creature. Check Reanimate's and Animate Dead's drawn / cast numbers
in the next `--analyze`; a real measure waits on the graveyard AI line (follow-up 1).

## 2026-11-07: Al Bhed Salvagers => Witch of the Moors (t3-t4, loose, 2,000 games per list per tier, look-ahead, paired)
Result: **t3 31.6% -> 32.5% (+0.9), t4 25.5% -> 26.9% (+1.4), noise ±1.6 per tier: about +1.2 over t3-t4, positive
both times but inside the noise.** **Adopted (JD's pick, 2026-11-07): Witch of the Moors is in the list.** Raw: `2026-11-07-witch/salvagers-to-witch.txt`.
Witch of the Moors (JD pulled it) was modeled for this test (Full, card code in sheoldred.rs): at your end step, if you
gained life this turn, each opponent sacrifices a creature and a creature card comes back from your graveyard to hand.
JD chose the cut (Al Bhed Salvagers, the lowest win rate when cast in the 11/07 analyze run, -4.2).
Why: the fit is real (Sheoldred's own draw trigger gains 2 a turn, and Blood Artist, Zulaport, Exquisite Blood, Vito
and Gray Merchant gain more, so the trigger is on most turns once Witch lands), but **Witch is stuck in hand in about
half the games it's drawn (stuck 48% at t3, 51% at t4; cast around round 6.4-6.9)** against Salvagers' 37% (round
5.0-5.9): a 5-drop competing with Sheoldred and the other late cards for mana (weakness 2). Where it lands, the games
look better (win rate when cast 42% / 39% against Salvagers' 33% / 32%: correlation, not causation). Damage +0.6 a game,
plan timing and the death causes unchanged. Good in this deck when cast; the question is the mana, not the card.

## 2026-11-07: Reanimate => Animate Dead (JD's choice, not simulated)
Animate Dead ({1}{B}, Aura; owned, from the retired Seph deck) replaces Reanimate ({B}, lose life equal to the mana
value). One more mana, no life paid; it still takes a creature card from any graveyard and returns legendaries (Persist
would not). The creature gets -1/-0 and is sacrificed if the Aura leaves, so enchantment removal (and Farewell) answers
it. Model status: Full (`rean=animate`). Not measured: Reanimate won 48% of the t4 games it was cast in (+20) but was
cast in only 5.5% of games, and the rest of the graveyard half (Entomb, Yawgmoth's Will) is barely played by the AI,
so a paired run would say little until that AI gap is fixed. Check its drawn / cast numbers in the next `--analyze`.

## Baseline 2026-11-07 (after package S as adopted; main 7c622f5f; look-ahead, loose, 10,000 games per tier, seed 500000)
t3 **30.6%** (29.7-31.5), t4 **28.2%** (27.3-29.0): **29.4% over t3, t4**, against 27.4% in Baseline B (t3 28.2%, t4
26.6%; 2,000 games per tier). +2.4 and +1.6: a change against Baseline B needs about ±2.1 per tier to be clear, so this
is a borderline gain, the first measure of package S as adopted (Diabolic Tutor kept, Darksteel Plate cut for Diabolic
Intent). The paired run of the earlier version (Diabolic Tutor out) was +1.9 / -0.3. The analyze run (2,000 games per
tier) is in `audit/baseline-2026-11-07/sheoldred-analyze.txt`. Raw: `audit/baseline-2026-11-07/sheoldred.txt`.

**Analyze (2,000 games per tier).** t3: eliminated in 70.5% of games (round 8.8), mostly by combat: Aurelia 12.8% (and 4.3%
commander damage), Marwyn 9.9% (and 8.1% combo), Tergrid 8.7%, Atraxa poison 4.3%. t4: 72.0% (round 7.8): Krenko 13.9% combat
and 5.3% triggers, Edgar Markov 10.1% and 4.4% combo, Chulane combo 8.0%, Yuriko triggers 6.0%. Package S cards: Massacre
Wurm wins 48% (t3) and 65% (t4, +37) of the games it is cast in, the go-wide answer it was picked for, but sits in hand
(cast 36% of the games drawn, t3); Diabolic Intent +15 (t3); Reanimate +20 (t4); Dreadhorde Invasion +0.1 (t4, neutral).
Entomb (cast 6-10% of the games drawn) and Yawgmoth's Will (4-5%) are still nearly never cast: the AI gap stays open.

## 2026-11-07: card-pile package S, adopted (paired swap run; t3, t4, look-ahead, loose, 1,500 games per list per tier, main b84ab9e6 with #292)
Raise Dead => Reanimate, Cast Away Doubt => Entomb, Diabolic Tutor => Diabolic Intent, Vanquish the Weak => Massacre Wurm,
Dark Deal => Yawgmoth's Will, Otherworld Atlas => Dreadhorde Invasion, Swamp => Barad-dûr (JD's spare-card pile,
`audit/jd-packages/2026-11-06-card-pile/`). **NO CLEAR EFFECT:** t3 28.2% -> 30.1% (+1.9, noise ±2.7), t4 27.5% -> 27.2%
(-0.3, ±2.7). Game shape unchanged: plan online 91.3% -> 91.5% (t3) and 87.5% -> 86.5% (t4), rounds survived, damage
and who eliminates it all inside noise.

Why nothing moved: the graveyard half is barely played. Entomb is cast in 6% / 9% of the games it is drawn (stuck in
hand 94% / 91%) and Yawgmoth's Will 8% / 4% (stuck 92% / 96%): the AI has no Entomb-into-Reanimate line for this deck
and gives Yawgmoth's Will priority 0 (the preflight warned). Reanimate is cast 47% / 34%. The cards that did play:
Dreadhorde Invasion 68% / 67% (round 5.0 / 4.3), Diabolic Intent 51% / 54%, Massacre Wurm 40% / 45%. What left was cast
more (Raise Dead 82-85%, Vanquish the Weak 81%) but won less when cast (Raise Dead 30% / 29% of the games it was cast in, Vanquish the Weak 26% / 23%); Dark Deal and Otherworld Atlas were
cast in only 14-17% of their games.

**Adopted by JD 2026-11-07 with one change, untested:** Diabolic Tutor stays and Diabolic Intent takes Darksteel Plate's
slot (Swiftfoot Boots is the protection now). Open: an Entomb / Reanimate / Yawgmoth's Will line for the AI, without
which three of the seven cards are close to dead in the sim. Raw:
`audit/jd-packages/2026-11-06-card-pile/runs/S-sheoldred.txt`.

## Baseline B 2026-11-06 (after #278-#284; main 63d783b5; look-ahead, loose, 2,000 games per tier, seed 500000)
t3 **28.2%** (26.3-30.3), t4 **26.6%** (24.7-28.5): **27.4% over t3, t4**, against 27.5% in the morning: no change. It includes her
Ad Nauseam fix (entry below) and the new Tier 3 Atraxa (#279). Raw: `audit/baseline-2026-11-06-b/sheoldred.txt`.

## 2026-11-06: Ad Nauseam held only with room for four cards (engine fix, branch t6-ai; T3-T4, loose, look-ahead, 2,000 per tier, seed 500000)
The AI kept mana up for Ad Nauseam even with a full hand, where it would never cast it (it casts Ad Nauseam only with room
for four cards). The Tier 6 decks already had the fix (C1b: stops held for Ad Nauseam 340 -> 144); JD approved it for
Sheoldred. Before (the #276 build) **28.2% over t3-t4** (27.6% / 28.8%); after **28.1%** (27.9% / 28.2%): no change,
inside the noise. Why: the wasted holds were turns late in a game with a full hand, when the mana had no better use, so
freeing it bought little. Kept because it is correct, not because it wins.

## Baseline 2026-11-06 (after the rewire, Doomsday and Phase C engine work)
Sheoldred, t3 / t4, 2,000 games per tier, seed 500000, loose, look-ahead, build ef343cc1: **27.7%** (25.8-29.7) / **27.2%**
(25.3-29.2), **27.5% over t3, t4**. Old build (main 1e027077, 2026-11-05): 28.8% (26.8-30.8) / 28.9% (27.0-30.9), 28.8% over t3, t4.
Change -1.1 / -1.7, average -1.3: inside the noise (all intervals overlap, each under 2 points), though both tiers move the
same way; not a proven loss. This is the first number for the Coffers drain package list (11/04; the old-build run already had it).
Engine changes that can touch this deck: C3 (the t3 pool has Atraxa, so infect is now combat damage and a poison threat; the
deck has no attack tax of its own) and opponents casting their commander on time (+1.5), which makes the table a little stronger.
That is the only plausible source of a small drop. No C2 card is in the deck; C1/C1b and the rewire do not touch it.
Raw: `audit/baseline-2026-11-06/sheoldred.txt` and `sheoldred-2026-11-05-oldbuild.txt`.

## 2026-11-04: Coffers drain package (JD's choice; not simulated)
Sanguine Bond, Exquisite Blood, Vito, Thorn of the Dusk Rose, Grim Tutor and Torment of Hailfire in for Seymour Flux,
Lethal Throwdown, Necrosquito, Phyrexian Rager and Burrog Banemaker (JD 11/04: Necrosquito leaves rather than Raise Dead,
which stays as the cheap way to recover a combo piece or Gray Merchant). Why: with 31 Swamps, Cabal Coffers is big mana on
its own, and in Y'shtola's Coffers tests Exquisite Blood + Sanguine Bond was the piece that worked (cast in 68% of the
games drawn, 61% wins when cast). Vito makes it any two of three; Grim Tutor is the third tutor. Triskelion stays (the
Mikaeus combo). Game Changers unchanged (4). No number yet: measure over T3-T4.

## 2026-11-04: Cabal Coffers for Command Tower (JD's physical deck; not simulated)
31 Swamps make Coffers a big-mana land on its own (six Swamps out: net {B}{B}{B}{B}) for Exsanguinate and the big
turns. Command Tower made only {B} here. No number yet: the next baseline (T3-T4) measures it.

## Current state (2026-10-15)
Win rate vs T4: **30.1%** (1,000 games, seed 1015, main 0f49b7d).

## Strengths
- **Drain wins:** 76% of the opponents it eliminates go by drain (30 of its 46 damage a game). Card-draw punishers
  and draw engines have the highest win rates when cast: Memory Jar 53%, Painful Lesson 50%, Read the Bones 49%,
  Howling Mine 49%, Underworld Dreams 48%.

## Weaknesses (by evidence)
1. **Combo decks at T4:** Heliod's combo kills it in 19% of games, Prosper's in 9%; Sheoldred has few ways to stop a
   combo.
2. **Sheoldred comes late:** resolves on turn 6.0 on average (by turn 6 in 53% of games, never in 19%); the first
   drain comes by turn 6 in 52% of games.
3. **Combat decks:** Krenko (12%) and Yuriko (7%) by combat.

## What was tried, and why it did or didn't work
### 2026-10-19: Ulna Alley Shopkeep => K'rrik, Son of Yawgmoth (T3, loose, 2,000 games per list, look-ahead; T4 stopped by JD)
Result: **T3 32.2% -> 31.5%, -0.7 (noise ±2.4): neutral.** Raw: `2026-10-19-krrik/t3.txt`. Ulna Alley Shopkeep was
the cut (second-lowest win rate when cast in the 10/19 card report).
Why: K'rrik is cast in 62% of the games it is drawn (Shopkeep 76%), around round 7.6, and stuck 38%; Sheoldred's
timing (turn 6.4), the plan rate and damage are unchanged. By the time the 7-mana legend is down most of the deck's
black spells are already cast, so paying life for {B} rarely matters; the AI also casts him eagerly only at 16+ life.
Found on the way: the shared Phyrexian-mana rule checked the starting life for every pip, so three {B/P} at 12 life
could go to 6; fixed on this branch (9c68a6a) to count the life already paid.

### 2026-10-19: Stir Up Trouble => Orcish Bowmasters, full run (T3-T4, loose, 2,000 games per list, look-ahead)
Result: **T3 +2.0 (±2.6), T4 +0.5 (±2.4): about +1.2 over t3-t4, positive but within noise.** Raw:
`2026-10-19-bowmasters/full-stir-up-trouble-t3.txt`, `t4.txt`.
Why: Bowmasters is cast in 88-91% of the games it is drawn, around round 5.4-6.1, against Stir Up Trouble's 79-86%
(stuck 14-21%). Damage +0.9 to +1.7 a game; plan timing and game length unchanged. At T3 the Atraxa losses drop
(8.2 -> 6.4%); at T4 the deck still dies to Heliod combo (17%) and Krenko (12%) as before, which a 1/1 pinger doesn't
stop. A small, real-looking edge over a weak card, smaller than the quick read suggested.

### 2026-10-19: Orcish Bowmasters for three cards (quick read: T3-T4, loose only, 700 games per list, look-ahead)
| Cut (=> Orcish Bowmasters) | T3 | T4 | Average |
|---|---|---|---|
| Stir Up Trouble | +3.3 | +1.9 | +2.6 |
| Fetid Imp | +3.3 | +1.3 | +2.3 |
| Soul Enervation | +1.4 | +2.1 | +1.8 |
Noise ±3.9-4.3 per cell. **Leans good: about +2.2 over t3-t4, all six cells positive, none beyond noise alone.** The
cuts share one baseline per tier (30.9% T3, 29.4% T4), so they are not independent; a full run of one cut is needed
to confirm. Raw: `2026-10-19-bowmasters/<cut>-<tier>.txt`.
Why: Bowmasters is cast in 84-90% of the games it is drawn, early (round 5.5-6.5), and wins 38-48% of the games it is
cast in. Damage to opponents rises in every cell (+0.7 to +3.7 a game): Howling Mine and Seizan hand opponents
extra draws, each a ping and a bigger Orc Army. The plan (Sheoldred out) comes no sooner, so the gain is extra
pressure, not speed. JD would buy a second copy (the Sauron copy stays).

| Change | Result | Why |
|---|---|---|
| AI: look-ahead ramp bonus (`ramp_w` 1; audit/ramp/2026-10-15) | T4 -1.9 (noise +-2) | Rocks come down first, and the late payoffs are cast less (Howling Mine 50% -> 43% of games drawn, Peer into the Abyss 31% -> 26%); Sheoldred's timing is unchanged (6.0). Merged off. |

**2026-10-17 land audit** (branch `land-audit2`, `rust/agents/report-land-audit2.md`): 12 lands across the decks now make only the mana their Oracle text allows (filter lands, Rugged Prairie, Mystic Gate, Spire of Industry, Exotic Orchard, Cavern of Souls, Spectator Seating, Hidden Grotto and others; Sulfur Falls and Fabled Passage were too strict). Same-seed check (adaptive AI, 300 games per tier, matched tiers): 40.1 → 39.2, inside the noise (about ±5). No measurable change to this deck.

### Ad Nauseam (2026-10-17 night; branch `adnaus`; would be the deck's first Game Changer, bracket 3 allows three)
Card report (adaptive, 1,000 games per tier, `audit/sheoldred/2026-10-17-adnaus/K0-analyze-adaptive.txt`): lowest win
rate when cast: Stir Up Trouble (bottom two at T3 and T4), Fetid Imp, Soul Enervation, Restless Bloodseeker (T3, T5).
The shared Ad Nauseam AI was fixed during these tests (see audit/kefka/FINDINGS.md: own-turn casts, flipping past a
full hand, a one-board floor; and the look-ahead overriding "hold mana").
| Swap (Ad Nauseam in for) | Result over t3–t5 | Why |
|---|---|---|
| Stir Up Trouble / Restless Bloodseeker (adaptive, old AI) | -1.5 / -2.0 | Cast on its own turn, down to 16–18 life, discarded back to seven. |
| Stir Up Trouble (look-ahead, old AI) | +0.7 (+2.7, +0.9, -1.4) | |
| Stir Up Trouble / Restless Bloodseeker (adaptive, new AI) | -0.9 / -0.5 | |
| **Restless Bloodseeker (look-ahead, new AI + hold credit, 1,000 paired per tier)** | **+1.6 (t3 +2.0, t4 +0.7, t5 +2.2; ±3.1–3.7)** | The best Ad Nauseam result, positive in every tier but still noise. Cast in 30% of the games it's drawn (twice Kefka's and Y'shtola's), wins 47% when cast vs 28–36% for Restless Bloodseeker; eliminations -1.8 (T3), -0.6 (T4). The lowest curve of the three decks (average mana value 2.98) and no hand-size problem. Baseline unchanged by the AI fixes (29.0% vs 29.5%). |
**Reading it:** the best home for Ad Nauseam among JD's decks. A 2,000-paired run would settle it. **Applied (JD, 2026-10-17): Ad Nauseam in, Restless Bloodseeker out (to bulk).**

## 2026-10-18: band calibration (proposed swaps, NOT applied; see audit/bands/REPORT.md)
Band 2 (T3-T4). Proposed: Gray Merchant of Asphodel to Seph FS, Necrosquito (spare) in. Final: **30.6% vs T3, 30.8% vs T4** (avg 30.7%; current 32.5 / 33.1 at 1,000). Why: Sheoldred was at the top of band 2; Gray Merchant is a better fit in Seph FS (32 Swamps), and losing it costs Sheoldred ~2 points, which tightens band 2.

## 2026-10-18: AI audit re-measure (branch ai-audit; 2,000 games per profile, look-ahead)
**31.9% vs T3, 31.6% vs T4** (before 30.6 / 30.8): +1, within noise. Protection held for Sheoldred; Ad Nauseam in the second main; Memory Jar waits for a punisher; Seizan, Necrosquito valued; Rooftop Assassin condition.

## 2026-10-19: K'rrik, Son of Yawgmoth modeled (smoke run only: Stir Up Trouble => K'rrik, T3, loose, 50 games, adaptive)
Modeled (impls/krrik.rs, engine/mana.rs): with K'rrik out the AI pays 2 life for a black pip its mana cannot pay,
only while its life stays at 15 or more afterwards; his own {B/P} costs use the 10-life Phyrexian rule; each black
spell adds a +1/+1 counter. Floor 15: Sheoldred's drain/gain plan makes life a resource, but at T3 the table kills from
around 12 in one round, and 15 stays clear of that. Cast priority 70 at 16+ life.
Smoke result: 32.0% -> 32.0% (0.0, noise ±5.7). Why: drawn 28% of games, cast in 64% of those (Stir Up Trouble 79%),
first cast round 6.3 vs 4.4: a 7-mana body that comes down later than the cheap spell it replaced; wins when cast
33% vs 36%. Needs a full-size run before judging.

## 2026-11-01: recalibration stage 2 remeasure

Sheoldred, t3 / t4: **15.9% / 17.2%** (average 16.5%) with the hand AI on (default) (2000 games, seeds 500000-501999, loose, look-ahead, defaults, rc=0; `audit/recal/stage2-2026-11-01.md` section 4, build efe45425). `AI_HAND=0` (generic + plan file): 17.4 / 17.4, level (average delta -0.8 [-2.5, +0.8]). Stage 1, same seeds: 17.8 / 17.6 (hand AI on). The 10/18 AI audit numbers (31.9 / 31.6) are from an older build and list. Fitted t3 knobs: 18.7% (+2.8 [+0.5, +5.1]).

## 2026-11-04 (late): AI changes, not simulated (JD: no sims for this batch)

- **Blood Artist**: drains once per Artist (two Artists drained once per death), and only when a creature dies (it fired for any permanent).

The numbers above predate these changes; a re-run on the deck's own tiers would show their effect.

- **Zulaport Cutthroat** drains once per Zulaport (two drained once) and for its own death; **Blood Artist** also for its own death (the dying creature had left the battlefield before the drains were counted). Not simulated either.
