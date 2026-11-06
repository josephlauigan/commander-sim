# pool-lists report (2026-11-06)

## Tier 6: lists vs their source tournament lists (EDHTop16 pages fetched; none gave a Moxfield link)

No card was changed. Every difference from a source list is a logged, deliberate swap (CHANGELOG 2026-10-24 "aligned with 8 recent
winning lists", 2026-10-24 later "JD: rebalance Tier 6", 2026-11-03 "JD: rework the three slow Tier 6 lists"). None of the six
source lists plays Brainstorm, Ponder, Preordain or Opt, so the low cantrip rate is the real lists (Rograkh & Silas plays
Gitaxian Probe in both source and ours).

| List | Cards differing from source | Lands (src / ours) | Cantrips before / after |
| --- | --- | --- | --- |
| Dack Fayden | out Gideon of the Trials, Kozilek's Command, Staff of Domination; in Coalition Relic, Prismatic Lens, Thorn of Amethyst | 27 / 27 | 0 / 0 |
| Kinnan | out Colossal Skyturtle, Dispel, Dizzy Spell, Emrakul, Exotic Orchard, Imposter Mech, Sudden Substitution, Twincast, Volatile Stormdrake; in Bloom Tender, Cryptolith Rite, Formidable Speaker, Gaea's Cradle, Green Sun's Zenith, Incubation Druid, Seedborn Muse, Veil of Summer, Whir of Invention | 24 / 24 | 0 / 0 |
| Kraum & Tymna | out Tarnished Citadel; in Windswept Heath | 25 / 25 | 0 / 0 |
| Rograkh & Silas | out Chromatic Star, Flashback, Talisman of Indulgence; in Commandeer, Snapback, Swan Song | 23 / 23 | 1 (Gitaxian Probe) / 1 |
| Rograkh & Thrasios | out Allosaurus Shepherd, Chatterstorm, Commandeer, Dramatic Reversal, Emerald Charm, Misdirection, Shang-Chi, Urza's Saga, Wan Shi Tong; in Delighted Halfling, Elvish Mystic, Growing Rites of Itlimoc, Kinnan, Llanowar Elves, Mana Confluence, Phyrexian Metamorph, Seedborn Muse, Sylvan Scrying | 27 / 27 | 0 / 0 |
| Sisay | out Avacyn's Pilgrim, Dismember, Elves of Deep Shadow, Forbidden Orchard, Lightning Greaves, Magda, Marsh Flats, Mockingbird, Redirect Lightning, Spider-Woman; in Ancient Tomb, Arcane Signet, Crop Rotation, Delighted Halfling, Force of Negation, Into the Flood Maw, Nature's Rhythm, Orcish Bowmasters, Scrubland, Swords to Plowshares | 28 / 28 | 0 / 0 |

(Dack: the source's "6 Plains" are 5 Plains + 1 Snow-Covered Plains; ours match. The missing land drops are mana-base size and the
mulligan rule, not list errors; the lists have the source's lands.)

## Tier 3 Atraxa (atraxa-infect)
Build: EDHREC Atraxa, Praetors' Voice average decks by theme (json.edhrec.com, read 2026-11-06): Infect/Upgraded (4,118 lists, the
list that was in), +1/+1 Counters (2,170), Planeswalkers (1,810), Proliferate (1,496). Infect as one plan, counters and planeswalkers as the other.
- Out (12): Blightbelly Rat, Contaminant Grafter, Venerated Rotpriest, Norn's Decree, Infectious Inquiry, Prologue to Phyresis,
  Drown in Ichor, Unnatural Restoration (infect-only cards the other themes do not run); Astral Cornucopia, Fellwar Stone
  (12 ramp pieces on 36 lands), Exotic Orchard, Rejuvenating Springs (tapped).
- In (12): Hardened Scales, Walking Ballista, Courser of Kruphix, Eternal Witness, Shalai, Spark Double, The Wandering Emperor,
  Ichormoon Gauntlet, Teferi Hero of Dominaria, Elspeth Sun's Champion, Liliana Dreadhorde General, Mana Confluence.
- All 12 in: Commander-legal, inside WUBG, **Full** in `cardgen audit` (the deck now has no Partial/Unmodeled card). Game Changers unchanged (Farewell, Narset, Rhystic Study).
- Lands 36 -> 35 (22 nonbasic + Mana Confluence + 12 basic). Sources per colour about W 17, U 17, B 18, G 19 (plus Birds, Bloom Tender, Signet, Lantern).
- Infect/toxic creatures 12 -> 9; poison spells/enchantments 6 -> 2 (Phyresis Outbreak, Infectious Bite); planeswalkers 5 -> 9;
  curve top-end: 3 more cards at 5-6 mana, Hardened Scales at 1.
- Not simulated (no sims allowed): the poison share and the round-2-4 hands are for the coordinator to measure.
- Cards that remain below Full: none new. The old deck had Full-auto and Approximate cards (Infectious Bite, Phyresis Outbreak
  Approximate; Blighted Agent, Ichor Rats, Bloated Contaminator, Metastatic Evangel, Glistening Sphere, Vraska's Fall Full-auto); unchanged by this task.

Files: decklists/pool/t3-high-b3/atraxa-infect.md, decklists/pool/README.md, decklists/pool/CHANGELOG.md, data/cards.json, data/decks.json, data/scryfall_cache.json (12 new cards + 20 audited), rust/agents/handoff-pool-lists.md.
