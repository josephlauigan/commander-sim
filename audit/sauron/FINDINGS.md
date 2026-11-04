# Sauron, the Dark Lord — findings (living file)

What the simulator has taught us about the Sauron Grixis amass deck: strengths, weaknesses, and what was tried and why
it did or didn't work. Updated after every run. Newest evidence first. Matched tiers: T3–T4 (`data/my_decks.toml`).
Older tests: the dated folders here.

## 2026-11-03: Godo package added (JD's choice; not simulated)

Godo, Helm of the Host, Hammer of Nazahn, Combat Celebrant and Peter Parker's Camera moved from The Ur-Dragon into
Sauron as a secondary finish. Phyrexian Arena, Sauron, the Necromancer, Sephiroth, Triple Triad and Urza left. This
makes the paper list Bracket 4 through the Godo/Helm and Celebrant/Helm infinite-combat lines. The old current-state
numbers below predate this rebuild and are not its baseline. The next measurement must report T3 and T4 separately,
using the loose profile and default look-ahead AI. (Done 2026-11-04: see the last section.)

## Previous state (2026-10-17, night)
Look-ahead, loose, current list (baselines of the Urza run, with the hold-mana search credit): **T3 28.2%, T4 31.6%,
T5 26.4% (28.7% over t3–t5).**

## Weaknesses (by evidence)
- **Combo tables at T4–T5:** the killers there are Heliod (combo 16% at T4), Kinnan (20% at T5), the Urza deck (16%)
  and Zur. At T3 it's creature decks in combat (Aurelia 17%, Marwyn 13%, Tergrid 12%).
- **Lowest win rate when cast** (adaptive card report, `audit/sauron/2026-10-17-adnaus/K0-analyze-adaptive.txt`):
  Disdainful Stroke, Arcane Denial, Spell Pierce (t3, t4), Inexorable Tide, Relic of Legends.

## What was tried, and why it did or didn't work
### 2026-11-01: Triple Triad in for Ral and the Implicit Maze (JD's choice; no run)
(Triple Triad left the list in the 11/03 Godo rebuild, so the sim gap below is moot for now.) Picked by fit, not simulated. Of JD's red decks Sauron has the heaviest curve (3.28 average mana value, 16 cards at 5+),
which is what Triple Triad's "lesser mana value" clause rewards. Ral was the card most often stuck in hand (cast in
20-30% of the games it was drawn, t3-t5, 10/17 card report), and swapping a 5-drop for a 6-drop keeps the curve about
the same. Risks: the top end is already stuck in hand (Titans, Wurm, Gearhulk cast in 27-45% of their draws), Triad's
{R}{R}{R} adds to that, and a land on top (a third of the deck) gives nothing. **Sim gap:** Triad is modeled Full
(`impls/mine.rs`, `triad_upkeep`), but the generic priority Sauron's AI uses gives it 0 (no tags, no prio hook), so
it is never cast; add a priority before any swap test of it.
### 2026-10-18: list matched to the physical deck (Urza in, Spell Pierce out; no new run)
JD's physical list already had Urza in for Spell Pierce, so the repo list now matches it (branch `printings`). The
evidence is the test below: +0.5, noise, a fair swap. The "Current state" numbers are from the old list; the next
run of the deck measures the new one.

### Urza, Lord High Artificer (2026-10-17 night; branch `adnaus`; Full, from the pool's Urza deck)
| Swap (Urza in for) | Result over t3–t5 | Why |
|---|---|---|
| Spell Pierce / Disdainful Stroke (adaptive) | -0.3 / -0.3 | No effect. |
| Spell Pierce (look-ahead, before the hold credit) | -1.1 (t3 -0.6, t4 -0.3, t5 -2.5) | Noise. |
| **Spell Pierce (look-ahead, with the hold credit, 1,000 paired per tier)** | **+0.5 (t3 +2.1, t4 0.0, t5 -0.6; ±3.3–3.7)** | Noise. Urza cast in 65% of the games he's drawn (52% at T5), wins 36–55% when cast. T3 eliminations -2.6 (an extra body and the Construct against creature decks); at T4–T5 losing a cheap counter to combo decks costs a little (T5 damage -2.0). Mana on turn 8 unchanged: Sauron's artifacts are mostly rocks that already make mana, so Urza's {U} ability adds almost nothing. |
**Reading it:** a fair swap, not an upgrade. Urza wants cheap non-mana artifacts and artifact tokens; Sauron has few.

## 2026-10-18: band calibration (proposed swaps, NOT applied; see audit/bands/REPORT.md)
Band 2 (T3-T4). Proposed: +Force of Will (spare) for Slaughter Pact; 3 Game Changers. Final: **30.2% vs T3, 31.6% vs T4** (avg 30.9%; current 28.9 / 31.6 at 1,000). Why: a free counter protects Sauron and stops T3-T4 combo turns; Slaughter Pact had the lowest win rate when cast.

## 2026-10-18: AI audit phase 2 (branch audit-grixis; adaptive AI vs t3, 600 games x 2 profiles, before -> after)
Win rate 26.7% -> 27.0% (noise). Aggravated Assault is now cast in 17.5-18.5% of games at a 43% win rate (it was the tutors'
default target and rarely cast with a Sword that is no longer in the list); Army power at T6/T8/T10 mean 1.6/2.3/2.4 -> 1.7/2.7/2.2.
Why the Army is mostly absent (traced 37 games): it is killed, not unmade. About 30 Army deaths in 37 games: opponents' wipes
(Toxic Deluge, Deed, austere, dmg13) and spot removal; plus Sauron's own Nibelheim Aflame (the sweep source is the biggest creature,
so a smaller Army died). Fixed the self-sweep (the Army counts 3x in wipe_eval unless The Ozolith is out) and the tutors now fetch The
Ozolith first once an Army is growing. Weakness that remains: one big Army against a removal-heavy table; the Ozolith is the answer.
Other fixes: tutors by board state (Sol Ring when short, Ozolith, Archivist with Bowmasters, Cloak/Passage for a huge Army, then
Assault, closers, Not of This World, Barad-dur when flooded); Force of Will pitches draw spells before engine creatures; Sauron holds
mana for counters like the outside decks; Cost of Brilliance gives an opponent cards while Bowmasters is out; Archivist wheel also
for Kaervek, or Sauron with a small hand.

## 2026-10-18: AI audit re-measure (branch ai-audit; 2,000 games per profile, look-ahead)
**30.4% vs T3, 32.0% vs T4** (before 30.2 / 31.6): no change. Board-state tutors; its own sweepers spare the Army; Force of Will pitches draw spells first; mana held for counters.

## 2026-10-18 evening: JD's new cards, coarse check (loose only, 500 games per list per tier, look-ahead; only big downgrades show, noise ±4-5 per tier)
**Relic of Legends + Bitter Triumph => Basalt Monolith + Rings of Brighthearth: T3 +1.6, T4 +2.2 (noise ±5, no
downgrade).** Basalt is cast in 82-84% of draws (a 3-mana rock that taps for 3 once), Rings in 52-57% (low priority
until the combo is near). With Urza out the pair is infinite colorless mana into Urza's free plays (the loop is
abstracted). Wins stayed by damage, so the combo itself rarely assembles in 500 games: the gain, if real, is mostly
Basalt's burst mana. Rings of Brighthearth is not owned yet. Raw: `2026-10-18-new-cards/`. JD plans to add the cards unless one is a big downgrade.
**PINNED (JD, 10/18):** JD owns Basalt Monolith but not Rings of Brighthearth yet; the package waits until he has Rings.

## 2026-10-19: Urza out to Tymna/Thrasios? (paired, loose, look-ahead, 1,000 games per tier; `2026-10-19-urza-out/`)
**Urza, Lord High Artificer => Spell Pierce: T3 +0.5 (±3.7), T4 +1.2 (±3.5); noise.** Urza cast in 65-69% of his
draws, Spell Pierce in 68-70%. The move tested -1.5 in Tymna/Thrasios (`audit/thrasios/FINDINGS.md`), so on both sides
it's a wash: not adopted, Urza stays.

## 2026-10-20: Urza out, Spell Pierce back (paired, 2,000 games per tier, seed 1020; `2026-10-20-urza-out/`)
| Swap | t3 | t4 | Why |
|---|---|---|---|
| Urza, Lord High Artificer => Spell Pierce | +0.3 (31.5% -> 31.8%, ±2.6) | -1.2 (31.9% -> 30.7%, ±2.5) | Noise both tiers (10/19 gave +0.5/+1.2). Urza is cast in 68% of his draws (round 8.2); Spell Pierce in 63%. The plan comes online slightly less without Urza (-2.8 / -1.3 points, inside noise). |
**Reading it:** Urza is neither needed nor a drag in Sauron; with Tymna/Thrasios not wanting him either, he stays.

## 2026-10-31: card models (branch card-models-1; no sims until Phase 4 recalibration)

Baleful Mastery: Partial -> Full (the {1}{B} alternative cost, an opponent draws; the AI uses it only when it can't pay {3}{B}). No new number yet.

## 2026-11-01: recalibration stage 2 remeasure

Sauron, t3 / t4: **21.4% / 20.3%** (average 20.9%) (2000 games, seeds 500000-501999, loose, look-ahead, defaults, rc=0; `audit/recal/stage2-2026-11-01.md` section 4, build efe45425). Stage 1, same seeds: 24.9 / 21.2. The 10/18 AI audit numbers (30.4 / 32.0, with the hand AI) are from an older build and list. Fitted t3 knobs: 20.9% (level).

## 2026-11-04: first baseline of the Godo, Helm and Celebrant list (t3, t4)

List: the 11/03 rebuild (Godo, Helm of the Host, Hammer of Nazahn, Combat Celebrant and Peter Parker's Camera in;
Phyrexian Arena, Sauron the Necromancer, Sephiroth, Triple Triad and Urza out) with the plan file rewritten for it
(`data/plans/sauron.toml`, 11/04). Run: 2,000 games per tier, seed 500000, loose, look-ahead AI, defaults, each tier
reported separately (`.claude/worktrees/sim-runs-2026-11-04/sauron-baseline.txt`).

| Tier | Win rate | 95% CI | vs even share (25%) |
|---|---|---|---|
| t3 (High B3) | **29.1%** | 27.2-31.1 | +4.1 |
| t4 (Low B4) | **25.9%** | 24.0-27.8 | +0.9 (inside the CI) |
| **Average over t3-t4** | **27.5%** | about +-1.4 (my pooling of the two cell CIs) | +2.5 |

- **t3 is the better tier by 3.2 points** (29.1 vs 25.9). The two cells' CIs overlap slightly (27.2-31.1 vs 24.0-27.8),
  so treat the gap as likely but not settled (different pods, 2,000 games each).
- **Not in this output:** win and loss turns, how it wins (damage vs combo vs the Godo/Helm lines) and who eliminates it.
  A multi-tier `--pool all` run prints only the win-rate table. Those need a single-tier run (`--pool t3`, `--pool t4`)
  or `--analyze`; none was run. So whether the Godo, Helm and Celebrant package actually fires in games is not yet
  measured here.
- **Old vs new not run, on purpose.** The plan file was rewritten for the Godo, Helm and Celebrant package, so the old
  list under the new plan (or the new list under the old plan) would not be a fair comparison. The older numbers in
  this file (30.4 / 32.0 on 10/18, 21.4 / 20.3 on 11/01) are from earlier lists and builds.
- **Model coverage:** Full 99, Approximate 1, Partial 0, Unmodeled 0 (the Approximate card is not named in the output).
  Opponents t3: Full 439, Full-auto 42, Approximate 19; t4: Full 435, Full-auto 37, Approximate 28.
- **Combo cards now visible to opponents:** the Ur-Dragon combo cards became catalogue piece names for every deck
  (`rust/agents/report-urdragon-combos.md`), which can shift opponents' removal and counter choices slightly. It is part of
  this baseline.
- **One seed per tier.** The CIs above are the only noise estimate.

## 2026-11-04 (late): AI changes, not simulated (JD: no sims for this batch)

- **Mystic Confluence** now costs {3}{U}{U} (data/carddb.txt had {3}{U}{U}{U}) and **Mauhúr, Uruk-hai Captain** {B}{R} (had {1}{B}{R}): both were a mana dearer for the AI than printed.
- **Iron Man, Armored Avenger**: its draw counter goes on the AI's best creature (it went only on the Army, and nowhere without one), as the Oracle text's "target creature" says.
- Unchanged on purpose: Bloodchief's Thirst is stored at its kicked cost {2}{B}{B}, which the AI always pays (a documented approximation; a person pays {B}).

The numbers above predate these changes; a re-run on the deck's own tiers would show their effect.
