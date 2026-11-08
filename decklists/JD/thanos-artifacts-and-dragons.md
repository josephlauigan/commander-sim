# Thanos, the Mad Titan — Five-Colour Artifacts and Dragons (High Bracket 4)

Fast artifact mana puts Thanos down on turn 2 or 3. Thanos is a cheap 4/4 with deathtouch and lifelink, he makes
Deflecting Swat free and gives Jeska's Will both modes, and his power-up is a wipe you aim at the opponents' side.
Behind him the deck builds card advantage with The One Ring, Rhystic Study, Smothering Tithe, Mystic Forge and Urza.
It wins with one of two infinite combos, **Basalt Monolith + Rings of Brighthearth** (into Walking Ballista) and
**Astral Dragon + Cursed Mirror** (with Terror of the Peaks), or with big threats the wipes clear the way for: The
Ur-Dragon and its Dragons (Glorybringer, Thundermaw Hellkite, Utvara Hellkite), Summon: Bahamut and Elesh Norn. It is weak to artifact
wipes and Null Rod effects, to fast combo before turn 4, and to tables that punish slow, expensive top end.

| | |
|---|---|
| **Commander** | Thanos, the Mad Titan ({R}{W}{B}, 4/4 deathtouch, lifelink); five colours (his power-up costs all five) |
| **Bracket** | 4: 16 Game Changers and two infinite combos. Played against Tier 5 (High Bracket 4) |
| **Game Changers (16)** | Ancient Tomb, Chrome Mox, Demonic Tutor, Enlightened Tutor, Force of Will, Grim Monolith, Imperial Seal, Jeska's Will, Mana Vault, Mox Diamond, Rhystic Study, Smothering Tithe, Teferi's Protection, The One Ring, Vampiric Tutor, Worldly Tutor |
| **Simulator** | Deck key `thanos`, tier T5, played from `data/plans/thanos.toml`; findings in `audit/thanos/FINDINGS.md`, history in `decklists/CHANGELOG.md`. Thanos's power-up is card code (`t6_thanos.rs`) |
| **Buys** | None |

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
  chapter I) and Strip Mine. Lands still make {C} under Chromatic Lantern if that's their own ability. The duals, fetches,
  Command Tower, City of Brass, Mana Confluence, Gilded Lotus, Coalition Relic, Arcane Signet, the Moxen (Mox Jasper
  too), Treasures and Birds can't pay it.
- **Once per Thanos.** A recast Thanos is a new object and can power up again.
- It's an activated ability, so counterspells and Silence effects don't stop it. Pithing Needle and Cursed Totem
  (opponents' cards) do. There's no sorcery-speed limit: you can wipe on an opponent's turn at full cost.
- **Rings of Brighthearth copies it.** Pay {2} more and Rings copies the power-up. A copy isn't an activation, so the
  once-only rule doesn't stop it. Choose odd for one and even for the other, and every other creature is destroyed,
  yours included. Thanos gets four counters.

**Which parity.** Look at what kills you first, then check your own side:

| Parity | Your creatures it destroys | T5 commanders it destroys |
|---|---|---|
| **Even** | Walking Ballista, Hangarback Walker (it leaves its Thopters) and every Construct token (0); Urza, Solemn Simulacrum, Nicol Bolas (4); Steel Hellkite (6); Astral Dragon, Utvara Hellkite and Mirror tokens copying Astral (8); Scion of Draco (12) | **All five:** Kinnan (2), Urza, Yawgmoth, Zur (4), Miirym (6) |
| **Odd** | Birds of Paradise, Ragavan (1); Cursed Mirror tokens that aren't copying anything (3); Goldspan Dragon, Terror of the Peaks, Elesh Norn, Glorybringer, Thundermaw Hellkite (5); Meteor Golem (7); Summon: Bahamut, The Ur-Dragon (9) | None |

- **Even kills every T5 commander,** so it's the usual choice when the commander is the problem. Most of what you lose
  to even leaves value behind (Solemn's draw, Hangarback's Thopters), so the real cost is usually Urza, a Ballista, Scion
  of Draco or an Astral Dragon. Cast those after the wipe where you can.
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
  permanent card from your hand onto the battlefield free: Summon: Bahamut, Utvara Hellkite, Elesh Norn, Meteor
  Golem, Astral Dragon. While it's on the battlefield, your other Dragon spells cost {1} less (Goldspan, Terror,
  Astral, Steel Hellkite, Nicol Bolas, Glorybringer, Thundermaw, Utvara, Scion of Draco and Bahamut are Dragons). In the 99 it costs the full nine. Cast it off
  Gilded Lotus, Thran Dynamo, Coalition Relic or a Basalt Monolith turn.
- **Hasty Dragons:** **Glorybringer** ({3}{R}{R}, flying, haste; exert it as it attacks for 4 damage to a non-Dragon
  creature: Kinnan, Urza, a blocker) and **Thundermaw Hellkite** ({3}{R}{R}, flying, haste; it taps and pings the
  opponents' fliers as it enters). Both attack the turn they land, so they trigger The Ur-Dragon at once.
- **Utvara Hellkite** ({6}{R}{R}): whenever a Dragon you control attacks, a 6/6 flying Dragon token. With Terror of the
  Peaks out each token is 6 damage as it enters.
- **Summon: Bahamut** ({9}, a Saga Dragon): chapters I and II destroy a nonland permanent each, III draws two, IV deals
  damage equal to the total mana value of your other permanents to each opponent.
- **Elesh Norn, Mother of Machines** ({4}{W}): your enter triggers happen twice (Terror of the Peaks, Astral Dragon,
  Solemn, Meteor Golem) and the opponents' don't happen at all.
- **Scion of Draco** ({12}, {2} less per basic land type among your lands, so usually {2}): a 4/4 flying artifact Dragon;
  your white creatures have vigilance, blue hexproof, black lifelink, red first strike, green trample.
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
- **Crucible of Worlds** ({3}): play lands from your graveyard: replay Urza's Saga after it sacrifices itself, the fetch
  lands, Wrenn and Six's lands, and **Strip Mine** every turn (it also makes {C} for the power-up).
- **Mox Jasper** ({0}): any colour, but only while you control a Dragon (Scion of Draco is the early one).
- **Hangarback Walker** ({X}{X}): grows with spare mana and leaves that many 1/1 flying Thopters when it dies.
- **Nicol Bolas, the Ravager** ({1}{U}{B}{R}): each opponent discards a card. Later, {4}{U}{B}{R} at sorcery speed turns
  him into a planeswalker (+2: draw two).
- **Jeska's Will, Sylvan Library, Wrenn and Six, Mind Stone, Urza's Bauble** fill in the cards.

### Game plan

**Turns 1-2.** Land and fast mana: Sol Ring, Mana Vault, Chrome Mox, Mox Diamond, Arcane Signet, Mind Stone,
Birds of Paradise; Scion of Draco for {2} with three or more basic land types out, then Mox Jasper. Mox Opal needs three artifacts, so play the Baubles and rocks first.
Thanos on turn 2 if the colours line up.

**Turns 3-4.** An engine (The One Ring, Rhystic Study, Mystic Forge, Urza, Karn, Scion of Urza, Crucible) and a bigger rock (Gilded Lotus,
Thran Dynamo, Coalition Relic). Keep {U}{U} or a free counter open on opponents' turns once the table starts to
threaten wins.

**Turns 5+.** Assemble a combo, or wipe and finish: power-up or Summon: Bahamut, then The Ur-Dragon, the hasty
Dragons or Utvara Hellkite. Tier 5 games run about eight rounds, so a turn-5 or turn-6 win is on pace.

**Hold up or go?**

- **Go** when you have a combo with protection, or when an opponent will win next turn and you can't stop it.
- **Wipe** when someone's board is about to win and you can't win this turn.
- **Hold** Force of Will, Pact of Negation, Mana Drain, Disdainful Stroke, Red Elemental Blast or Swat for the opponent who is closest
  to winning: Kinnan and Urza (artifact combos), Yawgmoth (death loops), Zur (Thassa's Oracle).

### Mulligans

The first mulligan is free; after that it's a London mulligan (draw 7, bottom one card per mulligan after the free
one).

| Hand | Keep when |
|---|---|
| 7 cards | 2-5 lands (1 land only with 3+ rocks or Moxen); 3+ mana by turn 2 or 4+ by turn 3; at least one coloured source; and something to do: an engine, a tutor, a combo piece, or Thanos castable by turn 3 |
| 6 or 5 cards | 1-5 lands, 3+ cards that make mana, and any action card |
| 4 cards | Keep |

- **Bottom first:** lands past the fourth, the top end (Summon: Bahamut, Utvara Hellkite, The Ur-Dragon, Meteor
  Golem), then the situational answers (Decimate, Chaos Warp).
- **A hand of only top end goes back,** even with mana. This deck wins with engines first.

| # | Hand | Call | Why |
|---|---|---|---|
| 1 | Underground Sea, Plateau, Wooded Foothills, Sol Ring, Rhystic Study, Force of Will, Glorybringer | Keep | Thanos on turn 3 (Plateau {R}{W}, Sea {B}; Sol Ring's {C}{C} can't pay him), Rhystic Study on turn 2, protection and a threat |
| 2 | Ancient Tomb, Mana Vault, Thran Dynamo, Gilded Lotus, Summon: Bahamut, Utvara Hellkite, Meteor Golem | Ship | Lots of mana but no coloured source, and nothing to do before turn 4 |
| 3 | Volcanic Island, Scrubland, Marsh Flats, Basalt Monolith, Rings of Brighthearth, Walking Ballista, Enlightened Tutor | Keep | The whole combo: Monolith turn 3, Rings turn 4, loop and Ballista on turn 5 |
| 4 | Tundra, Taiga, Bayou, Command Tower, Steam Vents, Swords to Plowshares, Chaos Warp | Ship | Five lands and two answers: no engine, no threat |
| 5 | Steam Vents, Volcanic Island, Mox Opal, Urza's Bauble, Wayfarer's Bauble, Mind Stone, Urza, Lord High Artificer | Keep | Turn 1: Vents, Opal and Urza's Bauble; turn 2: Mind Stone and Wayfarer's Bauble turn on metalcraft. Turn 3: both lands, Opal and Mind Stone pay for Urza |
| 6 | Bayou, Sacred Foundry, Arcane Signet, Mystic Forge, The One Ring, Imperial Seal, Pact of Negation | Keep (close) | Signet on turn 2, then Thanos ({R}{W}{B} from Foundry, Bayou and Signet). The engines need a third land |

### What to tutor for

| Situation | Tutor | Target |
|---|---|---|
| Basalt Monolith or Rings out, the other missing | Enlightened Tutor at the end step before your turn, or Demonic Tutor | The missing piece |
| Both combo pieces out, no finisher | Enlightened Tutor or Demonic Tutor | Walking Ballista (to hand, then cast it with the loop's mana) or Mystic Forge |
| Cursed Mirror out, no Astral Dragon | Worldly Tutor (a creature to the top), Demonic Tutor | Astral Dragon; Terror of the Peaks first if the table has flying blockers |
| Nothing going, turns 2-3 | Demonic Tutor, Imperial Seal | The One Ring, Rhystic Study or Mystic Forge |
| An opponent about to win | Vampiric Tutor at instant speed, Demonic on your turn | Force of Will, Red Elemental Blast (blue commanders) or Teferi's Protection |
| A board you can't beat | Demonic Tutor, Vampiric Tutor | Summon: Bahamut, Elesh Norn, or Rings to copy the power-up |
| Missing a colour or {C} | Expedition Map, Nature's Lore, Wayfarer's Bauble, Urza's Saga | Strip Mine or Ancient Tomb for {C}; a dual for the colour |

- **Imperial Seal** and **Vampiric Tutor** put the card on top: draw it next turn, or this turn with Mystic Forge
  (artifacts) or Jeska's Will.
- **Urza's Saga** chapter III finds an artifact with mana cost {0} or {1}: Sol Ring, Mana Vault, Mox Opal, Chrome Mox,
  Mox Diamond, Mox Jasper, Urza's Bauble, Wayfarer's Bauble or Expedition Map. Chapter II makes Constructs that grow
  with your artifacts.

### Protecting the win turn

Both combos are mostly abilities and triggers once the pieces are down, so the spells are what get countered: Basalt
Monolith, Rings of Brighthearth and the Walking Ballista for Combo 1; Astral Dragon for Combo 2. Order:

1. **Land the first piece on a quiet turn.** Basalt Monolith or Cursed Mirror alone looks like a rock or a copy card.
   Hold the second piece until you can cast it and finish in the same turn.
2. **Cast it with a counter up.** Force of Will and Pact of Negation answer any spell; Force of Negation a noncreature
   spell; Mana Drain any spell (and its mana pays for next turn); Disdainful Stroke a spell of mana value 4 or more;
   Red Elemental Blast a blue spell. Deflecting Swat (free with Thanos out) moves a counter or a removal spell to a new
   target.
3. **Protect the board, not just the spell.** Teferi's Protection phases your board out against a wipe or a lethal
   swing; The One Ring gives you protection from everything until your next turn when you cast it; Lightning Greaves
   (equip {0}) gives Thanos or Urza shroud. Elesh Norn stops opponents' enter triggers, so their flash blockers and
   enter-the-battlefield removal do nothing.

**Timing traps:**

- **Force of Negation** is free only on an opponent's turn. On your combo turn it costs {1}{U}{U}.
- **Pact of Negation:** at your next upkeep you pay {3}{U}{U} or lose. Use it on the turn you win, or with five mana and
  {U}{U} sure for next upkeep.
- **Deflecting Swat** is free only while you control a commander. If Thanos died this turn, it costs {2}{R}.
- **Mana Drain's** mana arrives at your next main phase, not this turn.
- **Disdainful Stroke** can't hit Kinnan (mana value 2) or a cheap combo piece such as Dramatic Reversal.
- **Removing Cursed Mirror** with an Astral Dragon trigger on the stack fizzles the trigger and ends the loop: keep a
  counter for their removal, not only for their counters.

### Interaction

| Card | Use it on |
|---|---|
| **Force of Will** | Any spell: free by exiling a blue card. Save it for a win or a wipe |
| **Force of Negation** | A noncreature spell, free on opponents' turns |
| **Pact of Negation** | Any spell, free now; pay {3}{U}{U} at your next upkeep or lose. Use it when you win this turn, or with five mana sure next upkeep |
| **Mana Drain** ({U}{U}) | Any spell; you get colourless mana equal to its mana value at your next main phase: a Basalt or Ur-Dragon turn |
| **Disdainful Stroke** ({1}{U}) | A spell with mana value 4 or more: Urza, Yawgmoth, Zur, Miirym and most combo payoffs (not Kinnan, who costs 2) |
| **Red Elemental Blast** ({R}) | Counter a blue spell, or destroy a blue permanent. Four of the five T5 commanders are blue: Kinnan, Urza, Zur, Miirym |
| **Deflecting Swat** | Free with Thanos out: redirect removal, a counter or a targeted wipe effect |
| **Swords to Plowshares** | Exile a creature: a commander, a combo creature or an undying creature |
| **Chaos Warp** | Any permanent: shuffled away (they may hit a permanent off the top) |
| **Decimate** | An artifact, a creature, an enchantment and a land at once; needs a legal target for all four |
| **Meteor Golem** | Destroy a nonland permanent an opponent controls as it enters (twice with Elesh Norn) |
| **Glorybringer** | Exert as it attacks: 4 damage to a non-Dragon creature, every other turn |
| **Strip Mine** | Destroy a key land (Cabal Coffers, Urborg, Ancient Tomb, Gaea's Cradle); Crucible replays it |
| **Teferi's Protection** | Phase your board out and lock your life until your next turn: answers a wipe, a lethal attack or a combo kill |
| **Lightning Greaves** | Shroud and haste for Thanos; equip {0} |

### Answering stax

The stax pieces at Tier 5 are Zur's (Drannith Magistrate, Opposition Agent, Grand Arbiter Augustin IV, Rule of Law,
Rest in Peace, Narset, Parter of Veils) and Urza's Narset.

| Stax piece (opponent's) | What it stops for us | Answer | What still works |
|---|---|---|---|
| Drannith Magistrate | Casting Thanos from the command zone | Swords, Chaos Warp, Decimate, Meteor Golem, Glorybringer's exert, an even power-up (mana value 2) | Both combos (cast from hand) |
| Opposition Agent | Every tutor and fetch land: they take what you find | Swords, Chaos Warp, Decimate, Meteor Golem, an odd power-up (mana value 3) | Combos drawn naturally; don't tutor while it's out |
| Grand Arbiter Augustin IV | Your spells cost {1} more | Red Elemental Blast (it's blue), Swords, Chaos Warp, Meteor Golem, an even power-up (mana value 4) | Everything, a turn slower |
| Rule of Law | One spell per player per turn | Chaos Warp, Decimate, Meteor Golem, Summon: Bahamut | Combo 1 with Basalt and Rings already out (then one spell: the Ballista); Combo 2 (Astral Dragon is one spell, the rest are triggers) |
| Narset, Parter of Veils | More than one draw a turn (The One Ring, Rhystic Study, Sylvan Library's extra cards) | Chaos Warp, Summon: Bahamut, Meteor Golem, attacking it | Combos, which don't draw |
| Rest in Peace | Crucible of Worlds: lands go to exile, not the graveyard, so there is nothing to replay (Strip Mine, Urza's Saga, the fetches) | Chaos Warp, Decimate, Summon: Bahamut, Meteor Golem | Both combos (no graveyard use) |

### Matchups (Tier 5)

| Opponent | Threat | Plan | Cards that matter |
|---|---|---|---|
| **Kinnan, Bonder Prodigy** (Simic mana combo) | Kinnan + Basalt Monolith (unlimited mana) into Walking Ballista or Thrasios; Kinnan + Freed from the Real or Pemmin's Aura | Kill Kinnan on sight; hold a counter for the combo turn | Red Elemental Blast (Kinnan is blue), Glorybringer, even power-up, Swords |
| **Urza, Lord High Artificer** (artifact combo and stax) | Isochron Scepter + Dramatic Reversal; Basalt + Rings; Hullbreaker Horror | Shut off their artifacts and keep a counter for the combo | Disdainful Stroke or Red Elemental Blast on Urza, Meteor Golem, Summon: Bahamut, even power-up (Urza and the Construct) |
| **Yawgmoth, Thran Physician** (mono-black aristocrats) | Yawgmoth + undying creatures + a death payoff | Exile, don't destroy: Swords, Chaos Warp; Elesh Norn stops their creatures' enter triggers | Wiping into a Blood Artist drains you; Teferi's Protection against a drain |
| **Zur the Enchanter** (Esper enchantment control, Oracle) | Heliod + Walking Ballista; Thassa's Oracle with a near-empty library; stax (Drannith Magistrate, Opposition Agent, Grand Arbiter) | Remove the stax creature first, then the enchantments | Red Elemental Blast or Disdainful Stroke on Zur, Decimate, Chaos Warp, Meteor Golem, Summon: Bahamut. Drannith stops you recasting Thanos from the command zone; Opposition Agent steals your tutors and fetches |
| **Miirym, Sentinel Wyrm** (Temur Dragons) | Copied Dragons, Terror of the Peaks, Wrathful Red Dragon + Blasphemous Act, Cyclonic Rift | Wipe when Miirym and the Dragons are out: even for Miirym (6), odd for 5-drops like Terror of the Peaks; Rings copies both | Power-up, Red Elemental Blast (Miirym is blue), Thundermaw Hellkite (taps their fliers); keep Teferi's Protection for Blasphemous Act turns |

### After a stopped attempt

| What happened | Rebuild |
|---|---|
| Basalt Monolith or Rings countered or destroyed | There is one of each and no way to get it back. Switch to Combo 2 (tutor Astral Dragon or Cursed Mirror) or the fair plan |
| Walking Ballista countered with the loop ready | The loop's mana still has a sink: Mystic Forge (cast artifacts off the top) or Urza ({5}: play the top card free). Tutor for either with Enlightened Tutor or Demonic Tutor |
| Astral Dragon countered | It's in the graveyard for good. Go to Combo 1 or the fair plan; keep Cursed Mirror out as a copy of your best creature |
| Cursed Mirror removed mid-loop | The tokens already made stay: with Terror of the Peaks out they have dealt their damage; attack with the hasty ones |
| Thanos removed | Recast him for {R}{W}{B} plus {2} per earlier cast. He's a new object, so his power-up works again; recast him the turn you can also power up ({C}{U}{G} on that turn) |
| Your board wiped | The One Ring and Teferi's Protection save it if cast first. After a wipe: Urza's Saga's Constructs, Crucible of Worlds replaying lands, Hangarback Walker's Thopters and the hand you held |

- A Pact of Negation cast in the failed turn must be paid at your next upkeep. Count that before the attempt.
- After a stop, the next try needs more protection: the table has seen the pieces.

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

Thanos gains it back with lifelink (and Scion of Draco gives your black creatures lifelink). Don't let The One Ring run past three or four burden counters
unless the game ends soon.

### Common mistakes

1. Choosing even with Urza or a big Ballista out when odd would do, or odd with The Ur-Dragon out. Every T5
   commander is even, so check the commanders first.
2. Wiping into Yawgmoth's death payoffs.
3. Trying to pay the power-up's {C} with Treasures, Moxen or any-colour rocks.
4. Casting a Walking Ballista found by Urza's ability for free: X is 0.
5. Running the Astral Dragon loop with The Ur-Dragon attacking and too few cards in the library.
6. Casting Utvara Hellkite or Summon: Bahamut into open counter mana when a cheaper threat would draw it out first.
7. Choosing even with Scion of Draco out when it is your only Dragon for Mox Jasper.
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
- **Colourless:** Ancient Tomb, Urza's Saga, Strip Mine.
- **Basic:** Forest (for Solemn Simulacrum, Wayfarer's Bauble and Nature's Lore).
- **Rocks:** Sol Ring, Mana Vault, Grim Monolith, Basalt Monolith, Thran Dynamo, Mind Stone, Arcane Signet, Chromatic
  Lantern (every land taps for any colour), Coalition Relic (store a charge for a burst next turn), Gilded Lotus;
  Moxen: Chrome Mox, Mox Diamond, Mox Opal, Mox Jasper (with a Dragon out). The Mightstone and Weakstone adds {C}{C} for
  artifacts and abilities.

## Bracket and Rule 0

Bracket 4 (High): 16 Game Changers and two infinite combos, Basalt Monolith + Rings of Brighthearth (unlimited
colourless mana) and Astral Dragon + Cursed Mirror (unlimited hasty Dragon tokens). Thanos's power-up is a repeatable
one-sided board wipe. No mass land destruction, no extra turns. Tell the table about both combos before the game.

## Decklist by type (100)

**Commander (1).** Thanos, the Mad Titan

**Creatures (19).** Astral Dragon, Birds of Paradise, Elesh Norn, Mother of Machines, Glorybringer, Goldspan Dragon, Hangarback Walker, Meteor Golem, Nicol Bolas, the Ravager, Ragavan, Nimble Pilferer, Scion of Draco, Solemn Simulacrum, Steel Hellkite, Summon: Bahamut, Terror of the Peaks, The Ur-Dragon, Thundermaw Hellkite, Urza, Lord High Artificer, Utvara Hellkite, Walking Ballista

**Planeswalkers (2).** Karn, Scion of Urza, Wrenn and Six

**Artifacts (24).** Arcane Signet, Basalt Monolith, Chromatic Lantern, Chrome Mox, Coalition Relic, Crucible of Worlds, Cursed Mirror, Expedition Map, Gilded Lotus, Grim Monolith, Lightning Greaves, Mana Vault, Mind Stone, Mox Diamond, Mox Jasper, Mox Opal, Mystic Forge, Rings of Brighthearth, Sol Ring, The Mightstone and Weakstone, The One Ring, Thran Dynamo, Urza's Bauble, Wayfarer's Bauble

**Enchantments (3).** Rhystic Study, Smothering Tithe, Sylvan Library

**Instants (13).** Chaos Warp, Deflecting Swat, Disdainful Stroke, Enlightened Tutor, Force of Negation, Force of Will, Mana Drain, Pact of Negation, Red Elemental Blast, Swords to Plowshares, Teferi's Protection, Vampiric Tutor, Worldly Tutor

**Sorceries (5).** Decimate, Demonic Tutor, Imperial Seal, Jeska's Will, Nature's Lore

**Lands (33).** Ancient Tomb, Badlands, Bayou, Blood Crypt, Bloodstained Mire, Breeding Pool, City of Brass, Command Tower, Flooded Strand, Hallowed Fountain, Mana Confluence, Marsh Flats, Misty Rainforest, Overgrown Tomb, Plateau, Polluted Delta, Sacred Foundry, Scrubland, Steam Vents, Stomping Ground, Strip Mine, Taiga, Temple Garden, Tropical Island, Tundra, Underground Sea, Urza's Saga, Verdant Catacombs, Volcanic Island, Watery Grave, Windswept Heath, Wooded Foothills, 1 Forest

## Import list (100)

```
1 Ancient Tomb
1 Arcane Signet
1 Astral Dragon
1 Badlands
1 Basalt Monolith
1 Bayou
1 Birds of Paradise
1 Blood Crypt
1 Bloodstained Mire
1 Breeding Pool
1 Chaos Warp
1 Chromatic Lantern
1 Chrome Mox
1 City of Brass
1 Coalition Relic
1 Command Tower
1 Crucible of Worlds
1 Cursed Mirror
1 Decimate
1 Deflecting Swat
1 Demonic Tutor
1 Disdainful Stroke
1 Elesh Norn, Mother of Machines
1 Enlightened Tutor
1 Expedition Map
1 Flooded Strand
1 Force of Negation
1 Force of Will
1 Gilded Lotus
1 Glorybringer
1 Goldspan Dragon
1 Grim Monolith
1 Hallowed Fountain
1 Hangarback Walker
1 Imperial Seal
1 Jeska's Will
1 Karn, Scion of Urza
1 Lightning Greaves
1 Mana Confluence
1 Mana Drain
1 Mana Vault
1 Marsh Flats
1 Meteor Golem
1 Mind Stone
1 Misty Rainforest
1 Mox Diamond
1 Mox Jasper
1 Mox Opal
1 Mystic Forge
1 Nature's Lore
1 Nicol Bolas, the Ravager
1 Overgrown Tomb
1 Pact of Negation
1 Plateau
1 Polluted Delta
1 Ragavan, Nimble Pilferer
1 Red Elemental Blast
1 Rhystic Study
1 Rings of Brighthearth
1 Sacred Foundry
1 Scion of Draco
1 Scrubland
1 Smothering Tithe
1 Sol Ring
1 Solemn Simulacrum
1 Steam Vents
1 Steel Hellkite
1 Stomping Ground
1 Strip Mine
1 Summon: Bahamut
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
1 Thundermaw Hellkite
1 Tropical Island
1 Tundra
1 Underground Sea
1 Urza, Lord High Artificer
1 Urza's Bauble
1 Urza's Saga
1 Utvara Hellkite
1 Vampiric Tutor
1 Verdant Catacombs
1 Volcanic Island
1 Walking Ballista
1 Watery Grave
1 Wayfarer's Bauble
1 Windswept Heath
1 Wooded Foothills
1 Worldly Tutor
1 Wrenn and Six
1 Forest
```
