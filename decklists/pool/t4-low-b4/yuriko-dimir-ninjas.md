# Yuriko — Dimir Ninja Tempo

Opponent pool deck — **Tier 4: Low Bracket 4**. Axis: Tempo / evasive chip damage.

- **Commander:** Yuriko, the Tiger's Shadow
- **Color identity:** Blue, Black
- **Game Changers (8):** Cyclonic Rift, Demonic Tutor, Fierce Guardianship, Force of Will, Imperial Seal, Mystical Tutor, Rhystic Study, Vampiric Tutor
- **Combos:** None infinite. A Yuriko reveal of Shadow of Mortality (15), Temporal Trespass (11), Dig Through Time or Treasure Cruise (8) is a big hit to every opponent; Temporal Mastery and Temporal Trespass give extra turns of Ninja hits.
- **Lands:** 30 (17 nonbasic + 13 basic), plus five spell // land cards (Agadeem's Awakening, Sea Gate Restoration, Hydroelectric Specimen, Fell the Profane, Sink into Stupor)

## Role in the pool

Damage that ignores your board and arrives from turn two. Tests cheap removal, blockers for tiny fliers, and whether combo turns get countered.

## At the table

- **Plan:** Attacks from turn two with cheap evasive creatures (Ornithopter, Faerie Seer, Slither Blade, Triton Shorestalker, Spectral Sailor), then ninjutsus Yuriko, the Tiger's Shadow or another Ninja in (Ninja of the Deep Hours, Skullsnatcher, Nashi, Kaito, Bane of Nightmares, Sakashima's Student). Counterspells hold up the turn.
- **Wins by:** Combat damage from Ninjas, with Yuriko revealing the top card and each opponent losing life equal to its mana value. Brainstorm, Scroll Rack, Sensei's Divining Top and the tutors that put a card on top (Vampiric Tutor, Imperial Seal, Scheming Symmetry, Lim-Dûl's Vault) stack Shadow of Mortality or Temporal Trespass for huge hits; the extra turns repeat them.
- **Watch out for:** Cheap evasive attackers early: keep blockers and removal ready. Cover of Darkness gives the Ninjas fear (only black or artifact creatures block them). Yuriko can come in from the command zone for {U}{B}. Kaito pumps the Ninjas and stuns your best blocker. Force of Will, Fierce Guardianship and Flare of Denial counter your key plays; Misdirection and Hydroelectric Specimen turn your removal aside.
- **Interaction:** Counterspell, Force of Will, Fierce Guardianship, Flare of Denial, Misdirection, Commandeer, Cyclonic Rift, Snuff Out, Dismember, Deadly Rollick, Fell the Profane, Sink into Stupor, An Offer You Can't Refuse, Baleful Strix; Dauthi Voidwalker and Skullsnatcher eat graveyards.

## Strategy

Attack with one-mana evasive creatures, ninjutsu Yuriko or another Ninja in, and reveal the top card for its mana value in damage to every opponent. Top-of-library control stacks the biggest card on top (Shadow of Mortality is 15 per opponent per hit). Satoru draws for every Ninja that comes in by ninjutsu, Ninja of the Deep Hours and Moon-Circuit Hacker draw on hits, and Temporal Mastery (miracle {1}{U}) or Temporal Trespass take another turn of hits. Backed by free and cheap counterspells.

## Sim modeling notes

Model these first; if any is Partial or unmodeled, this deck plays below its real strength and its results should be flagged.

Ninjutsu (return an unblocked attacker, put a Ninja onto the battlefield attacking; t4.rs, the new Ninjas' costs in t45_yuriko.rs); Yuriko reveal-for-MV damage to each opponent; top-of-library manipulation; free counterspells. Hand-written for this list (impls/t45_yuriko.rs): Kaito, Bane of Nightmares (Ninja creature on your turn, emblem, stun), Nashi, Skullsnatcher, Sakashima's Student, Satoru, the Infiltrator, Cover of Darkness and Shizo (fear), Mothdust Changeling and Wingcrafter (flying), the extra turns, Scheming Symmetry, Shadow of Mortality's cost, the spell // land cards and Clearwater Pathway's face. Approximate: Kaito (damage to him as a creature leaves loyalty), Nashi (casts at once, never plays a land), Satoru (a free cast counts as cast), Hydroelectric Specimen (its spell can't be countered). Gingerbrute is modelled elsewhere.

## Decklist by type (100)

**Commander (1).** Yuriko, the Tiger's Shadow

**Creatures (31).** Baleful Strix, Changeling Outcast, Dauthi Voidwalker, Faerie Seer, Fallen Shinobi, Gingerbrute, Higure, the Still Wind, Hydroelectric Specimen // Hydroelectric Laboratory, Ingenious Infiltrator, Mist-Cloaked Herald, Mist-Syndicate Naga, Mistblade Shinobi, Mockingbird, Moon-Circuit Hacker, Mothdust Changeling, Nashi, Moon Sage's Scion, Ninja of the Deep Hours, Ornithopter, Prosperous Thief, Sakashima's Student, Satoru, the Infiltrator, Shadow of Mortality, Silent-Blade Oni, Silver-Fur Master, Skullsnatcher, Slither Blade, Spectral Sailor, Tetsuko Umezawa, Fugitive, Thousand-Faced Shadow, Triton Shorestalker, Wingcrafter

**Planeswalkers (1).** Kaito, Bane of Nightmares

**Enchantments (3).** Cover of Darkness, Mystic Remora, Rhystic Study

**Artifacts (6).** Arcane Signet, Dimir Signet, Scroll Rack, Sensei's Divining Top, Sol Ring, Talisman of Dominance

**Sorceries (9).** Agadeem's Awakening // Agadeem, the Undercrypt, Demonic Tutor, Imperial Seal, Ponder, Scheming Symmetry, Sea Gate Restoration // Sea Gate, Reborn, Temporal Mastery, Temporal Trespass, Treasure Cruise

**Instants (19).** An Offer You Can't Refuse, Brainstorm, Commandeer, Counterspell, Cyclonic Rift, Dark Ritual, Deadly Rollick, Dig Through Time, Dismember, Fell the Profane // Fell Mire, Fierce Guardianship, Flare of Denial, Force of Will, Lim-Dûl's Vault, Misdirection, Mystical Tutor, Sink into Stupor // Soporific Springs, Snuff Out, Vampiric Tutor

**Lands (30).** Choked Estuary, Clearwater Pathway // Murkwater Pathway, Command Tower, Drowned Catacomb, Exotic Orchard, Flooded Strand, Misty Rainforest, Morphic Pool, Otawara, Soaring City, Polluted Delta, Reliquary Tower, Rogue's Passage, Shipwreck Marsh, Shizo, Death's Storehouse, Sunken Hollow, Underground River, Watery Grave, 7 Island, 6 Swamp

## Import list (100)

```
1 Agadeem's Awakening // Agadeem, the Undercrypt
1 An Offer You Can't Refuse
1 Arcane Signet
1 Baleful Strix
1 Brainstorm
1 Changeling Outcast
1 Choked Estuary
1 Clearwater Pathway // Murkwater Pathway
1 Command Tower
1 Commandeer
1 Counterspell
1 Cover of Darkness
1 Cyclonic Rift
1 Dark Ritual
1 Dauthi Voidwalker
1 Deadly Rollick
1 Demonic Tutor
1 Dig Through Time
1 Dimir Signet
1 Dismember
1 Drowned Catacomb
1 Exotic Orchard
1 Faerie Seer
1 Fallen Shinobi
1 Fell the Profane // Fell Mire
1 Fierce Guardianship
1 Flare of Denial
1 Flooded Strand
1 Force of Will
1 Gingerbrute
1 Higure, the Still Wind
1 Hydroelectric Specimen // Hydroelectric Laboratory
1 Imperial Seal
1 Ingenious Infiltrator
1 Kaito, Bane of Nightmares
1 Lim-Dûl's Vault
1 Misdirection
1 Mist-Cloaked Herald
1 Mist-Syndicate Naga
1 Mistblade Shinobi
1 Misty Rainforest
1 Mockingbird
1 Moon-Circuit Hacker
1 Morphic Pool
1 Mothdust Changeling
1 Mystic Remora
1 Mystical Tutor
1 Nashi, Moon Sage's Scion
1 Ninja of the Deep Hours
1 Ornithopter
1 Otawara, Soaring City
1 Polluted Delta
1 Ponder
1 Prosperous Thief
1 Reliquary Tower
1 Rhystic Study
1 Rogue's Passage
1 Sakashima's Student
1 Satoru, the Infiltrator
1 Scheming Symmetry
1 Scroll Rack
1 Sea Gate Restoration // Sea Gate, Reborn
1 Sensei's Divining Top
1 Shadow of Mortality
1 Shipwreck Marsh
1 Shizo, Death's Storehouse
1 Silent-Blade Oni
1 Silver-Fur Master
1 Sink into Stupor // Soporific Springs
1 Skullsnatcher
1 Slither Blade
1 Snuff Out
1 Sol Ring
1 Spectral Sailor
1 Sunken Hollow
1 Talisman of Dominance
1 Temporal Mastery
1 Temporal Trespass
1 Tetsuko Umezawa, Fugitive
1 Thousand-Faced Shadow
1 Treasure Cruise
1 Triton Shorestalker
1 Underground River
1 Vampiric Tutor
1 Watery Grave
1 Wingcrafter
1 Yuriko, the Tiger's Shadow
7 Island
6 Swamp
```
