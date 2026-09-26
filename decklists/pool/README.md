# Opponent Pools

Twenty-five outside decks for the pod simulator, in five power tiers of five. They replace Joseph's own four decks as opponents, so a change to one of his decks is measured against a fixed field instead of against his other decks (which made every result zero-sum).

## How the pools are meant to be used

For each game, seat the deck under test plus **three opponents drawn at random without replacement from one tier's pool**, with seat order randomized, from a seeded RNG so runs are reproducible and A/B comparisons can be paired (same seeds, same opponent draws). Report the test deck's win rate against the 25% even-share baseline, and break results out per opponent.

Within each tier, the five decks cover different axes: a combat/aggro deck, a grind or value deck, a combo or one-turn-kill threat, an interaction-heavy or control deck, and a pillowfort, stax, or disruption deck. Archetypes that overlap Joseph's own decks (Grixis spellslinger, reanimator, 5-color tokens, amass) are deliberately avoided.

## Tiers

| Tier | Folder | Target | Rules used to build it |
| --- | --- | --- | --- |
| 1 | `t1-high-b2-low-b3/` | High Bracket 2 / Low Bracket 3 | 0 Game Changers, no two-card infinite combos. Upgraded-precon power: focused synergy, some tutors, modest interaction. |
| 2 | `t2-mid-b3/` | Mid Bracket 3 | 1–2 Game Changers, no two-card infinite combos, no mass land denial or chained extra turns. Tuned synergy and real interaction. |
| 3 | `t3-high-b3/` | High Bracket 3 | Exactly 3 Game Changers. Late-game combos allowed (none that assemble early); efficient tutors and removal. |
| 4 | `t4-low-b4/` | Low Bracket 4 | Unrestricted Game Changers (typically 3–7). Compact combos allowed. Limited fast mana (Ancient Tomb at most; no zero-mana Moxen, Mana Vault, or Lotus Petal). |
| 5 | `t5-high-b4/` | High Bracket 4 | Unrestricted. Fast mana (Mana Vault, Chrome Mox, Mox Diamond, Lotus Petal), free counterspells, tutored compact combos. Just under cEDH. |

## Decks

| Tier | Deck | Commander | Axis | GCs | Lands | Combo |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | [Isshin — Mardu Attack Triggers](t1-high-b2-low-b3/isshin-mardu-attack-triggers.md) | Isshin, Two Heavens as One | Aggro / combat pressure | 0 | 35 | No |
| 1 | [Tatyova — Simic Landfall Ramp](t1-high-b2-low-b3/tatyova-simic-landfall.md) | Tatyova, Benthic Druid | Ramp / goldfish value | 0 | 38 | No |
| 1 | [Teysa Karlov — Orzhov Aristocrats](t1-high-b2-low-b3/teysa-orzhov-aristocrats.md) | Teysa Karlov | Aristocrats / incremental drain | 0 | 37 | No |
| 1 | [Light-Paws — Mono-White Aura Voltron](t1-high-b2-low-b3/light-paws-aura-voltron.md) | Light-Paws, Emperor's Voice | Voltron (commander damage) | 0 | 35 | No |
| 1 | [Lathril — Golgari Elves](t1-high-b2-low-b3/lathril-golgari-elves.md) | Lathril, Blade of the Elves | Go-wide creatures | 0 | 34 | No |
| 2 | [Kaalia of the Vast — Mardu Angels, Demons & Dragons](t2-mid-b3/kaalia-mardu-creature-cheat.md) | Kaalia of the Vast | Fast threats / combat | 2 | 37 | No |
| 2 | [Meren of Clan Nel Toth — Golgari Recursion](t2-mid-b3/meren-golgari-recursion.md) | Meren of Clan Nel Toth | Grind / recursion value | 2 | 36 | No |
| 2 | [Sythis — Selesnya Enchantress Pillowfort](t2-mid-b3/sythis-selesnya-enchantress.md) | Sythis, Harvest's Hand | Pillowfort / enchantress | 2 | 36 | No |
| 2 | [Brago — Azorius Blink Control](t2-mid-b3/brago-azorius-blink-control.md) | Brago, King Eternal | Control / interaction-heavy | 2 | 36 | No |
| 2 | [Lord Windgrace — Jund Lands](t2-mid-b3/lord-windgrace-jund-lands.md) | Lord Windgrace | Lands / ramp value | 2 | 41 | No |
| 3 | [Korvold — Jund Sacrifice & Treasure](t3-high-b3/korvold-jund-sacrifice.md) | Korvold, Fae-Cursed King | Sacrifice value engine | 3 | 35 | Yes |
| 3 | [Marwyn — Mono-Green Elves](t3-high-b3/marwyn-mono-green-elves.md) | Marwyn, the Nurturer | Go-wide / Craterhoof finish | 3 | 33 | No |
| 3 | [Atraxa — Four-Color Superfriends](t3-high-b3/atraxa-superfriends.md) | Atraxa, Praetors' Voice | Planeswalker value / proliferate | 3 | 37 | No |
| 3 | [Aurelia — Boros Extra Combats](t3-high-b3/aurelia-boros-extra-combats.md) | Aurelia, the Warleader | Aggro / extra combats | 3 | 35 | Yes |
| 3 | [Tergrid — Mono-Black Discard & Edicts](t3-high-b3/tergrid-mono-black-disruption.md) | Tergrid, God of Fright | Disruption / discard & edicts | 3 | 36 | No |
| 4 | [Yuriko — Dimir Ninja Tempo](t4-low-b4/yuriko-dimir-ninjas.md) | Yuriko, the Tiger's Shadow | Tempo / evasive chip damage | 6 | 34 | No |
| 4 | [Krenko, Mob Boss — Mono-Red Goblins](t4-low-b4/krenko-mono-red-goblins.md) | Krenko, Mob Boss | Explosive go-wide / combo | 3 | 34 | Yes |
| 4 | [Chulane — Bant Value & Aluren](t4-low-b4/chulane-bant-value-combo.md) | Chulane, Teller of Tales | Value engine / Aluren combo | 7 | 34 | Yes |
| 4 | [Prosper — Rakdos Exile & Treasure](t4-low-b4/prosper-rakdos-exile-treasure.md) | Prosper, Tome-Bound | Treasure / impulse-draw value | 6 | 35 | Yes |
| 4 | [Heliod, Sun-Crowned — Mono-White Stax](t4-low-b4/heliod-mono-white-stax.md) | Heliod, Sun-Crowned | Stax / hatebears + compact combo | 5 | 33 | Yes |
| 5 | [Urza, Lord High Artificer — Mono-Blue Artifacts](t5-high-b4/urza-mono-blue-artifacts.md) | Urza, Lord High Artificer | Artifact combo + stax | 14 | 29 | Yes |
| 5 | [Kinnan — Simic Mana Combo](t5-high-b4/kinnan-simic-mana-combo.md) | Kinnan, Bonder Prodigy | Mana combo | 18 | 30 | Yes |
| 5 | [Winota — Boros Humans & Hatebears](t5-high-b4/winota-boros-humans-cheat.md) | Winota, Joiner of Forces | Aggro-cheat with stax | 9 | 32 | Yes |
| 5 | [Zur the Enchanter — Esper Control & Oracle](t5-high-b4/zur-esper-enchantment-control.md) | Zur the Enchanter | Control + compact combo | 20 | 31 | Yes |
| 5 | [Yawgmoth — Mono-Black Undying Combo](t5-high-b4/yawgmoth-mono-black-aristocrats.md) | Yawgmoth, Thran Physician | Aristocrats combo | 12 | 31 | Yes |

## Notes on the lists

- Every list is exactly 100 cards, singleton, color-identity legal, free of banned cards, and checked against the 53-card Game Changers list as of the February 9, 2026 update (unchanged as of September 2026). Card names were validated against a card database; re-verify against Scryfall if anything fails to resolve.
- Tier 1 and 2 decks have no two-card infinite combos. Tier 3 decks have exactly three Game Changers and only late-game combos. Krenko sits in Tier 4 despite three Game Changers because Kiki-Jiki + Zealous Conscripts is an early two-card combo.
- Several decks carry graveyard hate on purpose (Rest in Peace, Bojuka Bog, Dauthi Voidwalker, Tormod's Crypt, Scavenger Grounds) so Sephiroth faces a realistic amount of it. Winota's Magus of the Moon and the stax pieces in Heliod and Urza test the multicolor mana bases.
- Grand Arbiter Augustin IV (Azorius stax) was Tier 4's stax deck until September 2026. It is kept in `retired/` and is no longer drawn. Its rules checked out, but the list had no reliable way to win in the sim: it won about 4% against Tier 3, 7.5% even when it could not lose life, and the same with its stax effects switched off. Heliod, Sun-Crowned took the slot (see pool-results.md).
- Six lists were tuned in September 2026 (with the user's approval) to bring within-tier balance into 15-35% under the look-ahead AI: Meren, Atraxa and Sythis were trimmed, Brago, Marwyn and Yawgmoth strengthened. Each file's notes list the swaps; the old lists are in git history.
- These are reasonable, representative builds rather than tuned tournament lists. If the sim shows one pool deck winning far more or less than 25% against its own tier, suspect card modeling first, then the list.
