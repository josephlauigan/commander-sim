# Trostani, Selesnya's Voice — Selesnya Counters, Blink and Lifegain

A Selesnya (green-white) deck that turns every creature entering the battlefield into life, counters or both. Doubling
Season, Branching Evolution and level 3 of Innkeeper's Talent multiply the +1/+1 counters, and Emiel the Blessed adds one
to each creature that enters and blinks creatures to replay their enters triggers. Trostani gains life equal to the
toughness of every other creature entering, so the more counters a creature enters with, the more life it gives. The life
total is the win condition: Archangel of Thune turns each gain into counters on the whole team, Felidar Sovereign wins at 40
life and Test of Endurance at 50, Aetherflux Reservoir spends 50 life as 50 damage, and Craterhoof Behemoth ends a wide
board. It is weak to a sweeper cast before the engine is out, to exile effects on Trostani and Doubling Season, and to a
table that races it before the life total climbs.

| | |
|---|---|
| **Commander** | Trostani, Selesnya's Voice ({G}{G}{W}{W}, 2/5 Dryad) |
| **Bracket** | 3 or 4 by table: one Game Changer (Teferi's Protection) and one two-card infinite (Heliod, Sun-Crowned + Walking Ballista, see "How the deck wins") |
| **Simulator** | deck key `trostani` (tiers t3 and t4); plan file `data/plans/trostani.toml`; findings in `audit/trostani/FINDINGS.md`, history in `decklists/CHANGELOG.md` |

## Strategy

### The commander

**Trostani, Selesnya's Voice** ({G}{G}{W}{W}, 2/5, Dryad). *Whenever another creature you control enters, you gain life
equal to that creature's toughness. {1}{G}{W}, {T}: Populate.*

- **The life trigger reads toughness as it resolves.** A creature that enters with counters (Kalonian Hydra, Hangarback
  Walker, Walking Ballista, Wildwood Scourge) is counted with them, so doubled counters mean doubled life. Elder Gargaroth
  enters as a 6/6 and gives 6.
- **Populate copies a creature token you control,** counters excluded: a token copy is a new token with the printed
  characteristics. Doubling Season or Anointed Procession doubles the copy, and each copy is another creature entering, so
  Trostani gains life for each.
- **She does not trigger for herself.** Verdant Sun's Avatar does ("this creature or another creature"), so it gains its
  own toughness on entering and each other creature's as well.
- **Rhox Faithmender doubles every gain,** and Archangel of Thune and Heliod, Sun-Crowned trigger once per gain event, not
  per point, so one big gain is worth the same counters as one small one.
- **If she dies,** recast her from the command zone; the deck keeps working without her, at a lower life total. Her five
  toughness makes her hard to damage out, so the threats are exile, bounce and "destroy" effects: Heroic Intervention,
  Flawless Maneuver (free while you control a commander) and Teferi's Protection cover them.

### The packages

Each package has a job. The 64 spells split as follows (Trostani is the 100th card, 35 lands the rest).

| Package | Cards | Job |
|---|---|---|
| Ramp (11) | Sol Ring, Arcane Signet, Moss Diamond, Talisman of Unity, Elvish Mystic, Llanowar Elves, Birds of Paradise, Cultivate, Kodama's Reach, Farseek, Nature's Lore | Turn 2-3 mana for the four-to-five-mana engine pieces; Cultivate, Kodama's Reach, Farseek and Nature's Lore also fix colours |
| Counter engine (6) | Doubling Season, Branching Evolution, Innkeeper's Talent, Conclave Mentor, Cathars' Crusade, Anointed Procession | Multiply counters and tokens (Season doubles both, Procession doubles tokens, Branching doubles +1/+1 counters on your creatures, Innkeeper's Talent doubles every counter at level 3) or add them (Conclave Mentor adds one, Cathars' Crusade puts one on each creature for each creature entering) |
| Emiel and blink (7) | Emiel the Blessed, Restoration Angel, Flickerwisp, Ephemerate, Cloudshift, Conjurer's Closet, Sunpearl Kirin | Replay enters triggers and give each creature a new Trostani life gain; Emiel adds a counter to each entering creature for {G/W} |
| Bodies (11) | Kalonian Hydra, Wildwood Scourge, Hangarback Walker, Walking Ballista, Verdant Sun's Avatar, Elder Gargaroth, Treebeard Gracious Host, Serra Angel, The Doctor EMH, Tireless Tracker, Wall of Omens | Creatures that enter with counters, grow from them or pay out when they enter; the X creatures are mana sinks that scale with the doublers |
| Lifegain and wins (7) | Archangel of Thune, Felidar Sovereign, Test of Endurance, Aetherflux Reservoir, Rhox Faithmender, Heliod Sun-Crowned, Craterhoof Behemoth | Make the life total the clock; Craterhoof finishes a wide board |
| Pillowfort (3) | Thalia the Survivor, Archon of Absolution, Sphere of Safety | Archon of Absolution and Sphere of Safety make attacking you cost mana while the life total climbs; Thalia is a lifelink body that taxes the opponents' noncreature spells |
| Reset (2) | Sunblast Angel, Austere Command | Two sweepers the deck can cast when it is behind; Austere Command picks two modes, so it can leave the enchantments or your creatures |
| Tokens and populate (2) | Felidar Retreat, Elspeth Sun's Champion | Tokens for Trostani to copy and for Doubling Season and Anointed Procession to double: Felidar Retreat makes a 2/2 Cat Beast on each land drop (or a +1/+1 counter on each creature), Elspeth three Soldiers a turn |
| Draw (4) | Guardian Project, Beast Whisperer, Sylvan Library, Harmonize | Refill; the creature-heavy list triggers Beast Whisperer and Guardian Project constantly |
| Tutors (3) | Green Sun's Zenith, Sterling Grove, Eladamri's Call | Find a creature (Zenith: a green one, X or less), the engine enchantment (Sterling Grove) or any creature (Eladamri's Call) |
| Interaction and protection (8) | Swords to Plowshares, Path to Exile, Generous Gift, Darksteel Mutation, Heroic Intervention, Teferi's Protection, Akroma's Will, Flawless Maneuver | Spot removal, and the four cards that protect the engine from a sweeper or exile (Witch Enchanter, counted with the lands, is the artifact and enchantment answer) |

Plus 35 lands (see "Mana").

### How the deck wins

Four lines. Lines 1 and 2 are the plan; 3 and 4 finish when the board or the life total is ready.

#### Line 1: the life total

1. Land Trostani and one or two doublers. Each creature that enters gives life equal to its toughness.
2. Archangel of Thune puts a +1/+1 counter on each creature you control for every gain event, and Heliod, Sun-Crowned puts
   one on a creature or enchantment. The counters raise toughness, so the next entry gains more.
3. At 40 life Felidar Sovereign wins at your upkeep; at 50 life Test of Endurance does. Both check at the beginning of your
   upkeep, so you must survive to it with the card on the battlefield, and Rhox Faithmender's doubling gets you there a turn earlier.
4. Aetherflux Reservoir turns the surplus into damage: pay 50 life for 50 damage to one target. You pay it at 50 life or
   more (and lose the game at exactly 50 unless the damage wins), and it kills any one opponent at 50 life or less.

#### Line 2: the counter engine

**Doubling Season** ({4}{G}), **Branching Evolution** ({2}{G}) and **Innkeeper's Talent** at level 3 ({1}{G}, then {G} for level 2 and {3}{G} for level 3) stack
as multipliers: a single +1/+1 counter becomes 2 with Season, 4 with Branching, and 8 with the level 3 Talent; Conclave
Mentor adds one before each doubling is applied, in the order you choose.

- **Entering with counters:** Kalonian Hydra enters with four counters, Hangarback Walker or Walking Ballista with X, and
  Wildwood Scourge with X. With Doubling Season Kalonian enters with eight; with Season and Branching, sixteen.
- **Kalonian Hydra's attack trigger doubles the counters on each creature you control;** the doubling is put as counters, so
  Doubling Season doubles it again (a creature with three counters ends at nine, not six). A creature with no counters stays
  at none.
- **Gavony Township** ({2}{G}{W}, {T}) puts a counter on each creature you control, every turn, with no card in hand.
- **Felidar Retreat** ({3}{W}) turns every land into a trigger: choose a 2/2 Cat Beast token (Trostani gains 2 life, and
  Doubling Season or Anointed Procession doubles it) or a +1/+1 counter on each creature you control (the doublers apply, and
  Wildwood Scourge grows once for each other creature that gets one).
- **Cathars' Crusade** puts a counter on each creature you control whenever a creature enters; the counters are put as the
  trigger resolves, and with Wildwood Scourge on the table each is another counter on the Scourge.
- **Counters are lost when a creature is blinked or bounced:** blink the creature with an enters trigger and few counters,
  not Kalonian Hydra.

#### Line 3: Heliod, Sun-Crowned + Walking Ballista

**Heliod, Sun-Crowned** ({2}{W}): *Whenever you gain life, put a +1/+1 counter on target creature or enchantment you
control. {1}{W}: Another target creature gains lifelink until end of turn.* **Walking Ballista** ({X}{X}): *Remove a +1/+1
counter: it deals 1 damage to any target.*

1. With Ballista holding at least two counters, pay {1}{W}: Ballista gains lifelink until end of turn.
2. Remove a counter and ping an opponent. Lifelink gains you 1 life, Heliod puts a counter on Ballista, and the Ballista is
   back where it started. Repeat for as much damage as the table has life.

- **It needs {1}{W} once and Ballista with two counters,** so one stays after the first ping: with a single counter the
  Ballista dies as the cost is paid and Heliod's trigger has no Ballista to target. Any doubler (Doubling Season,
  Branching Evolution, Innkeeper's Talent level 3, Conclave Mentor) makes each loop a net gain of counters.
- **Heliod is an enchantment even when it is not a creature,** so Sterling Grove gives it shroud against targeted removal.
- **Name it before the game:** it is a two-card infinite combo (see "Bracket and Rule 0").

#### Line 4: Craterhoof Behemoth

**Craterhoof Behemoth** ({5}{G}{G}{G}): *Haste. When it enters, creatures you control gain trample and get +X/+X until end
of turn, where X is the number of creatures you control.* Elspeth, Sun's Champion's three Soldiers (six with Doubling Season
or Anointed Procession, twelve with both), Trostani's populate and Hangarback Walker's Thopters fill the board.
Eladamri's Call and Green Sun's Zenith (X=8) find it.

### Game plan

1. **Turns 1-3: mana.** Play a dork or rock on turn 1-2 and fetch with Cultivate, Kodama's Reach, Farseek or Nature's Lore
   on turn 2-3. Keep Heroic Intervention, Flawless Maneuver or a Swords up from turn 3.
2. **Turns 3-5: the engine.** The best order is Trostani or Emiel first (so every later creature gives life or a counter),
   then Branching Evolution and Doubling Season, then the creatures that enter with counters.
3. **Emiel's {3} blink** replays a creature's enters trigger and gives another Trostani gain. Good targets: Treebeard (two
   Food), The Doctor (a counter on each countered creature), Wall of Omens (a card), Restoration Angel, Sunpearl Kirin and
   Flickerwisp. Do not blink Walking Ballista (it returns with no counters and dies) or a creature loaded with counters.
4. **Turn 5+: the life total.** Sit above 40, play Felidar Sovereign and, if the table can't punish a stall, finish with
   Test of Endurance, Aetherflux Reservoir or a Craterhoof turn.
5. **Hold the wipes.** Cast Austere Command or Sunblast Angel only when the board is lost. Sunblast Angel destroys all
   tapped creatures, including your own mana creatures, so tap your lands first and keep dorks for after it resolves.
   Elspeth's -3 destroys every creature with power 4 or more: yours with counters too.

### Mulligans

Keep a hand with two or three lands, a mana source and one of: an engine card, a creature with an enters trigger, a
protection spell. A hand of only engine cards and expensive creatures with no mana is a mulligan. Do not keep Craterhoof,
Test of Endurance or Felidar Sovereign for the first draw; they win a game the engine has already started. Sylvan Library
costs 4 life for each extra card, which this deck's lifegain pays back.

### What to tutor for

- **Green Sun's Zenith:** the green creature the board is missing: Elder Gargaroth (X=5) for a body that gains life or
  draws, Kalonian Hydra (X=5) with Doubling Season, Craterhoof (X=8) as the finisher. Do not fetch Wildwood Scourge: it
  enters with X counters and X would be zero.
- **Eladamri's Call:** any creature: the Ballista or Felidar Sovereign for a win, Emiel when the engine needs the counters.
- **Sterling Grove:** Doubling Season (it goes on top of the library), then sacrifice the Grove; the Grove also gives your other
  enchantments shroud while it stays, so keep it unless you need the card.

### Protecting the engine

- **Doubling Season, Branching Evolution, Anointed Procession, Cathars' Crusade, Heliod** and the other enchantments are
  vulnerable to Boseiju-style effects, Austere Command and Nevinyrral's Disk. Sterling Grove gives them shroud, which stops
  targeted removal but not a wipe.
- **Against a sweeper:** Teferi's Protection (cast in response; your permanents phase out and come back in your untap step),
  Heroic Intervention (hexproof and indestructible) and Flawless Maneuver (indestructible) all save the board in a
  different way; Akroma's Will gives indestructible and protection from each colour. Phasing leaves counters on the permanents.
- **Boseiju, Who Endures** answers a problem artifact, enchantment or nonbasic land from the hand for {1}{G} (one less for each legendary creature you control).

## What the simulator cannot do yet

- **The AI plays the deck from card code and a small plan, not a full strategy.** `data/plans/trostani.toml` holds the Heliod
  and Ballista line and priorities for the engine cards and Trostani. Card code decides when to use Gavony Township and Castle
  Ardenvale (an opponent's end step with the spare mana), when to populate, which Elder Gargaroth mode to take, when
  Aetherflux Reservoir shoots (only a kill, never while Felidar Sovereign or Test of Endurance would win at the next upkeep)
  and when Sunblast Angel is cast (whenever none of its own creatures would die, so also as a plain 4/5 flier). Emiel pays his
  {G/W} whenever it can, and which creature to blink is the generic rule: there is no mana reserve for Heroic Intervention or
  Teferi's Protection, and Austere Command's mode choice is not tuned for the deck.
- **Teferi's Protection is Approximate** (see `documents/design-teferis-protection-phasing.md`).

## Mana

35 land slots (34 lands and one modal double-faced card): Command Tower, Path of Ancestry, Exotic Orchard, Temple Garden, Sunpetal Grove, Fabled Passage, Evolving Wilds,
Gavony Township, Boseiju Who Endures, Castle Ardenvale, Witch Enchanter // Witch-Blessed Meadow, 12 Forest and 12 Plains. Twenty-one lands make {W} and twenty make
{G} (counting Command Tower, Path of Ancestry, Exotic Orchard, Temple Garden, Sunpetal Grove, Fabled Passage and Evolving
Wilds for both). Eleven ramp cards on top: three green mana creatures (Elvish Mystic, Llanowar Elves, Birds of Paradise, which
also makes {W}), Sol Ring, Arcane Signet, Moss Diamond (enters tapped, {G} only), Talisman of Unity ({G} or {W} for 1 damage
to you) and four land-fetch sorceries.

- **Trostani needs {G}{G}{W}{W}** and Craterhoof {5}{G}{G}{G}: count both colours before you cast either.
- **Tapped lands:** Path of Ancestry always, Sunpetal Grove and Castle Ardenvale unless you control the right land, Fabled
  Passage and Evolving Wilds (they fetch a tapped land), Temple Garden for 2 life or tapped.
- **Exotic Orchard** makes only colours an opponent's land could make; with one-colour opposition it may make none.
- **Castle Ardenvale** ({2}{W}{W}, {T}: a 1/1 Human) and **Gavony Township** ({2}{G}{W}, {T}) are the deck's lands that
  work late; Gavony makes {C} for itself.
- **Witch Enchanter // Witch-Blessed Meadow** is the 35th land slot. Played as a land it taps for {W} and enters tapped unless you
  pay 3 life; cast as a creature ({3}{W}, 2/2 Human Warlock) it destroys an artifact or enchantment an opponent controls as it
  enters, and Trostani and Emiel reward it like any creature. Either way it triggers Felidar Retreat if you play it as a land.
- **Fixing spells:** Farseek finds a Plains or Temple Garden, Nature's Lore a Forest or Temple Garden; Cultivate and Kodama's Reach find either basic.

## Bracket and Rule 0

Bracket 3 or 4, by what the table allows: one Game Changer (Teferi's Protection) and one two-card infinite, Heliod,
Sun-Crowned + Walking Ballista (a late-game combo with no way to start it early). Commander Spellbook also lists Archangel of Thune + Walking Ballista for this list (`data/spellbook/trostani.json`); check its steps before you name it at a table. There are no extra turns and no mass
land destruction. Tell the table before the game about the infinite and Aetherflux Reservoir (50 damage for 50 life, as often as the
life total allows), that the deck has two sweepers (Austere Command and Sunblast Angel) it only casts when behind, and that Archon of
Absolution and Sphere of Safety make attacking you cost mana.

## Decklist by type (100)
**Commander (1).** Trostani, Selesnya's Voice

**Lands (34).** Boseiju Who Endures, Castle Ardenvale, Command Tower, Evolving Wilds, Exotic Orchard, Fabled Passage, Gavony Township, Path of Ancestry, Sunpetal Grove, Temple Garden, 12 Forest, 12 Plains

**Creatures (29).** Archangel of Thune, Archon of Absolution, Beast Whisperer, Birds of Paradise, Conclave Mentor, Craterhoof Behemoth, Elder Gargaroth, Elvish Mystic, Emiel the Blessed, Felidar Sovereign, Flickerwisp, Hangarback Walker, Heliod Sun-Crowned, Kalonian Hydra, Llanowar Elves, Restoration Angel, Rhox Faithmender, Serra Angel, Sunblast Angel, Sunpearl Kirin, Thalia the Survivor, The Doctor EMH, Tireless Tracker, Treebeard Gracious Host, Verdant Sun's Avatar, Walking Ballista, Wall of Omens, Wildwood Scourge, Witch Enchanter // Witch-Blessed Meadow

**Planeswalkers (1).** Elspeth Sun's Champion

**Enchantments (12).** Anointed Procession, Branching Evolution, Cathars' Crusade, Darksteel Mutation, Doubling Season, Felidar Retreat, Guardian Project, Innkeeper's Talent, Sphere of Safety, Sterling Grove, Sylvan Library, Test of Endurance

**Artifacts (6).** Aetherflux Reservoir, Arcane Signet, Conjurer's Closet, Moss Diamond, Sol Ring, Talisman of Unity

**Sorceries (7).** Austere Command, Cultivate, Farseek, Green Sun's Zenith, Harmonize, Kodama's Reach, Nature's Lore

**Instants (10).** Akroma's Will, Cloudshift, Eladamri's Call, Ephemerate, Flawless Maneuver, Generous Gift, Heroic Intervention, Path to Exile, Swords to Plowshares, Teferi's Protection

## Import list (100)

```
1 Aetherflux Reservoir
1 Akroma's Will
1 Anointed Procession
1 Arcane Signet
1 Archangel of Thune
1 Archon of Absolution
1 Austere Command
1 Beast Whisperer
1 Birds of Paradise
1 Boseiju, Who Endures
1 Branching Evolution
1 Castle Ardenvale
1 Cathars' Crusade
1 Cloudshift
1 Command Tower
1 Conclave Mentor
1 Conjurer's Closet
1 Craterhoof Behemoth
1 Cultivate
1 Darksteel Mutation
1 Doubling Season
1 Eladamri's Call
1 Elder Gargaroth
1 Elspeth, Sun's Champion
1 Elvish Mystic
1 Emiel the Blessed
1 Ephemerate
1 Evolving Wilds
1 Exotic Orchard
1 Fabled Passage
1 Farseek
1 Felidar Retreat
1 Felidar Sovereign
1 Flawless Maneuver
1 Flickerwisp
1 Gavony Township
1 Generous Gift
1 Green Sun's Zenith
1 Guardian Project
1 Hangarback Walker
1 Harmonize
1 Heliod, Sun-Crowned
1 Heroic Intervention
1 Innkeeper's Talent
1 Kalonian Hydra
1 Kodama's Reach
1 Llanowar Elves
1 Moss Diamond
1 Nature's Lore
1 Path of Ancestry
1 Path to Exile
1 Restoration Angel
1 Rhox Faithmender
1 Serra Angel
1 Sol Ring
1 Sphere of Safety
1 Sterling Grove
1 Sunblast Angel
1 Sunpearl Kirin
1 Sunpetal Grove
1 Swords to Plowshares
1 Sylvan Library
1 Talisman of Unity
1 Teferi's Protection
1 Temple Garden
1 Test of Endurance
1 Thalia, the Survivor
1 The Doctor, EMH
1 Tireless Tracker
1 Treebeard, Gracious Host
1 Trostani, Selesnya's Voice
1 Verdant Sun's Avatar
1 Walking Ballista
1 Wall of Omens
1 Wildwood Scourge
1 Witch Enchanter // Witch-Blessed Meadow
12 Forest
12 Plains
```
