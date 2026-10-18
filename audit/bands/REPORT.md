# Deck bands: overnight calibration, 2026-10-18

Nothing in this report is applied to your physical decks or committed to the decklists. Every swap is a proposal
waiting for your approval. The proposals are on branch `soul-stone` (not pushed), in `audit/bands/final/lists.txt`.

## 1. The short version

- **The four bands now separate, with one soft spot.** Bands 1-2 and 2-3 have clear gaps. Band 3 sits only 1-2 points
  above Titania and Veyran at T3, which is inside the noise between them (section 2).
- **All 29 Game Changers stay in decks.** None go to the binder. Bands 1-3 each have at least one, and only Y'shtola
  (9) and Seph FS (4) have more than three.
- **14 cards move between decks, and 17 come from your spares** (bulk and binder). Nothing has to be bought.
- **Two AI problems found.** The sim never cast Worldly Tutor in Seph/Atraxa; that is fixed and committed. The
  Tymna/Kraum AI never casts Underworld Breach (not fixed; see decision 2).
- **Measurement.** The final lists were measured at 4,000 games per profile for bands 1-3 (about ±1 point per tier)
  and 2,000 for band 4 (±1.4 per tier, ±0.8 on the three-tier average). In total about 220,000 games between 01:22
  and 07:11.

## 2. Result: final lists, each deck against its own band tiers

Win rate is the mean of the two opponent AI profiles. 25% is an even share at a four-player table; 22-32% is a fair
fight. "Before" is the current list from the paired tests (1,000 games per profile, about ±2).

| Band | Deck | T1 | T2 | T3 | T4 | T5 | Before | GCs |
|---|---|---|---|---|---|---|---|---|
| 1 | Y'shtola | | | | | **24.3** | 23.2 | 7 → 9 |
| 1 | Seph FS | | | | | **23.7** | 21.5 | 1 → 4 |
| 2 | Seph/Atraxa | | | **35.1** | **28.3** | | 36.2 / 29.8 | 8 → 3 |
| 2 | Sauron | | | **30.2** | **31.6** | | 28.9 / 31.6 | 2 → 3 |
| 2 | Sheoldred | | | **30.6** | **30.8** | | 32.5 / 33.1 | 1 → 1 |
| 3 | Tymna/Kraum | | | **27.2** | | | 35.9 | 0 → 1 |
| 3 | Kefka | | | **26.7** | | | 27.4 | 6 → 3 |
| 4 | Veyran | **25.8** | **28.6** | **24.5** | | | 23.4 / 26.0 / 23.5 | 0 → 2 |
| 4 | Cloud (unchanged) | **29.6** | **26.3** | **20.4** | | | same list | 2 → 2 |
| 4 | Titania | **19.1** | **27.6** | **25.5** | | | 20.0 / 32.2 / 28.3 | 0 → 1 |

**Reading it:**
- **Bands 1-3 have one common table, T3, plus T5 for band 1.** Band 1 holds a fair share against T5, the strongest
  tier, and nothing below it is asked to.
- **The T3 ladder:**

  | Band | Win rate vs T3 |
  |---|---|
  | 2 | 30.2-35.1% |
  | 3 | 26.7-27.2% |
  | 4 | 20.4-25.5% |

  Band 2 to band 3 is a clear step of 3-8 points. Band 3 to band 4 is clear for Cloud (6 points) but small for Veyran
  (2.2) and Titania (1.2): see decision 5.
- **Band 4's tier pattern sets it apart.** All three are fair against T1-T2 except Titania against T1 (19.1%). Her
  weakness against Tier 1's fast aggro was there before the swaps (20.0%).
- **Seph/Atraxa is lopsided.** It is the strongest band-2 deck at T3 and the softest at T4. Its average (31.7%) is in
  line with Sauron (30.9%) and Sheoldred (30.7%).

## 3. Proposed swaps

Each line is "out → in". Cards moved between decks are marked with their source or destination; everything else comes
from or goes to your spares.

**Y'shtola (band 1)**
- Dakkon, Shadow Slayer → Force of Will (from Seph/Atraxa)
- Thought Vessel → Farewell (from Seph/Atraxa)
- Bolas's Citadel (to Seph FS) → Teferi's Protection (from Seph/Atraxa)

Why: Force of Will is cast in 76% of the games it is drawn, and answers Tier 5 combo. The sim never cast Dakkon, and
Y'shtola cast Citadel in only 25% of the games it drew it.

**Seph FS (band 1)**
- Star Compass → Demonic Tutor (from Kefka)
- Tome of Legends → Vampiric Tutor (from Kefka)
- Masterful Flourish → Rune-Scarred Demon (from Tymna/Kraum)
- Bitter Triumph → Bolas's Citadel (from Y'shtola)
- Rabid Attack → Gray Merchant of Asphodel (from Sheoldred)

Why: the three cards it gives up had the lowest win rates when cast at T5. The tutors find the wipes and Living Death
sooner. Citadel (62% win when cast) and Gray Merchant (46%) turn drain life into cards and reach.

**Seph/Atraxa (band 2)**
- Force of Will, Farewell, Teferi's Protection (to Y'shtola) → Hornet Queen, Harmonize, Tireless Tracker (from Titania)
- Consecrated Sphinx (to Veyran) → Austere Command (spare)
- Worldly Tutor (to Titania) → Arcane Denial (spare)

It keeps Demonic Tutor, Gifts Ungiven and Aura Shards. Why: with 8 GCs it is a Bracket 4 deck. Without five of them it
measures the same, because the new cards cover what they did: Hornet Queen is a strong reanimation target and
Harmonize/Tracker replace the Sphinx's cards.

**Sauron (band 2)**
- Slaughter Pact → Force of Will (spare)

**Sheoldred (band 2)**
- Gray Merchant of Asphodel (to Seph FS) → Necrosquito (spare)

Why: Sheoldred was at the top of band 2. This costs it about 2 points and gives Seph FS a card that fits its 32
Swamps.

**Kefka (band 3)**
- Demonic Tutor, Vampiric Tutor (to Seph FS) → Mana Leak, Expressive Iteration (spares)
- Mystical Tutor (to Veyran) → Sleight of Hand (spare)

It keeps Underworld Breach, Jeska's Will and Rhystic Study. Why: the combo is intact but found less often.

**Tymna/Kraum (band 3)**
- Rune-Scarred Demon (to Seph FS) → Underworld Breach (spare)
- Sunblast Angel → Shatterwing Pegasus
- Lyra Dawnbringer → Flensing Raptor
- Terror of the Peaks → Moonwing Moth
- Ghostly Prison → Thermal Glider
- Propaganda → Argivian Cavalier
- Elspeth, Sun's Champion → Warlord's Elite

Why: it was the biggest outlier (35.9% at T3). Each cut took only 2-3 points, so it needed two finishers, two attack
taxes and three more top cards to drop under band 2.

**Veyran (band 4)**
- Evacuation → Consecrated Sphinx (from Seph/Atraxa)
- Venser, Shaper Savant → Mystical Tutor (from Kefka)

**Titania (band 4)**
- Hornet Queen, Harmonize, Tireless Tracker (to Seph/Atraxa) → Cactarantula, Penumbra Spider, Paradise Druid (spares)
- Environmental Scientist → Worldly Tutor (from Seph/Atraxa)

**Cloud (band 4):** no change.

**Cards going to spares (all non-GC):** Dakkon, Thought Vessel, Star Compass, Tome of Legends, Masterful Flourish,
Bitter Triumph, Rabid Attack, Slaughter Pact, Evacuation, Venser, Environmental Scientist, Sunblast Angel, Lyra,
Terror of the Peaks, Ghostly Prison, Propaganda, Elspeth.

**GC homes:**

| Deck | GCs |
|---|---|
| Y'shtola | 9 |
| Seph FS | 4 |
| Seph/Atraxa | 3 |
| Sauron | 3 |
| Kefka | 3 |
| Cloud | 2 |
| Veyran | 2 |
| Sheoldred | 1 |
| Tymna/Kraum | 1 |
| Titania | 1 |

That is all 29, counting the spare Force of Will and Underworld Breach.

## 4. Process

**How a deck is measured.** The simulator seats your deck with three opponents drawn from a tier. The opponents play
under two AI profiles, conservative and loose, and the win rate is the mean of the two.

**Round 1 (01:22-03:14).** One paired swap test per deck: the current list and the swapped list play the same
opponents and draws. 1,000 games per list per profile, on the deck's band tiers. The current lists of Sheoldred and
Cloud were also measured. Every incoming card was checked first, and all are fully modeled. Results are in
`audit/bands/r1/SUMMARY.md`.

What round 1 showed:
- Y'shtola reached its target; Seph FS gained only about 1 point.
- Seph/Atraxa lost five GCs and got no weaker.
- The sim never cast Worldly Tutor (AI bug, fixed: section 6).
- Tymna/Kraum's current list is at 35.9% vs T3, far above band 3. That is higher than the 31% in its findings file;
  Avenge came in on 10/17.
- Band 3 and band 4 overlapped at T3.

**Round 2 (03:14-04:08).** The decks that missed:
- More for Seph FS: Citadel and Gray Merchant.
- Better filler for Kefka (no effect).
- Two steps of cuts for Tymna/Kraum.
- Seph/Atraxa re-run on the fixed AI.
- Keeping Vampiric Tutor instead of Rhystic Study in Kefka: no difference.

Results are in `audit/bands/r2/SUMMARY.md`.

**Final (04:08-07:11).** Each final list was measured alone, in a throwaway copy of the repository; the decklists on
the branch were never edited. Raw output is in `audit/bands/final/`. Each deck's `audit/<deck>/FINDINGS.md` has an
entry for this calibration.

## 5. Decisions I made without you

Each of these is a point where I would have asked you. What I chose and why:

1. **Spare cards count as owned.** Your bulk and binder lists are cards you own, so I used them: Austere Command,
   Arcane Denial, Force of Will, Underworld Breach and the filler. If you meant "only move cards between decks", the
   spare GCs (Force of Will to Sauron, Underworld Breach to Tymna/Kraum) are the ones to revisit.
2. **Underworld Breach is Tymna/Kraum's one GC, though the sim never casts it there.** Bands 1-3 need at least one GC,
   and Breach was the only spare GC in its colours. The alternatives:
   - Veyran, where it is strong, but band 4 would rise into band 3.
   - Teferi's Protection, a real card for Tymna/Kraum, but the deck had to come down 9 points.
   - The binder, which goes against "keep GCs in play".

   In real games Breach is a weak card in Tymna/Kraum too, so the sim's number is probably close to right.
3. **Teferi's Protection stays in Y'shtola, not Tymna/Kraum**, for the same reason: Tymna/Kraum needed to get weaker.
4. **Seph FS at 23.7% vs T5 counts as band 1.** I had set myself a 24% target, but 23.7% is inside the fair band
   against the strongest tier. The next lever was Necropotence from Y'shtola, which would weaken the other band-1
   deck.
5. **Band 3 and band 4 are only 1-2 points apart at T3 (Titania 25.5, Veyran 24.5 vs Kefka 26.7), and I stopped
   there.** Trimming band 4 further would push Titania lower against T1, where she is already below the fair line
   (19.1%). Your options, none tested:
   - Accept it. The bands still differ in which tiers each deck can handle.
   - Lift band 3 slightly, for example by giving Elspeth back to Tymna/Kraum.
   - Trim Titania's late game (Overwhelming Stampede), accepting a weaker T1.

   My pick is the second, with a quick test.
6. **Tymna/Kraum's last two cuts (Propaganda, Elspeth) went into the final run without their own paired test**, to
   save time. The final measurement is the check that matters, and it landed in band 3.
7. **Band 4 was measured at 2,000 games per profile, not 4,000.** The final runs were slower than estimated; at 4,000
   they would have finished at about 08:35, not 07:11.
8. **Kefka keeps Rhystic Study and gives Vampiric Tutor to Seph FS.** Keeping Vampiric instead measured the same
   (26.1% vs 25.8%). This version needs one fewer move: Rhystic would have needed a home and a cut in Y'shtola.
9. **The AI fix went in during round 1.** The first four tests (Seph FS, Y'shtola, Seph/Atraxa, Sauron) ran on the old
   program. The fix only changes Worldly Tutor and Mystical Tutor, so only Seph/Atraxa was affected, and I re-ran it.
   Each paired test always compares both lists on the same program.
10. **Seph FS ends with four GCs** (The One Ring, Demonic Tutor, Vampiric Tutor, Bolas's Citadel). That is allowed for
    band 1.
11. **CPU.** 22 threads at nice 19 all night, as you allowed. One sim at a time, memory capped at 24 GB, and a
    watchdog that would kill any sim running over 45 minutes and every sim at 08:15. Nothing was killed.

## 6. Engine and AI changes (committed on branch `soul-stone`, not pushed)

- **`e0064d2` The Soul Stone modeled** (from last night's question): an indestructible rock for {B}, harnessed at
  instant speed once a creature worth returning is in the graveyard; it then returns the best creature card every
  upkeep. Measured in Seph/Atraxa in place of Mind Stone: +0.25 points, noise. It is not part of the band proposal.
- **`73984b2` Band tiers** in `data/my_decks.toml` and the CLAUDE.md tier list, as you set them, and **`80456a5`** the
  test that pins each deck's tiers.
- **`ebd8d6f` AI fix for card-to-top tutors.** When a deck's own card priority doesn't know Worldly Tutor or Mystical
  Tutor, the card's own tutor timing is used. Before the fix, Seph/Atraxa never cast Worldly Tutor; after it, it casts
  it in 83-87% of the games it draws it.
- **Tests.** All pass except a practice-mode HTTP test that fails about 1 run in 4 on unchanged code under full CPU
  load. It is a race between the card-image loader and the "try it" reset; I didn't touch it.
- **Not committed:** any decklist change.

## 7. What I would look at next

- The band 3 / band 4 gap at T3 (decision 5): one 10-minute test of Elspeth back in Tymna/Kraum.
- Tymna/Kraum's AI has no use for Underworld Breach. If you want Breach to be a real card there, that is an AI task.
- Veyran never casts Triple Triad (untagged): a model gap that understates Veyran slightly.
- Rune-Scarred Demon is stuck in Seph FS's hand 61% of the time. A cheaper threat might serve band 1 better.

## 8. Final decisions (JD, morning of 10/18) and what was committed

- **Tymna/Kraum moves to band 1 (T5).** Measured against T5 (with JD's one-off `--any-tier`), its current list was
  22.0%. A band-1 GC package (Force of Will, Rhystic Study, Mystical Tutor, Enlightened Tutor, Teferi's Protection)
  made no difference (21.4%): the cards get cast, but losses to combo don't move. Final list: the original list with
  only Rune-Scarred Demon → Underworld Breach (Rune-Scarred Demon goes to Seph FS): **21.4% vs T5**. Its tier is now
  `t5`. None of the seven band-3 cuts above apply.
- **Titania gets Fblthp, Knows the Way instead of Penumbra Spider** (JD's choice). This card wasn't measured. Its
  land-search trigger is modeled; its domain power is 1 in mono-green.
- **Band 3 is Kefka alone.**
- **Committed:** the decklists (`cardgen deck`, logged in `decklists/CHANGELOG.md`), Tymna/Kraum's tier, the deck
  docs, and `bulk-cards.txt` / `binder-cards.txt`. 11 spare cards were taken out of the binder (3) and bulk (8), and
  11 deck cuts went into bulk.
