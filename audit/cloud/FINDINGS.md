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
| **M-lite mana** (Worn Powerstone, Foundry Inspector, Burnished Hart for Formation, Holy, Secure the Scene) | -0.1 / — | Cuts were dead cards, but the new mana is **cast late** (Powerstone avg round 6.8, stuck 28%; Hart 8.8, stuck 35%): turn-4 mana +0.07 only, plan timing unchanged. |
| **M mana** (M-lite + Weathered Wayfarer, Stoneforge Mystic) | +1.8 / — | Within noise. **Stoneforge Mystic first cast at avg round 7.1** (a turn-2 tutor); turn-4 mana unchanged, plan by turn 4 25.4% (-1.6). |
| M without Stoneforge | +1.0 / — | Within noise. |
| **AI ramp fix** (look-ahead bonus for rocks/ramp/tutors on turns 1-6; cloud only) | +0.7 / — | Rocks now come down early (turn-4 mana 3.98 -> 4.23) but **the plan comes online no sooner** (by turn 4: 27% -> 24%): turns 2-3 spent on a rock are turns not spent on Cloud + the first Sword. |
| **M mana on the fixed AI** | +1.2 / — | Stoneforge now cast earlier (stuck 15% -> 9%), turn-8 mana +0.4, but plan by turn 4 falls again (24% -> 21%). Extra mana helps a little late; it doesn't make the deck faster. |
| General ramp valuation fix (all decks, session mtg-sims-a9; merged OFF) | +0.1 (1,000 g) / — | Plan online at all 74.7 -> 78.7, but by turn 4 unchanged; ramp sooner doesn't bring payoffs sooner. |
| **Option #3 wide list** (16 swaps, Kaldra kept, AI spreads Equipment; branch cloud-wide) | +1.2 / — (vs today's list on the spread AI) | Equipment spreads (45% on Cloud, was 74%) and games last 0.4 rounds longer, but damage is flat (~33) and the new cards are cast late (rounds 8-9; Stonehewer stuck 43%, Triplicate Spirits 48%): same deployment bottleneck. |
| **D2 defense** (Winds of Abandon, Elspeth Sun's Champion, Wall of Omens for Formation, Holy, Recommission) | **+3.6 (±2.5) / +1.5 (±2.4)**; quick check +3.4 | Dies less (eliminated 79 -> 76%), deals more (33 -> 37 a game); Elspeth cast only 40% (6 mana) but wins 51% when cast and her -3 answers Light-Paws (its share of Cloud's deaths 18 -> 16%). Replaced cards were dead. |
| Sun Titan for Ultimate Magic: Holy | -0.4 / — | Not broken down; a 6-drop in a mana-starved deck. |
| Kor Haven for a Plains | -2.5 / — | Not broken down; colourless land in a {W}{W} deck, and its ability ties up {1}{W}. |

## What this suggests next (untested)
- **Defense is the lever (confirmed twice):** Package D (+6) and D2 (+3.6 at T1) are the only real gains; mana, speed, card draw, carriers and a wide rebuild all failed for the same reason (cards cast late). More cheap defense in weak slots is the most promising direction.
- **Mana is not the lever (10-15, confirmed):** casting ramp early (AI fix) and adding ramp (M, M-lite) both left the plan's timing unchanged or slower. The plan's total cost (Cloud {W}{W} + Equipment + equip) on turns 2-4 is the real limit; ramp competes with it for the same early turns. Cloud looks close to its ceiling (~21-22% at T1) with this design; defense (Package D) was the one big lever.
- **Check the AI's play order first (10-15):** two mana packages showed cheap ramp and tutors (Worn Powerstone, Stoneforge Mystic) first cast around round 7. If the AI plays Equipment and creatures before ramp, more mana cards can't help until ramp is cast early. Confirm in traces (time from draw to cast), then fix priorities.
- **More mana, not more cards:** the dead slots (Unbreakable Formation, Ultimate Magic: Holy) are the cheapest place to
  add ramp (mana rocks / land ramp), so Equipment can be cast and equipped sooner.
- **Answers to a voltron threat** (Light-Paws): Aura/creature removal that doesn't cost a body (Package R's lesson).
- Keep the creature count and rocks; cut only dead cards.
