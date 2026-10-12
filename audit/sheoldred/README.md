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

## Mikokoro, Center of the Sea for a Swamp (2026-10-12)

JD's buy ($4.54), in the list in place of a Swamp. Measured the other way round (`run.sh mikokoro`: Mikokoro back out
for a Swamp; raw output in `mikokoro/`), so the change below is the list with Mikokoro minus the one without:

| | T1 | T2 | T3 | T4 | T5 | All tiers |
|---|---|---|---|---|---|---|
| Without Mikokoro (a 32nd Swamp) | 30.0% | 40.5% | 31.2% | 31.8% | 22.5% | **31.2%** |
| With Mikokoro | 29.2% | 40.7% | 32.4% | 32.1% | 24.3% | **31.7%** |
| Change | -0.8 | +0.2 | +1.2 | +0.3 | +1.8 | **+0.5** |

Within noise, leaning positive (four tiers of five; T5 +1.8 against ±1.3). Eight traced games (Tier 3) showed it
activated in two, both at the end of an opponent's turn with Sheoldred out; in one the draw set up the kill next turn.

## Otherworld Atlas and Mikokoro for Temple Bell and Castle Locthwain (2026-10-12)

JD couldn't find Temple Bell or Castle Locthwain and locked in the replacements: Otherworld Atlas ($1.68) for the Bell,
Mikokoro ($4.54) for the Castle (back to 32 Swamps; the Mikokoro-for-a-Swamp list above is superseded). Measured the
other way round (`run.sh atlas-miko`: both back out for the originals; raw output in `atlas-miko/`):

| | T1 | T2 | T3 | T4 | T5 | All tiers |
|---|---|---|---|---|---|---|
| Temple Bell + Castle Locthwain | 27.7% | 39.3% | 32.7% | 31.2% | 23.8% | **30.9%** |
| Otherworld Atlas + Mikokoro | 28.1% | 41.7% | 32.2% | 30.4% | 22.6% | **31.0%** |
| Change | +0.4 | +2.4 | -0.5 | -0.8 | -1.2 | **+0.1** |

No difference (noise about ±3 per tier: two cards change, so the paired games diverge more). The Atlas is slow: it
takes two turns of counters before it draws with a punisher out (or one counter when the draw kills), and in about 190
traced games it was destroyed or Sheoldred was gone before that more often than not; it drew in two of them.

## Candidates: Black Market Connections, The Darkness Crystal, Zodiark (2026-10-12)

Measured on the list with Otherworld Atlas and Mikokoro (`run.sh bmc`, `crystal`, `crystal-zod`; raw output in the
folders of those names). The slots: Accursed Marauder (lowest win rate when cast in a Tier 3 `--analyze`, -6.3) and,
for the package, Restless Bloodseeker (+0.7, a body for a body). None is in the list.

| | T1 | T2 | T3 | T4 | T5 | All tiers |
|---|---|---|---|---|---|---|
| Current list | 28.1% | 41.7% | 32.1% | 30.4% | 22.6% | **31.0%** |
| Black Market Connections for the Marauder | +0.9 | +0.3 | +0.1 | +2.3 | +0.8 | **+0.9** |
| The Darkness Crystal for the Marauder | +1.1 | +1.4 | -0.7 | +0.7 | -0.8 | **+0.3** |
| The Crystal + Zodiark, Umbral God (also for Restless Bloodseeker) | +1.0 | +2.3 | +0.2 | +1.4 | +1.9 | **+1.4** |

All within noise (about ±3.5 per tier, about ±1.6 over all tiers). Black Market Connections and the package are up in
all five tiers. Modeling: Black Market Connections takes a Treasure every turn, a card above 15 life (above 5 with
Sheoldred out, 2026-10-12) and a 3/2 above 20. Forty traced Tier 2 games with the package: Zodiark resolved in 8
(2-3 creatures sacrificed each time), the Crystal exiled opponents' creatures (commanders go to the command zone), and
its {4}{B}{B} reanimation never came up.
