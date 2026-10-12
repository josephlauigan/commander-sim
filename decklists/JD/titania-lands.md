# Titania, Protector of Argoth — Mono-Green Lands to the Graveyard

**Simulator, 2026-10-12:** **22.6% over all tiers** (±0.4; rank 7 of the ten decks): T1 19.5%, T2 27.5%, T3 24.2%, T4 24.3%, T5 17.5%. Look-ahead AI, loose profile, 6,720 games per tier, every deck on the same code; `decklists/pool/pool-results.md` section 0d.

**Added 2026-10-11.** JD's mono-green deck, built from the bulk and binder lists around Titania, Protector of Argoth
(borderless Secret Lair printing, about $2.46), with no other buys apart from Arachnogenesis and Blossoming Defense
(bought 2026-10-11). Chosen over a Bristly Bill landfall build of mostly the same cards, which measured 15.0%.
Fully modeled and measured on 2026-10-11: **22.8% over all tiers** (see "Simulator status").

## Strategy

Whenever a land you control goes from the battlefield to the graveyard, Titania makes a 5/3 Elemental. When she
enters, she returns a land from your graveyard to the battlefield.

- **Lands that go to the graveyard.** Evolving Wilds, Terramorphic Expanse and Fabled Passage sacrifice themselves to
  find a basic: each is a 5/3. Titania's enter trigger returns one of them, ready to crack again, so even removal on
  her pays: every recast is another land, another 5/3 and (with Garruk's Uprising) another card.
- **Fill the graveyard and the land drops.** Satyr Wayfinder, Malevolent Rumble, Something Worth Saving, Scout the
  City, Pothole Mole (returns a land to hand) and the land ramp. The AI keeps fetch lands in the graveyard for
  Titania rather than taking them to hand.
- **Cash it in.** Garruk's Uprising draws a card for each 5/3 (power 4 or more) and gives them trample; Overwhelming
  Stampede and End-Raze Forerunners turn a pile of 5/3s into a lethal attack. Big green creatures (Ghalta, Hornet
  Queen, Paladin of Predation, Yavimaya Wurm) carry the deck when the engine is slow.
- **Stay alive.** Arachnogenesis (a fog plus a 1/2 reach Spider for each attacker), Blossoming Defense and Royal
  Treatment to protect Titania, and bite and fight spells (Bite Down, Hard-Hitting Question, Knockout Maneuver and
  friends) as removal.

**Cheap upgrades, if wanted later** (about $1.80, not in the list): Myriad Landscape, Escape Tunnel, Harrow,
Springbloom Druid and Splendid Reclamation, each one more land to the graveyard (Splendid Reclamation returns them all).

## Key lines

- **Turns 1-4:** ramp (Llanowar Elves, Rampant Growth, Nature's Lore, Cultivate) and graveyard fillers. Play a fetch
  land but hold it uncracked when Titania is a turn or two away.
- **Titania lands:** crack the held fetch lands (a 5/3 each); her enter trigger returns a cracked one to crack again.
- **Removal on her:** Blossoming Defense or Royal Treatment in response; if she dies anyway, recast her: each recast
  returns a fetch land.
- **Big attack coming:** Arachnogenesis when an attack would deal a third of your life or more.

## Consistency

38 lands: 35 Forest plus Evolving Wilds, Terramorphic Expanse and Fabled Passage. **An exception to JD's
basics-over-tapped-lands rule:** the basics they find enter tapped, but each sacrifice is a 5/3, the deck's engine.
Ramp: Llanowar Elves, Noxious Newt, Hardbristle Bandit, Topiary Lecturer, Silhana Starfletcher, Greenhouse Propagator,
Bugenhagen, plus Rampant Growth, Nature's Lore, Cultivate, Skyshroud Claim and Spider-Man.

## Simulator status

**Measured 2026-10-11** (look-ahead AI, loose profile, 960 games per tier, the same settings as the other decks;
`decklists/pool/pool-results.md` section 0d, results in `audit/titania/`): T1 21.0%, T2 27.0%, T3 24.8%, T4 23.6%,
T5 17.6%; **22.8% over all tiers**, near Veyran (23.7%). Wins come late (turn 13.9 on average); she is often removed
and recast, and each recast returns a fetch land for another 5/3.

Known simplifications: several lands going to the graveyard at once make one 5/3, not one each; day and night
(Infestation Expert) is modeled only as far as it changes that card.

## Where the cards come from

**New (3):** Titania, Protector of Argoth (the commander, borderless Secret Lair), Arachnogenesis and Blossoming
Defense (both bought 2026-10-11). **Basics:** 35 Forest, your own. Every other card comes off JD's lists (2026-10-11),
one copy each, grouped by color so you can pull them a color at a time: 2 from the binder (Llanowar Elves, Evolving
Wilds), 60 from bulk. Set codes are the printings the bulk list shows; where it names more than one, any of them.

### Green (59)

| Card | Type | From | Set |
|---|---|---|---|
| Annoyed Altisaur | Creature | Bulk | mh3 |
| Bestial Incursion | Sorcery | Bulk | fra |
| Bite Down | Instant | Bulk | j25 |
| Boughside Wanderers | Creature | Bulk | hob |
| Bugenhagen, Wise Elder | Creature | Bulk | fic |
| Cactarantula | Creature | Bulk | otj |
| Carnivorous Canopy | Sorcery | Bulk | one |
| Chelonian Tackle | Sorcery | Bulk | sos |
| Compel Brutality | Instant | Bulk | fra |
| Crushing Canopy | Instant | Bulk | xln |
| Cultivate | Sorcery | Bulk | fic, ump |
| Diplomatic Relations | Instant | Bulk | eoe |
| Elfhame Wurm | Creature | Bulk | dmu |
| End-Raze Forerunners | Creature | Bulk | blc |
| Environmental Scientist | Creature | Bulk | sos |
| Follow the Lumarets | Sorcery | Bulk | sos |
| Garruk's Uprising | Enchantment | Bulk | fdn |
| Genesis Hydra | Creature | Bulk | otc |
| Ghalta the Unstoppable | Creature | Bulk | fra |
| Glorious Decay | Instant | Bulk | sos |
| Greenbelt Radical | Creature | Bulk | mkm |
| Greenhouse Propagator | Creature | Bulk | fra |
| Hard-Hitting Question | Sorcery | Bulk | mkm |
| Hardbristle Bandit | Creature | Bulk | otj |
| Harmonize | Sorcery | Bulk | fic |
| Hornet Queen | Creature | Bulk | mkc |
| Infestation Expert | Creature | Bulk | vow |
| Knockout Maneuver | Sorcery | Bulk | soa |
| Lashwhip Predator | Creature | Bulk | eoe |
| Lifestream's Blessing | Instant | Bulk | fic |
| Llanowar Elves | Creature | Binder | — |
| Malevolent Rumble | Sorcery | Bulk | mh3 |
| Nature's Lore | Sorcery | Bulk | fic |
| Noxious Newt | Creature | Bulk | sos |
| Overwhelming Stampede | Sorcery | Bulk | msc |
| Paladin of Predation | Creature | Bulk | one |
| Penumbra Spider | Creature | Bulk | tsr |
| Pick Your Poison | Sorcery | Bulk | soa |
| Pothole Mole | Creature | Bulk | dft |
| Rampant Growth | Sorcery | Bulk | fic |
| Rending Vines | Instant | Bulk | sok |
| Royal Treatment | Instant | Bulk | soa |
| Satyr Wayfinder | Creature | Bulk | c16 |
| Scout the City | Sorcery | Bulk | spm |
| Shamanic Revelation | Sorcery | Bulk | scd |
| Silhana Starfletcher | Creature | Bulk | gpt |
| Skyshroud Claim | Sorcery | Bulk | bbd |
| Something Worth Saving | Instant | Bulk | fra |
| Spider-Man, Brooklyn Visionary | Creature | Bulk | spm |
| Studious First-Year | Creature | Bulk | sos |
| Symbiotic Beast | Creature | Bulk | dmr |
| Tenured Concocter | Creature | Bulk | sos |
| Thirsting Roots | Sorcery | Bulk | one |
| Thrashing Brontodon | Creature | Bulk | list |
| Topiary Lecturer | Creature | Bulk | sos |
| Track Down | Sorcery | Bulk | m21 |
| Tumbleweed Rising | Sorcery | Bulk | otj |
| Vinelasher Adept | Creature | Bulk | fra |
| Yavimaya Wurm | Creature | Bulk | m11 |

### Lands (3, colorless)

| Card | From | Set |
|---|---|---|
| Evolving Wilds | Binder | — |
| Fabled Passage | Bulk | soc |
| Terramorphic Expanse | Bulk | sos |

## Decklist by type (100)

**Commander (1).** Titania, Protector of Argoth

**Creatures (30).** Annoyed Altisaur, Boughside Wanderers, Bugenhagen Wise Elder, Cactarantula, Elfhame Wurm, End-Raze Forerunners, Environmental Scientist, Genesis Hydra, Ghalta the Unstoppable, Greenbelt Radical, Greenhouse Propagator, Hardbristle Bandit, Hornet Queen, Infestation Expert, Lashwhip Predator, Llanowar Elves, Noxious Newt, Paladin of Predation, Penumbra Spider, Pothole Mole, Satyr Wayfinder, Silhana Starfletcher, Spider-Man Brooklyn Visionary, Studious First-Year, Symbiotic Beast, Tenured Concocter, Thrashing Brontodon, Topiary Lecturer, Vinelasher Adept, Yavimaya Wurm

**Enchantments (1).** Garruk's Uprising

**Instants (11).** Arachnogenesis, Bite Down, Blossoming Defense, Compel Brutality, Crushing Canopy, Diplomatic Relations, Glorious Decay, Lifestream's Blessing, Rending Vines, Royal Treatment, Something Worth Saving

**Sorceries (19).** Bestial Incursion, Carnivorous Canopy, Chelonian Tackle, Cultivate, Follow the Lumarets, Hard-Hitting Question, Harmonize, Knockout Maneuver, Malevolent Rumble, Nature's Lore, Overwhelming Stampede, Pick Your Poison, Rampant Growth, Scout the City, Shamanic Revelation, Skyshroud Claim, Thirsting Roots, Track Down, Tumbleweed Rising

**Lands (38).** Evolving Wilds, Fabled Passage, Terramorphic Expanse, 35 Forest

## Import list (100)

```
1 Annoyed Altisaur
1 Arachnogenesis
1 Bestial Incursion
1 Bite Down
1 Blossoming Defense
1 Boughside Wanderers
1 Bugenhagen, Wise Elder
1 Cactarantula
1 Carnivorous Canopy
1 Chelonian Tackle
1 Compel Brutality
1 Crushing Canopy
1 Cultivate
1 Diplomatic Relations
1 Elfhame Wurm
1 End-Raze Forerunners
1 Environmental Scientist
1 Evolving Wilds
1 Fabled Passage
1 Follow the Lumarets
1 Garruk's Uprising
1 Genesis Hydra
1 Ghalta the Unstoppable
1 Glorious Decay
1 Greenbelt Radical
1 Greenhouse Propagator
1 Hard-Hitting Question
1 Hardbristle Bandit
1 Harmonize
1 Hornet Queen
1 Infestation Expert
1 Knockout Maneuver
1 Lashwhip Predator
1 Lifestream's Blessing
1 Llanowar Elves
1 Malevolent Rumble
1 Nature's Lore
1 Noxious Newt
1 Overwhelming Stampede
1 Paladin of Predation
1 Penumbra Spider
1 Pick Your Poison
1 Pothole Mole
1 Rampant Growth
1 Rending Vines
1 Royal Treatment
1 Satyr Wayfinder
1 Scout the City
1 Shamanic Revelation
1 Silhana Starfletcher
1 Skyshroud Claim
1 Something Worth Saving
1 Spider-Man, Brooklyn Visionary
1 Studious First-Year
1 Symbiotic Beast
1 Tenured Concocter
1 Terramorphic Expanse
1 Thirsting Roots
1 Thrashing Brontodon
1 Titania, Protector of Argoth
1 Topiary Lecturer
1 Track Down
1 Tumbleweed Rising
1 Vinelasher Adept
1 Yavimaya Wurm
35 Forest
```
