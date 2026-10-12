# Sheoldred: the Jodah trade blocks in the Sheoldred deck (2026-10-11)

Paired swap tests on the bulk-only list (`decklists/JD/sheoldred-mono-black.md`, code dfeb5ee): look-ahead AI, loose
profile, 1,000 games per list per tier, the same seeds, seats and opponents' draws for both lists (`run.sh sheo-A
sheo-B sheo-C`; raw output in `sheo-A/`, `sheo-B/`, `sheo-C/`; per-tier lines in `log.txt`). Over all five tiers the
noise is about ±1.4 points for A and B, ±0.8 for C.

| Block (Jodah -> Sheoldred) | Sheoldred cuts | T1 | T2 | T3 | T4 | T5 | All tiers |
|---|---|---|---|---|---|---|---|
| Current list | | 16.8% | 21.7% | 20.5% | 23.5% | 22.0% | **20.9%** |
| A: Memory Jar | Carnage Altar | +2.5 | **+6.8** | **+5.2** | +0.6 | -0.6 | **+2.9** (23.8%) |
| B: Toxic Deluge, Bitter Triumph | Depressurize, Multiply by Zero | +1.0 | **+4.0** | +0.2 | -2.2 | +0.5 | +0.7 (21.6%) |
| C: Korlash, Heir to Blackblade | Gravblade Heavy | +1.1 | **+2.3** | +0.1 | -0.4 | -1.1 | +0.4 (21.3%) |

Bold: outside that tier's noise. Only block A is a clear gain: Memory Jar cracked with Sheoldred out makes each
opponent draw seven (14 life each). B and C are within noise over all tiers.

Not measured yet (JD, 10/11: later): what each block costs Jodah with its bulk refills (`run.sh jodah-A jodah-B
jodah-C`), and all three blocks at once. The current list's 20.9% comes from these paired runs, not the section 0d
seeds (500000-500959), so it is close to but not directly comparable with the pool-results table (Titania 22.8%).

## The upgrade package (2026-10-11, buys, not yet decided)

Paired, same settings (`run.sh upgrade`, 22 worker threads; raw output in `upgrade/`), on the list with Memory Jar
(code 913c473). Seizan, Perverter of Truth, Underworld Dreams, Dark Deal and Painful Lesson (about $6.40) for Ribskiff,
Gravblade Heavy, Multiply by Zero and Locust Spray.

| | T1 | T2 | T3 | T4 | T5 | All tiers |
|---|---|---|---|---|---|---|
| Current list (with Memory Jar) | 18.1% | 27.1% | 24.9% | 27.5% | 23.9% | **24.3%** |
| With the package | 22.0% | 31.2% | 27.0% | 29.5% | 23.4% | **26.6%** |
| Change | **+3.9** | **+4.1** | +2.1 | +2.0 | -0.5 | **+2.3** |

Over all tiers the noise is about ±1.6 points, so +2.3 is a real gain; four of five tiers move up. The current list's
24.3% here matches the 23.8% estimated from the block A run (within noise).

## The second upgrade package (2026-10-11, buys, about $44 for all 20 cards, not yet decided)

Both packages at once (`run.sh upgrade2`; raw output in `upgrade2/`, code 3e3e24d): the first package plus Howling
Mine, Temple Bell, Fate Unraveler, Peer into the Abyss, Geier Reach Sanitarium, Castle Locthwain, Mind Stone,
Commander's Sphere, Charcoal Diamond, Phyrexian Arena, Read the Bones, Ambition's Cost, Syphon Mind, Exsanguinate, Gray
Merchant of Asphodel and Swiftfoot Boots, for Mabel, Moseo, Burglar Rat, Fathom Fleet Cutthroat, Canyon Crawler,
Necrosquito, Rampart Hunter, Callous Inspector, Depressurize, End of the Hunt, Tome of Legends, Diary of Dreams, Dark
Endurance, Lash of the Balrog and two Swamps.

| | T1 | T2 | T3 | T4 | T5 | All tiers |
|---|---|---|---|---|---|---|
| Current list (with Memory Jar) | 18.1% | 27.1% | 24.9% | 27.5% | 23.9% | **24.3%** |
| With both packages | 26.4% | 42.7% | 33.4% | 32.2% | 25.3% | **32.0%** |
| Change | **+8.3** | **+15.6** | **+8.5** | **+4.7** | +1.4 | **+7.7** |

Six traced games (Tier 2, adaptive AI) showed nothing suspicious: wins came from the drain plan (Underworld Dreams,
Howling Mine and Sheoldred killed all three opponents in three turns) and from Exsanguinate (X = 12 on turn 11, with 14
mana that checks out: a land from Path to Exile on Sheoldred, and Sheoldred turned into a Forest by Song of the Dryads).

## Gix, Yawgmoth Praetor on the upgraded list

Gix (from Jodah) for Hullcarver, measured on the list with both packages applied (`run.sh gix-u2`; the doc is rewritten
for the run and restored; raw output in `gix-u2/`):

| | T1 | T2 | T3 | T4 | T5 | All tiers |
|---|---|---|---|---|---|---|
| Upgraded list | 25.7% | 42.3% | 30.7% | 30.1% | 24.8% | **30.7%** |
| With Gix | 27.8% | 42.7% | 32.5% | 33.1% | 23.6% | **31.9%** |
| Change | +2.1 | +0.4 | +1.8 | +3.0 | -1.2 | **+1.2** |

Within noise (about ±1.5 over all tiers), leaning positive in four tiers. The upgraded list's 30.7% here and 32.0% in
the run above differ by about the noise.
