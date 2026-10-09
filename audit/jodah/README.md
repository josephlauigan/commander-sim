# Jodah audit (2026-10-08): how the AI pilots Jodah, and why it loses to Tiers 1-3

Deck: `decklists/JD/jodah-wubrg-legends.md` (key `jodah`). The starting point was the October 7 look-ahead run: 12.4%
over all tiers (10/10/9/15/18% for T1-T5), the only one of JD's decks that does better against stronger tiers.

Unless noted, numbers are from the adaptive AI, loose profile. "Paired" means the same seeds before and after (same
opponents, same draws); the noise band is ±2 standard errors of the per-seed differences. The instrumented baseline is
2,000 traced games (400 per tier, `jodah_stats.py`, before the AI changes); the A/B runs are 1,000-2,000 games per tier.

## Summary

| # | Problem | Kind | What it costs (measured) |
| --- | --- | --- | --- |
| 1 | Jodah gets removed 1.0-1.6 times a game at T1-T4 (0.35 at T5) and the deck has little to stop it | deck weakness, small modeling gap | **up to 15 points** (16-21 at T1-T4, 6 at T5): removal-proof Jodah diagnostic |
| 2 | Five colours: on its first turn with 5 mana Jodah is castable in 55% of games (never in 12%); double-pip cards get stuck | deck weakness (mostly), modeling gaps | **up to 8 points** (all colour costs free); 1.9 for Jodah's WUBRG alone |
| 3 | The AI holds back: mana kept up for counters, creatures kept home | AI misplay (play style) | **+1.9 ± 0.8** fixed (caution 0.65 → 0.30) |
| 4 | Random sampling picks a clearly worse play (Szadek at 5% over Jodah at 94%) | AI design (temperature 1.0 for every deck) | +1.6 ± 1.1 at temperature 0.5; not changed |
| 5 | Legends cast while Jodah was castable (0.2-0.5 a game): the cascade is lost | AI misplay | fixed; +0.4 ± 0.5 (cascades +1-3 points) |
| 6 | Tutors fetch Sisay's Ring (44% of Demonic Tutors), Helm of Kaldra alone (68% of Enlightened) | AI misplay | fixed; +0.7 ± 0.6 |
| 7 | Plaza of Heroes never made coloured mana for legendary spells | modeling gap (bug) | fixed; 0.0 ± 0.4 |
| 8 | Dead cards: Kaervek's Purge never cast; Desertion 12%, Dismiss 23%, Invoke Despair 29%; Szadek deals no damage | deck weakness | see suggestions |

All changes together: **+1.1 ± 1.2** on the adaptive AI (the same 5,000 seeds as the original code; measured one at a
time on more games the parts look larger: style +1.9 ± 0.8, sequencing +0.4 ± 0.5, tutors +0.7 ± 0.6). On the
look-ahead AI the check was too small to show anything:
**−0.3 ± 2.4** over 1,170 paired games (T1-T3 +2.9 ± 3.5; T4-T5 −2.8 ± 3.3), see "Look-ahead check" below.
The deck stays near the bottom either way: what holds it back is the list (a creature commander that the lower tiers
remove 1-1.6 times a game, little protection, a five-colour mana base), not how it is piloted.

## Why Tiers 1-3 beat it

| | T1 | T2 | T3 | T4 | T5 |
| --- | --- | --- | --- | --- | --- |
| Win rate (adaptive) | 16% | 16% | 17% | 19% | 20% |
| Jodah removed per game | 1.09 | 1.57 | 1.25 | 0.96 | 0.35 |
| ...by targeted removal / wipes | 0.54 / 0.35 | 0.91 / 0.45 | 0.51 / 0.33 | 0.47 / 0.28 | 0.15 / 0.07 |
| Eliminations by combat damage | 76% | 83% | 81% | 40% | 20% |
| Median turn of death | 9 | 11 | 10 | 9 | 7 |
| Jodah out on its last turn | 36% | 27% | 32% | 29% | 26% |
| A cascade happened | 55% | 56% | 55% | 50% | 38% |
| Win rate with / without a cascade | 27% / 3% | 27% / 2% | 31% / 1% | 33% / 6% | 46% / 3% |
| Removal-proof Jodah (diagnostic) | +17.6 | +20.8 | +16.0 | +15.6 | +6.1 |
| Colourless costs (diagnostic) | +9.3 | +9.8 | +7.8 | +6.9 | +5.1 |

- **They kill the engine.** Tiers 1-3 run creature removal and wipes (Swords, Path, Chaos Warp, Council's Judgment,
  Brago's bounce, Supreme Verdict, Sunfall, Kaya's Wrath, Elspeth, Damnation): Jodah leaves the battlefield 1.1-1.6
  times a game, against 0.35 at Tier 5, whose decks carry counterspells and combos instead. The deck wins almost only
  through Jodah (1-3% of games without a cascade), so every removal spell on Jodah is close to a lost game. Making Jodah
  untouchable by removal (`--diag safejodah`) adds 16-21 points at T1-T4 and only 6 at T5: that difference is most of
  the inverted tier curve.
- **They kill it in combat, slowly.** 76-83% of the eliminations at T1-T3 are combat damage, at turns 9-11, when Jodah
  is out on its last turn only a third of the time and it has about one creature ready. Tier 4-5 losses are combo or
  other damage at turns 7-9: those games are decided by whether a combo deck goes off, and Jodah is no worse at that
  than any other deck. When the combo doesn't happen, Tier 5's decks have few attackers and little creature removal,
  and a resolved Jodah wins 46% (27-31% at T1-T3).
- **Slow mana makes it worse in long games.** Jodah's first cast is on turn 6 (median); it is out by turn 5 in 28% of
  games and by turn 6 in 51%. The plan comes online as often at T1-T3 as at T4 (55%), but the boards it meets there
  are bigger and keep coming.
- The look-ahead AI widens the gap (10/10/9 vs 15/18 in the overnight run). Likely reason (not measured): it makes
  the creature decks better at attacking and at choosing removal targets, which is what beats Jodah.

## Findings in detail

### 1. Jodah's survival (deck weakness, a small modeling gap)
- Protection in the 99: Privileged Position, Shield of Kaldra (equip {4}), Grand Abolisher and Dragonlord Dromoka
  (your turn only), five counterspells. Removal aimed at Jodah met a held counterspell only 31 times in 2,000 games
  (10 countered): the counters are rarely in hand at the right time, so tuning when to counter can't help much.
- Raising Privileged Position's priority once Jodah is out was tried: **−0.7 ± 0.4**, dropped (it delays legends).
- Modeling gap: Plaza of Heroes' "{3}, {T}, exile: a legendary permanent gains hexproof and indestructible" is not
  implemented (noted in its card status). It would save Jodah once in some games; not done here.

### 2. Colours (deck weakness; modeling gaps)
- The first turn with 5 mana, Jodah is castable in 55% of games; in 12% it never becomes castable (50% of all turns
  with 5+ mana and Jodah not yet cast).
  Cards stuck on colours (castable by total mana, not by colours) on 15% of own turns: Grand Abolisher (WW), Kaervek's
  Purge (BR + X), the single-pip tutors, Genesis Hydra (GG), Wrenn and Six (RG), Dakkon (WUB).
- Mana: 3.7 lands on turn 4, 5.0 on turn 6, 0.7 missed land drops a game, 1.2 mana unspent after the second main
  phase. 36 lands, 10 of them tri-lands (tapped), 10 painlands, 8 basics; about 16 sources of each colour.
- **Bug fixed:** Plaza of Heroes read `PAY_FOR` from its own module (a copy taken at import, always None), so it never
  gave any colour for legendary spells. Now `E.PAY_FOR` (same fix for Unclaimed Territory, which no current deck plays).
  Measured 0.0 ± 0.4: it is one land.
- Modeling gap, not fixed: hybrid costs take the first colour (autotag `conv_cost`), so Privileged Position costs
  {2}{G}{G}{G} instead of {2}{G/W}{G/W}{G/W}. Engine-wide; changing it would change other decks' games.
- Diagnostics: all of Jodah's costs colourless +7.8 ± 1.4; Jodah's own WUBRG colourless +1.9 ± 1.0. Most of the colour
  cost is the 99's heavy pips (Razia RRWW, Sisters BBGG, Szadek UUBB, Invoke Despair BBBB, Dakkon Blackblade WUUB).

### 3. Play style: holding back (AI misplay, fixed)
Jodah's style was aggression 0.60 / caution 0.65. Caution sets how much mana the AI keeps up for its counters and how
many creatures stay home as blockers. Paired, 1,000 games per tier, before the other fixes:

| aggression / caution | change |
| --- | --- |
| 0.4 / 0.85 | −0.4 ± 0.7 |
| 0.8 / 0.65 | +0.4 ± 0.4 |
| 0.6 / 0.40 | +1.3 ± 0.7 |
| 0.8 / 0.45 | +1.3 ± 0.7 |
| **0.9 / 0.30** (now) | **+1.9 ± 0.8** |

Caution is the lever. The counters are rarely worth the mana (13% of the opposing spells they could answer are
countered, mostly commanders), and the deck's creatures are better used attacking than waiting.

### 4. Decision noise (AI design, not changed)
Every deck samples its plays at temperature 1.0. Jodah at 0.5: +1.6 ± 1.1. Example (T3, seed 700013, before the fixes),
turn 7 with Jodah castable:

```
R7    Jodah casts Szadek, Lord of Secrets
R7        [Jodah decides: Jodah, the Unifier 94%, Szadek, Lord of Secrets 5%, Korlash, Heir to Blackblade 1% ...]
```
The `first` fix below makes this particular mistake rarer; the temperature is a global choice ("human" play), so it
is left alone.

### 5. Sequencing around Jodah (AI misplay, fixed)
- Jodah is cast the turn it first becomes payable in 76% of games, later in 17%. A legend from hand was cast on a turn
  Jodah was payable, before it, 0.2-0.5 times a game: that legend's cascade is lost. Only 38% of the legends cast from
  hand had Jodah out.
- Fix (`jodah_prio`, switch `first`): while Jodah is castable, legends wait (priority 15) and so does any other spell
  that would leave too little mana for Jodah. Cascades +1-3 points per tier; win rate +0.4 ± 0.5 (2,000 games per tier).
- Holding expensive legends until Jodah is out on later turns was not tried (it trades a turn of board for a cascade).
- Cascades: 1.2 a game; the commonest hits are the cheap legends (Blackblade Reforged, Wrenn and Six, Urza, Helm of
  Kaldra, Dakkon, King Darien), because the deck's dearest legends are cast first (prio +2 per mana value), as intended.

### 6. Tutor targets (AI misplay, fixed)
Before: Demonic Tutor found Sisay's Ring 44% of the time, Vampiric 42%; Enlightened Tutor found Helm of Kaldra 68%
(Kaldra was assembled in 16 of 2,000 games); Worldly found Fyndhorn Elder 45%; Mystical found Demonic Tutor 54%.
The generic pick takes the highest cast priority up to turn 4 (a rock: 85) and "impact" after.

```
R3    Jodah casts Demonic Tutor
R3      Jodah tutors Sisay's Ring          (T2, seed 700034: Jodah comes down on turn 7)
```
Fix (`jodah_tutor`, switch `tutor`): before Jodah with under 4 mana, Coalition Relic and the colour sources; the third
Kaldra piece when the other two are in hand or out; else the dearest legendary creature (Razia, Sisters, Dromoka;
not Szadek) unless one is already in hand; instant/sorcery tutors take Toxic Deluge under heavy pressure, then
Demonic Tutor or Force of Will; Enlightened Tutor takes Blackblade, Privileged Position, Coalition Relic. Enlightened
Tutor's "artifact or enchantment" search now reaches the wish list (the DSL search kind `ae`).
Measured +0.7 ± 0.6 (2,000 games per tier).

### 7. Other observations
- **Counterspells:** 13% of the opposing spells Jodah could counter were countered: mostly commanders (Atraxa, Kaalia,
  Windgrace, Brago). It lets rocks through (Arcane Signet, Sol Ring), correctly. Force of Will 286 counters, Arcane
  Denial 238, Dissipate 124, Desertion 49, Dismiss 23 (2,000 games).
- **Mulligans:** the shared rule (keep 2-5 lands) keeps seven 96% of the time; mulligans don't explain anything.
- **Attacks:** creatures ready to attack in only 36% of its combats; it attacks in 88% of those. With Jodah out: 2.5
  legends, 16 power ready, attacked 64% of the time (before the style fix). Blackblade Reforged is equipped in 29% of
  the combats it is out (equip {3} competes with casting legends). Combat damage dealt: 21-27 a game.
- **Cards that sit:** Kaervek's Purge is never cast (406 games seen): X is the target's mana value, so a 5-drop costs
  {5}{B}{R}, and its value is low. Desertion (12%), Dismiss (23%), Dissipate (34%) are held counters. Invoke Despair
  29% (BBBB). Dakkon Blackblade 36%, Szadek 43%, Sisters 43%, Razia 44%: expensive and colour-heavy. Profane Tutor
  (seen in 386 games, suspended in 250) sat in hand with {1}{B} available for 839 turns: its suspend option is valued
  2.2, below most plays.
- **Szadek** turns its combat damage into +1/+1 counters and mill: in a deck that wins only by combat damage, it never
  lowers a life total (modeled correctly).

## Deck-list suggestions (not applied; the list is the user's call)

Paired, adaptive, 600 games per tier (seeds 500000-500599), with the AI changes in:

| Swap | change | note |
| --- | --- | --- |
| Invoke Despair → Chromatic Lantern | **+1.3 ± 0.8** | BBBB is rarely castable; Lantern fixes every colour |
| Sisay's Ring → Arcane Signet | +0.8 ± 0.7 | coloured mana a turn sooner instead of two colourless |
| Szadek → Lightning Greaves | +0.1 ± 0.9 | shroud and haste; flat in this simulator |
| Kaervek's Purge → Swords to Plowshares | −0.2 ± 0.7 | the Purge is dead, but one more removal spell doesn't move the result |
| Desertion → Heroic Intervention | −0.5 ± 0.8 | |

Only the Lantern is clearly above noise.
The bigger lever is mana that makes all five colours early (Chromatic Lantern, Arcane Signet, Birds of Paradise,
City of Brass / Mana Confluence; the last two are not in the card database yet) and protection that covers Jodah on
the opponents' turns; the removal-proof diagnostic shows how much room there is (+15).

## Look-ahead check

The overnight run's Jodah games (old code, seeds 500000-500174, look-ahead) are reproducible game for game (checked on
two seeds), so they serve as the baseline; the new code played the same seeds (`ab.py merge` builds the file).

| | T1 | T2 | T3 | T4 | T5 | all |
| --- | --- | --- | --- | --- | --- | --- |
| before | 9.7% | 9.8% | 9.2% | 15.0% | 18.4% | |
| after | 13.2% | 13.8% | 9.2% | 10.3% | 11.5% | |
| paired change | +4.6 ± 6.8 | +4.0 ± 5.7 | +0.0 ± 5.6 | −4.6 ± 6.5 | −6.9 ± 6.6 | −0.6 ± 2.8 |

T4 and T5 looked worse, so 150 more seeds per tier (500200-500349) were played with both the old code (an
`origin/main` copy) and the new: T4 +0.7 ± 7.0, T5 +0.7 ± 6.1. Pooled over everything, 1,170 paired look-ahead games:
**T1-T3 +2.9 ± 3.5, T4-T5 −2.8 ± 3.3, all −0.3 ± 2.4.** No tier moved by more than its noise; the look-ahead search
overrides much of what the style and priorities change, and a real effect of 1-2 points needs several thousand
look-ahead games to see.

## Code changes (branch `jodah-ai-audit`)

- `cards/impl/mine.py`: Plaza of Heroes and Unclaimed Territory read `E.PAY_FOR` (bug fix).
- `cards/impl/jodah.py`: `jodah_prio` casts Jodah first when it is payable; `jodah_tutor` wish list. Both can be
  switched off for A/B with `JODAH_AI` (default `first,tutor`; `JODAH_AI=none` restores the old choices).
- `ais.py`, `cards/dsl.py`: Jodah's tutor hook; the `ae` (artifact or enchantment) search kind for Enlightened Tutor.
- `ai/brain.py`: Jodah's style aggression 0.90, caution 0.30 (`ab.py --diag aggr=0.6,caution=0.65` plays the old one).
- `tests/fixtures/sim_guard.json`: the five Jodah entries re-recorded; no other deck's entry changed.

## Scripts

```
PYTHONPATH=. python3 audit/jodah/jodah_stats.py run t1,t2,t3,t4,t5 400 stats.json --jobs 15   # traced games
PYTHONPATH=. python3 audit/jodah/jodah_stats.py report stats.json                            # every table above
PYTHONPATH=. python3 audit/jodah/jodah_stats.py log t3 700013                                # one game's log
JODAH_AI=none PYTHONPATH=. python3 audit/jodah/ab.py run t1,t2,t3,t4,t5 1000 a.json --jobs 15  # paired A/B arms
PYTHONPATH=. python3 audit/jodah/ab.py run t1,t2,t3,t4,t5 1000 b.json --diag safejodah        # diagnostics
PYTHONPATH=. python3 audit/jodah/ab.py run t1,t2,t3,t4,t5 1000 c.json --swap "Szadek, Lord of Secrets=>Lightning Greaves"
PYTHONPATH=. python3 audit/jodah/ab.py compare a.json b.json
```
`ab.py --ai lookahead` runs the look-ahead AI (minutes per game; use small counts).

## Tried and reverted (2026-10-08): Kaervek's Purge → Champion's Helm

The swap, with the AI casting the Helm and equipping it to Jodah first, measured about +0.2 points, within noise
(adaptive, paired, 600 games per tier: T1 19.0 vs 18.8, T2 17.0 vs 15.8, T3 16.3 vs 16.8, T4 21.5 vs 21.3, T5 23.5
vs 23.7, Helm vs Purge). Over 450 games against Tiers 1-3, the Helm was on Jodah in only 11% of them; wearing it,
Jodah was removed 8 times, and without it 323 times. JD chose to keep Kaervek's Purge, so the swap and its AI code
were reverted.

