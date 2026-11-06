# Thanos, the Mad Titan — Five-Colour Artifacts and Dragons (High Bracket 4)

> Built 2026-11-05 from three of JD's decks: what the Tymna/Thrasios Doomsday rebuild leaves of the Thanos turbo deck,
> the Karn colourless artifacts deck, and The Ur-Dragon Dragon deck. The Ur-Dragon moves into the 99. The card moves
> are in `documents/doomsday-thanos-rebuild-swaps.md`. Not simulated yet.

Fast artifact mana puts Thanos down on turn 2 or 3. Thanos is a cheap 4/4 with deathtouch and lifelink, he makes
Deflecting Swat free and gives Jeska's Will both modes, and his power-up is a wipe you aim at the opponents' side.
Behind him the deck builds card advantage with The One Ring, Rhystic Study, Smothering Tithe, Mystic Forge and Urza.
It wins with one of two infinite combos, **Basalt Monolith + Rings of Brighthearth** (into Walking Ballista) and
**Astral Dragon + Cursed Mirror** (with Terror of the Peaks), or with big threats the wipes clear the way for: The
Ur-Dragon, Wurmcoil Engine, Blightsteel Colossus, Portal to Phyrexia, Karn Liberated and Ugin. It is weak to artifact
wipes and Null Rod effects, to fast combo before turn 4, and to tables that punish slow, expensive top end.

| | |
|---|---|
| **Commander** | Thanos, the Mad Titan ({R}{W}{B}, 4/4 deathtouch, lifelink); five colours (his power-up costs all five) |
| **Bracket** | 4: 17 Game Changers and two infinite combos. Played against Tier 5 (High Bracket 4) |
| **Game Changers (17)** | Ancient Tomb, Chrome Mox, Demonic Tutor, Enlightened Tutor, Force of Will, Gamble, Grim Monolith, Imperial Seal, Jeska's Will, Mana Vault, Mox Diamond, Rhystic Study, Smothering Tithe, Teferi's Protection, The One Ring, Vampiric Tutor, Worldly Tutor |
| **Simulator** | Deck key `thanos`, tier T5, played from `data/plans/thanos.toml`; findings in `audit/thanos/FINDINGS.md` (no sims yet). Thanos's power-up is card code (`t6_thanos.rs`). Replaced the old `thanos` list (T6) and retired `karn` and `urdragon` on 2026-11-05 |
| **Buys** | None: every card comes from the Thanos, Karn and Ur-Dragon decks |

## Strategy

### The commander

**Thanos, the Mad Titan** ({R}{W}{B}, 4/4). *Deathtouch, lifelink. Power-up — {C}{W}{U}{B}{R}{G}: Put two +1/+1
counters on Thanos. Choose odd or even. Destroy each other creature with mana value of the chosen quality. (Activate
each power-up ability only once. Reduce the cost by his mana cost if he entered this turn. Zero is even.)*

**When to cast him.** Turn 2 off Birds of Paradise or two rocks, turn 3 otherwise. He needs exactly {R}{W}{B}: Sol Ring,
Mana Vault, Grim Monolith, Thran Dynamo and Ancient Tomb can't pay for him, but Arcane Signet, Chromatic Lantern,
Coalition Relic, Mox Opal and the Moxen can. Cast him before your engines: he's the cheapest threat, a good blocker,
and he turns on the free spells.

**What he turns on while he's on the battlefield:**

- **Deflecting Swat** ({2}{R}, free while you control a commander): new targets for target spell or ability. Move
  removal off Thanos or The One Ring, or move a counterspell onto another spell on the stack.
- **Jeska's Will** ({2}{R}): with a commander out you get both modes: {R} for each card in an opponent's hand, *and*
  exile your top three cards and play them this turn. Aim the first mode at the fullest hand.

Both are checked as you cast the spell. If Thanos dies at your upkeep, Swat costs full price that turn.

**The power-up.**

- **Cost:** six mana, or {C}{U}{G} on the turn he entered (the cost drops by his own {R}{W}{B}). So Thanos plus the
  power-up is six mana on one turn.
- **{C} means colourless mana, not generic.** Sources: Sol Ring, Mana Vault, Grim Monolith, Basalt Monolith, Thran
  Dynamo, Mind Stone, The Mightstone and Weakstone (its mana can pay for abilities), Ancient Tomb, Urza's Saga (from
  chapter I) and Wastes. Lands still make {C} under Chromatic Lantern if that's their own ability. The duals, fetches,
  Command Tower, City of Brass, Mana Confluence, Gilded Lotus, Coalition Relic, Arcane Signet, the Moxen, Lotus Petal,
  Treasures and Birds can't pay it.
- **Once per Thanos.** A recast Thanos is a new object and can power up again.
- It's an activated ability, so counterspells and Silence effects don't stop it. Pithing Needle and Cursed Totem
  (opponents' cards) do. There's no sorcery-speed limit: you can wipe on an opponent's turn at full cost.
- **Rings of Brighthearth copies it.** Pay {2} more and Rings copies the power-up. A copy isn't an activation, so the
  once-only rule doesn't stop it. Choose odd for one and even for the other, and every other creature is destroyed,
  yours included. Thanos gets four counters.

**Which parity.** Look at what kills you first, then check your own side:

| Parity | Your creatures it destroys | T5 commanders it destroys |
|---|---|---|
| **Even** | Walking Ballista and every Construct token (0); Urza, Solemn Simulacrum, Nicol Bolas (4); Wurmcoil Engine (leaves two 3/3s), Steel Hellkite, Duplicant (6); Astral Dragon and Mirror tokens copying it (8). Blightsteel Colossus (12) is indestructible and survives | **All five:** Kinnan (2), Urza, Yawgmoth, Zur (4), Miirym (6) |
| **Odd** | Birds of Paradise, Ragavan (1); Cursed Mirror tokens that aren't copying anything (3); Goldspan Dragon, Terror of the Peaks (5); The Ur-Dragon (9) | None |

- **Even kills every T5 commander,** so it's the usual choice when the commander is the problem. Most of what you lose
  to even leaves value behind (Wurmcoil's tokens, Solemn's draw), so the real cost is usually Urza, a Ballista or an
  Astral Dragon. Cast those after the wipe where you can.
- **Odd is for the supporting creatures** (mana creatures, 1-drops, 3s and 5s: Miirym's Terror of the Peaks, Yawgmoth's
  Blood Artist) when the commander isn't the threat. Most of your own board survives it.
- **Against Yawgmoth, be careful.** Undying creatures come back stronger, and Blood Artist or Zulaport Cutthroat
  drains you for each death. Wipe when no death payoff is out, or exile the payoff first.
- **Wipe or win?** With a combo available this turn, win. Wipe when you can't win this turn and the wipe kills the
  creature that's about to win (Kinnan, Urza's Construct army, Yawgmoth's engine, Miirym's Dragons) or clears the way
  for a lethal attack.

### How the deck wins

#### Combo 1: Basalt Monolith + Rings of Brighthearth

- **Pieces:** Basalt Monolith ({3}, {T}: add {C}{C}{C}; {3}: untap) and Rings of Brighthearth ({3}; whenever you
  activate an ability that isn't a mana ability, you may pay {2} to copy it), both on the battlefield.
- **The loop:** tap the Monolith for {C}{C}{C}. Activate its untap ({3}) and pay {2} to copy it. The copy resolves
  first and untaps the Monolith: tap it again for three with the original still on the stack. The original then untaps
  it once more. That's {6} for {5}: one extra colourless mana each loop, as much as you like. The first loop needs five
  mana: the Monolith's own three and two more from elsewhere.
- **Finishers (all colourless):**
  - **Walking Ballista** ({X}{X}) from hand: cast it for a huge X and remove counters, one damage each, at every
    opponent. Its {4} ability adds more counters if you need them.
  - **Mystic Forge** on the battlefield lets you cast artifacts and colourless spells from the top of your library. Use
    its {T}, pay 1 life ability to exile cards you can't cast. Dig to Ballista and cast it for the huge X.
  - **Urza, Lord High Artificer:** {5}: shuffle, exile the top card, play it free this turn. With unlimited mana you
    see the whole library. A Ballista played free has X=0, so cast the Ballista you find with real mana: dig with Forge
    or tutor for it.
- **Tutors:** Enlightened Tutor (an artifact to the top) finds either piece or Mystic Forge. Demonic Tutor, Vampiric
  Tutor and Imperial Seal find anything. Karn, Scion of Urza and Urza's Saga can't (Saga finds only {0} and {1}
  artifacts).
- **Hold the pieces until both can land in one turn** with Force of Will, Pact or Swat behind them. An opponent who
  destroys Rings or the Monolith in response ends the loop.

#### Combo 2: Astral Dragon + Cursed Mirror (+ Terror of the Peaks)

- **Pieces:** Cursed Mirror on the battlefield *as an artifact* (cast it as a rock, or decline its copy); Astral
  Dragon ({6}{U}{U}) in hand; **Terror of the Peaks** on the battlefield to turn the loop into damage.
- **Sequence:**
  1. Cast Astral Dragon. Its enter trigger targets Cursed Mirror and makes two token copies of it, as 3/3 flying
     Dragons.
  2. As each token enters, Cursed Mirror's own text lets it become a copy of any creature until end of turn, with
     haste. **Choose Astral Dragon.**
  3. Each token entered as Astral Dragon, so it has its own enter trigger. Target the original Cursed Mirror again: two
     more tokens each.
  4. Repeat. Each pass doubles the hasty 4/4 flying Dragons.
- **The win:** Terror of the Peaks deals damage equal to each entering creature's power. Every token enters as a 4/4,
  so that's 4 damage per token, to any target. Name a number of loops big enough to kill the table.
- **Without Terror of the Peaks:** make as many hasty fliers as you like and attack the same turn. The copy lasts until
  end of turn, so attack now. Fogs and flying blockers can stop it; Terror can't be stopped that way.
- **Watch out:** removing the Mirror with a trigger on the stack fizzles that trigger and ends the loop. If The
  Ur-Dragon is attacking alongside, its trigger draws a card per attacking Dragon. Attack with no more Dragons than you
  have cards in your library.
- **The Ur-Dragon does it for free:** when your Dragons attack, The Ur-Dragon lets you put a permanent card from your
  hand onto the battlefield. Put Astral Dragon in with Cursed Mirror out, and its enter trigger starts the loop.

#### Fair wins

The power-up and the other wipes clear the board, and these finish:

- **The Ur-Dragon** ({4}{W}{U}{B}{R}{G}, 10/10 flier). Whenever your Dragons attack, draw that many cards, then put a
  permanent card from your hand onto the battlefield free: Portal to Phyrexia, Blightsteel Colossus, Karn Liberated,
  Ugin, Astral Dragon. While it's on the battlefield, your other Dragon spells cost {1} less (Goldspan, Terror,
  Astral, Steel Hellkite and Nicol Bolas are Dragons). It's in the 99 now, so it costs the full nine. Cast it off
  Gilded Lotus, Thran Dynamo, Coalition Relic or a Basalt Monolith turn.
- **Wurmcoil Engine** (deathtouch, lifelink, leaves two 3/3s when it dies) and **Blightsteel Colossus** (11/11 infect,
  trample, indestructible; one hit kills a player).
- **Portal to Phyrexia** ({9}): each opponent sacrifices three creatures, and every upkeep you reanimate a creature from
  any graveyard.
- **Karn Liberated** ({7}): +4 makes a player exile a card from hand; −3 exiles any permanent.
- **Ugin, the Spirit Dragon** ({8}): +2 deals 3 damage. **−X exiles every permanent with mana value X or less that has a
  colour.** Most of your board is colourless artifacts and survives, but Thanos (3), Rhystic Study, Smothering Tithe,
  Sylvan Library and Birds don't. Thanos goes back to the command zone.
- **Goldspan Dragon, Terror of the Peaks and Steel Hellkite** are hasty or evasive damage; Hellkite's {X} ability
  destroys each nonland permanent with mana value X controlled by a player it hit this turn, once each turn.

### The engines

- **The One Ring** ({4}): protection from everything until your next turn when you cast it, then {T}: a burden counter
  and that many cards (1, 2, 3...). You lose 1 life per burden counter each upkeep, so draw with it three or four
  times, then let it sit or bounce it.
- **Rhystic Study** and **Smothering Tithe** tax the table: cards or Treasures every time they cast or draw.
- **Mystic Forge** ({4}): look at the top card any time and cast artifacts and colourless spells from the top. About
  half the deck is colourless.
- **Urza, Lord High Artificer** ({2}{U}{U}): a Construct token, then every untapped artifact taps for {U}. His {5}
  ability plays your top card free each time.
- **Karn, Scion of Urza** ({4}): +1 is a card each turn (an opponent picks which of two); −2 makes a Construct.
- **Karn, the Great Creator** ({4}): **opponents' artifacts' activated abilities can't be activated**, mana abilities
  included. That turns off Kinnan's and Urza's artifact combos and their rocks. +1 makes a noncreature artifact of
  yours a creature until your next turn.
- **Nicol Bolas, the Ravager** ({1}{U}{B}{R}): each opponent discards a card. Later, {4}{U}{B}{R} at sorcery speed turns
  him into a planeswalker (+2: draw two).
- **Jeska's Will, Sylvan Library, Wrenn and Six, Mind Stone, Urza's Bauble** fill in the cards.

### Game plan

**Turns 1-2.** Land and fast mana: Sol Ring, Mana Vault, Chrome Mox, Mox Diamond, Lotus Petal (one use: hold it),
Arcane Signet, Mind Stone, Birds of Paradise. Mox Opal needs three artifacts, so play the Baubles and rocks first.
Thanos on turn 2 if the colours line up.

**Turns 3-4.** An engine (The One Ring, Rhystic Study, Mystic Forge, Urza, a Karn) and a bigger rock (Gilded Lotus,
Thran Dynamo, Coalition Relic). Keep {U}{U} or a free counter open on opponents' turns once the table starts to
threaten wins.

**Turns 5+.** Assemble a combo, or wipe and finish: power-up, Oblivion Stone or Ugin, then The Ur-Dragon, Wurmcoil,
Portal or Blightsteel. Tier 5 games run about eight rounds, so a turn-5 or turn-6 win is on pace.

**Hold up or go?**

- **Go** when you have a combo with protection, or when an opponent will win next turn and you can't stop it.
- **Wipe** when someone's board is about to win and you can't win this turn.
- **Hold** Force of Will, Pact of Negation, Mana Drain, Red Elemental Blast or Swat for the opponent who is closest
  to winning: Kinnan and Urza (artifact combos), Yawgmoth (death loops), Zur (Thassa's Oracle).

### Mulligans

The first mulligan is free; after that it's a London mulligan (draw 7, bottom one card per mulligan after the free
one).

| Hand | Keep when |
|---|---|
| 7 cards | 2-5 lands (1 land only with 3+ rocks or Moxen); 3+ mana by turn 2 or 4+ by turn 3; at least one coloured source; and something to do: an engine, a tutor, a combo piece, or Thanos castable by turn 3 |
| 6 or 5 cards | 1-5 lands, 3+ cards that make mana, and any action card |
| 4 cards | Keep |

- **Bottom first:** lands past the fourth, the top end (Portal to Phyrexia, Blightsteel, Ugin, The Ur-Dragon, Karn
  Liberated), then the situational answers (Decimate, Chaos Warp, Ratchet Bomb).
- **A hand of only top end goes back,** even with mana. This deck wins with engines first.

| # | Hand | Call | Why |
|---|---|---|---|
| 1 | Underground Sea, Plateau, Wooded Foothills, Sol Ring, Rhystic Study, Force of Will, Wurmcoil Engine | Keep | Thanos on turn 3 (Plateau {R}{W}, Sea {B}; Sol Ring's {C}{C} can't pay him), Rhystic Study on turn 2, protection and a threat |
| 2 | Ancient Tomb, Mana Vault, Thran Dynamo, Gilded Lotus, Portal to Phyrexia, Blightsteel Colossus, Karn Liberated | Ship | Lots of mana but no coloured source, and nothing to do before turn 4 |
| 3 | Volcanic Island, Scrubland, Arid Mesa, Basalt Monolith, Rings of Brighthearth, Walking Ballista, Enlightened Tutor | Keep | The whole combo: Monolith turn 3, Rings turn 4, loop and Ballista on turn 5 |
| 4 | Tundra, Taiga, Bayou, Command Tower, Steam Vents, Swords to Plowshares, Chaos Warp | Ship | Five lands and two answers: no engine, no threat |
| 5 | Steam Vents, Volcanic Island, Mox Opal, Urza's Bauble, Lotus Petal, Mind Stone, Urza, Lord High Artificer | Keep | Turn 1: Vents, Opal, Bauble and Petal turn on metalcraft. Turn 2: both lands, Opal and Petal pay for Urza |
| 6 | Bayou, Sacred Foundry, Arcane Signet, Mystic Forge, The One Ring, Gamble, Pact of Negation | Keep (close) | Signet on turn 2, then Thanos ({R}{W}{B} from Foundry, Bayou and Signet). The engines need a third land |

### What to tutor for

| Situation | Tutor | Target |
|---|---|---|
| Basalt Monolith or Rings out, the other missing | Enlightened Tutor at the end step before your turn, or Demonic Tutor | The missing piece |
| Both combo pieces out, no finisher | Enlightened Tutor or Demonic Tutor | Walking Ballista (to hand, then cast it with the loop's mana) or Mystic Forge |
| Cursed Mirror out, no Astral Dragon | Worldly Tutor (a creature to the top), Demonic Tutor | Astral Dragon; Terror of the Peaks first if the table has flying blockers |
| Nothing going, turns 2-3 | Demonic Tutor, Gamble | The One Ring, Rhystic Study or Mystic Forge |
| An opponent about to win | Vampiric Tutor at instant speed, Demonic on your turn | Force of Will, Red Elemental Blast (blue commanders) or Teferi's Protection |
| A board you can't beat | Demonic Tutor, Vampiric Tutor | Oblivion Stone, Ugin, Portal to Phyrexia, or Rings to copy the power-up |
| Missing a colour or {C} | Expedition Map, Nature's Lore, Wayfarer's Bauble, Urza's Saga | Wastes or Ancient Tomb for {C}; a dual for the colour |

- **Gamble** ({R}): the random discard comes after the search, from your whole hand. With only the found card in hand,
  you discard it. Cast it with three or more other cards in hand.
- **Imperial Seal** and **Vampiric Tutor** put the card on top: draw it next turn, or this turn with Mystic Forge
  (artifacts) or Jeska's Will.
- **Urza's Saga** chapter III finds an artifact with mana cost {0} or {1}: Sol Ring, Mana Vault, Mox Opal, Chrome Mox,
  Mox Diamond, Lotus Petal, Urza's Bauble, Wayfarer's Bauble or Expedition Map. Chapter II makes Constructs that grow
  with your artifacts.

### Interaction

| Card | Use it on |
|---|---|
| **Force of Will** | Any spell: free by exiling a blue card. Save it for a win or a wipe |
| **Force of Negation** | A noncreature spell, free on opponents' turns |
| **Pact of Negation** | Any spell, free now; pay {3}{U}{U} at your next upkeep or lose. Use it when you win this turn, or with five mana sure next upkeep |
| **Mana Drain** ({U}{U}) | Any spell; you get colourless mana equal to its mana value at your next main phase: a Basalt or Ur-Dragon turn |
| **Red Elemental Blast** ({R}) | Counter a blue spell, or destroy a blue permanent. Four of the five T5 commanders are blue: Kinnan, Urza, Zur, Miirym |
| **Deflecting Swat** | Free with Thanos out: redirect removal, a counter or a targeted wipe effect |
| **Swords to Plowshares** | Exile a creature: a commander, a combo creature or an undying creature |
| **Chaos Warp** | Any permanent: shuffled away (they may hit a permanent off the top) |
| **Decimate** | An artifact, a creature, an enchantment and a land at once; needs a legal target for all four |
| **Duplicant** | Exile a nontoken creature on entry |
| **Oblivion Stone** | {4}, {T}: protect a permanent of yours with a fate counter; {5}, {T}, sacrifice: destroy every other nonland permanent |
| **Ratchet Bomb** | Tick it up, then destroy every nonland permanent of that mana value. At 0 it kills tokens. Your Treasures and Constructs die too |
| **Teferi's Protection** | Phase your board out and lock your life until your next turn: answers a wipe, a lethal attack or a combo kill |
| **Lightning Greaves** | Shroud and haste for Thanos; equip {0} |

### Matchups (Tier 5)

| Opponent | Threat | Plan | Cards that matter |
|---|---|---|---|
| **Kinnan, Bonder Prodigy** (Simic mana combo) | Kinnan + Basalt Monolith (unlimited mana) into Walking Ballista or Thrasios; Kinnan + Freed from the Real or Pemmin's Aura | Kill Kinnan on sight; hold a counter for the combo turn | Karn, the Great Creator (their Monolith can't activate), Red Elemental Blast (Kinnan is blue), even power-up, Swords |
| **Urza, Lord High Artificer** (artifact combo and stax) | Isochron Scepter + Dramatic Reversal; Basalt + Rings; Hullbreaker Horror | Shut off their artifacts and keep a counter for the combo | Karn, the Great Creator (stops their artifacts' activated abilities, mana included), Red Elemental Blast on Urza, Oblivion Stone, Ratchet Bomb, even power-up (Urza and the Construct) |
| **Yawgmoth, Thran Physician** (mono-black aristocrats) | Yawgmoth + undying creatures + a death payoff | Exile, don't destroy: Swords, Duplicant, Karn Liberated, Ugin −X (everything of theirs has a colour) | Wiping into a Blood Artist drains you; Portal to Phyrexia makes them sacrifice; Teferi's Protection against a drain |
| **Zur the Enchanter** (Esper enchantment control, Oracle) | Heliod + Walking Ballista; Thassa's Oracle with a near-empty library; stax (Drannith Magistrate, Opposition Agent, Grand Arbiter) | Remove the stax creature first, then the enchantments | Red Elemental Blast on Zur, Decimate, Chaos Warp, Oblivion Stone, Ugin −X. Drannith stops you recasting Thanos from the command zone; Opposition Agent steals your tutors and fetches |
| **Miirym, Sentinel Wyrm** (Temur Dragons) | Copied Dragons, Terror of the Peaks, Wrathful Red Dragon + Blasphemous Act, Cyclonic Rift | Wipe when Miirym and the Dragons are out: even for Miirym (6), odd for 5-drops like Terror of the Peaks; Rings copies both | Power-up, Red Elemental Blast (Miirym is blue), Ugin −X; keep Teferi's Protection for Blasphemous Act turns |

### Life management

| Source | Cost |
|---|---|
| Fetch lands | 1 each |
| Ancient Tomb | 2 per tap |
| City of Brass, Mana Confluence | 1 per tap |
| Mana Vault | 1 per draw step while it's tapped |
| Vampiric Tutor, Imperial Seal | 2 each |
| Force of Will | 1 |
| The One Ring | 1 per burden counter, each upkeep |
| Sylvan Library | 4 per extra card kept |
| Mystic Forge | 1 per exile |
| Shock lands | 2 to enter untapped |

Thanos and Wurmcoil gain it back with lifelink. Don't let The One Ring run past three or four burden counters
unless the game ends soon.

### Common mistakes

1. Choosing even with Urza or a big Ballista out when odd would do, or odd with The Ur-Dragon out. Every T5
   commander is even, so check the commanders first.
2. Wiping into Yawgmoth's death payoffs.
3. Trying to pay the power-up's {C} with Treasures, Moxen or any-colour rocks.
4. Casting a Walking Ballista found by Urza's ability for free: X is 0.
5. Running the Astral Dragon loop with The Ur-Dragon attacking and too few cards in the library.
6. Casting Ugin's −X for 3 or more with Thanos you want to keep on the battlefield.
7. Casting Gamble with only one other card in hand.
8. Casting Pact of Negation on a turn you might not win, without {3}{U}{U} for your next upkeep.
9. Letting The One Ring's burden counters drain you out of a long game.
10. Tapping out for an engine when an opponent is about to win and you hold the counter.

## Mana

33 lands, 14 rocks and Moxen, Birds of Paradise and Nature's Lore.

- **Original duals (9):** Underground Sea, Volcanic Island, Badlands, Plateau, Scrubland, Taiga, Bayou, Tropical
  Island, Tundra.
- **Shock lands (9):** Blood Crypt, Breeding Pool, Hallowed Fountain, Overgrown Tomb, Sacred Foundry, Steam Vents,
  Stomping Ground, Temple Garden, Watery Grave.
- **Fetch lands (8):** Bloodstained Mire, Flooded Strand, Marsh Flats, Misty Rainforest, Polluted Delta, Verdant
  Catacombs, Windswept Heath, Wooded Foothills.
- **Any colour:** Command Tower, City of Brass, Mana Confluence.
- **Colourless:** Ancient Tomb, Urza's Saga, Wastes.
- **Basic:** Forest (for Solemn Simulacrum, Wayfarer's Bauble and Nature's Lore, with Wastes).
- **Rocks:** Sol Ring, Mana Vault, Grim Monolith, Basalt Monolith, Thran Dynamo, Mind Stone, Arcane Signet, Chromatic
  Lantern (every land taps for any colour), Coalition Relic (store a charge for a burst next turn), Gilded Lotus;
  Moxen and Petal: Chrome Mox, Mox Diamond, Mox Opal, Lotus Petal. The Mightstone and Weakstone adds {C}{C} for
  artifacts and abilities.

## Bracket and Rule 0

Bracket 4 (High): 17 Game Changers and two infinite combos, Basalt Monolith + Rings of Brighthearth (unlimited
colourless mana) and Astral Dragon + Cursed Mirror (unlimited hasty Dragon tokens). Thanos's power-up is a repeatable
one-sided board wipe. No mass land destruction, no extra turns. Tell the table about both combos before the game.

## Decklist by type (100)

**Commander (1).** Thanos, the Mad Titan

**Creatures (14).** Astral Dragon, Birds of Paradise, Blightsteel Colossus, Duplicant, Goldspan Dragon, Nicol Bolas, the Ravager, Ragavan, Nimble Pilferer, Solemn Simulacrum, Steel Hellkite, Terror of the Peaks, The Ur-Dragon, Urza, Lord High Artificer, Walking Ballista, Wurmcoil Engine

**Planeswalkers (5).** Karn Liberated, Karn, Scion of Urza, Karn, the Great Creator, Ugin, the Spirit Dragon, Wrenn and Six

**Artifacts (26).** Arcane Signet, Basalt Monolith, Chromatic Lantern, Chrome Mox, Coalition Relic, Cursed Mirror, Expedition Map, Gilded Lotus, Grim Monolith, Lightning Greaves, Lotus Petal, Mana Vault, Mind Stone, Mox Diamond, Mox Opal, Mystic Forge, Oblivion Stone, Portal to Phyrexia, Ratchet Bomb, Rings of Brighthearth, Sol Ring, The Mightstone and Weakstone, The One Ring, Thran Dynamo, Urza's Bauble, Wayfarer's Bauble

**Enchantments (3).** Rhystic Study, Smothering Tithe, Sylvan Library

**Instants (12).** Chaos Warp, Deflecting Swat, Enlightened Tutor, Force of Negation, Force of Will, Mana Drain, Pact of Negation, Red Elemental Blast, Swords to Plowshares, Teferi's Protection, Vampiric Tutor, Worldly Tutor

**Sorceries (6).** Decimate, Demonic Tutor, Gamble, Imperial Seal, Jeska's Will, Nature's Lore

**Lands (33).** Ancient Tomb, Badlands, Bayou, Blood Crypt, Bloodstained Mire, Breeding Pool, City of Brass, Command Tower, Flooded Strand, Forest, Hallowed Fountain, Mana Confluence, Marsh Flats, Misty Rainforest, Overgrown Tomb, Plateau, Polluted Delta, Sacred Foundry, Scrubland, Steam Vents, Stomping Ground, Taiga, Temple Garden, Tropical Island, Tundra, Underground Sea, Urza's Saga, Verdant Catacombs, Volcanic Island, Wastes, Watery Grave, Windswept Heath, Wooded Foothills

## Import list (100)

```
1 Ancient Tomb
1 Arcane Signet
1 Astral Dragon
1 Badlands
1 Basalt Monolith
1 Bayou
1 Birds of Paradise
1 Blightsteel Colossus
1 Blood Crypt
1 Bloodstained Mire
1 Breeding Pool
1 Chaos Warp
1 Chromatic Lantern
1 Chrome Mox
1 City of Brass
1 Coalition Relic
1 Command Tower
1 Cursed Mirror
1 Decimate
1 Deflecting Swat
1 Demonic Tutor
1 Duplicant
1 Enlightened Tutor
1 Expedition Map
1 Flooded Strand
1 Force of Negation
1 Force of Will
1 Forest
1 Gamble
1 Gilded Lotus
1 Goldspan Dragon
1 Grim Monolith
1 Hallowed Fountain
1 Imperial Seal
1 Jeska's Will
1 Karn Liberated
1 Karn, Scion of Urza
1 Karn, the Great Creator
1 Lightning Greaves
1 Lotus Petal
1 Mana Confluence
1 Mana Drain
1 Mana Vault
1 Marsh Flats
1 Mind Stone
1 Misty Rainforest
1 Mox Diamond
1 Mox Opal
1 Mystic Forge
1 Nature's Lore
1 Nicol Bolas, the Ravager
1 Oblivion Stone
1 Overgrown Tomb
1 Pact of Negation
1 Plateau
1 Polluted Delta
1 Portal to Phyrexia
1 Ragavan, Nimble Pilferer
1 Ratchet Bomb
1 Red Elemental Blast
1 Rhystic Study
1 Rings of Brighthearth
1 Sacred Foundry
1 Scrubland
1 Smothering Tithe
1 Solemn Simulacrum
1 Sol Ring
1 Steam Vents
1 Steel Hellkite
1 Stomping Ground
1 Swords to Plowshares
1 Sylvan Library
1 Taiga
1 Teferi's Protection
1 Temple Garden
1 Terror of the Peaks
1 Thanos, the Mad Titan
1 The Mightstone and Weakstone
1 The One Ring
1 The Ur-Dragon
1 Thran Dynamo
1 Tropical Island
1 Tundra
1 Ugin, the Spirit Dragon
1 Underground Sea
1 Urza, Lord High Artificer
1 Urza's Bauble
1 Urza's Saga
1 Vampiric Tutor
1 Verdant Catacombs
1 Volcanic Island
1 Walking Ballista
1 Wastes
1 Watery Grave
1 Wayfarer's Bauble
1 Windswept Heath
1 Wooded Foothills
1 Worldly Tutor
1 Wrenn and Six
1 Wurmcoil Engine
```
