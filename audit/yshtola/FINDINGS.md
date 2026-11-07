# Y'shtola, Night's Blessed — findings (living file)

What the simulator has taught us about the Y'shtola Esper drain deck: strengths, weaknesses, and what was tried and
why it did or didn't work. Updated after every run. Newest evidence first. Matched tier: T5 (`data/my_decks.toml`).

## 2026-11-07: card-pile package Y, adopted untested
Memory Trap => Counterspell, Dark Petition => Ringsight, Ill-Gotten Inheritance => Palantír of Orthanc, Island =>
Mistrise Village (JD's spare-card pile, `audit/jd-packages/2026-11-06-card-pile/`). **Not measured:** JD stopped the runs
and adopted it. Why: she loses about 35% of t5 games to the Kinnan and Urza combos, and Counterspell answers a combo
piece at instant speed; Ringsight ({1}{U}{B}, a tutor for a white, blue or black card with her out) costs 3 where Dark
Petition cost 5, and both trigger her; Palantír (mana value 3, triggers her) drains or draws every end step where
Ill-Gotten Inheritance drained 1 a turn; Mistrise Village makes the next spell uncounterable.

**A correction to the reason given when it was chosen:** Memory Trap was described as a conditional counterspell. It is
a {2}{W} enchantment that exiles a nonland permanent an opponent controls until it leaves: removal that triggers her
(mana value 3) and that Zur can fetch. The swap trades it for a counterspell that doesn't trigger her (mana value 2).
JD kept it (2026-11-07) over cutting Arrest instead. **Engine fix with it:** Ringsight used to search for a blue, black
or red card (Sauron's colours, `tut=ubr`); it now finds a nonland card that shares a colour with a legendary creature
you control, the Ring-bearer included (`tut=legcol`, `ai::decks::tut_kind`, `tests/cards_ringsight.rs`). The preflight
cast Ringsight in 82% of the games it was drawn and Counterspell in 57% (t5, 200 games, before the fix). To check in the
next baseline: the combo losses (Kinnan, Urza) and Palantír's cast rate (44% in the preflight).

## Baseline B 2026-11-06 (after #278-#284; main 63d783b5; look-ahead, loose, 2,000 games per tier, seed 500000)
**26.5%** (24.6-28.5) at t5, against 26.3% in the morning: no change. The Tier 5 table changes of #284 (no wasted protection
holds; Garruk's Uprising and Welding Jar cast) and #281 (Urza's Whir) don't move her number. Raw:
`audit/baseline-2026-11-06-b/yshtola.txt`.

## Baseline 2026-11-06 (after the rewire, Doomsday and Phase C engine work)
Y'shtola, t5, 2,000 games, seed 500000, loose, look-ahead, integration build ef343cc1: **26.2%** (95% CI 24.3-28.2) over t5
(1,994 games counted: 6 hit an internal simulator error, see below). Same command on main 1e027077 (2026-11-05, old build): **25.8%**
(23.9-27.8). Change +0.4: noise (intervals overlap almost completely, under 2 points). Plan online 65.4% (old 63.5%); wins by
damage 66% / combo 34% (old 70 / 30).
Engine changes that can touch this deck: C2 Loran of the Third Path (fires less, picks better targets); C3 attack taxes no
longer idle attackers (it plays Propaganda and Sphere of Safety; Toxic Deluge is not affected) and opponents cast their commander on time
(+1.5), which changes the t5 opponents' play. C1/C1b and the rewire do not touch it. Nothing here is a measured effect of any
one change: the net is noise.
**Re-run after the crash fixes:** the 6 skipped games were the same look-ahead `cast_counter` panic as Thanos's (latent on main,
fixed on #274); a first re-run then skipped one game on an overflow of the "life gained this turn" counter (Exquisite Blood off a
doubled Dragon army, seed 501953), also fixed on #274. Final re-run (integration 23ecd983): **26.3%** (95% CI 24.4-28.3) over t5, all
2,000 games, no errors; plan online 65.5%; damage 66% / combo 33%. Against the old build 25.8%: noise. This is the number of record.
Raw: `audit/baseline-2026-11-06/yshtola-rerun.txt`; first run `audit/baseline-2026-11-06/yshtola.txt` and `yshtola-2026-11-05-oldbuild.txt`.

## Current state (2026-11-04)
Look-ahead, loose, current list (Cabal Coffers in for a Plains): **T5 24.9%** (95% CI 23.0-26.8), 2,000 games.
Her losses are still fast combo (Kinnan, Urza, Zur) and Miirym combat, about 49% of games.
**Applied 11/04 (JD):** the Coffers package with Urza's Cave (Demonic Tutor, Exquisite Blood, Torment of Hailfire,
Expedition Map, Urza's Cave for Deep Analysis, Prison Sentence, Chromatic Lantern, Static Net, an Island). Measured
26.1% vs T5 (+1.3, noise ±2.4) with the board-aware land choice before the Cave/Map utility change; the list as
applied (that utility change, Torment in the plan file's X-drain wish) is not measured yet.
**Revised 11/04 (JD's physical list; not simulated):** Demonic Tutor took Bribery's slot and Profane Tutor took Hostage
Taker's, so the package's Deep Analysis slot goes to **Loran of the Third Path** instead: a blink target (Displacer
Kitten, Restoration Angel, Conjurer's Closet, Teleportation Circle) whose enter trigger destroys an artifact or
enchantment each time, and whose tap draws a card (the opponent draws too). Seven tutors; no theft left. Measure over T5.

## Weaknesses (by evidence)
- **Lowest win rate when cast** (adaptive card report, `audit/yshtola/2026-10-17-adnaus/K0-analyze-adaptive.txt`;
  nearly every card is above the deck average): Mystic Remora (39%), Lightning Greaves, Luminous Bonds, Prison
  Sentence, Esper Sentinel, Curiosity, Chromatic Lantern, Thought Vessel (43–44%).

## What was tried, and why it did or didn't work
### 2026-11-04: Cabal Coffers package (T5, loose, look-ahead, 2,000 games per list, seed 500000; branch yshtola-coffers)
**Baseline, Cabal Coffers in for a Plains** (JD's physical list, 11/04): **24.9% vs T5** (95% CI 23.0-26.8). Not a
measured Coffers effect: the 11/01 number (22.4%) is an older build. Raw: `2026-11-04-coffers/baseline-t5.txt`.

**Package**: Deep Analysis => Demonic Tutor, Prison Sentence => Exquisite Blood, Chromatic Lantern => Torment of
Hailfire, Static Net => Expedition Map, Island => Tolaria West (Diabolic Tutor kept: six tutors).
Result: **24.9% -> 24.7%, -0.1 (noise ±2.4): NO CLEAR EFFECT.** Raw: `2026-11-04-coffers/package-t5.txt`.
Why:
- **It changes how she wins, not how often.** 29% of the package's wins are the Sanguine Bond + Exquisite Blood combo
  (current list: 99% damage). Exquisite Blood is cast in 68% of the games it is drawn (round 7.6) and wins 61% of those,
  the best card in the package; it takes wins the drain plan would have got anyway.
- **Her losses don't move**: Urza, Kinnan and Zur combo plus Miirym combat (about 49% of games both ways). Games end at
  about 9 rounds, and the big-mana half arrives too late: Torment of Hailfire is cast in only 39% of the games it is
  drawn, at round 9.2 (stuck in hand 61%).
- **The cuts cost tempo.** Games end 0.3 rounds sooner (±0.2) and the plan comes online 3.4 points less often (±3.0).
  She draws 1.5 cards fewer per game (Deep Analysis out), casts 0.8 spells fewer and 0.2 spot removal fewer (Prison
  Sentence, Static Net out), and has 0.16 less mana on turn 6 (Lantern out; Tolaria West enters tapped). Damage to
  opponents falls 4.2 a game.
- Engine fix for the test: Torment of Hailfire's cast option (`impls/threats.rs` torment_options) was Sauron/Tergrid-only,
  so her AI never cast it (preflight: 0 of 48). Y'shtola now gets it, with a test. Tutors don't name it in her plan file
  (a plan file may name only cards in the deck).
Next candidate (not run): Exquisite Blood alone for Prison Sentence, the one piece that pulled its weight.

**Same package, Urza's Cave for the Island instead of Tolaria West**: **24.9% -> 26.1%, +1.3 (noise ±2.4): NO CLEAR
EFFECT.** Raw: `2026-11-04-coffers/package-cave-t5.txt`. Same seeds, so the 1.4 points over the Tolaria West version is
also noise. Same shape: 27% of wins by the Blood combo (Blood cast 68%, wins 63% when cast), the same losses (Kinnan,
Urza, Zur, Miirym), plan online -4.5 (±3.0), 1.35 cards fewer drawn, damage -3.4 a game. **Caveat: the Cave never
fetched anything for her.** Its AI (`impls/karn_engines.rs`) cracks it only for a Tron piece, Mishra's Workshop or
Ancient Tomb, so here it was a colourless land; Urborg and Coffers aren't on its want list. A real Cave test needs
that AI rule first.

**Same Cave package, board-aware land choice** (`karn_engines.rs` board_land_want: Coffers/Urborg as a pair, Serra's
Sanctum, a colour the hand needs; tests `tests/land_search_board.rs`): **24.9% -> 26.1%, +1.3 (noise ±2.4): NO CLEAR
EFFECT**, 12 threads (shared CPU). Raw: `2026-11-04-coffers/package-cave-board-t5.txt`. 60 traced games
(`2026-11-04-coffers/traces/`) show why it matches the run before: the Cave was played in 10 games and its option
offered 9 times (e.g. Coffers out, no Urborg, 10 lands), but **it was never chosen**: its utility is a flat
1.2 + want/6 (about 3), below Champion's Helm, Sphere of Safety or holding mana, not scaled to the mana Urborg would
add. Expedition Map now cracks (twice in the 60: Cabal Coffers, Flooded Strand); before this fix it never did, so the
two earlier package runs had a dead Map too. Open: the Cave's and the Map's option utility should grow with the mana
the fetched land adds.

**Follow-up, not measured:** `search_utility` (`karn_engines.rs`) now scores a board-based search at 1.2 + want/2
(Urborg beside Coffers on 8 lands: about 6), Tron/Workshop wants unchanged at 1.2 + want/6; tests in
`tests/land_search_board.rs`, full suite passed. The package rerun was stopped at JD's request (11/04), so there is no
number yet for a Cave that actually cracks.

### 2026-10-19: Curiosity => Orcish Bowmasters, confirmed (T5, loose, 2,000 games per list, look-ahead)
Result: **31.6% -> 34.1%, +2.5 (noise ±2.3): BETTER**, the first swap to clear the noise for her. Raw:
`2026-10-19-bowmasters/full-curiosity-t5.txt`.
Why: Bowmasters is cast in 81% of the games it is drawn (Curiosity 68%, stuck in hand 32%), around round 5.9, and
wins 48% of the games it is cast in. Damage to opponents +2.3 a game and the plan online +2.4 points (60.3 -> 62.7%),
same game length (9.4 rounds). The losses to Urza, Zur and Winota combo fall slightly (all within noise): the ping
and the Orc Army pick off the fast decks' mana creatures and small pieces. Recommended: buy a second Bowmasters for her.

### 2026-10-19: Orcish Bowmasters for three cards (quick read: T5, loose only, 700 games per list, look-ahead)
| Cut (=> Orcish Bowmasters) | T5 (noise ±4.0-4.1) |
|---|---|
| Curiosity | +3.1 |
| Mystic Remora | +2.0 |
| Lightning Greaves | +3.3 |
**Leans good: about +2.8, every cut positive, none beyond noise alone.** The three share one baseline (30.9%), so they
are not independent; a full run of one cut is needed to confirm. Raw: `2026-10-19-bowmasters/<cut>-t5.txt`.
Why: Bowmasters is cast in 79-86% of the games it is drawn, early (round 5.4-5.8), and wins 45-56% of the games it
is cast in (the deck averages about 31%). Damage to opponents rises 1.8-3.6 a game in all three, and the plan comes
online slightly more often (+0.9 to +3.3). The ping kills T5's mana creatures and 1-toughness pieces, and the fast
tables' own extra draws feed it. JD would buy a second copy (the Sauron copy stays).

### 2026-10-19: Talisman of Progress => The Mind Stone (T5, 2,000 games per list per profile, look-ahead)
Result: **+2.5 conservative (±2.0) / -1.0 loose (±1.9): about +0.75 averaged, no clear effect** (the tool's
"leans better" rests on one profile, and the other goes the other way). Raw: `2026-10-19-mind-stone/t5.txt`.
Why: a rock for a rock. Mana on turns 4 and 6 and the plan (online in about 61% of games, turn 6.9) are unchanged.
The Mind Stone is cast in 67-73% of the games it is drawn (Talisman 67-68%), around round 5.2-5.5, and stuck in hand
in 27-33%, as much as the Talisman: her rocks are often skipped for better plays, so it is not a Mind Stone priority
problem. Her losses are the same fast combo either way (Urza 15%, Zur 13%, Winota 11-20%), which ends T5 games at
about 9.4 rounds, before the {5}{W} harness matters. The 10/18 quick test (Chromatic Lantern out, loose only) was
also noise (+0.4). Verdict: not an upgrade for her.

### AI fix: cards her rules don't name (2026-10-17, `9b7e19a`)
`yshtola_prio` returned 0 (never cast by priority) for any card it didn't name: a swapped-in Ad Nauseam was cast in
0.5% of the games it was drawn, and **Dakkon, Shadow Slayer in her list was never cast**. Unnamed cards now get the
pool AI's generic priority (`yshtola_named_prio` holds her rules). T5 baseline 31.1 → 31.6% (adaptive, noise).

### Ad Nauseam (2026-10-17 night; branch `adnaus`; JD: keep Esper Sentinel)
The shared Ad Nauseam AI and the look-ahead's hold-mana credit were fixed during these tests (audit/kefka/FINDINGS.md).
| Swap (Ad Nauseam in for), T5 | Result | Why |
|---|---|---|
| Mystic Remora / Curiosity (adaptive, new AI) | -0.9 / +1.0 | |
| Curiosity (look-ahead, before the hold credit) | -2.0 (±3.2) | The search tapped out: Ad Nauseam cast in 5.7% of the games it was drawn. |
| **Curiosity (look-ahead, hold credit)** | **-0.2 (±3.2)** | Cast 15%, wins 56% when cast vs Curiosity's 39% (cast 80%). |
| **Thought Vessel (look-ahead, hold credit)** | **-1.4 (±3.2)** | Cast 17%, wins 58% when cast; Thought Vessel cast 70% (no maximum hand size, useful with Necropotence and Citadel). |
| **Mystic Remora (look-ahead, hold credit)** | **-1.9 (±3.1)** | Cast 14%, wins 54% when cast; Mystic Remora cast 75%. |
**Reading it:** Ad Nauseam wins when it's cast but is rarely cast: T5 games are short (about 8.5 rounds) and the deck
pays life already (Necropotence, Bolas's Citadel), so the safe window seldom opens. Not a fit; Sheoldred is its home.

## 2026-10-18: band calibration (proposed swaps, NOT applied; see audit/bands/REPORT.md)
Band 1 (T5). Proposed: +Force of Will, +Farewell, +Teferi's Protection (all from Seph/Atraxa) for Dakkon, Thought Vessel, Bolas's Citadel (to Seph FS). 9 Game Changers. Final: **24.3% vs T5** (current list 23.2%). Why: Force of Will is cast 76% of the games drawn and answers T5 combo; Farewell and Teferi's Protection are mostly held (stuck 68-75%) but win 45-60% when cast. Citadel was cast in only 25% of the games Y'shtola drew it, so it was the cheapest card to give up.

## 2026-10-18: AI audit phase 2 (branch audit-ysh; problems 3-6 of audit/ai-audit/yshtola.md)
Measured at T5 (her tier), adaptive AI, 4,000 games per list, conservative/loose win rate: before 32.3 / 31.6; after
31.7-32.0 / 31.9-32.0. No change beyond noise (one run's SE is 0.7); the fixes make the plays right, they do not move the
win rate. Weakness this leaves: she still loses 35% of games to Kinnan/Urza combos, which spot removal cannot reach in time.
- **Removal and her trigger.** `removal_adjust` (impls/yshtola.rs): a noncreature spell of mana value 3+ with her out
  (Anguished Unmaking, Generous Gift, Your Fate Ends Here) earns `trigger_bonus / 4` and needs a target worth 1; every
  other removal needs a target worth 3 (before: Path on Ornithopter, Push on Esper Sentinel). `trigger_bonus` adds 3 when
  an opponent already lost 2-3 life this turn (one more trigger makes four and she draws). The same check applies to the
  attack-response removal. The unplayed 4-player test: choosing the single best target instead of sampling cost -1.3
  points (31.0 vs 32.3), so the sample stays, restricted to targets at least half as valuable as the best.
- **Farewell / Austere Command.** The wipe also earns the trigger bonus. With Teferi's Protection in hand and its
  {2}{W} left over, the wipe is valued without her own losses and cast with Teferi's in response (her permanents phase
  out, the opponents' are exiled): `teferis_first`, `cast_wipe`. The opponent-end-step version of the second trigger
  is not possible: the AI's end-of-turn window runs after the end-step triggers have been checked.
- **Mystic Remora 66 -> 40, Esper Sentinel 62 -> 46** (early casts; the two were in the bottom group by win rate when
  cast). Neutral on the win rate (31.6 / 32.0 with both); kept because Remora was being cast and sacrificed unpaid.

## 2026-10-18: AI audit re-measure (branch ai-audit; 2,000 games per profile, look-ahead)
**24.5% vs T5** (before 24.3): no change. Fixes changed play (trigger timing, removal targets, Teferi's Protection as the full card, Mystical Tutor finishers), not the win rate. Details: audit/ai-audit/yshtola.md, REPORT.md.

## 2026-10-18 evening: Chromatic Lantern => The Mind Stone (quick pass: loose only, 700 games per list, T5, look-ahead)
Result: **24.4% -> 24.9% (+0.4, ±3.3): noise.** Raw: `2026-10-18-mind-stone/mind-stone-quick.txt`.
Why: a better rock than the Lantern (cast 70% of the games drawn vs 51%; mana on turn 8 +0.13), but her losses are
fast combo (Kinnan 20%, Urza 16%, Zur 11%, Winota 11%) and the 6-mana harness (blink Hostage Taker, Massacre Wurm,
Gonti) comes too late. Open AI question: The Mind Stone sits in hand in 30% of the games it's drawn; her rules
don't name it (generic priority), so a re-test should check that first. JD decided to wait on buying it.

## 2026-10-18 evening: JD's new cards, coarse check (loose only, 500 games per list per tier, look-ahead; only big downgrades show, noise ±4-5 per tier)
**Prison Sentence => Stroke of Midnight: T5 -2.6 (noise ±4.3, no big downgrade).** Cast in 80% of draws (Prison
Sentence 67%), but it doesn't change her losses (fast combo: Kinnan, Urza, Zur, Winota). Cloud is the other home
(-1.4 there); JD has one copy. Raw: `2026-10-18-new-cards/`. JD plans to add the cards unless one is a big downgrade.

## 2026-10-31: card models (branch card-models-1; no sims until Phase 4 recalibration)

Venat, Heart of Hydaelyn // Hydaelyn: Partial -> Full. Its card code had never run (registered under a name not in the data); before this the deck played it as a 3/3 that drew on every legendary spell. No new number yet.

## 2026-11-01: recalibration stage 2 remeasure

Y'shtola, t5: **22.4%** (CI 20.6-24.2) with the hand AI on (default) (2000 games, seeds 500000-501999, loose, look-ahead, defaults, rc=0; `audit/recal/stage2-2026-11-01.md` section 4, build efe45425). `AI_HAND=0` (generic + plan file): 21.9%, level (+0.5 [-2.1, +3.1]). Stage 1, same seeds: 19.8% (hand AI on), 15.7% (off). The 10/18 AI audit number (24.5%) is from an older build and list. Fitted t5 knobs: 23.1% (level).

## 2026-11-04 (late): AI changes, not simulated (JD: no sims for this batch)

- **Zur the Enchanter**: an Aura it finds goes on a legal host (an opponent's Aura on their best creature; lock Auras such as Arrest attach properly), not always on Zur; with no legal host it isn't fetched.

The numbers above predate these changes; a re-run on the deck's own tiers would show their effect.
