# Veyran, Voice of Duality — Izzet Spellslinger

Chain cheap instants and sorceries, and let Veyran make every cast-trigger fire twice. The deck turns those doubled
triggers into damage, cards, Treasure and a stream of creature tokens, and ends the game with a big turn: the pingers
(Guttersnipe, Kessig Flamebreather, Firebrand Archer), Crackle with Power, Rite of the Dragoncaller, or a board of tokens. It is weak to creature decks that attack
before the engine is online, to board wipes while the tokens are out, and to a removal spell on Veyran before the
chain starts.

| | |
|---|---|
| **Commander** | Veyran, Voice of Duality ({1}{U}{R}, 2/2, blue-red) |
| **Bracket** | 3 (2 Game Changers: Consecrated Sphinx, Mystical Tutor) |
| **Simulator** | deck key `veyran`; results in `decklists/pool/pool-results.md`, history in `decklists/CHANGELOG.md` |

## Strategy

### The commander

Veyran ({1}{U}{R}, 2/2) reads: *Magecraft — whenever you cast or copy an instant or sorcery spell, Veyran gets +1/+1 until end of turn. If you casting or copying an instant or sorcery spell causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time.*

The scope matters. Veyran doubles magecraft and "whenever you cast an instant or sorcery" abilities. It does **not** double attack triggers, combat-damage triggers, ETB triggers, or draw triggers, so the Emeritus attack triggers and Niv-Mizzet, Parun's "whenever you draw" pings fire once per draw. Veyran has no first strike, and magecraft gives +1/+1 until end of turn, not permanent counters. The doubling applies only when your casting or copying an instant or sorcery is what caused the trigger.

Veyran is a fragile engine, and he is removed often. The token makers keep working at half strength without him, so the deck does not stop when he dies, but every engine below is twice as good with him out.

### Game plan

**Early turns (1–3).** Play a cheap engine: **Young Pyromancer** or **Third Path Iconoclast** on turn 2, **Guttersnipe**, **Kessig Flamebreather** or **Firebrand Archer** on turn 2 or 3, **Harmonic Prodigy**, and a rock (**Sol Ring**, **Arcane Signet**, **Fellwar Stone**). Veyran on turn 3 is fine if you can protect him; otherwise build the board first and let the cheap spells (cantrips, burn, counters) trigger the engines at half strength. Keep a counterspell or **Arcane Denial** up when you can.

**Middle turns (4–6).** Deploy the engines before the chain: **Storm-Kiln Artist**, **Birgi, God of Storytelling**, **Murmuring Mystic**, **Archmage Emeritus**, **Ral, Storm Conduit**, **Alania, Divergent Storm**, and **Veyran** himself if he is not out yet, with **Lightning Greaves** down first. Cast cheap spells at the end of the turn to build tokens, and keep **Propaganda** and a counter ready against creature decks.

**Late game.** The big turn: pingers or token makers down, Veyran protected, **Fiery Emancipation** out if you have it, then chain. **Thousand-Year Storm** copies each spell once for each earlier spell, **Seething Song** and **Treasure** from Storm-Kiln Artist pay for it, and the pingers, **Crackle with Power** or the token board end the game.

### Pingers

**Guttersnipe** and **Kessig Flamebreather** each hit every opponent when you cast, and Veyran doubles both. **Firebrand Archer** ({1}{R}) deals 1 to each opponent for every noncreature spell, twice for instants and sorceries with Veyran out; it is a Human, not a Shaman, so Harmonic Prodigy doesn't add to it. Ral, Storm Conduit adds another ping on every cast or copy. Guttersnipe and Flamebreather are Shamans, so **Harmonic Prodigy** adds a third trigger. **Thor, Asgard's Avenger** (a 4/4 flier) adds 1 to all the damage your other sources deal to opponents and their permanents, so each doubled ping hits harder.

### Spells make creatures

Five token makers trigger when you cast an instant or sorcery, and Veyran doubles each one:

- **Young Pyromancer** ({1}{R}): a 1/1 red Elemental. It's a Shaman, so Harmonic Prodigy adds a third token.
- **Third Path Iconoclast** ({U}{R}): a 1/1 Soldier artifact creature. It triggers on any noncreature spell, but only instants and sorceries are doubled.
- **Murmuring Mystic** ({3}{U}): a 1/1 flying Bird Illusion. It's a Wizard, so Harmonic Prodigy adds a third.
- **Docent of Perfection** ({3}{U}{U}, 5/4 flier): a 1/1 Wizard. Once you control three Wizards it transforms into **Final Iteration**, which gives your Wizards +2/+1 and flying, so the Wizard tokens become 3/2 fliers. The Wizards you already have (Murmuring Mystic, Archmage Emeritus, Flux Channeler, Alania, Harmonic Prodigy and the Emeritus creatures) count.
- **Rite of the Dragoncaller** (an enchantment) makes a 5/5 flying Dragon for each instant or sorcery, doubled by Veyran.

With one of these out, a four-spell turn leaves eight tokens. The token makers keep working when Veyran is gone, just at half strength. On turns you can't go off, cheap spells build a board you can attack with. Tokens are poor blockers in practice: they are often tapped or not yet on the board when the big attack comes, so don't count on them to protect Veyran.

### Treasure and mana

**Storm-Kiln Artist pays for the chain.** Its magecraft makes a Treasure whenever you cast or copy an instant or sorcery. Veyran doubles it, and it's a Shaman, so Harmonic Prodigy adds a third. It gets +1/+0 for each artifact you control; Treasures and Iconoclast's Soldiers both count. Because it's magecraft, it also triggers on copies from Thousand-Year Storm and Alania. **Sanar, Unfinished Genius // Wild Idea** also makes a Treasure when you tap it after casting an instant or sorcery, and its prepared spell tutors an instant or sorcery. **Blazing Firesinger // Seething Song** enters prepared and gives {R}{R}{R}{R}{R} mid-chain.

### Fiery Emancipation

**Fiery Emancipation triples everything.** If a source you control would deal damage, it deals triple instead. With Veyran out, a Guttersnipe trigger pair becomes 12 damage to each opponent per spell. Every 1/1 token attacks for 3, a Drake for 6, Firebrand Archer pings for 3 per trigger, and Crackle with Power hits three times as hard. It costs six ({3}{R}{R}{R}), so it usually lands the turn before the big turn. It is not a Game Changer.

### Cards into damage

**Niv-Mizzet, Parun turns cards into damage.** Parun ({U}{U}{U}{R}{R}{R}, 5/5 flier, can't be countered) draws a card whenever *any* player casts an instant or sorcery (your casts doubled by Veyran), and pings for every card you draw. **Harmonic Prodigy doubles both** (Parun is a Wizard). **Archmage Emeritus** draws on every magecraft trigger, so one spell with Parun, Emeritus, and Veyran out can draw four and ping four.

### Copy engines

**Thousand-Year Storm** copies each instant or sorcery once for every other one you've cast before it this turn. Copies trigger magecraft (and Veyran doubles those) but aren't casts, so they don't trigger Guttersnipe, Kessig, Parun, or the token makers. Storm-Kiln Artist's magecraft does see them. The fourth spell of a turn becomes four spells' worth of magecraft.

**Alania, Divergent Storm copies the first instant and the first sorcery you cast each turn** (yours and opponents' turns alike), if you have a target opponent draw a card for each copy. Each copy is optional, so take only the ones worth a card. Casting the spell is what triggers Alania, so **Veyran makes it trigger twice**, and Alania is a Wizard, so **Harmonic Prodigy adds a third**: with both out, your first instant and first sorcery each resolve up to four times. Like Thousand-Year Storm's, the copies trigger magecraft (Archmage Emeritus draws, Veyran grows) but aren't casts, so Guttersnipe, Kessig, Parun and the token makers see only the original, while Storm-Kiln Artist makes a Treasure for each copy. The best first spells: Ancestral Recall, Expressive Iteration, Seething Song (each copy is another five red), Crackle with Power (copies keep X), and Lightning Bolt. A copied counterspell is rarely worth a card. Spells cast before Alania landed still count: if your first instant this turn came before Alania, the next one isn't copied.

**Mizzix's Mastery** copies an instant or sorcery from your graveyard and casts it free (so Veyran sees a cast): the single-card blowout on River's Rebuke or a large Crackle with Power. **Flashback** gives an instant or sorcery in your graveyard flashback for its mana cost.

### Protecting Veyran

- **Lightning Greaves** gives shroud and haste, and its equip is {0}, so you can move it to a new threat for free.
- **Triple Triad** ({3}{R}{R}{R}) gives a free card every upkeep: each player exiles their top card, and you may play yours and every exiled card with lower mana value free. Free instants and sorceries are still cast, so they trigger Veyran and the magecraft cards.
- **Arcane Denial** ({1}{U}) counters any spell; its controller may draw up to two cards at their next upkeep and you draw one. A cheap answer to the removal spell aimed at Veyran, the wipe, or the counter aimed at your big turn; the cards it gives away matter less than the spell it stops.
- **Old Fat Spider Can't See Me** (a Saga) gives a creature of yours hexproof while it stays (chapter I), later prevents a creature's damage, and draws two cards on chapters III and IV.
- **Unsummon** and **Banishing Betrayal** can save your own creature from removal by returning it to your hand, and **Counterspell**, **An Offer You Can't Refuse** and **Mystic Confluence** stop the removal itself.

### Protecting yourself from creature decks

**Propaganda taxes the attacks on you.** Each creature that attacks you costs its controller {2}. The deck's losses come mostly from decks full of creatures, and the tokens rarely protect you because they aren't on the board when the big attack comes. Propaganda works from the turn it lands and doesn't affect your own attacks. Cast it early when a creature deck is across the table. It's an enchantment, so no tutor in the deck finds it. **Blasphemous Act** (13 damage to each creature) and **Splatter Technique** (4 damage to each creature and planeswalker) reset the board; **River's Rebuke** returns all of one player's nonland permanents.

### Interaction and burn

- **Four ways to stop a spell:** Counterspell, An Offer You Can't Refuse, Arcane Denial and Mystic Confluence (counter unless its controller pays {3}, bounce a creature, or draw: choose three).
- **Spot removal:** Abrade (3 damage to a creature, or destroy an artifact), Burst Lightning, Lightning Bolt, Lightning Strike, Vibrant Outburst (3 damage and a tap), Chaos Warp, Banishing Betrayal and Prismari Charm.
- **Unsummon**: a one-mana bounce that also saves one of your own creatures.

Every one is also a magecraft trigger, so expect to hold up mana less often and win more by building a board.

### Other cards

- **Eris, Roar of the Storm** is the reward for a full graveyard: it costs {2} less per *different* mana value among instants and sorceries there. It is a 4/4 flier with prowess, and whenever you cast your second spell each turn it makes a 4/4 flying Dragon Elemental. 
- **Consecrated Sphinx** ({4}{U}{U}, 4/6 flier) draws you two cards whenever an opponent draws a card, so every opponent draw step is a free refill, and each of those cards is a draw for Niv-Mizzet, Parun to ping. It is a body to protect, not a combo piece, and its draw trigger is not an instant or sorcery cast, so Veyran does not double it.
- **Flux Channeler** is a small Wizard with a proliferate trigger: whenever you cast a noncreature spell, it proliferates (add another counter of each kind already on permanents and players you choose, such as Ral's loyalty). It counts toward Docent's three Wizards and is a cheap body.
- **Imperial Recruiter**, **Solve the Equation**, **Fabricate** and **Mystical Tutor** are the tutors (see the section below).
- **Cantrips** keep the chain going and fill the graveyard: Think Twice, Deduce, Quick Study, Dreams of Laguna, Light Up the Stage, Expressive Iteration, Flow State, Stock Up, Plunder the Trollshaws and Frantic Search (draw two, discard two, untap three lands: free, and it fills the graveyard for Mizzix's Mastery and Flashback).
- **Emeritus of Ideation // Ancestral Recall** and **Emeritus of Conflict // Lightning Bolt** are creatures with a prepared spell on the back (see Mana and consistency).

### How it wins

- **The big turn:** with a pinger or two, a token maker and Veyran out, a chain of four to six spells deals most of the damage by itself. Fiery Emancipation triples it.
- **The pingers** (Guttersnipe, Kessig Flamebreather, Firebrand Archer, Ral) turn a long turn, especially one extended by Thousand-Year Storm copies of cantrips, into damage to every opponent.
- **Rite of the Dragoncaller** makes a 5/5 flier per instant or sorcery, doubled by Veyran.
- **Crackle with Power** deals five times X damage to each of up to X targets, and **Mizzix's Mastery** can recast it free from the graveyard.
- **Going wide:** without a big turn, cast cheap spells to build a token board and attack. Under Fiery Emancipation every 1/1 hits for 3.

### Weak spots and how to play around them

- **Creature decks.** The deck loses most to decks that attack with many creatures. Land Propaganda early, hold Blasphemous Act, and keep the cheap interaction for the biggest attacker.
- **Board wipes.** The tokens die to them. Rebuild with the next few spells, and keep Arcane Denial up for the wipe when you can.
- **Veyran removed.** Put Lightning Greaves down before or with Veyran, not after he has already eaten removal, and keep the engines that don't need him in play.
- **Graveyard hate.** Eris, Mizzix's Mastery, Emeritus of Ideation and Flashback all want the graveyard. If an opponent exiles it, lean on the pingers, the token makers and the cards in hand instead.
- **Slow or clunky hands.** The list is top-heavy and has only three rocks. Keep hands with lands and a cheap engine (a pinger, Young Pyromancer, Third Path Iconoclast), and don't keep a hand of six-drops.

## Key lines

**The big turn.** Pingers or token makers down, Veyran protected, Fiery Emancipation out if you have it, then chain. Thousand-Year Storm plus a mid-turn Seething Song (Blazing Firesinger's prepared spell), or Mizzix's Mastery on a full yard is the most explosive line the deck has.

**The pingers** turn a long turn into damage to every opponent: with Veyran out, Guttersnipe, Kessig Flamebreather and Firebrand Archer each trigger twice per instant or sorcery you cast.

**Rite of the Dragoncaller** makes a 5/5 flier per instant or sorcery, doubled by Veyran.

**Going wide.** Without a big turn available, cast cheap spells to build a token board and attack. Under Fiery Emancipation every 1/1 hits for 3. Board wipes are the risk: rebuild with the next few spells, and keep Arcane Denial up for the wipe when you can.

**Mizzix's Mastery** on River's Rebuke or a large Crackle with Power is the single-card blowout.

**Deck-out warning.** Niv-Mizzet, Parun's draw is not optional, and it triggers off opponents' spells too. With Parun, Archmage Emeritus, Veyran, and Harmonic Prodigy all out, a long turn can draw a lot of cards. Watch your library.

**Sequencing.** Deploy pingers, token makers and Storm-Kiln Artist before Veyran and before the chain, not in the middle of it. Put Lightning Greaves down before or with Veyran, not after he has already eaten removal. Ral and Birgi both want to be down before the spell-dense turn. Emeritus of Ideation wants to exile eight cards from your graveyard on attack, which competes with Eris and Mizzix's Mastery for the same yard, so feed it lands and spent cantrips.

## Tutor targets by board state

Four tutors, each pointing somewhere different. **Solve the Equation only finds instants and sorceries**, **Fabricate finds artifacts**, **Imperial Recruiter finds a creature with power 2 or less**, and **Mystical Tutor finds an instant or sorcery** for {U} (it goes on top of the library). (Mizzix's Mastery is the graveyard rebuy.) Sanar's prepared spell, Wild Idea, is a fifth, for an instant or sorcery.

- **Need a damage engine → Imperial Recruiter for Guttersnipe, Kessig Flamebreather or Firebrand Archer.** Or Harmonic Prodigy when a Niv-Mizzet or a Shaman engine is already out.
- **Need a board → Imperial Recruiter for Young Pyromancer or Third Path Iconoclast.**
- **Need mana for the chain → Imperial Recruiter for Storm-Kiln Artist.**
- **Need more cards → Imperial Recruiter for Archmage Emeritus.**
- **Veyran keeps dying → Fabricate for Lightning Greaves**, or Solve the Equation for Arcane Denial.
- **A creature deck is attacking you → cast Propaganda early.** It's an enchantment, so no tutor here finds it.
- **Veyran needs protection → Fabricate for Lightning Greaves** before the long turn.
- **Need to protect a big turn → Solve the Equation for a counter** (or Mystical Tutor for it, a turn slower, at one mana).
- **Need reach to close → Solve the Equation for Crackle with Power**, or **Mizzix's Mastery** to recast River's Rebuke or a big Crackle from the yard.
- **Mana-screwed → Fabricate for Sol Ring or Fellwar Stone.**

Timing note: **Solve the Equation puts the card into your hand**, so you can cast it the same turn. **Mystical Tutor** puts the instant or sorcery on top of your library instead, so you draw it next (a cantrip or Archmage Emeritus can draw it the same turn).

## Mana and consistency

The list is exactly 100 cards.

34 lands (15 nonbasic, 11 Island, 8 Mountain) plus three rocks: Sol Ring, Arcane Signet, Fellwar Stone. Storm-Kiln Artist's Treasures add mana once spells start. **Birgi, God of Storytelling** adds {R} for every spell you cast, twice for instants and sorceries with Veyran out, and the mana lasts the whole turn, so one-mana spells net mana mid-chain. (Lightning Greaves is an artifact but not a mana source.)

| Measure | Count | Note |
| --- | --- | --- |
| Blue sources | 21 | 11 Island plus Command Tower, Exotic Orchard, Coastal Peak, Mistrise Village, Mystic Sanctuary, Path of Ancestry, Scorched Geyser, Shivan Reef, Spectacle Summit, Temple of Epiphany |
| Red sources | 17 | 8 Mountain plus Command Tower, Exotic Orchard, Coastal Peak, Path of Ancestry, Scorched Geyser, Shivan Reef, Spectacle Summit, Sokenzan, Temple of Epiphany |
| Colorless-only lands | 1 | Desolate Lighthouse (a looter: {1}{U}{R}, {T}: draw, then discard) |
| No mana the turn they land | 3 | Ash Barrens, Evolving Wilds, Terramorphic Expanse (they find a basic land) |
| Always enter tapped | 4 | Coastal Peak, Temple of Epiphany, Path of Ancestry, Spectacle Summit |
| Sometimes enter tapped | 3 | Scorched Geyser, Mistrise Village, Mystic Sanctuary |

**Curve pressure.** Niv-Mizzet, Parun, Thousand-Year Storm, Fiery Emancipation, Triple Triad, Consecrated Sphinx and Rite of the Dragoncaller cost six (Parun is {U}{U}{U}{R}{R}{R}), and Alania, Docent of Perfection, Emeritus of Ideation, Mystic Confluence and Splatter Technique cost five. Parun's triple-pip cost in both colours is the hardest cast in the list. The cheap engines are Young Pyromancer, Third Path Iconoclast, Guttersnipe, Kessig Flamebreather and Harmonic Prodigy (two or three mana each), and the deck has only three rocks, so a cheap body or a rock matters more than another six-drop.

**Spell density.** 32 instants and sorceries (21 instants, 11 sorceries), plus four prepare cards whose backs are castable spells: Emeritus of Ideation // Ancestral Recall, Emeritus of Conflict // Lightning Bolt, Blazing Firesinger // Seething Song, Sanar // Wild Idea.

**Finding cards.** Archmage Emeritus and Niv-Mizzet, Parun are repeatable draw; Mystic Confluence (draw three at an opponent's end step) and Splatter Technique (draw four) refill, and Alania's copies of a cantrip draw again. Cantrips: Think Twice, Deduce, Quick Study, Dreams of Laguna, Light Up the Stage, Expressive Iteration, Flow State, Stock Up, Plunder the Trollshaws, and **Frantic Search** (draw two, discard two, untap three lands). Desolate Lighthouse and Mystic Sanctuary (puts a spell from the graveyard on top of the library) also help.

**Interaction.** Four ways to stop a spell, eight pieces of spot removal, Unsummon, and three resets: Blasphemous Act, River's Rebuke and Splatter Technique.

**Redundancy.** Four recurring damage-on-cast sources (Guttersnipe, Kessig Flamebreather, Firebrand Archer, Ral), a draw-to-damage Niv-Mizzet, and four token makers. Losing Veyran or any one engine doesn't turn the deck off.

## Bracket and Rule 0

Bracket 3 with two Game Changers: **Consecrated Sphinx** and **Mystical Tutor**. Up to three Game Changers fit Bracket 3; a fourth moves the deck to Bracket 4. Flux Channeler, Imperial Recruiter, Thousand-Year Storm, Niv-Mizzet, Parun, Ral, Storm Conduit, Birgi, Fiery Emancipation and Propaganda are all off-list.

**No two-card combo.** The deck produces large, non-infinite turns.

Disclose before the game:

- **Consecrated Sphinx** and **Mystical Tutor** (the Game Changers).
- **Birgi, God of Storytelling**, which makes two red mana per instant or sorcery with Veyran out.
- **Thousand-Year Storm** and the pinger suite, which can turn one turn into a lot of damage to each opponent.
- **Fiery Emancipation**, which triples all of your damage (a Guttersnipe with Veyran out deals 12 to each opponent per spell).

Re-check the official list at https://commanderbrackets.com/faq before an event.

## Decklist by type (100)

**Commander (1).** Veyran, Voice of Duality

**Planeswalkers (1).** Ral Storm Conduit

**Creatures (22).** Alania Divergent Storm, Archmage Emeritus, Birgi God of Storytelling, Blazing Firesinger // Seething Song, Consecrated Sphinx, Docent of Perfection // Final Iteration, Emeritus of Conflict // Lightning Bolt, Emeritus of Ideation // Ancestral Recall, Eris Roar of the Storm, Firebrand Archer, Flux Channeler, Guttersnipe, Harmonic Prodigy, Imperial Recruiter, Kessig Flamebreather, Murmuring Mystic, Niv-Mizzet Parun, Sanar Unfinished Genius // Wild Idea, Storm-Kiln Artist, Third Path Iconoclast, Thor Asgard's Avenger, Young Pyromancer

**Artifacts (4).** Arcane Signet, Fellwar Stone, Lightning Greaves, Sol Ring

**Enchantments (6).** Fiery Emancipation, Old Fat Spider Can't See Me, Propaganda, Rite of the Dragoncaller, Thousand-Year Storm, Triple Triad

**Instants (21).** Abrade, An Offer You Can't Refuse, Arcane Denial, Banishing Betrayal, Burst Lightning, Chaos Warp, Counterspell, Deduce, Dreams of Laguna, Flashback, Frantic Search, Lightning Bolt, Lightning Strike, Mystic Confluence, Mystical Tutor, Plunder the Trollshaws, Prismari Charm, Quick Study, Think Twice, Unsummon, Vibrant Outburst

**Sorceries (11).** Blasphemous Act, Crackle with Power, Expressive Iteration, Fabricate, Flow State, Light Up the Stage, Mizzix's Mastery, River's Rebuke, Solve the Equation, Splatter Technique, Stock Up

**Lands (34).** Ash Barrens, Coastal Peak, Command Tower, Desolate Lighthouse, Evolving Wilds, Exotic Orchard, Mistrise Village, Mystic Sanctuary, Path of Ancestry, Scorched Geyser, Shivan Reef, Sokenzan Crucible of Defiance, Spectacle Summit, Temple of Epiphany, Terramorphic Expanse, 11 Island, 8 Mountain

## Import list (100)

```
1 Abrade
1 Alania, Divergent Storm
1 An Offer You Can't Refuse
1 Arcane Denial
1 Arcane Signet
1 Archmage Emeritus
1 Ash Barrens
1 Banishing Betrayal
1 Birgi, God of Storytelling
1 Blasphemous Act
1 Blazing Firesinger // Seething Song
1 Burst Lightning
1 Chaos Warp
1 Coastal Peak
1 Command Tower
1 Consecrated Sphinx
1 Counterspell
1 Crackle with Power
1 Deduce
1 Desolate Lighthouse
1 Docent of Perfection // Final Iteration
1 Dreams of Laguna
1 Emeritus of Conflict // Lightning Bolt
1 Emeritus of Ideation // Ancestral Recall
1 Eris, Roar of the Storm
1 Evolving Wilds
1 Exotic Orchard
1 Expressive Iteration
1 Fabricate
1 Fellwar Stone
1 Fiery Emancipation
1 Firebrand Archer
1 Flashback
1 Flow State
1 Flux Channeler
1 Frantic Search
1 Guttersnipe
1 Harmonic Prodigy
1 Imperial Recruiter
1 Kessig Flamebreather
1 Light Up the Stage
1 Lightning Bolt
1 Lightning Greaves
1 Lightning Strike
1 Mistrise Village
1 Mizzix's Mastery
1 Murmuring Mystic
1 Mystic Confluence
1 Mystic Sanctuary
1 Mystical Tutor
1 Niv-Mizzet, Parun
1 Old Fat Spider Can't See Me
1 Path of Ancestry
1 Plunder the Trollshaws
1 Prismari Charm
1 Propaganda
1 Quick Study
1 Ral, Storm Conduit
1 Rite of the Dragoncaller
1 River's Rebuke
1 Sanar, Unfinished Genius // Wild Idea
1 Scorched Geyser
1 Shivan Reef
1 Sokenzan, Crucible of Defiance
1 Sol Ring
1 Solve the Equation
1 Spectacle Summit
1 Splatter Technique
1 Stock Up
1 Storm-Kiln Artist
1 Temple of Epiphany
1 Terramorphic Expanse
1 Think Twice
1 Third Path Iconoclast
1 Thor, Asgard's Avenger
1 Thousand-Year Storm
1 Triple Triad
1 Unsummon
1 Veyran, Voice of Duality
1 Vibrant Outburst
1 Young Pyromancer
11 Island
8 Mountain
```
