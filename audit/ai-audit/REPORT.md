# Deck AI audit, 2026-10-18

JD asked for every card in his ten decks to be implemented correctly and valued by that deck's AI for that deck's
strategy, not treated as a generic value card. This branch (`ai-audit`) fixes what the audit found. No decklist
changes.

## 1. What changed in the win rates

Each deck against its own band tiers. Look-ahead AI; the win rate is the mean of the conservative and loose opponent
profiles. "Before" is the 10/18 final measurement on main's code: 4,000 games per profile for bands 1-3 and 2,000 for
band 4; Tymna/Kraum's before is its 4,000-game paired test of the final list. "After" is this branch at 2,000 games per
profile, so its noise is about ±1.4 points per tier.

| Band | Deck | Tier | Before | After | Change |
|---|---|---|---|---|---|
| 1 | Y'shtola | T5 | 24.3 | 24.5 | +0.2 |
| 1 | Seph FS | T5 | 23.7 | 23.6 | -0.1 |
| 1 | Tymna/Kraum | T5 | 21.4 | 20.8 | -0.6 |
| 2 | Seph/Atraxa | T3 / T4 | 35.1 / 28.3 | 36.3 / 27.6 | +1.2 / -0.7 |
| 2 | Sauron | T3 / T4 | 30.2 / 31.6 | 30.4 / 32.0 | +0.2 / +0.4 |
| 2 | Sheoldred | T3 / T4 | 30.6 / 30.8 | 31.9 / 31.6 | +1.3 / +0.8 |
| 3 | Kefka | T3 | 26.7 | 27.3 | +0.6 |
| 4 | Cloud | T1 / T2 / T3 | 29.6 / 26.3 / 20.4 | 28.9 / 23.8 / 21.4 | -0.7 / -2.5 / +1.0 |
| 4 | Veyran | T1 / T2 / T3 | 25.8 / 28.6 / 24.5 | 29.5 / 32.6 / 27.7 | **+3.7 / +4.0 / +3.2** |
| 4 | Titania | T1 / T2 / T3 | 19.1 / 27.6 / 25.5 | TBD (runs tonight) | TBD |

**Reading it:**
- **Veyran is the one real mover: +3 to +4 points at every tier**, beyond noise. Triple Triad is finally cast, Greaves comes down before Veyran, his tutors and Recruiter pick by state, and his engine creatures are valued. At T3 he is now level with Kefka (27.7 vs 27.3): **the band 3 / band 4 gap is gone** (JD to decide on rebalancing).
- **Every other deck is within noise**, except Cloud at T2 (-2.5), which is on the edge (see section 4). No other deck changed band.
- **The fixes change how the decks play much more than how often they win.** Tutors now fetch what the deck needs,
  protection is held for the threat, and dead cards are cast. In the agents' own paired checks (fast AI, up to 60,000
  games), the per-card numbers moved as intended: Momentary Blink cast in 17.6% of games, Breach 6.7% (was 0%),
  Triple Triad 15% (was 0%), Greaves before Veyran 51%. Apart from Veyran, win rates moved by under a point. The decks were already
  being carried by their core plans; these were the edges.
- Apart from Veyran, the band picture from the 10/18 calibration holds. Tymna/Kraum is the softest band-1 deck (20.8% vs T5).

## 2. What the audit found and fixed

Phase 1 surveyed every card (rules model + whether the deck's AI values it for the deck's plan): 77 problems
(`audit/ai-audit/<deck>.md`). Phase 2 fixed them on six branches with tests (`rust/agents/report-audit-*.md`).

Problems that hit several decks:
- **Held cards were cast anyway.** When a deck's own AI held a card (priority 0), a shared fallback replaced the 0
  with a generic value: Masterful Flourish and Battle-Rage Blessing (Sheoldred) and Blossoming Defense (Titania) were
  cast on turn 1 with no creature. Now, for JD's ten decks, a held instant, removal, wipe, counter or protection card
  stays held (removal and wipes are still cast by the removal and wipe logic; checked in traces).
- **Tutors fetched the wrong cards.** Seph FS had no wish list (it fetched Sol Ring and Citadel, never Exsanguinate or
  Gray Merchant); Sauron's list was left over from an old build (always Aggravated Assault); Kefka only ever hunted
  Breach; Veyran's Imperial Recruiter fetched a combo piece from a combo the deck no longer has; Y'shtola's Mystical
  Tutor could not find a finisher. Each now picks by board state.
- **Teferi's Protection (Y'shtola)** ran Clever Concealment's code (one creature, no life lock). It now uses the full
  engine card, cast for a wipe or a lethal threat.

Per deck (details in each deck's `FINDINGS.md`):
- **Seph/Atraxa:** Hornet Queen and Muldrotha valued as bombs (reanimation, blink, Entomb); one bomb threshold;
  Momentary Blink as protection; Grand Abolisher timing; Tracker before the land drop.
- **Seph FS:** board-state tutor pick; devotion-aware Gray Merchant; the Altar + Skeleton + Pawn loop recognised and
  run; Citadel gated on Sephiroth or a free outlet; a plan milestone (first flip).
- **Y'shtola:** spells timed around her trigger; no removal on weak targets; wipes cast with Teferi's Protection.
- **Tymna/Kraum:** Breach is cast (escapes wipes when behind); Dark Endurance protects; Diabolic Tutor fetches a
  finisher; Baird valued as defence; Firebird's attack pump and Lightning, Security Sergeant's impulse draw coded.
- **Sauron:** board-state tutors (Ozolith first once the Army grows); its own sweeper avoids killing the Army; Force
  of Will pitches draw spells first; mana held for counters; Cost of Brilliance feeds Bowmasters.
- **Sheoldred:** Ad Nauseam in its second main phase; Memory Jar waits for a punisher; Seizan valued with Dreams or
  Unraveler; Necrosquito valued; Rooftop Assassin's "dealt damage this turn" condition.
- **Kefka:** tutors fetch Crawler / Niv-Mizzet for a flipped Kefka; his discard keeps Breach, Yawgmoth's Will and
  protection; Mana Leak discounted into open mana; Profane Tutor suspended on an idle turn.
- **Cloud:** turn-2 search no longer defaults to Sword of Feast and Famine; Open the Armory fetches only Equipment or
  Auras; Austere Command doesn't destroy Cloud's own enchantments.
- **Veyran:** Triple Triad cast; Greaves before Veyran; Recruiter, Mystical and Solve the Equation pick by state;
  engine creatures valued; dead Kitten-combo code removed.
- **Titania:** Fblthp's {X} cost; Silhana's coloured mana; a Worldly Tutor pick; Arachnogenesis timing.

Not bugs (left as is, with the evidence in the deck files): Tymna attacking one opponent a turn (too few cheap bodies:
a deckbuilding question), Tymna's stuck top end (mana), Kefka's Breach dry runs (real storm shortfalls), Seph FS's
Massacre Girl formula, Rhystic Study in Kefka, some reactive cards held by design. Left open: Mystical Tutor in
Y'shtola is not held until a finisher can follow.

Also fixed: `cardgen audit --md` can no longer overwrite a decklist file, and `--cards` no longer prints stale
"not modeled" lines for cards that have code.

## 3. Effect on the pool decks (for the Tier 6 work)

`--calibrate within --games 40 --ai adaptive` is byte-identical to main for T1-T5. T0 differs through one correct
model fix: Firebird, Blazing Ranger's attack trigger (the Captain America precon plays it). Teferi's Protection did
not affect pool decks (they already used the full card).

## 4. Open items

- **Band 3 vs band 4:** Veyran (band 4) now matches Kefka (band 3) at T3. Options: trim Veyran slightly, lift Kefka, or accept. JD's call; not done here.

- **Cloud at T2 (-2.5):** on the edge of noise; the agent's own small check also leaned slightly negative. Suspects:
  the new turn-2 search order (Feast and Famine no longer first unless the hand is 7+) and the Austere Command mode
  rule. A paired run of this branch against main (Cloud vs T2, 4,000 games per profile) would settle it.
- **Titania:** measured tonight; the table above gets its number in a follow-up.
