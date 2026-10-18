# Tymna the Weaver & Kraum, Ludevic's Opus — Four-Colour Partners (W/U/B/R)

Two commanders who draw cards without spending mana, behind a wall of attack taxes. Tymna draws one card for each
opponent your creatures hit in combat, and Kraum draws whenever an opponent casts their second spell in a turn. The deck
slows the table's attacks with Ghostly Prison, Propaganda, Crawlspace and their friends, protects Tymna, and then wins
with a top end of flying finishers: Elspeth, the Titans, Baneslayer, Terror of the Peaks, Lyra, Glorybringer and more.
It is weak to decks that can pay the attack taxes every turn, to combo, and to falling behind on mana.

| | |
|---|---|
| **Commanders** | Tymna the Weaver ({1}{W}{B}, 2/2) and Kraum, Ludevic's Opus ({3}{U}{R}, 4/4); white-blue-black-red |
| **Bracket** | 3 (1 Game Changer: Underworld Breach) |
| **Simulator** | deck key `kraum`, matched tier T5 (band 1, since 10/18); history in `decklists/CHANGELOG.md`, findings in `audit/kraum/FINDINGS.md` |

## Strategy

### The commanders

**Tymna the Weaver** ({1}{W}{B}, 2/2 lifelink). *At the beginning of each of your postcombat main phases, you may pay X
life, where X is the number of opponents that were dealt combat damage this turn. If you do, draw X cards.*

**Kraum, Ludevic's Opus** ({3}{U}{R}, 4/4 flying, haste). *Whenever an opponent casts their second spell each turn,
draw a card.*

Both have plain Partner, so they share the command zone. Tymna comes down on turn 3 and draws off every opponent your
creatures reach; what she counts is *opponents* hit, not creatures, so a 1/1 token into each player is worth more than
one big hit. Kraum is a hasty 4/4 flier that connects the turn he lands, and he draws on the opponents' own turns with
no work from you. Keep **Lightning Greaves** for Tymna: she is the deck's most-removed card, and Greaves (equip {0})
gives her shroud and lets her attack at once.

### Game plan

**Early turns (1–3).** A 2-mana rock (Arcane Signet, Fellwar Stone, Orzhov Signet, Talisman of Conviction) or a cheap
creature on turn 2, then **Tymna on turn 3**. An attack tax on turn 3 is just as good when the table is aggressive.

**Middle turns (4–6).** Get the defense down: **Ghostly Prison**, **Propaganda**, **Crawlspace**, **Windborn Muse**,
**Archon of Absolution** and **Baird** make attacking you expensive, so opponents attack each other instead. Add Kraum
and the token makers (**Adeline**, **Hero of Bladehold**), and attack each opponent with something so Tymna draws.

**Late game.** The finishers take over. Most of them fly, several make more bodies, and the attack taxes keep the table
off you while they work: **Elspeth, Sun's Champion**, **Sun Titan**, **Grave Titan**, **Baneslayer Angel**, **Terror of
the Peaks**, **Lyra Dawnbringer**, **Glorybringer**, **Thundermaw Hellkite**, **Serra Angel**, **Resplendent Angel**.

### Defense: why it matters

Without defense this deck lost most of its games to creature decks in combat: its own creatures are needed to attack for
Tymna, so it can't also block. The attack taxes fix that:

- **Ghostly Prison** and **Propaganda** ({2} per attacker), **Windborn Muse** (the same, on a flier), **Archon of
  Absolution** ({1} per attacker, protection from white) and **Baird, Steward of Argive** ({1} per attacker) stack: a
  creature attacking you can cost {6} or more.
- **Crawlspace**: no more than two creatures can attack you each combat, which shuts down token swarms and extra-combat
  decks.
- **Elspeth**'s −3 destroys every creature with power 4 or more (yours too: count your Titans and Angels first);
  **Sunblast Angel** destroys every tapped creature when it enters (yours too, so cast it before you attack, after an
  opponent's big attack); **Austere Command** is the full reset (choose creatures with mana value 3 or less and 4 or
  more: your attack taxes and rocks survive); **Avenge** destroys every creature and gains you 1 life for each, and costs
  {2} less when a player attacked you on their last turn (hold it until their board is worth more than yours).

### Finishers

- **Elspeth, Sun's Champion** (+1: three 1/1 Soldiers a turn) is the best card in the deck: a wall of blockers and
  attackers in one.
- **Sun Titan** returns a permanent with mana value 3 or less from your graveyard whenever it enters or attacks: a dead
  Tymna, an attack tax, a rock. **Grave Titan** brings two 2/2 Zombies on entering and on each attack.
- **Baneslayer Angel**, **Lyra Dawnbringer** and **Serra Angel** are lifelinking or vigilant fliers that race and block.
  **Resplendent Angel** ({1}{W}{W}, 3/3 flier) makes a 4/4 flying, vigilant Angel at each end step in which you
  gained 5 or more life, which the lifelinkers (Baneslayer, Lyra, Vampire Nighthawk, Tymna) make easy.
- **Terror of the Peaks** turns every creature you cast into damage to any target; **Glorybringer** and **Thundermaw
  Hellkite** are hasty dragons (Glorybringer exerts to deal 4 to a creature; Thundermaw taps and pings the opponents'
  fliers). **Smaug the Magnificent** ({2}{R}{R}, 4/3 flying, haste) attacks the turn he lands, makes a Treasure
  each upkeep (colour fixing toward the five- and six-drops), and deals damage equal to your Treasures to any target
  when he attacks: spend the Treasures as mana by default, and save them only when the damage kills a blocker or a
  player.
- **Diabolic Tutor** ({2}{B}{B}) fetches any card. Fetch Elspeth when behind, a Titan or Terror of the Peaks to close.
- **Gilgamesh, Master-at-Arms** looks at six cards for Equipment on entering and attacking; **Shield of Kaldra** (equip
  {4}) makes a creature indestructible, **Helm of Kaldra** (equip {2}) gives first strike, trample and haste, and
  **Conqueror's Flail** gives +4/+4 with all four colours and stops opponents from casting spells on your turn.

### Tymna's attackers

Tymna wants every opponent hit each turn. **Adeline** makes a 1/1 attacking *each* opponent whenever you attack, and
**Hero of Bladehold** makes two attacking tokens with battle cry; **Dauthi Voidwalker** (shadow), **Vampire Nighthawk**,
**The Wasp** (taps a blocker when it attacks), **Nightwind Glider**, **Falcon and Redwing**, **Black Widow** and
**Lightning** (menace) get through on their own. Attack the opponents you can reach, keep enough back to block what
the taxes let through, and draw.

### Removal and interaction

**Swords to Plowshares** and **Path to Exile** (one mana), **Darksteel Mutation** (also answers a
commander), **Skyclave Apparition**, **Memory Trap**, **Bedevil** and **Disenchant**; **Glorybringer** and **Elspeth**
add more. **Arcane Denial** is the one counterspell, and **Aven Mindcensor** (flash) blunts tutors and land searches.
**Restoration Angel** (flash) blinks a creature out of removal and reuses an enters effect (a Titan, Skyclave
Apparition). **Dark Endurance** ({1}{B}: +2/+0 and indestructible) and **Lightning Greaves** protect
Tymna.

### Card advantage beyond the commanders

**Black Widow** draws when an opponent draws their second card in a turn, and **Fact or Fiction** and
**Expressive Iteration** dig.
**Lightning** plays the top card when it connects. **Underworld Breach** ({1}{R}) gives every nonland card in your
graveyard escape for the turn (its mana cost plus exiling three other cards from the graveyard), so a late game with a
full graveyard recasts a dead Elspeth, Titan, Angel or wipe (Austere Command, Avenge) in one turn. It is sacrificed at
the end step, and Sun Titan can bring it back (mana value 2). It needs {R} and four or more cards in the graveyard, so
save it for after a wipe or a turn of trades.

### How it wins

By combat in the air while nobody can attack you profitably. A typical win: the taxes are down, Tymna draws two or three
a turn off Adeline's tokens and the fliers, and a Titan, Elspeth or Baneslayer takes over the board. Terror of the Peaks
and Gilgamesh close faster when they land.

### Weak spots and how to play around them

- **Decks that pay the taxes or don't need to attack** (combo, drain, big mana). The taxes do nothing against them; hold
  Arcane Denial and the cheap removal for the combo piece, and race.
- **Mana.** The deck runs 38 lands and four 2-mana rocks because its finishers cost 5–6: keep a hand with 3+ lands and a
  rock, and play the rocks early.
- **Tymna dying.** She is removed more than any other card. Equip Lightning Greaves to her, keep Dark Endurance or
  Restoration Angel up when you can, and recast her (Sun Titan also brings her back).
- **Board wipes.** The defense is mostly enchantments and artifacts, which survive creature wipes; rebuild the attackers
  with Adeline, Hero, the Titans and Elspeth.

## Key lines

- **Turn 2 rock, turn 3 Tymna, turn 4 an attack tax:** the table attacks someone else while Tymna draws.
- **Adeline + Tymna:** every attack puts a token into each opponent, so Tymna draws three.
- **Crawlspace + Ghostly Prison:** at most two attackers, each costing {2}.
- **Sun Titan on a dead Tymna or a destroyed attack tax:** it comes back every attack.
- **Diabolic Tutor for Elspeth** when behind on board.
- **Underworld Breach after a wipe:** recast the Titan, Angel or wipe that died, paying three other graveyard cards each.
- **Sunblast Angel after an opponent's alpha strike:** every tapped attacker dies.

**Mulligan:** keep 3+ lands that make at least three of the four colours, with a rock or a turn-3 play. A hand that can't
cast Tymna by turn 3 or 4 is a mulligan.

## Mana and consistency

38 lands: 16 nonbasic and 22 basic (7 Plains, 6 Island, 6 Swamp, 3 Mountain); no land enters tapped every time. Mana
rocks: **Arcane Signet**, **Fellwar Stone**, **Orzhov Signet**, **Talisman of Conviction**.

- **Any colour:** Command Tower, Exotic Orchard, Unclaimed Territory (name **Human**).
- **Checklands** (untapped with a basic of the right type): Isolated Chapel (W/B), Drowned Catacomb (U/B), Sulfur Falls
  (U/R), Dragonskull Summit (B/R), Clifftop Retreat (R/W), Glacial Fortress (W/U).
- **Other two-colour lands:** Battlefield Forge, Caves of Koilos and Underground River (pain lands), Rugged Prairie
  (filter), Furycalm Snarl and Port Town (reveal a basic), Spectator Seating (untapped with two or more opponents).

**Card draw:** Tymna, Kraum, Black Widow, Fact or Fiction, Expressive Iteration.
**Tutors:** Diabolic Tutor.

## Bracket and Rule 0

**Bracket 3,** with one Game Changer, **Underworld Breach**. No two-card infinite combos, no mass land destruction, no extra turns. Mention the
attack taxes (Ghostly Prison, Propaganda, Crawlspace, Windborn Muse, Archon of Absolution, Baird), Aven Mindcensor and
Conqueror's Flail (no spells on your turn) before the game.

## Decklist by type (100)

**Commanders (2).** Tymna the Weaver, Kraum, Ludevic's Opus

**Creatures (34).** Adeline, Resplendent Cathar, Archon of Absolution, Aven Mindcensor, Baird, Steward of Argive, Baneslayer Angel, Bastion Protector, Black Widow, Agile Avenger, Dauthi Voidwalker, Elemental Mascot, Elenda, the Dusk Rose, Falcon and Redwing, Firebird, Blazing Ranger, Gilgamesh, Master-at-Arms, Glorybringer, Grave Titan, Hero of Bladehold, Lightning, Security Sergeant, Lyra Dawnbringer, Marchesa, the Black Rose, Nightveil Predator, Nightwind Glider, Resplendent Angel, Restoration Angel, Serra Angel, Skyclave Apparition, Smaug the Magnificent, Sphinx of New Prahv, Sun Titan, Sunblast Angel, Terror of the Peaks, The Wasp, Winsome Avenger, Thundermaw Hellkite, Vampire Nighthawk, Windborn Muse

**Planeswalkers (1).** Elspeth, Sun's Champion

**Artifacts (9).** Arcane Signet, Conqueror's Flail, Crawlspace, Fellwar Stone, Helm of Kaldra, Lightning Greaves, Orzhov Signet, Shield of Kaldra, Talisman of Conviction

**Enchantments (5).** Darksteel Mutation, Ghostly Prison, Memory Trap, Propaganda, Underworld Breach

**Instants (7).** Arcane Denial, Bedevil, Dark Endurance, Disenchant, Fact or Fiction, Path to Exile, Swords to Plowshares

**Sorceries (4).** Austere Command, Avenge, Diabolic Tutor, Expressive Iteration

**Lands (38).** Battlefield Forge, Caves of Koilos, Clifftop Retreat, Command Tower, Dragonskull Summit, Drowned Catacomb, Exotic Orchard, Furycalm Snarl, Glacial Fortress, Isolated Chapel, Port Town, Rugged Prairie, Spectator Seating, Sulfur Falls, Unclaimed Territory, Underground River, 6 Island, 7 Plains, 6 Swamp, 3 Mountain

## Import list (100)

```
1 Adeline, Resplendent Cathar
1 Arcane Denial
1 Arcane Signet
1 Archon of Absolution
1 Austere Command
1 Aven Mindcensor
1 Avenge
1 Baird, Steward of Argive
1 Baneslayer Angel
1 Bastion Protector
1 Battlefield Forge
1 Bedevil
1 Black Widow, Agile Avenger
1 Caves of Koilos
1 Clifftop Retreat
1 Command Tower
1 Conqueror's Flail
1 Crawlspace
1 Dark Endurance
1 Darksteel Mutation
1 Dauthi Voidwalker
1 Diabolic Tutor
1 Disenchant
1 Dragonskull Summit
1 Drowned Catacomb
1 Elemental Mascot
1 Elenda, the Dusk Rose
1 Elspeth, Sun's Champion
1 Exotic Orchard
1 Expressive Iteration
1 Fact or Fiction
1 Falcon and Redwing
1 Fellwar Stone
1 Firebird, Blazing Ranger
1 Furycalm Snarl
1 Ghostly Prison
1 Gilgamesh, Master-at-Arms
1 Glacial Fortress
1 Glorybringer
1 Grave Titan
1 Helm of Kaldra
1 Hero of Bladehold
1 Isolated Chapel
1 Kraum, Ludevic's Opus
1 Lightning Greaves
1 Lightning, Security Sergeant
1 Lyra Dawnbringer
1 Marchesa, the Black Rose
1 Memory Trap
1 Nightveil Predator
1 Nightwind Glider
1 Orzhov Signet
1 Path to Exile
1 Port Town
1 Propaganda
1 Resplendent Angel
1 Restoration Angel
1 Rugged Prairie
1 Serra Angel
1 Shield of Kaldra
1 Skyclave Apparition
1 Smaug the Magnificent
1 Spectator Seating
1 Sphinx of New Prahv
1 Sulfur Falls
1 Sun Titan
1 Sunblast Angel
1 Swords to Plowshares
1 Talisman of Conviction
1 Terror of the Peaks
1 The Wasp, Winsome Avenger
1 Thundermaw Hellkite
1 Tymna the Weaver
1 Unclaimed Territory
1 Underground River
1 Underworld Breach
1 Vampire Nighthawk
1 Windborn Muse
6 Island
3 Mountain
7 Plains
6 Swamp
```
