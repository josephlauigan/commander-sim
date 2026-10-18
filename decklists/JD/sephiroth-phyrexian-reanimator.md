# Sephiroth, the Savior — Phyrexian Reanimator

Fill the graveyard, cheat a Phyrexian bomb onto the battlefield three or four turns before anyone can cast one, and drain the table through sacrifice loops while the Praetors make the board unplayable. When the pieces line up, an infinite loop ends the game on the spot. The deck is weak to graveyard hate and to a fast combo deck that goes first.

Sephiroth, the Savior is the Final Fantasy printing of Atraxa, Grand Unifier, and appears under that name in the import list. (This is the Seph/Atraxa deck, key `seph`, not the Sephiroth FS deck.)

| | |
|---|---|
| **Commander** | Sephiroth, the Savior / Atraxa, Grand Unifier ({3}{G}{W}{U}{B}, 7/7, white-blue-black-green) |
| **Bracket** | 4 (3 Game Changers: Demonic Tutor, Gifts Ungiven, Aura Shards; the bracket comes from the four tutorable infinite loops, not the count) |
| **Simulator** | deck key `seph`; results in `decklists/pool/pool-results.md`, history in `decklists/CHANGELOG.md` |

## Strategy

### The commander

Atraxa, Grand Unifier: *Flying, vigilance, deathtouch, lifelink. When Atraxa enters, reveal the top ten cards of your library. For each card type, you may put a card of that type from among the revealed cards into your hand. Put the rest on the bottom of your library in a random order.* Sephiroth, the Savior plays the same way.

She is a 7/7 that can be cast for seven, and one entry is a whole hand: up to one creature, one land, one artifact, one enchantment, one instant, one sorcery and one planeswalker from the top ten (the deck has no planeswalkers or battles, so expect up to six cards). She is the best reanimation target in the deck, because the enter trigger comes with a 7/7 flying lifelink body. Blink her (Ephemerate, Teleportation Circle, Conjurer's Closet, Displacer Kitten) and she digs ten cards deeper. She is a four-colour card, so don't hard-cast her unless the mana is easy; reanimation is the main route. The cards she puts on the bottom are gone for the game, so don't bring her in with a library close to empty.

### Game plan

**Turns 1 to 3: mana and yard.** Sol Ring, Arcane Signet, Talisman of Hierarchy, Mind Stone, Chromatic Lantern, Birds of Paradise and Sakura-Tribe Elder for mana. **Tireless Tracker** ({2}{G}) turns every land drop into a Clue, and Sakura-Tribe Elder, Evolving Wilds and Fabled Passage add extra land drops, so it draws cards on its own. Stitcher's Supplier, Buried Alive, and Entomb load the graveyard. Tortured Existence keeps it filling every turn. **Gifts Ungiven** is a yard-filler and tutor in one: name four cards you're happy to have in either zone (for example four reanimation targets, or Mikaeus and Triskelion plus two reanimation spells), and whatever the table puts in your graveyard is exactly what you wanted there.

**Turns 3 to 5: cheat something out.** Reanimate, Animate Dead, Necromancy, Unburial Rites, and Dread Return point at Archon of Cruelty, Grave Titan, Hornet Queen, Sheoldred, or Sephiroth herself. **Hornet Queen** ({4}{G}{G}{G}, flying, deathtouch) enters with four 1/1 flying deathtouch Insects, a wall of blockers that also stops fliers, and every blink of her makes four more. Sheoldred, Whispering One then returns a creature every upkeep for free. **Dread Return** can be flashed back later by sacrificing three creatures, which is free with Zombie tokens or loop pieces and triggers Blood Artist and Zulaport Cutthroat three times.

**Late game.** Hold up Arcane Denial, Swan Song, Ephemerate and Galadriel's Dismissal, keep Austere Command as the reset, assemble a loop, and use the grind engines (Muldrotha, Sheoldred, Whispering One, the end-step flickers) to keep the pressure on if the first plan is answered. Sheoldred's, Archon's and Elesh Norn's drains and edicts keep the table small while you do it.

### Fill the yard

Stitcher's Supplier mills three when it enters and again when it dies, so it is a good first-turn play and a good sacrifice. Entomb puts any card in the graveyard for {B}, Buried Alive puts up to three creature cards there, and Tortured Existence ({B}, discard a creature card: return a creature card from the graveyard to your hand) turns an uncastable fatty in hand into the creature you need. **Skullclamp** on Stitcher's Supplier or a Zombie token kills it at once (the Clamp gives +1/-1) and draws two; with Supplier it also mills three more. Demonic Tutor and Diabolic Tutor are there to find the missing piece; see the tutor section below.

### Reanimation

Five reanimation spells: **Reanimate** ({B}, lose life equal to the card's mana value, and it can take a creature from any graveyard), **Animate Dead** and **Necromancy** (enchantments that bring a creature back and go with it; Necromancy can be cast at instant speed, but then it is sacrificed at the next cleanup step), **Unburial Rites** (flashback {3}{W}) and **Dread Return** (flashback: sacrifice three creatures). Every one of them can take a legendary creature, so any spell reaches any bomb. **Yawgmoth's Will** replays the graveyard for a turn: use it for a big turn once the yard is deep. Everything that would go to the graveyard that turn is exiled instead, so don't try the Nim Deathmantle loop under it. Sheoldred, Whispering One, Muldrotha and Grave Pact (below) are the other half of the reanimation plan.

### The card-advantage layer

**Phyrexian Arena** adds a card every upkeep for 1 life, **Harmonize** ({2}{G}{G}) draws three, and **Tireless Tracker**'s Clues draw one each for {2} (the Tracker grows with every Clue sacrificed). **Sheoldred, the Apocalypse** gains you 2 life for every card you draw and makes each opponent lose 2 whenever they draw, so with her out the Arena's draws gain more life than they cost, and Harmonize is worth 6 life.

**Displacer Kitten** blinks a nonland permanent every time you cast a noncreature spell. Blink Sephiroth for another top-ten dig, Archon of Cruelty for another drain-and-edict, Kitchen Finks for 2 more life, or Grave Titan for two more Zombies.

### Muldrotha turns the graveyard into a second hand

During each of your turns you may play a land and cast one permanent spell of each permanent type from your graveyard: a creature, an artifact, and an enchantment. In this deck that means:
- **A reanimation spell every turn.** Animate Dead and Necromancy are enchantments, so once one is in the graveyard Muldrotha recasts it each turn for another reanimation.
- **Loop pieces back after removal.** Triskelion (as the artifact or the creature), Kitchen Finks, Nim Deathmantle, Ashnod's Altar, or Viscera Seer, so one answer no longer breaks a combo.
- **Engines back after a wipe.** Aura Shards, Phyrexian Arena, or Skullclamp.
- **A land from the graveyard each turn.** Replay Evolving Wilds or Fabled Passage for a basic every turn, or Strip Mine for land destruction every turn (say so before the game).

She is a 6/6 for six and a fine reanimation target herself.

### Protection and blink

**Ephemerate is protection and a second Kitten in one card.** For {W} at instant speed, exile your creature and return it. In response to a removal spell, theft, or any other targeted effect, the spell fizzles because the creature is a new object, so a reanimated bomb survives the first answer the table throws at it. With nothing to protect, blink Archon of Cruelty, Grave Titan, or Sephiroth for their enter triggers. Either way, rebound casts it again for free at your next upkeep: a second drain-and-edict from Archon or two more Zombies from Titan. It can only target creatures, so it can't save Aura Shards. **Momentary Blink** is a second one for the Kitten: {1}{W} at instant speed (or flashback for {3}{U}), so a blink is available again after the first has been used.

**The flicker package re-buys enter triggers every turn.** **Teleportation Circle** and **Conjurer's Closet** each exile a creature you control at the beginning of your end step and return it: Sephiroth for another top-ten dig, Archon of Cruelty for another drain-and-edict, Grave Titan for two more Zombies. Teleportation Circle can take an artifact instead, so with no creature worth it, flicker a tapped Sol Ring or Signet and it comes back untapped for the opponents' turns. **Restoration Angel** is a 3/4 flash flier that does the same once at instant speed: at the end of an opponent's turn for value, or in response to removal or theft aimed at a bomb (the spell fizzles, as with Ephemerate). It can't flicker an Angel, so not Sephiroth herself. **Flickering Hound** does it on your turn instead: whenever you cast a creature spell (Muldrotha's recasts count), it flickers another creature you control, Sephiroth included. Don't flicker a creature carrying +1/+1 counters or Equipment unless its enter trigger is worth more than what it loses.

**Arcane Denial** ({1}{U}) counters any spell. Its controller may draw up to two cards at the next upkeep and you draw one, so save it for the spell that matters (a wipe, Rest in Peace, a counter aimed at your combo). **Swan Song** ({U}) counters an enchantment, instant or sorcery (it gives that player a 2/2 flying Bird, so it is best used on a wipe or a tutor). **Galadriel's Dismissal** ({W}) phases out one creature: protection from removal for the turn. Kicked, it phases out every creature a target player controls, which stops an attack or a one-sided fight for a turn. **Grand Abolisher** ({W}{W}) stops opponents casting spells or using creature, artifact and enchantment abilities during your turn.

### The stax layer is what makes it unfair

Elesh Norn, Grand Cenobite shrinks every opposing creature by 2/2 and grows yours. Elesh Norn, Mother of Machines doubles your own enter triggers and blanks theirs. Sheoldred, the Apocalypse taxes their draw steps; Sheoldred, Whispering One makes each opponent sacrifice a creature every upkeep. **Aura Shards** destroys an artifact or enchantment every time a creature enters under your control, so every reanimation, Grave Titan Zombie, and Kitten blink is removal. **Grave Pact** ({1}{B}{B}{B}): whenever a creature you control dies, each other player sacrifices a creature. Every free sacrifice, from a Zombie token to Stitcher's Supplier to a Kitchen Finks that persists back, is an edict on the whole table. It wants the Swamps and black duals.

### Removal

Swords to Plowshares, Path to Exile, Anguished Unmaking ({1}{W}{B}, exile any nonland permanent, you lose 3) and Assassin's Trophy ({B}{G}, destroy any permanent an opponent controls) handle one problem each. **Summon: Bahamut** is a 9/9 flying Saga creature for {9}, and a reanimation target as much as a spell. Its chapters I and II each destroy a nonland permanent, III draws two, and IV, **Mega Flare**, deals damage equal to the total mana value of your other permanents to each opponent before it's sacrificed. Flickering it starts it over at chapter I with two more destroys. **Karn's Bastion** ({4}, {T}: proliferate) moves Bahamut's lore toward Mega Flare and grows +1/+1 counters, such as Triskelion's. **Toxic Deluge** ({2}{B}, pay X life) and **Austere Command** ({4}{W}{W}) are the resets. Austere Command is "choose two": destroy all artifacts, all enchantments, all creatures with mana value 3 or less, or all creatures with mana value 4 or greater. Pick the two that hit the table and spare your own pieces (for example, big creatures and artifacts when your board is small creatures and enchantments), and reanimation rebuilds afterwards. Bojuka Bog exiles a graveyard when it enters.

### Mana, lands and utility

Chromatic Lantern lets every land tap for any colour. Exotic Orchard, Command Tower and Ash Barrens (basic landcycling) fix colours; Evolving Wilds and Fabled Passage fetch basics, and Muldrotha can replay them. Strip Mine is optional land destruction. Treno, Dark City is a tapped blue-black Town. Blood Artist and Zulaport Cutthroat drain whenever creatures die, so Dread Return's flashback, Grave Pact, Skullclamp and the loops all become life loss for the table. Viscera Seer is a one-mana free sacrifice outlet. Mikaeus, Kitchen Finks, Melira, Triskelion, Nim Deathmantle, Ashnod's Altar, Altar of Dementia and Grave Titan are the combo pieces below. Archon of Cruelty, Hornet Queen and Sheoldred, Whispering One are bombs to reanimate rather than hard-cast.

### How it wins

Four infinite loops (see Key lines), plus fair lines when the loops are answered:
- Archon of Cruelty rebought every turn (Sheoldred, Whispering One, or Muldrotha recasting Animate Dead).
- Altar of Dementia milling one player out with a big creature.
- Elesh Norn, Grand Cenobite's -2/-2 is a one-sided board shrink. Sheoldred, the Apocalypse and the Arena's draws drain the table slowly.
- Aura Shards and Grave Pact grind down their board and their hand of permanents.

### Weak spots and how to play around them

- **Graveyard and sacrifice hate.** Rest in Peace stops every loop. Counter it with Arcane Denial or Swan Song, or remove it with Aura Shards (once a creature of yours enters), Austere Command, Anguished Unmaking or Assassin's Trophy.
- **Torpor Orb and similar** shut off enter-the-battlefield payoffs. Remove them with Aura Shards, Anguished Unmaking or Assassin's Trophy first.
- **Exile-based removal and theft** on a reanimated bomb: keep Ephemerate, Restoration Angel or Galadriel's Dismissal up for it.
- **Wipes**: Arcane Denial, Ephemerate on the key creature, or hold the reanimation spell until after the wipe.
- **Fast combo decks.** The deck is slower than the very fastest decks; use the cheap counters (Arcane Denial, Swan Song) and the cheap tutors to win first or stop them.
- **Colour screw.** The commander costs four colours and the combo pieces need black, white and green. Count coloured sources for what you are about to cast and use Chromatic Lantern, Birds of Paradise and Exotic Orchard.

## Key lines

The deck has **four infinite loops**. The two Kitchen Finks loops need a free sacrifice outlet: **Viscera Seer**, **Ashnod's Altar**, or **Altar of Dementia**. Mikaeus + Triskelion needs only the two cards.

| # | Pieces | Result | Kills alone? |
| --- | --- | --- | --- |
| 1 | Mikaeus + Triskelion | Infinite damage | Yes |
| 2 | Mikaeus + Kitchen Finks + free sac outlet | Infinite life, ETBs, and deaths | Needs a payoff |
| 3 | Melira + Kitchen Finks + free sac outlet | Infinite life, ETBs, and deaths | Needs a payoff |
| 4 | Nim Deathmantle + Ashnod's Altar + Grave Titan | Infinite colourless mana, ETBs, and deaths | Needs a payoff |

**Mikaeus + Triskelion.** Mikaeus gives Triskelion undying and +1/+1, so with no counters it's a 2/2. When undying returns it with four counters (a 6/6), ping itself twice and opponents twice: the damage stays marked, and once the last counter is gone it's a 2/2 with 2 damage, so it dies with no counters and comes back again. That's 2 damage to opponents per loop, with no outlet needed. (The first time, with three counters, it's two pings at itself and one at an opponent.) **Elesh Norn, Grand Cenobite** breaks this version: her +2/+2 makes it a 4/4 with no counters, so all four pings have to go to itself. With her out, or Nim Deathmantle on Triskelion, sacrifice it to a free outlet after removing the counters instead: 4 damage per loop.

**Mikaeus + Kitchen Finks.** Persist and undying alternate: each return gives Finks one kind of counter, which leaves the other ability free on the next death. Every loop gains 2 life.

**Melira + Kitchen Finks.** Melira stops the -1/-1 counter, so persist always works. Same result as with Mikaeus, and it keeps the Finks loop alive if Mikaeus is answered.

**Nim Deathmantle + Ashnod's Altar + Grave Titan.** Sacrificing Titan gives {C}{C}, Deathmantle's {4} returns it, and its two Zombies sacrifice for {C}{C}{C}{C}. That's net +{2} per loop; the first time round you need two more mana (or sacrifice Zombies first). With Elesh Norn, Mother of Machines each return makes four Zombies.

**Payoffs that turn loops 2–4 into kills:**
- **Blood Artist or Zulaport Cutthroat:** each death drains the table.
- **Altar of Dementia as the outlet:** Finks mills a player every loop, so loops 2 and 3 win without a drain creature; opponents lose on their next draw. The Deathmantle loop can use it too, sacrificing the Zombies to mill.
- **Aura Shards:** every Finks or Titan return and every Zombie destroys an artifact or enchantment, which clears the table's.
- **Triskelion with loop 4:** the infinite mana keeps paying Deathmantle to return Triskelion for three more pings.
- **Grave Pact:** every creature of yours that dies makes each opponent sacrifice a creature, so any loop also empties their boards.

**Shared weak points.** Mikaeus is in two loops and Finks is in two, so Melira keeps Finks alive as a combo if Mikaeus is answered, and Triskelion covers you if Finks is. Graveyard and sacrifice hate hurts all four, since every loop relies on creatures dying and returning: Rest in Peace stops them outright, and a Torpor Orb-style card (no ETB triggers) shuts off the payoffs.

**Protect the combo turn.** Resolve **Grand Abolisher** first: opponents can't cast spells or activate abilities of artifacts, creatures, or enchantments during your turn. **Arcane Denial**, **Swan Song**, **Galadriel's Dismissal**, and **Ephemerate** cover what Abolisher doesn't, and Ephemerate also works on opponents' turns, when Abolisher does nothing.

**Openings.**
- Turn 1: Stitcher's Supplier, Entomb, Birds of Paradise, or Sol Ring. Turn 2: Sol Ring plus a Talisman or Signet puts Reanimate or Animate Dead in range for turn 3.
- A fast line: Entomb (or Buried Alive) for Archon of Cruelty or Grave Titan, then Reanimate or Animate Dead the same turn or the next.
- Cast Demonic Tutor into Entomb on turn two to set up Reanimate on turn three.

**Mulligans.** Keep hands with two or three lands (or a land plus Sol Ring), a way to fill the yard or find a reanimation spell, and a play before turn 4. A hand that is all reanimation spells with no way to fill the yard is slow; a hand that is all big creatures needs mana. Don't keep one-landers.

## Tutor targets by board state

Three tutors: **Demonic Tutor** and **Diabolic Tutor** (any card to hand) and **Gifts Ungiven** (four different cards, opponent splits them between your hand and graveyard).

- **One combo piece away → tutor it.** With Mikaeus out, Demonic Tutor for Triskelion (no outlet needed), or for Kitchen Finks if a sac outlet is out too. With Melira and an outlet out, tutor Finks. With Ashnod's Altar and Grave Titan out, tutor Nim Deathmantle. Resolve Grand Abolisher first if you can.
- **Empty yard, reanimation spell already in hand → tutor Entomb (or Buried Alive).** Demonic Tutor into Entomb on turn two sets up Reanimate on turn three.
- **Yard loaded, no way to cheat it out → tutor a reanimation spell.** Demonic Tutor leaves the most mana over to cast it the same turn.
- **Want both halves at once → Gifts Ungiven** for a reanimation target plus reanimation spells, or for Triskelion, Kitchen Finks, Melira, and a reanimation spell — any split between hand and graveyard still leaves you a working loop.
- **Loop assembled, need to close → tutor the payoff.** Blood Artist or Zulaport Cutthroat (or Altar of Dementia as the outlet) turns loops 2–4 into a kill; Yawgmoth's Will once the yard is deep.
- **Long game, combo answered → Muldrotha.** She replays the loop pieces, Aura Shards, and a reanimation spell from the graveyard every turn.
- **Under pressure → tutor interaction.** Toxic Deluge or Austere Command to reset, or Swords / Path / Anguished Unmaking / Assassin's Trophy for one problem.

Diabolic Tutor is the least efficient at four mana. Entomb and Buried Alive also work as combo tutors here: they put a creature piece in the yard for Reanimate, Animate Dead, Necromancy, or Dread Return.

## Mana and consistency

The list is exactly 100 cards. 37 lands: 23 nonbasic plus 6 Swamp, 3 Forest, 3 Plains, 2 Island (14 basics). The nonbasics are untapped shock and pain lands for fixing (Hallowed Fountain, Overgrown Tomb, Temple Garden, Watery Grave; Brushland, Caves of Koilos, Underground River, Yavimaya Coast), check lands (Glacial Fortress, Hinterland Harbor, Isolated Chapel, Sunpetal Grove, Woodland Cemetery), tapped lands (Arcane Sanctum, Treno, Dark City, Bojuka Bog), Command Tower, Exotic Orchard, Ash Barrens, Evolving Wilds, Fabled Passage, Strip Mine and Karn's Bastion. Arcane Sanctum gives blue for Arcane Denial, Swan Song and Gifts Ungiven. Karn's Bastion is a colourless land that proliferates.

Mana: five rocks (Sol Ring, Arcane Signet, Talisman of Hierarchy, Mind Stone, Chromatic Lantern), and two creatures (Birds of Paradise, Sakura-Tribe Elder). The deck cheats its threats in, so it has less ramp than a fair deck; Nim Deathmantle + Ashnod's Altar is mana only as part of the combo.

**Graveyard fill.** Stitcher's Supplier, Buried Alive, Entomb, Tortured Existence and Gifts Ungiven fill the yard. Displacer Kitten and the flicker cards replace some of the filling with value.

**Reanimation count is five** (Reanimate, Animate Dead, Necromancy, Unburial Rites, Dread Return) plus Sheoldred, Whispering One, Yawgmoth's Will, and Muldrotha (Animate Dead or Necromancy again from the graveyard every turn). Every reanimation spell can take a legendary creature.

**Card draw.** Harmonize, Tireless Tracker, Phyrexian Arena, Skullclamp, Mind Stone, Sheoldred (gain life per draw), and Atraxa's own enter trigger.

**Interaction.** Swords to Plowshares, Path to Exile, Anguished Unmaking, and Assassin's Trophy for spot removal, plus two destroys from Summon: Bahamut; Aura Shards for repeatable artifact/enchantment removal; Austere Command and Toxic Deluge as resets; Arcane Denial and Swan Song as counters; Galadriel's Dismissal, Ephemerate, and Grand Abolisher as protection; Hornet Queen as a blocker wall.

## Bracket and Rule 0

**Bracket 4.** Three Game Changers: **Demonic Tutor**, **Gifts Ungiven** and **Aura Shards**, which would be Bracket 3 on count alone. The deck is Bracket 4 because of its four infinite loops, including the two-card Mikaeus + Triskelion, all tutorable. Diabolic Tutor, Displacer Kitten, Grand Abolisher, Nim Deathmantle, Triskelion, Kitchen Finks, Melira, Assassin's Trophy, and Muldrotha are off-list.

Disclose before the game:

- **Four infinite loops:** Mikaeus + Triskelion (infinite damage, no outlet needed); Mikaeus or Melira + Kitchen Finks + a sac outlet (infinite life and deaths); Nim Deathmantle + Ashnod's Altar + Grave Titan (infinite mana, ETBs, and deaths).
- The three Game Changers above, especially **Demonic Tutor** and **Gifts Ungiven** (tutors for the loop pieces).
- **Grand Abolisher**, which shuts off their interaction on your turn.
- **Yawgmoth's Will**, which can produce a long turn out of nowhere.
- **Austere Command**, a sweeper that can hit your own board too.
- **Strip Mine** as land kill: one-shot, or **every turn with Muldrotha** replaying it from the graveyard.
- **Muldrotha** recasts a creature, an artifact, and an enchantment (Animate Dead, Necromancy) from your graveyard each turn.
- The deck reanimates out of *any* graveyard, so opponents' creatures dying is relevant information for them.

Re-check the official list at https://commanderbrackets.com/faq before an event.

## Decklist by type (100)

**Commander (1).** Sephiroth, the Savior (Atraxa, Grand Unifier)

**Creatures (24).** Archon of Cruelty, Birds of Paradise, Blood Artist, Displacer Kitten, Elesh Norn Grand Cenobite, Elesh Norn Mother of Machines, Flickering Hound, Grand Abolisher, Grave Titan, Hornet Queen, Kitchen Finks, Melira Sylvok Outcast, Mikaeus the Unhallowed, Muldrotha the Gravetide, Restoration Angel, Sakura-Tribe Elder, Sheoldred the Apocalypse, Sheoldred Whispering One, Stitcher's Supplier, Summon: Bahamut, Tireless Tracker, Triskelion, Viscera Seer, Zulaport Cutthroat

**Enchantments (7).** Animate Dead, Aura Shards, Grave Pact, Necromancy, Phyrexian Arena, Teleportation Circle, Tortured Existence

**Artifacts (10).** Altar of Dementia, Arcane Signet, Ashnod's Altar, Chromatic Lantern, Conjurer's Closet, Mind Stone, Nim Deathmantle, Skullclamp, Sol Ring, Talisman of Hierarchy

**Sorceries (10).** Austere Command, Buried Alive, Demonic Tutor, Diabolic Tutor, Dread Return, Harmonize, Reanimate, Toxic Deluge, Unburial Rites, Yawgmoth's Will

**Instants (11).** Anguished Unmaking, Arcane Denial, Assassin's Trophy, Entomb, Ephemerate, Galadriel's Dismissal, Gifts Ungiven, Momentary Blink, Path to Exile, Swan Song, Swords to Plowshares

**Lands (37).** Arcane Sanctum, Ash Barrens, Bojuka Bog, Brushland, Caves of Koilos, Command Tower, Evolving Wilds, Exotic Orchard, Fabled Passage, Glacial Fortress, Hallowed Fountain, Hinterland Harbor, Isolated Chapel, Karn's Bastion, Overgrown Tomb, Strip Mine, Sunpetal Grove, Temple Garden, Treno Dark City, Underground River, Watery Grave, Woodland Cemetery, Yavimaya Coast, 6 Swamp, 3 Forest, 3 Plains, 2 Island

## Import list (100)

```
1 Altar of Dementia
1 Anguished Unmaking
1 Animate Dead
1 Arcane Denial
1 Arcane Sanctum
1 Arcane Signet
1 Archon of Cruelty
1 Ash Barrens
1 Ashnod's Altar
1 Assassin's Trophy
1 Atraxa, Grand Unifier
1 Aura Shards
1 Austere Command
1 Birds of Paradise
1 Blood Artist
1 Bojuka Bog
1 Brushland
1 Buried Alive
1 Caves of Koilos
1 Chromatic Lantern
1 Command Tower
1 Conjurer's Closet
1 Demonic Tutor
1 Diabolic Tutor
1 Displacer Kitten
1 Dread Return
1 Elesh Norn, Grand Cenobite
1 Elesh Norn, Mother of Machines
1 Entomb
1 Ephemerate
1 Evolving Wilds
1 Exotic Orchard
1 Fabled Passage
1 Flickering Hound
1 Galadriel's Dismissal
1 Gifts Ungiven
1 Glacial Fortress
1 Grand Abolisher
1 Grave Pact
1 Grave Titan
1 Hallowed Fountain
1 Harmonize
1 Hinterland Harbor
1 Hornet Queen
1 Isolated Chapel
1 Karn's Bastion
1 Kitchen Finks
1 Melira, Sylvok Outcast
1 Mikaeus, the Unhallowed
1 Mind Stone
1 Momentary Blink
1 Muldrotha, the Gravetide
1 Necromancy
1 Nim Deathmantle
1 Overgrown Tomb
1 Path to Exile
1 Phyrexian Arena
1 Reanimate
1 Restoration Angel
1 Sakura-Tribe Elder
1 Sheoldred, the Apocalypse
1 Sheoldred, Whispering One
1 Skullclamp
1 Sol Ring
1 Stitcher's Supplier
1 Strip Mine
1 Summon: Bahamut
1 Sunpetal Grove
1 Swan Song
1 Swords to Plowshares
1 Talisman of Hierarchy
1 Teleportation Circle
1 Temple Garden
1 Tireless Tracker
1 Tortured Existence
1 Toxic Deluge
1 Treno, Dark City
1 Triskelion
1 Unburial Rites
1 Underground River
1 Viscera Seer
1 Watery Grave
1 Woodland Cemetery
1 Yavimaya Coast
1 Yawgmoth's Will
1 Zulaport Cutthroat
3 Forest
2 Island
3 Plains
6 Swamp
```
