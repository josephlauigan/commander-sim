# Brago — Azorius Blink Control

Opponent pool deck — **Tier 2: Mid Bracket 3**. Axis: Control / interaction-heavy.

- **Commander:** Brago, King Eternal
- **Color identity:** White, Blue
- **Game Changers (1):** Farewell
- **Combos:** Peregrine Drake + Archaeomancer + Ghostly Flicker (unbounded mana; EDHREC's Deadeye Navigator, which made a two-card infinite with the Drake, is cut for Skyclave Apparition under the Tier 2 rule). It wins through Agent of Treachery, blinked until it has taken every permanent.
- **Lands:** 35 (15 nonbasic + 20 basic)

## Role in the pool

The pool's enters-the-battlefield engine: every creature and Aura does something as it enters, and a dozen cards make it enter again. Tests whether decks can answer a value engine before it buries them, and whether their own permanents survive Agent of Treachery, Riptide Gearhulk and Reality Acid.

## At the table

- **Plan:** Builds a board of enters creatures (Mulldrifter, Wall of Omens, Reflector Mage, Agent of Treachery, Riptide Gearhulk, Sun Titan) and blinks them: Brago, King Eternal when he connects, Soulherder and Teleportation Circle each end step, Restoration Angel, Flickerwisp, Yorion, and the flicker spells Ephemerate, Cloudshift, Momentary Blink, Ghostly Flicker and Eerie Interlude. Panharmonicon, Elesh Norn, Mother of Machines, Starfield Vocalist and Strionic Resonator make each enters trigger count twice.
- **Wins by:** Grinds the table out of permanents and cards, then attacks with fliers (Brago, Cloudblazer, Restoration Angel, Sun Titan). Peregrine Drake with Archaeomancer and Ghostly Flicker makes unbounded mana and blinks Agent of Treachery until it has everything.
- **Watch out for:** Agent of Treachery takes your best permanent, again each time it is blinked; Reality Acid on your best permanent kills it when the Acid leaves, and Brago blinks it. Elesh Norn stops your own enters triggers. Kill Brago, and answer Peregrine Drake before Archaeomancer arrives.
- **Interaction:** Counterspell, Negate, Swan Song, Dovin's Veto and An Offer You Can't Refuse. Swords to Plowshares, Path to Exile, Reflector Mage, Venser, Shaper Savant, Riptide Gearhulk, Flickerwisp, Reality Acid, Witch Enchanter, Loran of the Third Path, Agent of Treachery; Farewell, Supreme Verdict and Time Wipe. Ghostly Prison taxes attacks.

## Strategy

Ramp with rocks (Sol Ring, Signets, Talisman, Basalt Monolith, Mind Stone, Thought Vessel), land a value creature a turn, and blink it as often as possible. Brago's combat damage blinks every enters permanent (and Reality Acid, whose host dies); Soulherder and Teleportation Circle do it each end step. Time Wipe saves Brago (back to hand) and wipes the rest. The combos are a bonus: the deck plays the value game first.

## Sim modeling notes

Model these first; if any is Partial or unmodeled, this deck plays below its real strength and its results should be flagged.

ETB re-use through flicker (Brago, Soulherder, Teleportation Circle, Restoration Angel, Flickerwisp, Yorion, the flicker spells); trigger doubling (Panharmonicon, Elesh Norn, Mother of Machines, Starfield Vocalist) and copying (Strionic Resonator); steals (Agent of Treachery); Reality Acid's delayed removal; counterspell decisions. All of the list's cards are modeled (impls/t12_brago_meren.rs for the Tier 2 additions); left out: Starfield Vocalist's warp, Yorion's companion, lands for Ghostly Flicker and Agent of Treachery.

## Decklist by type (100)

**Commander (1).** Brago, King Eternal

**Creatures (29).** Aether Channeler, Agent of Treachery, Archaeomancer, Charming Prince, Cloud of Faeries, Cloudblazer, Displacer Kitten, Dour Port-Mage, Elesh Norn, Mother of Machines, Flickerwisp, Helpful Hunter, Loran of the Third Path, Mulldrifter, Peregrine Drake, Reflector Mage, Restoration Angel, Riptide Gearhulk, Skyclave Apparition, Solemn Simulacrum, Soulherder, Spirited Companion, Starfield Vocalist, Sun Titan, Tribute Mage, Venser, Shaper Savant, Wall of Omens, Whirler Rogue, Witch Enchanter // Witch-Blessed Meadow, Yorion, Sky Nomad

**Planeswalkers (1).** Venser, the Sojourner

**Enchantments (6).** Ghostly Prison, Mystic Remora, Omen of the Sea, Reality Acid, Teleportation Circle, Unquestioned Authority

**Artifacts (13).** Arcane Signet, Azorius Signet, Basalt Monolith, Cryogen Relic, Fellwar Stone, Lightning Greaves, Mind Stone, Panharmonicon, Sol Ring, Strionic Resonator, Swiftfoot Boots, Talisman of Progress, Thought Vessel

**Sorceries (3).** Farewell, Supreme Verdict, Time Wipe

**Instants (12).** An Offer You Can't Refuse, Cloudshift, Counterspell, Dovin's Veto, Eerie Interlude, Ephemerate, Ghostly Flicker, Momentary Blink, Negate, Path to Exile, Swan Song, Swords to Plowshares

**Lands (35).** Adarkar Wastes, Azorius Chancery, Command Tower, Deserted Beach, Flooded Strand, Glacial Fortress, Hallowed Fountain, Nimbus Maze, Port Town, Prairie Stream, Reliquary Tower, Rogue's Passage, Sea of Clouds, Skycloud Expanse, Temple of Enlightenment, 10 Island, 10 Plains

## Import list (100)

```
1 Adarkar Wastes
1 Aether Channeler
1 Agent of Treachery
1 An Offer You Can't Refuse
1 Arcane Signet
1 Archaeomancer
1 Azorius Chancery
1 Azorius Signet
1 Basalt Monolith
1 Brago, King Eternal
1 Charming Prince
1 Cloud of Faeries
1 Cloudblazer
1 Cloudshift
1 Command Tower
1 Counterspell
1 Cryogen Relic
1 Deserted Beach
1 Displacer Kitten
1 Dour Port-Mage
1 Dovin's Veto
1 Eerie Interlude
1 Elesh Norn, Mother of Machines
1 Ephemerate
1 Farewell
1 Fellwar Stone
1 Flickerwisp
1 Flooded Strand
1 Ghostly Flicker
1 Ghostly Prison
1 Glacial Fortress
1 Hallowed Fountain
1 Helpful Hunter
1 Lightning Greaves
1 Loran of the Third Path
1 Mind Stone
1 Momentary Blink
1 Mulldrifter
1 Mystic Remora
1 Negate
1 Nimbus Maze
1 Omen of the Sea
1 Panharmonicon
1 Path to Exile
1 Peregrine Drake
1 Port Town
1 Prairie Stream
1 Reality Acid
1 Reflector Mage
1 Reliquary Tower
1 Restoration Angel
1 Riptide Gearhulk
1 Rogue's Passage
1 Sea of Clouds
1 Skyclave Apparition
1 Skycloud Expanse
1 Sol Ring
1 Solemn Simulacrum
1 Soulherder
1 Spirited Companion
1 Starfield Vocalist
1 Strionic Resonator
1 Sun Titan
1 Supreme Verdict
1 Swan Song
1 Swiftfoot Boots
1 Swords to Plowshares
1 Talisman of Progress
1 Teleportation Circle
1 Temple of Enlightenment
1 Thought Vessel
1 Time Wipe
1 Tribute Mage
1 Unquestioned Authority
1 Venser, Shaper Savant
1 Venser, the Sojourner
1 Wall of Omens
1 Whirler Rogue
1 Witch Enchanter // Witch-Blessed Meadow
1 Yorion, Sky Nomad
10 Island
10 Plains
```
