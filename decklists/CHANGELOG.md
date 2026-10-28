# Deck changelog

The history of the decks in `decklists/JD/` and `decklists/Avery/`: every change to a list (newest first), the
simulator results over time, and tuning notes. The deck documents themselves describe only the current list.
`cardgen deck KEY LIST --log "Why."` adds a new entry at the top of the deck's **Changes** list.

JD's decks: Cloud, Karn, Kefka, Sauron, Seph/Atraxa, Sheoldred, Tymna/Thrasios, Ur-Dragon, Veyran, Y'shtola.
Avery's decks: Alela, Galadriel.

## Cloud, Midgar Mercenary — Mono-White Equipment

Deck key `cloud`, `decklists/JD/cloud-midgar-equipment.md`.

### Changes

- **2026-10-27.** Out: Wall of Omens. In: Galadriel's Dismissal. JD's choice (2026-10-27), not simulated: Galadriel's Dismissal (from Seph/Atraxa) for Wall of Omens: {W} instant protection for Cloud and his Equipment.
- **2026-10-25.** Out: Wrecking Ball Arm. In: Buster Sword. JD's choice (10/25): Buster Sword is a cheaper Equipment (cast {3}, equip {2}) that gives +3/+2 and draws a card on each hit, with a free spell up to the damage dealt. Not simulated; the free-cast part isn't modeled yet.
- **2026-10-18.** Out: Citizen's Arrest, Hero's Blade. In: Andúril, Narsil Reforged, Stroke of Midnight. Citizen's Arrest -> Stroke of Midnight: -1.4 over T1-T3 (instant answer to any nonland permanent). Hero's Blade -> Andúril, Narsil Reforged: -0.3 over T1-T3 (a +1/+1 counter on every creature each swing).
- **2026-10-16.** Out: Duelist's Heritage, Ghostly Prison, Helm of Kaldra, Shield of Kaldra. In: Archangel of Tithes, Open the Armory, Silverblade Paladin, Sphere of Safety. Four swaps (JD, 10-16). Out: Helm of Kaldra, Shield of Kaldra, Ghostly Prison, Duelist's Heritage. In: Archangel of Tithes, Open the Armory, Sphere of Safety, Silverblade Paladin. JD is taking Helm and Shield of Kaldra for another deck; Ghostly Prison and Duelist's Heritage could not be found. Test: T1 29.1 -> 30.3 (+1.2, ±3.8, neutral). Buys: Sphere of Safety $4.38, Silverblade Paladin $0.87 = $5.25.
- **2026-10-16.** Out: Bronze Guardian, Kor Duelist. In: Danitha Capashen, Paragon, Sram, Senior Edificer. Bronze Guardian -> Sram, Senior Edificer and Kor Duelist -> Danitha Capashen, Paragon (JD's choice). Tested neutral: T1 29.2 -> 29.7 (+0.5, noise ±3.8, 1,000 games paired). Buys: Sram $1.73, Danitha $0.75.
- **2026-10-15.** Out: Crawlspace. In: Ghostly Prison. Crawlspace is in JD's Sheoldred deck (physical), so Ghostly Prison returns to its Package D slot (opponents pay {2} for each creature that attacks you). Ghostly Prison is a buy.
- **2026-10-15.** Out: Nim Deathmantle, Secure the Scene, Veteran's Powerblade. In: Akroma's Will, Duelist's Heritage, Fireshrieker. Double strike package (JD's choice, 2026-10-15). Out: Secure the Scene, Veteran's Powerblade, Nim Deathmantle (to bulk). In: Fireshrieker, Duelist's Heritage, Akroma's Will (buys, $19.36: Akroma's Will 16.67, Duelist's Heritage 2.46, Fireshrieker 0.23). Why: double strike on an equipped Cloud is 4 Sword triggers per attack plus double commander damage; the engine fix made double strike two damage steps. Numbers: quick T1 +6.1 (+/-3.6); 2,000 games T1 25.9 -> 30.3 (+4.5 +/-2.6), T2 20.4 -> 24.8 (+4.3 +/-2.5); measured on the list before the Crawlspace/Archon substitutes, which tested within noise of it; audit/cloud/2026-10-15/ds/.
- **2026-10-15.** Out: Recommission, Ultimate Magic: Holy, Unbreakable Formation. In: Elspeth, Sun's Champion, Wall of Omens, Winds of Abandon. Package D2 (JD's choice, 2026-10-15). Out: Unbreakable Formation, Ultimate Magic: Holy, Recommission (all to bulk; the cards were dead: Formation cast 1%, Holy 11%, Recommission stuck in hand 33-39%). In: Winds of Abandon, Elspeth, Sun's Champion, Wall of Omens (buys, about $5.46: Winds of Abandon 4.79, Elspeth 0.38, Wall of Omens 0.29). Defense is Cloud's one working lever. Paired test, look-ahead: T1 +3.6 (+/-2.5) at 2,000 games, +3.4 in a quick check; T2 +1.5 (+/-2.4); audit/cloud/2026-10-15/d2/.
- **2026-10-15.** Out: Archangel of Tithes, Ghostly Prison. In: Archon of Absolution, Crawlspace. Swapped Ghostly Prison -> Crawlspace and Archangel of Tithes -> Archon of Absolution, because JD could not find Ghostly Prison or Archangel of Tithes. Tested substitutes vs the Package D list: T1 -1.5 (noise +-1.9), T2 -0.9 (+-1.8), within noise, so adopted (audit/cloud/2026-10-15/subs/log.txt). Buys: the six ordered Package D cards (Norn's Annex, Wrath of God, Day of Judgment, Windborn Muse, Baird, Fumigate) plus Archon of Absolution ($0.20); Crawlspace comes from the binder.
- **2026-10-15.** Out: Beatrix, Loyal General, Ennis, Debate Moderator, Explorer's Scope, Rootborn Defenses, Stormfront Riders, Unfinished Business, Unyielding Gatekeeper, White Mage's Staff. In: Archangel of Tithes, Baird, Steward of Argive, Day of Judgment, Fumigate, Ghostly Prison, Norn's Annex, Windborn Muse, Wrath of God. Package D (JD's choice, 2026-10-15). Out: Stormfront Riders, Beatrix, Loyal General, Unyielding Gatekeeper, Ennis, Debate Moderator, Explorer's Scope, White Mage's Staff, Unfinished Business, Rootborn Defenses (all to bulk). In: Ghostly Prison, Norn's Annex, Archangel of Tithes, Windborn Muse, Baird, Steward of Argive, Wrath of God, Day of Judgment, Fumigate (buys, about $23.05 total: Ghostly Prison 6.67, Norn's Annex 5.75, Archangel of Tithes 4.39, Wrath of God 3.93, Day of Judgment 0.91, Windborn Muse 0.82, Baird 0.31, Fumigate 0.27). Attack taxes slow the combat decks and the wipes leave the Equipment behind. Paired test, look-ahead, 2,000 games per tier, on the AI-fixed Cloud: T1 15.8 -> 22.2, T2 13.8 -> 20.1 (+6.4 / +6.3); audit/cloud/2026-10-15/. Later the same day Ghostly Prison and Archangel of Tithes (not findable) were replaced by Crawlspace (binder) and Archon of Absolution ($0.20), so those two are not buys; see the entry above.
- **2026-10-13 (later).** Out: Patriot, Shield Wielder (JD couldn't find the card). In: Ultimate Magic: Holy (bulk; in
  the list until C4 made room for Giver of Runes): permanents you control gain indestructible until end of turn.
  Not simulated.
- **2026-10-13.** JD committed to the C3 + C4 list: 27 swaps. Out: Argivian Find, Call a Surprise Witness, Cho-Manno's
  Blessing, Conformer Shuriken, Essence Reliquary, Harsh Annotation, Hero's Heirloom, Hulkbuster Armor, Magitek Scythe,
  Magnetic Snuffler, Make Your Move, Medic's Kitesail, Memory Trap, Potioner's Trove, Rapier Wit, Recruitment Officer,
  Repel the Darkness, Shardmage's Rescue, Squire's Lightblade, Take Up the Shield, Tome of Legends, Trailblazer's
  Boots, Ultimate Magic: Holy, White Auracite. In, by group: **mana** (Sol Ring, Arcane Signet, Mind Stone, Fellwar
  Stone, Coalition Relic, Thought Vessel, Smothering Tithe, Solemn Simulacrum, Knight of the White Orchid, Loyal
  Warhound; 33 lands with Plaza of Heroes); **swords and protection** (Sword of Fire and Ice, Sword of Feast and
  Famine, Sword of Hearth and Home, Champion's Helm, Swiftfoot Boots, Giver of Runes, Grand Abolisher); **free equips
  and backup carriers** (Ardenn, Auriok Steelshaper, Kor Outfitter, Balan, Armored Skyhunter, Kor Duelist); **others
  from the retired Jodah deck** (Enlightened Tutor, Skyclave Apparition, Court of Ardenvale). Measured on the fixed
  Cloud AI (paired, look-ahead, loose, 500 games per tier, against the old list): **10.9% over all tiers against
  5.1%** (T1 14.0, T2 9.8, T3 7.4, T4 9.4, T5 13.8; +5.8, every tier above its noise);
  `audit/cloud/2026-10-13/screen/C4REV-*`. Sourcing: Sword of Feast and Famine, Sword of Hearth and Home, Smothering
  Tithe, Champion's Helm and Swiftfoot Boots came from other decks, 11 cards were bought 2026-10-13 for about $33.26.
- **2026-10-12.** Added. JD's candidate for a last deck with no black or green (no Swamps or Forests left). Built from
  the bulk and binder lists, the Kaldra pieces from Jodah (if this deck is chosen, Jodah gives them up), and one buy:
  Cloud, Midgar Mercenary (about $21). In the simulator as deck key `cloud`.

### Simulator history

- **2026-10-13.** Paired screen of the C3 + C4 list against the old list: 10.9% over all tiers against 5.1% (see the
  2026-10-13 entry above for the per-tier figures; `audit/cloud/2026-10-13/screen/C4REV-*`). The later Ultimate Magic:
  Holy swap was not simulated.

## Godo, Bandit Warlord — Mono-Red cEDH Equipment Turbo

Deck key `godo` (removed), `decklists/JD/godo-cedh-equipment.md` (removed).

**Retired 2026-10-25: never built.** Godo, Bandit Warlord and the Helm of the Host combo went into The Ur-Dragon (`decklists/JD/ur-dragon-equipment.md`). The entries below are the deck's history.

### Changes

- **2026-10-17.** Added. JD's first cEDH (Bracket 5) deck, an off-meta Godo + Helm of the Host turbo list built from
  the EDHREC average of 35 tournament Godo decks, checked against 2026 top finishes. Mana Crypt, Jeweled Lotus and
  Dockside Extortionist are banned and not in it. Not simulated.

## Karn, Legacy Reforged — Colourless Urza and Mishra Artifacts

Deck file `decklists/JD/karn-colorless-artifacts.md`; not in the simulator.

### Changes

- **2026-10-20.** Added. A colourless artifact-ramp deck built around Karn's upkeep mana, with every Urza and Mishra
  card that fits the plan (the Tron lands, Urza's Saga, Cave, Workshop and Factory, Mishra's Workshop, Factory and
  Foundry, both Baubles, Karn, Scion of Urza and others). All 99 checked Commander-legal and colourless on Scryfall.
  Bracket 4: five Game Changers. Eight expensive cards are proxied. Not simulated: many cards are not modeled yet.

## Kefka, Court Mage — Grixis Discard

Deck key `kefka`, `decklists/JD/kefka-grixis-discard.md`.

### Changes

- **2026-10-27.** Out: Rhystic Study. In: Stinkweed Imp. JD's choice (2026-10-27), not simulated: Rhystic Study goes to Seph/Atraxa; Stinkweed Imp (binder) takes the slot: discard fodder that dredges back. Two Game Changers, still Bracket 4 (the two-card win).
- **2026-10-19.** Out: Hallway Heckler. In: Kuja, Genome Sorcerer // Trance Kuja, Fate Defied. Out: Hallway Heckler. In: Kuja, Genome Sorcerer. JD approved 10/19: +2.2 at T3 (30.2% -> 32.5%, ±2.4; 2,000 paired games, look-ahead, loose; audit/kefka/2026-10-19-kuja/). Kuja's Wizard tokens double the deck's burn damage (2.7 -> 5.7 per game).
- **2026-10-18.** Out: Feed the Swarm. In: Marvin, Murderous Mimic. Feed the Swarm -> Marvin, Murderous Mimic: +0.2 at T3 in the coarse pass (copies Niv-Mizzet's draw, Fangtail, Blightspeaker, Rummaging Goblin).
- **2026-10-18.** Out: Blazing Firesinger, Crux of Fate. In: Bribery, Fiery Emancipation. Kefka lift (JD approved 10/18): Crux of Fate=>Fiery Emancipation; Blazing Firesinger=>Bribery
- **2026-10-18.** Out: Bitter Triumph. In: Decorum Dissertation. Matched to the physical deck (JD's list, 2026-10-18): Decorum Dissertation back in for Bitter Triumph; the 10/14 swap was never made physically.
- **2026-10-18.** Out: Demonic Tutor, Mystical Tutor, Vampiric Tutor. In: Expressive Iteration, Mana Leak, Sleight of Hand. Band calibration 10/18 (JD approved): Demonic Tutor=>Mana Leak;Vampiric Tutor=>Expressive Iteration;Mystical Tutor=>Sleight of Hand
- **2026-10-14.** Out: Decorum Dissertation. In: Bitter Triumph, Shivan Reef, Sulfurous Springs. Decorum Dissertation (to bulk) for Bitter Triumph; two Swamps for Sulfurous Springs and Shivan Reef (all three from the binder, the retired Jodah deck's leftovers). Measured -1.9 over all tiers (T1 -6.0, T2 -3.4, T3 -2.6, T4 +2.2, T5 +0.4; audit/leftovers/2026-10-14/README.md; look-ahead, loose, 500 paired games per tier); within noise; JD's choice to apply it. A pool run is pending.
- **2026-10-13 (package A, Yawgmoth's Will storm turn).** Out: Disintegrate, Kaervek's Purge, Twinflame. In: Mystical Tutor, Vampiric Tutor, Yawgmoth's Will. Twinflame, Kaervek's Purge and Disintegrate went to bulk; Yawgmoth's Will (JD's spare copy), Vampiric Tutor and Mystical Tutor (binder, from the retired Jodah deck) came in. Measured +1.0 over all tiers (T1 -3.2, T2 +1.4, T3 +1.6, T4 +2.2, T5 +2.8; look-ahead, loose, 500 paired games per tier; audit/jd-packages/2026-10-13/). Six Game Changers, still Bracket 4. Jin-Gitaxias stays (JD).
- **2026-10-13 (second change).** Out: Grim Tutor, Diabolic Intent, Swiftfoot Boots. In: Profane Tutor, Demonic Tutor, Chaos Warp. JD's choice, not simulated: the two tutors come from the retired Jodah deck instead of being bought (Profane Tutor is suspended for {1}{B} and finds any card two turns later; Demonic Tutor is a Game Changer, so the deck moved to Bracket 4). Swiftfoot Boots went to the Cloud deck (Kefka keeps Lightning Greaves for haste); Chaos Warp came back from bulk.
- **2026-10-13 (Breach package).** Out: Chaos Warp, Dark Inquiry, Duress, Ichor Slick, Postmortem Professor, Potioner's Trove, Thought Vessel. In: Brain Freeze, Dark Ritual, Diabolic Intent, Grapeshot, Grim Tutor, Jeska's Will, Lotus Petal. JD's choice: Sauron's Underworld Breach package (Brain Freeze, Dark Ritual, Lotus Petal) with Grapeshot (bulk), Grim Tutor, Diabolic Intent and Jeska's Will (from Veyran), to run the Breach + storm line alongside Kefka's own Underworld Breach. Measured +0.5 over all tiers, within noise (T1 +3.3, T2 +0.6, T3 -0.5, T4 -0.7, T5 -0.3; look-ahead, loose, 1,000 paired games per tier; audit/sauron/2026-10-12-weakness/kefka-k3/). Three Game Changers then (Rhystic Study, Underworld Breach, Jeska's Will).
- **2026-10-12.** Out: Stargaze (to bulk). In: Twinflame (binder). JD's choice, not simulated: hasty token copies of your creatures for a turn (a second Massacre Wurm, Jin-Gitaxias or Intellect Devourer); Stargaze was cast in only 11-12% of the games it was drawn (diagnostic, audit/kefka/diag).
- **2026-10-11 (trade with Veyran, moved as functional blocks).** In from Veyran: Deflecting Swat and Slip Out the Back (protection), Underworld Breach (graveyard), Niv-Mizzet, the Firemind (a second draw-to-damage kill with the flipped Kefka), Jin-Gitaxias, Progress Tyrant, and Steam Vents, Sulfur Falls, Spirebluff Canal and Stormcarved Coast (for 4 Mountains). Out: Mystic Confluence and Splatter Technique (to Veyran), Arcane Omens and Inner Fire (to bulk), Triskelion (to the binder). Paired look-ahead test, 1,000 games per tier: +1.3 points (28.1% -> 29.4%). Two Game Changers then (Rhystic Study, Underworld Breach).
- **2026-10-11 (mana base).** JD's rule: basics over slow lands. Out: Temple of Malice, Boiling Rock Prison, Sinister Hideout, Temple of Deceit, Treno, Dark City, Spectacle Summit, Temple of Epiphany, Vivid Marsh, Path of Ancestry, Evolving Wilds, Fabled Passage (everything that enters tapped or fetches a tapped basic). In: 5 Swamp, 3 Mountain, 3 Island (then 15 Swamp, 9 Mountain, 6 Island). The six untapped nonbasics stayed. Still 36 lands; the cut lands went back to the bulk list.
- **2026-10-11 (added).** Built from JD's bulk and binder around Kefka, Court Mage, who left Sauron for the job (Sephiroth, Planet's Heir took his slot there); nine cards came from Veyran in the trade above and the 24 basics were JD's own. Fully modeled and measured on 2026-10-11: 26.6% over all tiers (see Simulator history).

### Simulator history

- **2026-10-14.** The leftovers swap (Decorum Dissertation for Bitter Triumph; two Swamps for Sulfurous Springs and Shivan Reef) measured -1.9 over all tiers, within noise, in a paired test (T1 -6.0, T2 -3.4, T3 -2.6, T4 +2.2, T5 +0.4; look-ahead, loose, 500 games per list per tier).
- **2026-10-13.** Yawgmoth's Will + tutor package: +1.0 (paired, 500 games per list per tier; T1 -3.2, T2 +1.4, T3 +1.6, T4 +2.2, T5 +2.8). Breach + storm swaps: +0.5 (1,000 paired games per tier).
- **2026-10-12.** 29.4% over all tiers (±0.5; rank 4 of the ten decks, 6,720 games per tier; `decklists/pool/pool-results.md` section 0d).
- **2026-10-11, Veyran trade** (`audit/kefka-veyran/2026-10-11-trade/`, look-ahead, 1,000 paired games per tier): T1 27.7%, T2 39.8%, T3 27.8%, T4 29.3%, T5 22.4%: 29.4%, +1.3 points, no tier worse. Deflecting Swat and Slip Out the Back alone were +0.6 of it.
- **2026-10-11, AI fix** (`rust/agents/report-kefka-ai-fix.md`): the AI held Psychosis Crawler too long (cast in about 22% of the games it was drawn, then about 65%); it now casts Underworld Breach, aims Niv-Mizzet's pings so a short library is never drawn out, and gives Garrison Excavator its Spirits on Breach escapes. Fast AI on a fresh seed: 20.9% -> 26.8%. Look-ahead after the fix, before the trade: T1 26.6%, T2 38.1%, T3 24.4%, T4 29.4%, T5 21.9% (28.1%).
- **2026-10-11, first measurement** (look-ahead AI, loose profile, 960 games per tier, the same code as the other eight decks; `decklists/pool/pool-results.md` section 0d): T1 25.4%, T2 35.3%, T3 23.3%, T4 28.4%, T5 20.8%; 26.6% over all tiers, just above an even share, in Sauron's group. Deck key `kefka` (`commander-sim --deck kefka --pool t3`), fully modeled since 2026-10-11 (every card Full in `cardgen audit --mine`; card code and AI plan in `rust/crates/sim-core/src/impls/kefka.rs`, tests in `tests/cards_kefka.rs`).

### Tuning notes

- AI behaviour: it discards madness, mayhem and graveyard-castable cards to Kefka's trigger first, flips as soon as {8} is spare, and starts the Psychosis Crawler loop only when the library outlasts the highest opponent's life total (each card drawn costs every opponent 1 life).
- Known simulator simplifications: madness casts the card from the graveyard rather than from exile (so Garrison Excavator can see it leave); Garrison Excavator doesn't see an opponent exiling your graveyard.

## Sauron, the Dark Lord — Grixis Amass

Deck key `sauron`, `decklists/JD/sauron-grixis-amass.md`.

### Changes

- **2026-10-18.** Out: Spell Pierce. In: Urza, Lord High Artificer. Matched to the physical deck (JD's list, 2026-10-18): Urza, Lord High Artificer in for Spell Pierce (tested +0.5, noise, on branch adnaus).
- **2026-10-18.** Out: Slaughter Pact. In: Force of Will. Band calibration 10/18 (JD approved): Slaughter Pact=>Force of Will
- **2026-10-13 (package C).** Out: Bedevil. In: Sheoldred, the Apocalypse. JD's package C: Bedevil out (to bulk), Sheoldred, the Apocalypse in (JD's own spare copy): every opponent draw from Jace's Archivist's wheel costs 2 life. Measured +2.7 over all tiers (T1 +5.0, T2 +4.8, T3 +0.6, T4 +2.8, T5 +0.2; look-ahead, loose, 500 paired games per tier; audit/jd-packages/2026-10-13/). Not a Game Changer: still two, Bracket 3.
- **2026-10-13 (Swords).** Out: Sword of Feast and Famine, Sword of Hearth and Home. In: Chaos Warp, Spell Pierce. JD's choice: both Swords go to the Cloud deck; Chaos Warp and Spell Pierce come back from bulk (both had been cut earlier on 10-13) as answers. Paired test, look-ahead, 500 games per tier: -1.7 over four tiers (t1 +1.6, t2 -3.4, t3 -1.2, t5 -3.6; t4 was blocked by a Conspicuous Snoop crash, since fixed); `audit/cloud/2026-10-13/screen/D1-*`. The Sword of Feast and Famine + Aggravated Assault infinite-combat combo (made 09-22) is gone. Game Changers unchanged (2).
- **2026-10-13 (rebuild).** Out: Blasphemous Act, Brain Freeze, Chaos Warp, Conqueror's Flail, Dark Ritual, Deepglow Skate, Flux Channeler, Lotus Petal, Spell Pierce, Sword of Fire and Ice, Toxic Deluge, Underworld Breach. In: Chandra's Ignition, Comet Storm, Dimir Signet, Dreadhorde Invasion, Exsanguinate, Fellwar Stone, Inferno Titan, Lazotep Reaver, Massacre Wurm, Rakdos Signet, Ral and the Implicit Maze, Talisman of Indulgence. JD's choice, from the 2026-10-12/13 audit (audit/sauron/2026-10-12-weakness/): one-sided removal in place of the symmetric wipes, a faster start (rocks, Dreadhorde Invasion, Lazotep Reaver), and the Underworld Breach package replaced by Exsanguinate, Comet Storm and two rocks. Measured +8.3 over all tiers with Torment of Hailfire in Comet Storm's slot (24.3% -> 32.6%; T1 +6.8, T2 +8.8, T3 +10.0, T4 +9.9, T5 +5.8; look-ahead, loose, 1,000 paired games per tier); Comet Storm for Torment then measured -0.6, within noise. Brain Freeze, Dark Ritual and Lotus Petal go to Kefka, Flux Channeler to Veyran. Two Game Changers now (Cyclonic Rift, Orcish Bowmasters; Underworld Breach left). The rebuild made the deck faster (the first Army hit about a turn earlier) and kinder to the Army. Removed softest cards: the Breach package, the symmetric wipes, Deepglow Skate and the Swords that never connected.
- **2026-10-11 (fifth change).** Out: Phyrexian Arena. In: Cost of Brilliance. JD's choice: Arena moves to the Sheoldred deck (saving a buy); Cost of Brilliance (bulk; {2}{B} sorcery: target player draws two and loses 2 life, and a +1/+1 counter on up to one creature, the Army) takes the slot. Measured +0.6 over all tiers, within noise (look-ahead, loose, 1,000 paired games per tier; audit/sauron/2026-10-11-arena-cost-of-brilliance/).
- **2026-10-11 (fourth change).** Out: Sheoldred, the Apocalypse. In: Bedevil. JD's choice: Sheoldred leaves to lead her own mono-black deck (decklists/JD/sheoldred-mono-black.md); Bedevil comes back from the binder for the slot. Measured -2.1 points over all tiers (look-ahead, loose, 1,000 paired games per tier; Bedevil vs Sheoldred: T1 27.5 vs 28.9, T2 30.8 vs 34.6, T3 22.0 vs 25.5, T4 22.3 vs 24.2, T5 22.0 vs 22.1; audit/sauron/2026-10-11-sheoldred-bedevil/). No card in the deck was there mainly for her: Jace's Archivist's wheel still pays off with Orcish Bowmasters.
- **2026-10-11 (third change).** Out: Thought Vessel. In: Relic of Legends. JD's choice: Thought Vessel moves to Y'shtola, whose draw engines (Necropotence, Consecrated Sphinx, Notion Thief) overflow the hand most. Relic of Legends (taken from the Zada deck, which is being taken apart) keeps a mana rock toward the six-mana commander (Arcane Signet is already in the deck); it taps for any colour, or tap a legendary creature for one more. Not measured.
- **2026-10-11 (second change).** Out: Bedevil. In: Sephiroth, Planet's Heir. JD's choice, not measured: Sephiroth, Planet's Heir ({4}{U}{B}, 4/4 vigilance) gives opponents' creatures -2/-2 when he enters and grows as their creatures die, a board-control threat in Kefka's old slot. From the bulk list.
- **2026-10-11.** Out: Kefka, Court Mage. In: Bedevil. JD's choice: Kefka leaves to become the commander of his own deck (decklists/JD/kefka-grixis-discard.md); Bedevil ({B}{B}{R} instant: destroy target artifact, creature or planeswalker), cut on 10-01, comes back from the binder for the slot. Measured -1.8 points on average (look-ahead, loose, 1,000 paired games per tier; Bedevil vs Kefka: T1 24.1 vs 26.9, T2 30.6 vs 32.2, T3 23.1 vs 23.4, T4 19.5 vs 22.4, T5 20.4 vs 21.9). Each tier is within its own noise (about ±3), but all five favour Kefka, so losing him costs Sauron about two points (audit/sauron/2026-10-11-kefka-bedevil/).
- **2026-10-07.** Out: Marchesa, the Black Rose. In: Talrand, Sky Summoner. A 2/2 flying Drake for every instant or sorcery, so the deck has bodies besides the Orc Army (it loses to combat in about three games in four). Tested even with Marchesa (0.0 points, fast AI, 1,000 pairs per tier). Talrand took Marchesa's slot; the deck runs 25 instants and sorceries.
- **2026-10-06.** Out: Brush Off. In: Spell Pierce. One blue mana is easy to hold up on the Breach turn.
- **2026-10-05.** Out: Consecrated Sphinx. In: Fact or Fiction (took Consecrated Sphinx's slot). Back to Bracket 3: three Game Changers (Underworld Breach, Cyclonic Rift, Orcish Bowmasters). Tested: no loss (+0.1 points on average, 1,000 paired games per tier, adaptive AI).
- **2026-10-03 (second change).** Out: Reconnaissance Mission. In: Disdainful Stroke. A two-mana answer to wipes, big creatures and commanders (mana value 4 or more), held up while the Army grows. Reconnaissance Mission drew one card per hit for a one-creature attack plan. Reconnaissance Mission went to the binder.
- **2026-10-03.** Out: Ral Zarek, Guest Lecturer. In: Marchesa, the Black Rose. Marchesa gave every creature dethrone (a +1/+1 counter when attacking the player with the most life, the Army included) and returned any creature with a +1/+1 counter that died at the next end step. Ral Zarek's planeswalker slot did the least. Ral Zarek went to the binder.
- **2026-10-02.** Out: Plaza of Heroes. In: Thought Vessel. Thought Vessel is a two-mana rock toward the six-mana commander, and no maximum hand size suits the draw engines. Plaza of Heroes was the softest land (to the binder).
- **2026-10-01 (the Underworld Breach package).** Out: Bedevil, Feed the Swarm, Metallic Mimic, Slip Out the Back. In: Brain Freeze, Dark Ritual, Lotus Petal, Underworld Breach (Bedevil -> Underworld Breach, Slip Out the Back -> Brain Freeze, Metallic Mimic -> Dark Ritual, Feed the Swarm -> Lotus Petal). A Breach + Brain Freeze combo fuelled by Dark Ritual and Lotus Petal. The four cuts were the softest removal, the protection spell, and the Army's least important growth card (all four to the binder). Tested in the simulator before the change (look-ahead AI, loose profile): over all five tiers it was about even with the old list (-1.3 points, within noise), better against the Bracket 4 tiers and worse against Bracket 3 ones; a confirming 800-game run against Tiers 4 and 5 found +2.9 and -0.1. Of the three Breach versions tried (with Grapeshot, a lighter package without rituals, and this one), this was the best. Underworld Breach is a fourth Game Changer: the deck moved to Bracket 4.
- **2026-09-28.** Out: Big Score. In: Sword of Fire and Ice, a third combat Sword on the Army. Big Score's two cards and Treasure were among the softest slots; the Sword turned every Army hit into a card and 2 damage, and its protection from blue and red dodges most burn and bounce. A paired simulator test (500 games per tier, both interaction profiles, fast AI) found no clear change in any tier, with a slight negative lean overall (about -0.8 points): the Sword only pays off once the Army connects, where Big Score was immediate. The test also found the AI had been under-casting compiled Swords (Sword of Hearth and Home included); that is fixed. Bedevil was the alternative cut if you'd rather keep Big Score. Not a Game Changer, so the deck stayed Bracket 3 at the cap with three. Deck stayed at 100 and 36 lands. Big Score went to the binder.
- **2026-09-27.** Out: Tezzeret's Gambit. In: Diabolic Intent, a third tutor. Not a Game Changer. Roughly even in the simulator: the least costly of six cuts tried, within noise of zero. Its weakness in this deck is the sacrifice. Almost every creature here is an engine or a threat, so Diabolic Intent always costs something real. It's a third tutor for the Sword, not a card-advantage upgrade. Tezzeret's Gambit went to the binder.
- **2026-09-26 (second pass).** Out: Champion's Helm, Kindred Discovery, Rhystic Study, Sword of the Animist, Unearth. In: Consecrated Sphinx, Sheoldred, the Apocalypse, Kefka, Court Mage, Brush Off, Talisman of Dominance. Consecrated Sphinx took Rhystic Study's Game Changer slot, so the deck stayed at three. The draw-tax plan shifted from Rhystic Study (opponents pay or you draw) to two creatures that punish every card an opponent draws. In the simulator (heuristic AI, loose profile, 1,000 paired games per tier) the new list beat the old one by +2.5, +6.4, +0.5, +5.8, and +3.1 points against Tiers 1-5, clearest against Tiers 2 and 4. Sheoldred was measured on her own too: cutting Champion's Helm for her was the best of four cuts tried, about +3 points. The five cuts went to the binder.
- **2026-09-26.** Six swaps. None is a Game Changer, so the deck stayed Bracket 3 at the cap with three. Net effect: the softest slots (Erebos, Tome, Unclaimed Territory) are gone; the Army gains insurance against removal (The Ozolith) and a second Sword that re-buys ETBs; interaction adds a free kill spell and a one-sided artifact wipe in place of a soft counter. Deck stayed at 100 and 36 lands. All six outgoing cards went to the binder.

  | Out | In | What it does |
  | --- | --- | --- |
  | Erebos, God of the Dead | Sword of Hearth and Home | The softest slot (rarely a creature in Grixis) becomes a second combat Sword that flickers an ETB creature and ramps a basic on every hit. Simmed better than Erebos under both AI profiles. |
  | Hellkite Tyrant | Urabrask, Heretic Praetor | Top-end threat that also taxes opponents' draw steps and keeps them casting into Sauron. |
  | Witch-king, Bringer of Ruin | The Ozolith | Trades a six-drop evasive body for counter insurance on the Army. |
  | Memory Lapse | Slaughter Pact | Soft counter -> free unconditional (nonblack) kill spell for the combo turn. |
  | Tome of Legends | Vandalblast | A dead slot -> one-sided artifact wipe. |
  | Unclaimed Territory | Haunted Ridge | Orc-only fixing -> real B/R dual. Unclaimed Territory only made coloured mana for Orc creature spells; Haunted Ridge is a real B/R dual. |

- **2026-09-22.** Gollum -> Witch-king, Hero's Blade -> Infernal Grasp, Colossus Hammer -> Sword of Feast and Famine, Mask of Memory -> Jace's Archivist. The Sword completes an infinite-combat combo with Aggravated Assault.
- **2026-09-21.** Notion Thief -> Rhystic Study, then Thrummingbird -> Inexorable Tide.
- **2026-09-18 / 09-19 / 09-20.** Earlier passes: rebuild, Nibelheim Aflame, extra-combat + land upgrade, bracket correction.
- **2026-09-17.** Deck built (the old file carried only the bare date).

### Simulator history

- **2026-10-13 (paired tests, pool run pending).** The Sheoldred, the Apocalypse swap (Bedevil out, Sheoldred in) measured +2.7 over all tiers (T1 +5.0, T2 +4.8, T3 +0.6, T4 +2.8, T5 +0.2; look-ahead, loose, 500 paired games per list per tier). A pool run of the ten decks on this list had not been done yet. Rebuild result: 32.6% over all tiers against 24.3% for the list before it (1,000 paired games per tier, every tier up by more than its noise of about ±3.4 to ±4.0): T1 33.7%, T2 39.5%, T3 32.1%, T4 30.7%, T5 27.0%, measured with Torment of Hailfire in Comet Storm's slot (Comet Storm for Torment then measured -0.6, within noise).
- **2026-10-12.** Pool figure 24.8% (rank 5 of ten; `decklists/pool/pool-results.md` section 0d).

### Tuning notes

- Weakest remaining inclusions: Diabolic Intent if you find you never have a spare creature to feed it; Vandalblast on tables without artifacts.

## Sephiroth, Fabled SOLDIER — Mono-Black Wipe, Drain and Steal the Dead

Deck key `sephfs` (removed), `decklists/JD/sephiroth-fs-wipe-steal.md` (moved to `decklists/pool/retired/`).

**Retired 2026-10-27 (JD).** Its cards went to other decks, none of it simulated: Sephiroth, Fabled SOLDIER, Vampiric Tutor, Grave Researcher // Reanimate, Living Death, Rune-Scarred Demon, Dark Ritual, Phyrexian Altar, Dread Return and Phyrexian Tower to Seph/Atraxa; The One Ring, Demonic Tutor, Gray Merchant of Asphodel, Call of the Ring and Bolas's Citadel to Sheoldred; the rest to bulk. The list is kept in `decklists/pool/retired/sephiroth-fs-wipe-steal.md` so the Rust tests can seat it. The entries below are the deck's history.

### Changes

- **2026-10-18.** Out: Zombify. In: Dread Return. Zombify -> Dread Return: +4.8 at T5 in the coarse pass (500 games, just past noise); same cost and job, plus a flashback (sacrifice three creatures) whose deaths drain with Sephiroth out.
- **2026-10-18.** Out: Bitter Triumph, Masterful Flourish, Rabid Attack, Star Compass, Tome of Legends. In: Bolas's Citadel, Demonic Tutor, Gray Merchant of Asphodel, Rune-Scarred Demon, Vampiric Tutor. Band calibration 10/18 (JD approved): Star Compass=>Demonic Tutor;Tome of Legends=>Vampiric Tutor;Masterful Flourish=>Rune-Scarred Demon;Bitter Triumph=>Bolas's Citadel;Rabid Attack=>Gray Merchant of Asphodel
- **2026-10-14 (second change).** Out: Ophiomancer, Sneering Shadewriter. In: Grave Titan, Pawn of Ulamog. JD's package
  (2026-10-14): Grave Titan for Sneering Shadewriter, Pawn of Ulamog for Ophiomancer. Measured +0.3 ±1.6 over all tiers
  (paired, 1,000 games per tier; `audit/sephfs/2026-10-14-pkg/`): neutral; JD is buying both. Grave Titan is a second
  copy (the Atraxa deck has one; $0.33); Pawn of Ulamog $0.55.
- **2026-10-14 (first change).** Out: Caged Zombie. In: The One Ring. JD's choice: The One Ring for Caged Zombie.
  Measured +1.8 ±0.8 over all tiers and +2.8 ±1.0 against Tiers 1-3, the tiers this deck plays (paired, 4,000 games per
  tier; `audit/sephfs/2026-10-14-ring4k/`). The deck's first Game Changer: Bracket 3. The One Ring is $113.59.
- **2026-10-14 (added).** JD's mono-black control deck with Sephiroth, Fabled SOLDIER as commander and JD's new
  Phyrexian Tower and Phyrexian Altar. Not JD's Atraxa deck (`sephiroth-phyrexian-reanimator.md`, also called
  Sephiroth, key `seph`): that one reanimates its own creatures for combos; this one wipes the board, drains on every
  death and takes opponents' dead. 34 buys, about $183.38 (about $155.38 without Sephiroth; The One Ring is $113.59 of
  it). Crux of Fate and Animate Dead are second copies: JD's first copies stay in the Kefka and Atraxa decks (Animate
  Dead $7.29). Bitter Triumph $0.29 (the Jodah copy went to Kefka, #65). Everything else came off JD's bulk and binder
  lists (main, 2026-10-14); Toxic Deluge, Star Compass, Command Tower, Path of Ancestry came from the Jodah binder;
  Dark Ritual and Call of the Ring from the binder; Phyrexian Tower and Phyrexian Altar are JD's own (new); 32 Swamp his
  own.

### Simulator history

- **2026-10-14.** Not measured as a whole yet; only the paired tests above (+0.3 ±1.6 for the Grave Titan / Pawn of
  Ulamog package; +1.8 ±0.8 for The One Ring, +2.8 ±1.0 against Tiers 1-3).

### Tuning notes

- Doc fixes made in the 2026-10-14 cleanup: the old doc listed Extinction Event among the wipes that drain, but it
  exiles (no deaths, no drain), and listed Zombify as taking from any graveyard, but it returns from your own.

## Sephiroth, the Savior — Phyrexian Reanimator

Deck key `seph`, `decklists/JD/sephiroth-phyrexian-reanimator.md`. (Sephiroth, the Savior is the Final Fantasy printing of Atraxa, Grand Unifier; the import list uses the Atraxa name. Not the Sephiroth FS deck, key `sephfs`.)

### Changes

- **2026-10-27.** Out: Arcane Sanctum, Ash Barrens, Brushland, Evolving Wilds, Exotic Orchard, Fabled Passage, Glacial Fortress, Hinterland Harbor, Isolated Chapel, Sunpetal Grove, Treno, Dark City, Woodland Cemetery, Yavimaya Coast. In: Bayou, Flooded Strand, Godless Shrine, Marsh Flats, Misty Rainforest, Polluted Delta, Savannah, Scrubland, Tropical Island, Tundra, Underground Sea, Verdant Catacombs, Windswept Heath. Mana upgrade (JD): 13 slow, conditional and basic-fetch lands out for the six original duals, the six on-colour fetches and Godless Shrine. JD's choice, not simulated.
- **2026-10-27.** Out: Assassin's Trophy, Diabolic Tutor, Galadriel's Dismissal, Harmonize, Hornet Queen, Melira, Sylvok Outcast, Mind Stone, Momentary Blink, Tireless Tracker, Unburial Rites, Zulaport Cutthroat. In: Crucible of Worlds, Dark Ritual, Dread Return, Glimpse the Unthinkable, Grave Researcher // Reanimate, Living Death, Phyrexian Altar, Phyrexian Tower, Rhystic Study, Rune-Scarred Demon, Sephiroth, Fabled SOLDIER // Sephiroth, One-Winged Angel, Vampiric Tutor. JD's choice (2026-10-27), not simulated: Sephiroth FS is retired and nine of its cards come here (Sephiroth, Fabled SOLDIER, Vampiric Tutor, Grave Researcher // Reanimate, Living Death, Rune-Scarred Demon, Dark Ritual, Phyrexian Altar, Dread Return, Phyrexian Tower); Rhystic Study comes from Kefka; Crucible of Worlds and Glimpse the Unthinkable are buys. Zulaport Cutthroat goes to Sheoldred, Galadriel's Dismissal to Cloud, Assassin's Trophy to Tymna/Thrasios; with Melira out the deck has three loops. Five Game Changers, still Bracket 4.
- **2026-10-24.** Out: Flickering Hound. In: Open the Armory. Flickering Hound -> Open the Armory (JD's choice, 2026-10-24). The Hound moves to Tymna/Thrasios. Open the Armory ({1}{W}) finds Animate Dead, Skullclamp or Nim Deathmantle. It's a second copy (Cloud keeps its own), so it's a buy. Not simulated.
- **2026-10-18.** Out: Dread Return. In: Victimize. Dread Return -> Victimize: +0.8 over T3-T4 in the coarse pass (two creatures back for 3 mana, wins 77-82% when cast).
- **2026-10-18.** Out: Consecrated Sphinx, Farewell, Force of Will, Teferi's Protection, Worldly Tutor. In: Arcane Denial, Austere Command, Harmonize, Hornet Queen, Tireless Tracker. Band calibration 10/18 (JD approved): Force of Will=>Hornet Queen;Farewell=>Austere Command;Teferi's Protection=>Arcane Denial;Consecrated Sphinx=>Harmonize;Worldly Tutor=>Tireless Tracker
- **2026-10-14.** Out: Orzhov Signet. In: Worldly Tutor. Orzhov Signet (to bulk) for Worldly Tutor (binder, from the retired Jodah deck). Measured -0.1 over all tiers (T1 -2.0, T2 +0.8, T3 +2.2, T4 -1.0, T5 -0.4; audit/leftovers/2026-10-14/README.md; look-ahead, loose, 500 paired games per tier); JD's choice to keep it. Eight Game Changers, Bracket 4.
- **2026-10-13.** Out: Smothering Tithe. In: Momentary Blink. JD's choice: Smothering Tithe goes to the Cloud deck; Momentary Blink (bulk) is a second Ephemerate-style blink for the Kitten. Paired test, 500 games per tier: -1.8 (t1 -4.0, t2 -1.4, t3 +3.8, t4 -3.8, t5 -3.8); `audit/cloud/2026-10-13/screen/D2-*`. Game Changers 8 -> 7, still Bracket 4.
- **2026-10-10 (second pass).** Avacyn's Pilgrim -> Flickering Hound: the Pilgrim only made {W}, and the Hound turns every creature you cast (Muldrotha's recasts included) into another flicker. Still eight Game Changers, Bracket 4, 100 cards and 37 lands.
- **2026-10-10 — a flicker package.** Five swaps: Stinkweed Imp -> Teleportation Circle, Lash of the Balrog -> Conjurer's Closet, Grisly Salvage -> Restoration Angel, Llanowar Elves -> Summon: Bahamut, and Llanowar Wastes -> Karn's Bastion. None is a Game Changer, so the count stays at eight; still Bracket 4, 100 cards and 37 lands. Six swaps with the Hound (second pass):

  | Out | In | What it does |
  | --- | --- | --- |
  | Stinkweed Imp | Teleportation Circle | Yard filler -> an end-step flicker of a creature (or a tapped mana rock) every turn. |
  | Lash of the Balrog | Conjurer's Closet | Spot removal -> a second end-step flicker. |
  | Grisly Salvage | Restoration Angel | Yard filler -> an instant-speed flicker that also saves a bomb from removal. |
  | Llanowar Elves | Summon: Bahamut | A mana dork -> two destroys, two cards and Mega Flare; a reanimation target. |
  | Llanowar Wastes | Karn's Bastion | A painland -> a colorless land that proliferates (Bahamut's lore, +1/+1 counters). |
  | Avacyn's Pilgrim | Flickering Hound | A {W} mana dork -> a flicker on every creature you cast (second pass). |

  Chosen for flavour more than strength. Got: all six (Teleportation Circle, Conjurer's Closet, Restoration Angel, Summon: Bahamut, Karn's Bastion, Flickering Hound). To bulk: Stinkweed Imp, Lash of the Balrog, Grisly Salvage, Llanowar Elves, Llanowar Wastes, Avacyn's Pilgrim.
- **2026-10-05.** Out: Bolas's Citadel. In: Phyrexian Arena. Tested: no loss (0.0 points on average, 1,000 paired games per tier, adaptive AI). Eight Game Changers; still Bracket 4.
- **2026-10-02.** Out: Cultivate. In: Grave Pact. Grave Pact turns every sacrifice into an edict on each opponent: the free outlets, Grave Titan's Zombies and the loops all feed it. Cultivate was the least needed ramp in a deck that cheats its threats in. Cultivate went to the binder.
- **2026-10-01.** Out: Massacre Wurm. In: Muldrotha, the Gravetide. Muldrotha replays permanents from the graveyard (a land, a creature, an artifact, an enchantment each turn). Massacre Wurm was a fair finisher and one-sided wipe in a deck that wins with loops; Muldrotha is a grind engine that keeps the loops and the reanimation running through removal: a reanimation enchantment, a loop piece, and a land back from the graveyard every turn. Not a Game Changer, so the count stayed at nine. Not tested in the simulator yet: it models Muldrotha's body but not her graveyard casting, so a measurement then would undervalue her. Massacre Wurm went to the binder.
- **2026-09-28.** Unmarked Grave -> Ephemerate. Not a Game Changer, so the count stays at nine. Deck stays at 100 and 37 lands. Unmarked Grave was the weakest yard-filler: it can't bin a legendary card, and most of the bombs are legendary. Ephemerate adds a one-mana answer to targeted removal and theft on top of Galadriel's Dismissal, and doubles as a value blink with rebound. A paired simulator test (500 games per tier, both interaction profiles, fast AI) found no clear change in win rate in any tier; the swap buys resilience without costing speed.
- **2026-09-27.** Persist -> Dread Return, Dovin's Veto -> Force of Will, Diabolic Intent -> Teferi's Protection, and Blossoming Sands -> Arcane Sanctum. Force of Will and Teferi's Protection are Game Changers, so the deck then ran nine. (With the 09-26 second pass, 7 out, 7 in:)

  | Out | In | What it does |
  | --- | --- | --- |
  | Carrion Feeder | Avacyn's Pilgrim | A limited sac outlet -> a mana dork (and a spare body to sacrifice). |
  | Gray Merchant of Asphodel | Kitchen Finks | A drain -> half of two new infinite loops (with Melira or Mikaeus). |
  | Sephiroth, Planet's Heir | Melira, Sylvok Outcast | A bomb -> the other half of the Finks loop. |
  | Persist | Dread Return | Nonlegendary-only reanimation -> any creature, plus a free flashback. |
  | Dovin's Veto | Force of Will | A noncreature counter -> a free counter for anything. Game Changer. |
  | Diabolic Intent | Teferi's Protection | A tutor -> wipe and lethal insurance. Game Changer. |
  | Blossoming Sands | Arcane Sanctum | A tapped W/G land -> a tapped W/U/B land. |

  Arcane Sanctum swapped one tapped land for another, but with blue for Force of Will and Consecrated Sphinx in place of green. Carrion Feeder, Gray Merchant of Asphodel, Sephiroth, Planet's Heir, Persist, Dovin's Veto and Blossoming Sands went to the binder.
- **2026-09-26 (second pass).** Carrion Feeder -> Avacyn's Pilgrim, Gray Merchant of Asphodel -> Kitchen Finks, and Sephiroth, Planet's Heir -> Melira, Sylvok Outcast. Kitchen Finks with Melira or Mikaeus makes two more infinite loops, so the deck now has four. (Table under 2026-09-27.)
- **2026-09-26 — moved to Bracket 4.** Ten swaps: Phyrexian Metamorph -> Aura Shards, Night's Whisper -> Gifts Ungiven, Swiftfoot Boots -> Assassin's Trophy, Deadly Dispute -> Galadriel's Dismissal, Phyrexian Arena -> Bolas's Citadel, Evil Reawakened -> Consecrated Sphinx, Satyr Wayfinder -> Displacer Kitten, Lethal Scheme -> Grand Abolisher, Wrath of God -> Triskelion, and Cryptolith Rite -> Nim Deathmantle. The deck then ran seven Game Changers and two infinite combos (Mikaeus + Triskelion, and Nim Deathmantle + Ashnod's Altar + Grave Titan), so it is Bracket 4 on both counts and was the strongest deck by design. Deck stays at 100 and 37 lands.

  | Out | In | What it does |
  | --- | --- | --- |
  | Phyrexian Metamorph | Aura Shards | Clone -> repeatable artifact/enchantment removal on every creature ETB. Game Changer. |
  | Night's Whisper | Gifts Ungiven | Two cards -> a four-card tutor that also fills the yard. Game Changer. |
  | Swiftfoot Boots | Assassin's Trophy | Equipment -> instant removal for any permanent. |
  | Deadly Dispute | Galadriel's Dismissal | Sac-for-cards -> cheap protection. |
  | Phyrexian Arena | Bolas's Citadel | Slow draw engine -> play off the top and a ten-permanent finisher. Game Changer. |
  | Evil Reawakened | Consecrated Sphinx | A sixth reanimation spell -> a card-advantage bomb that is also a reanimation target. Game Changer. |
  | Satyr Wayfinder | Displacer Kitten | Yard filler -> ETB re-buy engine. |
  | Lethal Scheme | Grand Abolisher | Removal -> combo-turn protection. |
  | Wrath of God | Triskelion | A sweeper -> half of the Mikaeus infinite. |
  | Cryptolith Rite | Nim Deathmantle | Ramp -> half of the Grave Titan infinite. |

  Phyrexian Metamorph, Deadly Dispute, Evil Reawakened, Satyr Wayfinder, Wrath of God and Cryptolith Rite went to the binder.
- **2026-09-22.** Six swaps (Vampiric Tutor -> Diabolic Tutor, Phyrexian Altar -> Ashnod's Altar, Phyrexian Tower -> Watery Grave, Soul Enervation -> Zulaport Cutthroat, Phyrexian Delver -> Smothering Tithe, Witch-king -> Dovin's Veto), then Talisman of Dominance -> Cryptolith Rite.
- **2026-09-21.** Corroding Dragonstorm -> Swan Song and Path of Ancestry -> Blossoming Sands.
- **2026-09-20.** Mana-base upgrade only — four tapped lands out, four untapped duals in (net land count unchanged at 37).
- **2026-09-18.** Rebuilt for consistency: lower curve, more early mana, a real graveyard-filling package.
- **2026-09-17.** First version of the deck (the old file opened with this bare date).

### Simulator history

- **2026-10-14.** Swapping Orzhov Signet for Worldly Tutor measured -0.1 over all tiers, within noise, in a paired test (T1 -2.0, T2 +0.8, T3 +2.2, T4 -1.0, T5 -0.4; look-ahead, loose, 500 games per list per tier); JD chose to keep it. A pool run was pending. Previous list's figures: 36.9% over all tiers on 2026-10-12 (±0.5; rank 2 of the ten decks): T1 41.4%, T2 50.0%, T3 39.3%, T4 32.2%, T5 21.3%, 6,720 games per tier, every deck on the same code; `decklists/pool/pool-results.md` section 0d.
- **2026-10-13.** Smothering Tithe -> Momentary Blink: -1.8 (t1 -4.0, t2 -1.4, t3 +3.8, t4 -3.8, t5 -3.8), 500 paired games per tier; `audit/cloud/2026-10-13/screen/D2-*`.
- **2026-10-10.** Adaptive AI, loose profile, 1,000 paired games per tier: the first three flicker swaps with Soulherder in Teleportation Circle's slot were +0.6 ± 1.2 points, and all five (with Bahamut in Yawgmoth's Will's slot) were -0.5 ± 1.4: both within noise. Teleportation Circle works like Conjurer's Closet in the simulator, and the exact list had not been run.
- **2026-10-05.** Bolas's Citadel -> Phyrexian Arena: 0.0 points on average, 1,000 paired games per tier, adaptive AI.
- **2026-09-28.** Unmarked Grave -> Ephemerate: no clear change in any tier (500 games per tier, both interaction profiles, fast AI).
- **2026-09-27.** Look-ahead AI, loose profile, 120 paired games per tier: Force of Will and Teferi's Protection together were worth about +2 points on average. Force of Will did the work: on its own it was the best of five cuts tried. Teferi's Protection was about even on its own; it is insurance, so it only shows up in the games it saves. 20–38% of the deck's wins (depending on the tier) came from a loop, but adding the Finks lines barely moved the overall win rate: the loops mostly won games the deck was already winning. Force of Will was worth more than any of the cut cards.

### Tuning notes

- **Weakest remaining slots.** Caves of Koilos (a painland) is the softest land.
- **Vampiric Tutor** is in the binder and is the natural addition now that the Game Changer cap no longer applies.
- **Combo watch.** The two Kitchen Finks loops need a free sac outlet (Viscera Seer, Ashnod's Altar, or Altar of Dementia); Carrion Feeder left, so there are three. Mikaeus + Triskelion needs none unless Elesh Norn, Grand Cenobite is out. Sheoldred, Whispering One and Displacer Kitten don't complete an infinite on their own.
- **Where cut cards went.** To the binder: Cultivate, Massacre Wurm, Carrion Feeder, Gray Merchant of Asphodel, Sephiroth, Planet's Heir, Persist, Dovin's Veto, Blossoming Sands, Unmarked Grave, Phyrexian Metamorph, Deadly Dispute, Evil Reawakened, Satyr Wayfinder, Wrath of God, Cryptolith Rite. To bulk: Orzhov Signet, Stinkweed Imp, Lash of the Balrog, Grisly Salvage, Llanowar Elves, Llanowar Wastes, Avacyn's Pilgrim.
- **Known simulator simplification.** The simulator models Muldrotha's body but not her graveyard casting.

## Sheoldred, the Apocalypse — Mono-Black Drain

Deck key `sheoldred`, `decklists/JD/sheoldred-mono-black.md`.

### Changes

- **2026-10-27.** Out: Painful Lesson. In: Grave Titan. JD's choice (2026-10-27), not simulated: Grave Titan (the retired Sephiroth FS deck's copy, from bulk) for Painful Lesson: a six-drop that makes two 2/2 Zombies on entering and attacking, blockers that protect the life total and bodies for the sacrifice outlets.
- **2026-10-27.** Out: Charcoal Diamond, Crawlspace, Melancholic Poet, Read the Bones, Skyscanner, Ulna Alley Shopkeep. In: Bolas's Citadel, Call of the Ring, Demonic Tutor, Gray Merchant of Asphodel, The One Ring, Zulaport Cutthroat. JD's choice (2026-10-27), not simulated: from the retired Sephiroth FS deck, The One Ring, Demonic Tutor, Gray Merchant of Asphodel, Call of the Ring and Bolas's Citadel; Zulaport Cutthroat from Seph/Atraxa. Four Game Changers (Ad Nauseam, The One Ring, Demonic Tutor, Bolas's Citadel): Bracket 3 -> Bracket 4.
- **2026-10-25.** Out: Stir Up Trouble. In: Ob Nixilis, the Fallen. JD's choice (10/25): Stir Up Trouble had the lowest win rate when cast at T3 and T4. Ob Nixilis drains 3 per land drop. Not simulated; the landfall ability isn't modeled yet (body only).
- **2026-10-18.** Out: Gray Merchant of Asphodel. In: Necrosquito. Band calibration 10/18 (JD approved): Gray Merchant of Asphodel=>Necrosquito
- **2026-10-17.** Out: Restless Bloodseeker // Bloodsoaked Reveler. In: Ad Nauseam. Ad Nauseam (JD's pick): +1.6 over t3-t5 (look-ahead, 1,000 paired per tier, noise; positive in every tier) once the shared Ad Nauseam AI and the look-ahead's hold-mana credit were fixed. The deck's first Game Changer.
- **2026-10-15.** Out: Apex Witchstalker. In: Crawlspace. Synced with JD's physical deck: Crawlspace is in the deck in place of Apex Witchstalker (no more than two creatures can attack you each combat). Apex Witchstalker goes to bulk. Not simulated.
- **2026-10-13.** Out: Champion's Helm. In: Carnage Altar. JD's choice: Champion's Helm goes to the Cloud deck; Carnage
  Altar comes back from bulk (it was in the deck until 10-11). Paired test, 500 games per tier: -0.6 (t1 -2.2, t2 -1.4,
  t3 +3.0, t4 -1.6, t5 -0.6); `audit/cloud/2026-10-13/screen/D3-*`. Her protection equipment is now Swiftfoot Boots
  (hexproof) and Darksteel Plate (indestructible).
- **2026-10-12.** Out: Temple Bell, Castle Locthwain. In: Otherworld Atlas, Mikokoro, Center of the Sea. JD's choice:
  neither of the first two could be found, so the replacements were bought instead ($1.68 and $4.54). The Atlas taps for
  a charge counter or to make each player draw one card per counter; Mikokoro makes each player draw for {2}. Both make
  opponents draw, so with Sheoldred out each card costs them 2. Measured +0.1 points over all tiers against the Bell and
  the Castle (no difference; look-ahead, loose, 1,000 paired games per tier; `audit/sheoldred/`). The buy list is 19
  cards, $35.61.
- **2026-10-11 (third change).** No change to the list: Phyrexian Arena comes from Sauron instead of being bought
  (Sauron measured +0.6 with Cost of Brilliance in its slot, within noise;
  `audit/sauron/2026-10-11-arena-cost-of-brilliance/`). The buy list is 19 cards, $40.39.
- **2026-10-11 (second change).** Out: Ribskiff, Locust Spray, Gravblade Heavy, Multiply by Zero, Mabel, Bitter Recluse,
  Moseo, Vein's New Dean, Burglar Rat, Fathom Fleet Cutthroat, Canyon Crawler, Necrosquito, Rampart Hunter, Callous
  Inspector, Depressurize, End of the Hunt, Tome of Legends, Diary of Dreams, Dark Endurance, Lash of the Balrog,
  Hullcarver, 2 Swamp. In: the two upgrade packages (see the buy list in the 2026-10-11 build entry), and Gix,
  Yawgmoth Praetor from Jodah (Lagrella, the Magpie refills it there). JD's choice: the packages measured +7.7 points
  (24.3% → 32.0%) and Gix +1.2 on top (within noise, leaning positive); `audit/sheoldred/`. The cuts go back to bulk,
  Lash of the Balrog to the binder.
- **2026-10-11.** Out: Carnage Altar. In: Memory Jar. JD's choice: Memory Jar moves from Jodah (Szadek, Lord of Secrets
  refills it there); with Sheoldred out it makes each opponent draw seven, 14 life each. Measured +2.9 points over all
  tiers (look-ahead, loose, 1,000 paired games per tier; `audit/sheoldred/`). Carnage Altar goes back to bulk.
- **2026-10-11 (added).** JD's mono-black deck around Sheoldred, the Apocalypse, taken from Sauron (who measured -2.1
  points without her; Bedevil from the binder took her slot). First built from the bulk and binder lists with no buys
  (20.9%), then given Memory Jar and Gix from Jodah and two upgrade packages (19 cards bought on 2026-10-11 for about
  $40, plus Phyrexian Arena from Sauron). Fully modeled; measured at about 32% over all tiers. Sourcing: Sheoldred and
  Phyrexian Arena from Sauron; Memory Jar and Gix from Jodah; 19 bought cards (cheapest printing on Scryfall,
  2026-10-11: Seizan, Underworld Dreams, Dark Deal, Painful Lesson, Howling Mine, Fate Unraveler, Peer into the Abyss,
  Geier Reach Sanitarium, Mind Stone, Commander's Sphere, Charcoal Diamond, Read the Bones, Ambition's Cost, Syphon
  Mind, Exsanguinate, Gray Merchant of Asphodel, Swiftfoot Boots, plus Mikokoro and Otherworld Atlas on 2026-10-12;
  total $35.61); the rest from JD's bulk (44) and binder (Phyrexian Delver); 32 Swamp his own. Bulk's Arcane Signet
  and Fellwar Stone are spare copies. Bracket 3, no Game Changers.

### Simulator history

- **2026-10-12.** 30.8% over all tiers (±0.5; rank 3 of the ten decks): T1 26.5%, T2 39.7%, T3 32.5%, T4 30.9%, T5
  24.4%. Look-ahead AI, loose profile, 6,720 games per tier, every deck on the same code;
  `decklists/pool/pool-results.md` section 0d.
- **2026-10-11.** Measured with look-ahead AI, loose profile, 1,000 paired games per tier
  (`audit/sheoldred/README.md`):

  | List | T1 | T2 | T3 | T4 | T5 | All tiers |
  |---|---|---|---|---|---|---|
  | Bulk only (Carnage Altar for Memory Jar) | 16.8% | 21.7% | 20.5% | 23.5% | 22.0% | 20.9% |
  | + Memory Jar | 18.1% | 27.1% | 24.9% | 27.5% | 23.9% | 24.3% |
  | + both upgrade packages | 26.4% | 42.7% | 33.4% | 32.2% | 25.3% | **32.0%** |
  | + Gix (on a second set of games: 30.7% → 31.9%) | | | | | | +1.2 |

  These come from paired runs, not the section 0d seeds of `decklists/pool/pool-results.md`, so they are close to but
  not directly comparable with that table (Sephiroth 35.9%, Kefka 26.6%). Traced games show the wins coming from the
  drain plan and from Exsanguinate. Two other Jodah blocks were tried and not taken, both within noise: Toxic Deluge +
  Bitter Triumph (+0.7) and Korlash, Heir to Blackblade (+0.4).

### Tuning notes

- Modeling: every card is Full. Sheoldred's drain fires on every opponent draw (draw steps, extra draws, wheels) and
  her lifegain on every card you draw. One known simplification: Seizan's and Underworld Dreams' triggers resolve at
  once, so opponents can't respond to them.
- The AI holds the symmetric draw cards (the Mine, the Atlas, Mikokoro, the Sanitarium, Seizan, the wheels) until a
  punisher is out.

## Titania, Protector of Argoth — Mono-Green Lands to the Graveyard

Deck key `titania`, `decklists/JD/titania-lands.md` (both removed).

**Retired 2026-10-19: became the Omnath deck** (key `omnath`, see the Omnath section below and `audit/omnath/FINDINGS.md`). Titania stays as a card in Omnath's 99; her landfall plan keeps working there. The entries below are the deck's history.

### Changes

- **2026-10-18.** Out: Environmental Scientist, Harmonize, Hornet Queen, Tireless Tracker. In: Cactarantula, Fblthp, Knows the Way, Paradise Druid, Worldly Tutor. Band calibration 10/18 (JD approved): Hornet Queen=>Cactarantula;Harmonize=>Fblthp, Knows the Way;Tireless Tracker=>Paradise Druid;Environmental Scientist=>Worldly Tutor
- **2026-10-13.** Out: Cactarantula, Elfhame Wurm, Penumbra Spider, Yavimaya Wurm. In: Cartographer's Survey, Kura, the
  Boundless Sky, Tireless Tracker, Vorinclex, Monstrous Raider. JD's package D (landfall value):
  Yavimaya Wurm, Elfhame Wurm, Cactarantula and Penumbra Spider out (to bulk); Vorinclex, Monstrous Raider (JD's own
  copy), Kura, the Boundless Sky, Tireless Tracker and Cartographer's Survey (binder, from the retired Jodah) in.
  Measured +1.8 over all tiers (T1 -3.2, T2 +4.2, T3 +0.4, T4 +4.8, T5 +2.8; look-ahead, loose, 500 paired games per
  tier; `audit/jd-packages/2026-10-13/`).
- **2026-10-11.** Added. JD's mono-green deck, built from the bulk and binder lists around Titania, Protector of Argoth
  (borderless Secret Lair printing, about $2.46), with no other buys apart from Arachnogenesis and Blossoming Defense
  (bought 2026-10-11). Chosen over a Bristly Bill landfall build of mostly the same cards, which measured 15.0%.
  Fully modeled and measured on 2026-10-11: 22.8% over all tiers (see Simulator history).

### Simulator history

- **2026-10-13.** The landfall package (Vorinclex, Kura, Tireless Tracker and Cartographer's Survey in; four green
  Wurms and Spiders out) measured **+1.8 over all tiers** in a paired test (T1 -3.2, T2 +4.2, T3 +0.4, T4 +4.8,
  T5 +2.8; look-ahead, loose, 500 games per list per tier; up in four of five tiers). A pool run was pending.
- **2026-10-12.** Previous list's figures: **22.6% over all tiers** (±0.4; rank 7 of the ten decks): T1 19.5%,
  T2 27.5%, T3 24.2%, T4 24.3%, T5 17.5%. Look-ahead AI, loose profile, 6,720 games per tier, every deck on the same
  code; `decklists/pool/pool-results.md` section 0d.
- **2026-10-11.** Measured with the look-ahead AI, loose profile, 960 games per tier, the same settings as the other
  decks (`decklists/pool/pool-results.md` section 0d, results in `audit/titania/`): T1 21.0%, T2 27.0%, T3 24.8%,
  T4 23.6%, T5 17.6%; **22.8% over all tiers**, near Veyran (23.7%). Wins come late (turn 13.9 on average); she is
  often removed and recast, and each recast returns a fetch land for another 5/3.

### Tuning notes

- **Cheap upgrades, if wanted later** (about $1.80, not in the list): Myriad Landscape, Escape Tunnel, Harrow,
  Springbloom Druid and Splendid Reclamation, each one more land to the graveyard (Splendid Reclamation returns
  them all).
- **Known simulator simplifications:** several lands going to the graveyard at once make one 5/3, not one each; day and
  night (Infestation Expert) is modeled only as far as it changes that card.
- **Consistency note, 2026-10-11:** the 38-land base (35 Forest plus the three fetch lands) is an exception to JD's
  basics-over-tapped-lands rule.

## Tymna the Weaver & Kraum, Ludevic's Opus — Four-Colour Partners

Deck key `kraum` (removed), `decklists/JD/tymna-kraum-partners.md` (removed).

**Retired 2026-10-19: became Tymna/Thrasios** (key `thrasios`, see the Tymna/Thrasios section below and `audit/thrasios/FINDINGS.md`). The entries below are the deck's history.

### Changes

- **2026-10-16 (night).** JD doesn't own Toxic Deluge, Masterful Flourish, Moseo, Vein's New Dean or Shattered Sanctum
  (removed from the binder and bulk lists). Out: those four. In: Austere Command, Dark Endurance, The Wasp, Winsome
  Avenger and a Swamp (all from bulk). Measured +1.8 at T3 (2,000 paired games, ±2.7): no loss.
- **2026-10-16 (evening).** Tuned toward a solid Tier 3 deck (JD: 30% at T3, up to $50 of upgrades; Gilgamesh,
  Diabolic Tutor, Shield and Helm of Kaldra, Toxic Deluge and Fact or Fiction kept). Eleven rounds of paired tests at
  T3 (`audit/kraum/FINDINGS.md`). Out: Abigale, Poet Laureate, Arcane Sanctum, Boiling Rock Prison, Caparocti
  Sunborn, Chromatic Lantern, Crumbling Necropolis, Dissipate, Dragoon's Lance, Exit Specialist, Fate of the
  Sun-Cryst, Force of Will, Garland, Royal Kidnapper, Harsh Annotation, Hero's Blade, Hero's Heirloom, Irrigated
  Farmland, Laelia, the Blade Reforged, Make Your Move, Medic's Kitesail, Mystic Monastery, Nim Deathmantle, Nomad
  Outpost, Path of Ancestry, Photon, Mighty Marvel, Potioner's Trove, Quick Study, Samurai's Katana, Scorching
  Dragonfire, Silverquill Charm, Spectacle Summit, Spectral Lynx, Star Compass, Talisman of Hierarchy, Temple of
  Deceit, Temple of Epiphany, Temple of Malice, The Aetherspark, The Wasp, Winsome Avenger, Trailblazer's Boots,
  Urza, Powerstone Prodigy, Vanish from Sight, Vibrant Outburst, Vivid Creek, Wander Off. In: Adeline, Resplendent
  Cathar, Arcane Signet, Archon of Absolution, Baird, Steward of Argive, Baneslayer Angel, Crawlspace, Dauthi
  Voidwalker, Dragonskull Summit, Dream Trawler, Drowned Catacomb, Elspeth, Sun's Champion, Fellwar Stone, Ghostly
  Prison, Glorybringer, Grave Titan, Hero of Bladehold, Isolated Chapel, Lightning Greaves, Masterful Flourish,
  Orzhov Signet, Path to Exile, Propaganda, Restoration Angel, Rip Apart, Rune-Scarred Demon, Serra Angel, Skyclave
  Apparition, Sulfur Falls, Sunblast Angel, Sun Titan, Swords to Plowshares, Thundermaw Hellkite, Vampire Nighthawk,
  Windborn Muse. Basics: Plains 3 → 7, Island 3 → 6, Swamp 3 → 5, Mountain 2 → 3 (38 lands, was 36). Force of
  Will out: no Game Changers left. Upgrade buys about $48.65.
- **2026-10-16.** Brought back as a deck, with Gilgamesh, Master-at-Arms, Diabolic Tutor and an Equipment package.
  Starts from the 2026-10-13 comparison build (dropped 2026-10-14). Out: Adarkar Wastes, Shivan Reef, Sulfurous
  Springs, Bitter Triumph, Dakkon, Shadow Slayer (all five now in other decks), Ultimate Magic: Holy, Shardmage's
  Rescue, Reconnaissance Mission, Razia, Boros Archangel, Mordenkainen, Desertion, Dakkon Blackblade, Sol'kanar the
  Swamp King (the eight lowest win-rate-when-cast cards in all five tiers of the 2026-10-13 runs), Twisted Fates, Mana
  Leak. In: Irrigated Farmland, Spectacle Summit, Boiling Rock Prison, Diabolic Tutor, Gilgamesh, Master-at-Arms,
  Conqueror's Flail, Trailblazer's Boots, Medic's Kitesail, Nim Deathmantle, Dragoon's Lance, Samurai's Katana,
  Hero's Heirloom, Hero's Blade, Shield of Kaldra, Helm of Kaldra (the last two from the Cloud deck; Sword of Kaldra
  stays there). Buys: Kraum, Gilgamesh, Diabolic Tutor.
- **2026-10-13.** Built as the Kraum comparison for the Tymna deck from the retired Jodah deck's cards (40 of them).

### Simulator history

- **2026-10-16 (evening).** The tuned list scored **31.2% at T3** (4,000 games, look-ahead, loose, ±1.4;
  `audit/kraum/2026-10-16-iter/L8-analyze-t3.txt`), up from 12.6% for the morning's list. The gains, in paired tests:
  attack taxes +8.5, a lower curve with protection for Tymna +3.3, finishers +3.4, more mana and lands +3.1 and +2.1.
- **2026-10-16.** The new build scored **12.6% over t1–t5** (T1 12.7, T2 12.3, T3 12.6, T4 13.4, T5 12.1; 1,000
  games per tier, look-ahead, loose, `--any-tier` with JD's OK; `audit/kraum/2026-10-16/`). Below the fair band in
  every tier. Tymna hits only 0.5–0.7 opponents per turn and Kraum lands on turn 6.3. Findings in
  `audit/kraum/FINDINGS.md`.
- **2026-10-13.** The comparison build scored **15.3% over T1–T5** (T1 12.3, T2 16.5, T3 17.5, T4 14.3, T5 16.1;
  1,000 games per tier, look-ahead, loose; `audit/tymna/2026-10-13-fix/` on branch `tymna-thrasios`), against 10.1%
  for the Thrasios build.

## Veyran, Voice of Duality — Izzet Spellslinger

Deck key `veyran`, `decklists/JD/veyran-izzet-spellslinger.md`.

### Changes

- **2026-10-18.** Out: Evacuation, Venser, Shaper Savant. In: Consecrated Sphinx, Mystical Tutor. Band calibration 10/18 (JD approved): Evacuation=>Consecrated Sphinx;Venser, Shaper Savant=>Mystical Tutor
- **2026-10-13.** Out: Jeska's Will. In: Flux Channeler. JD's choice: Jeska's Will moves to Kefka; Flux Channeler (from Sauron) takes the slot. Measured -2.1 over all tiers (T1 -3.9, T2 -4.5, T3 -0.4, T4 -0.9, T5 -0.8; look-ahead, loose, 1,000 paired games per tier; `audit/sauron/2026-10-12-weakness/sofi-veyran/`). No Game Changers left. The deck lost some big-turn fuel (Jeska's Will gave four to seven mana and three more spells for one card) and gained a cheap Wizard body with a proliferate trigger.
- **2026-10-12.** Out: Return the Favor (to bulk). In: Triple Triad (new). JD's choice: each upkeep, every player exiles their top card; you may play yours and every cheaper one free, and free instants and sorceries still trigger Veyran. A partial test (Tiers 1-3, 1,000 paired games each: +0.5, +0.9, -2.1, all within noise) was stopped early.
- **2026-10-11 (trade with Kefka, moved as functional blocks).** Out to Kefka: Deflecting Swat, Slip Out the Back, Underworld Breach, Niv-Mizzet, the Firemind, Jin-Gitaxias, Progress Tyrant, Steam Vents, Sulfur Falls, Spirebluff Canal, Stormcarved Coast. In: Mystic Confluence and Splatter Technique (from Kefka), Return the Favor, Spell Pierce and Lightning Strike (from bulk), 2 Island and 2 Mountain. Paired look-ahead test, 1,000 games per tier: **+1.7 points** (22.9% -> 24.5%). One Game Changer now (Jeska's Will). Veyran refilled only from Kefka's cuts and the bulk list; Kefka gained 1.3 points from the same trade (28.1% -> 29.4%), so both decks came out ahead. Audit: `audit/kefka-veyran/2026-10-11-trade/`.

  | Out (to Kefka) | In | What it does |
  | --- | --- | --- |
  | Deflecting Swat | Return the Favor (bulk) | A free redirect -> a copy-or-redirect for {R}{R} plus {1} per mode; copies are magecraft triggers. |
  | Slip Out the Back | Spell Pierce (bulk) | A one-mana phase-out -> a one-mana soft counter. |
  | Underworld Breach | Lightning Strike (bulk) | The graveyard rebuy (a Game Changer) -> a two-mana instant burn spell. |
  | Niv-Mizzet, the Firemind | Mystic Confluence (from Kefka) | A six-mana draw-to-damage body -> a six-mana instant that picks three modes (counter unless {3} is paid, bounce a creature, draw a card), usually draw three at an opponent's end step. |
  | Jin-Gitaxias, Progress Tyrant | Splatter Technique (from Kefka) | A seven-mana copy-and-tax body -> draw four, or 4 damage to each creature and planeswalker. |
  | Steam Vents, Sulfur Falls, Spirebluff Canal, Stormcarved Coast | 2 Island, 2 Mountain | Kefka needed the blue more; colours stay balanced here. |

  (Niv-Mizzet, the Firemind went to Kefka, where it is a second kill with Kefka's flipped side.)
- **2026-10-11.** Out: Cyclonic Rift. In: Evacuation. Matched to JD's physical list: Cyclonic Rift is in Sauron, so Veyran plays Evacuation; two Game Changers now (Jeska's Will, Underworld Breach).
- **2026-10-06.** Out: Mystical Tutor. In: Underworld Breach. Every escape is a cast, so the pingers, token makers and Storm-Kiln Artist trigger again, doubled by Veyran. Still three Game Changers (Bracket 3).
- **2026-10-05.** Out: Crawlspace. In: Propaganda. Crawlspace -> Propaganda, the planned swap once Propaganda was found (it tested ahead of Crawlspace in the 10-04 rebuild).
- **2026-10-04 (second change).** Out: Twinflame. In: Crawlspace. No more than two creatures can attack you each combat, against the creature decks that cause most of the deck's losses. A stand-in: Propaganda tested better and took this slot once found. (Twinflame, half of the Dualcaster Mage infinite, left here.)
- **2026-10-04 (the tokens rebuild, 10 out, 10 in).** Out: Gandalf, Friend of the Shire, Mystic Confluence, Pongify, Reality Shift, Return the Favor, Spell Pierce, Swiftfoot Boots, Thought Vessel, Thunderdrum Soloist. In: Deflecting Swat, Docent of Perfection // Final Iteration, Fiery Emancipation, Lightning Greaves, Slip Out the Back, Storm-Kiln Artist, Talrand, Sky Summoner, Third Path Iconoclast, Young Pyromancer (plus Crawlspace for Twinflame, above). The tokens rebuild: spells now leave creatures behind, Fiery Emancipation triples every ping, and the protection package is Deflecting Swat, Lightning Greaves and Slip Out the Back. New singles: Crawlspace, Deflecting Swat, Young Pyromancer, Third Path Iconoclast, Talrand, Storm-Kiln Artist, Docent of Perfection, Fiery Emancipation, Lightning Greaves; Slip Out the Back came from the binder; the cut cards went to the binder.

  | Out | In | What it does |
  | --- | --- | --- |
  | Return the Favor | Deflecting Swat | A copy-or-redirect for {R}{R} -> a free redirect while you control a commander. |
  | Mystic Confluence | Lightning Greaves | Five-mana flexibility -> zero-cost shroud and haste for Veyran. |
  | Swiftfoot Boots | Slip Out the Back | Greaves takes the equipment slot; the one-mana phase-out also dodges board wipes. |
  | Spell Pierce | Young Pyromancer | A soft counter -> a 1/1 per instant or sorcery, doubled by Veyran. |
  | Pongify | Third Path Iconoclast | Spot removal -> a 1/1 artifact Soldier per noncreature spell. |
  | Reality Shift | Talrand, Sky Summoner | Spot removal -> a 2/2 flying Drake per instant or sorcery. |
  | Thunderdrum Soloist | Storm-Kiln Artist | A pinger -> a Treasure per instant or sorcery cast or copied. |
  | Gandalf, Friend of the Shire | Docent of Perfection | A utility Wizard -> a 5/4 flier that makes Wizard tokens; flips into Final Iteration, which pumps them. |
  | Thought Vessel | Fiery Emancipation | A rock -> triple damage from every source you control. |
  | Twinflame | Crawlspace | Copies of your engines -> no more than two creatures can attack you each combat. A stand-in: Propaganda tested better and takes this slot once found. |
- **2026-10-02.** Out: Hydro-Channeler. In: Alania, Divergent Storm. Your first instant and your first sorcery each turn are copied, each copy costing an opponent a free card, and Veyran and Harmonic Prodigy each add a copy. Not a Game Changer; the deck stays at 100 and three GCs (Bracket 3).
- **2026-09-27.** Out: Force of Will, Reenact the Crime. In: Jeska's Will, Twinflame. Jeska's Will takes Force of Will's Game Changer slot, so the deck stays at three. Free protection -> fuel: mana plus three cards for the big turn. In the simulator the trade came out about even (+1.5 points with the fast AI, -0.7 with look-ahead, both within noise). Reenact the Crime (a conditional recast) -> Twinflame (hasty copies of your pingers and value creatures for a turn).
- **2026-09-26 (second pass).** Out: Disdainful Stroke. In: Unsummon. A narrow counter -> a one-mana bounce (tempo, or a save for your own creature).
- **2026-09-26.** Seven swaps: Displacer Kitten -> Niv-Mizzet, Parun; Kylox, Visionary Inventor -> Niv-Mizzet, the Firemind; Burning Prophet -> Kessig Flamebreather; Muse Seeker -> Imperial Recruiter; Dualcaster Mage -> Thousand-Year Storm; Disruptor Flute -> Swiftfoot Boots; Sleight of Hand -> Visions of Beyond. Losing Displacer Kitten ends the Kitten + Blazing Firesinger infinite: the deck no longer has a two-card combo, and Aetherflux Reservoir is now a big-turn finisher rather than an infinite kill. None of the new cards is a Game Changer; the deck stays at 100, 34 lands, and three GCs (Bracket 3, at the cap). New singles from here: Niv-Mizzet, Parun; Niv-Mizzet, the Firemind; Kessig Flamebreather; Imperial Recruiter; Thousand-Year Storm; Swiftfoot Boots; Visions of Beyond.

  | Out | In | What it does |
  | --- | --- | --- |
  | Displacer Kitten | Niv-Mizzet, Parun | Ends the Firesinger infinite; Parun turns every instant/sorcery (anyone's) into a card and a ping. |
  | Kylox, Visionary Inventor | Niv-Mizzet, the Firemind | Swaps a token-sacrifice free-spell turn for a draw-to-damage engine that stacks with Parun. |
  | Burning Prophet | Kessig Flamebreather | Scry-on-cast body -> a third each-opponent pinger, doubled by Veyran. |
  | Muse Seeker | Imperial Recruiter | A draw body -> a creature tutor for the pingers, Archmage Emeritus, or Harmonic Prodigy. |
  | Dualcaster Mage | Thousand-Year Storm | Value-only copy body -> a turn-long copy engine. |
  | Disruptor Flute | Swiftfoot Boots | Proactive tax -> hexproof and haste for Veyran. |
  | Sleight of Hand | Visions of Beyond | Cantrip -> cantrip that becomes draw-three late. |
- **2026-09-22.** Scavenger Grounds -> Stormcarved Coast.
- **2026-09-21.** Terminal Moraine -> Spirebluff Canal.
- **2026-09-18 / 09-19 / 09-20.** Earlier passes (trim to 100, Ral + Aetherflux, Force of Will + Steam Vents, tutor guide).
- **2026-09-17.** Built.

### Simulator history

- **2026-10-13.** **-2.1 over all tiers** against the previous list in a paired test (T1 -3.9, T2 -4.5, T3 -0.4, T4 -0.9, T5 -0.8; look-ahead AI, loose profile, 1,000 paired games per tier; `audit/sauron/2026-10-12-weakness/sofi-veyran/`). The previous list's pool figure, not this list's, was 23.4% over all tiers (±0.4; rank 6 of the ten decks; 6,720 games per tier, `decklists/pool/pool-results.md` section 0d).
- **2026-10-12.** Triple Triad partial test (Tiers 1-3, 1,000 paired games each: +0.5, +0.9, -2.1, all within noise), stopped early.
- **2026-10-11 (trade with Kefka).** Look-ahead AI, loose profile, 1,000 paired games per tier; `audit/kefka-veyran/2026-10-11-trade/`:

  | Tier | Before | After | Change |
  | --- | --- | --- | --- |
  | T1 | 25.1% | 29.7% | +4.6 |
  | T2 | 28.6% | 28.5% | -0.1 |
  | T3 | 22.8% | 27.8% | +5.0 |
  | T4 | 22.6% | 23.7% | +1.1 |
  | T5 | 15.2% | 12.9% | -2.3 (noise ±2.9) |
  | All | 22.9% | 24.5% | **+1.7** |

  Kefka gained 1.3 points from the same trade (28.1% -> 29.4%).
- **2026-10-04 (tokens rebuild).** Loose profile, paired games against all five opponent tiers:

  | List | Adaptive (1,000 per tier) | Look-ahead (240 per tier) |
  | --- | --- | --- |
  | These nine swaps plus Fellwar Stone -> Birgi, God of Storytelling | +3.0 points | +3.3 (+5.0, +3.8, +3.3, +2.5, +2.1) |
  | These nine swaps (Fellwar Stone kept) | +2.2 | not run |
  | A middle list keeping Spell Pierce, Pongify and Reality Shift | +3.7 | +0.9 |
  | A storm rebuild (rituals, cantrips, Grapeshot, Galvanic Iteration) | +1.1 | -2.4 |
  | Then Twinflame -> Propaganda (attack tax of {2}) | +2.2 (5,000 per tier) | +1.2 |

  The tokens list was the only one ahead at every tier under both AIs. The engine (Veyran plus a payoff) came online in 8-13 points more games under look-ahead. Birgi was worth about +0.8 points and was left out to save a purchase. Tokens turned out to be poor blockers: in the simulator Veyran had untapped tokens in only about a quarter of the attacks that killed it. Of the attack deterrents, Propaganda tested well, Aetherize was rarely castable (Veyran is usually tapped out on opponents' turns), and Crawlspace was chosen without a full test. Veyran is removed about once a game in the simulator; most of the deck's losses are to creature decks (Lord Windgrace, Brago, Kaalia, Sythis, Aurelia).
- **2026-09-27.** Force of Will -> Jeska's Will: +1.5 points with the fast AI, -0.7 with look-ahead, both within noise.

### Tuning notes

- **Weakest remaining slots:** Visions of Beyond (a one-card cantrip for most of the game; the twenty-card graveyard is rare here, since Emeritus of Ideation and Eris both want the yard) is the softest spell. The curve is still top-heavy, so a cheap body or rock back in for a six-drop is worth simming. The 10-11 trade brought Spell Pierce and Mystic Confluence back, so there are five counters. The soft spot it left is T5 (-2.3, within noise): against the strongest tables, Deflecting Swat and Breach may have mattered more.
- **Game Changers.** None. Three would still fit Bracket 3; a fourth moves the deck to Bracket 4.
- **Rules notes carried forward.** Veyran has no first strike, and magecraft gives +1/+1 until end of turn, not permanent counters. The doubling applies only when your casting or copying an instant or sorcery is what caused the trigger.
- **Cut cards (to the binder):** Return the Favor, Mystic Confluence, Swiftfoot Boots, Spell Pierce, Pongify, Reality Shift, Thunderdrum Soloist, Gandalf, Friend of the Shire, Thought Vessel, Twinflame (10-04); Disdainful Stroke, Reenact the Crime; from 09-26, Burning Prophet, Dualcaster Mage, Kylox, Muse Seeker, Disruptor Flute, Sleight of Hand. Crawlspace went to bulk. Birgi, God of Storytelling was tested (+0.8) and not bought.

## Y'shtola, Night's Blessed — Esper Drain

Deck key `yshtola`, `decklists/JD/yshtola-esper-drain.md`.

### Changes

- **2026-10-27.** Out: Evolving Wilds, Exotic Orchard, Fabled Passage, Glacial Fortress, Isolated Chapel, Prairie Stream, Shattered Sanctum, Spire of Industry, Terramorphic Expanse. In: Flooded Strand, Godless Shrine, Hallowed Fountain, Marsh Flats, Polluted Delta, Scrubland, Tundra, Underground Sea, Watery Grave. Mana upgrade (JD): 9 slow, conditional and basic-fetch lands out for three original duals, three shocks and three fetches. JD's choice, not simulated.
- **2026-10-19.** Out: Curiosity. In: Orcish Bowmasters. Curiosity -> Orcish Bowmasters: +2.5 at T5 (loose, 2,000 games per list, noise ±2.3), the first Y'shtola swap past the noise. Bowmasters is cast in 81% of the games it is drawn (Curiosity 68%), damage +2.3 a game, plan online +2.4 points; the ping and the Orc Army pick off fast decks' mana creatures. JD buys a second copy (the Sauron copy stays).
- **2026-10-18.** Out: Bolas's Citadel, Dakkon, Shadow Slayer, Thought Vessel. In: Farewell, Force of Will, Teferi's Protection. Band calibration 10/18 (JD approved): Dakkon, Shadow Slayer=>Force of Will;Thought Vessel=>Farewell;Bolas's Citadel=>Teferi's Protection
- **2026-10-14.** Out: Triplicate Spirits. In: Adarkar Wastes, Dakkon, Shadow Slayer, Underground River. Triplicate Spirits (to bulk) for Dakkon, Shadow Slayer; a Swamp and an Island for Underground River and Adarkar Wastes (all three from the binder, the retired Jodah deck's leftovers). Measured -0.7 over all tiers (T1 +0.2, T2 -4.2, T3 -1.8, T4 +2.2, T5 0.0; audit/leftovers/2026-10-14/README.md; look-ahead, loose, 500 paired games per tier); JD's choice.
- **2026-10-11.** Out: Bastion Protector. In: Thought Vessel (from Sauron). JD's choice: no maximum hand size for the deck that draws the most each turn (Necropotence, Consecrated Sphinx, Notion Thief, Bolas's Citadel); Bastion Protector was a creature, so it never triggered Y'shtola. Not measured.
- **2026-10-09.** Out: Secure the Wastes. In: Venat, Heart of Hydaelyn // Hydaelyn, the Mothercrystal. JD's choice: Venat draws a card the first time each turn you cast a legendary spell; its flip side ({7}: exile a nonland permanent) makes a creature indestructible each turn. Measured +0.1 ±0.8 for Secure the Wastes (adaptive, paired, 1,000 games per tier). Not a Game Changer.
- **2026-10-07 (second).** Out: Crux of Fate. In: Grand Arbiter Augustin IV. Your white and blue spells cost {1} less each, opponents' spells {1} more. Crux was stuck in hand in about six games in ten. Seven Game Changers (Bracket 4). Tested at −0.6 points (fast AI, 1,000 pairs per tier), within noise.
- **2026-10-07.** Out: Statute of Denial. In: Polluted Bonds. A five-mana enchantment that triggers her when cast, then drains an opponent 2 (and gains you 2, feeding Sanguine Bond and Marauding Blight-Priest) on every land they play. Statute was stuck in hand in about three quarters of the games it was drawn.
- **2026-10-06.** Out: Vanquish the Horde. In: Mystical Tutor. Finds Exsanguinate or Debt to the Deathless at the end of an opponent's turn. Vanquish was her most redundant reset (Austere Command, Crux of Fate and Massacre Wurm stayed). Six Game Changers (Bracket 4).
- **2026-10-05.** Out: Enslave, Jester's Cap, Plea for Guidance. In: Bolas's Citadel, Consecrated Sphinx, Smothering Tithe (Enslave -> Consecrated Sphinx, Plea for Guidance -> Bolas's Citadel, Jester's Cap -> Smothering Tithe). Three cards that sat in hand most games out for three Game Changers: the deck moves to Bracket 4 with five. Tested: +1.2 points on average, ahead in all five tiers (1,000 paired games per tier, adaptive AI); Smothering Tithe's Treasures ease the mana the deck is short of.
- **2026-10-04.** Out: Arcane Sanctum, Dimir Guildgate, Irrigated Farmland, Scoured Barrens, Temple of Deceit, Temple of Enlightenment, Temple of Silence, Vivid Creek, Vivid Marsh, Vivid Meadow. In: 4 Island, 3 Plains, 3 Swamp. The ten lands that always entered tapped became basics. Tested: about +1.2 points on average (1,000 paired games per tier, adaptive AI), positive in four of five tiers; Y'shtola wants to land on turn 3 or 4 with a spell to follow, and no land now always enters tapped. Tapped lands were the biggest cost. To add fixing back without tapped lands: Drowned Catacomb, Godless Shrine, Watery Grave, Hallowed Fountain.
- **2026-10-03.** Out: Clever Concealment, Disdainful Stroke, Momentary Blink, Rootborn Defenses, Take Up the Shield. In: Cast Away Doubt, Memory Trap, Static Net, Statute of Denial, Your Fate Ends Here. Five rarely cast cards (each cast in 5% or less of the games it was drawn, or mana value 2) out for spells of mana value 3 or more that trigger Y'shtola: Static Net (exile, 2 life, a Powerstone), Cast Away Doubt (draw two; with her trigger each opponent loses 4), Your Fate Ends Here (instant removal), Memory Trap (exile) and Statute of Denial (a counter that triggers her). The protection instants (Take Up the Shield, Rootborn Defenses, Clever Concealment, Momentary Blink) were rarely cast, and removal does more against the creature decks that beat this one.
- **2026-10-03 (added).** In the simulator as deck key `yshtola` (`commander-sim --deck yshtola --pool t3`) and in practice mode. Every card is modeled. Zur the Enchanter is in the 99.

### Simulator history

- **2026-10-14.** The leftovers swap (Triplicate Spirits for Dakkon, Shadow Slayer; a Swamp for Underground River; an Island for Adarkar Wastes) measured -0.7 over all tiers, within noise, in a paired test (T1 +0.2, T2 -4.2, T3 -1.8, T4 +2.2, T5 0.0; look-ahead, loose, 500 games per list per tier); JD chose to apply it. A pool run was pending. Previous list's figures: 41.4% over all tiers on 2026-10-12 (±0.5; rank 1 of the ten decks): T1 45.4%, T2 55.1%, T3 44.8%, T4 39.2%, T5 22.3%, 6,720 games per tier, every deck on the same code; `decklists/pool/pool-results.md` section 0d.
- **2026-10-07 to 2026-10-09 tests.** Crux of Fate -> Augustin: -0.6 (fast AI, 1,000 pairs per tier). Secure the Wastes -> Venat: +0.1 ±0.8 (adaptive, paired, 1,000 games per tier).
- **2026-10-05.** Enslave / Plea for Guidance / Jester's Cap -> Sphinx / Citadel / Tithe: +1.2 on average, ahead in all five tiers (1,000 paired games per tier, adaptive AI). 2026-10-04 basics change: about +1.2, positive in four of five tiers (same settings).
- **2026-10-03.** Measured with the look-ahead AI against the loose profile (opponents counter and remove freely: the worst case), 120 games per tier, the same settings as the other recent decks in `decklists/pool/pool-results.md`. An even share is 25%. This was the list before the 10-03 change.

  | Tier | Y'shtola | Galadriel (same settings) |
  | --- | --- | --- |
  | T1 High B2 / Low B3 | **31.7%** (24-40) | 15.0% |
  | T2 Mid B3 | **50.8%** (42-60) | 11.7% |
  | T3 High B3 | **37.5%** (29-46) | 8.3% |
  | T4 Low B4 | **35.8%** (28-45) | 12.5% |
  | T5 High B4 | 14.2% (9-22) | 8.3% |

  Above an even share through Low Bracket 4, below it against High Bracket 4. Tier 2 is the standout, at about half of all games, level with Sephiroth. Against Tier 5 the fast combo decks (Kinnan, Urza, Yawgmoth) win before the drain gets there.

  How it wins (a 400-game breakdown against Tier 3 with the faster heuristic AI, which wins 39% there):
  - The damage it deals: 46 to opponents per game. 47% comes from Y'shtola's and Ill-Gotten Inheritance's triggers, 31% from drains (Sanguine Bond, Blight-Priest, the X spells, Syphon-Mage) and only 21% from combat.
  - The kills: 55% by drain, 23% by triggers, 21% by combat.
  - When the engine comes online: Y'shtola's first trigger lands by turn 6 in 43% of games and by turn 8 in 65% of games. She triggers at some point in 86% of games.
  - The finisher: Debt to the Deathless is the card most tied to winning. The deck wins 75% of the games it's cast in.
  - How it loses: almost always to combat (Aurelia above all), not to combo. Propaganda and Windborn Muse are worth casting early against creature decks.

  Cards that sat in hand: Rootborn Defenses, Clever Concealment, Take Up the Shield and Momentary Blink were cast in 5% or less of the games they were drawn. They only come out in answer to removal or a wipe, and the AI rarely needed them. They were replaced on 10-03 (with Disdainful Stroke) by Static Net, Cast Away Doubt, Your Fate Ends Here, Memory Trap and Statute of Denial.

### Tuning notes

- **Tapped lands** were the biggest cost and are gone (about +1.2 points in testing). Four duals still sometimes enter tapped (Glacial Fortress, Isolated Chapel, Prairie Stream, Shattered Sanctum); with 24 basics they rarely do.
- **Two triggers a turn is the target.** Her draw needs 4 life lost by one player, which one trigger alone never does. Cheap instants with mana value 3 (Anguished Unmaking, Generous Gift, Your Fate Ends Here) on an opponent's turn are the easiest second trigger. Cast Away Doubt does it alone: its 2 damage plus her 2 is 4 to each opponent.
- **Weakest slots for this commander.**
  - Fatal Push, Path to Exile, Dovin's Veto: good cards, but mana value 1 or 2 doesn't trigger Y'shtola.
  - Venat, Heart of Hydaelyn: a creature, so no trigger, and its draw needs legendary spells, which the deck has few of. JD's pick (measured +0.1, neutral).
  - Restoration Angel: a creature, so no trigger (Bastion Protector, the other, left on 10-11 for Thought Vessel).
- **Replacements worth testing.**
  - Removal with mana value 3: Mortify (instant) or Vindicate (sorcery).
  - More lifegain payoffs: Vito, Thorn of the Dusk Rose; Exquisite Blood. Exquisite Blood with Sanguine Bond is an infinite combo and moves the deck toward Bracket 4 on combos.
- **Life is still a resource.** Necropotence, Read the Bones, Night's Whisper, Anguished Unmaking, Tezzeret's Gambit, Deep Analysis's flashback, Caves of Koilos, Adarkar Wastes, Underground River, Spire of Industry and Talisman of Progress all cost life. The deck gains it back steadily: 2 per Y'shtola trigger, plus every drain.
- **Mystic Remora's upkeep grows.** Pay it for a turn or two at most, then let it go.

## Alela, Artful Provocateur — Esper Faerie Flyers

Deck key `alela`, `decklists/Avery/alela-esper-faeries.md`.

### Changes

- **2026-10-15.** Synced with Avery's physical deck (list given by JD). Out: Aether Hub, Ajani Fells the Godsire, Aven
  Fogbringer, Exotic Orchard, Fatehold Annex, Fatehold Charm, Hymn to Tourach, Initiates of the Ebon Hand, Keeper of
  the Quiet Hour, Opposition, Page, Loose Leaf, Ray of Command, Rhystic Study, Shattered Sanctum, Sphinx's Approach,
  Spite // Malice, Static Prison, Strixhaven Skycoach, Tainted Field, Tetsuko Umezawa, Fugitive, Tymna the Weaver,
  Vanquish the Weak. In: Angelic Page, Angelic Shield, Avenger en-Dal, Burnished Hart, Cabal Patriarch, Credit Voucher,
  Crystal Chimes, Fabricate, Gravedigger, Igneous Golem, Lightning Greaves, Mirage Mirror, Path to Exile, Scrapheap,
  Serra the Benevolent, Skullclamp, Swords to Plowshares, Tragic Poet, Wall of Junk. Basics: Plains 9 → 11, Swamp
  7 → 8 (34 lands, was 36). Game Changers down to one (Notion Thief). Not measured yet. All seven new cards the simulator
  did not fully model (Credit Voucher, Mirage Mirror, Scrapheap, Angelic Page, Cabal Patriarch, Serra the Benevolent,
  Wall of Junk) are now modeled; Crystal Chimes was fixed to return every enchantment card (it returned one).
- **2026-10-14.** Out: Gravedigger. In: Exotic Orchard, Shattered Sanctum, Tymna the Weaver. Gravedigger (back to Avery)
  for Tymna the Weaver; two Plains for Exotic Orchard and Shattered Sanctum (all three JD's, from his binder, the
  retired Jodah deck's leftovers). Measured +0.1 over all tiers (T1 +1.2, T2 +2.2, T3 -0.6, T4 +1.4, T5 -3.8;
  `audit/leftovers/2026-10-14/README.md`; look-ahead, loose, 500 paired games per tier). Within noise.
- **2026-10-05.** Added. In the simulator as deck key `alela` (`commander-sim --deck alela --pool t3`) and in practice
  mode.

### Simulator history

- **2026-10-15.** The new list (synced with Avery's physical deck, every card modeled) scored **9.8% against T1**
  (4,000 games, look-ahead, loose; `audit/alela/2026-10-15-newlist/analyze-t1.txt`). The old list scored 12.0% on
  10/14, but on older engine code, so the difference is not a clean list effect. Alela resolves in 79.7% of games
  (was 85.3%) and by turn 5 in 45% (was 53%): two fewer lands and two fewer mana creatures. Findings in
  `audit/alela/FINDINGS.md`.
- **2026-10-14.** The leftovers swap (Gravedigger for Tymna the Weaver; two Plains for Exotic Orchard and Shattered
  Sanctum) measured **+0.1 over all tiers, within noise,** in a paired test (T1 +1.2, T2 +2.2, T3 -0.6, T4 +1.4,
  T5 -3.8; look-ahead, loose, 500 games per list per tier). A pool run is pending.
- **2026-10-12.** Previous list's figures: **13.1% over all tiers** (±0.4; tied with Galadriel for 9th of the ten
  decks): T1 12.1%, T2 14.8%, T3 11.7%, T4 14.9%, T5 11.8%, 6,720 games per tier, every deck on the same code;
  `decklists/pool/pool-results.md` section 0d.

### Tuning notes

- **Bench (in bulk if you want to tune):** Unbreakable Formation, Cho-Manno's Blessing, Medic's Kitesail, Twisted Fates,
  Silence the Echo, Aven Fisher, Sanguine Glorifier, Way of the Mentor, Way of the Cryomancer, Rewrite Regrets,
  Mnemonic Wall, Bonders' Enclave.
- Gravedigger left on 2026-10-14 (it goes back to Avery).

## Galadriel, Light of Valinor — Bant Rebels

Deck key `galadriel`, `decklists/Avery/galadriel-bant-rebels.md`.

### Changes

- **2026-10-18.** Out: Dragonlord Dromoka. In: Flicker. Matched to the physical deck (Avery's list, 2026-10-18): Flicker back in for Dragonlord Dromoka; the 10/14 swap was never made physically.
- **2026-10-14.** Out: Flicker. In: Dragonlord Dromoka. Flicker (back to Avery) for Dragonlord Dromoka (JD's, from his
  binder, the retired Jodah deck's leftovers). Measured +1.1 over all tiers (T1 +0.8, T2 +3.0, T3 -0.8, T4 +2.2,
  T5 +0.4; `audit/leftovers/2026-10-14/README.md`; look-ahead, loose, 500 paired games per tier). Within noise.
- **2026-10-02.** Added. In the simulator as deck key `galadriel` (`commander-sim --deck galadriel --pool t3`) and in
  practice mode. Every card is modeled.

### Simulator history

- **2026-10-14.** Swapping Flicker for Dragonlord Dromoka measured **+1.1 over all tiers, within noise,** in a paired
  test (T1 +0.8, T2 +3.0, T3 -0.8, T4 +2.2, T5 +0.4; look-ahead, loose, 500 games per list per tier). A pool run is
  pending.
- **2026-10-12.** Previous list's figures: **13.2% over all tiers** (±0.4; tied with Alela for 9th of the ten decks):
  T1 13.1%, T2 15.9%, T3 12.3%, T4 12.1%, T5 12.6%, 6,720 games per tier, every deck on the same code;
  `decklists/pool/pool-results.md` section 0d.
- **Earlier, undated (What the simulator found).** Against the opponent pools with the look-ahead AI (loose profile, 120
  games per tier) the deck won 15.0% against Tier 1, 11.7% against Tier 2, 8.3% against Tier 3, 12.5% against Tier 4 and
  8.3% against Tier 5; an even share is 25%. The deck's result depends on finding a searcher. In games where one
  appears, the AI puts about three Rebels onto the battlefield per game, and one game in nine reaches eight or more. With
  six searchers in 99 cards (plus Recruiter of the Guard, which finds the Sergeant, Lieutenant or Captain), about 40% of
  games see none. Galadriel's first Alliance trigger comes on turn 6 in a typical game; the AI chooses counters most
  often, then mana, then the draw.

## Tymna the Weaver & Kraum, Ludevic's Opus — Four-Colour Partners (W/U/B/R)

Deck key `kraum` (removed), `decklists/JD/tymna-kraum-partners.md` (removed).

**Retired 2026-10-19: became Tymna/Thrasios** (key `thrasios`, see `audit/thrasios/FINDINGS.md`). The entries below are the deck's history.

### Changes

- **2026-10-19.** Out: Battlefield Forge, Bedevil, Black Widow, Agile Avenger, Clifftop Retreat, Dragonskull Summit, Expressive Iteration, Firebird, Blazing Ranger, Furycalm Snarl, Gilgamesh, Master-at-Arms, Glorybringer, Kraum, Ludevic's Opus, Lightning, Army of One, Lightning, Security Sergeant, Marchesa, the Black Rose, Mountain, Rugged Prairie, Smaug the Magnificent, Spectator Seating, Sulfur Falls, Talisman of Conviction, Terror of the Peaks, Thundermaw Hellkite, Underworld Breach. In: Avacyn's Pilgrim, Basalt Monolith, Bite Down, Blossoming Defense, Breeding Pool, Brushland, Coiling Oracle, Deathcap Glade, Dragonlord Dromoka, Faeburrow Elder, Forest, Llanowar Wastes, Paradox Surveyor, Rings of Brighthearth, Sunpetal Grove, Talisman of Curiosity, Talisman of Unity, Thrasios, Triton Hero, Undergrowth Stadium, Walking Ballista, Witherbloom Charm, Worldly Tutor, Yavimaya Coast. Tymna/Thrasios deck added (kraum list with the red cards swapped for green and colourless ones)
- **2026-10-18.** Out: Serra Angel. In: Cecil, Dark Knight. Serra Angel -> Cecil, Dark Knight: +0.4 at T5 vs Gilgamesh in the coarse pass (noise); JD keeps Gilgamesh and cuts Serra Angel, a vanilla 5-drop, for a 1-mana deathtouch attacker that starts Tymna's draws early.
- **2026-10-18.** Out: Elemental Mascot. In: Lightning, Army of One. Elemental Mascot -> Lightning, Army of One: -2.8 at T5 in the coarse pass (noise; JD's choice).
- **2026-10-18.** Out: Rune-Scarred Demon. In: Underworld Breach. Band calibration 10/18 (JD approved): Rune-Scarred Demon=>Underworld Breach
- **2026-10-17.** Out: Tocasia's Welcome. In: Smaug the Magnificent. Smaug the Magnificent (JD's card): +1.1 ±2.7 at T3, 2,000 paired look-ahead games, noise; a hasty flier for Tymna plus a Treasure each upkeep.
- **2026-10-17.** Out: Dream Trawler. In: Resplendent Angel. JD got Resplendent Angel instead of Dream Trawler. Measured at T3 (2,000 paired games): -1.8 (±2.6), inside the noise.
- **2026-10-17.** Out: Rip Apart. In: Avenge (from bulk). Measured at T3 (2,000 paired games): +3.1 (±2.6), list 31.7 → 34.8%: a second creature wipe, so the deck dies to combat less (eliminated 67.0 → 63.7%).

## Omnath, Locus of Rage — Red-Green Landfall Elementals

Deck key `omnath` (removed), `decklists/JD/omnath-landfall.md` (removed).

**Retired 2026-10-25: JD took 15 cards and a Forest for The Ur-Dragon** (`decklists/JD/ur-dragon-equipment.md`); the other 84 cards went to bulk. The entries below are the deck's history.

### Changes

- **2026-10-19.** Out: Exotic Orchard, Harrow. In: Game Trail, Lightning Bolt. Exotic Orchard -> Game Trail (JD has no spare Orchard; Game Trail is a real R/G dual from his bulk, untapped by revealing a Mountain or Forest), Harrow -> Lightning Bolt (JD's spare; saves a buy). Not simulated yet.
- **2026-10-19.** Out: Bite Down, Blossoming Defense, Worldly Tutor. In: Glorybringer, Smaug the Magnificent, Thundermaw Hellkite. Out: Worldly Tutor, Blossoming Defense, Bite Down (to Tymna/Thrasios). In: Glorybringer, Thundermaw Hellkite, Smaug the Magnificent (from Tymna/Kraum). JD approved 10/19 with the Thrasios switch: +3.1 over t1-t3 (T1 +2.7, T2 +5.1, T3 +1.4; 1,000 paired games per tier, look-ahead, loose; audit/omnath/2026-10-19-thrasios-trade/). The hasty dragons add about 4 damage per game while Omnath waits for landfall.
- **2026-10-19.** New deck: Omnath, Locus of Rage red-green landfall, built from the Titania list.

## Tymna the Weaver & Thrasios, Triton Hero — Four-Colour Partners (W/U/B/G)

Deck key `thrasios`, `decklists/JD/tymna-thrasios-partners.md`.

### Changes

- **2026-10-27.** Out: Disenchant. In: Assassin's Trophy. JD's choice (2026-10-27), not simulated: Assassin's Trophy (from Seph/Atraxa) for Disenchant.
- **2026-10-24.** Out: Avenge. In: Flickering Hound. Avenge -> Flickering Hound (JD's choice, 2026-10-24; the Hound moves over from Seph/Atraxa). Avenge was stuck in hand in 75% of the games it was drawn (10/19 analysis, T5) and a wipe that hits the deck's own 30 creatures; Austere Command stays as the reset. The Hound flickers a creature on every creature cast: Sunblast Angel, Grave Titan, Sun Titan, Coiling Oracle. Not simulated.

## The Ur-Dragon — Five-Colour Dragons and Equipment

`decklists/JD/ur-dragon-equipment.md`. Not in the simulator yet.

### Changes

- **2026-10-27.** Out: Assassin's Trophy (the proxy). In: Kindred Dominance. JD's choice, not simulated: the deck's first board wipe (name Dragon; one-sided with Maskwood Nexus out). The real Assassin's Trophy from Seph/Atraxa goes to Tymna/Thrasios.
- **2026-10-25.** Out: Arcane Sanctum, Arcane Signet, Battlefield Forge, Cinder Glade, Counterspell, Dragonskull Summit, Exotic Orchard, Fabled Passage, Gamble, Game Trail, Heroic Intervention, Jetmir's Garden, Karplusan Forest, Ketria Triome, Lightning Bolt, Mountain, Plains, Rootbound Crag, Spara's Headquarters, Ziatora's Proving Ground. In: Badlands, Bayou, Flooded Strand, Force of Negation, Godless Shrine, Grand Abolisher, Grim Monolith, Hallowed Fountain, Imperial Seal, Mana Drain, Marsh Flats, Plateau, Polluted Delta, Scrubland, Taiga, Tropical Island, Tundra, Underground Sea, Verdant Catacombs, Volcanic Island, Watery Grave. Proxy upgrades (JD, 10/25): 9 original duals, 4 fetch lands and 3 shock lands for the Triomes, tapped and painland duals, Fabled Passage, Exotic Orchard and three basics; Mana Drain for Counterspell, Grand Abolisher for Heroic Intervention, Grim Monolith for Arcane Signet, Imperial Seal for Gamble, Force of Negation for Lightning Bolt.
- **2026-10-25.** New deck (JD, 10/25): five-colour Godo + Helm of the Host combo with Equipment and Dragons; Gilgamesh, Master-at-Arms in the 99.
