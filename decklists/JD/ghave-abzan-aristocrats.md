# Ghave, Guru of Spores — Abzan Aristocrats

The Seph/Atraxa reanimator (`decklists/pool/retired/sephiroth-phyrexian-reanimator.md`) rebuilt around Ghave, Guru of Spores (JD's
choice, 10/30). The blue cards are out and the deck leans into sacrifice: Ghave makes Saprolings out of +1/+1 counters
and counters out of creatures, so he is a sacrifice outlet and a combo piece from the command zone. The reanimation
package stays as the fast start. Not simulated yet.

| | |
|---|---|
| **Commander** | Ghave, Guru of Spores ({2}{W}{B}{G}, 0/0 that enters with five +1/+1 counters, white-black-green) |
| **Bracket** | 4 (5 Game Changers: Demonic Tutor, Vampiric Tutor, Worldly Tutor, Aura Shards, Gaea's Cradle; the tutorable infinite combos also put it in Bracket 4) |
| **Simulator** | deck key `ghave`, tiers t3-t4 (Seph's seat since 2026-10-30); plays from `data/plans/ghave.toml` |

## Strategy

### The commander

Ghave, Guru of Spores: *Ghave enters with five +1/+1 counters. {1}, Remove a +1/+1 counter from a creature you
control: Create a 1/1 green Saproling creature token. {1}, Sacrifice a creature: Put a +1/+1 counter on target
creature.*

- **A sacrifice outlet in the command zone.** For {1}, any creature dies at instant speed: in response to removal or
  theft, or to trigger Blood Artist, Zulaport Cutthroat, Cruel Celebrant, Sephiroth, Fabled SOLDIER and Grave Pact.
- **A token maker.** Every counter on Ghave (or on any of your creatures) is a Saproling for {1}. Saprolings tap for
  Gaea's Cradle, carry Skullclamp (equip {1}, the Saproling dies at once, draw two), and are free edicts with Grave Pact.
- **A combo piece.** With an altar and a counter or token doubler he goes infinite (see Key lines).

He costs five and is cheap to recast, so cast him early as soon as there is a payoff on the battlefield.

### Game plan

**Turns 1 to 3: mana and set-up.** Sol Ring, Arcane Signet, Talisman of Hierarchy, Chromatic Lantern, Birds of Paradise
and Sakura-Tribe Elder for mana; Dark Ritual for a fast Entomb into Reanimate. Stitcher's Supplier and Viscera Seer are
the cheap creatures that feed the outlets.

**Turns 3 to 5: a payoff and Ghave.** Land a death payoff (Blood Artist, Zulaport Cutthroat, Cruel Celebrant,
Sephiroth, Fabled SOLDIER, Pitiless Plunderer, Grave Pact), then Ghave. From then on every spare {1} is a Saproling and
every Saproling is a drain, a card (Skullclamp) or an edict (Grave Pact). Reanimation is the other fast start: Entomb
into Reanimate or Animate Dead for Archon of Cruelty or Grave Titan.

**Late game.** Assemble a loop. Tutors find the missing piece; Grand Abolisher and Heroic Intervention protect the turn.

### Death payoffs

- **Blood Artist** (whenever a creature dies, target player loses 1 and you gain 1), **Zulaport Cutthroat** and
  **Cruel Celebrant** (whenever a creature of yours dies, each opponent loses 1 and you gain 1). With the last two, every
  loop hits the whole table.
- **Sephiroth, Fabled SOLDIER**: drains a target opponent for each other creature that dies, flips on the fourth in a
  turn, and his emblem keeps draining afterwards.
- **Grave Pact** ({1}{B}{B}{B}): each opponent sacrifices a creature whenever one of yours dies. Every Saproling is an
  edict.
- **Pitiless Plunderer**: a Treasure whenever another creature of yours dies, which pays for Ghave's {1}.
- **Yawgmoth, Thran Physician**: pay 1 life and sacrifice another creature to put a -1/-1 counter on up to one target
  creature and draw a card. A free outlet that draws, and the -1/-1 counters kill the table's small creatures.
  {B}{B}, discard a card: proliferate (more counters for Ghave, Mikaeus's undying creatures and Triskelion).
- **Skullclamp**: a Saproling equipped with it dies at once; draw two.
- **Aura Shards**: each Saproling or Zombie that enters destroys an artifact or enchantment.

### Counters and tokens

**Cathars' Crusade** puts a +1/+1 counter on each creature you control whenever a creature enters under your control, so
each Saproling refunds the counter it cost. **Doubling Season** doubles tokens and counters: Ghave enters with ten
counters, each activation makes two Saprolings, Grave Titan makes four Zombies. Both are combo pieces with Ghave and an
altar. **Karn's Bastion** ({4}, {T}: proliferate) and Yawgmoth add counters too.

### Reanimation

The Seph package stays as the fast start: **Entomb**, **Stitcher's Supplier** and **Tortured Existence** put creatures
in the graveyard; **Reanimate**, **Grave Researcher // Reanimate**, **Animate Dead**, **Necromancy**, **Victimize** and
**Dread Return** bring them back. Targets: Archon of Cruelty, Grave Titan, Rune-Scarred Demon (tutor on entry), both
Sheoldreds, both Elesh Norns, Summon: Bahamut. **Living Death** and **Yawgmoth's Will** are the big turns.
**Victimize** and **Dread Return**'s flashback (sacrifice three creatures) want the Saprolings.

### Protection

**Grand Abolisher** stops opponents casting spells or using creature, artifact and enchantment abilities during your
turn. **Heroic Intervention** ({1}{G}: your permanents gain hexproof and indestructible until end of turn) answers a
wipe or targeted removal on the combo turn. **Ephemerate** and **Restoration Angel** blink a creature in response to
removal. **Kitchen Finks** and Mikaeus's undying creatures come back from a wipe.

### Removal

Swords to Plowshares, Path to Exile, Anguished Unmaking, Summon: Bahamut's two destroys, Aura Shards, and Grave Pact's
edicts. Resets: Toxic Deluge, Austere Command, Living Death. Bojuka Bog exiles a graveyard.

### Weak spots and how to play around them

- **Graveyard hate.** Rest in Peace exiles everything that would die, Saprolings included, so it stops reanimation,
  undying and persist, and every "dies" payoff (Blood Artist, Zulaport, Celebrant, Sephiroth, Grave Pact, Plunderer).
  The Ghave + Ashnod's Altar loops still make mana but don't kill. Remove it with Aura Shards, Austere Command or
  Anguished Unmaking before you go off.
- **Torpor Orb** (creatures entering don't cause abilities to trigger) stops Cathars' Crusade, Aura Shards and the
  enter triggers of Archon, Grave Titan and Rune-Scarred Demon. Doubling Season is a replacement effect, so loops 3 and
  4 still work under it.
- **Wipes.** Ghave dies to them, so keep Heroic Intervention up on the turn you go for it, and keep a payoff in hand.
- **Fast combo decks.** No counterspells now; Grand Abolisher, the tutors and a fast loop are the answer.

## Key lines

Ghave's infinite combos each need Ghave, an altar and Cathars' Crusade or Doubling Season. The three older loops stay.

| # | Pieces | Result | Kills alone? |
| --- | --- | --- | --- |
| 1 | Ghave + Ashnod's Altar + Cathars' Crusade | Infinite colourless mana, Saprolings, enters and deaths | Needs a payoff |
| 2 | Ghave + Phyrexian Altar + Cathars' Crusade | Infinite Saprolings, enters and deaths | Needs a payoff |
| 3 | Ghave + Ashnod's Altar + Doubling Season | Infinite colourless mana, Saprolings and deaths | Needs a payoff |
| 4 | Ghave + Phyrexian Altar + Doubling Season | Infinite Saprolings and deaths | Needs a payoff |
| 5 | Mikaeus + Triskelion | Infinite damage | Yes |
| 6 | Mikaeus + Kitchen Finks + free sacrifice outlet | Infinite life, enters and deaths | Needs a payoff |
| 7 | Nim Deathmantle + Ashnod's Altar + Grave Titan | Infinite colourless mana, enters and deaths | Needs a payoff |

**1. Ghave + Ashnod's Altar + Cathars' Crusade.** Pay {1}, remove a counter from Ghave: a Saproling enters, and Crusade
puts a counter on each of your creatures, so Ghave gets it back. Sacrifice the Saproling to Ashnod's Altar for
{C}{C}. Net +{1} and one death per loop.

**2. With Phyrexian Altar instead.** The same loop, but the Altar gives one mana: net zero mana, one death per loop.

**3. Ghave + Ashnod's Altar + Doubling Season.** Pay {1}, remove a counter: two Saprolings. Sacrifice one to Ashnod's
Altar ({C}{C}) and pay {1} to sacrifice the other to Ghave, putting a counter on Ghave (doubled to two). Net zero mana,
one counter up, two deaths. Alternate with sacrificing both Saprolings to the Altar (net +{3}, one counter down): each
pair of loops is +{3} and four deaths.

**4. With Phyrexian Altar instead.** Alternate the same two loops: both Saprolings to the Altar (+{1}, one counter down),
then one to the Altar and one to Ghave (-{1}, one counter up). Each pair is zero mana and four deaths.

**5. Mikaeus + Triskelion.** Mikaeus gives Triskelion undying and +1/+1. It returns with four counters, pings itself
twice and opponents twice, dies with no counters and comes back: 2 damage per loop. With Elesh Norn, Grand Cenobite out
(it is a 4/4 with no counters), sacrifice it to Ghave or a free outlet after removing the counters instead.

**6. Mikaeus + Kitchen Finks + a free outlet** (Viscera Seer, Ashnod's Altar, Altar of Dementia, Phyrexian Altar, or
Yawgmoth). Persist and undying alternate; each loop gains 2 life. Yawgmoth draws a card every loop, so stop before your
library runs out, or use another outlet.

**7. Nim Deathmantle + Ashnod's Altar + Grave Titan.** Sacrificing Titan gives {C}{C}, Deathmantle's {4} returns it, and
the two Zombies sacrifice for four more: net +{2} per loop (four Zombies and +{6} with Doubling Season).

Ghave is not a free outlet ({1} per sacrifice), so he doesn't finish loop 6 alone; with infinite mana from loop 1, 3 or
7 he does.

**Payoffs that turn the loops into kills:**
- **Zulaport Cutthroat or Cruel Celebrant:** each opponent loses 1 per death.
- **Blood Artist or Sephiroth, Fabled SOLDIER:** one target per death; aim at one opponent at a time.
- **Altar of Dementia:** sacrifice the infinite Saprolings (or Finks) to mill each opponent out.
- **Infinite mana from loops 1, 3 or 7:** Triskelion and Nim Deathmantle (loop 7's pings), or Ghave putting infinite
  counters on one creature for a lethal attack.
- **Grave Pact:** each loop also empties their boards.

**Protect the combo turn.** Resolve **Grand Abolisher** first, and hold **Heroic Intervention** for a wipe or removal
in response.

**Openings.**
- Turn 1: Stitcher's Supplier, Entomb, Birds of Paradise or Sol Ring. Turn 2: a payoff or a rock. Turn 3 or 4: Ghave.
- A fast line: Entomb for Archon of Cruelty or Grave Titan, then Reanimate or Animate Dead the same turn or the next.
  With Dark Ritual, a Swamp pays for Entomb and Reanimate on turn one.

**Mulligans.** Keep hands with two or three lands (or a land plus Sol Ring) and either a death payoff or the
reanimation start. Don't keep one-landers.

## Tutor targets by board state

Six tutors: **Demonic Tutor** (any card), **Vampiric Tutor** (any card, to the top), **Worldly Tutor** (a creature, to
the top), **Eladamri's Call** (a creature, to hand, instant), **Open the Armory** (an Aura or Equipment: Animate Dead,
Necromancy, Skullclamp, Nim Deathmantle) and **Rune-Scarred Demon** (any card when it enters).

- **Ghave and an altar out → tutor Cathars' Crusade or Doubling Season** (Demonic or Vampiric).
- **Ghave and Crusade or Season out → tutor the altar.**
- **Mikaeus out → Triskelion** (Worldly Tutor or Eladamri's Call), or Kitchen Finks with an outlet out.
- **Loop assembled, no payoff → Zulaport Cutthroat or Cruel Celebrant** (Eladamri's Call at instant speed).
- **Empty yard, reanimation spell in hand → Entomb.**
- **Under pressure → Toxic Deluge or Austere Command**, or Grand Abolisher before the combo turn.

## Mana and consistency

The list is exactly 100 cards. 37 lands: 26 nonbasic plus 5 Swamp, 3 Forest, 3 Plains (11 basics). The nonbasics are the
three on-colour original duals (Bayou, Savannah, Scrubland), three shocks (Overgrown Tomb, Temple Garden, Godless
Shrine), three check lands (Woodland Cemetery, Isolated Chapel, Sunpetal Grove), three pain lands (Caves of Koilos,
Brushland, Llanowar Wastes), Indatha Triome and Sandsteppe Citadel (both enter tapped), the six fetch lands (Flooded
Strand, Polluted Delta, Windswept Heath, Marsh Flats, Verdant Catacombs, Misty Rainforest; each finds a Plains, Swamp or
Forest dual, and every fetch can get Indatha Triome), Command Tower, Gaea's Cradle, Bojuka Bog (tapped), Strip Mine,
Karn's Bastion and Phyrexian Tower. **Gaea's Cradle** taps for {G} per creature you control, so the Saprolings are mana.

Mana: four rocks (Sol Ring, Arcane Signet, Talisman of Hierarchy, Chromatic Lantern), Birds of Paradise,
Sakura-Tribe Elder, Dark Ritual, Gaea's Cradle, Pitiless Plunderer's Treasures, and sacrifice mana from the altars and
Phyrexian Tower. **Crucible of Worlds** replays the fetch lands and Strip Mine.

**Card draw.** Sylvan Library, Phyrexian Arena, Skullclamp, Yawgmoth, Sephiroth, Fabled SOLDIER and Sheoldred, the
Apocalypse (gain life per draw).

**Interaction.** Swords to Plowshares, Path to Exile, Anguished Unmaking, Summon: Bahamut, Aura Shards, Grave Pact;
Toxic Deluge, Austere Command and Living Death as resets; Grand Abolisher, Heroic Intervention, Ephemerate and
Restoration Angel as protection.

## Bracket and Rule 0

**Bracket 4.** Five Game Changers: **Demonic Tutor**, **Vampiric Tutor**, **Worldly Tutor**, **Aura Shards** and
**Gaea's Cradle**, more than Bracket 3 allows; the tutorable infinite combos would put it in Bracket 4 anyway.

Disclose before the game:

- **Seven infinite loops**, four of them with the commander (Ghave + an altar + Cathars' Crusade or Doubling Season),
  plus Mikaeus + Triskelion, Mikaeus + Kitchen Finks + an outlet, and Nim Deathmantle + Ashnod's Altar + Grave Titan.
- The five Game Changers above.
- **Grand Abolisher**, **Yawgmoth's Will**, **Austere Command**, **Living Death**, and **Strip Mine** (every turn with
  Crucible of Worlds).
- The deck reanimates out of *any* graveyard.

Re-check the official list at https://commanderbrackets.com/faq before an event.

## Decklist by type (100)

**Commander (1).** Ghave, Guru of Spores

**Creatures (24).** Archon of Cruelty, Birds of Paradise, Blood Artist, Cruel Celebrant, Elesh Norn Grand Cenobite, Elesh Norn Mother of Machines, Grand Abolisher, Grave Researcher // Reanimate, Grave Titan, Kitchen Finks, Mikaeus the Unhallowed, Pitiless Plunderer, Restoration Angel, Rune-Scarred Demon, Sakura-Tribe Elder, Sephiroth Fabled SOLDIER // Sephiroth One-Winged Angel, Sheoldred the Apocalypse, Sheoldred Whispering One, Stitcher's Supplier, Summon: Bahamut, Triskelion, Viscera Seer, Yawgmoth Thran Physician, Zulaport Cutthroat

**Enchantments (9).** Animate Dead, Aura Shards, Cathars' Crusade, Doubling Season, Grave Pact, Necromancy, Phyrexian Arena, Sylvan Library, Tortured Existence

**Artifacts (10).** Altar of Dementia, Arcane Signet, Ashnod's Altar, Chromatic Lantern, Crucible of Worlds, Nim Deathmantle, Phyrexian Altar, Skullclamp, Sol Ring, Talisman of Hierarchy

**Sorceries (9).** Austere Command, Demonic Tutor, Dread Return, Living Death, Open the Armory, Reanimate, Toxic Deluge, Victimize, Yawgmoth's Will

**Instants (10).** Anguished Unmaking, Dark Ritual, Eladamri's Call, Entomb, Ephemerate, Heroic Intervention, Path to Exile, Swords to Plowshares, Vampiric Tutor, Worldly Tutor

**Lands (37).** Bayou, Bojuka Bog, Brushland, Caves of Koilos, Command Tower, Flooded Strand, Gaea's Cradle, Godless Shrine, Indatha Triome, Isolated Chapel, Karn's Bastion, Llanowar Wastes, Marsh Flats, Misty Rainforest, Overgrown Tomb, Phyrexian Tower, Polluted Delta, Sandsteppe Citadel, Savannah, Scrubland, Strip Mine, Sunpetal Grove, Temple Garden, Verdant Catacombs, Windswept Heath, Woodland Cemetery, 5 Swamp, 3 Forest, 3 Plains

## Import list (100)

```
1 Altar of Dementia
1 Anguished Unmaking
1 Animate Dead
1 Arcane Signet
1 Archon of Cruelty
1 Ashnod's Altar
1 Aura Shards
1 Austere Command
1 Bayou
1 Birds of Paradise
1 Blood Artist
1 Bojuka Bog
1 Brushland
1 Cathars' Crusade
1 Caves of Koilos
1 Chromatic Lantern
1 Command Tower
1 Crucible of Worlds
1 Cruel Celebrant
1 Dark Ritual
1 Demonic Tutor
1 Doubling Season
1 Dread Return
1 Eladamri's Call
1 Elesh Norn, Grand Cenobite
1 Elesh Norn, Mother of Machines
1 Entomb
1 Ephemerate
1 Flooded Strand
1 Gaea's Cradle
1 Ghave, Guru of Spores
1 Godless Shrine
1 Grand Abolisher
1 Grave Pact
1 Grave Researcher // Reanimate
1 Grave Titan
1 Heroic Intervention
1 Indatha Triome
1 Isolated Chapel
1 Karn's Bastion
1 Kitchen Finks
1 Living Death
1 Llanowar Wastes
1 Marsh Flats
1 Mikaeus, the Unhallowed
1 Misty Rainforest
1 Necromancy
1 Nim Deathmantle
1 Open the Armory
1 Overgrown Tomb
1 Path to Exile
1 Phyrexian Altar
1 Phyrexian Arena
1 Phyrexian Tower
1 Pitiless Plunderer
1 Polluted Delta
1 Reanimate
1 Restoration Angel
1 Rune-Scarred Demon
1 Sakura-Tribe Elder
1 Sandsteppe Citadel
1 Savannah
1 Scrubland
1 Sephiroth, Fabled SOLDIER // Sephiroth, One-Winged Angel
1 Sheoldred, the Apocalypse
1 Sheoldred, Whispering One
1 Skullclamp
1 Sol Ring
1 Stitcher's Supplier
1 Strip Mine
1 Summon: Bahamut
1 Sunpetal Grove
1 Swords to Plowshares
1 Sylvan Library
1 Talisman of Hierarchy
1 Temple Garden
1 Tortured Existence
1 Toxic Deluge
1 Triskelion
1 Vampiric Tutor
1 Verdant Catacombs
1 Victimize
1 Viscera Seer
1 Windswept Heath
1 Woodland Cemetery
1 Worldly Tutor
1 Yawgmoth, Thran Physician
1 Yawgmoth's Will
1 Zulaport Cutthroat
3 Forest
3 Plains
5 Swamp
```
