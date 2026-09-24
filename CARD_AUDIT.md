# Card audit

Every card was checked against its official Oracle text (from the Forge card database).
**Modeled** = the effect is captured as far as the engine abstracts Magic.
**Approximate** = captured with a simplification. **Partial** = main effect in, secondary effect missing.
**Not modeled** = the sim never casts it. **Unverified** = Oracle text not found; tags are a best guess.

Engine-wide simplifications: no politics or deals, no stack beyond one response window, triggers resolve immediately,
scry/surveil/card selection count as nothing or as plain draws, and planeswalkers have no loyalty (they persist and act each turn).


## Sephiroth (Atraxa, Grand Unifier)

| Card | Status | Notes |
|---|---|---|
| Altar of Dementia | Partial | free sac outlet; the mill-a-player mode is ignored |
| Anguished Unmaking | Modeled | tags: `rem=exile tgt=nl lose=3` |
| Animate Dead | Modeled | tags: `rean=animate` |
| Arcane Signet | Approximate | any colour in identity |
| Archon of Cruelty | Modeled | tags: `bomb=9 pow=6 fly archon` |
| Ash Barrens | Modeled | tags: `c=C` |
| Ashnod's Altar | Modeled | tags: `sac` |
| Atraxa, Grand Unifier | Approximate | enters-the-battlefield modeled as drawing 4 (typical hit rate from the top ten) |
| Birds of Paradise | Modeled | tags: `dork=A noatk fly` |
| Blood Artist | Modeled | tags: `bartist pow=0 noatk human` |
| Blossoming Sands | Modeled | tags: `c=GW t` |
| Bojuka Bog | Partial | mana only; the graveyard exile is ignored |
| Brushland | Approximate | pain damage ignored |
| Buried Alive | Modeled | tags: `fill=buried` |
| Carrion Feeder | Modeled | tags: `sac pow=1 noatk noblock` |
| Caves of Koilos | Approximate | pain damage ignored |
| Chromatic Lantern | Modeled | tags: `rock=1:A lantern` |
| Command Tower | Modeled | tags: `c=A` |
| Cryptolith Rite | Modeled | tags: `rite` |
| Cultivate | Modeled | tags: `lr=1 lrt lh=1` |
| Deadly Dispute | Approximate | sacrifices Stitcher's Supplier or a Treasure |
| Demonic Tutor | Modeled | tags: `tut=any` |
| Diabolic Intent | Modeled | tags: `tut=any needsac` |
| Diabolic Tutor | Modeled | tags: `tut=any` |
| Dovin's Veto | Modeled | tags: `ctr=nc unc` |
| Elesh Norn, Grand Cenobite | Modeled | tags: `bomb=8 pow=4 tgh=7 vig leg normgc anthem2` |
| Elesh Norn, Mother of Machines | Modeled | tags: `bomb=7 pow=4 tgh=7 vig leg mother` |
| Entomb | Modeled | tags: `fill=entomb` |
| Evil Reawakened | Modeled | tags: `rean=evil` |
| Evolving Wilds | Modeled | tags: `c=A f` |
| Exotic Orchard | Modeled | tags: `c=A` |
| Fabled Passage | Modeled | tags: `c=A f` |
| Farewell | Approximate | chooses modes by net value: creatures always, artifacts/enchantments when it gains; never graveyards |
| Glacial Fortress | Modeled | tags: `c=WU ck` |
| Grave Titan | Modeled | tags: `bomb=7 pow=6 dt titan` |
| Gray Merchant of Asphodel | Modeled | tags: `bomb=4 pow=2 gray` |
| Grisly Salvage | Approximate | always takes a land; bombs in the five go to the graveyard |
| Hallowed Fountain | Modeled | tags: `c=WU` |
| Hinterland Harbor | Modeled | tags: `c=GU ck` |
| Isolated Chapel | Modeled | tags: `c=WB ck` |
| Lash of the Balrog | Modeled | tags: `rem=destroy tgt=c sacor4` |
| Lethal Scheme | Partial | convoke works; connive on convoking creatures ignored |
| Llanowar Elves | Modeled | tags: `dork=G noatk human` |
| Llanowar Wastes | Approximate | pain damage ignored |
| Massacre Wurm | Modeled | tags: `bomb=6 pow=6 tgh=5 wurm wurmdrain` |
| Mikaeus, the Unhallowed | Partial | anthem and undying modeled; intimidate and the Human-destroying clause ignored |
| Mind Stone | Partial | mana only; the sacrifice-to-draw ability is ignored |
| Necromancy | Partial | flash mode ignored |
| Night's Whisper | Modeled | tags: `draw=2` |
| Orzhov Signet | Modeled | tags: `rock=1:WB` |
| Overgrown Tomb | Modeled | tags: `c=BG` |
| Path to Exile | Modeled | tags: `rem=exile tgt=c rland` |
| Persist | Modeled | tags: `rean=persist` |
| Phyrexian Arena | Modeled | tags: `eng=1` |
| Phyrexian Metamorph | Approximate | enters as a copy of the best creature/artifact in play, including its ETB; Phyrexian mana paid as blue |
| Reanimate | Modeled | tags: `rean=reanimate` |
| Sakura-Tribe Elder | Approximate | fetches its land on entry instead of waiting to chump-block |
| Satyr Wayfinder | Modeled | tags: `fill=wayfinder pow=1 noatk` |
| Sephiroth, Planet's Heir | Modeled | tags: `bomb=5 pow=4 vig leg heir` |
| Sheoldred, Whispering One | Approximate | swampwalk checks basic Swamps and Swamp-typed shocklands |
| Sheoldred, the Apocalypse | Modeled | tags: `bomb=7 pow=4 tgh=5 dt leg sheoA` |
| Skullclamp | Approximate | used on 1-toughness fodder up to twice a turn |
| Smothering Tithe | Approximate | opponents fail to pay half the time |
| Sol Ring | Modeled | tags: `rock=2:C` |
| Stinkweed Imp | Approximate | combat 'destroy' treated as deathtouch; dredge used when a reanimation spell is waiting |
| Stitcher's Supplier | Modeled | tags: `fill=stitcher pow=1 noatk` |
| Strip Mine | Partial | mana only; land destruction ignored |
| Sunpetal Grove | Modeled | tags: `c=GW ck` |
| Swan Song | Modeled | tags: `ctr=ise swan` |
| Swiftfoot Boots | Modeled | tags: `prot=boots` |
| Swords to Plowshares | Modeled | tags: `rem=exile tgt=c rgain` |
| Talisman of Hierarchy | Approximate | the 1 damage for coloured mana is ignored |
| Temple Garden | Modeled | tags: `c=GW` |
| Tortured Existence | Approximate | used to put a bomb in the graveyard; regrowth picks a small creature |
| Toxic Deluge | Approximate | pays X equal to the biggest opposing toughness (max 10) |
| Treno, Dark City | Modeled | tags: `c=UB t` |
| Unburial Rites | Modeled | tags: `rean=rites` |
| Underground River | Approximate | pain damage ignored |
| Unmarked Grave | Modeled | tags: `fill=unmarked` |
| Viscera Seer | Partial | free sac outlet; scry ignored |
| Watery Grave | Modeled | tags: `c=UB` |
| Woodland Cemetery | Modeled | tags: `c=BG ck` |
| Wrath of God | Modeled | tags: `wipe=destroy` |
| Yavimaya Coast | Approximate | pain damage ignored |
| Yawgmoth's Will | Partial | only used to replay reanimation and graveyard-filling spells |
| Zulaport Cutthroat | Modeled | tags: `drain pow=1 noatk human` |

## Veyran

| Card | Status | Notes |
|---|---|---|
| Abrade | Modeled | tags: `rem=dmg3 tgt=c alsoart` |
| Aetherflux Reservoir | Modeled | tags: `aether` |
| An Offer You Can't Refuse | Modeled | tags: `ctr=nc offer` |
| Arcane Signet | Approximate | any colour in identity |
| Archmage Emeritus | Modeled | tags: `pow=2 spelldraw noatk wizard` |
| Ash Barrens | Modeled | tags: `c=C` |
| Banishing Betrayal | Approximate | surveil ignored |
| Blasphemous Act | Modeled | tags: `wipe=dmg13 perCreature` |
| Blazing Firesinger // Seething Song | Approximate | combo piece; the Seething Song mana outside the loop is ignored |
| Burning Prophet | Partial | body only; scry ignored |
| Burst Lightning | Approximate | kicker ignored (always 2 damage) |
| Chaos Warp | Partial | shuffle-away modeled; the revealed-permanent replacement ignored |
| Coastal Peak | Modeled | tags: `c=UR t` |
| Command Tower | Modeled | tags: `c=A` |
| Counterspell | Modeled | tags: `ctr=any` |
| Crackle with Power | Approximate | X chosen from available mana; only aimed at players |
| Cyclonic Rift | Modeled | tags: `wipe=rift rem=bounce tgt=nl` |
| Deduce | Modeled | tags: `draw=1 clue` |
| Desolate Lighthouse | Partial | mana only; loot ability ignored |
| Disdainful Stroke | Modeled | tags: `ctr=mv4` |
| Displacer Kitten | Partial | combo piece only; value blinks (Venser, Emeritus re-prepare) ignored |
| Disruptor Flute | Not modeled | never cast; naming-a-card effects aren't modeled |
| Dreams of Laguna | Modeled | tags: `draw=1 fb=3U` |
| Dualcaster Mage | Approximate | copies your most recent instant/sorcery instead of one on the stack |
| Emeritus of Conflict // Lightning Bolt | Approximate | third spell each turn casts a Lightning Bolt copy (creature or face); first strike ignored |
| Emeritus of Ideation // Ancestral Recall | Unverified | no Oracle text found; assumed to enter prepared and cast an Ancestral Recall copy (draw 3) |
| Eris, Roar of the Storm | Approximate | cost reduction and second-spell Dragons modeled; prowess ignored |
| Evolving Wilds | Modeled | tags: `c=A f` |
| Exotic Orchard | Modeled | tags: `c=A` |
| Expressive Iteration | Approximate | card selection treated as a plain draw |
| Fabricate | Modeled | tags: `tut=art` |
| Fellwar Stone | Approximate | treated as any colour in identity |
| Flashback | Modeled | tags: `fbgrant` |
| Flow State | Approximate | card selection treated as plain draws |
| Force of Will | Modeled | tags: `ctr=any free` |
| Gandalf, Friend of the Shire | Approximate | lets sorceries be cast in the end-of-turn window; Ring draw ignored |
| Guttersnipe | Modeled | tags: `ping=2 pow=2 noatk shaman` |
| Harmonic Prodigy | Approximate | trigger doubling for Shamans/Wizards modeled; prowess ignored |
| Hydro-Channeler | Approximate | treated as a blue mana dork (spend restriction ignored) |
| Jin-Gitaxias, Progress Tyrant | Approximate | copies your first spell each turn (extra magecraft + draw effect); counters opponents' first artifact/instant/sorcery each turn |
| Kylox, Visionary Inventor | Partial | haste and attack trigger modeled (sacrifices tokens); menace and ward ignored |
| Light Up the Stage | Approximate | spectacle cost modeled; exiled cards treated as draws |
| Lightning Bolt | Modeled | tags: `rem=dmg3 tgt=c face` |
| Mistrise Village | Modeled | tags: `c=U ck mistrise` |
| Mizzix's Mastery | Partial | overload modeled; the single-target mode is ignored |
| Murmuring Mystic | Modeled | tags: `mystic pow=1 tgh=5 noatk wizard` |
| Muse Seeker | Approximate | draws then discards the worst card unless the spell cost 5+ |
| Mystic Confluence | Modeled | tags: `ctr=any soft=3 eotdraw=3` |
| Mystic Sanctuary | Partial | mana only; the instant/sorcery return is ignored |
| Mystical Tutor | Modeled | tags: `tut=is` |
| Old Fat Spider Can't See Me | Partial | hexproof on your best creature and draws on chapters III-IV; chapter II damage prevention ignored |
| Path of Ancestry | Modeled | tags: `c=A t` |
| Plunder the Trollshaws | Modeled | tags: `draw=1 fb=3U fbdraw=2` |
| Pongify | Modeled | tags: `rem=destroy tgt=c rtok=3` |
| Prismari Charm | Partial | only the surveil/draw mode |
| Quick Study | Modeled | tags: `draw=2` |
| Ral, Storm Conduit | Partial | magecraft ping modeled; loyalty abilities (scry, spell copy) ignored |
| Reality Shift | Modeled | tags: `rem=exile tgt=c rtok=2` |
| Reenact the Crime | Approximate | modeled as a cantrip |
| Return the Favor | Not modeled | never cast; stack-copy/redirect effects aren't modeled |
| Rite of the Dragoncaller | Modeled | tags: `dragoncaller` |
| River's Rebuke | Modeled | tags: `wipe=rebuke` |
| Sanar, Unfinished Genius // Wild Idea | Approximate | enters prepared (Wild Idea tutor copy); one Treasure per turn after an instant/sorcery |
| Scorched Geyser | Modeled | tags: `c=UR ck` |
| Shivan Reef | Modeled | tags: `c=UR` |
| Sleight of Hand | Approximate | card selection treated as a plain draw |
| Sokenzan, Crucible of Defiance | Partial | mana only; channel ignored |
| Sol Ring | Modeled | tags: `rock=2:C` |
| Solve the Equation | Modeled | tags: `tut=is` |
| Spectacle Summit | Partial | mana only; surveil ability ignored |
| Spell Pierce | Modeled | tags: `ctr=nc soft=2` |
| Spirebluff Canal | Modeled | tags: `c=UR` |
| Steam Vents | Modeled | tags: `c=UR` |
| Stock Up | Approximate | card selection treated as plain draws |
| Stormcarved Coast | Modeled | tags: `c=UR ck` |
| Sulfur Falls | Modeled | tags: `c=UR ck` |
| Temple of Epiphany | Modeled | tags: `c=UR t` |
| Terramorphic Expanse | Modeled | tags: `c=A f` |
| Think Twice | Modeled | tags: `draw=1 fb=2U` |
| Thor, Asgard's Avenger | Modeled | tags: `pow=4 thor` |
| Thought Vessel | Modeled | tags: `rock=1:C` |
| Thunderdrum Soloist | Modeled | tags: `ping=1 pow=1 tgh=3 noatk opus3` |
| Venser, Shaper Savant | Partial | bounces a permanent on entry; the bounce-a-spell mode is ignored |
| Veyran, Voice of Duality | Approximate | trigger doubling modeled; its own +1/+1 ignored (stays home) |
| Vibrant Outburst | Modeled | tags: `rem=dmg3 tgt=c face` |

## Sauron

| Card | Status | Notes |
|---|---|---|
| Aggravated Assault | Modeled | tags: `assault` |
| Arcane Denial | Approximate | draws happen immediately instead of next upkeep |
| Arcane Signet | Approximate | any colour in identity |
| Barad-dûr | Partial | mana only; amass ability ignored |
| Bedevil | Modeled | tags: `rem=destroy tgt=cap` |
| Big Score | Modeled | tags: `draw=2 treas=2 discard1` |
| Bitter Triumph | Modeled | tags: `rem=destroy tgt=cp lose=3` |
| Blasphemous Act | Modeled | tags: `wipe=dmg13 perCreature` |
| Blood Crypt | Modeled | tags: `c=BR` |
| Bloodchief's Thirst | Approximate | always cast kicked |
| Bloodsoaked Insight // Sanguine Morass | Partial | only used as a land |
| Call of the Ring | Approximate | modeled as a card per upkeep for 2 life; Ring-bearer details ignored |
| Champion's Helm | Not modeled | never equipped; the Army isn't legendary so its hexproof rarely applies |
| Chaos Warp | Partial | shuffle-away modeled; the revealed-permanent replacement ignored |
| Chromatic Lantern | Modeled | tags: `rock=1:A lantern` |
| Command Tower | Modeled | tags: `c=A` |
| Conqueror's Flail | Modeled | tags: `flail` |
| Counterspell | Modeled | tags: `ctr=any` |
| Crumbling Necropolis | Modeled | tags: `c=UBR t` |
| Cyclonic Rift | Modeled | tags: `wipe=rift rem=bounce tgt=nl` |
| Deepglow Skate | Modeled | tags: `skate pow=3` |
| Diabolic Tutor | Modeled | tags: `tut=any` |
| Drowned Catacomb | Modeled | tags: `c=UB ck` |
| Erebos, God of the Dead | Partial | no-lifegain and draw modeled; creature mode ignored |
| Exotic Orchard | Modeled | tags: `c=A` |
| Feed the Swarm | Modeled | tags: `rem=destroy tgt=ce losemv` |
| Flux Channeler | Modeled | tags: `prolif pow=2 noatk` |
| Foreboding Ruins | Modeled | tags: `c=BR ck` |
| Frostboil Snarl | Modeled | tags: `c=UR ck` |
| Go for the Throat | Modeled | tags: `rem=destroy tgt=cna` |
| Grave Titan | Modeled | tags: `bomb=7 pow=6 dt titan` |
| Hellkite Tyrant | Modeled | tags: `pow=6 tgh=5 fly trample hellkite` |
| Inexorable Tide | Modeled | tags: `prolif prolifall` |
| Infernal Grasp | Modeled | tags: `rem=destroy tgt=c lose=2` |
| Iron Man, Armored Avenger | Partial | +1/+1 counter on the Army per card drawn; the flying grant ignored |
| Izzet Boilerworks | Modeled | tags: `c=UR t amt=2 bounceland` |
| Jace's Archivist | Approximate | wheel used when Bowmasters is out |
| Kaervek the Merciless | Approximate | damage always goes to the caster's face |
| Kindred Discovery | Approximate | a card per upkeep instead of per Orc entering/attacking |
| Lightning Greaves | Modeled | tags: `prot=boots` |
| Mauhúr, Uruk-hai Captain | Approximate | +1 Army counter per amass; menace ignored |
| Memory Lapse | Modeled | tags: `ctr=any lapse` |
| Metallic Mimic | Partial | only the extra counter on a new Army |
| Mind Stone | Partial | mana only; the sacrifice-to-draw ability is ignored |
| Nibelheim Aflame | Modeled | tags: `wipe=nib fb=5RR fbnib` |
| Night's Whisper | Modeled | tags: `draw=2` |
| Not of This World | Modeled | tags: `prot=notw` |
| Noxious Gearhulk | Modeled | tags: `gearhulk pow=5 tgh=4 rem=destroy tgt=c etb gaintgh` |
| Orcish Bowmasters | Approximate | ETB shoots a valuable 1-toughness creature, else a face; draw triggers ping faces |
| Path of Ancestry | Modeled | tags: `c=A t` |
| Phyrexian Arena | Modeled | tags: `eng=1` |
| Plaza of Heroes | Approximate | treated as colourless |
| Ral Zarek, Guest Lecturer | Partial | each opponent discards for three turns; +1 and -2 ignored |
| Reconnaissance Mission | Approximate | a card per upkeep; cycling ignored |
| Rhystic Study | Modeled | tags: `rhystic` |
| Ringsight | Modeled | tags: `tut=ubr` |
| Rogue's Passage | Modeled | tags: `c=C passage` |
| Sauron, the Dark Lord | Approximate | amass per opponent spell, ward treated as untargetable, Ring draw-four when the hand is small |
| Sauron, the Necromancer | Approximate | attacking 3/3 Wraith token; copied abilities ignored |
| Scarlet Witch, Chaotic Avenger | Partial | 3/3 flier; free-spell trigger ignored |
| Scavenger Grounds | Modeled | tags: `c=C desert` |
| Shivan Reef | Modeled | tags: `c=UR` |
| Slip Out the Back | Modeled | tags: `prot=phase` |
| Sol Ring | Modeled | tags: `rock=2:C` |
| Steam Vents | Modeled | tags: `c=UR` |
| Sulfur Falls | Modeled | tags: `c=UR ck` |
| Sword of Feast and Famine | Modeled | tags: `sword` |
| Sword of the Animist | Modeled | tags: `animist` |
| Talisman of Creativity | Approximate | the 1 damage for coloured mana is ignored |
| Terminate | Modeled | tags: `rem=destroy tgt=c` |
| Tezzeret's Gambit | Modeled | tags: `draw=2 prolif1` |
| Tome of Legends | Not modeled | never cast |
| Toxic Deluge | Approximate | pays X equal to the biggest opposing toughness (max 10) |
| Treno, Dark City | Modeled | tags: `c=UB t` |
| Unclaimed Territory | Approximate | treated as colourless |
| Undermine | Modeled | tags: `ctr=any undermine` |
| Unearth | Approximate | returns the best creature with MV 3 or less; cycling ignored |
| Vision, Synthezoid Avenger | Partial | 3/3 flier; phasing and counters ignored |
| Vraska, Betrayal's Sting | Partial | 0 ability every turn (draw, lose 1, proliferate); -2 and ultimate ignored |
| War Machine, Avenging Arsenal | Approximate | double strike applied to the Army |
| Whispersilk Cloak | Modeled | tags: `cloak` |
| Witch-king, Bringer of Ruin | Modeled | tags: `witchking pow=5 tgh=3 fly leg` |

## Najeela

| Card | Status | Notes |
|---|---|---|
| Adeline, Resplendent Cathar | Modeled | tags: `pow=1 tgh=4 vig tokatk=3 leg adeline` |
| Aether Hub | Approximate | treated as any colour |
| Anguished Unmaking | Modeled | tags: `rem=exile tgt=nl lose=3` |
| Arcane Sanctum | Modeled | tags: `c=WUB t` |
| Arcane Signet | Approximate | any colour in identity |
| Ash Barrens | Modeled | tags: `c=C` |
| Atraxa, Grand Unifier | Approximate | enters-the-battlefield modeled as drawing 4 (typical hit rate from the top ten) |
| Atraxa, Praetors' Voice | Modeled | tags: `bomb=5 pow=4 fly vig dt lifelink leg pvprolif` |
| Austere Command | Modeled | tags: `wipe=austere2` |
| Birds of Paradise | Modeled | tags: `dork=A noatk fly` |
| Bonders' Enclave | Partial | mana only; draw ability ignored |
| Cathars' Crusade | Approximate | counters capped at +60 per creature |
| Chaos Warp | Partial | shuffle-away modeled; the revealed-permanent replacement ignored |
| Chromatic Lantern | Modeled | tags: `rock=1:A lantern` |
| Clever Concealment | Modeled | tags: `prot=phase convoke` |
| Coalition Relic | Partial | one mana per turn; charge counters ignored |
| Command Tower | Modeled | tags: `c=A` |
| Crumbling Necropolis | Modeled | tags: `c=UBR t` |
| Cultivate | Modeled | tags: `lr=1 lrt lh=1` |
| Diabolic Tutor | Modeled | tags: `tut=any` |
| Dispatch | Modeled | tags: `rem=exile tgt=c needart3` |
| Duty Beyond Death | Partial | indestructible vs wipes; sacrifice cost and counters ignored |
| Elrond, Lord of Rivendell | Not modeled | body only; scry and Ring tempt ignored |
| Emmara, Soul of the Accord | Approximate | one lifelink Soldier per attack |
| End-Raze Forerunners | Modeled | tags: `pow=7 haste vig trample endraze` |
| Ephemerate | Partial | blink to dodge removal; rebound ignored |
| Evolving Wilds | Modeled | tags: `c=A f` |
| Exotic Orchard | Modeled | tags: `c=A` |
| Fabled Passage | Modeled | tags: `c=A f` |
| Felidar Retreat | Modeled | tags: `landfall2` |
| Fellwar Stone | Approximate | treated as any colour in identity |
| Garruk's Uprising | Modeled | tags: `uprising` |
| Gemstone Mine | Approximate | treated as any colour (no depletion) |
| Generous Gift | Modeled | tags: `rem=destroy tgt=p rtok=3` |
| Harmonize | Modeled | tags: `draw=3` |
| Hornet Queen | Modeled | tags: `pow=2 fly dt tok=4 tokfly tokdt` |
| Jinnie Fay, Jetmir's Second | Approximate | always makes 2/2 hasty Cats from small tokens |
| Jungle Shrine | Modeled | tags: `c=RGW t` |
| Legion's Landing // Adanto, the First Fort | Partial | token on entry; the transform to a land ignored |
| Lightning Greaves | Modeled | tags: `prot=boots` |
| March of the Multitudes | Modeled | tags: `tokx convoke toklife` |
| Mirror Entity | Approximate | sets creatures to X/X with leftover mana after activations |
| Mycoloth | Modeled | tags: `pow=4 mycoloth` |
| Mystic Remora | Approximate | a card per upkeep for four turns |
| Najeela, the Blade-Blossom | Approximate | Warrior tokens per attacking Warrior, up to two WUBRG extra combats |
| Nature's Lore | Modeled | tags: `lr=1` |
| Nicol Bolas, Dragon-God | Approximate | +1 each turn for up to five turns (draw, each opponent discards); -3 and ultimate ignored |
| Overwhelming Stampede | Modeled | tags: `stampede` |
| Path of Ancestry | Modeled | tags: `c=A t` |
| Prairie Stream | Modeled | tags: `c=WU t` |
| Professional Face-Breaker | Modeled | tags: `pow=2 tgh=3 warrior facebreaker` |
| Rabble Rousing | Partial | a Citizen per attacker; hideaway ignored |
| Relic of Legends | Partial | one mana per turn; tapping legends for mana ignored |
| Restoration Angel | Modeled | tags: `pow=3 fly flash prot=blink` |
| Rhys the Redeemed | Approximate | a Warrior token each upkeep plus the token-doubling activation when wide |
| Sauron, the Lidless Eye | Partial | pump-and-drain activation modeled; ETB threaten ignored |
| Savage Lands | Modeled | tags: `c=BRG t` |
| Seaside Citadel | Modeled | tags: `c=GWU t` |
| Secure the Wastes | Modeled | tags: `tokx warrior` |
| Shamanic Revelation | Modeled | tags: `drawcre` |
| Skyclave Apparition | Partial | exile on entry; the token when it leaves ignored |
| Skyshroud Claim | Modeled | tags: `lr=2` |
| Sol Ring | Modeled | tags: `rock=2:C` |
| Spectator Seating | Modeled | tags: `c=A` |
| Spire of Industry | Approximate | treated as any colour |
| Sterling Grove | Not modeled | never cast |
| Sun Titan | Modeled | tags: `pow=6 vig suntitan` |
| Swords to Plowshares | Modeled | tags: `rem=exile tgt=c rgain` |
| Sylvan Library | Modeled | tags: `eng=1` |
| Talisman of Dominance | Approximate | the 1 damage for coloured mana is ignored |
| Temple of Epiphany | Modeled | tags: `c=UR t` |
| Temple of Malice | Modeled | tags: `c=BR t` |
| Temple of Silence | Modeled | tags: `c=WB t` |
| Terramorphic Expanse | Modeled | tags: `c=A f` |
| The Dawning Archaic | Partial | cost reduction modeled; the attack-trigger recast ignored |
| The World Tree | Partial | any-colour mana with six lands; the tutor ability ignored |
| Treno, Dark City | Modeled | tags: `c=UB t` |
| Trostani's Summoner | Modeled | tags: `pow=1 tokbig` |
| Ultimate Magic: Holy | Approximate | indestructible vs wipes; foretell ignored |
| Unbreakable Formation | Partial | indestructible vs wipes; addendum counters ignored |
| Vandalblast | Modeled | tags: `rem=destroy tgt=a wipe=vandal` |
| Vanquish the Horde | Modeled | tags: `wipe=destroy perCreature` |
| Vivid Creek | Approximate | treated as any colour |
| Vivid Grove | Approximate | treated as any colour |
| Vivid Marsh | Approximate | treated as any colour |
| Wargate | Approximate | tutors to hand instead of the battlefield |
| Warleader's Call | Modeled | tags: `warleader` |
| Wispdrinker Vampire | Modeled | tags: `pow=2 tgh=4 fly wisp` |
| Wurmcoil Engine | Modeled | tags: `pow=6 dt lifelink wurmcoil` |

## Tagged test cards (not in any current list)

| Card | Status | Notes |
|---|---|---|
| Imperial Recruiter | Modeled | tags: `pow=1 noatk recruit` |
| Inventors' Fair | Modeled | tags: `c=C fair` |
| Storm-Kiln Artist | Approximate | Treasures modeled; its attack power is irrelevant (stays home) |
| Birgi, God of Storytelling // Harnfel, Horn of Bounty | Approximate | mana per spell modeled; back face and boast ignored |
| Evacuation | Modeled | tags: `wipe=evac` |
| Talrand, Sky Summoner | Modeled | tags: `pow=2 noatk leg wizard spelltok=2 spelltokfly` |
| Young Pyromancer | Modeled | tags: `pow=2 tgh=1 noatk shaman spelltok=1` |
| Third Path Iconoclast | Modeled | tags: `pow=2 tgh=1 noatk spelltok=1` |
| Kessig Flamebreather | Modeled | tags: `pow=1 tgh=3 noatk shaman ping=1` |
| Firebrand Archer | Modeled | tags: `pow=2 tgh=1 noatk ping=1` |
| Insatiable Avarice | Modeled | tags: `avarice` |
| Grim Tutor | Modeled | tags: `tut=any lose=3` |
| Aura Shards | Modeled | tags: `shards` |
