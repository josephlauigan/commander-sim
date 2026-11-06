# Kefka, Court Mage — findings (living file)

What the simulator has taught us about the Kefka Grixis discard deck: strengths, weaknesses, and what was tried and why
it did or didn't work. Updated after every run. Newest evidence first. Matched tiers: t3–t5 (`data/my_decks.toml`).

## Baseline 2026-11-06 (after the rewire, Doomsday and Phase C engine work)
Kefka, t3 (its only tier now), 2,000 games, seed 500000, loose, look-ahead, build ef343cc1: **17.1%** (15.6-18.9) over t3.
Old build (main 1e027077, 2026-11-05): **17.8%** (16.2-19.5). Change -0.7: noise (intervals overlap, under 2 points). Plan online
(Kefka flipped) 14.9% (old 15.3%); wins by damage 100% in both. Deaths: Tergrid 21.5%, Aurelia 20.5%, Marwyn 20.5% (old 21.5 / 19.9 / 18.4).
Engine changes that can touch this deck: C3 (the t3 pool has Atraxa: infect is now combat damage and a poison threat, so its
share of eliminations moved 10.3% -> 9.1%, inside noise) and opponents casting their commander on time (+1.5). The deck has no
attack tax and no C2 card. C1/C1b and the rewire do not touch it. The net is noise.
Raw: `audit/baseline-2026-11-06/kefka.txt` and `kefka-2026-11-05-oldbuild.txt`.

## 2026-11-04: Sauron, the Necromancer for Contract Killing, Triple Triad for Alpharael (JD's choice; not simulated)
Sauron, the Necromancer turns the graveyard's creatures into attacking 3/3 copies that keep their abilities (an Archon
of Cruelty copy triggers on entering); Kefka keeps five removal spells and Counterspell. Triple Triad's free casts,
yours and the opponents', trigger Niv-Mizzet, Parun, Veyran, Guttersnipe and the other pingers; the risk is Kefka's low
curve (a cheap card on top frees little) and {R}{R}{R} on 6 Mountains. Alpharael was one of six looters. Measure over T3.

## 2026-11-04: Cabal Coffers for a Mountain (JD's physical deck; not simulated)
With Urborg out every land is a Swamp, so L lands make 2L - 3 mana for a Breach or Yawgmoth's Will turn. Without
Urborg it nets little (7 Swamps; it makes nothing until two other lands can pay its {2}). No number yet: the next
baseline (T3) measures it.

## Current state (2026-10-17, evening; superseded: the newest baseline is the 2026-11-04 section at the end)
Card report (adaptive AI, loose, 1,000 games per tier; `audit/kefka/2026-10-17-smaug/K0-analyze-adaptive.txt`):
T3 24.3%, T4 32.5%, T5 26.4% on the quick AI. Look-ahead baselines from the swap run below: **T3 26.9%, T4 30.5%,
T5 23.1% (26.8% over t3–t5).**

## Strengths
- **The combo finishers win when they land** (win rate when cast, every tier): Underworld Breach, Brain Freeze,
  Psychosis Crawler, Niv-Mizzet, the Firemind (38–68%).

## Weaknesses (by evidence)
1. **The combo pieces sit in hand:** Grapeshot, Jin-Gitaxias, Yawgmoth's Will, Brain Freeze, Underworld Breach and
   Massacre Wurm are cast in only 9–25% of the games they're drawn; Deflecting Swat and Slip Out the Back almost never.
2. **Kefka flips late:** online in about 34% of games, 6–7% by turn 8. Kefka resolves by turn 6 in 58% of games.
3. **Deaths are creature decks in combat** (T3: Aurelia 20%, Tergrid 15%, Marwyn 10%).
4. **Lowest win rate when cast (quick AI):** Bitter Triumph (bottom ten in all three tiers), Feed the Swarm (t3, t4),
   Rank Rat, Hallway Heckler, Grim Servant (t4).

## What was tried, and why it did or didn't work

### Kuja, Genome Sorcerer (2026-10-19; JD bought it; modeled Full in `impls/kuja.rs`, branch `thrasios-deck`)
Look-ahead, loose, T3, 2,000 paired games per swap (`audit/kefka/2026-10-19-kuja/`).
| Swap (Kuja in for) | Result at T3 | Why |
|---|---|---|
| **Hallway Heckler** | **+2.2 (30.2% -> 32.5%, ±2.4)** | Best of the two, just inside noise. Kuja is cast in 62% of the games he's drawn (round 7.5) and wins 46% when cast vs Heckler's 34%. His Wizard tokens double the deck's burn damage (2.7 -> 5.7 per game, beyond noise); total damage +2.8 per game. Heckler was stuck 45% of its draws. |
| Rank Rat | +1.7 (30.2% -> 31.9%, ±2.5) | Same picture (burn 2.7 -> 5.3 per game). |
**Reading it:** Kuja adds a steady burn clock to a deck that casts many noncreature spells. Like Smaug he's stuck in hand
38% of the time: {B}{R} in a mostly black deck. Proposed: Kuja in for Hallway Heckler (JD to approve).

### 2026-10-18: list matched to the physical deck (Decorum Dissertation in, Bitter Triumph out; no new run)
The 10/14 Jodah-leftover swap (Bitter Triumph for Decorum Dissertation) was never made in the physical deck, so the
repo list goes back to Decorum Dissertation (branch `printings`). That swap tested as noise (`audit/leftovers`), and
Bitter Triumph was the bottom-ten card when cast in all three tiers (weakness 4). The "Current state" numbers are from
the old list; the next run of the deck measures the new one.

### Smaug the Magnificent (2026-10-17 evening; modeled Full in `impls/smaug.rs`, branch `smaug`)
| Swap (Smaug in for) | Result over t3–t5 | Why |
|---|---|---|
| Rank Rat (adaptive, 1,000 paired per tier) | +1.0 (t3 +0.4, t4 +1.7, t5 +1.0; ±1.5–1.9 each) | Best of four on the quick AI. |
| Hallway Heckler (adaptive) | +0.9 (+0.6, +2.1, +0.1) | |
| Feed the Swarm (adaptive) | +0.5 (+0.8, +0.9, -0.1) | |
| Grim Servant (adaptive) | +0.4 (-1.6, +2.1, +0.6) | |
| **Rank Rat (look-ahead, loose, 1,000 paired per tier)** | **-0.1 (t3 +0.2, t4 -1.0, t5 +0.4; ±3.1–3.4 each)** | No effect. Smaug is stuck in hand 41% of the games he's drawn ({R}{R} with 5 Mountains in a mostly black deck) and is first cast in round 8.0 vs Rank Rat's 6.1. Wins 42% when cast vs Rank Rat's 32% (correlation). Eliminations -1.1 pts, flip +1.0 pt, both noise. |
**Reading it:** Smaug doesn't move Kefka. The red mana is the problem, and the deck's Treasure makers (Big Score,
Seize the Spoils, Contract Killing, Skullport Merchant) aren't enough to make his attack trigger matter.

### Ad Nauseam (2026-10-17 night; branch `adnaus`; JD: keep Rhystic Study)
Ad Nauseam was already modeled; the tests exposed AI misplays, fixed in shared code before the final numbers:
- **Old Ad Nauseam AI** (traces): cast in its own main phase, tapped out, flipped past a full hand (the cleanup
  discarded half) down to ~15–22 life, and died within two rounds in 4 of 8 traced casts.
- **New shared AI** (`8208586`): cast at the end of the turn before yours, mana held for it, stops at a usable hand
  (7 + next turn's mana / 3), life per card from the library's average mana value, a floor that counts every board.
- **Look-ahead hold credit** (`d973bea`, `search::hold_bonus`): the search overrode "hold mana" to tap out.
| Swap (Ad Nauseam in for) | Result over t3–t5 | Why |
|---|---|---|
| Feed the Swarm (adaptive, old AI) | -3.1 (-2.5, -3.2, -3.7) | The misplays above. |
| Hallway Heckler (adaptive, old AI) | -2.8 (-3.6, -2.0, -2.8) | |
| Hallway Heckler (look-ahead, old AI) | +0.4 (-0.7, -0.6, +2.4) | The search avoided some misplays. |
| Feed the Swarm / Hallway Heckler (adaptive, new AI) | -1.8 / -1.1 | Better, still slightly negative. |
| **Hallway Heckler (look-ahead, new AI + hold credit, 1,000 paired per tier)** | **-0.3 (t3 -1.8, t4 -1.4, t5 +2.4; ±3.1–3.5)** | Noise. Cast in only 15% of the games it's drawn (stuck 85%): Kefka keeps big hands, so there's rarely room for four cards, and `kefka::discard_choice` still discards it to the wheel (5 of 40 traced games). Wins 42% when cast. Baseline unchanged by the AI fixes (26.4% vs 26.8% over t3–t5). |
**Reading it:** Ad Nauseam is the weakest of its three homes here. Kefka's discard plan (hand-coded) treats it as fodder.

## 2026-10-18: band calibration (proposed swaps, NOT applied; see audit/bands/REPORT.md)
Band 3 (T3). Proposed: Demonic Tutor, Vampiric Tutor (to Seph FS), Mystical Tutor (to Veyran) out; Mana Leak, Expressive Iteration, Sleight of Hand (spares) in. 6 -> 3 Game Changers (Underworld Breach, Jeska's Will, Rhystic Study). Final: **26.7% vs T3** (current 27.4% at 1,000). Why: the combo is intact but found less often. Filler choice made no difference (Star Compass / Tome / Flourish gave 25.6%), nor did keeping Vampiric instead of Rhystic (26.1%).

## 2026-10-18: AI audit phase 2 (branch audit-grixis; adaptive AI vs t3, 600 games x 2 profiles, before -> after)
Win rate 22.2% -> 22.2% (no change; these are play fixes, not a power change). What moved: Deflecting Swat cast 4.0% -> 7.7% of draws
(it is no longer binned to Kefka's own trigger), Underworld Breach cast-in 24% with 44.8% win (kept in hand, not discarded), Psychosis
Crawler cast-in 27% -> 29%, Breach line pieces held on our turn 26.5% -> 28.5%.
Strengths: Kefka's discard engine, the Crawler loop, Breach line (every execution wins). Weaknesses: the Breach line needs storm >= the
table's life (three 40-life opponents), so most dry-run failures are real shortfalls, not a bug (traced 200 games: failures at
life 11-25 with 6-10 mana left over); Kefka dies to removal 0.25/game.
Tried and why: tutors now read the board (flipped Kefka and no Crawler/Niv -> Crawler/Niv, else the Breach line; Grim Servant reads the
same list within its devotion limit); Breach, Yawgmoth's Will and protection get discard keep values; Mana Leak into open mana is
valued x0.3; Profane Tutor suspends at 4.0 on a turn with no 1-2 mana play; doc: the Crawler is cast early, held only after the flip.

## 2026-10-18: AI audit re-measure (branch ai-audit; 2,000 games per profile, look-ahead)
**27.3% vs T3** (before 26.7): within noise. Tutors fetch Crawler/Niv for a flipped Kefka; his discard keeps Breach, Will and protection; Mana Leak discounted into open mana.

## 2026-10-18: lift Kefka to sit between band 4 and band 2 at T3 (APPLIED, JD approved)
Goal: after the AI audit Veyran (band 4) matched Kefka at T3 (27.7 vs 27.3), so Kefka needed about +2 while staying
under band 2 (Sauron 30.4, Sheoldred 31.9). JD's new cards: Fiery Emancipation and Bribery (Cecil, Oppression and
Marvin judged poor fits: Kefka doesn't attack, Oppression punishes its own storm turns, Marvin unmodeled).
Swap: Crux of Fate => Fiery Emancipation, Blazing Firesinger => Bribery (paired, vs T3, 2,000 games per profile).
1. Without Kefka AI rules: 27.4 -> 25.9 / 27.1 -> 26.9 (-0.8, noise). Bribery was **never cast** (Kefka's priority had
   no rule for it), Fiery Emancipation cast in only 31% of draws.
2. With two Kefka-only rules (Bribery 62; Fiery Emancipation 78 with Crawler/Niv out or in hand or after the flip, else
   45): **27.4 -> 28.6 / 27.1 -> 29.1 (+1.6 on average)**, positive in both profiles, inside each one's noise band.
   Bribery cast in 51-54% of draws (wins 46-50% when cast); Fiery Emancipation wins 53-60% when cast but is cast in
   only 23% of draws (six mana). The cut cards won 25% (Crux) and 30-35% (Firesinger) when cast.
Result: Kefka about 28.9 at T3: above Veyran (27.7), below band 2. The test ran on the list before #92 matched Kefka to
the physical deck (Bitter Triumph, now Decorum Dissertation): one card different. Outputs: 2026-10-18-lift/.

## 2026-10-18 evening: JD's new cards, coarse check (loose only, 500 games per list per tier, look-ahead; only big downgrades show, noise ±4-5 per tier)
- **Feed the Swarm => Oppression: T3 +1.6 (noise ±5.2, no downgrade).** Cast in only 44% of draws (held on combo
  turns, by design) and stuck 56%; when it lands the opponents lose cards for every spell while Kefka empties his hand.
- **Feed the Swarm => Marvin, Murderous Mimic (Partial: copies Niv-Mizzet, Fangtail, Blightspeaker's drain, Rummaging
  Goblin): T3 +0.2 (noise ±5.0, no downgrade).** Cast in 62% of draws, wins 33% when cast vs Feed the Swarm's 25%; a
  cheap body whose copied ability only matters with one of those creatures out.
Raw: `2026-10-18-new-cards/`. JD plans to add the cards unless one is a big downgrade.
**JD's decision (10/18):** Marvin in for Feed the Swarm (applied to the list); Oppression not added.

## 2026-10-31: card models (branch card-models-1; no sims until Phase 4 recalibration)

Price of Progress: Unmodeled -> Full (damage to each player for its nonbasic lands; the AI casts it when the opponents lose more). Glimpse the Unthinkable (new from Seph): Full (mills itself with Yawgmoth's Will or a reanimation spell in hand, else an opponent it can mill out). No new number yet.

## 2026-11-01: recalibration stage 2 remeasure

Kefka, t3: **13.6%** (CI 12.2-15.2) (2000 games, seeds 500000-501999, loose, look-ahead, defaults, rc=0; `audit/recal/stage2-2026-11-01.md` section 4, build efe45425). Stage 1, same seeds: 14.1%. The 10/18 AI audit number (27.3%, with the hand AI) is from an older build and list. Fitted t3 knobs: 12.9% (level).

## 2026-11-04: Urborg, Tomb of Yawgmoth vs a Swamp (paired, t3, loose, look-ahead, 2,000 games per list)

JD swapped a Swamp for Urborg on 11/02 (list-only, not simulated); this is its number. The run was set up as NEW=>OLD
(Urborg => Swamp), so the tool's "baseline -> variant" is the current list (Urborg) -> Swamp back in. **Flipped here to
read as Urborg's effect over a Swamp.** Raw: `.claude/worktrees/sim-runs-2026-11-04/kefka-urborg-vs-swamp.txt`.

- **Win rate: Urborg 18.3%, Swamp 18.1%; Urborg over Swamp +0.1 points (the tool prints -0.1 for the swap; the axis table
  rounds it to -0.2), noise +-1.1 (paired).** Verdict NO CLEAR EFFECT, well inside the noise. Only t3 (Kefka's single tier
  here); t4 and t5 were not run.
- **The model has Urborg working:** it makes every land a Swamp, so every land can tap for {B} (`engine/mana.rs`,
  `impls/common.rs`, tests `tests/lands_urborg_coffers.rs`, main as of 11/04). The preflight shows the Swamp is only played,
  never cast; Urborg is drawn in 29.1% of games, the Swamp in 87.6% (it is one of eight in the swapped list).
- **Why nothing moves, from the numbers:** every axis is flat inside its band. Mana available on turn 4 is 3.81 with Urborg
  vs 3.82 with the Swamp, turn 6 5.18 vs 5.18, turn 8 6.62 vs 6.61; missed land drops in turns 1-5 0.61 vs 0.61; extra
  mulligans 0.11 vs 0.11; the commander resolves by turn 4 in 14.8% vs 14.8% and by turn 6 in 66.1% vs 65.8%; plan online at
  all 15.6% vs 14.9% (inside +-2.3), plan turn 9.40 vs 9.47; spells cast per game 10.46 vs 10.45; rounds survived 9.20 vs
  9.21; damage per game 30.2 vs 29.9; commander removals 0.22 vs 0.22. Nothing the deck does changes.
- **The "already plenty of {B}" reading is plausible but not shown by this run.** The breakdown has no coloured-mana count,
  only total mana. From the list (`data/decks.json`, my count) the deck already has 7 Swamps, Blood Crypt, Haunted Ridge,
  Sulfurous Springs and Underground River (11 lands that make {B}) plus Command Tower, Exotic Orchard and Gemstone Mine,
  and the swapped list has an 8th Swamp. Urborg adds {B} from the other lands (the Islands, Mountains and the blue-red
  duals), which would matter only when black is the missing colour. Showing that would need a coloured-mana metric (stuck
  for black, black spells cast), which the explain output does not give.
- **Wins by damage 100% in both lists;** eliminated by Aurelia, Tergrid and Marwyn in the same order and shares either way
  (Aurelia via combat 15.6% vs 16.0%, Tergrid 11.7% vs 11.1%).
- **Decision input, not a recommendation:** at t3 Urborg is neither better nor worse than a Swamp to within about +-1.1 points.
  Its non-simulated risks and benefits (it also makes opponents' lands Swamps; Blood Moon turns it off) were not measured.
- **Baseline note:** Kefka's 18.3% at t3 (current list) is not like-for-like with the 11/01 remeasure (13.6%, CI 12.2-15.2): the
  list and the build may have changed since (Ghave breakup 11/02, JD's 11/01 Palantir swap, Urborg now tapping for {B}). The cause of the
  difference was not investigated here.
