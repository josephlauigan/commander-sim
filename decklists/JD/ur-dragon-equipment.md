# The Ur-Dragon — Five-Colour Dragons and Equipment

A five-colour combo deck that wins with Godo, Bandit Warlord and Helm of the Host, with Equipment and Dragons as the
value engine around it. **Godo** puts Helm of the Host onto the battlefield when he enters; Helm makes a hasty token
copy of him every combat, and each copy's first attack adds another combat, so the turn never ends. **Gilgamesh,
Master-at-Arms** is a Samurai, so every Godo attack untaps him and he digs six cards for Equipment on every attack.
**Maskwood Nexus** makes every creature a Dragon (Eminence discount, Ur-Dragon draws) and a Samurai (Godo untaps the
whole team). The Ur-Dragon itself is a nine-drop: in most games it is a cost reducer from the command zone and a late
card engine, and the deck wins before it is cast. It is weak to artifact hate (Null Rod, Collector Ouphe, Vandalblast),
to trigger counters on Godo, and to fast combo that wins before turn 5.

| | |
|---|---|
| **Commander** | The Ur-Dragon ({4}{W}{U}{B}{R}{G}, 10/10, five colours) |
| **Bracket** | 4 (16 Game Changers; a two-card infinite combo) |
| **Simulator** | Not wired in yet. Eminence and The Ur-Dragon's attack trigger are modeled in PR #139 (`impls/t45_urdragon.rs`); build on that when adding the deck key. |

## Strategy

### The commander

The Ur-Dragon ({4}{W}{U}{B}{R}{G}, Legendary Creature — Dragon Avatar, 10/10, flying) reads: *Eminence — As long as
The Ur-Dragon is in the command zone or on the battlefield, other Dragon spells you cast cost {1} less to cast.
Whenever one or more Dragons you control attack, draw that many cards, then you may put a permanent card from your
hand onto the battlefield.*

- **Eminence works from the command zone.** Every Dragon costs one less from turn 1, and with Maskwood Nexus out every
  creature does, Godo and Gilgamesh included.
- **The attack trigger counts every attacking Dragon**, every combat. With Maskwood Nexus and Godo's extra combats it
  draws a card per attacker per combat, and the free permanent is Kaldra Compleat, Helm of the Host or a Dragon.
- **He costs nine.** Don't wait for him: the deck's plan is Godo.

### Game plan

**Opening hand.** Keep two or more mana sources with fast mana or a mana creature among them, plus a path to Godo: Godo
himself, Imperial Recruiter, Worldly Tutor or Demonic Tutor. Ship hands with no acceleration.

**Turns 1-3.** Fast mana, mana creatures, land ramp. Puresteel Paladin or Stoneforge Mystic if there is a spare mana.

**Turns 3-5: the win turn.** Cast Godo with six mana (five with Maskwood Nexus out), plus Hammer of Nazahn already out,
an untapped Brass Squire, Puresteel Paladin with metalcraft, Sigarda's Aid, or {5} to attach Helm. Hold Force of Will,
Pact of Negation or Teferi's Protection for the response. Grand Abolisher on the battlefield, or Conqueror's Flail
equipped first, means opponents can't cast spells during your turn at all.

**Late game.** If the combo is answered, the Dragons and Equipment win the long game: Terror of the Peaks, the hasty
Dragons, Kaldra Compleat, and The Ur-Dragon's card draw.

### The Helm loop

**Helm of the Host** ({4}, equip {5}): at the beginning of combat on your turn, create a token copy of the equipped
creature, except it isn't legendary, and it gains haste. On Godo:

1. Godo enters and fetches Helm. Attach it: free with Hammer of Nazahn or Sigarda's Aid out, free with Puresteel Paladin
   and three artifacts, by tapping Brass Squire, or by paying {5}.
2. Beginning of combat: Helm makes a hasty token Godo. Its enter trigger fetches another Equipment (Hammer if it's
   missing, otherwise Kaldra Compleat or Conqueror's Flail).
3. The token attacks: untap it and all Samurai, and add a combat.
4. The new combat begins, Helm makes another hasty token, and it repeats until the table is dead.

Godo himself never has to attack. Backups when the trigger is countered or Godo is removed: **Peter Parker's Camera**
copies Godo's enter trigger (Hammer first, then Helm, which Hammer attaches), and **Jaxis, the Troublemaker** makes a
hasty token copy of Godo, with a fresh enter trigger.

### Gilgamesh and the Samurai

**Gilgamesh, Master-at-Arms** ({4}{R}{R}, 6/6, Human Samurai): whenever he enters or attacks, look at the top six cards
and put any number of Equipment from among them onto the battlefield; attach one to a Samurai.

- Godo untaps all Samurai on his first attack each turn. In the Helm loop every Godo token's first attack untaps
  Gilgamesh, so he attacks every combat and digs six cards every combat.
- **Sting, the Glinting Dagger** untaps the equipped creature at the beginning of each combat, so on Gilgamesh he
  attacks each combat without Godo.
- **Helm of the Host on Gilgamesh** makes a Gilgamesh token every combat, and each token's enter trigger digs again.

### Maskwood Nexus

*Creatures you control are every creature type. The same is true for creature spells you control and creature cards you
own that aren't on the battlefield.*

- **Every creature is a Dragon:** Godo, Gilgamesh and the mana creatures cost {1} less, and every attacker counts for
  The Ur-Dragon's draw.
- **Every creature is a Samurai:** Godo's attack untaps the whole team for the extra combat.

### The Equipment

| Card | Role |
|---|---|
| **Helm of the Host** | The combo piece (above). |
| **Hammer of Nazahn** | Attaches every Equipment as it enters, so Helm goes onto Godo the moment it's fetched; +2/+0 and indestructible. |
| **Kaldra Compleat** | A 5/5 indestructible haste germ with first strike and trample: the best free permanent for Gilgamesh and The Ur-Dragon. |
| **Embercleave** | Flash, costs less per attacker, double strike and trample, attaches as it enters. |
| **Sword of Fire and Ice** | Protection from red and blue; 2 damage and a card on each hit. |
| **Sword of Feast and Famine** | Protection from black and green; discard, and your lands untap. |
| **Shadowspear** | Trample and lifelink; strips hexproof and indestructible from opponents' permanents. |
| **Conqueror's Flail** | Opponents can't cast spells during your turn while it's equipped: protects the win turn. |
| **Sting, the Glinting Dagger** | Haste, and untaps the equipped creature every combat. |
| **Lightning Greaves, Swiftfoot Boots** | Haste and protection. |

Support: **Puresteel Paladin** (equip {0} with metalcraft, a card per Equipment), **Sigarda's Aid** (flash Equipment,
attach on entering), **Brass Squire** (attach for free), **Stoneforge Mystic**, **Steelshaper's Gift** and
**Enlightened Tutor** (find Equipment), **Inventors' Fair** (finds Helm with three artifacts).

### Dragons

- **Hasty:** Glorybringer (exert for 4 to a creature), Thundermaw Hellkite (taps every opposing flier), Smaug the
  Magnificent (a Treasure each upkeep, damage equal to your Treasures on attack), Goldspan Dragon (Treasures that make
  two mana).
- **Engines:** Terror of the Peaks (damage equal to each creature's power as it enters; spells that target it cost 3
  life), Hellkite Tyrant (steals artifacts, wins with 20 artifacts at your upkeep), Nicol Bolas, the Ravager (each
  opponent discards; transforms into a planeswalker that draws two or deals 10), Dragonlord Ojutai (hexproof while
  untapped, card selection on each hit), Kura, the Boundless Sky (flying deathtouch; three lands or an X/X when it
  dies).

### Backup combo

**Dualcaster Mage** ({1}{R}{R}, flash) plus **Twinflame**: cast Twinflame on any creature you control and hold priority,
flash in Dualcaster, copy Twinflame targeting Dualcaster; each token Dualcaster copies the spell again. Stop at as many
hasty 2/2s as you need. **Imperial Recruiter** finds Dualcaster or Godo (power 2 and 3).

### Interaction and protection

- **Stack:** Force of Will, Force of Negation (noncreature spells; free on an opponent's turn), Pact of Negation, Mana Drain (the mana comes
  back on your next main phase: put it toward Godo).
- **Removal:** Swords to Plowshares, Decimate, Chaos Warp, Cyclonic Rift.
- **Board wipe:** **Kindred Dominance** ({5}{B}{B}, sorcery): name Dragon and every creature that isn't a Dragon is
  destroyed. With Maskwood Nexus out all your creatures are Dragons, so it is one-sided; without it, Godo, the mana
  creatures and the Paladins die too.
- **Protection:** Grand Abolisher (opponents can't cast spells or activate abilities of artifacts, creatures or
  enchantments during your turn), Teferi's Protection, Conqueror's Flail, Lightning Greaves, Swiftfoot Boots.

### Card flow

The One Ring, Rhystic Study, Sylvan Library, Garruk's Uprising (a card for each creature with power 4 or more, and
trample), Wrenn and Six (returns fetch lands), Puresteel Paladin, and The Ur-Dragon's attack trigger.

## Key lines

- **Turn 1-2:** fast mana and a mana creature; Puresteel Paladin or Stoneforge Mystic.
- **Godo turn:** six mana plus a way to attach Helm; Grand Abolisher down or Conqueror's Flail equipped first if you
  have them.
- **Godo answered:** Camera or Jaxis copies Godo for a fresh trigger; otherwise switch to Dualcaster and Twinflame.
- **Maskwood turn:** every creature is a Dragon and a Samurai: cast creatures for {1} less and attack with everything.

## Mana and consistency

34 lands, every one untapped: the nine original dual lands (Badlands, Bayou, Plateau, Scrubland, Taiga, Tropical
Island, Tundra, Underground Sea, Volcanic Island), nine shock lands (all but Godless Shrine), eight fetch lands
(Bloodstained Mire, Flooded Strand, Marsh Flats, Misty Rainforest, Polluted Delta, Verdant Catacombs, Windswept Heath,
Wooded Foothills), City of Brass, Mana Confluence, Command Tower, Ancient Tomb, Cavern of Souls (name Dragon),
Inventors' Fair, Gaea's Cradle and one Forest. Every fetch finds a dual or shock land of two colours, and Nature's Lore
finds any land with the Forest type (Bayou, Taiga, Tropical Island and the green shock lands). Rampant Growth finds only
a basic, so it needs the Forest, which is also there for Blood Moon. Fast mana: Sol Ring, Mana Vault, Grim Monolith,
Chrome Mox, Mox Diamond and Lotus Petal, plus three mana creatures and two land-ramp spells.

**Gaea's Cradle** taps for {G} per creature you control. With a mana creature or two out it is a second Mana Vault, but
on turns 1-2 with no creatures it makes nothing, so don't keep it as one of only two lands.

## Bracket and Rule 0

Bracket 4: 16 Game Changers (Mana Vault, Grim Monolith, Chrome Mox, Mox Diamond, The One Ring, Rhystic Study,
Enlightened Tutor, Demonic Tutor, Vampiric Tutor, Imperial Seal, Worldly Tutor, Force of Will, Cyclonic Rift, Teferi's
Protection, Ancient Tomb, Gaea's Cradle)
and a two-card infinite combo (Godo plus Helm of the Host). Tell the table about the combo before the game.

## Decklist by type (100)
**Commander (1).** The Ur-Dragon

**Creatures (21).** Birds of Paradise, Brass Squire, Dragonlord Ojutai, Dualcaster Mage, Gilgamesh Master-at-Arms, Glorybringer, Godo Bandit Warlord, Goldspan Dragon, Grand Abolisher, Hellkite Tyrant, Imperial Recruiter, Jaxis the Troublemaker, Kura the Boundless Sky, Llanowar Elves, Nicol Bolas the Ravager, Paradise Druid, Puresteel Paladin, Smaug the Magnificent, Stoneforge Mystic, Terror of the Peaks, Thundermaw Hellkite

**Planeswalkers (1).** Wrenn and Six

**Enchantments (4).** Garruk's Uprising, Rhystic Study, Sigarda's Aid, Sylvan Library

**Artifacts (20).** Chrome Mox, Conqueror's Flail, Embercleave, Grim Monolith, Hammer of Nazahn, Helm of the Host, Kaldra Compleat, Lightning Greaves, Lotus Petal, Mana Vault, Maskwood Nexus, Mox Diamond, Peter Parker's Camera, Shadowspear, Sol Ring, Sting the Glinting Dagger, Swiftfoot Boots, Sword of Feast and Famine, Sword of Fire and Ice, The One Ring

**Sorceries (8).** Decimate, Demonic Tutor, Imperial Seal, Kindred Dominance, Nature's Lore, Rampant Growth, Steelshaper's Gift, Twinflame

**Instants (11).** Chaos Warp, Cyclonic Rift, Enlightened Tutor, Force of Negation, Force of Will, Mana Drain, Pact of Negation, Swords to Plowshares, Teferi's Protection, Vampiric Tutor, Worldly Tutor

**Lands (34).** Ancient Tomb, Badlands, Bayou, Blood Crypt, Bloodstained Mire, Breeding Pool, Cavern of Souls, City of Brass, Command Tower, Flooded Strand, Gaea's Cradle, Hallowed Fountain, Inventors' Fair, Mana Confluence, Marsh Flats, Misty Rainforest, Overgrown Tomb, Plateau, Polluted Delta, Sacred Foundry, Scrubland, Steam Vents, Stomping Ground, Taiga, Temple Garden, Tropical Island, Tundra, Underground Sea, Verdant Catacombs, Volcanic Island, Watery Grave, Windswept Heath, Wooded Foothills, 1 Forest

## Import list (100)

```
1 Ancient Tomb
1 Badlands
1 Bayou
1 Birds of Paradise
1 Blood Crypt
1 Bloodstained Mire
1 Brass Squire
1 Breeding Pool
1 Cavern of Souls
1 Chaos Warp
1 Chrome Mox
1 City of Brass
1 Command Tower
1 Conqueror's Flail
1 Cyclonic Rift
1 Decimate
1 Demonic Tutor
1 Dragonlord Ojutai
1 Dualcaster Mage
1 Embercleave
1 Enlightened Tutor
1 Flooded Strand
1 Force of Negation
1 Force of Will
1 Gaea's Cradle
1 Garruk's Uprising
1 Gilgamesh, Master-at-Arms
1 Glorybringer
1 Godo, Bandit Warlord
1 Goldspan Dragon
1 Grand Abolisher
1 Grim Monolith
1 Hallowed Fountain
1 Hammer of Nazahn
1 Hellkite Tyrant
1 Helm of the Host
1 Imperial Recruiter
1 Imperial Seal
1 Inventors' Fair
1 Jaxis, the Troublemaker
1 Kaldra Compleat
1 Kindred Dominance
1 Kura, the Boundless Sky
1 Lightning Greaves
1 Llanowar Elves
1 Lotus Petal
1 Mana Confluence
1 Mana Drain
1 Mana Vault
1 Marsh Flats
1 Maskwood Nexus
1 Misty Rainforest
1 Mox Diamond
1 Nature's Lore
1 Nicol Bolas, the Ravager
1 Overgrown Tomb
1 Pact of Negation
1 Paradise Druid
1 Peter Parker's Camera
1 Plateau
1 Polluted Delta
1 Puresteel Paladin
1 Rampant Growth
1 Rhystic Study
1 Sacred Foundry
1 Scrubland
1 Shadowspear
1 Sigarda's Aid
1 Smaug the Magnificent
1 Sol Ring
1 Steam Vents
1 Steelshaper's Gift
1 Sting, the Glinting Dagger
1 Stomping Ground
1 Stoneforge Mystic
1 Swiftfoot Boots
1 Sword of Feast and Famine
1 Sword of Fire and Ice
1 Swords to Plowshares
1 Sylvan Library
1 Taiga
1 Teferi's Protection
1 Temple Garden
1 Terror of the Peaks
1 The One Ring
1 The Ur-Dragon
1 Thundermaw Hellkite
1 Tropical Island
1 Tundra
1 Twinflame
1 Underground Sea
1 Vampiric Tutor
1 Verdant Catacombs
1 Volcanic Island
1 Watery Grave
1 Windswept Heath
1 Wooded Foothills
1 Worldly Tutor
1 Wrenn and Six
1 Forest
```
