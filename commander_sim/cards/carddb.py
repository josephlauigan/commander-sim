# name | types | cost | tags   (hand-tagged roles; see sim.py for meaning)
DB_TEXT = r"""
Forest|L|-|c=G
Island|L|-|c=U
Plains|L|-|c=W
Swamp|L|-|c=B
Mountain|L|-|c=R
Altar of Dementia|A|2|sac
Anguished Unmaking|I|1WB|rem=exile tgt=nl lose=3
Animate Dead|E|1B|rean=animate
Arcane Signet|A|2|rock=1:A
Archon of Cruelty|C|6BB|bomb=9 pow=6 fly archon
Ash Barrens|L|-|c=C
Ashnod's Altar|A|3|sac
Atraxa, Grand Unifier|C|3GWUB|bomb=9 pow=7 fly vig dt lifelink leg atraxa
Birds of Paradise|C|G|dork=A noatk fly
Blood Artist|C|1B|bartist pow=0 noatk human
Blossoming Sands|L|-|c=GW t
Bojuka Bog|L|-|c=B t
Brushland|L|-|c=GW
Buried Alive|S|2B|fill=buried
Carrion Feeder|C|B|sac pow=1 noatk noblock
Caves of Koilos|L|-|c=WB
Chromatic Lantern|A|3|rock=1:A lantern
Command Tower|L|-|c=A
Cryptolith Rite|E|1G|rite
Cultivate|S|2G|lr=1 lrt lh=1
Deadly Dispute|I|1B|fill=dispute draw=2
Demonic Tutor|S|1B|tut=any
Diabolic Intent|S|1B|tut=any needsac
Diabolic Tutor|S|2BB|tut=any
Dovin's Veto|I|WU|ctr=nc unc
Elesh Norn, Grand Cenobite|C|5WW|bomb=8 pow=4 tgh=7 vig leg normgc anthem2
Elesh Norn, Mother of Machines|C|4W|bomb=7 pow=4 tgh=7 vig leg mother
Entomb|I|B|fill=entomb
Evil Reawakened|S|4B|rean=evil
Evolving Wilds|L|-|c=A f
Exotic Orchard|L|-|c=A
Fabled Passage|L|-|c=A f
Farewell|S|4WW|wipe=farewell
Glacial Fortress|L|-|c=WU ck
Grave Titan|C|4BB|bomb=7 pow=6 dt titan
Gray Merchant of Asphodel|C|3BB|bomb=4 pow=2 gray
Grisly Salvage|I|BG|fill=grisly
Hallowed Fountain|L|-|c=WU
Hinterland Harbor|L|-|c=GU ck
Isolated Chapel|L|-|c=WB ck
Lash of the Balrog|S|B|rem=destroy tgt=c sacor4
Lethal Scheme|I|2BB|rem=destroy tgt=cp convoke
Llanowar Elves|C|G|dork=G noatk human
Llanowar Wastes|L|-|c=BG
Massacre Wurm|C|3BBB|bomb=6 pow=6 tgh=5 wurm wurmdrain
Mikaeus, the Unhallowed|C|3BBB|bomb=5 pow=5 leg mikaeus anthemnh
Mind Stone|A|2|rock=1:C
Necromancy|E|2B|rean=necro
Night's Whisper|S|1B|draw=2
Orzhov Signet|A|2|rock=1:WB
Overgrown Tomb|L|-|c=BG
Path to Exile|I|W|rem=exile tgt=c rland
Persist|S|1B|rean=persist
Phyrexian Arena|E|1BB|eng=1
Phyrexian Metamorph|AC|3U|pow=0 clone
Reanimate|S|B|rean=reanimate
Sakura-Tribe Elder|C|1G|lr=1 lrt noatk
Satyr Wayfinder|C|3G|fill=wayfinder pow=1 noatk
Sephiroth, Planet's Heir|C|4UB|bomb=5 pow=4 vig leg heir
Sheoldred, the Apocalypse|C|2BB|bomb=7 pow=4 tgh=5 dt leg sheoA
Sheoldred, Whispering One|C|5BB|bomb=8 pow=6 leg sheoW swampwalk
Skullclamp|A|1|clamp
Avacyn's Pilgrim|C|G|dork=W noatk human pow=1
Brush Off|I|2UU|ctr=any brushoff
Melira, Sylvok Outcast|C|1G|pow=2 leg human melira
Slaughter Pact|I|0|rem=destroy tgt=c nonblack pactpay=2B
Smothering Tithe|E|3W|tithe
Sol Ring|A|1|rock=2:C
Stinkweed Imp|C|2B|pow=1 tgh=2 fly dt dredge noatk
Stitcher's Supplier|C|B|fill=stitcher pow=1 noatk
Strip Mine|L|-|c=C
Sunpetal Grove|L|-|c=GW ck
Swan Song|I|U|ctr=ise swan
Swiftfoot Boots|A|2|prot=boots
Swords to Plowshares|I|W|rem=exile tgt=c rgain
Talisman of Hierarchy|A|2|rock=1:WB
Temple Garden|L|-|c=GW
Tortured Existence|E|B|fill=tortured
Toxic Deluge|S|2B|wipe=minus deluge
Treno, Dark City|L|-|c=UB t
Unburial Rites|S|4B|rean=rites
Underground River|L|-|c=UB
Unmarked Grave|S|1B|fill=unmarked
Viscera Seer|C|B|sac pow=1 noatk
Watery Grave|L|-|c=UB
Woodland Cemetery|L|-|c=BG ck
Wrath of God|S|2WW|wipe=destroy
Yavimaya Coast|L|-|c=GU
Yawgmoth's Will|S|2B|yawg
Zulaport Cutthroat|C|1B|drain pow=1 noatk human
Eternal Witness|C|1GG|witness pow=2 human
Heroic Intervention|I|1G|prot=hi
Dread Return|S|2BB|rean=dread
Counterspell|I|UU|ctr=any
Mnemonic Wall|C|4U|wall pow=0 tgh=4 noatk
Rune-Scarred Demon|C|5BB|bomb=6 pow=6 fly rsd
Assassin's Trophy|I|BG|rem=destroy tgt=p
Tishana's Tidebinder|C|2U|tide pow=3 flash
Abrade|I|1R|rem=dmg3 tgt=c alsoart
Aetherflux Reservoir|A|4|aether
An Offer You Can't Refuse|I|U|ctr=nc offer
Archmage Emeritus|C|2UU|pow=2 spelldraw noatk wizard
Banishing Betrayal|I|1U|rem=bounce tgt=nl
Blasphemous Act|S|8R|wipe=dmg13 perCreature
Blazing Firesinger // Seething Song|C|2R|vfire pow=2 tgh=3 noatk prepare
Burning Prophet|C|1R|pow=1 tgh=3 noatk
Burst Lightning|I|R|rem=dmg2 tgt=c face kick=4
Chaos Warp|I|2R|rem=tuck tgt=p
Coastal Peak|L|-|c=UR t
Crackle with Power|S|RR|crackle
Cyclonic Rift|I|1U|wipe=rift rem=bounce tgt=nl
Deduce|I|1U|draw=1 clue
Desolate Lighthouse|L|-|c=C
Disdainful Stroke|I|1U|ctr=mv4
Displacer Kitten|C|3U|vkitten pow=2 noatk
Disruptor Flute|A|2|flash flute
Dreams of Laguna|I|1U|draw=1 fb=3U
Dualcaster Mage|C|1RR|pow=2 flash dualcaster
Emeritus of Conflict // Lightning Bolt|C|1R|pow=2 tgh=2 conflict
Emeritus of Ideation // Ancestral Recall|C|3U|pow=5 tgh=5 fly prepare
Eris, Roar of the Storm|C|8UR|pow=4 fly eris leg
Expressive Iteration|S|UR|draw=1
Fabricate|S|2U|tut=art
Fellwar Stone|A|2|rock=1:A
Flashback|I|R|fbgrant
Flow State|S|1U|draw=2
Force of Will|I|3UU|ctr=any free
Gandalf, Friend of the Shire|C|3U|pow=2 tgh=4 flash gandalf leg noatk
Guttersnipe|C|2R|ping=2 pow=2 noatk shaman
Harmonic Prodigy|C|1R|pow=1 noatk prodigy
Hydro-Channeler|C|1U|pow=1 tgh=3 noatk dork=U
Jin-Gitaxias, Progress Tyrant|C|5UU|pow=5 leg jin
Kylox, Visionary Inventor|C|5UR|pow=4 haste leg kylox
Light Up the Stage|S|2R|draw=2 spectacle
Lightning Bolt|I|R|rem=dmg3 tgt=c face
Mistrise Village|L|-|c=U ck mistrise
Mizzix's Mastery|S|3R|mastery
Murmuring Mystic|C|3U|mystic pow=1 tgh=5 noatk wizard
Muse Seeker|C|1U|pow=1 tgh=2 noatk spellloot
Mystic Confluence|I|3UUU|ctr=any soft=3 eotdraw=3
Mystic Sanctuary|L|-|c=U t
Mystical Tutor|I|U|tut=is
Old Fat Spider Can't See Me|E|2U|spider
Path of Ancestry|L|-|c=A t
Plunder the Trollshaws|I|1U|draw=1 fb=3U fbdraw=2
Pongify|I|U|rem=destroy tgt=c rtok=3
Prismari Charm|I|UR|draw=1
Quick Study|I|2U|draw=2
Ral, Storm Conduit|P|2UR|ping=1 ral
Reality Shift|I|1U|rem=exile tgt=c rtok=2
Reenact the Crime|I|1UUU|reenact
Return the Favor|I|RR|rtf
Rite of the Dragoncaller|E|4RR|dragoncaller
River's Rebuke|S|4UU|wipe=rebuke
Sanar, Unfinished Genius // Wild Idea|C|UR|pow=0 tgh=4 noatk leg prepare sanar
Scorched Geyser|L|-|c=UR ck
Shivan Reef|L|-|c=UR
Sleight of Hand|S|U|draw=1
Sokenzan, Crucible of Defiance|L|-|c=R
Solve the Equation|S|2U|tut=is
Spectacle Summit|L|-|c=UR t
Spell Pierce|I|U|ctr=nc soft=2
Spirebluff Canal|L|-|c=UR
Steam Vents|L|-|c=UR
Stock Up|S|2U|draw=2
Stormcarved Coast|L|-|c=UR ck
Sulfur Falls|L|-|c=UR ck
Temple of Epiphany|L|-|c=UR t
Terramorphic Expanse|L|-|c=A f
Think Twice|I|1U|draw=1 fb=2U
Thor, Asgard's Avenger|C|2RR|pow=4 thor
Thought Vessel|A|2|rock=1:C
Thunderdrum Soloist|C|1R|ping=1 pow=1 tgh=3 noatk opus3
Venser, Shaper Savant|C|2UU|pow=2 rem=bounce tgt=nl etb flash
Veyran, Voice of Duality|C|1UR|veyran pow=2 leg noatk
Vibrant Outburst|I|UR|rem=dmg3 tgt=c face
Aggravated Assault|E|2R|assault
Arcane Denial|I|1U|ctr=any denial
Barad-dûr|L|-|c=B
Bedevil|I|BBR|rem=destroy tgt=cap
Big Score|I|3R|draw=2 treas=2 discard1
Bitter Triumph|I|1B|rem=destroy tgt=cp lose=3
Blood Crypt|L|-|c=BR
Bloodchief's Thirst|S|2BB|rem=destroy tgt=cp
Bloodsoaked Insight // Sanguine Morass|L|-|c=B t
Call of the Ring|E|1B|callring
Champion's Helm|A|3|helm
Conqueror's Flail|A|2|flail
Crumbling Necropolis|L|-|c=UBR t
Deepglow Skate|C|4U|skate pow=3
Drowned Catacomb|L|-|c=UB ck
Erebos, God of the Dead|E|3B|erebos
Feed the Swarm|S|1B|rem=destroy tgt=ce losemv
Flux Channeler|C|2U|prolif pow=2 noatk
Foreboding Ruins|L|-|c=BR ck
Frostboil Snarl|L|-|c=UR ck
Go for the Throat|I|1B|rem=destroy tgt=cna
Hellkite Tyrant|C|4RR|pow=6 tgh=5 fly trample hellkite
Inexorable Tide|E|3UU|prolif prolifall
Infernal Grasp|I|1B|rem=destroy tgt=c lose=2
Iron Man, Armored Avenger|AC|3U|pow=2 fly leg ironman
Izzet Boilerworks|L|-|c=UR t amt=2 bounceland
Jace's Archivist|C|1UU|archivist pow=2 noatk
Kaervek the Merciless|C|5BR|kaervek pow=5 leg
Kindred Discovery|E|3UU|kindred
Lightning Greaves|A|2|prot=boots
Mauhúr, Uruk-hai Captain|C|1BR|mauhur pow=2 leg
Memory Lapse|I|1U|ctr=any lapse
Metallic Mimic|AC|2|pow=2 tgh=1 noatk mimic
Nibelheim Aflame|S|2RR|wipe=nib fb=5RR fbnib
Not of This World|I|7|prot=notw
Noxious Gearhulk|AC|4BB|gearhulk pow=5 tgh=4 rem=destroy tgt=c etb gaintgh
Orcish Bowmasters|C|1B|bowmasters pow=1 flash
Phyrexian Arena|E|1BB|eng=1
Plaza of Heroes|L|-|c=C
Ral Zarek, Guest Lecturer|P|1BB|ralzarek
Reconnaissance Mission|E|2UU|recon
Rhystic Study|E|2U|rhystic
Ringsight|S|1UB|tut=ubr ringtempt
Rogue's Passage|L|-|c=C passage
Sauron, the Dark Lord|C|3UBR|sauron pow=7 leg
Sauron, the Necromancer|C|3BB|pow=4 leg necromancer
Scarlet Witch, Chaotic Avenger|C|2UR|pow=3 fly leg
Scavenger Grounds|L|-|c=C desert
Slip Out the Back|I|U|prot=phase
Sword of Feast and Famine|A|3|sword
Sword of the Animist|A|2|animist
Talisman of Creativity|A|2|rock=1:UR pain
Terminate|I|BR|rem=destroy tgt=c
Tezzeret's Gambit|S|3U|draw=2 prolif1
Tome of Legends|A|2|x
The Ozolith|A|1|ozolith
Unclaimed Territory|L|-|c=C
Undermine|I|UUB|ctr=any undermine
Unearth|S|B|unearth
Vision, Synthezoid Avenger|AC|4|pow=3 fly leg
Vraska, Betrayal's Sting|P|4BB|vraska
War Machine, Avenging Arsenal|AC|4R|warmachine pow=3 tgh=5 fly leg
Whispersilk Cloak|A|3|cloak
Witch-king, Bringer of Ruin|C|4BB|witchking pow=5 tgh=3 fly leg
Adeline, Resplendent Cathar|C|1WW|pow=1 tgh=4 vig tokatk=3 leg adeline
Aether Hub|L|-|c=A
Arcane Sanctum|L|-|c=WUB t
Atraxa, Praetors' Voice|C|GWUB|bomb=5 pow=4 fly vig dt lifelink leg pvprolif
Austere Command|S|4WW|wipe=austere2
Bonders' Enclave|L|-|c=C
Cathars' Crusade|E|3WW|crusade
Clever Concealment|I|2WW|prot=phase convoke
Coalition Relic|A|3|rock=1:A
Dispatch|I|W|rem=exile tgt=c needart3
Duty Beyond Death|I|1W|prot=indes
Elrond, Lord of Rivendell|C|2U|pow=3 leg
Emmara, Soul of the Accord|C|2W|pow=2 tokatk=1 leg
End-Raze Forerunners|C|5GGG|pow=7 haste vig trample endraze
Ephemerate|I|W|prot=blink
Felidar Retreat|E|3W|landfall2
Garruk's Uprising|E|2G|uprising
Gemstone Mine|L|-|c=A
Generous Gift|I|2W|rem=destroy tgt=p rtok=3
Harmonize|S|2GG|draw=3
Hornet Queen|C|4GGG|pow=2 fly dt tok=4 tokfly tokdt
Jinnie Fay, Jetmir's Second|C|RGW|pow=3 leg jinnie
Jungle Shrine|L|-|c=RGW t
Legion's Landing // Adanto, the First Fort|E|W|tok=1
March of the Multitudes|I|GWW|tokx convoke toklife
Mirror Entity|C|2W|mirror pow=1
Mycoloth|C|3GG|pow=4 mycoloth
Mystic Remora|E|U|eng=1 remora
Najeela, the Blade-Blossom|C|2R|najeela pow=3 tgh=2 warrior leg
Nature's Lore|S|1G|lr=1
Nicol Bolas, Dragon-God|P|UBBBR|bolas
Overwhelming Stampede|S|3GG|stampede
Prairie Stream|L|-|c=WU t
Professional Face-Breaker|C|2R|pow=2 tgh=3 warrior facebreaker
Rabble Rousing|E|4W|rabble
Relic of Legends|A|3|rock=1:A
Restoration Angel|C|3W|pow=3 fly flash prot=blink
Rhys the Redeemed|C|G|pow=1 tokup=1 warrior leg rhys
Sauron, the Lidless Eye|C|3BR|pow=4 leg lidless
Savage Lands|L|-|c=BRG t
Seaside Citadel|L|-|c=GWU t
Secure the Wastes|I|W|tokx warrior
Shamanic Revelation|S|3GG|drawcre
Skyclave Apparition|C|1WW|pow=2 rem=exile tgt=nl etb mv4
Skyshroud Claim|S|3G|lr=2
Spectator Seating|L|-|c=A
Spire of Industry|L|-|c=A
Sterling Grove|E|GW|x
Sun Titan|C|4WW|pow=6 vig suntitan
Sylvan Library|E|1G|eng=1
Talisman of Dominance|A|2|rock=1:UB
Temple of Malice|L|-|c=BR t
Temple of Silence|L|-|c=WB t
The Dawning Archaic|C|10|pow=7 leg dawning
The World Tree|L|-|c=G t worldtree
Trostani's Summoner|C|5GW|pow=1 tokbig
Ultimate Magic: Holy|I|2W|prot=indes
Unbreakable Formation|I|2W|prot=indes
Vandalblast|S|R|rem=destroy tgt=a wipe=vandal
Vanquish the Horde|S|6WW|wipe=destroy perCreature
Vivid Creek|L|-|c=A t
Vivid Grove|L|-|c=A t
Vivid Marsh|L|-|c=A t
Wargate|S|2GWU|tut=perm
Warleader's Call|E|1RW|warleader
Wispdrinker Vampire|C|2WB|pow=2 tgh=4 fly wisp
Wurmcoil Engine|AC|6|pow=6 dt lifelink wurmcoil
Imperial Recruiter|C|2R|pow=1 noatk recruit
Inventors' Fair|L|-|c=C fair
Storm-Kiln Artist|C|3R|pow=2 noatk shaman kiln
Birgi, God of Storytelling // Harnfel, Horn of Bounty|C|2R|pow=3 noatk leg birgi
Birgi, God of Storytelling|C|2R|pow=3 noatk leg birgi
Evacuation|I|3UU|wipe=evac
Talrand, Sky Summoner|C|2UU|pow=2 noatk leg wizard spelltok=2 spelltokfly
Young Pyromancer|C|1R|pow=2 tgh=1 noatk shaman spelltok=1
Third Path Iconoclast|C|UR|pow=2 tgh=1 noatk spelltok=1
Kessig Flamebreather|C|1R|pow=1 tgh=3 noatk shaman ping=1
Firebrand Archer|C|1R|pow=2 tgh=1 noatk ping=1
Insatiable Avarice|S|B|avarice
Grim Tutor|S|1BB|tut=any lose=3
Aura Shards|E|1GW|shards
Ancient Tomb|L|-|c=C amt=2 tomb
Mishra's Workshop|L|-|c=C amt=3 workshop
Glacial Chasm|L|-|c=C chasm
The Tabernacle at Pendrell Vale|L|-|c=C leg tabernacle
Field of the Dead|L|-|c=C t fotd
Mana Vault|A|1|rock=3:C nountap vaultping
Grim Monolith|A|2|rock=3:C nountap grim
Chrome Mox|A|0|chromemox
Mox Diamond|A|0|rock=1:A moxd
Lion's Eye Diamond|A|0|led
Fierce Guardianship|I|2U|ctr=nc fierce
Gifts Ungiven|I|3U|gifts
Intuition|I|2U|intuition
Jeska's Will|S|2R|jeska
Underworld Breach|E|1R|breach
Panoptic Mirror|A|5|panoptic
Ad Nauseam|I|3BB|adnaus
Bolas's Citadel|A|3BBB|leg citadel
Braids, Cabal Minion|C|2BB|pow=2 leg braids
Imperial Seal|S|B|seal
Vampiric Tutor|I|B|seal
Necropotence|E|BBB|necro
Opposition Agent|C|2B|pow=3 tgh=2 flash agent
Tergrid, God of Fright // Tergrid's Lantern|C|3BB|pow=4 tgh=5 leg tergrid
"""