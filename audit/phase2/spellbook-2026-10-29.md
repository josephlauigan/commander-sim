# Phase 2: the combo catalogue against Commander Spellbook (2026-10-29)

Combo data by **Commander Spellbook** (https://commanderspellbook.com), from its public `find-my-combos` API
(https://backend.commanderspellbook.com/find-my-combos), fetched 2026-10-29. Commander Spellbook is unofficial Fan
Content permitted under the Wizards of the Coast Fan Content Policy; its code is MIT-licensed, and its API asks for
sparse, named requests, credit and a link back. The site states no separate licence for the combo data, so the
stored copy keeps only what the sim needs and credits Spellbook in every file.

## How

- `scripts/spellbook-fetch.sh` (curl + jq) sends one POST per deck in `data/decks.json` (12 of JD's, 36 pool decks,
  48 requests, 2 s apart, named User-Agent, 429 handled) and writes `data/spellbook/<deck>.json` (3.3 MB in all):
  the source, credit and fetch date, the request's SHA-256 (an unchanged deck is not fetched again; `--force`
  refetches), then per combo its Spellbook id, cards (zones, "must be commander"), results (Spellbook's standalone and
  contextual features; its helper features are dropped), templates ("a sacrifice outlet"), prerequisites, mana,
  bracket tag, Commander legality and popularity. `included` (every card in the list) and `almost_included` (one card
  missing, named in `missing`; empty when only a template's card is missing) are kept apart.
  `included_by_changing_commanders` is kept too (empty for every deck); the "by adding colours" lists are not.
- `rust/crates/sim-core/tests/spellbook.rs` matches each Spellbook combo's card set against every modelled line: the
  combo catalogue (`combos::all()`, 72 lines incl. the quiet Tier 6 lines by their real groups) and Sephiroth's
  `mine::LOOPS`, one piece per group. `Exact`: the line's pieces are the combo's cards. `Smaller`: the line needs
  only some of them. `Template`: a line group fills one of Spellbook's template slots with a card the deck plays
  (loose: in Kinnan, Kinnan fills "a permanent castable for {C}"). `Bigger`: every combo card is a piece but the
  line needs more (usually the payoff Spellbook lists as a separate combo, or a mana sink for an infinite-mana line).
  A short hand-checked list (`CARD_CODE`) marks combos the sim plays through card code rather than a line.
- `spellbook_report` (ignored test) writes the tables below:
  `SPELLBOOK_REPORT=<file> cargo test --release --test spellbook -- --ignored spellbook_report`.

## Summary

253 legal "included" combos over the 48 decks; the sim covers 120 (a line or card code). 32 uncovered two- and
three-card lines win the game (or drain / deal infinite damage), 20 of them in Yawgmoth (t5).

| tiers | included | covered | uncovered 2-3 card wins |
|---|---|---|---|
| JD's decks | 18 | 16 | 2 |
| t0 | 0 | 0 | 0 |
| t1 | 21 | 10 | 0 |
| t2 | 9 | 1 | 1 |
| t3 | 31 | 20 | 4 |
| t4 | 44 | 16 | 3 |
| t5 | 96 | 39 | 20 |
| t6 | 34 | 18 | 2 |

Tier 0 and most casual lists hold no Spellbook combo at all, which agrees with the casual-combos audit
(`audit/realism/casual-combos-2026-10-26.md`). Most uncovered combos do not win: infinite tokens, mana or ETBs with
no payoff in the list (Dack's Abdel + Felidar blinks, Tatyova's Retreat to Coralhelm lands, Teysa's Broodmoth loops).

## The lines that matter most (2-3 cards, win, deck with tutors)

Tutor counts are cards whose Oracle text searches for a nonland card (approximate).

| deck (tier) | tutors | Spellbook line(s) the sim lacks |
|---|---|---|
| **kefka (JD, t3)** | 3 (Diabolic, Profane, Grim Servant) | Underworld Breach + Lotus Petal + Brain Freeze (needs 6+ other cards in the graveyard). The two Kefka loops are played (card code). |
| scion (JD, t6) | 9 | Underworld Breach + Lotus Petal + Brain Freeze (the sim plays Breach + LED + Brain Freeze and Scion-as-Prismari + Breach + Manamorphose + Brain Freeze, not the Petal version) |
| kraum-tymna, rograkh-silas (t6) | 10, 9 | the same Breach + Lotus Petal + Brain Freeze line; rograkh-silas also Breach + Wheel of Fortune + Jeska's Will / LED, Breach + Grinding Station (no win alone) |
| yawgmoth (t5) | 5 | 20 lines: Warren Soultrader + Gravecrawler or Forsaken Miner + a drain (Blood Artist, Vengeful Bloodwitch, Sephiroth, Ayara); Forsaken Miner + Phyrexian Altar + a drain; Yawgmoth + Nest of Scarabs + a drain; Yawgmoth + Mikaeus + Gray Merchant / Ayara; Yawgmoth + Pawn of Ulamog / Sifter of Skulls + Sephiroth / Bloodwitch (an undying creature). The sim's lines know Blood Artist-style drains (`DRAIN_PAYOFFS`) but not Bloodwitch, Sephiroth, Ayara or Nest of Scarabs, and no Soultrader + Gravecrawler / Forsaken Miner loop. |
| meren (t2) | 6 | Yawgmoth + Mikaeus + Gray Merchant (Gray Merchant is not in `DRAIN_PAYOFFS`) |
| korvold (t3) | 3 | Bootleggers' Stash + Revel in Riches (Revel's win is card code; whether Bootleggers' Stash is, was not checked) |
| krenko (t4) | 2 | Kiki-Jiki + Thornbite Staff + Goblin Bombardment / Goblin Sharpshooter; Skirk Prospector + Pashalik Mons + Mana Echoes. Also (no win alone) Krenko + Skirk Prospector + a haste source or untapper (Goblin Warchief, Goblin Chieftain, Howlsquad Heavy, Fervor, Thousand-Year Elixir, Staff of Domination, Umbral Mantle: 7 variants), Kiki-Jiki + Conspicuous Snoop + Goblin Recruiter, Kiki-Jiki + Thornbite Staff + Skirk Prospector / Ashnod's Altar. |
| aurelia (t3) | 4 | Aurelia + Helm of the Host; Aurelia + Sword of Hearth and Home; Sword of Feast and Famine + Aggravated Assault (infinite combats; Aurelia has card code in `t3_aurelia.rs`, but none of these loops is a line) |

Also worth a later look, not wins alone: Kinnan t5's Basalt Monolith + Nyxbloom Ancient, Incubation Druid +
Freed from the Real / Pemmin's Aura, Tidespout Tyrant + a 0-1 cost rock; Urza's 17 (Hullbreaker + Mana Vault, Mystic
Forge + Top + cost reducer ...); Sisay's Derevi + Emiel + Mana Vault / Selvala; Marwyn's Wirewood Channeler + Staff
of Domination / Umbral Mantle; Miirym's Astral Dragon + Cursed Mirror; Zur's Solemnity + Phyrexian Unlife (a lock);
Kaalia + Master of Cruelties (a kill when it connects).

Not modelled here, by design: none of these was added (brief rule 4). The guard's `KNOWN_GAPS` lists the 15
(deck, combo) pairs whose cards are all catalogue pieces; the rest are in the per-deck tables.

## The sim's lines that Spellbook does not list

| deck | sim line | why, as far as I can tell |
|---|---|---|
| scion (JD) | Scion as Prismari + Breach + Manamorphose + Brain Freeze | the line needs Scion to become Prismari; Spellbook does not model commander copies |
| lord-windgrace (t2) | Dark Depths + Thespian's Stage | Spellbook returns no Dark Depths variant for the list; the result is one 20/20, not an infinite, which is likely why. The line is real (Oracle text). |
| meren (t2) | Yawgmoth + undying loop + a death payoff (Yawgmoth + Mikaeus + Blood Artist / Zulaport) | Spellbook's Yawgmoth + Mikaeus variants each name a third creature (Gray Merchant, Ayara, Kokusho); the loop needs a second non-Human body, which the sim line should check. Worth a look. |
| chulane (t4) | Aluren + Chulane + Shrieking Drake / Whitemane Lion | Spellbook returns no Aluren + bouncer variant here; by Oracle text the loop draws the deck (Aluren casts the Drake free, Chulane draws, the Drake returns itself), so the sim is right |
| miirym (t5) | Wrathful Red Dragon + Blasphemous Act | a one-shot burn, not a combo by Spellbook's definition |
| dack (t6) | Dack's first-flip piles; Preston + Felidar + Radiant Solar (loop running) | the pile plan is a sim abstraction; Spellbook lists the pieces as Abdel / Preston + Felidar + a blink (infinite tokens) |
| kraum-tymna, rograkh-silas (t6) | Ad Nauseam into a win | a draw spell with a follow-up, not a combo |
| rograkh-thrasios (t6) | Badgermole Cub + clone + Oboro Breezecaller + Gaea's Cradle; + Eternal Witness + Snap | Spellbook has no Badgermole Cub (2025) variants for this list yet |

21 catalogue lines are held by no deck: the Godo lines (the Godo deck has left the pool), Kiki-Jiki + Conscripts,
Karn the Great Creator + Mycosynth Lattice (JD's Karn has no Lattice), Dualcaster, Rionya, Heliod + Triskelion,
Peregrine Drake + Deadeye, Melira + Finks, Deathmantle + Priest of Gix and others (list at the end).

## Spot checks (deck lists and `data/scryfall_cache.json`)

Matches:
1. yawgmoth: Warren Soultrader + Pitiless Plunderer + Reassembling Skeleton + Blood Artist (Exact). All four are in
   the list; each sacrifice makes two Treasures (Soultrader, Plunderer) for the Skeleton's {1}{B} and drains 1 for the
   1 life Soultrader costs.
2. kinnan-simic: Kinnan + Basalt Monolith (Exact). Basalt taps for {C}{C}{C}, Kinnan adds one more, the untap costs
   {3}: one net mana a loop.
3. edgar: Sanguine Bond + Exquisite Blood (Exact): each trigger feeds the other once life is gained.
4. karn (JD): Rings of Brighthearth + Basalt Monolith (Bigger: the sim line also wants a mana sink). Rings copies the
   untap for {2}; Spellbook gives infinite colourless mana only, so needing a sink is right.
5. kinnan-simic: Hullbreaker Horror + Sol Ring (Template, via Kinnan): the sim line needs Kinnan, which the deck
   always has; a loose but fair match.

Gaps (all pieces in the lists; Oracle text agrees the line works):
1. sheoldred (JD): Sheoldred + Peer into the Abyss. Played by card code (`peer_options` aims Peer at an opponent
   when the draw toll kills); marked `CARD_CODE`.
2. kefka (JD): Kefka, Ruler of Ruin + Psychosis Crawler. Card code (`kefka.rs` `loop_safe`); marked `CARD_CODE`.
3. krenko: Kiki-Jiki + Thornbite Staff + Goblin Bombardment. Kiki makes a hasty token, Bombardment sacrifices it, the
   Staff untaps Kiki: infinite damage. A real gap.
4. kefka (JD): Underworld Breach + Lotus Petal + Brain Freeze. Petal escapes for {0} plus three cards and makes one
   mana, each cast is storm; Brain Freeze escapes for {1}{U}. A real gap (graveyard fuel permitting).
5. kinnan-simic: Basalt Monolith + Nyxbloom Ancient. Basalt makes nine, the untap costs three. Nyxbloom is card code
   (`engine/mana.rs`) but the loop is not a line: a real gap (infinite mana, no win alone).

## Checks

- `cargo test --release -j 12` (memory cap): rc 0, 217 binaries, 3,587 passed, 0 failed (the new `spellbook` binary: 2 passed, 1 ignored). `cargo build --release --all-targets`: rc 0, 0 warnings.
  `cardgen validate`: rc 0, "all decks passed".
- No engine change: `git diff origin/main -- rust/crates/sim-core/src` is empty (the PR adds a test, data and a
  script), so the sim's results cannot change and no byte-identity run was needed.

## Per deck

| tier | deck | included (legal) | sim covers (line or card code) | uncovered 2-3 card wins | catalogue lines held | almost included |
|---|---|---|---|---|---|---|
| JD (t1) | alela | 0 | 0 | 0 | 0 | 11 |
| JD (t1, t2) | galadriel | 0 | 0 | 0 | 0 | 26 |
| JD (t1, t2, t3) | cloud | 0 | 0 | 0 | 0 | 1 |
| JD (t1, t2, t3) | veyran | 0 | 0 | 0 | 0 | 50 |
| JD (t3) | kefka | 3 | 2 | 1 | 0 | 29 |
| JD (t3, t4) | sauron | 0 | 0 | 0 | 0 | 34 |
| JD (t3, t4) | seph | 7 | 7 | 0 | 3 | 680 |
| JD (t3, t4) | sheoldred | 2 | 2 | 0 | 0 | 21 |
| JD (t4, t5) | karn | 1 | 1 | 0 | 1 | 38 |
| JD (t5) | thrasios | 1 | 1 | 0 | 1 | 48 |
| JD (t5) | yshtola | 0 | 0 | 0 | 0 | 36 |
| JD (t6) | scion | 4 | 3 | 1 | 4 | 93 |
| t0 | anikthea-enduring-enchantments | 0 | 0 | 0 | 0 | 20 |
| t0 | captain-america-avengers-assemble | 0 | 0 | 0 | 0 | 7 |
| t0 | giada-calling-all-angels | 0 | 0 | 0 | 0 | 1 |
| t0 | stella-lee-quick-draw | 0 | 0 | 0 | 0 | 22 |
| t0 | zurgo-mardu-surge | 0 | 0 | 0 | 0 | 36 |
| t1 | isshin-mardu-attack-triggers | 0 | 0 | 0 | 0 | 19 |
| t1 | lathril-golgari-elves | 0 | 0 | 0 | 0 | 16 |
| t1 | light-paws-aura-voltron | 0 | 0 | 0 | 0 | 9 |
| t1 | tatyova-simic-landfall | 3 | 0 | 0 | 0 | 79 |
| t1 | teysa-orzhov-aristocrats | 18 | 10 | 0 | 3 | 246 |
| t2 | brago-azorius-blink-control | 7 | 1 | 0 | 2 | 143 |
| t2 | kaalia-mardu-creature-cheat | 1 | 0 | 0 | 0 | 58 |
| t2 | lord-windgrace-jund-lands | 0 | 0 | 0 | 1 | 82 |
| t2 | meren-golgari-recursion | 1 | 0 | 1 | 1 | 137 |
| t2 | sythis-selesnya-enchantress | 0 | 0 | 0 | 0 | 16 |
| t3 | atraxa-infect | 2 | 0 | 0 | 0 | 31 |
| t3 | aurelia-boros-extra-combats | 4 | 1 | 3 | 1 | 74 |
| t3 | korvold-jund-sacrifice | 13 | 11 | 1 | 2 | 291 |
| t3 | marwyn-mono-green-elves | 12 | 8 | 0 | 2 | 87 |
| t3 | tergrid-mono-black-disruption | 0 | 0 | 0 | 0 | 1 |
| t4 | chulane-bant-value-combo | 11 | 4 | 0 | 3 | 191 |
| t4 | edgar-markov-mardu-vampires | 6 | 6 | 0 | 2 | 87 |
| t4 | krenko-mono-red-goblins | 21 | 5 | 3 | 2 | 137 |
| t4 | prosper-rakdos-exile-treasure | 6 | 1 | 0 | 1 | 69 |
| t4 | yuriko-dimir-ninjas | 0 | 0 | 0 | 0 | 32 |
| t5 | kinnan-simic-mana-combo | 21 | 15 | 0 | 9 | 121 |
| t5 | miirym-temur-dragons | 2 | 0 | 0 | 1 | 75 |
| t5 | urza-mono-blue-artifacts | 22 | 5 | 0 | 4 | 229 |
| t5 | yawgmoth-mono-black-aristocrats | 49 | 18 | 20 | 4 | 307 |
| t5 | zur-esper-enchantment-control | 2 | 1 | 0 | 1 | 72 |
| t6 | dack-fayden-white-stax | 10 | 2 | 0 | 3 | 95 |
| t6 | kinnan-bonder-mana-combo | 5 | 5 | 0 | 5 | 78 |
| t6 | kraum-tymna-blue-breach-midrange | 4 | 3 | 1 | 4 | 92 |
| t6 | rograkh-silas-turbo-breach | 7 | 3 | 1 | 4 | 67 |
| t6 | rograkh-thrasios-creature-combo | 2 | 2 | 0 | 4 | 128 |
| t6 | sisay-legends-toolbox | 6 | 3 | 0 | 2 | 261 |

### alela (JD (t1))

Spellbook finds no combo in this list.

### galadriel (JD (t1, t2))

Spellbook finds no combo in this list.

Almost included (one card missing), winning, 2-3 cards: 1

- Biorhythm + Austere Command (3778-7299): missing Biorhythm

### cloud (JD (t1, t2, t3))

Spellbook finds no combo in this list.

### veyran (JD (t1, t2, t3))

Spellbook finds no combo in this list.

Almost included (one card missing), winning, 2-3 cards: 20

- Curiosity + Niv-Mizzet, Parun (1089-2353): missing Curiosity
- Niv-Mizzet, Parun + Niv-Mizzet, Visionary (2353-6092): missing Niv-Mizzet, Visionary
- Blasphemous Act + Repercussion (2484-4083): missing Repercussion
- Niv-Mizzet, Parun + Ophidian Eye (2353-2396): missing Ophidian Eye
- Tandem Lookout + Niv-Mizzet, Parun (406-2353): missing Tandem Lookout
- Toralf, God of Fury // Toralf's Hammer + Blasphemous Act (1775-2484): missing Toralf, God of Fury // Toralf's Hammer
- Karn, the Great Creator + Aetherflux Reservoir (4740-5089): missing Karn, the Great Creator
- Aetherflux Reservoir + Cyberdrive Awakener (2734-4740): missing Cyberdrive Awakener
- Brash Taunter + Aetherflux Reservoir (220-4740): missing Brash Taunter
- Fury Storm + Ral, Storm Conduit (2655-5005): missing Fury Storm
- Aetherflux Reservoir + Tezzeret, Betrayer of Flesh (3301-4740): missing Tezzeret, Betrayer of Flesh
- Aetherflux Reservoir + Myr Welder (132-4740): missing Myr Welder
- ... and 8 more

### kefka (JD (t3))

| Spellbook id | cards | combo | wins | results | prerequisites | sim line |
|---|---|---|---|---|---|---|
| 1368-1414-4856 | 3 | Underworld Breach + Lotus Petal + Brain Freeze | **win** | Infinite self-mill; Near-infinite mill | You have at least six other cards in your graveyard, plus an additional four cards if Brain Freeze is in your graveyard. | no |
| 2384-6574 | 2 | Kefka, Court Mage // Kefka, Ruler of Ruin + Psychosis Crawler | **win** | Infinite card draw; Near-infinite lifeloss | You have a way to draw a card. | card code: kefka.rs: the Psychosis Crawler loop (`loop_safe`) |
| 4971-6574 | 2 | Niv-Mizzet, the Firemind + Kefka, Court Mage // Kefka, Ruler of Ruin | **win** | Near-infinite damage; Infinite card draw | You have a way to draw a card. | card code: kefka.rs: the Niv-Mizzet loop (`niv_safe`) |

Almost included (one card missing), winning, 2-3 cards: 15

- Underworld Breach + Lion's Eye Diamond + Brain Freeze (1368-3518-4856): missing Lion's Eye Diamond
- Psychosis Crawler + Peer into the Abyss (1561-2384): missing Peer into the Abyss
- Niv-Mizzet, the Firemind + Curiosity (1089-4971): missing Curiosity
- Niv-Mizzet, the Firemind + Niv-Mizzet, Visionary (4971-6092): missing Niv-Mizzet, Visionary
- Niv-Mizzet, the Firemind + Ophidian Eye (2396-4971): missing Ophidian Eye
- Niv-Mizzet, the Firemind + Tandem Lookout (406-4971): missing Tandem Lookout
- Niv-Mizzet, Parun + Kefka, Court Mage // Kefka, Ruler of Ruin (2353-6574): missing Niv-Mizzet, Parun
- Enter the Infinite + Psychosis Crawler (953-2384): missing Enter the Infinite
- Magmakin Artillerist + Kefka, Court Mage // Kefka, Ruler of Ruin (6428-6574): missing Magmakin Artillerist
- Kefka, Court Mage // Kefka, Ruler of Ruin + Glint-Horn Buccaneer (4869-6574): missing Glint-Horn Buccaneer
- Kefka, Court Mage // Kefka, Ruler of Ruin + Feast of Sanity (602-6574): missing Feast of Sanity
- Kefka, Court Mage // Kefka, Ruler of Ruin + Starving Revenant (5796-6574): missing Starving Revenant
- ... and 3 more

### sauron (JD (t3, t4))

Spellbook finds no combo in this list.

Almost included (one card missing), winning, 2-3 cards: 21

- Sheoldred, the Apocalypse + Peer into the Abyss (1561-2498): missing Peer into the Abyss
- The Reaver Cleaver + Aggravated Assault (3750-5039): missing The Reaver Cleaver
- Orcish Bowmasters + Peer into the Abyss (1561-3862): missing Peer into the Abyss
- Sword of Feast and Famine + Aggravated Assault (900-3750): missing Sword of Feast and Famine
- Ancient Copper Dragon + Aggravated Assault (778-3750): missing Ancient Copper Dragon
- Neheb, the Eternal + Aggravated Assault (3750-4945): missing Neheb, the Eternal
- Cavern-Hoard Dragon + Aggravated Assault (3750-5988): missing Cavern-Hoard Dragon
- Magda, Brazen Outlaw + Aggravated Assault (1486-3750): missing Magda, Brazen Outlaw
- Blightsteel Colossus + Chandra's Ignition (1571-2560): missing Blightsteel Colossus
- Neheb, Dreadhorde Champion + Aggravated Assault (1697-3750): missing Neheb, Dreadhorde Champion
- Howlsquad Heavy + Aggravated Assault (3750-6410): missing Howlsquad Heavy
- Hoard Hauler + Aggravated Assault (451-3750): missing Hoard Hauler
- ... and 9 more

### seph (JD (t3, t4))

| Spellbook id | cards | combo | wins | results | prerequisites | sim line |
|---|---|---|---|---|---|---|
| 628-2034--5 | 2 | Mikaeus, the Unhallowed + Ashnod's Altar |  | Infinite colorless mana | Persist Creature | Template: Mikaeus + Triskelion |
| 628-4050--5 | 2 | Mikaeus, the Unhallowed + Phyrexian Altar |  | Infinite colored mana | Persist Creature | Template: Mikaeus + Triskelion |
| 628-2292--5 | 2 | Mikaeus, the Unhallowed + Viscera Seer |  |  | Persist Creature | Template: Mikaeus + Triskelion |
| 2034-5003--73 | 2 | Nim Deathmantle + Ashnod's Altar |  |  | Creature with Persist or Undying | Template: Nim Deathmantle + Ashnod's Altar + Grave Titan |
| 628-5256--5 | 2 | Mikaeus, the Unhallowed + Altar of Dementia | **win** | Infinite mill; Infinite self-mill | Persist Creature | Template: Mikaeus + Triskelion |
| 628-1495 | 2 | Mikaeus, the Unhallowed + Triskelion | **win** | Infinite damage | Triskelion has at least two +1/+1 counters on it. | Exact: Mikaeus + Triskelion |
| 2034-3968-5003 | 3 | Nim Deathmantle + Ashnod's Altar + Grave Titan |  | Infinite colorless mana; Infinite creature tokens |  | Exact: Nim Deathmantle + Ashnod's Altar + Grave Titan |

Catalogue lines the deck holds:

- Mikaeus + Triskelion: Spellbook 628-1495 (Exact)
- Mikaeus + Kitchen Finks: Spellbook 628-2034--5 (Template)
- Nim Deathmantle + Ashnod's Altar + Grave Titan: Spellbook 2034-3968-5003 (Exact)

Almost included (one card missing), winning, 2-3 cards: 114

- Sheoldred, the Apocalypse + Peer into the Abyss (1561-2498): missing Peer into the Abyss
- Forsaken Miner + Phyrexian Altar + Blood Artist (2842-4050-5544): missing Forsaken Miner
- Forsaken Miner + Phyrexian Altar + Sephiroth, Fabled SOLDIER // Sephiroth, One-Winged Angel (4050-5544-6578): missing Forsaken Miner
- Mikaeus, the Unhallowed + Warren Soultrader + Blood Artist (628-2842-5670--5): missing Warren Soultrader
- Luminous Broodmoth + Altar of Dementia (2018-5256--73): missing Luminous Broodmoth
- Approach of the Second Sun + Demonic Tutor (553-4105): missing Approach of the Second Sun
- Altar of Dementia + The Great Henge (3139-5256--5): missing The Great Henge
- Approach of the Second Sun + Vampiric Tutor (553-1827): missing Approach of the Second Sun
- Mikaeus, the Unhallowed + Warren Soultrader + Sephiroth, Fabled SOLDIER // Sephiroth, One-Winged Angel (628-5670-6578--5): missing Warren Soultrader
- Nim Deathmantle + Ashnod's Altar + Marionette Apprentice (2034-5003-6199): missing Marionette Apprentice
- Tombstone Stairwell + Blood Artist (2842-6215): missing Tombstone Stairwell
- Heliod, Sun-Crowned + Triskelion (1274-1495): missing Heliod, Sun-Crowned
- ... and 102 more

### sheoldred (JD (t3, t4))

| Spellbook id | cards | combo | wins | results | prerequisites | sim line |
|---|---|---|---|---|---|---|
| 1561-2498 | 2 | Sheoldred, the Apocalypse + Peer into the Abyss | **win** | Near-infinite card draw for target opponent; Near-infinite lifeloss for target opponent; Target opponent loses the game | An opponent has a library size that is at least half their life total. | card code: sheoldred.rs: Peer into the Abyss at an opponent (`peer_options`) |
| 1561-2720 | 2 | Peer into the Abyss + Underworld Dreams |  | Target opponent loses the game | An opponent has cards in their library greater than their life total. | card code: sheoldred.rs: Peer into the Abyss at an opponent (`peer_options`, `draw_toll`) |

Almost included (one card missing), winning, 2-3 cards: 7

- Orcish Bowmasters + Peer into the Abyss (1561-3862): missing Orcish Bowmasters
- Psychosis Crawler + Peer into the Abyss (1561-2384): missing Psychosis Crawler
- Tombstone Stairwell + Blood Artist (2842-6215): missing Tombstone Stairwell
- Overkill + Jaws of Defeat (6562-6648): missing Jaws of Defeat
- Skirge Familiar + Peer into the Abyss + Exsanguinate (1561-1742-3245): missing Skirge Familiar
- Feast of Sanity + Peer into the Abyss (602-1561): missing Feast of Sanity
- Blitzwing, Cruel Tormentor // Blitzwing, Adaptive Assailant + Peer into the Abyss (1561-5008): missing Blitzwing, Cruel Tormentor // Blitzwing, Adaptive Assailant

### karn (JD (t4, t5))

| Spellbook id | cards | combo | wins | results | prerequisites | sim line |
|---|---|---|---|---|---|---|
| 4131-4235 | 2 | Rings of Brighthearth + Basalt Monolith |  | Infinite colorless mana |  | Bigger: Power Artifact / Rings of Brighthearth + Monolith |

Catalogue lines the deck holds:

- Power Artifact / Rings of Brighthearth + Monolith: Spellbook 4131-4235 (Bigger)

Almost included (one card missing), winning, 2-3 cards: 3

- Cybermen Squadron + Blightsteel Colossus (181-1571): missing Cybermen Squadron
- Karn, the Great Creator + Aetherflux Reservoir (4740-5089): missing Aetherflux Reservoir
- Blightsteel Colossus + Blade of Selves (1571-3851): missing Blade of Selves

### thrasios (JD (t5))

| Spellbook id | cards | combo | wins | results | prerequisites | sim line |
|---|---|---|---|---|---|---|
| 4131-4235 | 2 | Rings of Brighthearth + Basalt Monolith |  | Infinite colorless mana |  | Bigger: Power Artifact / Rings of Brighthearth + Monolith |

Catalogue lines the deck holds:

- Power Artifact / Rings of Brighthearth + Monolith: Spellbook 4131-4235 (Bigger)

Almost included (one card missing), winning, 2-3 cards: 11

- Heliod, Sun-Crowned + Walking Ballista (1274-3693): missing Heliod, Sun-Crowned
- Archangel of Thune + Walking Ballista (2919-3693): missing Archangel of Thune
- Cleric Class + Walking Ballista (104-3693): missing Cleric Class
- Approach of the Second Sun + Diabolic Tutor (553-5273): missing Approach of the Second Sun
- Light of Promise + Walking Ballista (338-3693): missing Light of Promise
- Walking Ballista + The Destined White Mage (3693-6996): missing The Destined White Mage
- Walking Ballista + Sunbond (3693-4017): missing Sunbond
- Walking Ballista + Spider-Man, Peter Parker (3693-6824): missing Spider-Man, Peter Parker
- Walking Ballista + Heroic Feast (3693-7743): missing Heroic Feast
- Biorhythm + Austere Command (3778-7299): missing Biorhythm
- Vraska, Swarm's Eminence + Walking Ballista (2190-3693): missing Vraska, Swarm's Eminence

### yshtola (JD (t5))

Spellbook finds no combo in this list.

Almost included (one card missing), winning, 2-3 cards: 11

- Sanguine Bond + Exquisite Blood (690-3966): missing Exquisite Blood
- Sanguine Bond + Bloodthirsty Conqueror (690-6191): missing Bloodthirsty Conqueror
- Marauding Blight-Priest + Exquisite Blood (110-3966): missing Exquisite Blood
- Marauding Blight-Priest + Bloodthirsty Conqueror (110-6191): missing Bloodthirsty Conqueror
- Orcish Bowmasters + Peer into the Abyss (1561-3862): missing Peer into the Abyss
- Approach of the Second Sun + Mystical Tutor (553-2856): missing Approach of the Second Sun
- Sanguine Bond + Revival // Revenge (690-4624): missing Revival // Revenge
- Approach of the Second Sun + Diabolic Tutor (553-5273): missing Approach of the Second Sun
- Approach of the Second Sun + Necropotence (553-3591): missing Approach of the Second Sun
- Enduring Angel // Angelic Enforcer + Sanguine Bond (690-1162): missing Enduring Angel // Angelic Enforcer
- Near-Death Experience + Necropotence (1624-3591): missing Near-Death Experience

### scion (JD (t6))

| Spellbook id | cards | combo | wins | results | prerequisites | sim line |
|---|---|---|---|---|---|---|
| 1368-1414-4856 | 3 | Underworld Breach + Lotus Petal + Brain Freeze | **win** | Infinite self-mill; Near-infinite mill | You have at least six other cards in your graveyard, plus an additional four cards if Brain Freeze is in your graveyard. | no |
| 1368-3518-4856 | 3 | Underworld Breach + Lion's Eye Diamond + Brain Freeze | **win** | Infinite self-mill; Near-infinite colored mana; Near-infinite mill | You have at least six other cards in hand and/or graveyard. | Bigger: Underworld Breach + Lion's Eye Diamond + Brain Freeze (Thassa's Oracle finish) |
| 742-1295 | 2 | Demonic Consultation + Thassa's Oracle | **win** | Win the game |  | Exact: Thassa's Oracle + Demonic Consultation / Tainted Pact |
| 1295-3093 | 2 | Tainted Pact + Thassa's Oracle | **win** | Win the game | No two cards in library share a name. | Exact: Thassa's Oracle + Demonic Consultation / Tainted Pact |

Catalogue lines the deck holds:

- Thassa's Oracle + Demonic Consultation / Tainted Pact: Spellbook 742-1295 (Exact)
- Underworld Breach + Lion's Eye Diamond + Brain Freeze (Thassa's Oracle finish): Spellbook 1368-3518-4856 (Bigger)
- Underworld Breach + Lion's Eye Diamond + Brain Freeze (at the table): Spellbook 1368-3518-4856 (Bigger)
- Scion as Prismari + Underworld Breach + Manamorphose + Brain Freeze (at the table): **not in Spellbook's list for this deck**

Almost included (one card missing), winning, 2-3 cards: 27

- Orcish Bowmasters + Peer into the Abyss (1561-3862): missing Peer into the Abyss
- Ancient Copper Dragon + Aggravated Assault (778-3750): missing Aggravated Assault
- Approach of the Second Sun + Mystical Tutor (553-2856): missing Approach of the Second Sun
- Demonic Consultation + Laboratory Maniac (742-2836): missing Laboratory Maniac
- Doomsday + Thassa's Oracle (1295-3189): missing Doomsday
- Enter the Infinite + Thassa's Oracle (953-1295): missing Enter the Infinite
- Hermit Druid + Thassa's Oracle (1295-4400): missing Hermit Druid
- Tainted Pact + Laboratory Maniac (2836-3093): missing Laboratory Maniac
- Approach of the Second Sun + Demonic Tutor (553-4105): missing Approach of the Second Sun
- Demonic Consultation + Jace, Wielder of Mysteries (742-3096): missing Jace, Wielder of Mysteries
- Leveler + Thassa's Oracle (1068-1295): missing Leveler
- Approach of the Second Sun + Vampiric Tutor (553-1827): missing Approach of the Second Sun
- ... and 15 more

### anikthea-enduring-enchantments (t0)

Spellbook finds no combo in this list.

### captain-america-avengers-assemble (t0)

Spellbook finds no combo in this list.

Almost included (one card missing), winning, 2-3 cards: 1

- Ant-Man, Elusive Avenger + Mechanized Production (2047-7748): missing Mechanized Production

### giada-calling-all-angels (t0)

Spellbook finds no combo in this list.

### stella-lee-quick-draw (t0)

Spellbook finds no combo in this list.

Almost included (one card missing), winning, 2-3 cards: 4

- Curiosity + Niv-Mizzet, Parun (1089-2353): missing Curiosity
- Niv-Mizzet, Parun + Niv-Mizzet, Visionary (2353-6092): missing Niv-Mizzet, Visionary
- Niv-Mizzet, Parun + Ophidian Eye (2353-2396): missing Ophidian Eye
- Tandem Lookout + Niv-Mizzet, Parun (406-2353): missing Tandem Lookout

### zurgo-mardu-surge (t0)

Spellbook finds no combo in this list.

Almost included (one card missing), winning, 2-3 cards: 6

- Breath of Fury + Loyal Apprentice (33-3259): missing Breath of Fury
- Breath of Fury + Legion Warboss (1476-3259): missing Breath of Fury
- Blightsteel Colossus + Blade of Selves (1571-3851): missing Blightsteel Colossus
- Éomer, Marshal of Rohan + Blade of Selves (3851-3951): missing Éomer, Marshal of Rohan
- Éomer, Marshal of Rohan + Legion Loyalty (3951-4711): missing Éomer, Marshal of Rohan
- Blightsteel Colossus + Legion Loyalty (1571-4711): missing Blightsteel Colossus

### isshin-mardu-attack-triggers (t1)

Spellbook finds no combo in this list.

Almost included (one card missing), winning, 2-3 cards: 15

- Blasphemous Act + Repercussion (2484-4083): missing Repercussion
- Aurelia, the Warleader + Helm of the Host (1692-4364): missing Helm of the Host
- Toralf, God of Fury // Toralf's Hammer + Blasphemous Act (1775-2484): missing Toralf, God of Fury // Toralf's Hammer
- Aurelia, the Warleader + Sword of Hearth and Home (1550-4364): missing Sword of Hearth and Home
- Breath of Fury + Rising of the Day (1879-3259--95): missing Breath of Fury
- Caesar, Legion's Emperor + Breath of Fury (2801-3259): missing Breath of Fury
- Aurelia, the Warleader + Rionya, Fire Dancer (1908-4364): missing Rionya, Fire Dancer
- Aurelia, the Warleader + Gilraen, Dúnedain Protector (847-4364): missing Gilraen, Dúnedain Protector
- Aurelia, the Warleader + The Neutrinos (4364-7334): missing The Neutrinos
- Breath of Fury + Adeline, Resplendent Cathar + Rising of the Day (178-1879-3259): missing Breath of Fury
- Ingris Stingerquill + Mana Echoes (2440-8159): missing Mana Echoes
- Sandstorm Crasher + Aurelia, the Warleader (4364-6426): missing Sandstorm Crasher
- ... and 3 more

### lathril-golgari-elves (t1)

Spellbook finds no combo in this list.

Almost included (one card missing), winning, 2-3 cards: 1

- Tombstone Stairwell + Poison-Tip Archer (6215-6216): missing Tombstone Stairwell

### light-paws-aura-voltron (t1)

Spellbook finds no combo in this list.

Almost included (one card missing), winning, 2-3 cards: 2

- Light of Promise + Walking Ballista (338-3693): missing Walking Ballista
- Light of Promise + Triskelion (338-1495): missing Triskelion

### tatyova-simic-landfall (t1)

| Spellbook id | cards | combo | wins | results | prerequisites | sim line |
|---|---|---|---|---|---|---|
| 224-1671-3177 | 3 | Retreat to Coralhelm + Simic Growth Chamber + Sakura-Tribe Scout |  |  |  | no |
| 224-433-1671 | 3 | Retreat to Coralhelm + Llanowar Scout + Simic Growth Chamber |  |  |  | no |
| 224-872-1671 | 3 | Retreat to Coralhelm + Scaled Herbalist + Simic Growth Chamber |  |  |  | no |

### teysa-orzhov-aristocrats (t1)

| Spellbook id | cards | combo | wins | results | prerequisites | sim line |
|---|---|---|---|---|---|---|
| 1580-2921-4647-4871 | 4 | Pitiless Plunderer + Reassembling Skeleton + Bartolomé del Presidio + Teysa Karlov |  | Infinite +1/+1 counters on a creature |  | no |
| 1580-2018-2292-4871 | 4 | Luminous Broodmoth + Pitiless Plunderer + Viscera Seer + Reassembling Skeleton |  |  |  | no |
| 1580-4050-4619 | 3 | Teysa, Orzhov Scion + Phyrexian Altar + Reassembling Skeleton |  |  |  | no |
| 1580-2018-2438-4871 | 4 | Luminous Broodmoth + Pitiless Plunderer + Reassembling Skeleton + Carrion Feeder |  | Infinite +1/+1 counters on a creature |  | no |
| 1580-2018-4050 | 3 | Luminous Broodmoth + Phyrexian Altar + Reassembling Skeleton |  |  |  | no |
| 1580-2018-2921-4871 | 4 | Luminous Broodmoth + Pitiless Plunderer + Reassembling Skeleton + Bartolomé del Presidio |  | Infinite +1/+1 counters on a creature |  | no |
| 1580-2018-3967-4871 | 4 | Luminous Broodmoth + Pitiless Plunderer + Reassembling Skeleton + Yahenni, Undying Partisan |  |  |  | no |
| 1580-2921-4871-5111 | 4 | Pitiless Plunderer + Reassembling Skeleton + Bartolomé del Presidio + Drivnod, Carnage Dominus |  | Infinite +1/+1 counters on a creature |  | no |
| 1580-2034-4871 | 3 | Pitiless Plunderer + Ashnod's Altar + Reassembling Skeleton |  | Infinite colorless mana |  | Bigger: Reassembling Skeleton / Gravecrawler + Pitiless Plunderer + an altar + a death payoff |
| 1580-4050-4871 | 3 | Pitiless Plunderer + Reassembling Skeleton + Phyrexian Altar |  |  |  | Bigger: Reassembling Skeleton / Gravecrawler + Pitiless Plunderer + an altar + a death payoff |
| 1580-4050-4314 | 3 | Reassembling Skeleton + Phyrexian Altar + Pawn of Ulamog |  |  |  | Bigger: Phyrexian Altar + Reassembling Skeleton + Pawn of Ulamog / Sifter of Skulls + a death payoff |
| 1580-2292-4871-5111 | 4 | Pitiless Plunderer + Reassembling Skeleton + Viscera Seer + Drivnod, Carnage Dominus |  |  |  | Bigger: Teysa Karlov / Drivnod + Pitiless Plunderer + Reassembling Skeleton + a free outlet + a death payoff |
| 1580-2292-4647-4871 | 4 | Pitiless Plunderer + Reassembling Skeleton + Viscera Seer + Teysa Karlov |  |  |  | Bigger: Teysa Karlov / Drivnod + Pitiless Plunderer + Reassembling Skeleton + a free outlet + a death payoff |
| 1580-2438-4871-5111 | 4 | Pitiless Plunderer + Reassembling Skeleton + Carrion Feeder + Drivnod, Carnage Dominus |  | Infinite +1/+1 counters on a creature |  | Bigger: Teysa Karlov / Drivnod + Pitiless Plunderer + Reassembling Skeleton + a free outlet + a death payoff |
| 1580-2438-4647-4871 | 4 | Pitiless Plunderer + Reassembling Skeleton + Carrion Feeder + Teysa Karlov |  | Infinite +1/+1 counters on a creature |  | Bigger: Teysa Karlov / Drivnod + Pitiless Plunderer + Reassembling Skeleton + a free outlet + a death payoff |
| 1580-2393-4050 | 3 | Reassembling Skeleton + Phyrexian Altar + Sifter of Skulls |  |  |  | Bigger: Phyrexian Altar + Reassembling Skeleton + Pawn of Ulamog / Sifter of Skulls + a death payoff |
| 1580-3967-4647-4871 | 4 | Pitiless Plunderer + Reassembling Skeleton + Yahenni, Undying Partisan + Teysa Karlov |  |  |  | Bigger: Teysa Karlov / Drivnod + Pitiless Plunderer + Reassembling Skeleton + a free outlet + a death payoff |
| 1580-3967-4871-5111 | 4 | Pitiless Plunderer + Reassembling Skeleton + Yahenni, Undying Partisan + Drivnod, Carnage Dominus |  |  |  | Bigger: Teysa Karlov / Drivnod + Pitiless Plunderer + Reassembling Skeleton + a free outlet + a death payoff |

Catalogue lines the deck holds:

- Reassembling Skeleton / Gravecrawler + Pitiless Plunderer + an altar + a death payoff: Spellbook 1580-2034-4871 (Bigger)
- Teysa Karlov / Drivnod + Pitiless Plunderer + Reassembling Skeleton + a free outlet + a death payoff: Spellbook 1580-2292-4871-5111 (Bigger)
- Phyrexian Altar + Reassembling Skeleton + Pawn of Ulamog / Sifter of Skulls + a death payoff: Spellbook 1580-4050-4314 (Bigger)

Almost included (one card missing), winning, 2-3 cards: 11

- Forsaken Miner + Phyrexian Altar + Blood Artist (2842-4050-5544): missing Forsaken Miner
- Nim Deathmantle + Ashnod's Altar + Marionette Apprentice (2034-5003-6199): missing Nim Deathmantle
- Tombstone Stairwell + Blood Artist (2842-6215): missing Tombstone Stairwell
- Mirkwood Bats + Plague of Vermin (2612-5035): missing Plague of Vermin
- Eirdu, Carrier of Dawn // Isilu, Carrier of Twilight + Viscera Seer + Marionette Apprentice (2292-6199-6972): missing Eirdu, Carrier of Dawn // Isilu, Carrier of Twilight
- Eirdu, Carrier of Dawn // Isilu, Carrier of Twilight + Ashnod's Altar + Marionette Apprentice (2034-6199-6972): missing Eirdu, Carrier of Dawn // Isilu, Carrier of Twilight
- Eirdu, Carrier of Dawn // Isilu, Carrier of Twilight + Carrion Feeder + Marionette Apprentice (2438-6199-6972): missing Eirdu, Carrier of Dawn // Isilu, Carrier of Twilight
- Eirdu, Carrier of Dawn // Isilu, Carrier of Twilight + Phyrexian Altar + Marionette Apprentice (4050-6199-6972): missing Eirdu, Carrier of Dawn // Isilu, Carrier of Twilight
- Eirdu, Carrier of Dawn // Isilu, Carrier of Twilight + Bartolomé del Presidio + Marionette Apprentice (2921-6199-6972): missing Eirdu, Carrier of Dawn // Isilu, Carrier of Twilight
- Eirdu, Carrier of Dawn // Isilu, Carrier of Twilight + Yahenni, Undying Partisan + Marionette Apprentice (3967-6199-6972): missing Eirdu, Carrier of Dawn // Isilu, Carrier of Twilight
- Veinwitch Coven + Vindictive Vampire + Phyrexian Altar (1829-1957-4050--204): missing Veinwitch Coven

### brago-azorius-blink-control (t2)

| Spellbook id | cards | combo | wins | results | prerequisites | sim line |
|---|---|---|---|---|---|---|
| 2397-3634-3821 | 3 | Peregrine Drake + Panharmonicon + Venser, Shaper Savant |  | Infinite mana lands you control can produce; Infinite blue mana; Infinite untap of lands you control | You control up to five lands that can collectively tap to produce at least {3}{U}{U}. You have enough mana to pay for commander tax minus {1} if applicable. | no |
| 2751-3557 | 2 | Brago, King Eternal + Strionic Resonator |  | Infinite mana artifacts you control can produce | You control artifacts that can tap to add at least {2}. Brago does not have summoning sickness. Opponents cannot block and kill Brago. | no |
| 3634-3821-6751 | 3 | Peregrine Drake + Venser, Shaper Savant + Starfield Vocalist |  | Infinite mana lands you control can produce; Infinite blue mana; Infinite untap of lands you control | You control up to five lands that can collectively tap to produce at least {3}{U}{U}. You have enough mana to pay for commander tax minus {1} if applicable. | no |
| 1741-1987-2397-2493 | 4 | Cloud of Faeries + Archaeomancer + Ghostly Flicker + Panharmonicon |  | Infinite casts of any instants and/or sorcery cards in your hand or graveyard; Infinite mana lands you control can produce | You control up to two lands that can tap to produce at least {1}{U}. | no |
| 1770-3634-3821 | 3 | Peregrine Drake + Elesh Norn, Mother of Machines + Venser, Shaper Savant |  | Infinite mana lands you control can produce; Infinite blue mana; Infinite untap of lands you control | You control up to five lands that can collectively tap to produce at least {3}{U}{U}. | no |
| 1741-1770-1987-2493 | 4 | Cloud of Faeries + Archaeomancer + Ghostly Flicker + Elesh Norn, Mother of Machines |  | Infinite casts of any instants and/or sorcery cards in your hand or graveyard; Infinite mana lands you control can produce | You control up to two lands that can tap to produce at least {1}{U}. | no |
| 1987-2493-3821 | 3 | Ghostly Flicker + Peregrine Drake + Archaeomancer |  | Infinite mana lands you control can produce; Infinite untap of lands you control | You control up to five lands that can tap to produce at least {2}{U}. | Bigger: Peregrine Drake + Archaeomancer + Ghostly Flicker (+ Agent of Treachery) |

Catalogue lines the deck holds:

- Peregrine Drake + Archaeomancer + Ghostly Flicker (+ Agent of Treachery): Spellbook 1987-2493-3821 (Bigger)
- Displacer Kitten + Archaeomancer + Peregrine Drake or Cloud of Faeries + a flicker spell (+ Agent of Treachery): Spellbook 1987-2493-3821 (Bigger)

Almost included (one card missing), winning, 2-3 cards: 1

- Venser, Shaper Savant + Approach of the Second Sun (553-3634): missing Approach of the Second Sun

### kaalia-mardu-creature-cheat (t2)

| Spellbook id | cards | combo | wins | results | prerequisites | sim line |
|---|---|---|---|---|---|---|
| 29-137 | 2 | Kaalia of the Vast + Master of Cruelties |  | Target opponent loses the game | Kaalia of the Vast does not have summoning sickness. An opponent cannot block and kill Kaalia of the Vast. That opponent cannot block Master of Cruelties. | no |

Almost included (one card missing), winning, 2-3 cards: 31

- Blasphemous Act + Repercussion (2484-4083): missing Repercussion
- Aurelia, the Warleader + Helm of the Host (1692-4364): missing Helm of the Host
- Ancient Copper Dragon + Aggravated Assault (778-3750): missing Aggravated Assault
- Dragon Tempest + Ancient Gold Dragon (2855-5982): missing Ancient Gold Dragon
- Toralf, God of Fury // Toralf's Hammer + Blasphemous Act (1775-2484): missing Toralf, God of Fury // Toralf's Hammer
- Orthion, Hero of Lavabrink + Terror of the Peaks (1110-2136): missing Orthion, Hero of Lavabrink
- Aurelia, the Warleader + Sword of Hearth and Home (1550-4364): missing Sword of Hearth and Home
- Mycosynth Lattice + Hellkite Tyrant (523-3263): missing Mycosynth Lattice
- Heartless Hidetsugu + Twinflame Tyrant (3011-6093): missing Heartless Hidetsugu
- Approach of the Second Sun + Demonic Tutor (553-4105): missing Approach of the Second Sun
- Rionya, Fire Dancer + Terror of the Peaks (1110-1908): missing Rionya, Fire Dancer
- Devastating Onslaught + Terror of the Peaks (1110-6785): missing Devastating Onslaught
- ... and 19 more

### lord-windgrace-jund-lands (t2)

Spellbook finds no combo in this list.

Catalogue lines the deck holds:

- Dark Depths + Thespian's Stage: **not in Spellbook's list for this deck**

Almost included (one card missing), winning, 2-3 cards: 8

- Blasphemous Act + Repercussion (2484-4083): missing Repercussion
- Toralf, God of Fury // Toralf's Hammer + Blasphemous Act (1775-2484): missing Toralf, God of Fury // Toralf's Hammer
- Maze's End + Scapeshift (3063-3261): missing Maze's End
- Perilous Forays + Omnath, Locus of Rage + Lotus Cobra (880-3470-4862): missing Perilous Forays
- Perilous Forays + Omnath, Locus of Rage + Tireless Provisioner (880-3875-4862): missing Perilous Forays
- Perilous Forays + Omnath, Locus of Rage + Spelunking (880-4862-5478): missing Perilous Forays
- Yavimaya Bloomsage // Channel + Torment of Hailfire (7439-7442): missing Yavimaya Bloomsage // Channel
- Anzrag, the Quake-Mole (5881--101): missing Anzrag, the Quake-Mole

### meren-golgari-recursion (t2)

| Spellbook id | cards | combo | wins | results | prerequisites | sim line |
|---|---|---|---|---|---|---|
| 328-628-4279 | 3 | Mikaeus, the Unhallowed + Yawgmoth, Thran Physician + Gray Merchant of Asphodel | **win** | Infinite card draw; Near-infinite lifegain; Near-infinite lifegain triggers; Near-infinite lifeloss | Your life total is at least 2. You control at least one additional nontoken non-Human creature with toughness two or greater. | no |

Catalogue lines the deck holds:

- Yawgmoth + undying loop + a death payoff: **not in Spellbook's list for this deck**

Almost included (one card missing), winning, 2-3 cards: 34

- Warren Soultrader + Gravecrawler + Blood Artist (2577-2842-5670): missing Gravecrawler
- Warren Soultrader + Gravecrawler + Zulaport Cutthroat (1385-2577-5670): missing Gravecrawler
- Warren Soultrader + Forsaken Miner + Blood Artist (2842-5544-5670): missing Forsaken Miner
- Warren Soultrader + Zulaport Cutthroat + Chatterfang, Squirrel General (1385-3000-5670): missing Chatterfang, Squirrel General
- Warren Soultrader + Blood Artist + Chatterfang, Squirrel General (2842-3000-5670): missing Chatterfang, Squirrel General
- Mikaeus, the Unhallowed + Warren Soultrader + Blood Artist (628-2842-5670--5): missing 
- Yawgmoth, Thran Physician + Nest of Scarabs + Blood Artist (2760-2842-4279): missing Nest of Scarabs
- Yawgmoth, Thran Physician + Zulaport Cutthroat + Nest of Scarabs (1385-2760-4279): missing Nest of Scarabs
- Yawgmoth, Thran Physician + Mikaeus, the Unhallowed + Ayara, First of Locthwain (628-884-4279): missing Ayara, First of Locthwain
- Mikaeus, the Unhallowed + Triskelion (628-1495): missing Triskelion
- Tombstone Stairwell + Blood Artist (2842-6215): missing Tombstone Stairwell
- Warren Soultrader + Chatterfang, Squirrel General + Cauldron of Essence (3000-5670-7452): missing Chatterfang, Squirrel General
- ... and 22 more

### sythis-selesnya-enchantress (t2)

Spellbook finds no combo in this list.

Almost included (one card missing), winning, 2-3 cards: 3

- Heliod, Sun-Crowned + Walking Ballista (1274-3693): missing Walking Ballista
- Heliod, Sun-Crowned + Triskelion (1274-1495): missing Triskelion
- Biorhythm + Austere Command (3778-7299): missing Biorhythm

### atraxa-infect (t3)

| Spellbook id | cards | combo | wins | results | prerequisites | sim line |
|---|---|---|---|---|---|---|
| 1529-1887 | 2 | Vraska, Betrayal's Sting + Vorinclex, Monstrous Raider |  | Target opponent loses the game | An opponent has eight or fewer poison counters. | no |
| 1529-5902 | 2 | Vraska, Betrayal's Sting + Innkeeper's Talent |  | Target opponent loses the game | Innkeeper's Talent is level 3. An opponent has eight or fewer poison counters. | no |

Almost included (one card missing), winning, 2-3 cards: 1

- Prologue to Phyresis + Expansion Algorithm (1131-7873): missing Expansion Algorithm

### aurelia-boros-extra-combats (t3)

| Spellbook id | cards | combo | wins | results | prerequisites | sim line |
|---|---|---|---|---|---|---|
| 900-3750 | 2 | Sword of Feast and Famine + Aggravated Assault | **win** | Infinite combat phases | Sword of Feast and Famine attached to a creature that can attack this turn. An opponent is unable to block and kill the creature with Sword of Feast and Famine attached. Lands you control can tap to produce at least {3}{R}{R}. | no |
| 1692-4364 | 2 | Aurelia, the Warleader + Helm of the Host | **win** | Infinite combat phases; Infinite creature tokens with haste; Infinite mana creatures you control can produce | Helm attached to Aurelia. | no |
| 1550-4364 | 2 | Aurelia, the Warleader + Sword of Hearth and Home | **win** | Infinite combat damage; Infinite combat phases | Sword of Hearth and Home attached to any other creature you control. The creature with Sword of Hearth and Home attached can attack this turn. An opponent cannot block and kill the creature with Sword of Hearth and Home attached and Aurelia. | no |
| 1692-2105 | 2 | Combat Celebrant + Helm of the Host | **win** | Infinite combat phases; Infinite mana creatures you control can produce; Infinite tapped creature tokens | Helm of the Host attached to Combat Celebrant. | Exact: Helm of the Host + Combat Celebrant |

Catalogue lines the deck holds:

- Helm of the Host + Combat Celebrant: Spellbook 1692-2105 (Exact)

Almost included (one card missing), winning, 2-3 cards: 57

- The Reaver Cleaver + Aggravated Assault (3750-5039): missing The Reaver Cleaver
- Blasphemous Act + Repercussion (2484-4083): missing Repercussion
- Ancient Copper Dragon + Aggravated Assault (778-3750): missing Ancient Copper Dragon
- Godo, Bandit Warlord + Helm of the Host (1692-4251): missing Godo, Bandit Warlord
- Neheb, the Eternal + Aggravated Assault (3750-4945): missing Neheb, the Eternal
- Toralf, God of Fury // Toralf's Hammer + Blasphemous Act (1775-2484): missing Toralf, God of Fury // Toralf's Hammer
- Combat Celebrant + Kiki-Jiki, Mirror Breaker (618-2105): missing Kiki-Jiki, Mirror Breaker
- Rionya, Fire Dancer + Combat Celebrant (1908-2105): missing Rionya, Fire Dancer
- Cavern-Hoard Dragon + Aggravated Assault (3750-5988): missing Cavern-Hoard Dragon
- Magda, Brazen Outlaw + Aggravated Assault (1486-3750): missing Magda, Brazen Outlaw
- Port Razer + Helm of the Host (1548-1692): missing Port Razer
- Scourge of the Throne + Helm of the Host (1405-1692): missing Scourge of the Throne
- ... and 45 more

### korvold-jund-sacrifice (t3)

| Spellbook id | cards | combo | wins | results | prerequisites | sim line |
|---|---|---|---|---|---|---|
| 3000-4231-5670 | 3 | Chatterfang, Squirrel General + Warren Soultrader + Academy Manufactor |  | Infinite artifact tokens; Infinite colored mana; Infinite lifegain triggers; Infinite lifegain; Infinite card draw; Infinite Treasure tokens; Infinite creature tokens | You control at least one additional creature. Your life total is at least 3. | no |
| 2429-3340 | 2 | Bootleggers' Stash + Revel in Riches | **win** | Win the game at the beginning of your next upkeep | You control at least ten in any combination of lands and/or Treasure tokens. | no |
| 1800-2429 | 2 | Old Gnawbone + Revel in Riches | **win** | Win the game at the beginning of your next upkeep | You control creatures with total power ten or greater that can attack this turn and that cannot be blocked and killed by an opponent. | card code: common.rs: Revel in Riches wins at ten Treasures; Old Gnawbone makes them (t3.rs) |
| 865-2034-2429 | 3 | Mahadi, Emporium Master + Revel in Riches + Ashnod's Altar | **win** | Win the game at the beginning of your next upkeep | You control at least ten in any combination of Treasures and/or creatures. | card code: common.rs: Revel in Riches wins at ten Treasures; Mahadi makes them (t4.rs) |
| 865-2292-2429 | 3 | Mahadi, Emporium Master + Revel in Riches + Viscera Seer | **win** | Win the game at the beginning of your next upkeep | You control at least ten in any combination of Treasures and/or creatures. | card code: common.rs: Revel in Riches wins at ten Treasures; Mahadi makes them (t4.rs) |
| 865-2429-4050 | 3 | Mahadi, Emporium Master + Revel in Riches + Phyrexian Altar | **win** | Win the game at the beginning of your next upkeep | You control at least ten in any combination of Treasures and/or creatures. | card code: common.rs: Revel in Riches wins at ten Treasures; Mahadi makes them (t4.rs) |
| 1580-2034-4871 | 3 | Pitiless Plunderer + Ashnod's Altar + Reassembling Skeleton |  | Infinite colorless mana |  | Bigger: Reassembling Skeleton / Gravecrawler + Pitiless Plunderer + an altar + a death payoff |
| 3000-4871 | 2 | Chatterfang, Squirrel General + Pitiless Plunderer |  | Infinite artifact tokens; Infinite colored mana; Infinite Treasure tokens | You control an additional Squirrel. | Bigger: Chatterfang + Pitiless Plunderer + a sacrifice outlet + a death payoff |
| 1580-4050-4871 | 3 | Pitiless Plunderer + Reassembling Skeleton + Phyrexian Altar |  |  |  | Bigger: Reassembling Skeleton / Gravecrawler + Pitiless Plunderer + an altar + a death payoff |
| 1385-3000-5670 | 3 | Warren Soultrader + Zulaport Cutthroat + Chatterfang, Squirrel General | **win** | Infinite artifact tokens; Infinite colored mana; Infinite lifegain triggers; Infinite lifeloss; Infinite Treasure tokens | You control at least one additional creature. Your life total is at least 2. | Bigger: Chatterfang + Pitiless Plunderer + a sacrifice outlet + a death payoff |
| 2842-3000-5670 | 3 | Warren Soultrader + Blood Artist + Chatterfang, Squirrel General | **win** | Infinite artifact tokens; Infinite colored mana; Infinite lifegain triggers; Infinite lifeloss; Infinite Treasure tokens | You control at least one additional creature. Your life total is at least 2. | Bigger: Chatterfang + Pitiless Plunderer + a sacrifice outlet + a death payoff |
| 1580-2842-4871-5670 | 4 | Warren Soultrader + Pitiless Plunderer + Reassembling Skeleton + Blood Artist | **win** | Infinite lifeloss | Your life total is 2 or higher. | Exact: Reassembling Skeleton / Gravecrawler + Pitiless Plunderer + an altar + a death payoff |
| 1385-1580-4871-5670 | 4 | Warren Soultrader + Pitiless Plunderer + Zulaport Cutthroat + Reassembling Skeleton | **win** | Infinite lifeloss | Your life total is 2 or higher. | Exact: Reassembling Skeleton / Gravecrawler + Pitiless Plunderer + an altar + a death payoff |

Catalogue lines the deck holds:

- Reassembling Skeleton / Gravecrawler + Pitiless Plunderer + an altar + a death payoff: Spellbook 1580-2842-4871-5670 (Exact)
- Chatterfang + Pitiless Plunderer + a sacrifice outlet + a death payoff: Spellbook 3000-4871 (Bigger)

Almost included (one card missing), winning, 2-3 cards: 62

- Warren Soultrader + Gravecrawler + Blood Artist (2577-2842-5670): missing Gravecrawler
- The Reaver Cleaver + Aggravated Assault (3750-5039): missing Aggravated Assault
- Warren Soultrader + Gravecrawler + Zulaport Cutthroat (1385-2577-5670): missing Gravecrawler
- Blasphemous Act + Repercussion (2484-4083): missing Repercussion
- Warren Soultrader + Forsaken Miner + Blood Artist (2842-5544-5670): missing Forsaken Miner
- Aggravated Assault + Old Gnawbone (1800-3750): missing Aggravated Assault
- Toralf, God of Fury // Toralf's Hammer + Blasphemous Act (1775-2484): missing Toralf, God of Fury // Toralf's Hammer
- Forsaken Miner + Phyrexian Altar + Blood Artist (2842-4050-5544): missing Forsaken Miner
- Old Gnawbone + Hellkite Charger (1800-3398): missing Hellkite Charger
- Goblin Bombardment + Rhythm of the Wild (2376-5147--5): missing 
- Warren Soultrader + Chatterfang, Squirrel General + Bastion of Remembrance (3000-3025-5670): missing Bastion of Remembrance
- Brass's Bounty + Revel in Riches (2429-5211): missing Brass's Bounty
- ... and 50 more

### marwyn-mono-green-elves (t3)

| Spellbook id | cards | combo | wins | results | prerequisites | sim line |
|---|---|---|---|---|---|---|
| 1902-2645 | 2 | Staff of Domination + Wirewood Channeler |  | Infinite card draw; Infinite colored mana; Infinite lifegain; Infinite lifegain triggers; Infinite mana creatures you control can produce | Wirewood Channeler does not have summoning sickness. There are at least five Elves on the battlefield. | no |
| 1902-2816 | 2 | Umbral Mantle + Wirewood Channeler |  | Infinite colored mana; Infinitely large creature until end of turn; Infinite mana creatures you control can produce | Umbral Mantle attached to Wirewood Channeler. Wirewood Channeler does not have summoning sickness. There are at least three Elves on the battlefield. | no |
| 498-2050-2645 | 3 | Elvish Guidance + Arbor Elf + Staff of Domination |  | Infinite card draw; Infinite green mana; Infinite lifegain; Infinite lifegain triggers; Infinite mana creatures you control can produce | Elvish Guidance attached to a Forest you control. Arbor Elf does not have summoning sickness. You control at least four Elves. | no |
| 498-2050-2816 | 3 | Elvish Guidance + Arbor Elf + Umbral Mantle |  | Infinite green mana; Infinitely large creature until end of turn; Infinite mana creatures you control can produce | Umbral Mantle attached to Arbor Elf. Elvish Guidance attached to a Forest you control. Arbor Elf does not have summoning sickness. You control at least three Elves. | no |
| 1376-2645 | 2 | Staff of Domination + Priest of Titania |  | Infinite card draw; Infinite lifegain; Infinite lifegain triggers; Infinite mana creatures you control can produce; Infinite green mana | Priest of Titania does not have summoning sickness. There are at least five Elves on the battlefield. | Bigger: Staff of Domination + an Elf that taps for five + Ezuri / Finale / Craterhoof |
| 2645-3151 | 2 | Staff of Domination + Elvish Archdruid |  | Infinite card draw; Infinite lifegain; Infinite lifegain triggers; Infinite mana creatures you control can produce; Infinite green mana | Elvish Archdruid does not have summoning sickness. You control at least five Elves. | Bigger: Staff of Domination + an Elf that taps for five + Ezuri / Finale / Craterhoof |
| 2436-2645 | 2 | Staff of Domination + Marwyn, the Nurturer |  | Infinite card draw; Infinite lifegain; Infinite lifegain triggers; Infinite mana creatures you control can produce; Infinite green mana | Marwyn does not have summoning sickness, and has power 5 or greater. | Bigger: Staff of Domination + an Elf that taps for five + Ezuri / Finale / Craterhoof |
| 1526-2645 | 2 | Staff of Domination + Circle of Dreams Druid |  | Infinite card draw; Infinite lifegain; Infinite lifegain triggers; Infinite mana creatures you control can produce; Infinite green mana | Circle of Dreams Druid does not have summoning sickness. You control at least five creatures. | Bigger: Staff of Domination + an Elf that taps for five + Ezuri / Finale / Craterhoof |
| 1376-2816 | 2 | Umbral Mantle + Priest of Titania |  | Infinite green mana; Infinitely large creature until end of turn; Infinite mana creatures you control can produce | Umbral Mantle attached to Priest of Titania. Priest of Titania does not have summoning sickness. There are at least three Elves on the battlefield. | Bigger: Umbral Mantle + a big mana Elf + Ezuri |
| 2816-3151 | 2 | Umbral Mantle + Elvish Archdruid |  | Infinite green mana; Infinitely large creature until end of turn; Infinite mana creatures you control can produce | Umbral Mantle attached to Elvish Archdruid. Elvish Archdruid does not have summoning sickness. You control at least three Elves. | Bigger: Umbral Mantle + a big mana Elf + Ezuri |
| 2436-2816 | 2 | Umbral Mantle + Marwyn, the Nurturer |  | Infinite green mana; Infinitely large creature until end of turn; Infinite mana creatures you control can produce | Umbral Mantle attached to Marwyn. Marwyn does not have summoning sickness. | Bigger: Umbral Mantle + a big mana Elf + Ezuri |
| 1526-2816 | 2 | Umbral Mantle + Circle of Dreams Druid |  | Infinite green mana; Infinitely large creature until end of turn; Infinite mana creatures you control can produce | Umbral Mantle attached to Circle of Dreams Druid. Circle of Dreams Druid does not have summoning sickness. You control at least three creatures. | Bigger: Umbral Mantle + a big mana Elf + Ezuri |

Catalogue lines the deck holds:

- Umbral Mantle + a big mana Elf + Ezuri: Spellbook 1376-2816 (Bigger)
- Staff of Domination + an Elf that taps for five + Ezuri / Finale / Craterhoof: Spellbook 1376-2645 (Bigger)

### tergrid-mono-black-disruption (t3)

Spellbook finds no combo in this list.

### chulane-bant-value-combo (t4)

| Spellbook id | cards | combo | wins | results | prerequisites | sim line |
|---|---|---|---|---|---|---|
| 751-3314-4093 | 3 | Chulane, Teller of Tales + Mana Breach + Shrieking Drake |  | Infinite card draw | Land you control that can produce {U}. | no |
| 751-1671-3314-3470 | 4 | Chulane, Teller of Tales + Shrieking Drake + Lotus Cobra + Simic Growth Chamber |  | Infinite card draw |  | no |
| 751-907-3314-3470 | 4 | Chulane, Teller of Tales + Shrieking Drake + Lotus Cobra + Selesnya Sanctuary |  | Infinite card draw |  | no |
| 751-1671-3314-3875 | 4 | Chulane, Teller of Tales + Shrieking Drake + Tireless Provisioner + Simic Growth Chamber |  | Infinite card draw |  | no |
| 751-3147-3314-3470 | 4 | Chulane, Teller of Tales + Shrieking Drake + Lotus Cobra + Azorius Chancery |  | Infinite card draw |  | no |
| 751-907-3314-3875 | 4 | Chulane, Teller of Tales + Shrieking Drake + Tireless Provisioner + Selesnya Sanctuary |  | Infinite card draw |  | no |
| 751-3147-3314-3875 | 4 | Chulane, Teller of Tales + Shrieking Drake + Tireless Provisioner + Azorius Chancery |  | Infinite card draw |  | no |
| 1636-3314 | 2 | Chulane, Teller of Tales + Intruder Alarm |  | Infinite card draw; Near-infinite mana creatures you control can produce; Near-infinite untap of all creatures; Put all lands from your hand and library onto the battlefield | Chulane does not have summoning sickness. You have at least one creature card in hand. Creatures you control can tap to produce at least {3} plus enough mana to cast the creature card. | Bigger: Intruder Alarm + Shrieking Drake / Whitemane Lion + a mana creature + Chulane |
| 751-1636 | 2 | Intruder Alarm + Shrieking Drake |  | Infinite mana creatures you control can produce; Infinite untap of creatures | Creatures you control can tap to produce at least {U}. | Bigger: Intruder Alarm + Shrieking Drake / Whitemane Lion + a mana creature + Chulane |
| 2232-4191 | 2 | Aluren + Cloudstone Curio |  |  | You control a nontoken nonartifact creature with mana value 3 or less. You have a nonartifact creature card with mana value 3 or less in your hand. | Bigger: Aluren + Cloudstone Curio + Chulane + two cheap creatures |
| 1636-4770 | 2 | Intruder Alarm + Whitemane Lion |  | Infinite mana creatures you control can produce; Infinite untap of creatures | Creatures you control can tap to produce at least {1}{W}. | Bigger: Intruder Alarm + Shrieking Drake / Whitemane Lion + a mana creature + Chulane |

Catalogue lines the deck holds:

- Aluren + Chulane + a bouncing creature: **not in Spellbook's list for this deck**
- Intruder Alarm + Shrieking Drake / Whitemane Lion + a mana creature + Chulane: Spellbook 1636-3314 (Bigger)
- Aluren + Cloudstone Curio + Chulane + two cheap creatures: Spellbook 2232-4191 (Bigger)

Almost included (one card missing), winning, 2-3 cards: 18

- Enter the Infinite + Thassa's Oracle (953-1295): missing Enter the Infinite
- Hermit Druid + Thassa's Oracle (1295-4400): missing Hermit Druid
- Leveler + Thassa's Oracle (1068-1295): missing Leveler
- Leveler + Laboratory Maniac (1068-2836): missing Leveler
- Hermit Druid + Laboratory Maniac (2836-4400): missing Hermit Druid
- Thassa's Oracle + Thought Lash (1295-2853): missing Thought Lash
- Laboratory Maniac + Thought Lash (2836-2853): missing Thought Lash
- Paradigm Shift + Thassa's Oracle (1295-3367): missing Paradigm Shift
- Paradigm Shift + Laboratory Maniac (2836-3367): missing Paradigm Shift
- Mirror of Fate + Thassa's Oracle (1164-1295): missing Mirror of Fate
- Chulane, Teller of Tales + Village Bell-Ringer + Laboratory Maniac (2059-2836-3314): missing Village Bell-Ringer
- Phyrexian Devourer + Thassa's Oracle (1295-4899): missing Phyrexian Devourer
- ... and 6 more

### edgar-markov-mardu-vampires (t4)

| Spellbook id | cards | combo | wins | results | prerequisites | sim line |
|---|---|---|---|---|---|---|
| 690-3966 | 2 | Sanguine Bond + Exquisite Blood | **win** | Infinite lifegain triggers; Infinite lifeloss; Infinite lifegain | You have a way to gain life. | Exact: Sanguine Bond + Exquisite Blood |
| 314-3966 | 2 | Vito, Thorn of the Dusk Rose + Exquisite Blood | **win** | Infinite lifegain triggers; Infinite lifeloss; Infinite lifegain | You have a way to gain life. | Exact: Vito / Sanguine Bond / Marauding Blight-Priest + Exquisite Blood / Bloodthirsty Conqueror |
| 314-6191 | 2 | Vito, Thorn of the Dusk Rose + Bloodthirsty Conqueror | **win** | Infinite lifegain triggers; Infinite lifeloss; Infinite lifegain | You have a way to gain life. | Exact: Vito / Sanguine Bond / Marauding Blight-Priest + Exquisite Blood / Bloodthirsty Conqueror |
| 690-6191 | 2 | Sanguine Bond + Bloodthirsty Conqueror | **win** | Infinite lifegain triggers; Infinite lifeloss; Infinite lifegain | You have a way to gain life. | Exact: Vito / Sanguine Bond / Marauding Blight-Priest + Exquisite Blood / Bloodthirsty Conqueror |
| 110-3966 | 2 | Marauding Blight-Priest + Exquisite Blood | **win** | Infinite lifegain triggers; Infinite lifeloss; Infinite lifegain | You have a way to gain life. | Exact: Vito / Sanguine Bond / Marauding Blight-Priest + Exquisite Blood / Bloodthirsty Conqueror |
| 110-6191 | 2 | Marauding Blight-Priest + Bloodthirsty Conqueror | **win** | Infinite lifegain triggers; Infinite lifeloss; Infinite lifegain | You have a way to gain life. | Exact: Vito / Sanguine Bond / Marauding Blight-Priest + Exquisite Blood / Bloodthirsty Conqueror |

Catalogue lines the deck holds:

- Sanguine Bond + Exquisite Blood: Spellbook 690-3966 (Exact)
- Vito / Sanguine Bond / Marauding Blight-Priest + Exquisite Blood / Bloodthirsty Conqueror: Spellbook 690-3966 (Exact)

Almost included (one card missing), winning, 2-3 cards: 35

- Exquisite Blood + Enduring Tenacity (3966-5755): missing Enduring Tenacity
- Enduring Tenacity + Bloodthirsty Conqueror (5755-6191): missing Enduring Tenacity
- Aetherflux Reservoir + Exquisite Blood (3966-4740): missing Aetherflux Reservoir
- Aetherflux Reservoir + Bloodthirsty Conqueror (4740-6191): missing Aetherflux Reservoir
- Exquisite Blood + Starscape Cleric (3966-5867): missing Starscape Cleric
- Starscape Cleric + Bloodthirsty Conqueror (5867-6191): missing Starscape Cleric
- Forsaken Miner + Phyrexian Altar + Blood Artist (2842-4050-5544): missing Forsaken Miner
- Vito, Thorn of the Dusk Rose + Beacon of Immortality (314-4996): missing Beacon of Immortality
- Exquisite Blood + Cliffhaven Vampire (3966-4789): missing Cliffhaven Vampire
- Vito, Thorn of the Dusk Rose + Revival // Revenge (314-4624): missing Revival // Revenge
- Sanguine Bond + Revival // Revenge (690-4624): missing Revival // Revenge
- Vizkopa Guildmage + Exquisite Blood (92-3966): missing Vizkopa Guildmage
- ... and 23 more

### krenko-mono-red-goblins (t4)

| Spellbook id | cards | combo | wins | results | prerequisites | sim line |
|---|---|---|---|---|---|---|
| 38-659-1288 | 3 | Krenko, Mob Boss + Skirk Prospector + Goblin Warchief |  | Infinite commander casts; Infinite creature tokens with haste; Infinite red mana | You control a number of Goblins that is equal to or greater than six plus Krenko's commander tax. | no |
| 38-659-3954 | 3 | Krenko, Mob Boss + Skirk Prospector + Goblin Chieftain |  | Infinite commander casts; Infinite creature tokens with haste; Infinite red mana | You control a number of Goblins that is equal to or greater than six plus Krenko's commander tax. | no |
| 618-2240-2943 | 3 | Conspicuous Snoop + Kiki-Jiki, Mirror Breaker + Goblin Recruiter |  | Infinite creature tokens with haste | Conspicuous Snoop does not have summoning sickness. | no |
| 38-659-6410 | 3 | Krenko, Mob Boss + Skirk Prospector + Howlsquad Heavy |  | Infinite red mana; Infinite creature tokens with haste; Infinite commander casts | You control a number of Goblins that is equal to or greater than six plus Krenko's commander tax. | no |
| 38-659-5295 | 3 | Krenko, Mob Boss + Skirk Prospector + Thousand-Year Elixir |  | Infinite red mana; Infinite creature tokens with haste; Infinite commander casts | You control a number of Goblins that is equal to or greater than six plus Krenko's commander tax. | no |
| 38-659-2645 | 3 | Skirk Prospector + Krenko, Mob Boss + Staff of Domination |  | Infinite creature tokens; Infinite lifegain triggers; Infinite lifegain; Infinite card draw; Infinite red mana; Infinite mana creatures you control can produce | Krenko does not have summoning sickness. You control at least X Goblins. | no |
| 659-2178-2813 | 3 | Krenko, Mob Boss + Thornbite Staff + Goblin Trashmaster |  | Infinite creature tokens | Thornbite Staff attached to Krenko. Krenko does not have summoning sickness. You control at least one additional Goblin. | no |
| 38-659-2816 | 3 | Skirk Prospector + Krenko, Mob Boss + Umbral Mantle |  | Infinite creature tokens; Infinite red mana; Infinite mana creatures you control can produce; Infinitely large creature until end of turn | Umbral Mantle attached to Krenko. Krenko does not have summoning sickness. You control at least three Goblins. | no |
| 618-2178-5147 | 3 | Kiki-Jiki, Mirror Breaker + Thornbite Staff + Goblin Bombardment | **win** | Infinite damage | Thornbite Staff attached to Kiki-Jiki. You control at least one nonlegendary creature. | no |
| 38-618-2178 | 3 | Kiki-Jiki, Mirror Breaker + Thornbite Staff + Skirk Prospector |  | Infinite red mana | Thornbite Staff attached to Kiki-Jiki. | no |
| 38-659-969 | 3 | Krenko, Mob Boss + Skirk Prospector + Fervor |  | Infinite red mana; Infinite creature tokens with haste; Infinite commander casts | You control a number of Goblins that is equal to or greater than six plus Krenko's commander tax. | no |
| 38-2440-2967 | 3 | Skirk Prospector + Pashalik Mons + Mana Echoes | **win** | Infinite colorless mana; Infinite creature tokens; Infinite damage; Infinite red mana | You control an additional Goblin. | no |
| 2645-6410 | 2 | Staff of Domination + Howlsquad Heavy |  | Infinite lifegain triggers; Infinite lifegain; Infinite red mana; Infinite card draw; Infinite mana creatures you control can produce | Howlsquad Heavy does not have summoning sickness. You have max speed. You control at least five Goblins. | no |
| 618-2034-2178 | 3 | Kiki-Jiki, Mirror Breaker + Thornbite Staff + Ashnod's Altar |  | Infinite colorless mana | Thornbite Staff attached to Kiki-Jiki. You control at least one nonlegendary creature. | no |
| 618-1228-2178 | 3 | Kiki-Jiki, Mirror Breaker + Thornbite Staff + Goblin Sharpshooter | **win** | Infinite damage | Thornbite Staff attached to Kiki-Jiki. Goblin Sharpshooter does not have summoning sickness. | no |
| 2816-6410 | 2 | Umbral Mantle + Howlsquad Heavy |  | Infinite red mana; Infinitely large creature until end of turn; Infinite mana creatures you control can produce | Umbral Mantle attached to Howlsquad Heavy. You have max speed. You control at least three Goblins. | no |
| 38-659-2178 | 3 | Krenko, Mob Boss + Thornbite Staff + Skirk Prospector | **win** | Infinite creature tokens; Infinite damage; Infinite red mana | Thornbite Staff attached to Krenko. Krenko does not have summoning sickness. | Exact: Krenko + Thornbite Staff + a sacrifice outlet |
| 659-2178-5147 | 3 | Krenko, Mob Boss + Thornbite Staff + Goblin Bombardment | **win** | Infinite creature tokens; Infinite damage | Thornbite Staff attached to Krenko. Krenko does not have summoning sickness. You control at least one additional Goblin. | Exact: Krenko + Thornbite Staff + a sacrifice outlet |
| 659-2034-2178 | 3 | Krenko, Mob Boss + Thornbite Staff + Ashnod's Altar |  | Infinite colorless mana; Infinite creature tokens | Thornbite Staff attached to Krenko. Krenko does not have summoning sickness. You control at least one additional Goblin. | Exact: Krenko + Thornbite Staff + a sacrifice outlet |
| 659-2034-2645 | 3 | Krenko, Mob Boss + Ashnod's Altar + Staff of Domination |  | Infinite card draw; Infinite colorless mana; Infinite creature tokens; Infinite lifegain; Infinite lifegain triggers; Infinite mana creatures you control can produce | Krenko, Mob Boss does not have summoning sickness. You control at least two Goblins. | Exact: Krenko + Umbral Mantle / Staff of Domination + Ashnod's Altar / Mana Echoes |
| 659-2034-2816 | 3 | Krenko, Mob Boss + Umbral Mantle + Ashnod's Altar |  | Infinite colorless mana; Infinite creature tokens; Infinite mana creatures you control can produce; Infinitely large creature until end of turn | Krenko, Mob Boss does not have summoning sickness. Umbral Mantle attached to Krenko. You control at least two Goblins. | Exact: Krenko + Umbral Mantle / Staff of Domination + Ashnod's Altar / Mana Echoes |

Catalogue lines the deck holds:

- Krenko + Thornbite Staff + a sacrifice outlet: Spellbook 38-659-2178 (Exact)
- Krenko + Umbral Mantle / Staff of Domination + Ashnod's Altar / Mana Echoes: Spellbook 659-2034-2645 (Exact)

Almost included (one card missing), winning, 2-3 cards: 38

- Blasphemous Act + Repercussion (2484-4083): missing Repercussion
- Toralf, God of Fury // Toralf's Hammer + Blasphemous Act (1775-2484): missing Toralf, God of Fury // Toralf's Hammer
- Combat Celebrant + Kiki-Jiki, Mirror Breaker (618-2105): missing Combat Celebrant
- Skirk Prospector + Krenko, Mob Boss + Aggravated Assault (38-659-3750): missing Aggravated Assault
- Kiki-Jiki, Mirror Breaker + Port Razer (618-1548): missing Port Razer
- Kiki-Jiki, Mirror Breaker + Fear of Missing Out (618-6188): missing Fear of Missing Out
- Kiki-Jiki, Mirror Breaker + Lightning Crafter + Skirk Prospector (38-618-3580): missing Lightning Crafter
- Breath of Fury + Legion Warboss (1476-3259): missing Breath of Fury
- Kiki-Jiki, Mirror Breaker + Lightning Crafter + Goblin Bombardment (618-3580-5147): missing Lightning Crafter
- Howlsquad Heavy + Aggravated Assault (3750-6410): missing Aggravated Assault
- Kiki-Jiki, Mirror Breaker + Breath of Fury (618-3259): missing Breath of Fury
- Kiki-Jiki, Mirror Breaker + Thornbite Staff + Mogg Fanatic (618-821-2178): missing Mogg Fanatic
- ... and 26 more

### prosper-rakdos-exile-treasure (t4)

| Spellbook id | cards | combo | wins | results | prerequisites | sim line |
|---|---|---|---|---|---|---|
| 392-967-5078 | 3 | Birgi, God of Storytelling // Harnfel, Horn of Bounty + Sensei's Divining Top + Prosper, Tome-Bound |  | Near-infinite looting |  | no |
| 967-2257-4166-5078 | 4 | Ob Nixilis, Captive Kingpin + Prosper, Tome-Bound + Sensei's Divining Top + Reckless Fireweaver | **win** | Near-infinite +1/+1 counters on a creature; Near-infinite damage; Infinite card draw |  | no |
| 967-2815-5078-5184 | 4 | Prosper, Tome-Bound + Professional Face-Breaker + Sensei's Divining Top + Xorn |  | Infinite card draw | You control a Treasure token. | no |
| 967-1678-4166-5078 | 4 | Ob Nixilis, Captive Kingpin + Prosper, Tome-Bound + Sensei's Divining Top + Mayhem Devil | **win** | Infinite card draw; Near-infinite +1/+1 counters on a creature; Near-infinite damage | You control at least one Treasure token. | no |
| 967-4166-5078-6847 | 4 | Ob Nixilis, Captive Kingpin + Prosper, Tome-Bound + Sensei's Divining Top + Weftstalker Ardent | **win** | Near-infinite damage; Infinite card draw; Near-infinite +1/+1 counters on a creature |  | no |
| 2173-4740-5078 | 3 | Sensei's Divining Top + Aetherflux Reservoir + Bolas's Citadel | **win** | Infinite card draw; Near-infinite damage; Near-infinite lifegain; Near-infinite lifegain triggers | Your life total is at least 2. | Exact: Bolas's Citadel + Sensei's Divining Top + Aetherflux Reservoir |

Catalogue lines the deck holds:

- Bolas's Citadel + Sensei's Divining Top + Aetherflux Reservoir: Spellbook 2173-4740-5078 (Exact)

Almost included (one card missing), winning, 2-3 cards: 26

- Aetherflux Reservoir + Exquisite Blood (3966-4740): missing Exquisite Blood
- Blasphemous Act + Repercussion (2484-4083): missing Repercussion
- Aetherflux Reservoir + Bloodthirsty Conqueror (4740-6191): missing Bloodthirsty Conqueror
- Toralf, God of Fury // Toralf's Hammer + Blasphemous Act (1775-2484): missing Toralf, God of Fury // Toralf's Hammer
- Brass's Bounty + Revel in Riches (2429-5211): missing Brass's Bounty
- Magda, Brazen Outlaw + Aggravated Assault (1486-3750): missing Aggravated Assault
- Karn, the Great Creator + Aetherflux Reservoir (4740-5089): missing Karn, the Great Creator
- Ob Nixilis, Captive Kingpin + All Will Be One (2390-4166): missing All Will Be One
- Mirkwood Bats + Plague of Vermin (2612-5035): missing Plague of Vermin
- Sensei's Divining Top + Aetherflux Reservoir + Gwenom, Remorseless (4740-5078-6900): missing Gwenom, Remorseless
- Academy Manufactor + Mirkwood Bats + There and Back Again (974-2612-4231): missing There and Back Again
- Birgi, God of Storytelling // Harnfel, Horn of Bounty + Underworld Breach + Burning Inquiry (392-1368-2172): missing Burning Inquiry
- ... and 14 more

### yuriko-dimir-ninjas (t4)

Spellbook finds no combo in this list.

Almost included (one card missing), winning, 2-3 cards: 22

- Thousand-Faced Shadow + Peregrine Drake + Silver-Fur Master (307-3821-4520): missing Peregrine Drake
- Thousand-Faced Shadow + Peregrine Drake + Moon-Circuit Hacker (307-2327-3821): missing Peregrine Drake
- Thousand-Faced Shadow + Peregrine Drake + Mistblade Shinobi (86-307-3821): missing Peregrine Drake
- Thousand-Faced Shadow + Great Whale + Silver-Fur Master (307-2792-4520): missing Great Whale
- Thousand-Faced Shadow + Peregrine Drake + Skullsnatcher (307-344-3821): missing Peregrine Drake
- Yuriko, the Tiger's Shadow + Thousand-Faced Shadow + Great Whale (307-2792-4307): missing Great Whale
- Thousand-Faced Shadow + Palinchron + Silver-Fur Master (307-3542-4520): missing Palinchron
- Thousand-Faced Shadow + Great Whale + Moon-Circuit Hacker (307-2327-2792): missing Great Whale
- Thousand-Faced Shadow + Great Whale + Mistblade Shinobi (86-307-2792): missing Great Whale
- Thousand-Faced Shadow + Great Whale + Prosperous Thief (307-2306-2792): missing Great Whale
- Thousand-Faced Shadow + Great Whale + Ingenious Infiltrator (307-2792-3403): missing Great Whale
- Yuriko, the Tiger's Shadow + Thousand-Faced Shadow + Palinchron (307-3542-4307): missing Palinchron
- ... and 10 more

### kinnan-simic-mana-combo (t5)

| Spellbook id | cards | combo | wins | results | prerequisites | sim line |
|---|---|---|---|---|---|---|
| 542-5034 | 2 | Tidespout Tyrant + Sol Ring |  | Infinite colorless mana | You control a nontoken permanent that can be cast using {C}. | no |
| 3430-4131 | 2 | Basalt Monolith + Nyxbloom Ancient |  | Infinite colorless mana |  | no |
| 542-2364 | 2 | Tidespout Tyrant + Mana Vault |  | Infinite colorless mana | You control a nontoken permanent that can be cast using {C}{C}. | no |
| 3990-4222 | 2 | Incubation Druid + Freed from the Real |  | Infinite mana lands you control can produce | Freed from the Real attached to Incubation Druid. Incubation Druid does not have summoning sickness. Incubation Druid has at least one +1/+1 counter on it. You control a land that can produce {U}. | no |
| 542-3682 | 2 | Tidespout Tyrant + Mox Amber |  |  | You control at least one colored legendary creature or planeswalker. You control a nontoken permanent that can be cast using one mana that can be generated by Mox Amber. | no |
| 3525-3990 | 2 | Incubation Druid + Pemmin's Aura |  | Infinite mana lands you control can produce | Pemmin's Aura attached to Incubation Druid. Incubation Druid does not have summoning sickness. Incubation Druid has at least one +1/+1 counter on it. You control a land that can produce {U}. | no |
| 513-3682 | 2 | Hullbreaker Horror + Mox Amber |  | Infinite colored mana | You control a legendary creature that has a color that matches the mana symbol in the other permanent's mana cost. | Bigger: Hullbreaker Horror + Kinnan + two rocks cast between each other |
| 4222-5168 | 2 | Bloom Tender + Freed from the Real |  | Infinite non-blue mana of colors among permanents you control; Infinite mana of colors among permanents you control | Freed from the Real attached to Bloom Tender. Bloom Tender does not have summoning sickness. | Bigger: Freed from the Real / Pemmin's Aura + Bloom Tender |
| 3525-5168 | 2 | Bloom Tender + Pemmin's Aura |  | Infinite non-blue mana of colors among permanents you control; Infinite mana of colors among permanents you control | Pemmin's Aura attached to Bloom Tender. Bloom Tender does not have summoning sickness. | Bigger: Freed from the Real / Pemmin's Aura + Bloom Tender |
| 3430-4222 | 2 | Nyxbloom Ancient + Freed from the Real |  | Infinite blue mana; Infinite mana creatures you control can produce | You control a creature that can tap to produce at least {U}, and Freed from the Real is attached to that creature. | Bigger: Freed from the Real / Pemmin's Aura + Nyxbloom Ancient + a blue mana creature |
| 3430-3525 | 2 | Nyxbloom Ancient + Pemmin's Aura |  | Infinite blue mana; Infinite mana creatures you control can produce | You control a creature that can tap to produce at least {U}, and Pemmin's Aura is attached to that creature. | Bigger: Freed from the Real / Pemmin's Aura + Nyxbloom Ancient + a blue mana creature |
| 4222-7008 | 2 | Badgermole Cub + Freed from the Real |  | Infinite green mana | You control an untapped land that can tap to produce at least {U}. | Bigger: Freed from the Real / Pemmin's Aura + Badgermole Cub + a blue mana creature |
| 3525-7008 | 2 | Badgermole Cub + Pemmin's Aura |  | Infinite green mana | You control an untapped land that can tap to produce at least {U}. | Bigger: Freed from the Real / Pemmin's Aura + Badgermole Cub + a blue mana creature |
| 513--62 | 1 | Hullbreaker Horror |  |  | Permanent castable for {0} that survives | Bigger: Hullbreaker Horror + Kinnan + two rocks cast between each other |
| 513-5034--46 | 2 | Hullbreaker Horror + Sol Ring |  | Infinite colorless mana | Permanent Castable for {C} | Template: Hullbreaker Horror + Kinnan + two rocks cast between each other |
| 513-2364--47 | 2 | Hullbreaker Horror + Mana Vault |  | Infinite colorless mana | Permanent Castable for {C}{C} | Template: Hullbreaker Horror + Kinnan + two rocks cast between each other |
| 129-4222--140 | 2 | Kinnan, Bonder Prodigy + Freed from the Real |  | Infinite blue mana; Infinite mana a specific creature can produce; Put most creature cards from your library onto the battlefield | Mana Dork or Mana Dork Creator; Freed from the Real is attached to the mana dork. The mana dork doesn't have summoning sickness. | Smaller: Kinnan + Freed from the Real / Pemmin's Aura |
| 129-3525--140 | 2 | Kinnan, Bonder Prodigy + Pemmin's Aura |  | Infinite blue mana; Infinite mana a specific creature can produce; Put most creature cards from your library onto the battlefield | Mana Dork or Mana Dork Creator; Pemmin's Aura is attached to the mana dork. The mana dork doesn't have summoning sickness. | Smaller: Kinnan + Freed from the Real / Pemmin's Aura |
| 129-4222-5814 | 3 | Kinnan, Bonder Prodigy + Delighted Halfling + Freed from the Real |  | Infinite colored mana that can only be spent to cast legendary spells |  | Smaller: Kinnan + Freed from the Real / Pemmin's Aura |
| 129-3525-5814 | 3 | Kinnan, Bonder Prodigy + Delighted Halfling + Pemmin's Aura |  | Infinite colored mana that can only be spent to cast legendary spells |  | Smaller: Kinnan + Freed from the Real / Pemmin's Aura |
| 129-4131 | 2 | Kinnan, Bonder Prodigy + Basalt Monolith |  | Infinite colorless mana |  | Exact: Kinnan + Basalt Monolith |

Catalogue lines the deck holds:

- Kinnan + Basalt Monolith: Spellbook 129-4131 (Exact)
- Kinnan + Freed from the Real / Pemmin's Aura: Spellbook 129-4222--140 (Smaller)
- Kinnan + Basalt Monolith + Thrasios (draw the deck): Spellbook 129-4131 (Bigger)
- Kinnan + Basalt Monolith + Mirage Mirror ({U}{G} outlet): Spellbook 129-4131 (Bigger)
- Hullbreaker Horror + Kinnan + two rocks cast between each other: Spellbook 513-5034--46 (Template)
- Freed from the Real / Pemmin's Aura + Bloom Tender: Spellbook 4222-5168 (Bigger)
- Freed from the Real / Pemmin's Aura + Nyxbloom Ancient + a blue mana creature: Spellbook 3430-4222 (Bigger)
- Freed from the Real / Pemmin's Aura + Badgermole Cub + a blue mana creature: Spellbook 4222-7008 (Bigger)
- Hullbreaker Horror + Urza / Sol Ring + free artifacts: Spellbook 513-5034--46 (Template)

Almost included (one card missing), winning, 2-3 cards: 4

- Walking Ballista + Heroic Feast (3693-7743): missing Heroic Feast
- Neerdiv, Devious Diver + Freed from the Real + Enduring Vitality (4222-6006-6210): missing Neerdiv, Devious Diver
- Neerdiv, Devious Diver + Pemmin's Aura + Enduring Vitality (3525-6006-6210): missing Neerdiv, Devious Diver
- Rite of Passage + Mirage Mirror + Walking Ballista (292-2500-3693): missing Rite of Passage

### miirym-temur-dragons (t5)

| Spellbook id | cards | combo | wins | results | prerequisites | sim line |
|---|---|---|---|---|---|---|
| 703-5291 | 2 | Astral Dragon + Cursed Mirror |  | Infinite copies of noncreature permanents you control; Infinite creature tokens with haste | Cursed Mirror cannot be copying a creature. | no |
| 380-703-2557 | 3 | Miirym, Sentinel Wyrm + Astral Dragon + Parallel Lives |  | Near-infinite creature tokens |  | no |

Catalogue lines the deck holds:

- Wrathful Red Dragon + Blasphemous Act: **not in Spellbook's list for this deck**

Almost included (one card missing), winning, 2-3 cards: 18

- Blasphemous Act + Repercussion (2484-4083): missing Repercussion
- Ancient Copper Dragon + Aggravated Assault (778-3750): missing Aggravated Assault
- Aggravated Assault + Old Gnawbone (1800-3750): missing Aggravated Assault
- Toralf, God of Fury // Toralf's Hammer + Blasphemous Act (1775-2484): missing Toralf, God of Fury // Toralf's Hammer
- Old Gnawbone + Hellkite Charger (1800-3398): missing Hellkite Charger
- Savage Ventmaw + Aggravated Assault (62-3750): missing Aggravated Assault
- Orthion, Hero of Lavabrink + Terror of the Peaks (1110-2136): missing Orthion, Hero of Lavabrink
- Heartless Hidetsugu + Twinflame Tyrant (3011-6093): missing Heartless Hidetsugu
- Rionya, Fire Dancer + Terror of the Peaks (1110-1908): missing Rionya, Fire Dancer
- Devastating Onslaught + Terror of the Peaks (1110-6785): missing Devastating Onslaught
- Ganax, Astral Hunter + Aggravated Assault + Utvara Hellkite (84-2972-3750): missing Aggravated Assault
- Terror of the Peaks + Rite of Replication (1110-1744): missing Rite of Replication
- ... and 6 more

### urza-mono-blue-artifacts (t5)

| Spellbook id | cards | combo | wins | results | prerequisites | sim line |
|---|---|---|---|---|---|---|
| 513-2364--47 | 2 | Hullbreaker Horror + Mana Vault |  | Infinite colorless mana | Permanent Castable for {C}{C} | no |
| 777-985-5078 | 3 | Sensei's Divining Top + Foundry Inspector + Mystic Forge |  | Infinite card draw |  | no |
| 985-1976-5078 | 3 | Sensei's Divining Top + Mystic Forge + Etherium Sculptor |  | Infinite card draw |  | no |
| 1976-5078-5243 | 3 | Sensei's Divining Top + Etherium Sculptor + The Reality Chip |  | Infinite card draw |  | no |
| 777-5078-5243 | 3 | Sensei's Divining Top + Foundry Inspector + The Reality Chip |  | Infinite card draw |  | no |
| 457-1976-5078-5390 | 4 | Urza, Lord High Artificer + Forensic Gadgeteer + Sensei's Divining Top + Etherium Sculptor |  | Infinite card draw |  | no |
| 457-777-5078-5390 | 4 | Urza, Lord High Artificer + Forensic Gadgeteer + Sensei's Divining Top + Foundry Inspector |  | Infinite card draw |  | no |
| 1170-5078-5243--30 | 3 | Displacer Kitten + Sensei's Divining Top + The Reality Chip |  | Infinite card draw | Artifact / Creature with "{T}: Add Mana" that can tap upon ETB (Mana Rock); The Reality Chip attached to a creature you control. | no |
| 985-1170-5078--30 | 3 | Displacer Kitten + Sensei's Divining Top + Mystic Forge |  | Infinite card draw | Artifact / Creature with "{T}: Add Mana" that can tap upon ETB (Mana Rock) | no |
| 1170-1976-3363-5078 | 4 | Displacer Kitten + The One Ring + Sensei's Divining Top + Etherium Sculptor |  | Infinite card draw |  | no |
| 777-1170-3363-5078 | 4 | Displacer Kitten + The One Ring + Sensei's Divining Top + Foundry Inspector |  | Infinite card draw |  | no |
| 876-1170-1329-1414 | 4 | Emry, Lurker of the Loch + Displacer Kitten + Lightning Greaves + Lotus Petal |  | Infinite colored mana |  | no |
| 876-1170-1329-2478 | 4 | Emry, Lurker of the Loch + Displacer Kitten + Lightning Greaves + Mishra's Bauble |  | Infinite card draw at the beginning of the next upkeep |  | no |
| 1170-1177-1976-5078 | 4 | Sensei's Divining Top + Displacer Kitten + Narset, Parter of Veils + Etherium Sculptor |  | Infinite card draw |  | no |
| 876-1170-1329-4211 | 4 | Emry, Lurker of the Loch + Displacer Kitten + Lightning Greaves + Tormod's Crypt |  |  |  | no |
| 777-1170-1177-5078 | 4 | Sensei's Divining Top + Displacer Kitten + Narset, Parter of Veils + Foundry Inspector |  | Infinite card draw |  | no |
| 876-1170-1329-1492 | 4 | Emry, Lurker of the Loch + Displacer Kitten + Lightning Greaves + Welding Jar |  |  | There is at least one additional artifact on the battlefield. | no |
| 4821-5261 | 2 | Dramatic Reversal + Isochron Scepter |  | Infinite mana nonland permanents you control can produce; Infinite untap of nonland permanents you control | Nonland permanents you control can tap to produce at least {3}. | Bigger: Isochron Scepter + Dramatic Reversal |
| 4131-4235 | 2 | Rings of Brighthearth + Basalt Monolith |  | Infinite colorless mana |  | Bigger: Power Artifact / Rings of Brighthearth + Monolith |
| 513--62 | 1 | Hullbreaker Horror |  |  | Permanent castable for {0} that survives | Bigger: Hullbreaker Horror + Urza / Sol Ring + free artifacts |
| 513-5034--46 | 2 | Hullbreaker Horror + Sol Ring |  | Infinite colorless mana | Permanent Castable for {C} | Template: Hullbreaker Horror + Urza / Sol Ring + free artifacts |
| 4131-5390 | 2 | Basalt Monolith + Forensic Gadgeteer |  | Infinite colorless mana |  | Exact: Basalt Monolith + Forensic Gadgeteer |

Catalogue lines the deck holds:

- Isochron Scepter + Dramatic Reversal: Spellbook 4821-5261 (Bigger)
- Power Artifact / Rings of Brighthearth + Monolith: Spellbook 4131-4235 (Bigger)
- Basalt Monolith + Forensic Gadgeteer: Spellbook 4131-5390 (Exact)
- Hullbreaker Horror + Urza / Sol Ring + free artifacts: Spellbook 513-5034--46 (Template)

Almost included (one card missing), winning, 2-3 cards: 2

- Sensei's Divining Top + Glaring Fleshraker + Mystic Forge (985-5078-5705): missing Glaring Fleshraker
- Sensei's Divining Top + Glaring Fleshraker + The Reality Chip (5078-5243-5705): missing Glaring Fleshraker

### yawgmoth-mono-black-aristocrats (t5)

| Spellbook id | cards | combo | wins | results | prerequisites | sim line |
|---|---|---|---|---|---|---|
| 2842-5544-5670 | 3 | Warren Soultrader + Forsaken Miner + Blood Artist | **win** | Infinite lifegain triggers; Infinite lifeloss | Your life total is at least 2. | no |
| 2577-5670-6578 | 3 | Warren Soultrader + Gravecrawler + Sephiroth, Fabled SOLDIER // Sephiroth, One-Winged Angel | **win** | Infinite lifegain triggers; Infinite lifeloss | Your life total is at least 2. | no |
| 884-2577-5670 | 3 | Warren Soultrader + Gravecrawler + Ayara, First of Locthwain | **win** | Infinite lifegain triggers; Infinite lifeloss | Your life total is at least 2. | no |
| 5544-5670-6578 | 3 | Warren Soultrader + Forsaken Miner + Sephiroth, Fabled SOLDIER // Sephiroth, One-Winged Angel | **win** | Infinite lifegain triggers; Infinite lifeloss | Your life total is at least 2. | no |
| 2842-4050-5544 | 3 | Forsaken Miner + Phyrexian Altar + Blood Artist | **win** | Infinite lifegain; Infinite lifegain triggers; Infinite lifeloss |  | no |
| 1580-4871-5670-6578 | 4 | Warren Soultrader + Pitiless Plunderer + Reassembling Skeleton + Sephiroth, Fabled SOLDIER // Sephiroth, One-Winged Angel | **win** | Infinite lifeloss | You either have Sephiroth, Fabled SOLDIER on the battlefield or have the emblem granted by Sephiroth, One-Winged Angel. Your life total is 2 or higher. | no |
| 4050-5544-6578 | 3 | Forsaken Miner + Phyrexian Altar + Sephiroth, Fabled SOLDIER // Sephiroth, One-Winged Angel | **win** | Infinite lifegain triggers; Infinite lifegain; Infinite lifeloss |  | no |
| 328-628-4279 | 3 | Mikaeus, the Unhallowed + Yawgmoth, Thran Physician + Gray Merchant of Asphodel | **win** | Infinite card draw; Near-infinite lifegain; Near-infinite lifegain triggers; Near-infinite lifeloss | Your life total is at least 2. You control at least one additional nontoken non-Human creature with toughness two or greater. | no |
| 2760-2842-4279 | 3 | Yawgmoth, Thran Physician + Nest of Scarabs + Blood Artist | **win** | Near-infinite card draw; Near-infinite lifegain triggers; Near-infinite lifeloss | Your life total is at least 2. You control at least one additional creature. There is at least one other creature on the battlefield. | no |
| 4279-4314-6578--216 | 3 | Yawgmoth, Thran Physician + Pawn of Ulamog + Sephiroth, Fabled SOLDIER // Sephiroth, One-Winged Angel | **win** | Infinite card draw; Near-infinite lifegain triggers; Near-infinite lifeloss | Creature with Undying; You either have Sephiroth, Fabled SOLDIER on the battlefield or have the emblem granted by Sephiroth, One-Winged Angel. The undying creature doesn't have a +1/+1 counter on it. Your life total is 2 or higher. | no |
| 3240-4050-4314 | 3 | Nether Traitor + Phyrexian Altar + Pawn of Ulamog |  | Infinite colored mana; Infinite colorless mana |  | no |
| 1385-2760-4279 | 3 | Yawgmoth, Thran Physician + Zulaport Cutthroat + Nest of Scarabs | **win** | Near-infinite card draw; Near-infinite lifegain triggers; Near-infinite lifeloss | You control at least one additional creature. Your life total is at least 2. An opponent controls a creature. | no |
| 2577-5670-6094 | 3 | Warren Soultrader + Gravecrawler + Vengeful Bloodwitch | **win** | Infinite lifegain triggers; Infinite lifeloss | Your life total is at least 2. | no |
| 5544-5670-6094 | 3 | Warren Soultrader + Forsaken Miner + Vengeful Bloodwitch | **win** | Infinite lifegain triggers; Infinite lifeloss | Your life total is at least 2. | no |
| 628-884-4279 | 3 | Yawgmoth, Thran Physician + Mikaeus, the Unhallowed + Ayara, First of Locthwain | **win** | Infinite card draw; Near-infinite lifegain; Near-infinite lifegain triggers; Near-infinite lifeloss | Your life total is at least 2. You control at least one additional nontoken non-Human creature with toughness two or greater. | no |
| 1580-4871-5670-6094 | 4 | Warren Soultrader + Pitiless Plunderer + Reassembling Skeleton + Vengeful Bloodwitch | **win** | Infinite lifeloss | Your life total is 2 or higher. | no |
| 2760-3025-4279 | 3 | Yawgmoth, Thran Physician + Nest of Scarabs + Bastion of Remembrance | **win** | Near-infinite card draw; Near-infinite lifegain triggers; Near-infinite lifeloss | You control at least one additional creature. Your life total is at least 2. An opponent controls a creature. | no |
| 4050-5544-6094 | 3 | Forsaken Miner + Phyrexian Altar + Vengeful Bloodwitch | **win** | Infinite lifegain triggers; Infinite lifegain; Infinite lifeloss |  | no |
| 4279-4314-6094--216 | 3 | Yawgmoth, Thran Physician + Pawn of Ulamog + Vengeful Bloodwitch | **win** | Infinite card draw; Near-infinite lifegain triggers; Near-infinite lifeloss | Creature with Undying; The undying creature doesn't have a +1/+1 counter on it. Your life total is 2 or higher. | no |
| 2393-3240-4050 | 3 | Nether Traitor + Phyrexian Altar + Sifter of Skulls |  | Infinite colored mana; Infinite colorless mana |  | no |
| 2393-4279-6578--216 | 3 | Yawgmoth, Thran Physician + Sifter of Skulls + Sephiroth, Fabled SOLDIER // Sephiroth, One-Winged Angel | **win** | Infinite card draw; Near-infinite lifegain triggers; Near-infinite lifeloss | Creature with Undying; You either have Sephiroth, Fabled SOLDIER on the battlefield or have the emblem granted by Sephiroth, One-Winged Angel. The undying creature doesn't have a +1/+1 counter on it. Your life total is 2 or higher. | no |
| 1580-4050-4556 | 3 | Phyrexian Altar + Reassembling Skeleton + Blight Mound |  | Infinite lifegain; Infinite lifegain triggers |  | no |
| 1531-2760-4279 | 3 | Yawgmoth, Thran Physician + Nest of Scarabs + Deathgreeter |  | Near-infinite card draw; Near-infinite lifegain triggers | Your life total is at least 2. You control at least one additional creature that will trigger Deathgreeter when it dies. There is at least one additional creature on the battlefield. | no |
| 3240-4050-4556 | 3 | Nether Traitor + Phyrexian Altar + Blight Mound |  | Infinite colored mana; Infinite lifegain triggers; Infinite lifegain |  | no |
| 1531-4279-4314--216 | 3 | Yawgmoth, Thran Physician + Deathgreeter + Pawn of Ulamog |  | Infinite card draw; Near-infinite lifegain triggers | Creature with Undying; The undying creature doesn't have a +1/+1 counter on it. Your life total is 2 or higher. | no |
| 2760-4279-6094 | 3 | Yawgmoth, Thran Physician + Nest of Scarabs + Vengeful Bloodwitch | **win** | Near-infinite card draw; Near-infinite lifegain triggers; Near-infinite lifeloss | You control at least one additional creature. Your life total is at least 2. An opponent controls a creature. | no |
| 2760-4279-6578 | 3 | Yawgmoth, Thran Physician + Nest of Scarabs + Sephiroth, Fabled SOLDIER // Sephiroth, One-Winged Angel | **win** | Near-infinite lifegain triggers; Near-infinite card draw; Near-infinite lifeloss | Your life total is at least 2. You control at least one additional creature. There is at least one other creature on the battlefield. | no |
| 2393-4279-6094--216 | 3 | Yawgmoth, Thran Physician + Sifter of Skulls + Vengeful Bloodwitch | **win** | Infinite card draw; Near-infinite lifegain triggers; Near-infinite lifeloss | Creature with Undying; The undying creature doesn't have a +1/+1 counter on it. Your life total is 2 or higher. | no |
| 1531-2577-5670 | 3 | Warren Soultrader + Gravecrawler + Deathgreeter |  | Infinite lifegain triggers | Your life total is at least 2. | no |
| 1531-2393-4279--216 | 3 | Yawgmoth, Thran Physician + Deathgreeter + Sifter of Skulls |  | Infinite card draw; Near-infinite lifegain triggers | Creature with Undying; The undying creature doesn't have a +1/+1 counter on it. Your life total is 2 or higher. | no |
| 1531-1580-4871-5670 | 4 | Warren Soultrader + Pitiless Plunderer + Deathgreeter + Reassembling Skeleton |  |  | Your life total is 2 or higher. | no |
| 2577-4050 | 2 | Gravecrawler + Phyrexian Altar |  |  | You control an additional Zombie. | Bigger: Gravecrawler + Phyrexian Altar + a Zombie + a death payoff |
| 1580-2034-4871 | 3 | Pitiless Plunderer + Ashnod's Altar + Reassembling Skeleton |  | Infinite colorless mana |  | Bigger: Reassembling Skeleton / Gravecrawler + Pitiless Plunderer + an altar + a death payoff |
| 2577-2842-5670 | 3 | Warren Soultrader + Gravecrawler + Blood Artist | **win** | Infinite lifegain triggers; Infinite lifeloss | Your life total is at least 2. | Bigger: Reassembling Skeleton / Gravecrawler + Pitiless Plunderer + an altar + a death payoff |
| 1385-2577-5670 | 3 | Warren Soultrader + Gravecrawler + Zulaport Cutthroat | **win** | Infinite lifegain triggers; Infinite lifeloss | Your life total is at least 2. | Bigger: Reassembling Skeleton / Gravecrawler + Pitiless Plunderer + an altar + a death payoff |
| 1580-4050-4871 | 3 | Pitiless Plunderer + Reassembling Skeleton + Phyrexian Altar |  |  |  | Bigger: Reassembling Skeleton / Gravecrawler + Pitiless Plunderer + an altar + a death payoff |
| 2034-2577-4871 | 3 | Gravecrawler + Pitiless Plunderer + Ashnod's Altar |  | Infinite colorless mana | You control an additional Zombie. | Bigger: Reassembling Skeleton / Gravecrawler + Pitiless Plunderer + an altar + a death payoff |
| 2577-3025-5670 | 3 | Warren Soultrader + Gravecrawler + Bastion of Remembrance | **win** | Infinite lifegain triggers; Infinite lifeloss | Your life total is at least 2. | Bigger: Reassembling Skeleton / Gravecrawler + Pitiless Plunderer + an altar + a death payoff |
| 1580-4050-4314 | 3 | Reassembling Skeleton + Phyrexian Altar + Pawn of Ulamog |  |  |  | Bigger: Phyrexian Altar + Reassembling Skeleton + Pawn of Ulamog / Sifter of Skulls + a death payoff |
| 1580-2393-4050 | 3 | Reassembling Skeleton + Phyrexian Altar + Sifter of Skulls |  |  |  | Bigger: Phyrexian Altar + Reassembling Skeleton + Pawn of Ulamog / Sifter of Skulls + a death payoff |
| 1385-4279-4314--216 | 3 | Yawgmoth, Thran Physician + Zulaport Cutthroat + Pawn of Ulamog | **win** | Infinite card draw; Near-infinite lifegain triggers; Near-infinite lifeloss | Creature with Undying; The undying creature doesn't have a +1/+1 counter on it. Your life total is 2 or higher. | Template: Yawgmoth + undying loop + a death payoff |
| 2842-4279-4314--216 | 3 | Yawgmoth, Thran Physician + Blood Artist + Pawn of Ulamog | **win** | Infinite card draw; Near-infinite lifegain triggers; Near-infinite lifeloss | Creature with Undying; The undying creature doesn't have a +1/+1 counter on it. Your life total is 2 or higher. | Template: Yawgmoth + undying loop + a death payoff |
| 3025-4279-4314--216 | 3 | Yawgmoth, Thran Physician + Bastion of Remembrance + Pawn of Ulamog | **win** | Infinite card draw; Near-infinite lifegain triggers; Near-infinite lifeloss | Creature with Undying; The undying creature doesn't have a +1/+1 counter on it. Your life total is 2 or higher. | Template: Yawgmoth + undying loop + a death payoff |
| 1385-2393-4279--216 | 3 | Yawgmoth, Thran Physician + Zulaport Cutthroat + Sifter of Skulls | **win** | Infinite card draw; Near-infinite lifegain triggers; Near-infinite lifeloss | Creature with Undying; The undying creature doesn't have a +1/+1 counter on it. Your life total is 2 or higher. | Template: Yawgmoth + undying loop + a death payoff |
| 2393-2842-4279--216 | 3 | Yawgmoth, Thran Physician + Sifter of Skulls + Blood Artist | **win** | Infinite card draw; Near-infinite lifegain triggers; Near-infinite lifeloss | Creature with Undying; The undying creature doesn't have a +1/+1 counter on it. Your life total is 2 or higher. | Template: Yawgmoth + undying loop + a death payoff |
| 2393-3025-4279--216 | 3 | Yawgmoth, Thran Physician + Sifter of Skulls + Bastion of Remembrance | **win** | Infinite card draw; Near-infinite lifegain triggers; Near-infinite lifeloss | Creature with Undying; The undying creature doesn't have a +1/+1 counter on it. Your life total is 2 or higher. | Template: Yawgmoth + undying loop + a death payoff |
| 1580-2842-4871-5670 | 4 | Warren Soultrader + Pitiless Plunderer + Reassembling Skeleton + Blood Artist | **win** | Infinite lifeloss | Your life total is 2 or higher. | Exact: Reassembling Skeleton / Gravecrawler + Pitiless Plunderer + an altar + a death payoff |
| 1385-1580-4871-5670 | 4 | Warren Soultrader + Pitiless Plunderer + Zulaport Cutthroat + Reassembling Skeleton | **win** | Infinite lifeloss | Your life total is 2 or higher. | Exact: Reassembling Skeleton / Gravecrawler + Pitiless Plunderer + an altar + a death payoff |
| 1580-3025-4871-5670 | 4 | Warren Soultrader + Pitiless Plunderer + Reassembling Skeleton + Bastion of Remembrance | **win** | Infinite lifeloss | Your life total is 2 or higher. | Exact: Reassembling Skeleton / Gravecrawler + Pitiless Plunderer + an altar + a death payoff |

Catalogue lines the deck holds:

- Gravecrawler + Phyrexian Altar + a Zombie + a death payoff: Spellbook 2577-4050 (Bigger)
- Yawgmoth + undying loop + a death payoff: Spellbook 1385-4279-4314--216 (Template)
- Reassembling Skeleton / Gravecrawler + Pitiless Plunderer + an altar + a death payoff: Spellbook 1580-2842-4871-5670 (Exact)
- Phyrexian Altar + Reassembling Skeleton + Pawn of Ulamog / Sifter of Skulls + a death payoff: Spellbook 1580-4050-4314 (Bigger)

Almost included (one card missing), winning, 2-3 cards: 106

- Sheoldred, the Apocalypse + Peer into the Abyss (1561-2498): missing Peer into the Abyss
- Orcish Bowmasters + Peer into the Abyss (1561-3862): missing Peer into the Abyss
- Warren Soultrader + Gravecrawler + Bontu's Monument (2577-2932-5670): missing Bontu's Monument
- Mikaeus, the Unhallowed + Warren Soultrader + Blood Artist (628-2842-5670--5): missing 
- Gravecrawler + Pitiless Plunderer + Altar of Dementia (2577-4871-5256): missing Altar of Dementia
- Ayara, First of Locthwain + Plague of Vermin (884-5035): missing Plague of Vermin
- Warren Soultrader + Gravecrawler + Funeral Room // Awakening Hall (2577-5670-5931): missing Funeral Room // Awakening Hall
- Mikaeus, the Unhallowed + Triskelion (628-1495): missing Triskelion
- Mikaeus, the Unhallowed + Warren Soultrader + Sephiroth, Fabled SOLDIER // Sephiroth, One-Winged Angel (628-5670-6578--5): missing 
- Nim Deathmantle + Ashnod's Altar + Marionette Apprentice (2034-5003-6199): missing Nim Deathmantle
- Tombstone Stairwell + Blood Artist (2842-6215): missing Tombstone Stairwell
- Warren Soultrader + Aetherflux Reservoir + Gravecrawler (2577-4740-5670): missing Aetherflux Reservoir
- ... and 94 more

### zur-esper-enchantment-control (t5)

| Spellbook id | cards | combo | wins | results | prerequisites | sim line |
|---|---|---|---|---|---|---|
| 4053-4551 | 2 | Solemnity + Phyrexian Unlife |  | You can't lose the game due to having 0 or less life |  | no |
| 1274-3693 | 2 | Heliod, Sun-Crowned + Walking Ballista | **win** | Infinite damage; Infinite lifegain; Infinite lifegain triggers | Walking Ballista has at least two +1/+1 counters on it. | Exact: Heliod, Sun-Crowned + Walking Ballista |

Catalogue lines the deck holds:

- Heliod, Sun-Crowned + Walking Ballista: Spellbook 1274-3693 (Exact)

Almost included (one card missing), winning, 2-3 cards: 28

- Demonic Consultation + Thassa's Oracle (742-1295): missing Demonic Consultation
- Tainted Pact + Thassa's Oracle (1295-3093): missing Tainted Pact
- Approach of the Second Sun + Mystical Tutor (553-2856): missing Approach of the Second Sun
- Doomsday + Thassa's Oracle (1295-3189): missing Doomsday
- Enter the Infinite + Thassa's Oracle (953-1295): missing Enter the Infinite
- Archangel of Thune + Walking Ballista (2919-3693): missing Archangel of Thune
- Approach of the Second Sun + Demonic Tutor (553-4105): missing Approach of the Second Sun
- Cleric Class + Walking Ballista (104-3693): missing Cleric Class
- Leveler + Thassa's Oracle (1068-1295): missing Leveler
- Approach of the Second Sun + Vampiric Tutor (553-1827): missing Approach of the Second Sun
- Heliod, Sun-Crowned + Triskelion (1274-1495): missing Triskelion
- Repay in Kind + Necropotence (2656-3591--84): missing Repay in Kind
- ... and 16 more

### dack-fayden-white-stax (t6)

| Spellbook id | cards | combo | wins | results | prerequisites | sim line |
|---|---|---|---|---|---|---|
| 372-2781-5144 | 3 | Abdel Adrian, Gorion's Ward + Felidar Guardian + Ephemerate |  | Infinite creature tokens |  | no |
| 372-2781-3603 | 3 | Abdel Adrian, Gorion's Ward + Felidar Guardian + Cloudshift |  | Infinite creature tokens |  | no |
| 372-1734-2781 | 3 | Abdel Adrian, Gorion's Ward + Fiend Hunter + Felidar Guardian |  | Infinite creature tokens | Felidar Guardian exiled by Fiend Hunter | no |
| 372-2781-5887 | 3 | Abdel Adrian, Gorion's Ward + Felidar Guardian + Werefox Bodyguard |  | Infinite creature tokens | Felidar Guardian exiled by Werefox Bodyguard | no |
| 372-1734-4605-5144 | 4 | Preston, the Vanisher + Abdel Adrian, Gorion's Ward + Fiend Hunter + Ephemerate |  | Infinite creature tokens |  | no |
| 372-1734-3603-4605 | 4 | Preston, the Vanisher + Abdel Adrian, Gorion's Ward + Fiend Hunter + Cloudshift |  | Infinite creature tokens |  | no |
| 372-2781-6966 | 3 | Abdel Adrian, Gorion's Ward + Felidar Guardian + All-Fates Stalker |  | Infinite creature tokens |  | no |
| 372-2781-5694 | 3 | Abdel Adrian, Gorion's Ward + Felidar Guardian + Eldrazi Confluence |  | Infinite creature tokens |  | no |
| 2781-4605-5144 | 3 | Preston, the Vanisher + Felidar Guardian + Ephemerate |  | Infinite creature tokens; Infinite mana lands you control that enter the battlefield untapped can produce; Infinite mana lands you control can produce | You have enough mana to cast Ephemerate. | Bigger: Preston + Felidar Guardian + Radiant Solar (flicker spell) |
| 2781-3603-4605 | 3 | Preston, the Vanisher + Felidar Guardian + Cloudshift |  | Infinite creature tokens; Infinite mana lands you control that enter the battlefield untapped can produce; Infinite mana lands you control can produce | You have enough mana to cast Cloudshift. | Bigger: Preston + Felidar Guardian + Radiant Solar (flicker spell) |

Catalogue lines the deck holds:

- Preston + Felidar Guardian + Radiant Solar (flicker spell): Spellbook 2781-4605-5144 (Bigger)
- Preston + Felidar Guardian + Radiant Solar (loop running): **not in Spellbook's list for this deck**
- Dack Fayden: the first flip wins the turn (35 piles): **not in Spellbook's list for this deck**

Almost included (one card missing), winning, 2-3 cards: 4

- Approach of the Second Sun + Reprieve (553-1891): missing Approach of the Second Sun
- Karn, the Great Creator + Aetherflux Reservoir (4740-5089): missing Aetherflux Reservoir
- Approach of the Second Sun + Scroll Rack (553-1271): missing Approach of the Second Sun
- Approach of the Second Sun + Bilbo's Gambit (553-7910): missing Approach of the Second Sun

### kinnan-bonder-mana-combo (t6)

| Spellbook id | cards | combo | wins | results | prerequisites | sim line |
|---|---|---|---|---|---|---|
| 513-3682 | 2 | Hullbreaker Horror + Mox Amber |  | Infinite colored mana | You control a legendary creature that has a color that matches the mana symbol in the other permanent's mana cost. | Bigger: Hullbreaker Horror + Kinnan + two rocks cast between each other |
| 513--62 | 1 | Hullbreaker Horror |  |  | Permanent castable for {0} that survives | Bigger: Hullbreaker Horror + Kinnan + two rocks cast between each other |
| 513-5034--46 | 2 | Hullbreaker Horror + Sol Ring |  | Infinite colorless mana | Permanent Castable for {C} | Template: Hullbreaker Horror + Kinnan + two rocks cast between each other |
| 513-2364--47 | 2 | Hullbreaker Horror + Mana Vault |  | Infinite colorless mana | Permanent Castable for {C}{C} | Template: Hullbreaker Horror + Kinnan + two rocks cast between each other |
| 129-4131 | 2 | Kinnan, Bonder Prodigy + Basalt Monolith |  | Infinite colorless mana |  | Exact: Kinnan + Basalt Monolith |

Catalogue lines the deck holds:

- Kinnan + Basalt Monolith: Spellbook 129-4131 (Exact)
- Kinnan + Basalt Monolith + Thrasios (draw the deck): Spellbook 129-4131 (Bigger)
- Kinnan + Basalt Monolith + Mirage Mirror ({U}{G} outlet): Spellbook 129-4131 (Bigger)
- Hullbreaker Horror + Kinnan + two rocks cast between each other: Spellbook 513-5034--46 (Template)
- Hullbreaker Horror + Urza / Sol Ring + free artifacts: Spellbook 513-5034--46 (Template)

### kraum-tymna-blue-breach-midrange (t6)

| Spellbook id | cards | combo | wins | results | prerequisites | sim line |
|---|---|---|---|---|---|---|
| 1368-1414-4856 | 3 | Underworld Breach + Lotus Petal + Brain Freeze | **win** | Infinite self-mill; Near-infinite mill | You have at least six other cards in your graveyard, plus an additional four cards if Brain Freeze is in your graveyard. | no |
| 1368-3518-4856 | 3 | Underworld Breach + Lion's Eye Diamond + Brain Freeze | **win** | Infinite self-mill; Near-infinite colored mana; Near-infinite mill | You have at least six other cards in hand and/or graveyard. | Bigger: Underworld Breach + Lion's Eye Diamond + Brain Freeze (Thassa's Oracle finish) |
| 742-1295 | 2 | Demonic Consultation + Thassa's Oracle | **win** | Win the game |  | Exact: Thassa's Oracle + Demonic Consultation / Tainted Pact |
| 1295-3093 | 2 | Tainted Pact + Thassa's Oracle | **win** | Win the game | No two cards in library share a name. | Exact: Thassa's Oracle + Demonic Consultation / Tainted Pact |

Catalogue lines the deck holds:

- Thassa's Oracle + Demonic Consultation / Tainted Pact: Spellbook 742-1295 (Exact)
- Underworld Breach + Lion's Eye Diamond + Brain Freeze (Thassa's Oracle finish): Spellbook 1368-3518-4856 (Bigger)
- Underworld Breach + Lion's Eye Diamond + Brain Freeze (at the table): Spellbook 1368-3518-4856 (Bigger)
- Ad Nauseam into a win: **not in Spellbook's list for this deck**

Almost included (one card missing), winning, 2-3 cards: 26

- Orcish Bowmasters + Peer into the Abyss (1561-3862): missing Peer into the Abyss
- Approach of the Second Sun + Mystical Tutor (553-2856): missing Approach of the Second Sun
- Demonic Consultation + Laboratory Maniac (742-2836): missing Laboratory Maniac
- Doomsday + Thassa's Oracle (1295-3189): missing Doomsday
- Enter the Infinite + Thassa's Oracle (953-1295): missing Enter the Infinite
- Tainted Pact + Laboratory Maniac (2836-3093): missing Laboratory Maniac
- Approach of the Second Sun + Demonic Tutor (553-4105): missing Approach of the Second Sun
- Demonic Consultation + Jace, Wielder of Mysteries (742-3096): missing Jace, Wielder of Mysteries
- Leveler + Thassa's Oracle (1068-1295): missing Leveler
- Approach of the Second Sun + Vampiric Tutor (553-1827): missing Approach of the Second Sun
- Tainted Pact + Jace, Wielder of Mysteries (3093-3096): missing Jace, Wielder of Mysteries
- Thassa's Oracle + Thought Lash (1295-2853): missing Thought Lash
- ... and 14 more

### rograkh-silas-turbo-breach (t6)

| Spellbook id | cards | combo | wins | results | prerequisites | sim line |
|---|---|---|---|---|---|---|
| 1368-1414-4856 | 3 | Underworld Breach + Lotus Petal + Brain Freeze | **win** | Infinite self-mill; Near-infinite mill | You have at least six other cards in your graveyard, plus an additional four cards if Brain Freeze is in your graveyard. | no |
| 1368-1878-2706 | 3 | Underworld Breach + Wheel of Fortune + Jeska's Will |  | Infinite looting for all players | You have at least six additional cards in your graveyard, plus an additional six cards between your hand and/or graveyard. An opponent has at least six cards in hand. | no |
| 1368-2706-3518 | 3 | Underworld Breach + Lion's Eye Diamond + Wheel of Fortune |  | Infinite looting | You have at least six other cards in hand and/or graveyard. | no |
| 1368-3698--44 | 2 | Underworld Breach + Grinding Station |  | Infinite self-mill; Near-infinite mana among colors of legendary creatures and planeswalkers you control | Artifact Castable for {0} that survives | no |
| 1368-3518-4856 | 3 | Underworld Breach + Lion's Eye Diamond + Brain Freeze | **win** | Infinite self-mill; Near-infinite colored mana; Near-infinite mill | You have at least six other cards in hand and/or graveyard. | Bigger: Underworld Breach + Lion's Eye Diamond + Brain Freeze (Thassa's Oracle finish) |
| 742-1295 | 2 | Demonic Consultation + Thassa's Oracle | **win** | Win the game |  | Exact: Thassa's Oracle + Demonic Consultation / Tainted Pact |
| 1295-3093 | 2 | Tainted Pact + Thassa's Oracle | **win** | Win the game | No two cards in library share a name. | Exact: Thassa's Oracle + Demonic Consultation / Tainted Pact |

Catalogue lines the deck holds:

- Thassa's Oracle + Demonic Consultation / Tainted Pact: Spellbook 742-1295 (Exact)
- Underworld Breach + Lion's Eye Diamond + Brain Freeze (Thassa's Oracle finish): Spellbook 1368-3518-4856 (Bigger)
- Underworld Breach + Lion's Eye Diamond + Brain Freeze (at the table): Spellbook 1368-3518-4856 (Bigger)
- Ad Nauseam into a win: **not in Spellbook's list for this deck**

Almost included (one card missing), winning, 2-3 cards: 18

- Demonic Consultation + Laboratory Maniac (742-2836): missing Laboratory Maniac
- Doomsday + Thassa's Oracle (1295-3189): missing Doomsday
- Enter the Infinite + Thassa's Oracle (953-1295): missing Enter the Infinite
- Tainted Pact + Laboratory Maniac (2836-3093): missing Laboratory Maniac
- Demonic Consultation + Jace, Wielder of Mysteries (742-3096): missing Jace, Wielder of Mysteries
- Leveler + Thassa's Oracle (1068-1295): missing Leveler
- Tainted Pact + Jace, Wielder of Mysteries (3093-3096): missing Jace, Wielder of Mysteries
- Birgi, God of Storytelling // Harnfel, Horn of Bounty + Underworld Breach + Burning Inquiry (392-1368-2172): missing Burning Inquiry
- Thassa's Oracle + Thought Lash (1295-2853): missing Thought Lash
- Paradigm Shift + Thassa's Oracle (1295-3367): missing Paradigm Shift
- Divining Witch + Thassa's Oracle (1295-4267): missing Divining Witch
- Mirror of Fate + Thassa's Oracle (1164-1295): missing Mirror of Fate
- ... and 6 more

### rograkh-thrasios-creature-combo (t6)

| Spellbook id | cards | combo | wins | results | prerequisites | sim line |
|---|---|---|---|---|---|---|
| 802-864-2404 | 3 | Eternal Witness + Snap + Gaea's Cradle |  | Infinite green mana; Infinite mana lands you control can produce; Infinite untap of lands you control | You control a mana-producing land that can tap to produce at least {U}. You control at least [#] creatures. You control four or more creatures. | Bigger: Eternal Witness + Snap + Gaea's Cradle |
| 1174-2404-6678 | 3 | Oboro Breezecaller + Talon Gates of Madara + Gaea's Cradle |  | Infinite mana lands you control can produce; Infinite green mana | You control at least five creatures. | Bigger: Oboro Breezecaller + Gaea's Cradle + Talon Gates of Madara |

Catalogue lines the deck holds:

- Oboro Breezecaller + Gaea's Cradle + Talon Gates of Madara: Spellbook 1174-2404-6678 (Bigger)
- Eternal Witness + Snap + Gaea's Cradle: Spellbook 802-864-2404 (Bigger)
- Badgermole Cub + clone + Oboro Breezecaller + Gaea's Cradle: **not in Spellbook's list for this deck**
- Badgermole Cub + clone + Eternal Witness + Snap: **not in Spellbook's list for this deck**

Almost included (one card missing), winning, 2-3 cards: 5

- Aggravated Assault + Enduring Vitality (3750-6006): missing Aggravated Assault
- Aggravated Assault + Cryptolith Rite (3750-4289): missing Aggravated Assault
- Blazing Sunsteel + Phyrexian Metamorph (2723-3734): missing Blazing Sunsteel
- Blazing Sunsteel + Clever Impersonator (2723-4753): missing Blazing Sunsteel
- Hateflayer + Training Grounds + Cryptolith Rite (4289-4449-4893): missing Hateflayer

### sisay-legends-toolbox (t6)

| Spellbook id | cards | combo | wins | results | prerequisites | sim line |
|---|---|---|---|---|---|---|
| 2364-2608-4499 | 3 | Derevi, Empyrial Tactician + Emiel the Blessed + Mana Vault |  |  |  | no |
| 527-2608-4499 | 3 | Derevi, Empyrial Tactician + Emiel the Blessed + Selvala, Heart of the Wilds |  | Infinite untap of permanents you control; Infinite colored mana; Infinite mana permanents you control can produce | Selvala does not have summoning sickness. | no |
| 527-4499-4927 | 3 | Selvala, Heart of the Wilds + Emiel the Blessed + Tyvar, Jubilant Brawler |  | Infinite colored mana |  | no |
| 1432-5963 | 2 | Marvin, Murderous Mimic + Ioreth of the Healing House |  | Infinite untap of legendary creatures you control; Infinite mana legendary creatures you control can produce | There is at least one other legendary creature on the battlefield. Marvin and Ioreth do not have summoning sickness. | Bigger: Ioreth + Marvin + a legend that taps for mana (DDR) |
| 2608-4499-5168 | 3 | Derevi, Empyrial Tactician + Emiel the Blessed + Bloom Tender |  | Infinite untap of permanents you control; Infinite mana of colors among permanents you control; Infinite mana permanents you control can produce |  | Bigger: Derevi + Emiel + Gaea's Cradle / Bloom Tender (Flash Unicorn) |
| 2404-2608-4499 | 3 | Derevi, Empyrial Tactician + Emiel the Blessed + Gaea's Cradle |  | Infinite untap of permanents you control; Infinite green mana; Infinite mana permanents you control can produce | You control at least three creatures. | Bigger: Derevi + Emiel + Gaea's Cradle / Bloom Tender (Flash Unicorn) |

Catalogue lines the deck holds:

- Ioreth + Marvin + a legend that taps for mana (DDR): Spellbook 1432-5963 (Bigger)
- Derevi + Emiel + Gaea's Cradle / Bloom Tender (Flash Unicorn): Spellbook 2608-4499-5168 (Bigger)

Almost included (one card missing), winning, 2-3 cards: 12

- Orcish Bowmasters + Peer into the Abyss (1561-3862): missing Peer into the Abyss
- Aggravated Assault + Selvala, Heart of the Wilds (527-3750): missing Aggravated Assault
- Magda, Brazen Outlaw + Aggravated Assault (1486-3750): missing Aggravated Assault
- Najeela, the Blade-Blossom + Derevi, Empyrial Tactician (2011-2608): missing Najeela, the Blade-Blossom
- Approach of the Second Sun + Demonic Tutor (553-4105): missing Approach of the Second Sun
- Approach of the Second Sun + Vampiric Tutor (553-1827): missing Approach of the Second Sun
- Aggravated Assault + Enduring Vitality (3750-6006): missing Aggravated Assault
- Najeela, the Blade-Blossom + Selvala, Heart of the Wilds (527-2011): missing Najeela, the Blade-Blossom
- Bruce Banner // The Incredible Hulk + Legolas's Quick Reflexes (149-7124): missing Bruce Banner // The Incredible Hulk
- Agatha's Soul Cauldron + Neerdiv, Devious Diver + Ioreth of the Healing House (1432-4613-6210): missing Neerdiv, Devious Diver
- Agatha's Soul Cauldron + Servant of Tymaret + Ioreth of the Healing House (1432-4613-6859): missing Servant of Tymaret
- Agatha's Soul Cauldron + Siren of the Silent Song + Ioreth of the Healing House (1432-1557-4613): missing Siren of the Silent Song

### Catalogue lines no deck holds

- Kiki-Jiki + Zealous Conscripts / Felidar / Resto (2 groups)
- Karn, the Great Creator + Mycosynth Lattice (2 groups)
- Godo + Helm of the Host (Hammer of Nazahn out) (3 groups)
- Godo + Helm of the Host (Brass Squire attaches) (3 groups)
- Godo + Helm of the Host (Magnetic Theft attaches) (3 groups)
- Godo + Helm of the Host (equip {5}) (2 groups)
- Godo + Peter Parker's Camera copies the enter trigger (4 groups)
- Godo + Hammer first, then a copy spell on Godo (4 groups)
- Godo + Hammer first, then Cursed Mirror / Sculpting Steel as Godo (4 groups)
- Godo + Hammer first, then Jaxis copies Godo (4 groups)
- Dualcaster Mage + a copy spell (2 groups)
- Goblin Welder + Goblin Engineer, then Godo (5 groups)
- Godo already out, Hammer out: Cursed Mirror / Sculpting Steel as a second Godo (4 groups)
- Godo already out, Hammer out: Jaxis blitzes a copy of Godo (4 groups)
- Godo already out, Hammer out: a token copy spell on Godo (4 groups)
- Godo + Basalt Monolith + a copied Kinnan (4 groups)
- Rionya, Fire Dancer + Combat Celebrant (2 groups)
- Heliod, Sun-Crowned + Triskelion (2 groups)
- Peregrine Drake + Deadeye Navigator (+ Agent of Treachery) (3 groups)
- Melira + Kitchen Finks (3 groups)
- Nim Deathmantle + Ashnod's Altar + Priest of Gix (3 groups)

72 catalogue lines, 170 piece names.
