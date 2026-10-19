# Omnath, Locus of Rage — Red-Green Landfall Elementals

> Built from the Titania deck (2026-10-19).

A red-green ramp deck that turns every land drop into a 5/5. **Omnath, Locus of Rage** makes a 5/5 red and green
Elemental each time a land enters under your control, and deals 3 damage to any target whenever Omnath or another
Elemental you control dies. The deck is built to put lands onto the battlefield in bulk (ramp spells, Harrow, Oracle of
Mul Daya, fetch lands) and to make the Elementals die on purpose (Goblin Bombardment, Terror of the Peaks, Blasphemous
Act). **Titania, Protector of Argoth** is in the 99: her 5/3 green Elementals are Elementals too, so each one that dies
is another 3 damage. It wins through the board, not through a combo, and it is weak to fast decks, to sweepers that
get Omnath before the tokens come, and to graveyard hate.

| | |
|---|---|
| **Commander** | Omnath, Locus of Rage ({3}{R}{G}, 5/5, red-green) |
| **Colour identity** | Red-green |
| **Bracket** | 3 (1 Game Changer: Worldly Tutor; no infinite combo) |
| **Simulator** | deck key `omnath`; results in `decklists/pool/pool-results.md`, history in `decklists/CHANGELOG.md` |

## Strategy

### The commander
**Omnath, Locus of Rage** reads: *Landfall — Whenever a land you control enters, create a 5/5 red and green Elemental
creature token. Whenever Omnath or another Elemental you control dies, Omnath deals 3 damage to any target.*

- Every land that enters is a 5/5 body: land drops, ramp spells that put lands onto the battlefield, fetch lands and
  their basics, Harrow, Oracle of Mul Daya's lands from the top of the library, and Titania's returned land.
- Every Elemental that dies is 3 damage at anything: a creature, a planeswalker or a player. The tokens, Titania's
  5/3s and Omnath himself all count.
- He costs five. If he dies, recast him: the tokens he already made keep their trigger only while he is on the
  battlefield, so the sooner he is back the better.

### Game plan
- **Turns 1-3.** Mana creature or ramp spell every turn (Llanowar Elves, Birds of Paradise, Rampant Growth, Nature's
  Lore). Play a land every turn; with Omnath out each is a 5/5.
- **Turns 4-6.** Cast Omnath, then play lands: every land drop, fetch land and ramp spell that fetches a land is
  another 5/5. Cast Titania so that fetch lands (and Harrow's sacrificed land) make 5/3s as well.
- **Late game.** The 5/5s attack. When a sweeper or a block kills them, each death is 3 damage. Finish with Terror of
  the Peaks, Hellkite Tyrant, or Overwhelming Stampede, or burn the last life points out with Goblin Bombardment and the
  3-damage triggers.

### Landfall and the two Elemental engines
- **Fetch lands** (Evolving Wilds, Terramorphic Expanse, Fabled Passage) are two landfalls each (the fetch land, then
  the basic) and, with Titania out, a 5/3.
- **Harrow** sacrifices a land (a Titania trigger, a land in the graveyard) and puts two basic lands onto the battlefield
  untapped: two 5/5s with Omnath out, and the mana to cast something else the same turn.
- **Oracle of Mul Daya** gives an extra land drop and lets you play lands from the top of the library: land after land,
  each a 5/5.
- **Wrenn and Six** returns a fetch land from the graveyard to hand (+1), to be played again for another landfall, and
  pings an X/1 (-1).
- **Titania** returns a land from the graveyard when she enters, then makes a 5/3 Elemental for every land that goes to
  the graveyard from the battlefield.

### Making Elementals die
- **Goblin Bombardment** turns the Elementals into 1 damage each, and Omnath adds 3 to each death: sacrifice a 5/5 for
  4 damage. Keep it for a kill or a creature that needs to die; don't throw away a token that could attack.
- **Blasphemous Act** (costs {1} less for each creature on the battlefield) kills every creature: Omnath's Elementals
  die and each is 3 damage to an opponent. Cast it when the damage is worth more than the board.
- **Terror of the Peaks** deals damage equal to a creature's power whenever a creature enters under your control: a
  5/5 token is 5 damage, every land drop. Spells that target it cost life, which keeps removal off it.
- **Scorching Dragonfire, Disintegrate, Malevolent Rumble and the bite spells** remove blockers and creatures.

### Ramp and land drops
- **Mana creatures:** Llanowar Elves, Fyndhorn Elder, Birds of Paradise, Noxious Newt, Paradise Druid, Hardbristle Bandit, Topiary
  Lecturer, Silhana Starfletcher, Greenhouse Propagator and Bugenhagen, Wise Elder.
- **Spells:** Rampant Growth, Nature's Lore, Cultivate, Skyshroud Claim and Harrow, each a land onto the battlefield.
- **Creatures that find land:** Fblthp, Knows the Way, Vinelasher Adept (basic landcycling {2}), Cartographer's Survey
  (two lands from the top seven), Track Down, Boughside Wanderers and Follow the Lumarets.
- **Fill the graveyard:** Satyr Wayfinder, Malevolent Rumble, Something Worth Saving, Scout the City and Pothole Mole
  put lands in the graveyard for Titania and Wrenn and Six to return.

### Card draw
Garruk's Uprising (draws for each creature with power 4 or more that enters: every 5/5), Shamanic Revelation,
Lifestream's Blessing, Track Down, Tenured Concocter and Kura, the Boundless Sky.

### Big creatures
Annoyed Altisaur, Cactarantula, Ghalta the Unstoppable (costs {X} less for your greatest power: with a 5/5 out she costs {3}{G}), Hellkite Tyrant, Genesis Hydra, Paladin of Predation, Symbiotic Beast, Vorinclex, Monstrous Raider
and Lashwhip Predator. Tumbleweed Rising makes a token as big as your greatest power.

### Worldly Tutor
**Worldly Tutor** ({G}, instant) puts a creature on top of the library. Use it at the end of an opponent's turn.
Targets: Terror of the Peaks or Oracle of Mul Daya for the engine, Ghalta or Annoyed Altisaur for a top end.

### How it wins
- **Elemental swarm.** Four or five 5/5s plus Overwhelming Stampede (all creatures gain trample and +X/+X) or
  End-Raze Forerunners is lethal against most tables.
- **Damage.** Terror of the Peaks, Goblin Bombardment, Omnath's 3-damage triggers and Blasphemous Act burn a table out
  when the board is in the way.

### Weak spots and how to play around them
- **Omnath dying early.** The engine is five mana. Keep Blossoming Defense and Royal Treatment for him and recast him.
- **Graveyard hate** turns off Titania, Wrenn and Six and the Harrow value. The Elemental engine still works.
- **Slow starts.** Mulligan hands without a land or a mana creature.

## Key lines

- **Turns 1-4:** ramp and graveyard fillers; hold nothing back. Play a fetch land when Omnath is out.
- **Omnath turn:** cast him, then play a land (a 5/5), crack a fetch land (two more) and cast Harrow if you have it.
- **Titania turn:** every fetch land and Harrow's sacrifice is a 5/3 as well as a landfall for Omnath.
- **Sweeper (yours or theirs):** every Elemental that dies is 3 damage: Blasphemous Act with a full board is the
  finisher.
- **Ghalta turn:** once you control a 5/5, Ghalta costs {3}{G}.

## Mana and consistency

37 lands: 17 Forest, 12 Mountain, Command Tower, Exotic Orchard, Cinder Glade, Karplusan Forest, Rootbound Crag,
and the three fetch lands (Evolving Wilds, Terramorphic Expanse, Fabled Passage) that find the basics. Birds of Paradise, Paradise Druid and Bugenhagen fix red.

## Bracket and Rule 0

One Game Changer (**Worldly Tutor**), one tutor and no infinite combo: Bracket 3. Tell the table before the game that
the deck makes many large creatures and that a sweeper turns them into burn.

## Cards to confirm

Terror of the Peaks, Blasphemous Act, Birds of Paradise, Command Tower and Exotic Orchard come from JD's spares and each
also appears in another deck. JD is checking he owns a second copy of each.

## Decklist by type (100)

**Commander (1).** Omnath, Locus of Rage

**Creatures (31).** Annoyed Altisaur, Birds of Paradise, Boughside Wanderers, Bugenhagen Wise Elder, Cactarantula, End-Raze Forerunners, Fblthp Knows the Way, Fyndhorn Elder, Genesis Hydra, Ghalta the Unstoppable, Greenbelt Radical, Greenhouse Propagator, Hardbristle Bandit, Hellkite Tyrant, Kura the Boundless Sky, Lashwhip Predator, Llanowar Elves, Noxious Newt, Oracle of Mul Daya, Paladin of Predation, Paradise Druid, Pothole Mole, Satyr Wayfinder, Silhana Starfletcher, Symbiotic Beast, Terror of the Peaks, Thrashing Brontodon, Titania Protector of Argoth, Topiary Lecturer, Vinelasher Adept, Vorinclex Monstrous Raider

**Planeswalkers (1).** Wrenn and Six

**Enchantments (2).** Garruk's Uprising, Goblin Bombardment

**Sorceries (17).** Bestial Incursion, Blasphemous Act, Carnivorous Canopy, Cartographer's Survey, Cultivate, Decimate, Disintegrate, Follow the Lumarets, Malevolent Rumble, Nature's Lore, Overwhelming Stampede, Rampant Growth, Scout the City, Shamanic Revelation, Skyshroud Claim, Track Down, Tumbleweed Rising

**Instants (11).** Arachnogenesis, Bite Down, Blossoming Defense, Compel Brutality, Harrow, Lifestream's Blessing, Rending Vines, Royal Treatment, Scorching Dragonfire, Something Worth Saving, Worldly Tutor

**Lands (37).** Cinder Glade, Command Tower, Evolving Wilds, Exotic Orchard, Fabled Passage, Karplusan Forest, Rootbound Crag, Terramorphic Expanse, 17 Forest, 12 Mountain

## Import list (100)

```
1 Annoyed Altisaur
1 Arachnogenesis
1 Bestial Incursion
1 Birds of Paradise
1 Bite Down
1 Blasphemous Act
1 Blossoming Defense
1 Boughside Wanderers
1 Bugenhagen, Wise Elder
1 Cactarantula
1 Carnivorous Canopy
1 Cartographer's Survey
1 Cinder Glade
1 Command Tower
1 Compel Brutality
1 Cultivate
1 Decimate
1 Disintegrate
1 End-Raze Forerunners
1 Evolving Wilds
1 Exotic Orchard
1 Fabled Passage
1 Fblthp, Knows the Way
1 Follow the Lumarets
1 Fyndhorn Elder
1 Garruk's Uprising
1 Genesis Hydra
1 Ghalta the Unstoppable
1 Goblin Bombardment
1 Greenbelt Radical
1 Greenhouse Propagator
1 Hardbristle Bandit
1 Harrow
1 Hellkite Tyrant
1 Karplusan Forest
1 Kura, the Boundless Sky
1 Lashwhip Predator
1 Lifestream's Blessing
1 Llanowar Elves
1 Malevolent Rumble
1 Nature's Lore
1 Noxious Newt
1 Omnath, Locus of Rage
1 Oracle of Mul Daya
1 Overwhelming Stampede
1 Paladin of Predation
1 Paradise Druid
1 Pothole Mole
1 Rampant Growth
1 Rending Vines
1 Rootbound Crag
1 Royal Treatment
1 Satyr Wayfinder
1 Scorching Dragonfire
1 Scout the City
1 Shamanic Revelation
1 Silhana Starfletcher
1 Skyshroud Claim
1 Something Worth Saving
1 Symbiotic Beast
1 Terramorphic Expanse
1 Terror of the Peaks
1 Thrashing Brontodon
1 Titania, Protector of Argoth
1 Topiary Lecturer
1 Track Down
1 Tumbleweed Rising
1 Vinelasher Adept
1 Vorinclex, Monstrous Raider
1 Worldly Tutor
1 Wrenn and Six
17 Forest
12 Mountain
```
