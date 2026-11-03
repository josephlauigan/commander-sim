# Brago — Azorius Blink Control

Opponent pool deck — **Tier 2: Mid Bracket 3**. Axis: Control / interaction-heavy.

- **Commander:** Brago, King Eternal
- **Color identity:** White, Blue
- **Game Changers (2):** Consecrated Sphinx, Farewell
- **Combos:** Peregrine Drake + Archaeomancer + Ghostly Flicker (unbounded mana; EDHREC's Deadeye Navigator, which made a two-card infinite with the Drake, is cut for Skyclave Apparition under the Tier 2 rule). It wins through Agent of Treachery, blinked until it has taken every permanent.
- **Lands:** 34 (14 nonbasic + 20 basic)

## Role in the pool

The pool's enters-the-battlefield engine: every creature and Aura does something as it enters, and a dozen cards make it enter again. Tests whether decks can answer a value engine before it buries them, and whether their own permanents survive Agent of Treachery, Riptide Gearhulk and Reality Acid.

## At the table

- **Plan:** Builds a board of enters creatures (Mulldrifter, Wall of Omens, Reflector Mage, Agent of Treachery, Riptide Gearhulk, Sun Titan, Angel of Serenity) and blinks them: Brago, King Eternal when he connects, Soulherder and Teleportation Circle each end step, Restoration Angel, Flickerwisp, Yorion, and the flicker spells Ephemerate, Cloudshift, Momentary Blink, Ghostly Flicker and Eerie Interlude. Panharmonicon and Elesh Norn, Mother of Machines make each enters trigger count twice. Oblivion Ring, Banishing Light and Detention Sphere are removal that a blink re-aims, and Mother of Runes and Giver of Runes shield Brago and the blinkers.
- **Wins by:** Grinds the table out of permanents and cards, then attacks with fliers and token makers (Brago, Cloudblazer, Restoration Angel, Baneslayer Angel, Resplendent Angel, Archangel of Tithes, Hero of Bladehold, Adeline). Peregrine Drake with Archaeomancer and Ghostly Flicker makes unbounded mana and blinks Agent of Treachery until it has everything.
- **Watch out for:** Agent of Treachery takes your best permanent, again each time it is blinked; Reality Acid on your best permanent kills it when the Acid leaves, and Brago blinks it. Elesh Norn stops your own enters triggers. Kill Brago, and answer Peregrine Drake before Archaeomancer arrives.
- **Interaction:** Counterspell, Mana Drain, Dovin's Veto and An Offer You Can't Refuse. Swords to Plowshares, Path to Exile, Reflector Mage, Palace Jailer, Venser, Shaper Savant, Riptide Gearhulk, Flickerwisp, Reality Acid, Witch Enchanter, Loran of the Third Path, Agent of Treachery; Oblivion Ring, Banishing Light and Detention Sphere; Man-o'-War bounces; Farewell is the one sweeper. Ghostly Prison taxes attacks.

## Strategy

Ramp with rocks (Sol Ring, Signets, Talisman, Basalt Monolith, Mind Stone), land a value creature a turn, and blink it as often as possible. Brago's combat damage blinks every enters permanent (and Reality Acid, whose host dies); Soulherder and Teleportation Circle do it each end step. The combos are a bonus: the deck plays the value game first.

## Sim modeling notes

Model these first; if any is Partial or unmodeled, this deck plays below its real strength and its results should be flagged.

ETB re-use through flicker (Brago, Soulherder, Teleportation Circle, Restoration Angel, Flickerwisp, Yorion, the flicker spells); trigger doubling (Panharmonicon, Elesh Norn, Mother of Machines); steals (Agent of Treachery); Reality Acid's delayed removal; counterspell decisions. All of the list's cards are modeled (impls/t12_brago_meren.rs for the Tier 2 additions); left out: Yorion's companion, lands for Ghostly Flicker and Agent of Treachery.

## Decklist by type (100)

**Commander (1).** Brago, King Eternal

**Creatures (42).** Adeline, Resplendent Cathar, Agent of Treachery, Angel of Serenity, Archaeomancer, Archangel of Tithes, Aven Mindcensor, Baneslayer Angel, Brimaz, King of Oreskos, Cathar Commando, Cloudblazer, Consecrated Sphinx, Displacer Kitten, Dream Trawler, Eldrazi Displacer, Elesh Norn, Mother of Machines, Faerie Mastermind, Fiend Hunter, Flickerwisp, Giver of Runes, Hero of Bladehold, Luminarch Aspirant, Lyra Dawnbringer, Man-o'-War, Mirror Entity, Mother of Runes, Mulldrifter, Palace Jailer, Peregrine Drake, Reflector Mage, Resplendent Angel, Restoration Angel, Riptide Gearhulk, Serra Angel, Skyclave Apparition, Solitude, Soulherder, Spark Double, Sun Titan, Venser, Shaper Savant, Wall of Omens, Whirler Rogue, Yorion, Sky Nomad

**Planeswalkers (4).** Elspeth, Sun's Champion, Teferi, Hero of Dominaria, The Wandering Emperor, Venser, the Sojourner

**Enchantments (4).** Banishing Light, Ghostly Prison, Oblivion Ring, Reality Acid

**Artifacts (5).** Arcane Signet, Azorius Signet, Mind Stone, Sol Ring, Talisman of Progress

**Sorceries (2).** Council's Judgment, Farewell

**Instants (8).** An Offer You Can't Refuse, Counterspell, Ephemerate, Fateful Absence, Ghostly Flicker, Mana Drain, Path to Exile, Swords to Plowshares

**Lands (34).** Adarkar Wastes, Azorius Chancery, Command Tower, Deserted Beach, Flooded Strand, Glacial Fortress, Hallowed Fountain, Nimbus Maze, Prairie Stream, Reliquary Tower, Rogue's Passage, Sea of Clouds, Skycloud Expanse, Temple of Enlightenment, 10 Island, 10 Plains

## Import list (100)

```
1 Adarkar Wastes
1 Adeline, Resplendent Cathar
1 Agent of Treachery
1 An Offer You Can't Refuse
1 Angel of Serenity
1 Arcane Signet
1 Archaeomancer
1 Archangel of Tithes
1 Aven Mindcensor
1 Azorius Chancery
1 Azorius Signet
1 Baneslayer Angel
1 Banishing Light
1 Brago, King Eternal
1 Brimaz, King of Oreskos
1 Cathar Commando
1 Cloudblazer
1 Command Tower
1 Consecrated Sphinx
1 Council's Judgment
1 Counterspell
1 Deserted Beach
1 Displacer Kitten
1 Dream Trawler
1 Eldrazi Displacer
1 Elesh Norn, Mother of Machines
1 Elspeth, Sun's Champion
1 Ephemerate
1 Faerie Mastermind
1 Farewell
1 Fateful Absence
1 Fiend Hunter
1 Flickerwisp
1 Flooded Strand
1 Ghostly Flicker
1 Ghostly Prison
1 Giver of Runes
1 Glacial Fortress
1 Hallowed Fountain
1 Hero of Bladehold
1 Luminarch Aspirant
1 Lyra Dawnbringer
1 Man-o'-War
1 Mana Drain
1 Mind Stone
1 Mirror Entity
1 Mother of Runes
1 Mulldrifter
1 Nimbus Maze
1 Oblivion Ring
1 Palace Jailer
1 Path to Exile
1 Peregrine Drake
1 Prairie Stream
1 Reality Acid
1 Reflector Mage
1 Reliquary Tower
1 Resplendent Angel
1 Restoration Angel
1 Riptide Gearhulk
1 Rogue's Passage
1 Sea of Clouds
1 Serra Angel
1 Skyclave Apparition
1 Skycloud Expanse
1 Sol Ring
1 Solitude
1 Soulherder
1 Spark Double
1 Sun Titan
1 Swords to Plowshares
1 Talisman of Progress
1 Teferi, Hero of Dominaria
1 Temple of Enlightenment
1 The Wandering Emperor
1 Venser, Shaper Savant
1 Venser, the Sojourner
1 Wall of Omens
1 Whirler Rogue
1 Yorion, Sky Nomad
10 Island
10 Plains
```
