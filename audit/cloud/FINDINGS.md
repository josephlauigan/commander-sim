# Cloud, Midgar Mercenary — findings (living file)

What the simulator has taught us about this deck: strengths, weaknesses, and what was tried and why it did or didn't
work. Updated after every run (CLAUDE.md "Swap tests explain themselves"). Newest evidence first; numbers are
look-ahead, loose profile, vs Cloud's matched tiers (T1, T2) unless noted.

## Current state (2026-10-15)
List: C4 + Package D with substitutes (Crawlspace, Archon of Absolution), plus the 10/15 AI fixes.
Win rate: **T1 20.6%, T2 18.2%** (was 14.0 / 11.9 on the morning of 10/15). Fair band for its tiers: 22-32%.

## Strengths
- **Cloud lands on time:** resolved by turn 4 in 99% of games (he costs {W}{W} and tutors an Equipment).
- **Defense works:** attack taxes + wipes (Package D) were the one big gain, +6.4 / +6.3. The deck dies to combat
  races, and slowing the race is what it needs most.
- **When it gets going it hits hard:** ~33 damage per game, 94% of it combat.

## Weaknesses (by evidence)
1. **Deployment is the bottleneck, not card count.** The hand sits at ~6.2 cards after the turn-4 land drop and ~5.1
   at turn 8; even cheap cards stay stuck in hand in 20-30% of the games they're drawn (Mentor of the Meek 39%,
   Lightning Greaves 29%, Brass Squire 28%). Mana is ~4.0 on turn 4 and ~7.1 on turn 8, and every Equipment costs
   mana twice (cast + equip). The plan (an equipped Cloud attacking) is online by turn 4 in only ~26% of games, by
   turn 6 in ~58%, never in ~23%.
2. **Light-Paws is the worst matchup in T1:** it causes ~18% of Cloud's eliminations by commander damage (voltron vs
   voltron); then Isshin (~15%, combat). Cloud has few answers to a big Aura'd creature.
3. **Dead cards in the list:** Unbreakable Formation (cast in 1% of the games it's drawn), Ultimate Magic: Holy (11%).
   These slots do nothing today.
4. **Cutting mana rocks or bodies costs more than the new cards give** (packages S, B, R all lost what they cut).
5. Cloud is removed ~0.6 times a game at T1 (mostly exile and wipes); protection is rarely used.

## What was tried, and why it did or didn't work
| Change | Result (T1 / T2) | Why (paired breakdown, T1, 1,000 games) |
|---|---|---|
| AI fixes (attack when equipped, Riders, tutor order, Unfinished Business) | +1.8 / +1.9 | Equipped Cloud no longer sits home. |
| **Package D** (attack taxes + wipes) | **+6.4 / +6.3** | Slows the combat race that kills Cloud. Adopted. |
| Substitutes Crawlspace + Archon of Absolution for Ghostly Prison + Archangel of Tithes | -1.5 / -0.9 vs full D (noise) | Same job (limit/tax attackers). Adopted. |
| R removal (6 swaps) | -1.9 / -1.1 | Cut 6 creatures/Equipment; fewer bodies and threats. |
| **C card draw** (Sram, Mentor, Welcoming Vampire, Skullclamp, Wall of Omens) | +1.1 (2,000 g) and +3.2 (1,000 g) / — | **Draws +1.5 cards a game (significant), casts +0.8 spells, but the plan comes online no sooner** (26.8% by turn 4 vs 25.6%) and the hand gets bigger. Extra cards mostly pile up behind the mana bottleneck; a small real gain (~+2) through slightly more damage. Mana on turn 4 drops (Mind Stone cut). |
| **B backup carriers** (Fencing Ace, Mirran Crusader, Kor Skyfisher, Selfless Spirit, Adeline) | -0.6 (2,000 g) and -1.9 (1,000 g) / — | Carriers do get cast (~75-86%), but **the plan comes online less often (-4.9 points, significant)** and spot removal falls (Disenchant cut: -0.2 a game). The cuts were cheap early plays (Forum Familiar cast 94%, Helitrooper 92%) and a land; the carriers don't replace an equipped Cloud. |
| **S speed** (Brass Squire, Lightning Greaves, Danitha, Sigarda's Aid) | -1.6 / +0.9 (2,000 g); +0.8 (1,000 g) | Sigarda's Aid is cast 96%, but Squire and Greaves sit in hand 28-29%; plan by turn 4 only +1.9. It replaced two dead cards (Formation, Holy) and two rocks: the rocks' mana is what the faster equips needed. (Danitha's {1} discount is not modeled.) |
| Sun Titan for Ultimate Magic: Holy | -0.4 / — | Not broken down; a 6-drop in a mana-starved deck. |
| Kor Haven for a Plains | -2.5 / — | Not broken down; colourless land in a {W}{W} deck, and its ability ties up {1}{W}. |

## What this suggests next (untested)
- **More mana, not more cards:** the dead slots (Unbreakable Formation, Ultimate Magic: Holy) are the cheapest place to
  add ramp (mana rocks / land ramp), so Equipment can be cast and equipped sooner.
- **Answers to a voltron threat** (Light-Paws): Aura/creature removal that doesn't cost a body (Package R's lesson).
- Keep the creature count and rocks; cut only dead cards.
