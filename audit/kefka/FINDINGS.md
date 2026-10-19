# Kefka, Court Mage — findings (living file)

What the simulator has taught us about the Kefka Grixis discard deck: strengths, weaknesses, and what was tried and why
it did or didn't work. Updated after every run. Newest evidence first. Matched tiers: t3–t5 (`data/my_decks.toml`).

## Current state (2026-10-17, evening)
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
