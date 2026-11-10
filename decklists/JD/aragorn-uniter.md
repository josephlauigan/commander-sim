# Aragorn, the Uniter — Four-Colour Spell Burn

A four-colour spell deck. Aragorn makes every spell you cast do something: a red spell is 3 damage to an opponent, a
white spell is a 1/1 Soldier, a blue spell is scry 2, a green spell is +4/+4. So the deck is full of cheap spells:
rituals, cantrips, burn, free counters. It wins three ways: an **Underworld Breach storm** (Brain Freeze, Grapeshot), a
**Jeska's Will turn** into Comet Storm or a pile of red spells, and **Fiery Emancipation**, which triples every point of
damage it deals. In between it grinds with Niv-Mizzet, Talrand, Elspeth and Rhystic Study, and holds the board with
three sweepers and two attack taxes. It is slow (a goldfish kill takes until about turn 12), it needs four colours, and
it is weak to graveyard hate, one-spell-per-turn effects, Blood Moon and Back to Basics.

| | |
|---|---|
| **Commander** | Aragorn, the Uniter ({R}{G}{W}{U}, 5/5, Legendary Creature — Human Noble) |
| **Bracket** | 4 (11 Game Changers, see Bracket) |
| **Simulator** | Deck key `aragorn`, tiers T4-T5, played from `data/plans/aragorn.toml`; findings in `audit/aragorn/FINDINGS.md`, history in `decklists/CHANGELOG.md` |
| **Spells** | 64 (red 23, blue 19, green 11, white 14, colourless 6; 9 multicolour), average mana value 2.69 |

## Strategy

### The commander

**Aragorn, the Uniter** ({R}{G}{W}{U}, 5/5) reads: *Whenever you cast a white spell, create a 1/1 white Human Soldier
creature token. Whenever you cast a blue spell, scry 2. Whenever you cast a red spell, Aragorn deals 3 damage to target
opponent. Whenever you cast a green spell, target creature gets +4/+4 until end of turn.*

He is the engine. Without him the deck is a pile of cheap spells with a few pingers.

- **Cast triggers only, and only while he is on the battlefield.** Casting Aragorn himself triggers nothing. Copies
  (Grapeshot's and Brain Freeze's storm copies) don't trigger him. Every *cast* does, however it is paid for: free
  Fierce Guardianship and Deflecting Swat (a commander is out), Force of Will paid with a pitched card, Triple Triad's free
  plays, escape (Underworld Breach), flashback and rebound (Ephemerate).
- **A multicolour spell triggers once for each colour.** Twinned Vision ({1}{U/R}), Expressive Iteration and Third Path
  Iconoclast are blue and red (scry 2 and 3 damage). Manamorphose ({1}{R/G}) is red and green (3 damage and +4/+4). Kitchen
  Finks, Faeburrow Elder and Aura Shards are green and white (a +4/+4 and a Soldier).
- **Veyran** (Voice of Duality) doubles all four triggers on instants and sorceries: a red instant is 6 damage, a white
  one makes two Soldiers. **Fiery Emancipation** triples the damage: 9 per red spell, 18 with Veyran on an instant or sorcery.
- **Spark Double** ({3}{U}) enters as a copy of Aragorn, with a +1/+1 counter and not legendary. Two Aragorns means
  every spell triggers twice, and the second one is the backup when the first dies. It copies Niv-Mizzet, Talrand,
  Guttersnipe and Elspeth the same way.
- **His damage hits one opponent.** In a three-opponent game, Aragorn alone kills one player at a time. Firebrand
  Archer, Guttersnipe, Comet Storm and Grapeshot's copies reach the others.
- **Defence.** The white Soldiers chump-block ground creatures. Skullclamp turns each into two cards; Aura Shards turns
  each into an artifact or enchantment kill.
- **He is the card opponents remove most** (about 0.3 per game in the sims). Protect him: see "Protecting Aragorn and
  the burn turn".

**When to cast him.** Turn 3 if you can make {R}, {G}, {W} and {U} (a mana creature, Arcane Signet or Lotus Petal
supplies the missing colour), otherwise turn 4; he resolved on a median of turn 4 in the sims. Against a blue opponent
with two open mana, don't cast him bare: keep a counter for their counter (Force of Will, Counterspell, Mana Drain,
Swan Song, or Fierce Guardianship for {2}{U}, since a commander in the command zone doesn't make it free), or bait with Rhystic Study or
Smothering Tithe first. Each recast from the command zone costs {2} more.

**Faeburrow Elder** ({1}{G}{W}) with Aragorn out taps for four mana (one for each colour among your permanents: red,
green, white, blue) and is a 4/4. It is the best mana creature after him.

### How the deck wins

#### Line 1: Underworld Breach storm

*Underworld Breach* ({1}{R}, enchantment): every nonland card in your graveyard has escape; the escape cost is the
card's mana cost plus exiling three other cards from the graveyard. At the beginning of the end step, sacrifice it, so the
whole line happens in one main phase. Escaped instants and sorceries go back to the graveyard when they resolve, so they
can be cast again if three more cards are there to exile. Pieces:

| Piece | Cost | What it gives | Net mana | With Aragorn out |
|---|---|---|---|---|
| Seething Song | {2}{R} | {R}{R}{R}{R}{R} | +2 | 3 damage |
| Desperate Ritual | {1}{R} | {R}{R}{R} | +1 | 3 damage |
| Rite of Flame | {R} | {R}{R} | +1 | 3 damage |
| Manamorphose | {1}{R/G} | two mana of any colours, draw a card | 0 | 3 damage and +4/+4 |
| Lotus Petal | {0} | one mana of any colour (sacrifice) | +1 | nothing (colourless), but it counts for storm |
| Frantic Search | {2}{U} | draw two, discard two (fuel), untap up to three lands | 0 | scry 2 |
| Lightning Bolt | {R} | 3 damage | -1 | 3 more damage (6 in all) |
| Brain Freeze | {1}{U} | storm: mills three per copy | -2 | scry 2 |
| Grapeshot | {1}{R} | storm: 1 damage per copy | -2 | 3 damage |

**Go / no-go before you start:**

1. Aragorn is on the battlefield and you can protect him this turn. Without him a Breach turn is only Grapeshot's storm
   count (about 20 damage in the example below, instead of 44).
2. Breach will resolve. It is the one card the table must counter, so hold Fierce Guardianship, Force of Will or a
   Counterspell for it. No tutor finds Breach (it is an enchantment); you dig for it with Expressive Iteration, Fact or
   Fiction, Frantic Search, Rhystic Study and Sylvan Library.
3. Count on paper: the graveyard (the first escape needs the card plus three others, any cards, lands included), the mana,
   and the colours. Brain Freeze needs {U}; the rest is red, and the rituals make red.
4. The table has no graveyard hate on the battlefield: Bojuka Bog, Rest in Peace, Tormod's Crypt, Dauthi Voidwalker (all
   in the t4-t5 decks), and no Drannith Magistrate (it stops casting from the graveyard) or Rule of Law / Eidolon of
   Rhetoric (one spell per turn).
5. Brain Freeze mills three cards per copy: aim it at yourself only if your library holds more than three times the
   copies, otherwise at an opponent.

**Example turn** (Aragorn out, 5 mana, hand: Breach, Brain Freeze, Grapeshot; graveyard of 8 cards including Seething
Song, Desperate Ritual, Rite of Flame, Manamorphose and Lightning Bolt). Exile the junk first and keep the four useful
cards and the storm cards.

| # | Cast | Mana left | Damage |
|---|---|---|---|
| 1 | Underworld Breach | 3 | 3 (Aragorn) |
| 2 | Escape Seething Song | 5 | 3 |
| 3 | Brain Freeze at yourself: 2 spells before it, so 3 copies mill 9 (graveyard 5 to 15) | 3 | 0 (scry 2) |
| 4 | Escape Desperate Ritual | 4 | 3 |
| 5 | Escape Rite of Flame | 5 | 3 |
| 6 | Escape Lightning Bolt | 4 | 6 (3 and 3) |
| 7 | Escape Manamorphose (draw a card) | 4 | 3 |
| 8 | Grapeshot: 7 spells before it, so 8 hits | 2 | 11 (3 and 8) |
| 9 | Escape Grapeshot: 8 spells before it, so 9 hits | 0 | 12 (3 and 9) |

That is **44 damage**, which is one full opponent. It is arithmetic, not a sim result. To end the whole table the turn
needs Veyran (Aragorn's eight triggers double, 24 more), Fiery Emancipation (all of it triples, over 130), Guttersnipe or
Firebrand Archer (they hit every opponent), or opponents already burned down. Aim Aragorn's triggers at one opponent
and split Grapeshot's copies. Storm-Kiln Artist turns every spell cast or copied into a Treasure (two with Veyran), which
pays for more escapes. A counter on Grapeshot or Brain Freeze doesn't stop its copies (storm is a trigger of its own).

The sims show the line works when it comes up: every Breach line the deck ran won, but Breach and a storm card were
held together on its turn in only 6.5% (t4) and 4.6% (t5) of games. Treat it as a bonus when the cards line up.

#### Line 2: Jeska's Will turn

*Jeska's Will* ({2}{R}, sorcery): choose one; if you control a commander as you cast it you may choose both. One mode
adds {R} for each card in target opponent's hand. The other exiles the top three cards of your library and lets you play
them this turn. With Aragorn out, it is both, and the spell itself is a red spell (3 damage, 6 with Veyran).

1. Have Aragorn out (otherwise you pick one mode). Better still, have Guttersnipe, Firebrand Archer, Young Pyromancer,
   Third Path Iconoclast, Storm-Kiln Artist or Talrand out, so each cheap spell pays twice.
2. Target the opponent with the most cards in hand: every card is one red mana.
3. Cast it in your first main phase, with the mana you need left over. The red mana empties at the end of the phase, and
   the exiled cards (lands too) can only be played this turn.
4. Spend it all: Comet Storm, Lightning Bolt, Seething Song and Desperate Ritual for more mana, creatures, and the three
   exiled cards.

*Example.* 7 mana and an opponent with 6 cards in hand: Jeska's Will costs 3, leaves 4, adds 6, so you have 10. Comet
Storm ({X}{R}{R}, multikicker {1}) with X = 6 and two extra targets costs exactly 10: **6 damage to each opponent**, plus
Aragorn's 3 from Jeska's Will and 3 from Comet Storm on one of them. That kills opponents already at 6 or less (12 for
the one Aragorn hits); at full life it needs Emancipation (18 to each) or burned-down opponents.

**Comet Storm** also works on its own at the end of a long game: it is an instant, with X paid in colourless mana (Mana
Drain's mana counts), and kills a table of small creatures or finishes players.

#### Line 3: Fiery Emancipation

*Fiery Emancipation* ({3}{R}{R}{R}, enchantment): if a source you control would deal damage, it deals triple. It needs
six mana with {R}{R}{R} and no tutor finds it, so cast it with a counter backup.

| Source | Normal | With Emancipation |
|---|---|---|
| Aragorn's red-spell trigger | 3 | 9 (18 with Veyran, on an instant or sorcery) |
| Lightning Bolt | 3 | 9 |
| Grapeshot (each copy) | 1 | 3 |
| Firebrand Archer (each noncreature spell, every opponent) | 1 | 3 |
| Guttersnipe (each instant or sorcery, every opponent) | 2 | 6 |
| Niv-Mizzet, Parun (each card drawn) | 1 | 3 |
| Aragorn attacking | 5 | 15 |

With Aragorn, Guttersnipe and Firebrand Archer out, **each red instant costs the table 36 in all**: 18 to the target (9
Aragorn, 6 Guttersnipe, 3 Archer) and 9 to each other opponent, plus the spell itself (a Bolt adds 9). By arithmetic,
three or four red instants kill three opponents at 40 life; with Veyran, two or three. The cheap red spells in the list
are Lightning Bolt, Manamorphose, Desperate Ritual, Seething Song, Rite of Flame, Grapeshot, Deflecting Swat (free),
Twinned Vision and Expressive Iteration. In the sims Fiery Emancipation is among the cards with the highest win rate
when cast (a correlation: it is also cast mostly in games already going well).

Nibelheim Aflame is damage too: Emancipation triples it onto your own creatures as well.

#### The grind (the fair win)

- **Niv-Mizzet, Parun** ({U}{U}{U}{R}{R}{R}, flying, can't be countered): whenever you draw, 1 damage to any target;
  whenever any player casts an instant or sorcery, you draw. Every counterspell, burn spell and cantrip at the table
  pings. Veyran doubles the draws from your own instants. It needs three blue and three red sources and is often stuck in
  hand in the sims (cast in about a third of the games it is drawn); when it is cast it wins far more than average.
- **Talrand, Sky Summoner** ({2}{U}{U}): a 2/2 flying Drake for each instant or sorcery you cast. Drakes fly over
  Soldier-chump decks and carry Aragorn's +4/+4.
- **Elspeth, Sun's Champion**: +1 makes three Soldiers (chump-blockers, Skullclamp fodder); -3 destroys every creature
  with power 4 or more, **including Aragorn and Niv-Mizzet**.
- **Rhystic Study, Smothering Tithe, Sylvan Library, Skullclamp** keep cards and Treasure coming, and **Windborn Muse**
  and **Ghostly Prison** ({2} per attacking creature each) hold the ground while Aragorn's red spells and the pingers
  do the damage. In the sims' elimination counts, trigger damage took the most players (51% at t4, 39% at t5), then
combat (27% and 38%) and burn.
- **Green pump.** Worldly Tutor, Manamorphose, Heroic Intervention and any green creature give +4/+4: use it on an
  unblocked flier (Drake, Restoration Angel, Windborn Muse) at combat.

### Game plan

**Turns 1-3: mana, colours, engines.**
- Play fast lands and basics early and tapped lands (Frontier Bivouac, Jungle Shrine, Mystic Monastery, Seaside
  Citadel, Path of Ancestry) on a turn with nothing to cast. Spirebluff Canal is untapped only with two or fewer other
  lands, Stormcarved Coast with two or more.
- Birds of Paradise, Llanowar Elves, Paradise Druid, Sol Ring, Arcane Signet and Nature's Lore (fetch Taiga, Tropical
  Island or Savannah: they come in untapped, Temple Garden asks for 2 life) are the opening. Rhystic Study on turn 3,
  Sylvan Library or Smothering Tithe when there is a spare turn.
- Cheap engines (Firebrand Archer, Young Pyromancer, Third Path Iconoclast) start the clock. Keep one removal spell
  (Lightning Bolt, Swords, Path) open against t5's combo creatures.

**Turns 4-6: Aragorn, then protect him.** Cast him (see above) and chain cheap spells. Lightning Greaves goes on him
if the table has targeted removal. Keep counters for the turn that matters; hold Restoration Angel and Heroic
Intervention for their removal. Fiery Emancipation and Triple Triad come down when Aragorn is safe.

**Late game: a burn turn.** Jeska's Will, rituals and Comet Storm; Breach if it lines up; Emancipation with any red spells;
or Niv-Mizzet / Talrand taking over a long game.

**Hold up or go?**

- **Hold** (leave Counterspell, Mana Drain or Fierce Guardianship, plus a removal spell, up on their turn) against t5's
  combo decks (Kinnan, Urza, Zur, Yawgmoth) from turn 3: they win on turns 3-6. In the sim traces the deck tapped out or spent its counters
  on small things the turn they won (it equipped Lightning Greaves the turn Zur won, and used Mystic Confluence on a
  one-drop the turn Urza won).
- **Go** when a line is complete and you hold protection for its key spell, when the table is tapped low, or when someone
  at the table will win before your next turn and holding does nothing about it (then sweep, or race with what you have).
- Against t4's go-wide decks (Krenko, Edgar Markov), the right play is often a sweeper, not a threat: Austere Command
  and Nibelheim Aflame are sorceries, so you tap out; do it on the turn it saves you.

### Mulligans

The first mulligan is free (you see two 7-card hands). After that it's a London mulligan: draw 7, put one card on the
bottom for each mulligan after the free one.

| Hand | Keep when |
|---|---|
| 7 cards | 2-4 lands (1 land only with two of Sol Ring, Lotus Petal, Birds of Paradise and Llanowar Elves that the land can cast; 6 or more lands go back); at least two cards that make mana by turn 3 (lands, rocks, mana creatures); red plus at least one more colour available, with the rest reachable by fetches or mana creatures; and two plays before turn 4: a mana piece, Rhystic Study, Sylvan Library, Smothering Tithe, a cheap engine (Firebrand Archer, Young Pyromancer, Third Path Iconoclast) or cheap interaction (Lightning Bolt, Swords to Plowshares, Path to Exile, Counterspell) |
| 6 or 5 cards | 2-4 lands and a mana piece, with one play before turn 4 |
| 4 cards | Keep |

- **Aragorn is always there,** so a hand needs no commander piece. It needs the four colours: count fetches as the
  duals they can find, and check that green and white are reachable (Llanowar Elves and Taiga don't make white).
- **Bottom first:** Elspeth, Austere Command, Triple Triad, Niv-Mizzet and Comet Storm (expensive cards the sims leave
  in hand), then Brain Freeze or Grapeshot with no Breach, a third protection spell, lands past the fourth.
- **Counters alone aren't a hand.** Free counters (Force of Will, Fierce Guardianship, Mana Drain) don't develop the board.

| # | Hand | Call | Why |
|---|---|---|---|
| 1 | Taiga, Steam Vents, Plains, Sol Ring, Birds of Paradise, Rhystic Study, Lightning Bolt | Keep | Three lands that make all four colours with Birds. Sol Ring on turn 1; Birds and Rhystic Study on turn 2 (Sol Ring pays {2} of it); Aragorn on turn 3 (Taiga R, Vents U, Plains W, Birds G) |
| 2 | Island, Volcanic Island, Mystic Sanctuary, Steam Vents, Spirebluff Canal, Stormcarved Coast, Grapeshot | Ship | Six lands and one spell; no green or white source, nothing to do |
| 3 | Island, Sol Ring, Mana Drain, Counterspell, Force of Will, Fierce Guardianship, Swan Song | Ship | One land, and the rest is a counter suite with no engine or mana |
| 4 | Tropical Island, Plateau, Command Tower, Nature's Lore, Young Pyromancer, Skullclamp, Counterspell | Keep | Nature's Lore fetches Taiga or Savannah on turn 2 for the fourth colour; Aragorn on turn 3 from four lands; Young Pyromancer and Skullclamp are a cheap engine |
| 5 | Volcanic Island, Mystic Monastery, Birds of Paradise, Underworld Breach, Seething Song, Brain Freeze, Grapeshot | Keep (close) | The whole Breach line is in hand, and Birds makes three mana on turn 2. It needs Aragorn, more lands and a full graveyard first, which is why it is close |
| 6 | Sunpetal Grove, Spirebluff Canal, Arcane Signet, Faeburrow Elder, Rhystic Study, Fierce Guardianship, Lightning Bolt | Keep | Two lands and a Signet make three mana on turn 3 (Grove G, Signet W, Canal): Faeburrow Elder or Rhystic Study; Bolt answers Kinnan or Yuriko |
| 7 | Sunpetal Grove, Taiga, Temple Garden, Tundra, Island, Elspeth, Sun's Champion, Austere Command | Ship | Five lands and two six-mana cards: nothing to cast before turn 6, and the sims show the deck loses by round 8 or 9 |
| 8 | Taiga, Steam Vents, Llanowar Elves, Young Pyromancer, Grapeshot, Desperate Ritual, Austere Command | Keep, bottom Austere Command | Six-card hand: two lands and Llanowar Elves on turn 1, Young Pyromancer on turn 2. Austere Command is the six-mana card |

### What to tutor for

The deck has two true tutors (**Mystical Tutor**, **Worldly Tutor**), a land tutor (**Nature's Lore**), an artifact
tutor (**Tezzeret, Cruel Captain**), a graveyard-to-library land (**Mystic Sanctuary**) and a set of dig spells that draw
or look at cards without choosing one. This section lists each of them, the rules that decide when a found card is
usable, and then what to find for each board state.

#### Every tutor and finder

**Tutors (they name a card):**

| Card | Cost | Where the card goes | What it finds | Speed | Catch |
|---|---|---|---|---|---|
| Mystical Tutor | {U} | Top of the library (revealed, library shuffled) | An instant or sorcery | Instant | You draw it at your next draw, not now, and the table sees it. A blue spell: with Aragorn out his scry 2 resolves first and the shuffle undoes it |
| Worldly Tutor | {G} | Top of the library (revealed, library shuffled) | A creature | Instant | Same timing. A green spell: with Aragorn out its +4/+4 trigger is real (pump an unblocked flier). The creature is then cast at sorcery speed, except Restoration Angel (flash) |
| Nature's Lore | {1}{G} | Battlefield, untapped | A Forest card: Taiga ({R}/{G}), Savannah ({G}/{W}), Tropical Island ({G}/{U}), Temple Garden ({G}/{W}), Forest | Sorcery | Ramp and colour fixing in one card. Temple Garden asks for 2 life or enters tapped. A green spell (+4/+4 trigger with Aragorn out) |
| Tezzeret, Cruel Captain | {3} | Hand | An artifact with mana value 1 or less: Sol Ring, Lotus Petal, Skullclamp | Sorcery; loyalty 4, the -3 leaves 1 | Colourless, so no Aragorn trigger. Each artifact entering adds a loyalty counter. His 0 untaps an artifact or creature (Sol Ring or a mana creature) for extra mana |
| Mystic Sanctuary | Land | Top of the library, no shuffle | An instant or sorcery in your graveyard | When it enters untapped | Enters untapped only with three other Islands: Island, Volcanic Island, Steam Vents, Tropical Island and Tundra count, and a fetch land can bring it in. It is your land drop. The card goes on top of whatever is there, so a card you tutored sits one deeper |

**Digs (they look at cards or draw them, you don't name one):**

| Card | Cost | What it does | Speed | Catch |
|---|---|---|---|---|
| Expressive Iteration | {U}{R} | Look at the top three: one to hand, one to the bottom, one exiled that you may play this turn | Sorcery | Keep mana for the exiled card or it is lost. With Aragorn out it is two triggers: scry 2 resolves first and sets up the three cards, and 3 damage |
| Frantic Search | {2}{U} | Draw two, discard two, untap up to three lands | Instant | Pays for itself when three lands were tapped for it. You choose the discards, which fill the graveyard for Breach |
| Fact or Fiction | {3}{U} | Reveal five; an opponent splits them into two piles; you take one pile and the other goes to your graveyard | Instant | They choose the split. The graveyard pile fuels Breach |
| Mystic Confluence | {3}{U}{U} | Choose three: draw a card, bounce a creature, or counter unless they pay {3} (modes can repeat) | Instant | Five mana, but it is also a counter |
| Twinned Vision | {1}{U/R} | Draw a card; draw two if it wasn't cast from your hand | Instant | Flashback {1}{U/R}{U/R} and discard a card, or escape from Underworld Breach, draws two. Blue and red |
| Manamorphose | {1}{R/G} | Two mana of any colours, draw a card | Instant | Net zero mana. Red and green |
| Jeska's Will | {2}{R} | Exile the top three; you may play them this turn (or add red for their hand size; both modes with a commander) | Sorcery | Cards you don't play are gone; a land only if you have a land drop left |
| Sylvan Library | {1}{G} | Each draw step: draw two extra cards, then put two back or pay 4 life each | Each turn | Enchantment: you see three cards a turn. A tutored card is the first of the three |
| Skullclamp | {1}, equip {1} | Draw two when the equipped creature dies | Any time you can equip | A 1/1 token with it dies at once: two cards for {1} |
| Rhystic Study, Niv-Mizzet, Aragorn | | Rhystic Study draws on their spells unless they pay {1}. Niv-Mizzet draws whenever any player casts an instant or sorcery. Aragorn scries 2 for each blue spell you cast | Passive | Card flow that doesn't pick |

**Faeburrow Elder** finds nothing; it is the mana answer (see the mana rows). The key cards no tutor can find:
Underworld Breach, Fiery Emancipation, Triple Triad, Rhystic Study, Aura Shards, Ghostly Prison, Smothering Tithe and Sylvan
Library (enchantments), Lightning Greaves and Arcane Signet (mana value 2), Elspeth (a planeswalker). Dig for them.

#### Rules that decide timing

- **A tutored card is on top, so you draw it next.** Cast Mystical Tutor or Worldly Tutor at the end step of the player
  before you and the card is in your hand for your turn, with your mana free. Cast on your own turn, it arrives a full
  turn later unless you draw it now.
- **Draw it now.** After a tutor, any spell that looks at or draws the top card gets it this turn: Expressive Iteration
  (top three), Frantic Search, Twinned Vision, Manamorphose, Mystic Confluence (draw), Jeska's Will (exile three, play them)
  and Fact or Fiction (top five, but the opponent splits the piles). With Frantic Search the tutored card is the first
  of the two you draw; discard the other cards.
- **A second tutor undoes the first.** Both tutors shuffle before putting the card on top. Cast only one before you draw.
- **Aragorn's scry 2 and a tutor.** A blue spell's trigger resolves before the spell, so scry 2 before a tutor is lost to
  the shuffle. After a tutor, a blue spell's scry 2 looks at the tutored card: keep it on top.
- **Tutors are revealed.** The table sees what you took and can answer it (hold a counter, kill Aragorn).
- **Burst mana empties at the end of each step and phase.** Spend it in the phase you make it. See the next table.
- **Opposition Agent** takes every card you search for. Kill it before you tutor or crack a fetch land.

#### Burst mana and what it can pay

| Source | Cost | Makes | Net | Pays for |
|---|---|---|---|---|
| Rite of Flame | {R} | {R}{R} | +1 | Red only |
| Desperate Ritual | {1}{R} | {R}{R}{R} | +1 | Red only |
| Seething Song | {2}{R} | {R}{R}{R}{R}{R} | +2 | Red only |
| Manamorphose | {1}{R/G} | Two mana of any colours | 0, and draws a card | Any colour: fixes blue for Mystical Tutor |
| Lotus Petal | {0} | One mana of any colour | +1 once | Any colour |
| Jeska's Will | {2}{R} | {R} for each card in an opponent's hand | Hand size minus 3 | Red only |
| Mana Drain | {U}{U} | {C} for the countered spell's mana value, in your next main phase | | Colourless: Comet Storm's X, generic costs |

Red-only mana pays Emancipation's {R}{R}{R}, Comet Storm, Grapeshot, Breach and escapes, and every generic cost. It can't
pay the {U} of Mystical Tutor, Brain Freeze or Swan Song, the {G} of Worldly Tutor, or Niv-Mizzet's {U}{U}{U}.

#### What each spell colour does with Aragorn out

Spells cast before Aragorn is on the battlefield trigger nothing, so keep cheap spells in hand until he resolves
(mana and colour pieces excepted).

| Spell colour | Trigger | Best use | Spells that give it |
|---|---|---|---|
| Red | 3 damage to target opponent | Reach and finishing: count red spells | Lightning Bolt, Seething Song, Desperate Ritual, Rite of Flame, Jeska's Will, Comet Storm, Grapeshot, Chaos Warp, Deflecting Swat, Nibelheim Aflame |
| White | A 1/1 Soldier | Chump-blockers, Skullclamp fodder, Aura Shards triggers | Swords, Path, Ephemerate, Teferi's Protection, Austere Command, Restoration Angel, Skyclave Apparition, Windborn Muse |
| Blue | Scry 2 | Digs toward the next piece; best before a draw or look-at-the-top spell | Counters, Frantic Search, Fact or Fiction, Mystic Confluence, Cyclonic Rift, Spark Double |
| Green | Target creature +4/+4 | An unblocked flier at combat | Worldly Tutor, Heroic Intervention, Nature's Lore, Sylvan Library, Birds of Paradise, Llanowar Elves, Paradise Druid |
| Two colours | Both triggers | The cheapest way to get two triggers from one card | Twinned Vision, Expressive Iteration, Third Path Iconoclast (blue and red); Manamorphose (red and green); Kitchen Finks, Faeburrow Elder, Aura Shards (green and white) |
| Colourless | None | Still a spell cast for storm and for Firebrand Archer | Lotus Petal, Sol Ring, Skullclamp, Arcane Signet, Lightning Greaves, Tezzeret |

#### By board state

Each row names the board, the window and the one tutor to use. "End step" means the end step of the turn before the one
you need the card. Mana is what you will have when the card is cast.

**Aragorn is not cast yet.** He needs {R}{G}{W}{U}; casting him triggers nothing.

| Your board and mana | Opponents' board | Window | Tutor | Target | Why and what to watch for |
|---|---|---|---|---|---|
| Two lands, one of Aragorn's four colours out of reach | Nothing that stops lands | Turn 2 | Nature's Lore | Taiga for red, Tropical Island for blue, Savannah for white (Temple Garden also, for 2 life); any Forest source for green | The land is untapped, so Aragorn can land on turn 3 from four lands. A fetch land in hand does the same job and keeps the tutor |
| No mana creature or rock, nothing else to cast, {G} open | No pressure | End step of turn 1 | Worldly Tutor | Birds of Paradise (any colour), else Paradise Druid; Llanowar Elves makes only {G} | A last resort: a premium tutor spent on mana. Prefer Nature's Lore when you hold it |
| Four sources of all four colours ready for next turn; {U} spare | Blue mana open, counters likely | End step, turns 4-5 | Mystical Tutor | Swan Song ({U}: counters their counterspell, they get a 2/2 Bird), or Force of Will if you hold a spare blue card | Swan Song and Fierce Guardianship can't counter a creature spell, but both can counter their counter. Fierce Guardianship costs {2}{U} until a commander is on the battlefield. Counterspell or Mana Drain need {U}{U} left over |
| Three mana, Tezzeret in hand, nothing better | Nothing | Turn 3 | Tezzeret (-3) | Lotus Petal if a colour is missing for Aragorn next turn; Sol Ring if only generic mana is short | Sol Ring makes {C}{C} and can't pay Aragorn's colours. The artifact entering adds a loyalty counter |
| Cheap red and blue spells in hand | Any | Before Aragorn | None | Hold them | A spell cast before he resolves does nothing; after, a red spell is 3 damage |

**Aragorn is on the battlefield.**

| Your board and mana | Opponents' board | Window | Tutor | Target | Why and what to watch for |
|---|---|---|---|---|---|
| Aragorn out, one opponent in burn range, {R} spare | Low life | End step, then your turn | Mystical Tutor | Lightning Bolt: 3 from Aragorn and 3 from the Bolt for {R} | With Emancipation each is 9 |
| Aragorn out, 8 or more mana | Several opponents low | End step, then your turn (instant) | Mystical Tutor | Comet Storm: {X}{R}{R}, {1} more for each extra target | X goes to every target; Aragorn's 3 hits one opponent. Mana Drain's {C} pays X |
| Aragorn out, 7 or more mana next turn | An opponent holding five or more cards | End step | Mystical Tutor | Jeska's Will | With a commander you take both modes: {R} for each card in their hand and three exiled cards to play, plus 3 from Aragorn. Cast it first in the main phase; the red empties at end of phase |
| Aragorn out, an unblocked flier (Drake, Windborn Muse, Restoration Angel, Birds of Paradise) in combat | Attackers declared or no blockers | Combat | Worldly Tutor | Any creature, as a cast: the +4/+4 is the point | Cast after blockers. Heroic Intervention or Manamorphose also give +4/+4. Shroud (Greaves) on Aragorn means pump another creature |
| Aragorn out, empty hand, 3 to 5 mana | Any | Your turn | A dig, not a tutor | Expressive Iteration, Twinned Vision, Frantic Search, Fact or Fiction | Scry 2 resolves first, so the cards you see are better. Nothing here names a card |
| Aragorn and Aura Shards out | A lock or combo artifact or enchantment: Blood Moon, Back to Basics, Winter Orb, Rule of Law, Isochron Scepter | End step | Worldly Tutor | Kitchen Finks ({1}{G/W}{G/W}) | It makes a Soldier (white) and pumps (green); the Soldier entering and Kitchen Finks entering each trigger Aura Shards: two destroys for one spell |
| Aragorn out, Soldiers or other tokens on your side | Any | Your turn | Tezzeret (-3) | Skullclamp | Each token is two cards for {1} (equip {1}); a Soldier with it dies at once |
| Aragorn out, you want a second engine body | Removal likely | End step | Worldly Tutor | Spark Double ({3}{U}) | It enters as a copy of Aragorn, non-legendary, so both stay and every spell triggers twice. Blue spell: scry 2 |

**Underworld Breach.** Breach ({1}{R}) gives every nonland card in your graveyard escape for its mana cost plus exiling
three other cards from the graveyard, and is sacrificed at the end step. For the count: one escape needs 4 cards (the
card and three to exile); an instant or sorcery returns to the graveyard when it resolves, so each further escape
costs three more cards net. 7 cards is two escapes, 10 is three, before Brain Freeze or discards add more.
**Grapeshot** deals 1 damage per copy, and the storm count is every spell cast before it this turn (escaped casts,
Breach, Lotus Petal and cantrips all count), so a count of n is n + 1 damage, tripled by Fiery Emancipation, plus
Aragorn's 3 for each red spell. **Brain Freeze** mills three cards per copy, so a count of n mills 3(n + 1). Aim it at
yourself to refill the graveyard if your library holds more than three times the copies; at an opponent only if their
library is short enough to finish. Without Aragorn, the 44-damage example above is about 20.

| Your board and mana | Opponents' board | Window | Tutor | Target | Why and what to watch for |
|---|---|---|---|---|---|
| Breach in hand, Aragorn out, 7 or more cards in the graveyard, no storm card | No graveyard hate, no one-spell limit | End step | Mystical Tutor | Grapeshot | The default payoff: each copy can pick a different target. A sorcery, so your turn only |
| Breach and Grapeshot in hand, graveyard of 3 or fewer | Any | Your turn | A dig, not a tutor | Frantic Search (discard two), Fact or Fiction (the other pile), Twinned Vision flashback | Starting with a short graveyard wastes the line; fill it first |
| Breach, Grapeshot and Aragorn, mana short | Any | End step | Mystical Tutor | Seething Song (+2), else Desperate Ritual or Rite of Flame (+1) | All red. A ritual in the graveyard can be escaped again but costs three cards |
| Mid-chain, Aragorn out, the next piece is in the library | Any | Your turn, mid-chain | Mystical Tutor, then a cantrip | Grapeshot or Seething Song, drawn with Manamorphose, Twinned Vision or Frantic Search | Both are spells, so both raise the storm count. Mystical Tutor needs {U}, which rituals can't give |
| Breach turn next, 3 or more opponents | Several players alive | End step before | Worldly Tutor | Guttersnipe: 2 to each opponent for each instant or sorcery you cast | Aragorn hits one opponent; an eight-spell chain is 16 to each. Storm copies are not casts |
| Breach turn next, mana-hungry chain | Any | End step before | Worldly Tutor | Storm-Kiln Artist ({3}{R}) | A Treasure for each instant or sorcery cast or copied, Grapeshot's copies included. It dies to removal; one pays for a ritual |
| Emancipation out or castable | Any | End step | Mystical Tutor | Lightning Bolt (9, plus Aragorn's 9), or Comet Storm with mana | Emancipation triples every source you control. No tutor finds Emancipation itself; cast it with a counter ready |
| Niv-Mizzet out | Players casting instants and sorceries | End step | Mystical Tutor | Mystic Confluence | Casting any instant or sorcery draws a card and pings; Confluence's three draw modes are three more pings (Frantic Search draws two). Fact or Fiction puts cards in hand without drawing. Niv-Mizzet needs {U}{U}{U}{R}{R}{R} to cast |

**An opponent is about to win.** A tutored card is on top and arrives at your next draw. That helps only if the combo
needs another turn, or if you draw it now with an instant (Mystical Tutor, then Frantic Search, which also untaps three
lands to pay for the counter you find). Otherwise hold the counters you already have.

| Their key piece | Counter it | Remove or answer it on the battlefield | Notes |
|---|---|---|---|
| A creature (Kinnan, Urza, Yawgmoth, Zur) | Counterspell, Mana Drain, Force of Will | Swords or Path (instant, Mystical Tutor finds them), Lightning Bolt for 3 toughness or less, Skyclave Apparition (Worldly Tutor; mana value 4 or less, which covers all four) | Fierce Guardianship and Swan Song can't hit a creature spell. Tutor at the end step, use the card next turn |
| An artifact (Basalt Monolith, Isochron Scepter, Winter Orb) | Fierce Guardianship (free with Aragorn out), Counterspell, Mana Drain, Force of Will | Cyclonic Rift, Chaos Warp, Skyclave Apparition, Aura Shards | Rift the Monolith before it untaps |
| An enchantment (Aluren, Intruder Alarm, Rule of Law, Blood Moon, Back to Basics) | Swan Song ({U}), Fierce Guardianship, the rest | Cyclonic Rift, Chaos Warp, Skyclave Apparition, Aura Shards, Austere Command | Swan Song gives them a 2/2 Bird |
| An instant or sorcery (Dramatic Reversal, Toxic Deluge, Damnation) | Swan Song, Fierce Guardianship, the rest | None: it is a spell | Supreme Verdict can't be countered: use Heroic Intervention or Teferi's Protection |
| A damage or drain win aimed at you (Walking Ballista pings, a Craterhoof attack, a drain) | Counter the enabler if it is a spell | Teferi's Protection ({2}{W}) | Your life total can't change and you have protection from everything, but it is no answer to a library or alternate win, and your permanents (lands too) phase out: spend your mana first |
| Their counterspell is up against your key spell | Find Force of Will (needs a spare blue card), Fierce Guardianship (noncreature, free with Aragorn out), or Swan Song | Bait with a cheaper spell first | Counters on Breach, Emancipation or Aragorn are the ones that cost the game |

Walking Ballista has mana value 0 on the battlefield, so Skyclave Apparition can exile it.

**A go-wide board (Krenko, Edgar Markov).**

| Your board and mana | Opponents' board | Window | Tutor | Target | Why and what to watch for |
|---|---|---|---|---|---|
| Six mana, tapping out is safe | Many cheap creatures | End step, then your turn (sorcery) | Mystical Tutor | Austere Command: "mana value 3 or less" plus artifacts or enchantments if useful | Never choose "mana value 4 or greater" with Aragorn out (mana value 4); the 3-or-less mode kills your Soldiers, and the artifact and enchantment modes destroy your own Sol Ring, Arcane Signet, Greaves, Skullclamp and enchantments |
| Four mana, Aragorn out | Many creatures with toughness 5 or less | End step, then your turn (sorcery) | Mystical Tutor | Nibelheim Aflame: Aragorn deals 5 to each other creature (9 after a green spell) | Aragorn survives (he is the source); your other creatures die. A red spell, so 3 more to an opponent |
| Four or more mana, they attack with tokens | Tokens with 1 toughness | End step, then their combat (instant) | Mystical Tutor | Comet Storm with X of 1 and a target for each token | Costs X + 2 plus {1} for each extra target, so X of 1 and n targets is n + 2 mana. It also hits players |
| Seven mana, or Seething Song | A wide board | End step, then their end step or declare attackers | Mystical Tutor | Cyclonic Rift, overloaded ({6}{U}) | One-sided: every nonland permanent they control returns to hand, and tokens vanish. The best answer to a swarm |

**Removal on your commander.** Pick by what they cast. Ephemerate and Restoration Angel save him only from targeted
removal: he returns before a wipe resolves and dies anyway.

| Your board and mana | Opponents' board | Window | Tutor | Target | Why and what to watch for |
|---|---|---|---|---|---|
| Aragorn out, tapped out | Targeted removal (Swords, Terminate, Fatal Push, Heartless Act, Anguished Unmaking) | End step | Mystical Tutor | Fierce Guardianship (counter the spell, free) or Deflecting Swat (redirect, free, red: 3 damage) | Both are free with a commander on the battlefield. Force of Will is free with a spare blue card |
| Aragorn out, {1}{G}, {W} or {2}{W} spare | Targeted removal | End step | Mystical Tutor | Heroic Intervention (hexproof), Ephemerate ({W}, rebound blinks him again next upkeep), or Teferi's Protection | The white spell triggers Aragorn for a Soldier. Heroic Intervention makes any targeted removal fizzle |
| Aragorn out, 4 mana held | Targeted removal | End step | Worldly Tutor | Restoration Angel (flash) | Blink in response; a 3/4 flier stays |
| Aragorn out | A destroy wipe (Damnation, Supreme Verdict, Damn overload, Blasphemous Act) | End step | Mystical Tutor | Heroic Intervention (indestructible) or Teferi's Protection | Supreme Verdict can't be countered. Toxic Deluge and The Meathook Massacre shrink creatures: Heroic Intervention doesn't help, Teferi's Protection phases him out, a counter stops it |
| Aragorn dead or in the command zone, tax too high | Any | End step | Worldly Tutor | Talrand, Guttersnipe, Veyran or Young Pyromancer | Keeps the spell plan going. Recasting costs {2} more for each earlier cast; Spark Double copies whatever creature remains |

**Needing mana.**

| Your board and mana | Opponents' board | Window | Tutor | Target | Why and what to watch for |
|---|---|---|---|---|---|
| Aragorn out, 4 to 5 mana, the big turn needs 8 or more | Any | End step | Worldly Tutor | Faeburrow Elder ({1}{G}{W}) | With Aragorn it taps for {R}{G}{W}{U}, four mana, and is a 4/4. Lightning Greaves (equip {0}) gives it haste so it taps the turn it lands. Casting it triggers Aragorn twice (Soldier and +4/+4) |
| Aragorn out, plenty of lands, short of red | Any | End step | Mystical Tutor | Seething Song for Emancipation or Comet Storm, Jeska's Will if they hold cards | Red-only mana. Don't count on it for blue spells |
| Colours short on turn 3 or 4 | Any | Turn 2-4 | Nature's Lore, then Tezzeret | The missing colour, then Lotus Petal | A fetch land that finds the dual does the same without a tutor |

**A creature for Worldly Tutor.** Choose the creature you can cast the turn you draw it.

| Need | Creature | Why |
|---|---|---|
| Mana and colours | Birds of Paradise, Paradise Druid ({1}{G}, hexproof while untapped) | Any colour. Llanowar Elves makes only {G} |
| A lock or combo piece of mana value 4 or less | Skyclave Apparition ({1}{W}{W}) | Exiles Kinnan, Urza, Yawgmoth, Zur, Rule of Law, Eidolon of Rhetoric, Blood Moon, Back to Basics, Winter Orb, Narset, Drannith Magistrate, Dauthi Voidwalker, Aluren, Intruder Alarm, Opposition Agent. When it leaves, the owner gets an X/X token. Not a land |
| Protection for Aragorn | Restoration Angel ({3}{W}, flash) | Held up with four mana |
| A backup engine | Spark Double ({3}{U}) | Second Aragorn |
| Damage to every opponent | Guttersnipe ({2}{R}), Firebrand Archer ({1}{R}: 1 per noncreature spell) | Aragorn hits one opponent |
| Mana and Treasures in a chain | Storm-Kiln Artist ({3}{R}) | A Treasure per instant or sorcery |
| A doubled turn | Veyran ({1}{U}{R}) | Doubles Aragorn, Guttersnipe, Pyromancer and Archer triggers from your instants and sorceries |
| Value in a long game | Talrand ({2}{U}{U}), Young Pyromancer ({1}{R}), Third Path Iconoclast ({U}{R}) | A body per spell; Third Path Soldiers are artifacts (Storm-Kiln Artist gets +1/+0 for each) |
| Defence | Windborn Muse ({3}{W}, 2/3 flier, {2} per attacker), Kitchen Finks | Taxes attackers on top of Ghostly Prison |
| The big finisher | Niv-Mizzet, Parun | Only with three blue and three red sources |

**By turn window.**

| Turns | Mystical Tutor | Worldly Tutor | Nature's Lore, Tezzeret |
|---|---|---|---|
| 1-3 | Hold it, except a counter for Aragorn against open blue mana | Mana creature only if the hand is stuck | Nature's Lore on turn 2 for the missing colour; Tezzeret only with nothing else to do |
| 4-6 | Counters and protection, or Jeska's Will if Aragorn is out and the plan is next turn | Skyclave Apparition for a lock or combo piece; Restoration Angel; Spark Double | Fetch the last colour; Petal for Aragorn's colours |
| 7 and later | The finishing spell: Grapeshot, Bolt, Comet Storm, Jeska's Will, Cyclonic Rift | Guttersnipe, Storm-Kiln Artist, Spark Double | Mystic Sanctuary returns a used key spell to the top |

#### What to fetch first

1. **Stop an immediate loss.** The counter for their combo (Swan Song, Fierce Guardianship, Force of Will,
   Counterspell, Mana Drain), or Skyclave Apparition on the lock piece, or Teferi's Protection against a drain.
2. **Protect Aragorn before the burn turn.** Fierce Guardianship, Deflecting Swat, Heroic Intervention or Restoration Angel.
3. **Complete the kill.** Breach turn: Grapeshot, then a ritual if mana is short; Emancipation out: Lightning Bolt; a
   full-handed opponent: Jeska's Will.
4. **Fix mana.** Nature's Lore for the missing colour, then Worldly Tutor for Birds of Paradise only when the hand has
   nothing else.
5. **Rebuild.** Spark Double or Guttersnipe after Aragorn is gone.

For the lands and rocks: fetch the colour you cannot make now. Tezzeret finds Lotus Petal for a missing colour, Sol Ring
for generic mana, Skullclamp for tokens.

#### Common tutor mistakes

1. Tutoring on your own turn for a card you need this turn, with no draw spell to get it.
2. Casting a second tutor before drawing the first: it shuffles the first card away.
3. Counting on Aragorn's scry 2 before a tutor: the shuffle undoes it.
4. Fetching Swan Song or Fierce Guardianship against a creature combo piece. Swan Song hits only enchantments, instants
   and sorceries; Fierce Guardianship only noncreature spells.
5. Looking for Underworld Breach or Fiery Emancipation with a tutor. Neither can be found; dig for them.
6. Fetching a creature you can't cast on the turn you draw it, such as Niv-Mizzet without three blue and three red sources.
7. Paying for Mystical Tutor, Brain Freeze or Worldly Tutor with ritual mana: it is red only.
8. Spending Mystical Tutor early on a random cheap spell when a counter for Aragorn was the card to hold it for.
9. Taking Sol Ring with Tezzeret when the shortage is a coloured mana.
10. Fetching Temple Garden with Nature's Lore at low life, or a dual that duplicates a colour you already have.
11. Tutoring or cracking a fetch land while Opposition Agent is on the battlefield.
12. Choosing "mana value 4 or greater" with Austere Command and killing Aragorn, or tapping out for the sweeper into a
    counter.

### Protecting Aragorn and the burn turn

Protect in this order, and before the first spell of the turn:

1. **Lightning Greaves** ({2}, equip {0}): shroud and haste. Equip him on the turn before the burn turn. It stops
   targeted removal and Swords, not sweepers. **Shroud also stops his own green pump:** the +4/+4 trigger can't target
   him, so pump another creature.
2. **Free counters:** Fierce Guardianship (noncreature spells; free with Aragorn out), Force of Will (any spell), Deflecting
   Swat (free with a commander: change the target of a removal spell or ability, ideally onto their own creature).
3. **Reactive protection:** Heroic Intervention ({1}{G}: hexproof and indestructible for your permanents: beats Damnation,
   Supreme Verdict, Damn, Blasphemous Act and Ruinous Ultimatum; not Toxic Deluge or The Meathook Massacre, which shrink),
   Ephemerate or Restoration Angel in response to a targeted spell (Aragorn leaves and returns, the spell fizzles),
   Teferi's Protection ({2}{W}: protection from everything and your permanents phase out; best at their turn or in
   response to a wipe; you can't pay life while it lasts, so no fetch land or Force of Will's life payment).
4. **Counter war last:** Counterspell, Mana Drain (its colourless mana arrives in your next main phase), Swan Song (a
   counter for an instant, sorcery or enchantment; it gives them a 2/2 Bird), Mystic Confluence (counter unless they
   pay {3}).

- **Their triggers still resolve.** If they kill Aragorn in response to a red spell, the damage trigger already on the
  stack still resolves, but the next spells don't trigger.
- **Cast Breach last** among the setup pieces: it is the only piece lost completely if it is countered, and it is
  sacrificed at the end step.
- **Spark Double** is a second Aragorn: cast it with the first on the battlefield, so one removal spell no longer
  stops the engine.
- **Don't tap out for a setup piece into a blue opponent** with untapped mana and a full hand, unless you hold a counter
  or Fierce Guardianship free.
- **Rhystic Study and Mystic Remora in their decks:** Chulane, Yuriko, Kinnan, Miirym, Urza and Zur run Rhystic Study
  (Remora too, except Miirym). Pay the {1} on cheap spells in a slow turn if you can; on the storm turn, don't (the cards
  they draw don't matter if you win). Remora draws them a card unless you pay {4}: ignore it.

### Interaction on opponents' turns

| Card | Cost | Use it on |
|---|---|---|
| Counterspell | {U}{U} | A creature or noncreature combo piece: Kinnan, Miirym, Heliod, Walking Ballista, Isochron Scepter |
| Mana Drain | {U}{U} | The same, when you want the mana on your turn (counter a big spell, spend the colourless mana on Comet Storm's X) |
| Force of Will | {3}{U}{U}, or 1 life and a pitched blue card | Their combo piece, or their counter on your Breach or Emancipation. Pitching a blue card is the cost of casting it free |
| Fierce Guardianship | {2}{U}, free with a commander | Noncreature spells only: Basalt Monolith, Isochron Scepter, Aluren, Sanguine Bond, Ruinous Ultimatum, a counterspell on your Breach. Can't hit Kinnan or any creature |
| Swan Song | {U} | Instants, sorceries and enchantments: Dramatic Reversal, Freed from the Real, Aluren, Blasphemous Act, Toxic Deluge. They get a 2/2 flier |
| Mystic Confluence | {3}{U}{U} | A flexible turn: counter unless they pay {3}, bounce a creature, draw a card, three times |
| Deflecting Swat | {2}{R}, free with a commander | Redirect removal aimed at Aragorn; a red spell, so 3 damage |
| Cyclonic Rift | {1}{U} (overload {6}{U}) | One nonland permanent: Basalt Monolith, Isochron Scepter, Winter Orb, Back to Basics, Blood Moon, Rule of Law. Overload is a one-sided wipe of all of their nonland permanents, best at their end step before your burn turn |
| Swords to Plowshares / Path to Exile | {W} | Exile a creature at instant speed: Kinnan, Zur, Yuriko, Krenko, a Dragon (the controller gains life from Swords or a land from Path) |
| Lightning Bolt | {R} | Creatures with 3 toughness or less: Kinnan, Krenko, Yuriko, Opposition Agent, Dauthi Voidwalker |
| Chaos Warp | {2}{R} | Any permanent: Blood Moon, Winter Orb, Aluren, Isochron Scepter, a huge Dragon. They shuffle it away and reveal the top card; if it is a permanent it enters, so use it on a lone key piece |
| Comet Storm | {X}{R}{R} + {1} per extra target | A table of 1-toughness creatures, or lethal damage to several players |
| Restoration Angel | {3}{W}, flash | A 3/4 flying blocker with flash; blink Aragorn in response to removal |

- **Sweepers (your turn, sorcery):** Austere Command ({4}{W}{W}, choose two: artifacts, enchantments, creatures with mana
  value 3 or less, creatures with mana value 4 or more); Nibelheim Aflame ({2}{R}{R}: a creature you control deals damage equal to its power to
  each other creature; Aragorn is a 5/5, and a green spell first makes him 9/9); Elspeth -3. They all hit your own
  creatures: Austere Command's "4 or more" mode kills Aragorn (mana value 4), and "3 or less" kills your Soldiers.
- **Enchantment and artifact answers:** Aura Shards (any creature of yours entering, Soldiers included), Skyclave Apparition,
  Chaos Warp, Cyclonic Rift, Austere Command (also destroys your own Sol Ring, Arcane Signet, Greaves and Skullclamp).
- **Attack taxes:** Ghostly Prison and Windborn Muse, {2} per attacker each (they stack to {4}). They don't stop
  triggers (Impact Tremors, Purphoros, Sheoldred) or combos.

### Matchups (Tier 4 and Tier 5)

The t4 decks win by combat and drain; the t5 decks by combos on turns 3-6. The sim eliminations below are from
`audit/aragorn/FINDINGS.md`.

**Tier 4**

| Opponent | Their threat | Your plan | Cards that matter |
|---|---|---|---|
| Chulane (Bant value and Aluren) | Aluren, Intruder Alarm or Cloudstone Curio with Shrieking Drake or Whitemane Lion and Chulane: free creature loops that draw the deck, then Thassa's Oracle, Laboratory Maniac or Craterhoof. Counters: Counterspell, Dovin's Veto, Swan Song, Fierce Guardianship. Mana Breach, Drannith Magistrate and Aven Mindcensor slow you. The combo ended about 10-11% of t4 games | Hold a counter for the loop's enchantment (Aluren, Intruder Alarm) and remove it if it lands. Kill Chulane (Swords, Path) when he is the loop's last piece. Their early board is small, so cheap pingers and a fast Aragorn pressure them | Swan Song, Counterspell, Force of Will, Skyclave Apparition (exiles Aluren, Intruder Alarm, Curio), Aura Shards, Austere Command, Chaos Warp, Cyclonic Rift |
| Edgar Markov (Mardu Vampires) | A wide Vampire board that makes a token for each Vampire spell cast, first strike and +1/+1 counters; Sanguine Bond with Exquisite Blood, Blood Artist, Cruel Celebrant and Bloodletter drain; cheap removal (Lightning Bolt, Terminate, Fatal Push, Heartless Act, Swords, Anguished Unmaking); Damn, Ruinous Ultimatum. Combat ended about 11-12% of t4 games | Sweep early: Austere Command ("mana value 3 or less") and Nibelheim Aflame kill nearly all the cheap Vampires. Ghostly Prison and Windborn Muse tax the rest. Chump with Soldiers and Kitchen Finks. Keep Greaves, Ephemerate or a counter ready for their removal on Aragorn | Austere Command, Nibelheim Aflame, Cyclonic Rift (overload), Ghostly Prison, Windborn Muse, Elspeth, Swan Song for Sanguine Bond or Exquisite Blood |
| Krenko (Mono-Red Goblins) | Krenko, Mob Boss doubling Goblins every turn; haste (Fervor, Goblin Warchief, Lightning Greaves); Goblin Bombardment, Impact Tremors, Purphoros; Thornbite Staff, Ashnod's Altar and Umbral Mantle untap combos; Blood Moon makes your 32 nonbasic lands tap only for red; Blasphemous Act, Vandalblast, Abrade, Deflecting Swat. Krenko ended about 12-13% of t4 games by combat and 8% by triggers | Kill Krenko the turn he lands (Lightning Bolt kills the 3/3; Swords, Path, Skyclave). Sweep the tokens (Austere Command, Nibelheim Aflame, Rift overload). Ghostly Prison plus Windborn Muse make an attack cost {4} per Goblin. If Blood Moon lands, only the Forest, Island and Plains and your mana creatures make other colours: answer it | Lightning Bolt, Swords, Path, Skyclave Apparition, Nibelheim Aflame, Austere Command, Ghostly Prison, Windborn Muse, Aura Shards (Bombardment, Tremors, Blood Moon), Chaos Warp |
| Prosper (Rakdos Exile and Treasure) | Card flow into Treasures, with Marionette Master, Mayhem Devil, Disciple of the Vault and Weftstalker Ardent as payoffs; Bolas's Citadel with Sensei's Divining Top and Aetherflux Reservoir; Revel in Riches; its own Underworld Breach; Opposition Agent, Dauthi Voidwalker and Bojuka Bog hit your Breach graveyard; Toxic Deluge, Blasphemous Act. Its triggers ended about 5% of t4 games in the t4 analysis | Its payoffs need Treasures and creatures, so kill or counter the payoff pieces. Kill Opposition Agent and Dauthi Voidwalker on sight (Bolt). Counter Bolas's Citadel and Revel in Riches. Don't run Breach into Voidwalker or Bog | Lightning Bolt, Swords, Path, Counterspell, Mana Drain, Swan Song (Revel in Riches), Skyclave Apparition |
| Yuriko (Dimir Ninjas) | Cheap evasive creatures (Slither Blade, Triton Shorestalker, Ornithopter, Spectral Sailor, Faerie Seer), then ninjutsu Yuriko or Ninja of the Deep Hours; each hit reveals a card and you lose life equal to its mana value (Shadow of Mortality 15, Temporal Trespass 11, Dig Through Time 8), stacked with Sensei's Top and Vampiric Tutor. Counters: Force of Will, Fierce Guardianship, Counterspell, Misdirection, Flare of Denial. Its triggers ended about 11% of t4 games and its combat 6% | Tax every attacker (Ghostly Prison, Windborn Muse), kill the evasive creature and keep Lightning Bolt, Swords or Path for the Ninja that arrives attacking (Yuriko is a 1/3). Keep life high | Ghostly Prison, Windborn Muse, Lightning Bolt, Swords, Path, Restoration Angel (a flying blocker), Kitchen Finks |

**Tier 5**

| Opponent | Their threat | Your plan | Cards that matter |
|---|---|---|---|
| Kinnan (Simic Mana Combo) | Turn 3-5 wins: Kinnan with Basalt Monolith, or Freed from the Real / Pemmin's Aura on a mana creature, then Walking Ballista or Thrasios. Counters: Counterspell, Mana Drain, Force of Will, Force of Negation, Flusterstorm, Pact of Negation, Fierce Guardianship, Swan Song. The combo ended about 16% of t5 games, the top killer with Urza | Don't race: it wins long before a 12-turn goldfish. Hold removal from turn 2: Lightning Bolt, Swords or Path on Kinnan (a 2/2), a counter on Basalt Monolith or the Aura. Cyclonic Rift on the Monolith in response to its untap | Lightning Bolt, Swords, Path, Counterspell, Mana Drain, Force of Will, Swan Song (the Auras), Cyclonic Rift, Skyclave Apparition |
| Miirym (Temur Dragons) | Ramp into Dragons that Miirym copies: Terror of the Peaks, Scourge of Valkas, Twinflame Tyrant, Thrakkus, Wrathful Red Dragon with Blasphemous Act. Counters: Counterspell, Mana Drain, Fierce Guardianship; Cyclonic Rift, Heroic Intervention. Combat ended about 14% of t5 games | Fliers aren't stopped by Soldiers. Counter Miirym or the first big Dragon (Miirym has ward {2}; Terror of the Peaks adds 3 life to any spell aimed at it), then sweep: Elspeth -3 kills the Dragons with power 4 or more, and so does Austere Command's "4 or more" mode. Ghostly Prison and Windborn Muse tax fliers too | Elspeth, Austere Command, Nibelheim Aflame, Swords, Path, Counterspell, Mana Drain, Windborn Muse |
| Urza (Mono-Blue Artifacts) | Isochron Scepter with Dramatic Reversal, Basalt Monolith with Rings of Brighthearth or Forensic Gadgeteer, or Hullbreaker Horror, ending in Walking Ballista; Winter Orb, Static Orb and Back to Basics (your nonbasic lands don't untap); Narset. Counters: Force of Will, Force of Negation, Mana Drain, Counterspell, Fierce Guardianship, Swan Song, Pact of Negation. The combo ended about 15% of t5 games | Hold a counter for the loop pieces: Fierce, Counterspell, Force of Will or Mana Drain on Scepter, Rings or Monolith; Swan Song on Dramatic Reversal. Skyclave Apparition exiles Scepter, Rings, the Orbs, Back to Basics and Narset; Aura Shards destroys artifacts. Rift a piece in response. Bait with a cheap spell before the one that matters | Fierce Guardianship, Swan Song, Counterspell, Skyclave Apparition, Cyclonic Rift, Aura Shards, Chaos Warp, Swords on Urza |
| Yawgmoth (Mono-Black Aristocrats) | Yawgmoth with two undying creatures (Geralf's Messenger, Butcher Ghoul, Hancock) and a death payoff (Blood Artist, Zulaport Cutthroat, Bastion of Remembrance, The Meathook Massacre); Gravecrawler with Phyrexian Altar; Sheoldred, the Apocalypse (you lose 2 for each card you draw); Mikaeus destroys a Human that damages them; Damnation, Toxic Deluge; Bojuka Bog, Dauthi Voidwalker. The combo ended about 10% of t5 games | Exile, don't destroy: Swords, Path, Skyclave on the engine pieces; counter the altar or the payoff. Draw less while Sheoldred lives (see Life management). Point Aragorn's damage at another opponent while Mikaeus is out (it kills Aragorn, Firebrand Archer, Young Pyromancer and Third Path Iconoclast, which are all Humans) | Swords, Path, Skyclave Apparition, Swan Song, Counterspell, Soldier tokens as chump-blockers, Lightning Bolt for Bowmasters and Voidwalker |
| Zur (Esper Enchantments) | Zur attacks (a 1/4 flier) and fetches an enchantment of mana value 3 or less each time: Rule of Law and Eidolon of Rhetoric (one spell a turn), Propaganda, Ghostly Prison, Rest in Peace, Necropotence, armor Auras that make Zur hard to kill; Heliod, Sun-Crowned with Walking Ballista wins; Drannith Magistrate, Grand Abolisher, Opposition Agent, Toxic Deluge, Supreme Verdict. The combo ended about 11% of t5 games | Kill Zur before his first attack: Swords, Path or Skyclave (Lightning Bolt alone doesn't kill a 1/4; Nibelheim Aflame does). Answer a one-spell lock at once: Aura Shards, Skyclave, Chaos Warp, Austere Command, Rift. Counter Heliod (Counterspell, Fierce, Swan Song) | Swords, Path, Skyclave Apparition, Aura Shards, Chaos Warp, Austere Command, Counterspell, Fierce Guardianship, Cyclonic Rift |

### After a stopped attempt

| What happened | Rebuild |
|---|---|
| Aragorn killed or exiled | Recast from the command zone for {R}{G}{W}{U} plus {2} for each earlier cast. Meanwhile Niv-Mizzet, Talrand, Guttersnipe, Firebrand Archer and Young Pyromancer run the spell plan. Spark Double copies a creature that is left |
| Aragorn countered | The commander may go to the command zone: recast next turn at +{2} with a counter of your own up |
| Underworld Breach countered | A dead card; the graveyard is untouched. Dig for the storm cards again; no tutor finds Breach |
| A ritual or an escaped spell countered mid-chain | You lose its mana, not the chain. Recount: if you can't reach the next storm spell, spend what is left on Lightning Bolt and Aragorn's triggers. Breach is sacrificed at the end step and the exiled cards are gone |
| Grapeshot or Brain Freeze countered | The storm copies still resolve (storm is a trigger of its own); only the original is lost |
| Graveyard exiled in response (Bojuka Bog, Rest in Peace, Voidwalker, Tormod's Crypt) | The line is over; spend what is left on the pingers and Jeska's Will |
| Fiery Emancipation countered or destroyed | Aragorn and the pingers still work: burn at 1x and rebuild |
| A wipe (Damnation, Toxic Deluge, Supreme Verdict, Blasphemous Act, Meathook) | Kitchen Finks persists through "destroy" wipes. Hold Restoration Angel and Heroic Intervention; Elspeth +1 and Sokenzan's channel ({3}{R}, {1} less per legendary creature you control) rebuild |
| Opponent locks you (Blood Moon, Back to Basics, Rule of Law) | Skyclave Apparition, Chaos Warp, Cyclonic Rift, Aura Shards, Austere Command. Blood Moon: your basics and mana creatures still make colours |

- After a stop the table has seen your plan: for the next try, keep one more counter up than last time.

### Life management

| Source | Cost |
|---|---|
| Fetch lands (Flooded Strand, Marsh Flats, Misty Rainforest, Polluted Delta, Verdant Catacombs, Windswept Heath) | 1 each |
| Painlands (Battlefield Forge, Brushland, Shivan Reef, Yavimaya Coast) | 1 for each coloured mana tapped; colourless mana is free |
| Shock lands (Steam Vents, Temple Garden) | 2 to enter untapped |
| Force of Will | 1 (the free cast) |
| Sylvan Library | 4 for each extra card kept |
| Opponents' effects | Sheoldred, the Apocalypse (Yawgmoth): 2 for each card you draw. Terror of the Peaks (Miirym): 3 life to target it with a spell. Yuriko's reveal: the revealed card's mana value |

| Gain | Amount |
|---|---|
| Kitchen Finks | 2 when it enters, and 2 more when it persists back. Ephemerate, Restoration Angel and Spark Double each replay the enter trigger |
| Teferi's Protection | Not a gain: your life total can't change until your next turn. It blocks a drain, and also your own life payments |

The deck has no other life gain. Ghostly Prison, Windborn Muse and the Soldiers keep the damage from arriving instead.

**Rough budget at 40 life** (an estimate): a typical turn-1-to-5 opening costs 3 to 6 (a fetch or two, a few painland
taps, a shock land). Against Sheoldred, the Apocalypse, skip Sylvan Library's extra draws and think before Fact or
Fiction, Frantic Search or paying for Rhystic Study; Orcish Bowmasters (Yawgmoth) pings you for each draw after the
first in a turn. Swords to Plowshares gains the opponent life equal to the creature's power: use Path to Exile on a big
creature when life matters.

### Common mistakes

1. Casting Aragorn into open Counterspell mana with no counter behind him. He is the card removed most.
2. Starting a Breach turn without Aragorn on the battlefield, or without counting the graveyard, mana and colours first.
3. Casting Breach before you hold the storm card and the mana: it is sacrificed at the end step.
4. Aiming Brain Freeze at yourself with a short library (three cards per copy), or forgetting that exiling cards for
   escape also exiles the rituals you wanted to cast next.
5. Tapping out on your own turn against Kinnan, Urza, Zur or Yawgmoth. The sims show the deck losing to their combos on
   turns 3-6 with its mana spent.
6. Austere Command: "creatures with mana value 4 or more" kills Aragorn, and "3 or less" kills your Soldiers. Nibelheim
   Aflame and Elspeth -3 also kill your own creatures. Choose the mode and the creature before you cast.
7. Putting Lightning Greaves on Aragorn and then trying to target him with his green pump (shroud).
8. Using Swan Song or Force of Will on a draw spell, or Fierce Guardianship on a creature (it only counters noncreature
   spells).
9. Casting Jeska's Will without a commander on the battlefield (you get one mode), or leaving red mana unspent: it
   empties at the end of the phase.
10. Cracking a fetch land or searching with Nature's Lore or a tutor while Opposition Agent is on the battlefield (they
    take the card), or pointing Aragorn's damage at the player who controls Mikaeus, the Unhallowed.

### Weak spots

- **Mana.** Aragorn needs {R}{G}{W}{U}; Triple Triad and Fiery Emancipation need {R}{R}{R}, Niv-Mizzet
  {U}{U}{U}{R}{R}{R}. 35 lands, four mana creatures, three rocks, Nature's Lore and Smothering Tithe's Treasures carry it. The sims
  show the deck short of mana about a fifth of the time.
- **Slow and low on finishers.** A goldfish kill takes until about turn 12; the lines are single copies. The win comes
  from many small hits, so the deck lives only if the board is stable.
- **Graveyard hate and spell limits.** Bojuka Bog, Rest in Peace, Dauthi Voidwalker and Tormod's Crypt hurt Breach;
  Drannith Magistrate stops escape; Rule of Law and Eidolon of Rhetoric stop every chain.
- **Lock pieces.** Blood Moon (Krenko) and Back to Basics (Urza) take away the colours; keep Skyclave Apparition and Aura
  Shards for them.

## Mana

35 lands, 4 mana creatures, 3 rocks, Nature's Lore, and Smothering Tithe's Treasures.

- **Original duals (6):** Plateau, Savannah, Taiga, Tropical Island, Tundra, Volcanic Island.
- **Fetch lands (6):** Flooded Strand, Marsh Flats, Misty Rainforest, Polluted Delta, Verdant Catacombs, Windswept Heath.
  Each finds a card with one of its two land types, so between them they reach the duals above, Steam Vents, Temple
  Garden, Mystic Sanctuary (an Island) and the basics.
- **Shock lands (2):** Steam Vents, Temple Garden. **Painlands (4):** Battlefield Forge, Brushland, Shivan Reef, Yavimaya Coast.
- **Tapped three-colour lands (4):** Frontier Bivouac (G/U/R), Jungle Shrine (R/G/W), Mystic Monastery (U/R/W), Seaside Citadel (G/W/U).
- **Conditional duals (5):** Glacial Fortress, Spirebluff Canal, Stormcarved Coast, Sulfur Falls, Sunpetal Grove.
- **Others:** Command Tower and Path of Ancestry (any of Aragorn's four colours), Exotic Orchard, Mystic Sanctuary,
  Sokenzan, Crucible of Defiance, and 1 Forest, 1 Island, 1 Plains.
- **Coloured sources among the lands** (fetches not counted): red 15, green 13, white 14, blue 17.
- **Other mana:** Birds of Paradise, Llanowar Elves, Paradise Druid, Faeburrow Elder; Sol Ring, Arcane Signet, Lotus
  Petal; Nature's Lore; the one-turn mana of Seething Song, Desperate Ritual, Rite of Flame and Manamorphose; Jeska's Will's
  red; Mana Drain's colourless mana; Treasures from Smothering Tithe and Storm-Kiln Artist.

## Bracket and Rule 0
**Bracket 4,** with 11 Game Changers: **Underworld Breach**, **Jeska's Will**, **Force of Will**, **Cyclonic Rift**,
**Mystical Tutor**, **Worldly Tutor**, **Aura Shards**, **Teferi's Protection**, **Smothering Tithe**, **Fierce
Guardianship** and **Rhystic Study**. That is well past Bracket 3's limit of three, so the deck is Bracket 4 only.
There's no two-card infinite combo. The Breach line needs the graveyard to fill first, as in Kefka (Lion's Eye
Diamond would change that).

## Decklist by type (100)

**Commander (1).** Aragorn, the Uniter

**Creatures (17).** Birds of Paradise, Faeburrow Elder, Firebrand Archer, Guttersnipe, Kitchen Finks, Llanowar Elves, Niv-Mizzet Parun, Paradise Druid, Restoration Angel, Skyclave Apparition, Spark Double, Storm-Kiln Artist, Talrand Sky Summoner, Third Path Iconoclast, Veyran Voice of Duality, Windborn Muse, Young Pyromancer

**Planeswalkers (2).** Elspeth Sun's Champion, Tezzeret Cruel Captain

**Enchantments (8).** Aura Shards, Fiery Emancipation, Ghostly Prison, Rhystic Study, Smothering Tithe, Sylvan Library, Triple Triad, Underworld Breach

**Artifacts (5).** Arcane Signet, Lightning Greaves, Lotus Petal, Skullclamp, Sol Ring

**Sorceries (7).** Austere Command, Expressive Iteration, Grapeshot, Jeska's Will, Nature's Lore, Nibelheim Aflame, Rite of Flame

**Instants (25).** Brain Freeze, Chaos Warp, Comet Storm, Counterspell, Cyclonic Rift, Deflecting Swat, Desperate Ritual, Ephemerate, Fact or Fiction, Fierce Guardianship, Force of Will, Frantic Search, Heroic Intervention, Lightning Bolt, Mana Drain, Manamorphose, Mystic Confluence, Mystical Tutor, Path to Exile, Seething Song, Swan Song, Swords to Plowshares, Teferi's Protection, Twinned Vision, Worldly Tutor

**Lands (35).** Battlefield Forge, Brushland, Command Tower, Exotic Orchard, Flooded Strand, Frontier Bivouac, Glacial Fortress, Jungle Shrine, Marsh Flats, Misty Rainforest, Mystic Monastery, Mystic Sanctuary, Path of Ancestry, Plateau, Polluted Delta, Savannah, Seaside Citadel, Shivan Reef, Sokenzan Crucible of Defiance, Spirebluff Canal, Steam Vents, Stormcarved Coast, Sulfur Falls, Sunpetal Grove, Taiga, Temple Garden, Tropical Island, Tundra, Verdant Catacombs, Volcanic Island, Windswept Heath, Yavimaya Coast, 1 Forest, 1 Island, 1 Plains

## Import list (100)

```
1 Aragorn, the Uniter
1 Arcane Signet
1 Aura Shards
1 Austere Command
1 Battlefield Forge
1 Birds of Paradise
1 Brain Freeze
1 Brushland
1 Chaos Warp
1 Comet Storm
1 Command Tower
1 Counterspell
1 Cyclonic Rift
1 Deflecting Swat
1 Desperate Ritual
1 Elspeth, Sun's Champion
1 Ephemerate
1 Exotic Orchard
1 Expressive Iteration
1 Fact or Fiction
1 Faeburrow Elder
1 Fierce Guardianship
1 Fiery Emancipation
1 Firebrand Archer
1 Flooded Strand
1 Force of Will
1 Frantic Search
1 Frontier Bivouac
1 Ghostly Prison
1 Glacial Fortress
1 Grapeshot
1 Guttersnipe
1 Heroic Intervention
1 Jeska's Will
1 Jungle Shrine
1 Kitchen Finks
1 Lightning Bolt
1 Lightning Greaves
1 Llanowar Elves
1 Lotus Petal
1 Mana Drain
1 Manamorphose
1 Marsh Flats
1 Misty Rainforest
1 Mystic Confluence
1 Mystic Monastery
1 Mystic Sanctuary
1 Mystical Tutor
1 Nature's Lore
1 Nibelheim Aflame
1 Niv-Mizzet, Parun
1 Paradise Druid
1 Path of Ancestry
1 Path to Exile
1 Plateau
1 Polluted Delta
1 Restoration Angel
1 Rhystic Study
1 Rite of Flame
1 Savannah
1 Seaside Citadel
1 Seething Song
1 Shivan Reef
1 Skullclamp
1 Skyclave Apparition
1 Smothering Tithe
1 Sokenzan, Crucible of Defiance
1 Sol Ring
1 Spark Double
1 Spirebluff Canal
1 Steam Vents
1 Storm-Kiln Artist
1 Stormcarved Coast
1 Sulfur Falls
1 Sunpetal Grove
1 Swan Song
1 Swords to Plowshares
1 Sylvan Library
1 Taiga
1 Talrand, Sky Summoner
1 Teferi's Protection
1 Temple Garden
1 Tezzeret, Cruel Captain
1 Third Path Iconoclast
1 Triple Triad
1 Tropical Island
1 Tundra
1 Twinned Vision
1 Underworld Breach
1 Verdant Catacombs
1 Veyran, Voice of Duality
1 Volcanic Island
1 Windborn Muse
1 Windswept Heath
1 Worldly Tutor
1 Yavimaya Coast
1 Young Pyromancer
1 Forest
1 Island
1 Plains
```
