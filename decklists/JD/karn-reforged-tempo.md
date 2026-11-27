# Karn, Legacy Reforged — Artifact Tempo (Colourless, Bracket 4)

Karn, Legacy Reforged is a five-mana artifact creature that is as big as your biggest artifact and, every upkeep, adds
{C} for each artifact you control. That mana can't cast nonartifact spells, but it pays for artifact spells and for every
ability cost: equip, Mystic Forge, Skullclamp, land abilities and Rings of Brighthearth copies. The deck is built to land
him on turn 2 or 3 with fast mana, then turn each later artifact into more mana: a Sword or a Clamp ramps like a rock, so
the plan is to cast cheap artifacts, equip, and end the game with a hexproof or unblockable Karn, with Wurmcoil Engine,
Blightsteel Colossus and the planeswalkers behind him. Mana Crypt and Jeweled Lotus (the turn-1 Karn line) are banned in
Commander, so the fastest start is turn 2. The deck is colourless, so All Is Dust and the Ugin sweepers hit only the
opponents' coloured permanents. It is weak to artifact removal that exiles or destroys everything (Vandalblast, Bane of
Progress, Austere Command), to Collector Ouphe, Null Rod and Stony Silence, to a counterspell on Karn when he is cast, and
to chump blockers when Karn has no evasion. **No sim has measured this list; every number below is a count from the list or
a labelled estimate.**

| | |
|---|---|
| **Commander** | Karn, Legacy Reforged ({5}, legendary artifact creature — Golem, power and toughness each equal to the greatest mana value among artifacts you control; colourless) |
| **Bracket** | 4: six Game Changers and one infinite line. Played against Tier 4 and Tier 5 |
| **Game Changers (6)** | Ancient Tomb, Grim Monolith, Mana Vault, Mishra's Workshop, Mox Diamond, The One Ring |
| **Simulator** | Deck key `karn_reforged`, tiers t4-t5; findings in `audit/karn_reforged/FINDINGS.md`, history in `decklists/CHANGELOG.md`; plan file `data/plans/karn_reforged.toml` |
| **Buys** | None: every card he does not own is proxied |

## Strategy

### The commander

**Karn, Legacy Reforged** ({5}, colourless). *Karn's power and toughness are each equal to the greatest mana value among
artifacts you control. At the beginning of your upkeep, add {C} for each artifact you control. This mana can't be spent to
cast nonartifact spells. Until end of turn, you don't lose this mana as steps and phases end.*

- He is an artifact himself, so he is at least 5/5 and counts himself. Blightsteel Colossus (mana value 12) makes him 12/12.
- The mana comes at upkeep and lasts the turn: with six artifacts that is {C}{C}{C}{C}{C}{C} for artifact spells and
  abilities, on top of your lands. The six nonartifact cards in the deck (five planeswalkers and All Is Dust) need lands,
  Sol Ring or other rocks, not Karn's mana. Mishra's Workshop pays for artifact spells only, so it also can't cast them.
- **Fast starts.** Turn 2 Karn from Sol Ring or Mox Diamond plus Ancient Tomb, Mana Vault, Grim Monolith, Mox Opal,
  Lotus Petal or Mishra's Workshop. A hand with Sol Ring and any two of those casts him.
- **If he is removed.** Command Beacon ({T}, sacrifice) puts him into your hand from the command zone, so he is cast
  from hand without commander tax (from the command zone the recasts cost {7}, then {9}). Swiftfoot Boots (hexproof), Lightning Greaves (shroud and haste) and Mithril Coat (indestructible) keep him
  alive; Champion's Helm gives hexproof for equip {1} because he is legendary, and Hammer of Nazahn gives indestructible.
  None of them stops an exile or bounce wipe, and Defense Grid is the stack protection for the turn he lands.

### The packages

1. **Fast mana (15 cards).** Sol Ring, Mana Vault, Grim Monolith, Mox Opal, Lotus Petal, Mox Diamond, Basalt Monolith,
   Thran Dynamo, Mind Stone, Thought Vessel, Everflowing Chalice, Worn Powerstone, Coalition Relic, Fellwar Stone and Solemn
   Simulacrum. Every one is also an artifact for Karn's count. Voltaic Key untaps Mana Vault and the Monoliths, and
   Unwinding Clock untaps all artifacts during an opponent's untap step. *Chrome Mox, Mox Amber, Arcane Signet and
   Commander's Sphere are not in the list: they cannot make mana with a colourless commander.*
2. **Cheap artifacts (8).** Ornithopter, Mishra's Bauble, Walking Ballista, Hangarback Walker, Hope of Ghirapur, Foundry
   Inspector, Retrofitter Foundry and Wayfarer's Bauble raise Karn's count for little, and Walking Ballista, Hangarback
   Walker and Retrofitter Foundry are mana sinks for the Karn mana.
3. **Card flow and tutors (10).** Skullclamp turns every X/1 into cards, The One Ring protects and draws, Mystic Forge
   plays the top artifact, Mazemind Tome and Scroll Rack sink Karn's mana, Sensei's Divining Top finds the next piece,
   Expedition Map fetches the land you need (an Urza land, Mutavault, Rogue's Passage), Sculpting Steel copies the best artifact (it can't copy Karn while he is on the battlefield without losing one to the legend rule), and Staff of Domination draws with spare mana.
4. **Equipment and threats (16).** Eight Swords and equipment (Fire and Ice, Feast and Famine, Truth and Justice, Forge and
   Frontier, War and Peace, Sinew and Steel, Batterskull, Umezawa's Jitte), Shadowspear, Champion's Helm and Whispersilk
   Cloak (unblockable and shroud), Wurmcoil Engine, Myr Battlesphere, Steel Hellkite, Triplicate Titan and Blightsteel
   Colossus. Rogue's Passage ({4}, {T}) makes Karn unblockable too, paid by Karn's mana.
5. **Planeswalkers (5).** Tezzeret, Cruel Captain, Karn, the Great Creator (opponents' artifacts can't use their abilities),
   Karn, Scion of Urza, Ugin, the Ineffable and Ugin, the Spirit Dragon.
6. **Interaction (5) and disruption (3).** Meteor Golem, Duplicant, Spine of Ish Sah, All Is Dust, The Mightstone and
   Weakstone (-5/-5 or draw two, and it is an artifact that Karn's mana pays for), Winter Orb (only lands stop untapping:
   Karn's mana and the rocks ignore it), Defense Grid and Unwinding Clock.
7. **Late engines (4) and enablers (4).** Mycosynth Lattice (every permanent becomes an artifact, so Karn's upkeep mana
   counts every permanent on the table and his size is the greatest mana value of any permanent; it also makes everything
   colourless, which turns off All Is Dust and both Ugin sweepers, so don't count them live with it out), Platinum Angel, Cityscape Leveler, Portal to Phyrexia,
   Aetherflux Reservoir, Hammer of Nazahn, Rings of Brighthearth and Steel Overseer.

### How the deck wins

1. **Commander damage and combat.** Karn is 5/5 at minimum. Equipped and unblockable (Whispersilk Cloak, Rogue's Passage)
   he needs 21 commander damage: four hits as a 5/5, and fewer as he grows (Blightsteel makes him 12/12) or carries a
   Sword or Batterskull.
2. **Basalt Monolith + Rings of Brighthearth.** With {5} available, Basalt Monolith's untap copied by Rings of Brighthearth
   makes {C} each loop; Walking Ballista (or Staff of Domination, or Mycosynth Lattice with an artifact outlet) ends the
   game from that mana. Karn's upkeep mana pays for it.
3. **Wurmcoil, Blightsteel, Portal and the Ugins** close when the commander plan is stopped.

### Game plan

Turn 1: a fast rock or a cheap artifact. Turn 2-3: Karn, with Defense Grid or a protection piece ready if you can.
Each upkeep: count the artifacts, then spend Karn's mana first (on equip, Mystic Forge, Skullclamp, Rogue's Passage and
artifact spells) and keep lands and rocks for the nonartifact cards. Wasteland can destroy only a nonbasic land; Strip
Mine any land. Use them on a utility or fixing land when it costs the opponent the most.

### Mulligans

Keep: any hand with a fast rock and Karn in reach (two or more of Sol Ring, Mox Diamond, Ancient Tomb, Mana Vault, Grim
Monolith, Mox Opal, Lotus Petal, Mishra's Workshop); any hand with a cheap artifact engine (Skullclamp plus Hangarback
Walker or Ballista) and a land. Mulligan: only lands with no artifact; only five-plus-drops; no land and no fast mana.

## Limits (what the simulator and the engine do not do)

- No sim has measured this list.
- Karn's mana is modelled to cast only artifact spells and to pay ability costs; the Rings of Brighthearth loop resolves as
  its result.

## Mana

26 lands: Ancient Tomb, Mishra's Workshop, Urza's Saga, Darksteel Citadel, Command Beacon, Urza's Mine, Urza's Power Plant,
Urza's Tower, Strip Mine, Wasteland, Mutavault, Inventors' Fair, Rogue's Passage, War Room, Bonders' Enclave, Buried Ruin
and ten Wastes. The three Urza lands make {C}{C}{C} together and Urza's Saga fetches a zero- or one-mana artifact (Sol Ring,
Mox Opal, Lotus Petal, Mox Diamond, Ornithopter, Mishra's Bauble, Walking Ballista, Voltaic Key, Wayfarer's Bauble). With
the fifteen mana artifacts, that is 41 mana sources before Karn. War Room draws for {3} and no life, because Karn has no
colours. Bonders' Enclave draws when you control a creature with power 4 or more (Karn is at least 5/5).

## Bracket and Rule 0

**Bracket 4,** with six Game Changers: **Ancient Tomb**, **Grim Monolith**, **Mana Vault**, **Mishra's Workshop**,
**Mox Diamond** and **The One Ring**. It has no extra turns. It does have land destruction (Strip Mine and Wasteland, one
land each) and a stax piece (Winter Orb).

Disclose before the game:

- **The six Game Changers** above.
- **Infinite line:** Basalt Monolith with Rings of Brighthearth, ending in Walking Ballista.
- **Land destruction and stax:** Strip Mine, Wasteland and Winter Orb.
- **Tutors:** Urza's Saga (a zero- or one-mana artifact) and Expedition Map (a land).
- **Fast mana:** Sol Ring, Mana Vault, Grim Monolith, Mox Opal, Lotus Petal, Mox Diamond and Ancient Tomb.

Re-check the official list at https://commanderbrackets.com/faq before an event.

## Decklist by package (100)

**Commander (1).** Karn, Legacy Reforged

**Lands (26).** Ancient Tomb, Mishra's Workshop, Urza's Saga, Darksteel Citadel, Command Beacon, Urza's Mine, Urza's Power Plant, Urza's Tower, Strip Mine, Wasteland, Mutavault, Inventors' Fair, Rogue's Passage, War Room, Bonders' Enclave, Buried Ruin, 10 Wastes

**Mana artifacts (15).** Sol Ring, Mana Vault, Grim Monolith, Mox Opal, Lotus Petal, Mox Diamond, Basalt Monolith, Thran Dynamo, Mind Stone, Thought Vessel, Everflowing Chalice, Worn Powerstone, Coalition Relic, Fellwar Stone, Solemn Simulacrum

**Cheap artifacts (8).** Ornithopter, Mishra's Bauble, Walking Ballista, Hangarback Walker, Hope of Ghirapur, Foundry Inspector, Retrofitter Foundry, Wayfarer's Bauble

**Card flow and tutors (10).** Skullclamp, The One Ring, Mystic Forge, Staff of Domination, Sculpting Steel, Expedition Map, Voltaic Key, Sensei's Divining Top, Mazemind Tome, Scroll Rack

**Planeswalkers (5).** Tezzeret, Cruel Captain, Karn, the Great Creator, Ugin, the Spirit Dragon, Karn, Scion of Urza, Ugin, the Ineffable

**Threats and equipment (16).** Wurmcoil Engine, Myr Battlesphere, Steel Hellkite, Blightsteel Colossus, Triplicate Titan, Sword of Fire and Ice, Sword of Feast and Famine, Sword of Truth and Justice, Sword of Forge and Frontier, Sword of War and Peace, Sword of Sinew and Steel, Whispersilk Cloak, Batterskull, Umezawa's Jitte, Shadowspear, Champion's Helm

**Interaction (5).** Meteor Golem, Duplicant, Spine of Ish Sah, All Is Dust, The Mightstone and Weakstone

**Disruption (3).** Winter Orb, Defense Grid, Unwinding Clock

**Protection (3).** Swiftfoot Boots, Lightning Greaves, Mithril Coat

**Late engines and finishers (4).** Mycosynth Lattice, Platinum Angel, Cityscape Leveler, Portal to Phyrexia

**Engine enablers (4).** Aetherflux Reservoir, Hammer of Nazahn, Rings of Brighthearth, Steel Overseer

## Import list (100)

```
1 Aetherflux Reservoir
1 All Is Dust
1 Ancient Tomb
1 Basalt Monolith
1 Batterskull
1 Blightsteel Colossus
1 Bonders' Enclave
1 Buried Ruin
1 Champion's Helm
1 Cityscape Leveler
1 Coalition Relic
1 Command Beacon
1 Darksteel Citadel
1 Defense Grid
1 Duplicant
1 Everflowing Chalice
1 Expedition Map
1 Fellwar Stone
1 Foundry Inspector
1 Grim Monolith
1 Hammer of Nazahn
1 Hangarback Walker
1 Hope of Ghirapur
1 Inventors' Fair
1 Karn, Legacy Reforged
1 Karn, Scion of Urza
1 Karn, the Great Creator
1 Lightning Greaves
1 Lotus Petal
1 Mana Vault
1 Mazemind Tome
1 Meteor Golem
1 Mind Stone
1 Mishra's Bauble
1 Mishra's Workshop
1 Mithril Coat
1 Mox Diamond
1 Mox Opal
1 Mutavault
1 Mycosynth Lattice
1 Myr Battlesphere
1 Mystic Forge
1 Ornithopter
1 Platinum Angel
1 Portal to Phyrexia
1 Retrofitter Foundry
1 Rings of Brighthearth
1 Rogue's Passage
1 Scroll Rack
1 Sculpting Steel
1 Sensei's Divining Top
1 Shadowspear
1 Skullclamp
1 Sol Ring
1 Solemn Simulacrum
1 Spine of Ish Sah
1 Staff of Domination
1 Steel Hellkite
1 Steel Overseer
1 Strip Mine
1 Swiftfoot Boots
1 Sword of Feast and Famine
1 Sword of Fire and Ice
1 Sword of Forge and Frontier
1 Sword of Sinew and Steel
1 Sword of Truth and Justice
1 Sword of War and Peace
1 Tezzeret, Cruel Captain
1 The Mightstone and Weakstone
1 The One Ring
1 Thought Vessel
1 Thran Dynamo
1 Triplicate Titan
1 Ugin, the Ineffable
1 Ugin, the Spirit Dragon
1 Umezawa's Jitte
1 Unwinding Clock
1 Urza's Mine
1 Urza's Power Plant
1 Urza's Saga
1 Urza's Tower
1 Voltaic Key
1 Walking Ballista
1 War Room
1 Wasteland
10 Wastes
1 Wayfarer's Bauble
1 Whispersilk Cloak
1 Winter Orb
1 Worn Powerstone
1 Wurmcoil Engine
```
