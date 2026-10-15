# Cloud: pinned upgrade packages (not applied, not tested)

JD, 2026-10-13: keep the Cloud deck as it is (the C4 list) for now. These two packages are parked for when JD wants to
push Cloud toward parity (about 25%) with Tiers 1 and 2. Both are built against the C4 list and use only fully
modeled cards (checked with the scratch audit on 2026-10-13). Prices are Scryfall's cheapest printing that day.

## Why
The C4 analysis (`audit/cloud/2026-10-13/c4/`, 480 games per tier) shows Cloud at 14.2% (T1) and 9.6% (T2), dead by
round 9-11, almost always to **combat**: Light-Paws (commander damage), Isshin, Tatyova, Lathril at T1; Brago, Kaalia,
Lord Windgrace, Sythis, Meren at T2. At T2 Cloud is also removed 1.3 times a game. Two rounds of AI fixes moved the
win rate by under a point (`audit/cloud/2026-10-13/aifix2/`), so the remaining gap is the list, not the AI.

## Package D: defense against combat races (`d-defense.swaps`, 8 swaps, about $23)
| Out | In | Price |
|---|---|---|
| Stormfront Riders | Ghostly Prison | $6.62 |
| Beatrix, Loyal General | Norn's Annex | $5.69 |
| Unyielding Gatekeeper | Archangel of Tithes | $4.53 |
| Ennis, Debate Moderator | Windborn Muse | $0.81 |
| Explorer's Scope | Baird, Steward of Argive | $0.32 |
| White Mage's Staff | Wrath of God | $3.97 |
| Unfinished Business | Day of Judgment | $0.92 |
| Rootborn Defenses | Fumigate | $0.25 |

Attack taxes slow the creature decks; the wipes leave Cloud's Equipment on the battlefield and Cloud comes back from
the command zone. (The earlier Cloud, Planet's Champion deck measured Ghostly Prison +1.2 and Windborn Muse +1.0.)

## Package R: removal and Cloud protection (`r-removal.swaps`, 6 swaps, about $24)
| Out | In | Price |
|---|---|---|
| Adelbert Steiner | Swords to Plowshares | $0.98 |
| Jocasta, Automaton Avenger | Path to Exile | $0.84 |
| Secure the Scene | Council's Judgment | $1.09 |
| Recommission | Generous Gift | $0.97 |
| Nim Deathmantle | Mother of Runes | $11.76 |
| Helitrooper | Flawless Maneuver | $8.86 |

Cheaper options: Oblivion Ring ($0.29) or Banishing Light ($0.66) for Flawless Maneuver; drop Mother of Runes to save
$12.

## Test plan when resumed
Three paired runs at Tiers 1 and 2 only, 1,000 games per tier: D alone, R alone, D + R (concatenate the two files).
About 15 minutes each at 7 threads. Command per tier (run from the repo root, one `--swap` per line of the file):

```
commander-sim --deck cloud --pool t1 --profile loose --games 1000 --ai lookahead --jobs 7 --swap "OUT=>IN" ...
```

Neither package adds a Game Changer, so Cloud stays Bracket 3. If D + R together still fall well short of 25%, the
likely conclusion is that a 2/1 voltron commander can't reach T1/T2 parity in this pool.

---

# 2026-10-15: Package D applied; packages S, C, B designed (not tested)

Package D was applied to the real list on 2026-10-15 (JD's choice; `d-defense.swaps` is kept as the record). Measured on
the AI-fixed Cloud, look-ahead, 2,000 games per tier, paired: T1 15.8 -> 22.2 (+6.4), T2 13.8 -> 20.1 (+6.3). Package R
was -1.9 / -1.1 (cutting creatures and Equipment for removal spells hurt). The three packages below are built **on top of
the D list** (every "Out" is in it), use disjoint cuts so they can be combined, and keep the creature count up (S +3,
C +3, B +2 creatures, since the cuts are mostly rocks, reactive instants and lands). Every "In" card is Full in
`cardgen audit` (Wall of Omens is Full-auto, a plain "enters, draw a card"). None of the cards is owned: JD's Lightning
Greaves is in Kefka and Skullclamp is in Kefka and Seph, and decks never share cards, so all are buys (Scryfall
cheapest printing, 2026-10-15). None is a Game Changer; Cloud stays Bracket 3.

## Package S: speed, Equipment costs mana once (`s-speed.swaps`, 5 swaps, $52.01; $5.70 without Sigarda's Aid and Stoneforge Mystic)
| Out | In | Price | Why |
|---|---|---|---|
| Fellwar Stone | Brass Squire | $1.20 | {T}: attach an Equipment for free every turn (modeled to attach to Cloud before combat) |
| Thought Vessel | Lightning Greaves | $3.75 | equip {0}: Cloud gets haste and shroud and the Sword moves for free |
| Unbreakable Formation | Danitha Capashen, Paragon | $0.75 | Equipment (and Aura) spells cost {1} less; first strike, vigilance, lifelink carrier. Formation was cast 2% of the games it was drawn |
| Ultimate Magic: Holy | Sigarda's Aid | $16.88 | flash Equipment, and a new Equipment attaches to a creature for free: cast and equip in one |
| Secure the Scene | Stoneforge Mystic | $29.43 | fetches the Sword on turn 2 and puts a big Equipment (Kaldra pieces, Hammer, Arm) onto the battlefield for {1}{W} |
The "lite" version is the first three swaps. Stonehewer Giant (4/4, {1}{W},{T}: Equipment from the library onto the
battlefield) was rejected: its compiled text fetched a non-Equipment artifact (Sol Ring) in a test, so it is not Full.

## Package C: card draw (`c-draw.swaps`, 5 swaps, $14.93)
| Out | In | Price | Why |
|---|---|---|---|
| Nim Deathmantle | Sram, Senior Edificer | $1.73 | draws a card for each of the 14 Equipment cast |
| Citizen's Arrest | Mentor of the Meek | $4.72 | pays {1} to draw when a power-2-or-less creature enters |
| Mind Stone | Welcoming Vampire | $3.15 | 2/3 flier; draws a card (once a turn) when a small creature enters |
| Terminal Moraine | Skullclamp | $5.04 | equip {1} on Cloud (toughness 1) draws two, and Cloud re-enters from the command zone and tutors again |
| Page, Loose Leaf | Wall of Omens | $0.29 | 0/4 blocker that replaces itself |
The hand holds 5.5-6 cards at round 4, so this is as much about spending the hand as about drawing more. Lands go from
33 to 32 and rocks from 6 to 5; the audit found mana on curve. Esper Sentinel ($55) was left out for price.

## Package B: backup carriers (`b-carriers.swaps`, 5 swaps, $4.73)
| Out | In | Price | Why |
|---|---|---|---|
| Helitrooper | Fencing Ace | $0.14 | {1}{W} double strike: a Sword of Fire and Ice on it triggers on both damage steps |
| Loyal Warhound | Mirran Crusader | $0.42 | double strike, protection from black and green (blocked by, damaged by neither) |
| Evolving Wilds | Kor Skyfisher | $0.63 | 2/3 flier for {1}{W}; can bounce Cloud himself to retutor, or a Sword to recast |
| Disenchant | Selfless Spirit | $0.45 | 2/1 flier carrier; sacrifice makes the team indestructible against the wipes |
| Forum Familiar | Adeline, Resplendent Cathar | $3.09 | legendary (Hero's Blade attaches), vigilance, makes a Human for each opponent when she attacks |
Fencing Ace and Mirran Crusader were Partial (printed keywords only) and are now Full with tests in
`cards_cloud_pkg2.rs`. Creatures replace creatures or low-use cards: the count rises from 28 to 30.

## Test plan (needs JD's go; look-ahead, loose, paired against the D list)
- Runs: S, C, B each alone at T1 and T2 (6 runs), then the best combination (and S-lite if S alone is positive but the
  gain does not cover $52). 2,000 games per tier is about 5 minutes at 22 threads: 6 runs about 30 minutes, the best
  combination about 10 minutes more, so roughly 40-45 minutes in all. At 7 threads multiply by three.
- Combining all three removes 4 rocks and 2 lands (6 -> 2 mana rocks and 31 lands): if the full combination loses,
  rerun it keeping Mind Stone and Evolving Wilds.
- Command per tier, one `--swap` per line of each file:
  `commander-sim --deck cloud --pool t1 --profile loose --games 2000 --ai lookahead --jobs 22 --swap "OUT=>IN" ...`

## Results, 2026-10-15 (look-ahead, loose, 2,000 games per tier, paired, on the AI-fixed binary)

| Package | Base | T1 | T2 | Verdict |
|---|---|---|---|---|
| D defense | current list | 15.8 -> 22.2 (+6.4) | 13.8 -> 20.1 (+6.3) | **chosen by JD, applied** |
| R removal | current list | -1.9 | -1.1 | no |
| D + R | current list | +4.9 | (stopped) | worse than D alone |
| S speed (Stoneforge Mystic dropped by JD: 4 swaps) | D list | 22.1 -> 20.6 (-1.6) | 19.1 -> 19.9 (+0.9) | no gain (Danitha's discount not modeled) |
| C draw | D list | 22.1 -> 23.2 (+1.1) | skipped | within noise |
| B carriers | D list | 22.1 -> 21.5 (-0.6) | skipped | no |
| S + C + B | D list | -1.5 | skipped | no |

S-lite was skipped (S showed no gain). Tier 2 runs were skipped when Tier 1 gained under +3. Outputs:
`audit/cloud/2026-10-15/` (AI fixes, D, R, D+R) and `audit/cloud/2026-10-15/pkg2/`.

---

# 2026-10-15: Package M (mana), designed, not tested (`m-mana.swaps`, `m-lite.swaps`)

Built against the current list (C4 + D + AI fixes). Smoke-run only (200 games, T1): both load, pre-flight shows no
WARNING, no result claimed. Prices: Scryfall batch, 10/15. None owned (nothing in bulk/binder; the owned rocks Moss
Diamond, Talisman of Conviction, Chromatic Lantern, Star Compass are outside white's colour identity or in other decks).
No Game Changer added (Bracket 3 unchanged).

| Out | In | Price | Why |
|---|---|---|---|
| Unbreakable Formation (cast 2.9% when drawn) | Worn Powerstone | $0.38 | taps for {C}{C}; late-game equip mana (+0.7 mana on turn 8 in a probe) |
| Ultimate Magic: Holy (11.3%) | Foundry Inspector | $0.39 | artifact spells cost {1} less: 22 artifacts incl. every Sword; cast 84% when drawn in probe |
| Secure the Scene (stuck 49.5%) | Burnished Hart | $0.30 | 3 mana for two basics: ramp and thinning (Full-auto, not hand-modeled) |
| Recommission (stuck 44%) | Weathered Wayfarer | $2.44 | {W} one-drop; fetches a land when behind on lands |
| Veteran's Powerblade (stuck 36%) | Stoneforge Mystic | $29.43 | fetches an Equipment; {1}{W},{T} puts one onto the battlefield, skipping its cost |

**M-lite** = first three rows, about $1.07. **M** = all five, about $33 (Stoneforge is $29 of it; drop its row for
about $3.5).

Probed and rejected: Everflowing Chalice (AI never casts it: 0% in 500 games), Thran Dynamo and Hedron Archive (stuck
50%, cast round 8), Pearl Medallion (white spells only, Equipment is colourless), Mox Amber ($85), Mana Vault, Chrome
Mox, Ancient Tomb (Game Changers, $110+), Marble Diamond (tapped, same as Fellwar), Talisman of Conviction (R).
Caveat from the probe: the new rocks are cast late (round 6.5-8), so mana on turn 4 did not move (3.95 vs 3.99);
the gain, if any, is turn 6-8. Cheap early mana is scarce in mono-white.

Test: paired look-ahead, `--swap` per line, 2,000 games per tier, T1 first, T2 only if T1 gains +3.
