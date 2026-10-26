# Godo, Bandit Warlord — Mono-Red cEDH Equipment Turbo

Make mana as fast as possible, cast Godo, and let his enter trigger put Helm of the Host onto the battlefield. With
Helm attached, every combat makes a hasty token copy of Godo, every copy's first attack adds another combat, and the
turn never ends. The backup win, Dualcaster Mage plus a copy spell, does not need Godo at all. The main win is a
chain of triggers rather than a spell, so free counterspells don't stop it. It is weak to creature removal in
response to Godo, to artifact hate (Null Rod, Collector Ouphe) and to trigger counters (Stifle, Tishana's
Tidebinder).

| | |
|---|---|
| **Commander** | Godo, Bandit Warlord ({5}{R}, 3/3, mono-red) |
| **Bracket** | 5 (cEDH; nine Game Changers) |
| **Simulator** | not in the simulator; the opponent pools top out at high Bracket 4. History in `decklists/CHANGELOG.md` |

## Strategy

### The commander

Godo ({5}{R}, 3/3, Legendary Human Barbarian) reads: *When Godo enters, you may search your library for an Equipment card, put it onto the battlefield, then shuffle. Whenever Godo attacks for the first time each turn, untap it and all Samurai you control. After this phase, there is an additional combat phase.*

Two things make Godo a cEDH commander. The enter trigger is a free tutor that puts the Equipment straight onto the battlefield, so Godo is effectively a one-card combo with Helm of the Host. And the extra-combat trigger is "first time each turn" *per object*: each token copy of Godo is a new object, so each one's first attack adds another combat. The Samurai clause does nothing here.

Godo costs six, the most expensive part of the deck. Everything else (18 rocks, five rituals, five mana creatures, Ancient Tomb and City of Traitors) exists to cast him on turn 2 or 3. Ruby Medallion takes him to five. Command Beacon puts him back in your hand from the command zone, so a recast doesn't pay commander tax.

### Game plan

**Opening hand.** Keep two or more mana sources with fast mana among them, plus a path to the win: Helm, Hammer of Nazahn, a tutor, or Dualcaster Mage with a copy spell. Ship all-land hands, one land with no rocks, and hands with no acceleration and no tutor. The first mulligan is free in multiplayer.

**Turns 1–2.** Deploy the fast mana: Sol Ring, Mana Vault, Grim Monolith, the Moxes, Lotus Petal, Ancient Tomb, City of Traitors. Set up the support pieces that make the combo cheaper: Hammer of Nazahn (any later Equipment attaches itself), Brass Squire, Goblin Welder. A Blood Moon or Trinisphere against a greedy table buys time (see Sequencing).

**The win turn.** Cast Godo with six mana, plus either Hammer already out, an untapped Brass Squire, Magnetic Theft ({R}) or {5} to attach Helm. Hold Deflecting Swat (free with Godo out), Pyroblast or Red Elemental Blast for the response. If Godo's trigger is countered or he is removed, rebuild with Cursed Mirror or Jaxis, or switch to the Dualcaster line.

### The Helm loop

**Helm of the Host** ({4}, equip {5}): at the beginning of combat on your turn, create a token copy of the equipped creature, except it isn't legendary, and it gains haste. On Godo:

1. Godo enters and fetches Helm. Attach it: free with Hammer of Nazahn out (Hammer attaches each Equipment as it enters), by tapping Brass Squire, with Magnetic Theft, or by paying {5}.
2. Beginning of combat: Helm makes a hasty token Godo. Its enter trigger fetches another Equipment (Hammer if it's missing, otherwise Sting or Conqueror's Flail).
3. The token attacks. Its "first time each turn" trigger adds a combat phase after this one.
4. The new combat begins, Helm makes another hasty token, and it repeats. Each combat adds one more 3/3 attacker until the table is dead.

Godo himself never has to attack, so his summoning sickness doesn't matter. Hammer of Nazahn ({4}) also gives the equipped creature +2/+0 and indestructible, so Godo with Hammer survives damage-based and destroy removal.

### Doubling and copying Godo

- **Peter Parker's Camera** ({1}, enters with three film counters; {2}, {T}, remove a film counter: copy target activated or triggered ability you control) copies Godo's enter trigger. The copy resolves first and fetches Hammer of Nazahn; the original then fetches Helm, which Hammer attaches as it enters. That is 8 mana with Camera already out (9 if you cast it the same turn), with no other piece needed.
- **Cursed Mirror** ({2}{R}, taps for {R}) can enter as a copy of any creature until end of turn, with haste. As a copy of Godo it gets the enter trigger (fetch Helm) and attacks for an extra combat. **Sculpting Steel** can enter as a copy of Cursed Mirror and then become a copy of Godo the same way.
- **Jaxis, the Troublemaker** ({R}, {T}, discard a card, sorcery speed) makes a hasty token copy of another creature you control. A Jaxis copy of Godo fetches Helm and attacks. Blitz ({1}{R}) gives Jaxis haste, so it works the turn it lands.

These are the answer to a Stifle on Godo's trigger: Godo is still on the battlefield, so a copy gets a fresh trigger.

### The Dualcaster loop

**Dualcaster Mage** ({1}{R}{R}, flash; when it enters, copy target instant or sorcery spell) wins without Godo. Cast **Twinflame**, **Heat Shimmer**, **Molten Duplication** or **Electroduplicate** (all sorceries that make a hasty token copy of a creature) targeting any creature you control, and hold priority. Flash in Dualcaster. Its trigger copies the spell, targeting Dualcaster; the token copy of Dualcaster enters with haste and copies the original spell again, and so on. Stop at as many hasty 2/2s as you need, then attack. Do it precombat on your own turn. It costs five or six mana, and **Imperial Recruiter** finds Dualcaster (power 2).

### Tutors and card flow

- **Gamble** ({R}): any card to your hand, then discard a card at random. Cast it with a full hand. A discarded Helm or Hammer isn't lost while Goblin Welder is out.
- **Imperial Recruiter** ({2}{R}): a creature with power 2 or less. Dualcaster, Brass Squire, Goblin Welder, Hexing Squelcher, Jaxis and Magus of the Moon all qualify.
- **Goblin Engineer** ({1}{R}): puts any artifact from your library into the graveyard. With **Goblin Welder** ({R}; {T}: swap an artifact you control for an artifact card in your graveyard) that becomes "put Hammer or Helm onto the battlefield for {R}". Engineer's own ability returns artifacts with mana value 3 or less, which covers Sting, Conqueror's Flail and most rocks.
- **Tezzeret, Cruel Captain** ({3}, loyalty 4, +1 loyalty whenever an artifact enters): −3 finds an artifact with mana value 1 or less (Camera, Sol Ring, Mana Vault, a Key); 0 untaps an artifact, so it's another mana rock on a Monolith or Thran Dynamo.
- **Urza's Saga** finds a {0} or {1} artifact on chapter III; **Inventors' Fair** finds any artifact (Helm) with three artifacts out.
- **Card flow:** The One Ring (protection from everything the turn you cast it, then growing draws), Wheel of Fortune, Party Thrasher (discard, then exile two and play one each turn), Ragavan (Treasure and a card from the defending player's library) and Jeska's Will (with Godo out, both modes: red mana for each card in an opponent's hand and three cards from your library).

### Fast mana

- **Zero and one mana:** Sol Ring, Mana Vault, Chrome Mox, Mox Diamond, Mox Opal (needs three artifacts), Lotus Petal, Lion's Eye Diamond, Everflowing Chalice, Arcane Signet, Voltaic Key and Manifold Key (each untaps an artifact for {1}: a Monolith or Thran Dynamo is +2).
- **Big rocks:** Grim Monolith, Basalt Monolith, Thran Dynamo, Honor-Worn Shaku (untaps by tapping a legendary permanent, such as Godo, Helm or Hammer), Staff of Compleation (any colour for 2 life), Krark-Clan Ironworks (sacrifice an artifact for {C}{C}).
- **Creatures:** Metalworker ({C}{C} per artifact card revealed from hand), Birgi, God of Storytelling ({R} per spell you cast, kept through the turn), Treasonous Ogre (pay 3 life: {R}), _____ Goblin ({2}{R}, adds {R} for each unique vowel on the name sticker you give it), Simian Spirit Guide (exile from hand for {R}).
- **Rituals:** Rite of Flame, Desperate Ritual, Pyretic Ritual, Seething Song, Jeska's Will.
- **Lands:** Ancient Tomb, City of Traitors, Crystal Vein, Dwarven Ruins and Sandstone Needle make two at once.

**Lion's Eye Diamond** (discard your hand, sacrifice: three mana of one colour) is best in response to your own Wheel of Fortune: you lose a hand you were discarding anyway and draw seven.

### Interaction and protection

- **Against blue:** Pyroblast and Red Elemental Blast (counter a blue spell or destroy a blue permanent), Ricochet Trap ({R} if an opponent cast a blue spell this turn: change the target of a spell with one target).
- **Redirects:** Deflecting Swat (free while you control a commander; new targets for a spell or ability), Redirect Lightning ({R} plus 5 life or {2}), Untimely Malfunction (redirect, destroy an artifact, or two creatures can't block).
- **Counters and removal:** Tibalt's Trickery (counter; its controller casts a random free spell instead), Chaos Warp, Pyrokinesis (free by exiling a red card; 4 damage divided among creatures), Blasphemous Act, Shatterskull Smashing.
- **Protecting the combo:** Hexing Squelcher (can't be countered; your spells can't be countered; your other creatures have ward—pay 2 life), Defense Grid (each spell costs {3} more except on its controller's turn), Conqueror's Flail (while attached, opponents can't cast spells during your turn), Vexing Bauble (counters any spell cast with no mana spent, such as Force of Will or Fierce Guardianship).
- **Cavern of Souls** naming Human makes Godo, Dualcaster Mage, Imperial Recruiter, Jaxis and Magus of the Moon uncounterable.

### Stax

**Blood Moon** and **Magus of the Moon** turn every nonbasic land into a Mountain, which shuts down multicolour cEDH mana bases while your 14 Mountains and Great Furnace keep working. **Trinisphere** makes every spell cost at least three while it's untapped, which hurts the table's free and cheap interaction more than your deployed rocks.

### How it wins

- **Godo + Helm of the Host:** infinite combats with a growing army of hasty Godo tokens.
- **Dualcaster + a copy spell:** as many hasty 2/2s as you need, without Godo.
- **Final Fortune / Last Chance:** an extra turn so tapped or summoning-sick pieces can finish. You lose at the end of that turn, so cast it only when the extra turn wins.

### Weak spots and how to play around them

- **Creature removal in response.** Godo's trigger still resolves if Godo dies in response, but Helm then has nothing to copy. Hold Deflecting Swat or a redirect, attach with Hammer (indestructible), or rebuild with Cursed Mirror or Jaxis.
- **Trigger counters.** Stifle and Tishana's Tidebinder stop the enter trigger. The copy effects (Camera, Cursed Mirror, Jaxis) give a second trigger, and Dualcaster doesn't use triggers that matter.
- **Artifact hate.** Null Rod and Collector Ouphe turn off most of the mana and Camera, and an artifact wipe sets you back a turn or two. Keep mana creatures, rituals and lands, and remove the hate piece with Chaos Warp or Untimely Malfunction.
- **Graveyard hate** only matters for the Welder and Engineer lines.

## Key lines

**Godo + Helm, 6 to 11 mana.** Godo (6) fetches Helm; attach free with Hammer out, by tapping Brass Squire, with Magnetic Theft (+1) or with equip (+5). Go to combat.

**Godo + Camera, 8 mana with Camera out.** Copy Godo's enter trigger: the copy fetches Hammer, the original fetches Helm, Hammer attaches it.

**Engineer + Welder for Hammer, then Godo.** With Welder out since your last turn, Goblin Engineer puts Hammer into the graveyard and Welder swaps a used rock for it. Godo then fetches Helm and Hammer attaches it.

**Rebuild after a Stifle.** Cursed Mirror (or Sculpting Steel copying Cursed Mirror) or Jaxis copies Godo: the copy fetches Helm, attaches it and attacks for an extra combat, and the Helm loop runs from there.

**Dualcaster, 5 to 6 mana.** Copy spell on your creature, Dualcaster in response, its copy targets Dualcaster, repeat. Precombat, own turn.

**Sequencing.**
- **Blood Moon and Magus hit your own lands too.** Under them Ancient Tomb and City of Traitors tap for only {R}, and Urza's Saga loses its chapters and is sacrificed at once. Use the fast lands and Saga first, or keep the Moon for a table whose mana base it breaks.
- **Vexing Bauble counters your own free spells:** Moxes, Lotus Petal, Lion's Eye Diamond, a free Deflecting Swat or a pitched Pyrokinesis. Deploy the zero-drops before Bauble, or sacrifice it to draw first.
- **Trinisphere** makes your own zero- and one-mana spells cost three while it's untapped. Play it after the cheap rocks are down.
- **Deflecting Swat and Jeska's Will's double mode** need a commander on the battlefield; cast them after Godo.

## Tutor targets by board state

- **Godo in hand or in the command zone, Hammer not out:** Goblin Engineer + Welder, or Gamble, for Hammer; then Godo fetches Helm.
- **Hammer out:** just cast Godo for Helm.
- **Godo's trigger keeps getting stopped:** Imperial Recruiter for Hexing Squelcher (your spells can't be countered) or Tezzeret −3 for Peter Parker's Camera.
- **No Godo line available:** Imperial Recruiter for Dualcaster Mage; Gamble for the copy spell.
- **Short on mana:** Urza's Saga or Tezzeret −3 for Mana Vault or Sol Ring.
- **Opponents are on blue interaction:** Gamble for Pyroblast or Deflecting Swat before going for it.
- **Need cards:** The One Ring or Wheel of Fortune (with Lion's Eye Diamond in response).

## Mana and consistency

The list is exactly 100 cards: Godo plus 99, singleton except 14 Mountains, all red or colourless.

29 lands (28 plus Shatterskull Smashing, a land on its back), 18 rocks, five mana creatures and five rituals.

| Measure | Count | Note |
| --- | --- | --- |
| Red-producing lands | 18 | 14 Mountain, Great Furnace, Dwarven Ruins, Sandstone Needle, Shatterskull, the Hammer Pass |
| Red sometimes | 4 | Gemstone Caverns (with a luck counter), Talon Gates of Madara and Abstergo Entertainment ({1}, {T}: any colour), Cavern of Souls (creatures of the chosen type) |
| Colourless-only lands | 7 | Ancient Tomb, City of Traitors, Crystal Vein, Urza's Saga, Inventors' Fair, Command Beacon, Urza's Cave |
| Two mana from one land | 5 | Ancient Tomb, City of Traitors, Crystal Vein (sacrifice), Dwarven Ruins (sacrifice), Sandstone Needle |
| Always enter tapped | 2 | Dwarven Ruins, Sandstone Needle |
| Sometimes enter tapped | 1 | Shatterskull, the Hammer Pass (unless you pay 3 life) |

**Curve.** Godo costs six, Blasphemous Act and Pyrokinesis have cheaper ways to cast them, and nothing else costs more than four. Most of the deck costs zero to two.

**Card types.** 15 creatures, 14 instants, 11 sorceries (including Shatterskull Smashing), 29 artifacts, 1 enchantment, 1 planeswalker, 28 lands, plus Godo.

**Redundancy.** Two independent wins, each with several ways to assemble it: Helm comes from Godo, Gamble, Engineer + Welder or Inventors' Fair; the attach from Hammer, Brass Squire, Magnetic Theft or equip; Dualcaster from Recruiter or Gamble with four copy spells.

## Bracket and Rule 0

**Bracket 5 (cEDH).** Nine Game Changers: **Ancient Tomb**, **Chrome Mox**, **Gamble**, **Grim Monolith**, **Jeska's Will**, **Lion's Eye Diamond**, **Mana Vault**, **Mox Diamond** and **The One Ring**. Mana Crypt, Jeweled Lotus and Dockside Extortionist are banned and not in the list.

Disclose before the game:

- **Godo + Helm of the Host** is a one-card infinite combat combo (Godo tutors Helm).
- **Dualcaster Mage + any of four copy spells** is an infinite hasty-token combo.
- **Stax:** Blood Moon, Magus of the Moon, Trinisphere, Defense Grid, Conqueror's Flail.
- **Extra turns:** Final Fortune and Last Chance.

Re-check the official list at https://commanderbrackets.com/faq before an event.

## Decklist by type (100)

**Commander (1).** Godo, Bandit Warlord

**Planeswalkers (1).** Tezzeret Cruel Captain

**Creatures (15).** _____ Goblin, Birgi God of Storytelling // Harnfel Horn of Bounty, Brass Squire, Dualcaster Mage, Goblin Engineer, Goblin Welder, Hexing Squelcher, Imperial Recruiter, Jaxis the Troublemaker, Magus of the Moon, Metalworker, Party Thrasher, Ragavan Nimble Pilferer, Simian Spirit Guide, Treasonous Ogre

**Artifacts (29).** Arcane Signet, Basalt Monolith, Chrome Mox, Conqueror's Flail, Cursed Mirror, Defense Grid, Everflowing Chalice, Grim Monolith, Hammer of Nazahn, Helm of the Host, Honor-Worn Shaku, Krark-Clan Ironworks, Lion's Eye Diamond, Lotus Petal, Mana Vault, Manifold Key, Mox Diamond, Mox Opal, Peter Parker's Camera, Ruby Medallion, Sculpting Steel, Sol Ring, Staff of Compleation, Sting the Glinting Dagger, The One Ring, Thran Dynamo, Trinisphere, Vexing Bauble, Voltaic Key

**Enchantments (1).** Blood Moon

**Instants (14).** Chaos Warp, Deflecting Swat, Desperate Ritual, Final Fortune, Magnetic Theft, Pyretic Ritual, Pyroblast, Pyrokinesis, Red Elemental Blast, Redirect Lightning, Ricochet Trap, Seething Song, Tibalt's Trickery, Untimely Malfunction

**Sorceries (11).** Blasphemous Act, Electroduplicate, Gamble, Heat Shimmer, Jeska's Will, Last Chance, Molten Duplication, Rite of Flame, Shatterskull Smashing // Shatterskull the Hammer Pass, Twinflame, Wheel of Fortune

**Lands (28).** Abstergo Entertainment, Ancient Tomb, Cavern of Souls, City of Traitors, Command Beacon, Crystal Vein, Dwarven Ruins, Gemstone Caverns, Great Furnace, Inventors' Fair, Sandstone Needle, Talon Gates of Madara, Urza's Cave, Urza's Saga, 14 Mountain

## Import list (100)

```
1 _____ Goblin
1 Abstergo Entertainment
1 Ancient Tomb
1 Arcane Signet
1 Basalt Monolith
1 Birgi, God of Storytelling
1 Blasphemous Act
1 Blood Moon
1 Brass Squire
1 Cavern of Souls
1 Chaos Warp
1 Chrome Mox
1 City of Traitors
1 Command Beacon
1 Conqueror's Flail
1 Crystal Vein
1 Cursed Mirror
1 Defense Grid
1 Deflecting Swat
1 Desperate Ritual
1 Dualcaster Mage
1 Dwarven Ruins
1 Electroduplicate
1 Everflowing Chalice
1 Final Fortune
1 Gamble
1 Gemstone Caverns
1 Goblin Engineer
1 Goblin Welder
1 Godo, Bandit Warlord
1 Great Furnace
1 Grim Monolith
1 Hammer of Nazahn
1 Heat Shimmer
1 Helm of the Host
1 Hexing Squelcher
1 Honor-Worn Shaku
1 Imperial Recruiter
1 Inventors' Fair
1 Jaxis, the Troublemaker
1 Jeska's Will
1 Krark-Clan Ironworks
1 Last Chance
1 Lion's Eye Diamond
1 Lotus Petal
1 Magnetic Theft
1 Magus of the Moon
1 Mana Vault
1 Manifold Key
1 Metalworker
1 Molten Duplication
1 Mox Diamond
1 Mox Opal
1 Party Thrasher
1 Peter Parker's Camera
1 Pyretic Ritual
1 Pyroblast
1 Pyrokinesis
1 Ragavan, Nimble Pilferer
1 Red Elemental Blast
1 Redirect Lightning
1 Ricochet Trap
1 Rite of Flame
1 Ruby Medallion
1 Sandstone Needle
1 Sculpting Steel
1 Seething Song
1 Shatterskull Smashing
1 Simian Spirit Guide
1 Sol Ring
1 Staff of Compleation
1 Sting, the Glinting Dagger
1 Talon Gates of Madara
1 Tezzeret, Cruel Captain
1 The One Ring
1 Thran Dynamo
1 Tibalt's Trickery
1 Treasonous Ogre
1 Trinisphere
1 Twinflame
1 Untimely Malfunction
1 Urza's Cave
1 Urza's Saga
1 Vexing Bauble
1 Voltaic Key
1 Wheel of Fortune
14 Mountain
```
