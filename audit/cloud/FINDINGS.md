# Cloud, Midgar Mercenary — findings (living file)

What the simulator has taught us about this deck: strengths, weaknesses, and what was tried and why it did or didn't
work. Updated after every run (CLAUDE.md "Swap tests explain themselves"). Newest evidence first; numbers are
look-ahead, loose profile, vs Cloud's matched tiers (T1, T2) unless noted.

## Current state (2026-10-16; superseded: the newest baseline is the 2026-11-04 section at the end)
List: C4 + Package D with substitutes (Crawlspace, Archon of Absolution), plus the 10/15 AI fixes.
Win rate: **T1 20.6%, T2 18.2%** (was 14.0 / 11.9 on the morning of 10/15). Fair band for its tiers: 22-32%.
Double strike package (Fireshrieker, Duelist's Heritage, Akroma's Will) applied 10/15: 2,000 games, T1 25.9 -> 30.3 (+4.5), T2 20.4 -> 24.8 (+4.3), measured on the list before the Crawlspace/Archon substitutes (within noise of it). Not re-measured on the current list.
Four swaps applied 10/16 (Archangel of Tithes, Open the Armory, Sphere of Safety, Silverblade Paladin for Helm of Kaldra, Shield of Kaldra, Ghostly Prison, Duelist's Heritage): T1 29.1 -> 30.3 (+1.2, ±3.8, neutral; 1,000 games), so the win rate is about 30% on T1.

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
| **Double strike** (Fireshrieker, Duelist's Heritage, Akroma's Will for Secure the Scene, Veteran's Powerblade, Nim Deathmantle; needed the engine fix: double strike = two damage steps) | **+4.5 / +4.3** (2,000 g; quick T1 +6.1) | Equipped double-strike Cloud triggers each Sword twice per attack and deals commander damage twice. Confirmed, adopted. |
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
| AI: Sword choice by table colours + protection timing + attack-target weighting (branch cloud-ai4) | 29.3 -> 26.9 (T1, 2,000) | Plan online sooner (by turn 4 29% -> 33%), but the target weighting spread Cloud's damage across players so nobody died. Not adopted. |
| Same without the attack-target change | 29.3 -> 28.4 (T1, 2,000) | Within noise; Sword choice and protection timing don't move the win rate. Not adopted. Lesson: a voltron commander needs focused damage. |
| Sram + Danitha for Bronze Guardian + Kor Duelist (10-16) | +0.5 (T1, 1,000, ±3.8) | Neutral: Sram cast 82% when drawn, Danitha 70% (stuck 30%); damage +1.2 a game, plan timing unchanged. Fine to play, no measurable gain. |
| Helm of Kaldra, Shield of Kaldra, Ghostly Prison, Duelist's Heritage for Archangel of Tithes, Open the Armory, Sphere of Safety, Silverblade Paladin (10-16) | +1.2 (T1, 1,000, ±3.8) | Neutral (29.1 -> 30.3). JD takes Helm and Shield of Kaldra for another deck and could not find the other two. Open the Armory cast 87%, Silverblade Paladin 66%, Sphere of Safety 58% (stuck 42%); Helm and Shield of Kaldra had been stuck 22% and 44%. |
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

**2026-10-17 land audit** (branch `land-audit2`, `rust/agents/report-land-audit2.md`): 12 lands across the decks now make only the mana their Oracle text allows (filter lands, Rugged Prairie, Mystic Gate, Spire of Industry, Exotic Orchard, Cavern of Souls, Spectator Seating, Hidden Grotto and others; Sulfur Falls and Fabled Passage were too strict). Same-seed check (adaptive AI, 300 games per tier, matched tiers): 18.3 → 18.8, inside the noise (about ±5). No measurable change to this deck.

## 2026-10-18: band calibration (proposed swaps, NOT applied; see audit/bands/REPORT.md)
Band 4 (T1-T3). No change. Final: **29.6% vs T1, 26.3% vs T2, 20.4% vs T3** (avg 25.4%, 2,000 per profile). Fair against T1-T2, clearly weaker at T3: the band-4 shape.

## 2026-10-18: AI audit re-measure (branch ai-audit; 2,000 games per profile, look-ahead)
**28.9 / 23.8 / 21.4% vs T1 / T2 / T3** (before 29.6 / 26.3 / 20.4). T2 -2.5 is on the edge of noise; suspects: the new turn-2 search order (Feast and Famine first only with 7+ cards) and the Austere Command mode rule. A paired check vs main is the open item.

## 2026-10-19: The Mind Stone for three rocks (quick read: loose only, 700 games per list, T1-T3, look-ahead)
| Cut (=> The Mind Stone) | T1 | T2 | T3 | Average |
|---|---|---|---|---|
| Mind Stone | -2.0 | +0.1 | -0.4 | -0.8 |
| Thought Vessel | -1.0 | +1.9 | -2.9 | -0.7 |
| Fellwar Stone | +1.9 | -0.9 | -1.6 | -0.2 |
Noise ±2.8-3.5 per cell. **All three neutral**: no cut shows a gain at any tier beyond noise, and the averages lean
slightly negative. Raw: `2026-10-19-mind-stone/<cut>-<tier>.txt`.
Why: as a rock it changes nothing (mana on turn 4 3.99 -> 3.99; plan online 78%, turn 5.7, unchanged). It is drawn
in 19-29% of games and cast in 76-88% of those, but late (average round 6.5-8.1) and stuck in hand 12-24%, as the
deck's other cheap cards are (deployment is the bottleneck, weakness 1). The {5}{W} harness then competes with
equipping for the same mana, and the model only blinks enters-effect creatures (Cloud's own tutor is the main one,
and blinking him drops his Equipment), never Equipment. Verdict: not worth buying for Cloud; matches the 10/18 T1 read.

## 2026-10-18 evening: Mind Stone => The Mind Stone (quick pass, stopped after T1; loose only, 700 games per list)
Result: **T1 29.9% -> 30.6% (+0.7, ±3.2): noise.** T2 and T3 not run (JD stopped the sims and decided to wait on
buying the Stone). Raw: `2026-10-18-mind-stone/mind-stone-quick-STOPPED-t1only.txt`.

## 2026-10-18 evening: JD's new cards, coarse check (loose only, 500 games per list per tier, look-ahead; only big downgrades show, noise ±4-5 per tier)
- **Hero's Blade => Andúril, Narsil Reforged: T1 +0.4, T2 -2.0, T3 +0.8 (about -0.3; noise, no downgrade).** Cast in
  83-88% of draws (Hero's Blade 79-86%), wins 56% when cast at T1; a counter on every creature per swing (two with the
  city's blessing) is a team pump, but Cloud wins by one equipped attacker, so it trades evenly with a cheap pump.
- **Citizen's Arrest => Stroke of Midnight: T1 -3.8, T2 +0.8, T3 -1.2 (about -1.4; noise, no big downgrade).** Held
  as an instant answer (cast 73-79% of draws vs the Arrest's 76-84%) and stuck in hand more; the 1/1 it gives the
  opponent and its {2}{W} cost make it no better than the Arrest here. Y'shtola is the other home being tested.
Raw: `2026-10-18-new-cards/`. JD plans to add the cards unless one is a big downgrade.

## 2026-10-31: card models (branch card-models-1; no sims until Phase 4 recalibration)

Buster Sword: Partial -> Full (draw, then a free spell up to the damage). No new number yet.

## 2026-11-01: recalibration stage 2 remeasure

Cloud, t1 / t2 / t3: **25.6% / 14.2% / 17.0%** (average 18.9%) with the hand AI on (default) (2000 games, seeds 500000-501999, loose, look-ahead, defaults, rc=0; `audit/recal/stage2-2026-11-01.md` section 4, build efe45425). `AI_HAND=0` (generic + plan file): 23.1 / 13.8 / 16.8, level (average delta +1.0 [-0.3, +2.4]). The 10/18 numbers (28.9 / 23.8 / 21.4) are from the old list.

## 2026-11-04: Judgment of Alexander and Forge Anew vs the cards they replaced (paired, loose, look-ahead, 2,000 games per list per tier, t1-t3)

Both cards are in the current list (JD's choices 11/01 and the 11/02 Ghave breakup; neither had a number until now). The
runs were set up as NEW=>OLD, so the tool's "baseline -> variant" is the current list -> the old card back in. **The
numbers below are flipped to read as the new card's effect over the old one.** Raw:
`.claude/worktrees/sim-runs-2026-11-04/cloud-judgment-vs-standup.txt`, `cloud-forge-vs-sphere.txt`. Same seeds in both
lists; "noise" is the paired +-2 SE the tool prints next to the win rate (the per-axis tables print wider bands, +-2.6-2.9
on win rate, which are not the paired ones).

**Current-list baseline (both runs share it): t1 31.9%, t2 22.3%, t3 22.9% (average 25.7% over t1-t3).** This is not
like-for-like with the 11/01 remeasure (25.6 / 14.2 / 17.0, average 18.9): the list may have changed since (JD's swaps of 11/01,
the Ghave breakup 11/02, Stoneforge Mystic and Steelshaper's Gift 11/03) and so did the build and plan file. The size of
the jump was not investigated here.

### Judgment of Alexander over Stand Up for Yourself
| Tier | With Judgment | With Stand Up | Judgment over Stand Up | Noise (+-2 SE) | Verdict |
|---|---|---|---|---|---|
| t1 | 31.9% | 30.1% | **+1.8** | +-2.1 | inside noise |
| t2 | 22.3% | 22.1% | **+0.2** | +-2.1 | inside noise |
| t3 | 22.9% | 21.9% | **+1.0** | +-1.9 | inside noise |
| **Average over t1-t3** | 25.7% | 24.7% | **+1.0** | about +-1.2 (my pooling of the three bands) | inside noise |

**Reading it:** the sign is positive at all three tiers but every cell, and the average, is inside its noise band, so this
is "no clear effect", at best a small lean toward Judgment. The tool's verdict was NO CLEAR EFFECT at every tier.

**Why (paired breakdown, per tier t1 / t2 / t3):**
| | Judgment of Alexander | Stand Up for Yourself |
|---|---|---|
| Drawn in | 24.4 / 26.2 / 22.8% of games | 24.8 / 26.4 / 23.2% |
| Cast, of games drawn | **23.1 / 34.3 / 17.5%** | **79.2 / 86.0 / 75.9%** |
| Stuck in hand | 76.9 / 65.7 / 82.5% | 20.8 / 14.0 / 24.1% |
| Average round of first cast | 9.8 / 11.7 / 9.4 | 7.1 / 8.3 / 6.8 |
| Win rate when cast (correlation only) | 54.0 / 28.9 / 55.0% | 44.0 / 31.1 / 33.2% |
| Removed by opponents, per game | 0.00 | 0.00 |
- **Judgment is mostly held, and cast late:** it is drawn about as often as Stand Up but cast in only 17-34% of those games,
  at round 9-12, because the AI never casts it in the main phase (priority 0) and keeps it for a big unblocked attack or a
  burn spell. When it is cast the games are won more often at t1 and t3 (54-55% vs 33-44%), but that is correlation only (the tool says so too).
- **Stand Up is cast in 76-86% of draws, as spot removal:** the one axis that moves beyond noise at all three tiers is spot
  removal cast per game, 0.97 / 1.05 / 0.77 with Judgment vs 1.14 / 1.21 / 0.93 with Stand Up (Stand Up +0.15-0.17, +-0.06).
  That extra removal does not turn into more wins: the Stand Up list wins 1.0 less on average.
- **Game shape is the same either way:** rounds survived 10.03 / 12.08 / 9.70 with Judgment vs 10.02 / 12.01 / 9.70; plan
  online at all 75.8 / 74.2 / 68.6% vs 75.8 / 73.5 / 67.9%; commander removals per game 0.36 / 1.07 / 0.41 vs 0.35 / 1.05 / 0.41.
  The Stand Up list is eliminated 1.8 / 0.8 / 1.0 points more often and deals 1.0-1.5 less damage per game (+-3.3-3.8, noise),
  with plan-online by turn 8 at t3 2.1 points lower (62.0 vs 59.9). None of these is beyond noise.
- **One more starred axis, probably chance:** at t2 the Stand Up list resolves 0.11 fewer tutors per game (+-0.10); the tool
  itself warns that one or two stars appear by chance in ~40 checks.
- **Model notes (`impls/cloud.rs`, `rust/agents/report-cloud-gaps.md`):** Judgment is modelled (the 11/04 coverage line shows no Approximate, Partial or Unmodeled card in the list). The shield is a per-turn flag
  read by the damage-prevention path (combat, burn and every damage to the player). The strike back runs after combat
  damage for creatures whose combat damage was prevented, with every commander creature's power added into one source per
  creature; it kills when that reaches the toughness (damage is not kept marked). **Simplification:** a creature's noncombat
  damage to the player (a ping) is prevented but has no strike back, because noncombat damage to a player carries no
  source permanent. The AI casts it after blockers when the unblocked damage is lethal, heavy (at least max(6, 40% of life))
  or the commanders can kill attackers worth at least 5 with 3+ damage coming, or in response to a face burn spell of 6+,
  lethal or 4+ and half their life. Stand Up for Yourself is Full-auto (compiled from the Oracle text as a destroy-target-creature
  removal spell), cast as a response / removal only.

### Forge Anew over Sphere of Safety
| Tier | With Forge Anew | With Sphere of Safety | Forge over Sphere | Noise (+-2 SE) | Verdict |
|---|---|---|---|---|---|
| t1 | 31.9% | 30.6% | **+1.4** | +-1.8 | inside noise |
| t2 | 22.3% | 20.4% | **+1.9** | +-1.8 | just outside the band (the tool: Sphere WORSE) |
| t3 | 22.9% | 22.6% | **+0.4** | +-1.6 | inside noise |
| **Average over t1-t3** | 25.7% | 24.5% | **+1.2** | about +-1.0 (my pooling) | borderline: 1.2 against a 1.0 band |

**Reading it:** Forge Anew is ahead of Sphere of Safety at all three tiers. Only t2 clears its band (by 0.1), and one tier in three is thin, so call it a likely small edge for Forge
(about +1), not a proven one.

**Why (paired breakdown, per tier t1 / t2 / t3):**
| | Forge Anew | Sphere of Safety |
|---|---|---|
| Drawn in | 24.6 / 26.8 / 21.3% of games | 23.8 / 26.2 / 21.1% |
| Cast, of games drawn | **64.4 / 68.5 / 54.7%** | **42.1 / 50.8 / 44.1%** |
| Stuck in hand | 35.6 / 31.5 / 45.3% | 57.9 / 49.2 / 55.9% |
| Average round of first cast | 8.7 / 9.1 / 8.0 | 8.5 / 9.4 / 8.1 |
| Win rate when cast (correlation only) | 62.8 / 43.1 / 55.4% | 55.0 / 29.3 / 44.1% |
| Removed by opponents, per game | 0.00 / 0.01 / 0.00 | 0.01 / 0.03 / 0.00 |
- **Forge is cast 10-22 points more often when drawn** (64 / 69 / 55% vs 42 / 51 / 44%), at about the same round, so it is
  not that Forge comes down sooner; it is that Sphere sits in hand more (49-58% stuck). A likely reason (not shown by the
  breakdown): Sphere costs {4}{W} against Forge's {2}{W}, and both compete with equips for mana, the deck's known deployment
  bottleneck.
- **Damage and survival:** with Sphere the games run slightly longer (rounds survived +0.04 / +0.14 / +0.10 for Sphere, all
  inside the +-0.16-0.21 noise), but Cloud is eliminated more often with Sphere
  (+1.4 / +2.2 / +0.3 points) and deals less damage (t1 52.6 vs 52.6 flat, t2 47.4 vs 45.0, t3 40.7 vs 40.1; all inside the
  +-3.4-3.9 noise).
- **The one starred axis:** damage from "other triggers" is lower with Sphere at every tier
  (4.51 vs 4.22, 3.18 vs 2.86 starred at t2, 3.98 vs 3.79). The breakdown does not say why; the likely reason is Forge's free
  first equip and instant-speed equips, which let Equipment re-equip and trigger more, but that is a reading, not a measurement.
- **Plan timing is unchanged:** plan online by turn 6 is 55.8 / 51.2 / 48.4% with Forge vs 55.6 / 50.3 / 48.1% with Sphere;
  turn-4 and turn-8 mana, commander resolution and commander removals do not move. So Forge's free equip is not making the plan
  come online sooner; its edge, if real, is in the damage and the fewer eliminations once the plan is running.
- **Model notes (`impls/cloud.rs`, `rust/agents/report-cloud-gaps.md`):** Forge Anew is hand-written card code: the enter trigger returns the best Equipment card from the graveyard (the compiled reanimate had no Equipment filter, so
  it was replaced); the first equip each own turn costs {0} (`equip_cost` / `forge_free`); equips can be activated at instant
  speed on Cloud's own turn. **Simplifications:** the AI uses instant speed only by moving Equipment from loose or
  non-attacking holders onto the best unblocked attacker after blockers (free equips if worth anything, paid ones only
  at cost 2 or less with a large gain); Forge's cast priority is 68 with an Equipment in the graveyard, 45 with two or more
  Equipment and a creature out, otherwise 0 (so it is held until there is something to use it on). Sphere of Safety is Full
  with no tags and a generic priority of 55.

### What this changes
- Neither new card is shown to be better than the one it replaced beyond noise; both lean positive (+1.0 and +1.2
  average over t1-t3). No reason to swap either back.
- Judgment's low cast rate (17-34% of draws) comes from the AI's conservative trigger, not the card being dead; a looser
  trigger is untested.
- Open: both averages are pooled by hand from per-tier bands, so a 4x run (or `--ai adaptive`) is needed to settle the Forge
  edge.
