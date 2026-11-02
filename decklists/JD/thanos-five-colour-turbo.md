# Thanos, the Mad Titan — Five-Colour cEDH Turbo

JD's five-colour cEDH turbo deck with Thanos, the Mad Titan as the commander. It started on 10/30 as the Scion v1 list
(`decklists/pool/retired/scion-ur-dragon-turbo.md`, retired 10/30) minus Scion's copy targets; on 11/01 the list was
brought in line with JD's physical deck (22 cards each way, see `decklists/CHANGELOG.md`). Not simulated since.

- **Commander:** Thanos, the Mad Titan
- **Color identity:** White, Blue, Black, Red, Green (his power-up cost has all five colours)
- **Game Changers (23):** Ad Nauseam, Ancient Tomb, Chrome Mox, Demonic Tutor, Enlightened Tutor, Fierce Guardianship, Force of Will, Gamble, Grim Monolith, Imperial Seal, Intuition, Jeska's Will, Lion's Eye Diamond, Mana Vault, Mox Diamond, Mystical Tutor, Necropotence, Orcish Bowmasters, Rhystic Study, Thassa's Oracle, Underworld Breach, Vampiric Tutor, Worldly Tutor
- **Simulator:** deck key `thanos`, Tier 6 (Scion's seat, 10/30). The power-up is card code
  (`rust/crates/sim-core/src/impls/t6_thanos.rs`), the plan is `data/plans/thanos.toml`; results in
  `audit/thanos/FINDINGS.md`.

## Strategy

Fast mana and rituals into the cEDH finishes: Thassa's Oracle with Demonic Consultation or Tainted Pact, Underworld
Breach with Lion's Eye Diamond and Brain Freeze, and Ad Nauseam to dig for either. Free counters, Silence, Orim's
Chant, Grand Abolisher, Voice of Victory and Teferi, Time Raveler protect the win turn. The
commander is not the plan; he is the five-colour identity, a cheap body, and the switch for the free spells.

The design reasoning (the stance, the role counts, why each card is in) is in JD's design doc, "Thanos Turbo:
First-Principles Design" (decisions D1-D12). Its odds come from rough models, so the numbers quoted below from it are
approximate. The sim's cEDH AI plays far slower than people do (median goldfish kill turn 11), so its turn numbers
only compare lists.

### Why Thanos instead of Scion

- **He is cheap.** Thanos costs {R}{W}{B}: a 4/4 with deathtouch and lifelink on turn 2-3 off a mana creature, and
  cheap to recast ({2} more each time). Scion needs exactly {W}{U}{B}{R}{G}.
- **He makes three spells free.** Fierce Guardianship (counter a noncreature spell), Deflecting Swat (new targets
  for a spell or ability) and Deadly Rollick (exile a creature) cost nothing while you control your commander, and
  Jeska's Will gets both modes. A 3-mana
  commander is usually on the battlefield; Scion rarely was.
- **Scion's engine barely won.** In the goldfish, the Prismari line was 0.9% of Scion's kills and Thassa's Oracle 73%.
  Ancient Copper Dragon, Korvold and Prismari were copy targets for Scion and dead cards without him.
- **What Scion had that Thanos doesn't:** a backup win from the command zone (become Prismari for storm, or Ancient
  Copper Dragon for Treasures). If the Oracle and Breach lines are both stopped, Thanos has no third plan.

## Pilot's guide

Written for a competent cEDH player who has never piloted this deck. "Free" means no mana; the card may still cost
a card, life or a condition.

### 1. How the deck wins

**Line 1: Thassa's Oracle + Demonic Consultation or Tainted Pact.** Oracle {U}{U} plus Consultation {B} (3 mana) or
Tainted Pact {1}{B} (4 mana).

1. Cast Thassa's Oracle. Let it resolve. Its enter trigger goes on the stack.
2. With the trigger on the stack, cast Demonic Consultation or Tainted Pact (both instants).
   - **Consultation:** name a card that is not in your library. Thassa's Oracle itself is a safe name: the deck has
     one, and it is on the battlefield. Consultation exiles the top six, then reveals until it finds the name, so it
     exiles the whole library.
   - **Tainted Pact:** decline every card. The deck is singleton, so two cards with the same name never come up and
     it exiles the whole library.
3. The trigger resolves: X (your devotion to blue) is at least 0 cards in the library, and you win. Removing Oracle
   in response doesn't help them: X can be 0 and 0 >= 0 still wins.

Why Oracle goes first: if Oracle is countered, Consultation is still in hand and you have lost a card, not the
game. Cast Consultation first and a counter on Oracle leaves you with no library, losing at your next draw.

What can stop each half:

| Piece | Stopped by (opponents' cards) | Not stopped by |
|---|---|---|
| Thassa's Oracle (blue creature, mana value 2) | Force of Will, Pact of Negation, Mindbreak Trap (exiles it), Red Elemental Blast / Pyroblast | Swan Song, Flusterstorm, Fierce Guardianship, Force of Negation, Mental Misstep (all noncreature-only or mana value 1) |
| Demonic Consultation (black instant, 1) | Force of Will, Pact of Negation, Mindbreak Trap, Swan Song, Flusterstorm, Fierce Guardianship, Force of Negation, Mental Misstep | Red Elemental Blast / Pyroblast (it is black) |
| Tainted Pact (black instant, 2) | The same as Consultation except Mental Misstep | Red Elemental Blast / Pyroblast, Mental Misstep (mana value 2) |
| The trigger | Stifle-type effects (counter an ability) | Every counterspell |

- Cavern of Souls naming Wizard makes Oracle uncounterable (Mindbreak Trap still exiles it). Pay Oracle's {U}{U}
  with Cavern's coloured mana and one other blue source.
- **If the trigger is countered after the library is gone,** you lose at your next draw. Outs this turn: Mockingbird
  for X=1 ({1}{U}) copies the Oracle still on the battlefield and gives a new trigger; or bounce your own Oracle with
  Snapback (free by exiling a blue card), Chain of Vapor or Otawara and recast it for {U}{U}. Don't use Teferi's -3:
  it draws a card from the empty library.
- **If Consultation is countered,** the trigger resolves normally (look at the top cards, no win). Oracle stays, so a
  later Mockingbird copying it plus Tainted Pact is a second try for 4 mana.

**Line 2: Underworld Breach + Lion's Eye Diamond + Brain Freeze.** Worked out from the Oracle text:

- Breach gives each nonland card in your graveyard escape: its mana cost plus exiling three other cards from your
  graveyard. Breach is sacrificed at the beginning of the end step.
- Lion's Eye Diamond costs {0}. "Discard your hand, sacrifice: add three mana of any one colour", instant speed only.
  Escaped, it costs {0} plus three cards, so each loop is +3 mana for 3 graveyard cards.
- Brain Freeze ({1}{U}) mills three per copy, with storm: it and its copies mill 3 x (spells cast before it this
  turn + 1). Escaping counts as casting, so every loop raises the storm count. Escaped spells go back to the
  graveyard when they resolve; nothing in Breach exiles them.

Start: Breach in play costs {1}{R}. You need LED, Brain Freeze (in hand or graveyard), and at least three other cards
in your graveyard (not LED or Brain Freeze) when you first escape Brain Freeze. LED's discard counts.

1. Cast Underworld Breach ({1}{R}). Cast it before LED: if it's countered, LED is still in hand.
2. Cast LED ({0}). Activate it: discard your hand (Brain Freeze goes to the graveyard if it was in hand), add {U}{U}{U}.
3. Escape Brain Freeze ({1}{U} + exile three; keep LED). Target yourself. 1 blue mana floats.
4. Escape LED ({0} + exile three), activate for {U}{U}{U}.
5. Repeat 3-4. Each LED + Brain Freeze cycle gains 1 mana, exiles 6 cards and adds 2 to the storm count. Once a Brain
   Freeze has a storm count of 1 or more it mills at least 6, which pays for the next cycle, and every cycle mills
   more than the last. In the order above, Breach and LED were cast this turn, so the first Brain Freeze has a storm
   count of 2 and mills 9. If LED went down on an earlier turn, a ritual first does the same job.
6. **Finish with Thassa's Oracle.** Mill yourself to an empty library, then cast Oracle from hand or escape it from
   the graveyard ({U}{U} + exile three; LED makes the blue). The trigger sees 0 cards and you win.

- Never exile Thassa's Oracle as escape fuel, and make sure it is in hand or the graveyard (not exiled) before you
  mill yourself out. If the loop breaks with your library empty, you lose at your next draw.
- **If Oracle is gone,** point Brain Freeze copies at the opponents until all three libraries are empty. They lose
  only when they next draw, and Endurance (Kinnan runs it) can put a graveyard back into a library, so this is the
  backup finish.
- Rituals stretch the loop: escaping Dark Ritual ({B} + three) nets {B}{B}, Cabal Ritual with seven cards in the
  graveyard nets {B}{B}{B}. Don't escape Manamorphose with an empty library: it draws a card.
- Assumption: no opponent casts anything during the loop. A free counter on Breach, Brain Freeze or LED's casts, or
  graveyard hate in response, breaks it.

**Ways in: Necropotence and Ad Nauseam.**

| Card | Cost | Life per card | When | Watch out |
|---|---|---|---|---|
| Necropotence | {B}{B}{B} (turn 1 with Dark Ritual) | 1 per card | Main phase 1; cards arrive at the beginning of this turn's end step | You discard down to 7 at cleanup, and Necro exiles discards. Pay for about 7 minus your hand, plus what you can cast at instant speed in the end step. It skips your draw step (no Sylvan Library; tapped Mana Vault doesn't hurt you) |
| Ad Nauseam | {3}{B}{B} instant | Mana value of each card: the 99 add up to 132, about 1.3 per card (27 lands cost 0) | End step of the player before you, or your own turn with the combo mana up | Force of Will 5; Deadly Rollick, Beseech the Mirror and Mindbreak Trap 4 |

- Ad Nauseam rough budget: 10 cards cost about 13-14 life, 20 cards about 27. On an opponent's end step, stop around
  10 life. On your kill turn, keep only what the rest of the kill costs (fetch 1, Confluence 1, Force of Will 1,
  Phyrexian mana 2, Vampiric or Seal 2, Grim Tutor 3).
- Both put cards into hand without drawing, so an opposing Orcish Bowmasters doesn't punish them.

### 2. What Thanos does for the deck

**When to cast him.** Turn 2 off Birds of Paradise or Noble Hierarch, turn 3 otherwise. He needs exactly {R}{W}{B}:
Sol Ring, Mana Vault, Grim Monolith and Ancient Tomb can't help. Cast him when the kill isn't on this turn and the 3
mana isn't needed for protection. With the kill available, win instead. The sim's FINDINGS flag this exact trade-off
(73 decisions where casting Thanos competed with holding mana for the win turn).

**What he turns on while he's on the battlefield:**

- **Fierce Guardianship:** counter target noncreature spell, free. It can't counter a creature (an opposing Oracle
  or Dack Fayden).
- **Deflecting Swat:** new targets for target spell or ability, free. Moves removal off Thanos or a lock creature,
  moves an opponent's Chain of Vapor copy, and moves a targeted counter onto another legal spell (it needs one on the
  stack: another spell still waiting to resolve, such as one cast while you held priority).
- **Deadly Rollick:** exile target creature, free. Your instant-speed answer to Drannith Magistrate, Collector
  Ouphe, Lavinia, Opposition Agent, Archon of Emeria or a creature commander (a commander usually goes back to the
  command zone, costing its owner {2} more next time).
- **Jeska's Will:** both modes ({R} per card in an opponent's hand, and exile your top three to play this turn).

The free cost is checked when you cast the spell. Killing Thanos in response doesn't stop it; killing him earlier
(at your upkeep) does. Deflecting Swat can save him from targeted removal while he's still out.

**The power-up.** *{C}{W}{U}{B}{R}{G}: put two +1/+1 counters on Thanos. Choose odd or even. Destroy each other
creature with mana value of the chosen quality.*

- **Cost:** six mana, or {C}{U}{G} on the turn he entered (the cost drops by his {R}{W}{B}). So he plus the power-up
  is six mana on one turn.
- **{C} is colourless mana, not generic.** Sure sources: Sol Ring, Mana Vault, Grim Monolith, Ancient Tomb and Cavern
  of Souls (its {C} ability). Gemstone Caverns only if you played it as a land: with a luck counter it makes coloured
  mana instead of {C}. The duals, fetches, Command Tower, Mana Confluence, Lotus Petal, the Moxen, LED, Treasures,
  rituals, Manamorphose, Birds and Hierarch can't pay it.
- **Once per Thanos.** The text says each power-up is activated only once; the sim treats a recast or blinked Thanos
  as a new object that can power up again, at the cheaper cost on the turn he returns.
- It's an activated ability: counterspells, Silence, Archon of Emeria and Deafening Silence don't stop it. Cursed
  Totem and Pithing Needle (opponents' cards) do. The text has no sorcery-speed limit, so you can wipe on an opponent's turn (at full
  cost).
- Zero is even: tokens (Bowmasters' Orc Army, Voice of Victory's Warriors, Fish, Birds) and Rograkh die to even.

| Parity | Your creatures it destroys | What it destroys at the table (opponents' cards) |
|---|---|---|
| **Even** | Thassa's Oracle, Orcish Bowmasters (and its Orc Army), Lotho, Hexing Squelcher, Grand Abolisher, Voice of Victory (2) | Rograkh (0); Kinnan, Thrasios, Drannith Magistrate, Collector Ouphe, Lavinia, Dauthi Voidwalker, Grand Abolisher, Thalia, Orcish Bowmasters (2); Dack Fayden (6) |
| **Odd** | Birds of Paradise, Noble Hierarch, Ragavan, Esper Sentinel, Mockingbird (1); Ranger-Captain of Eos, Valley Floodcaller (3) | Mana creatures, Ragavan, Esper Sentinel (1); Tymna, Sisay, Silas Renn, Archon of Emeria, Opposition Agent, Spellseeker, Endurance, Magus of the Moon (3); Kraum (5) |

**Which parity: it depends on the matchup.** Even against Kinnan, Thrasios, Rograkh, Dack Fayden and creature stax
(Drannith Magistrate, Collector Ouphe, Lavinia). Odd against Tymna, Sisay, Silas Renn, Kraum, Archon of Emeria and
Opposition Agent. Then check your own side: even kills your Grand Abolisher and Voice of Victory (don't wipe even on
the turn you rely on them) and any Oracle left on the battlefield; odd kills your mana creatures and Ranger-Captain.

**Wipe or win?** With six mana on his entry turn and both combo pieces, win. Wipe when you can't win this turn and the
wipe either removes the creature that stops your next kill (Drannith, Ouphe, Lavinia, Archon, Opposition Agent) or
stops an opponent who is about to win with creatures (Kinnan, Thrasios, Sisay, Dack). If the stax piece only stops
one line (Drannith only stops Breach), win with the other line instead.

### 3. Mulligans

The first mulligan is free: you see two 7-card hands before any cost. It's a London mulligan (draw 7, put one card
on the bottom per mulligan after the free one).

| Hand | Keep when |
|---|---|
| 7 cards (the first and the free one) | 1-4 lands (0 lands only with 3+ fast mana; 5+ lands go back); 3+ mana by turn 2 from lands, rocks, rituals and spirit guides (Mox Diamond needs a spare land, Chrome Mox a spare nonland card); and a reason to go: a win-line card, a tutor, or Necropotence, Ad Nauseam, Borne Upon a Wind, Gitaxian Probe or Sylvan Library |
| 6 or 5 cards | 1-5 lands, 3+ cards that make mana, and any of the cards above |
| 4 cards | Keep |

- **Protection is not a keep requirement.** The design model found requiring it costs about 6 points of turn-3 kills,
  and protection is in hand on about 75% of kill turns anyway.
- **The rule counts mana, not colours.** A hand with no coloured source for its spells is a ship even if the count
  passes.
- **Bottom first:** lands past the third, the disruption creatures (Ragavan, Esper Sentinel,
  Lotho, Orcish Bowmasters), then the flex cards (Hexing Squelcher, Seething Song, Diabolic Intent). Keep win-line
  cards and tutors.
- **Gemstone Caverns:** if it's in your kept hand and you aren't the starting player (3 seats in 4), you may start
  with it on the battlefield with a luck counter, tapping for any colour, by exiling a card from your hand. Exile
  what you would bottom (a disruption creature, a flex card, a fourth land). It then can't pay the power-up's {C}.
- Rough odds from the design model: with this rule, a kill is possible by turn 3 in about 29% of games and by turn 4
  in about 41%. Pace yourself for a turn-3-to-5 kill, not a turn-2 one.

Example opening 7s:

| # | Hand | Call | Why |
|---|---|---|---|
| 1 | Underground Sea, Volcanic Island, Polluted Delta, Sol Ring, Thassa's Oracle, Force of Will, Mystic Remora | Keep | 4 mana on turn 2, a line card, and protection |
| 2 | Tundra, Scalding Tarn, Dark Ritual, Necropotence, Lotus Petal, Gitaxian Probe, Swan Song | Keep | Tarn finds Underground Sea, Dark Ritual makes {B}{B}{B}: Necropotence on turn 1 |
| 3 | Volcanic Island, Simian Spirit Guide, Rite of Flame, Underworld Breach, Lion's Eye Diamond, Brain Freeze, Force of Will | Keep | All three Breach pieces; it needs one more land or fetch for graveyard fuel, then turn 2 |
| 4 | Badlands, Mox Diamond, Chrome Mox, Silence, Force of Will, Pact of Negation, Fierce Guardianship | Ship | All protection, no line card, tutor or dig, and Mox Diamond has no spare land |
| 5 | Plateau, Savannah, Taiga, Tropical Island, Noble Hierarch, Orcish Bowmasters, Teferi, Time Raveler | Ship | Good mana and a lock, but no line card, tutor or dig |
| 6 | Ancient Tomb, Sol Ring, Mana Vault, Grim Monolith, Ad Nauseam, Rite of Flame, Gamble | Ship | Passes the count, but there's no coloured source: Ad Nauseam needs {B}{B}, Rite and Gamble {R} |
| 7 | Lotus Petal, Chrome Mox, Elvish Spirit Guide, Dark Ritual, Demonic Tutor, Thassa's Oracle, Brain Freeze | Keep (close) | 0 lands but 4 fast-mana cards. Petal for {B}, Ritual, Demonic Tutor; Mox imprinting Brain Freeze is a blue source. It needs a land for Oracle's {U}{U} |
| 8 | Mana Confluence, Flooded Strand, Birds of Paradise, Rhystic Study, Sylvan Library, Esper Sentinel, Swan Song | Keep (close) | 3 mana on turn 2 and Sylvan Library counts as dig, so the rule keeps it; no line card, so expect a slow start |
| 9 | Bayou, Wooded Foothills, Marsh Flats, Arid Mesa, Command Tower, Vampiric Tutor, Valley Floodcaller | Ship | Five lands go back, whatever the tutor |
| 10 | Scrubland, Arid Mesa, Dark Ritual, Vampiric Tutor, Silence, Ragavan, Esper Sentinel (on 6) | Keep, bottom Esper Sentinel | 6-card rule: 3 mana cards and a tutor |

### 4. Turns 1-3

| Turn | Usual play | Notes |
|---|---|---|
| 1 | Land, then fast mana (Sol Ring, Mana Vault, Chrome Mox, Mox Diamond). Then one of: Birds or Noble Hierarch (Thanos on turn 2); Dark Ritual into Necropotence; Mystic Remora; Esper Sentinel or Ragavan | Hold Lotus Petal until you need the mana: it's one-shot. Crack fetches at an opponent's end step |
| 2 | Thanos off a mana creature, or Necropotence, Rhystic Study, Grand Abolisher or Voice of Victory (protects turn 3), or a lock you can keep up. Go for the kill if you can win through what the table shows | Instant tutors at the end step of the player before you |
| 3 | The main kill turn: lock first, then the combo with protection behind it (section 6) | Teferi, Time Raveler or Hexing Squelcher a turn early if the mana is spare |

**Go now or wait?** A stopped attempt costs about 2 turns (D1's rule of thumb: win turn ≈ goldfish turn + 2 per
stop), and it often costs the hand too (LED discards it).

- **Go now** when a lock has resolved (Silence, Grand Abolisher, Voice of Victory, Teferi), the blue players are
  tapped low, you hold two or more pieces of protection, or a fast opponent (Rograkh/Silas, Kinnan) is likely to win
  before your next turn.
- **Wait a turn** when you have the combo but no protection, two or more blue players are untapped with full hands,
  and nobody is about to win. Spend the turn on Thanos, a lock creature, Hexing Squelcher or Teferi, and keep a free
  counter for their turn.
- Mystic Remora's cumulative upkeep grows by {1} each turn. Pay it once or twice, then let it go.

### 5. What to tutor for

Instant-speed tutors that put the card on top (Vampiric Tutor, Mystical Tutor, Enlightened Tutor, Worldly Tutor) go at
the end step of the player before you, so you draw the card next. Imperial Seal is a sorcery that also puts it on
top: cast it on your turn for next turn, or at instant speed with Teferi's +1, Valley Floodcaller or Borne Upon a
Wind. On your own turn, a top-of-library tutor followed by Manamorphose, Gitaxian Probe or Borne Upon a Wind draws the
card at once. Under Necropotence you skip the draw step, so top-of-library tutors are slow.

| Board state | Tutor | Target | Why |
|---|---|---|---|
| Oracle in hand, need the exile half, your turn | Demonic Tutor ({1}{B}) or Grim Tutor ({1}{B}{B}, 3 life) | Tainted Pact into Mental Misstep decks, else Demonic Consultation | Pact dodges Misstep; Consultation is a mana cheaper |
| Oracle in hand, need the exile half, opponent's end step | Mystical Tutor ({U}) or Vampiric Tutor ({B}, 2 life) | Consultation or Pact | On top for your draw |
| Exile half in hand, need Oracle | Eladamri's Call ({G}{W}, instant, to hand) at end step; Worldly Tutor ({G}, top); Beseech the Mirror bargained (cast it free) | Thassa's Oracle | Call is the cleanest: hand, instant |
| Breach + LED in hand, need Brain Freeze | Merchant Scroll ({1}{U}, to hand), Mystical Tutor ({U}, top) | Brain Freeze | Scroll finds a blue instant, Mystical any instant or sorcery |
| LED + Brain Freeze, need Breach | Enlightened Tutor ({W}, top) at end step; Demonic Tutor; Beseech bargained | Underworld Breach | Beseech casts Breach free (mana value 2) |
| Two combo pieces, opponents hold free counters | Mystical Tutor, Merchant Scroll, Worldly Tutor, Eladamri's Call | Silence or Orim's Chant (Mystical); Force of Will or Pact (Scroll); Grand Abolisher or Voice of Victory (Call, Worldly) | One lock wins more games than a third combo piece |
| Stax piece blocks the line you hold | Mystical Tutor, Merchant Scroll; any-card tutors for the lands | Snapback, Chain of Vapor, Into the Flood Maw; Boseiju or Otawara | See section 8 for which answer fits which piece |
| Opposition Agent in play | Don't tutor or crack fetches | Answer it first (odd power-up, Snapback, Chain, Flood Maw, Otawara) | They take every card you search for |
| Nothing yet, 3+ mana, sorcery speed | Demonic Tutor | Necropotence on turns 1-2; otherwise the missing piece | Necro finds the rest for life |

**Intuition piles** ({2}{U} instant; an opponent picks one for your hand, the other two go to the graveyard). Name
three cards where any one does the job:

| You hold | Pile | Why it works |
|---|---|---|
| Thassa's Oracle | Demonic Consultation, Tainted Pact, Mystical Tutor | Any of the three gets the exile half |
| Consultation or Pact | Thassa's Oracle, Eladamri's Call, Worldly Tutor | Any of the three gets Oracle |
| Underworld Breach (in hand) | Lion's Eye Diamond, Brain Freeze, Thassa's Oracle | With Breach, a graveyard card can be escaped, so the two they don't give you are still live. You still need three other cards of escape fuel per escape (never Oracle) |
| Combo in hand, need a lock | Silence, Grand Abolisher, Voice of Victory | Each stops every opponent on your turn (Orim's Chant stops only one player) |

**Merchant Scroll** ({1}{U}, blue instant to hand): Brain Freeze; Mystical Tutor (a second hop to Consultation, Pact,
Silence or Orim's Chant); Intuition (a pile above); Force of Will or Pact of Negation (protection); Snapback, Chain of
Vapor or Into the Flood Maw (an answer); Fierce Guardianship when Thanos is out.

**Gamble** ({R}): the random discard comes after the search, from the whole hand. With only the found card in hand,
you discard it. Cast it with three or more other cards in hand, or when the card is fine in the graveyard (Breach in
play, or Noxious Revival in hand). Under Necropotence the discard is exiled.

**Beseech the Mirror** ({1}{B}{B}{B}): bargain by sacrificing an artifact, enchantment or token as you cast it, and you
may cast the found card free if its mana value is 4 or less. Fodder: a Treasure (Lotho, Ragavan), Lotus Petal, a
Mox, Sol Ring, an old Mystic Remora, the Orc Army token. A Mox or Sol Ring can tap for mana while you cast Beseech and
then be sacrificed to bargain (mana abilities come before costs are paid). Best targets: Thassa's Oracle (hold the
exile half), Underworld Breach, Necropotence. Never Demonic Consultation: it would resolve with no Oracle trigger.
Unbargained, or above mana value 4, the card goes to hand. Drannith Magistrate stops the free cast (the card goes to
hand).

**Diabolic Intent** ({1}{B}, sacrifice a creature): tap Birds or Noble Hierarch for mana first, then sacrifice it.
Also good: the Orc Army token, a spent Esper Sentinel or Ragavan, Mockingbird, an Oracle left on the battlefield after
a failed try. Don't sacrifice the lock creature you need this turn, or Ranger-Captain (sacrifice it for its own lock).
Sacrificing Thanos sends him to the command zone with {2} more tax and turns off the free spells.

**Eladamri's Call and Worldly Tutor** find Thassa's Oracle, or a lock creature: Grand Abolisher, Voice of Victory,
Ranger-Captain of Eos (which then fetches a 1-drop: Birds, Hierarch, Esper Sentinel, Ragavan or Mockingbird; not
Oracle, which costs 2). Against Rograkh and mana creatures, Call finds Orcish Bowmasters to flash in.

**Tutor for protection or an answer instead of a combo piece** when you already hold both pieces (or a sure tutor for
the second), when a stax piece blocks the line you hold, or when two or more blue players are untapped with cards.

### 6. Protecting the win turn

Order:

1. **The lock, before you commit a combo piece.** Silence ({W}) stops every opponent; Orim's Chant ({W}) stops one
   (aim it at the player with the most blue mana or cards); Grand Abolisher and Voice of Victory stop everyone on your
   turn and are best cast a turn early; Teferi, Time Raveler stops everyone from casting outside their own main
   phase; Ranger-Captain of Eos (sacrifice) stops noncreature spells, and as an ability it can't be countered by a
   spell.
2. **Veil of Summer in response to a counter** ({G}): your spells can't be countered this turn, so the counter on the
   stack does nothing. It also beats a counter aimed at your Silence. It doesn't stop Mindbreak Trap (exile, not
   counter), a Silence cast before yours, or Stifle.
3. **Then the counter war:** Force of Will (any spell), Pact of Negation (any spell), Fierce Guardianship (noncreature),
   Deflecting Swat (move a targeted counter), Mental Misstep (their 1-mana counters, or a Silence aimed at you),
   Flusterstorm (their instant counter, with your storm count behind it), Red Elemental Blast (any blue counter).

**Timing traps:**

- **Force of Negation** is free only when it isn't your turn. On your kill turn it costs {1}{U}{U}.
- **Mindbreak Trap** is free only once an opponent has cast three or more spells this turn. On your turn that
  rarely happens, so plan on {2}{U}{U}.
- **Pact of Negation:** at your next upkeep you pay {3}{U}{U} or lose. Use it on the turn you win, or when you're sure
  of five mana with {U}{U} next upkeep.
- **Fierce Guardianship, Deflecting Swat and Deadly Rollick** need Thanos on the battlefield when you cast them. Opponents who kill
  him at your upkeep switch them off.
- **Mental Misstep** hits only mana value 1: Swan Song, Flusterstorm, Silence, Orim's Chant, Consultation. Not Force
  of Will, Pact or Fierce.
- **Free spells against Vexing Bauble or Lavinia** (spells cast with no mana are countered): pay mana where you can
  (Fierce for {2}{U}, Snapback for {1}{U}), or have Hexing Squelcher out.

**Hexing Squelcher first** ({1}{R}; it can't be countered): once it's out, spells you control can't be countered. Cast
it first, ideally the turn before, against counter-heavy tables (Kinnan, Kraum/Tymna, Rograkh/Thrasios) and against
Vexing Bauble, Lavinia and Chalice of the Void, whose "counter that spell" it also stops. It does nothing against
Silence or Orim's Chant, Mindbreak Trap's exile, Stifle or removal (ward costs only 2 life), and it dies to your
own even wipe.

### 7. Interaction on opponents' turns

Keep one free counter for their kill. On their turn, Force of Negation is free (exile a blue card), Force of Will is
free, and Mindbreak Trap turns free once they cast their third spell.

| Their combo | Counter the... | With |
|---|---|---|
| Thassa's Oracle + Consultation/Pact (Kraum/Tymna, Rograkh/Silas) | The instant, cast in response to the Oracle trigger | Force of Negation, Swan Song, Flusterstorm, Fierce Guardianship, Mental Misstep (Consultation only), Force of Will, Pact |
| The same, if they cast Consultation first | Thassa's Oracle (a creature) | Force of Will, Pact, Mindbreak Trap, Red Elemental Blast |
| Underworld Breach loops (Kraum/Tymna, Rograkh/Silas) | Breach when it's cast; mid-loop, Brain Freeze | Breach: Swan Song, Force of Negation, Fierce, Force of Will. Mid-loop: Mindbreak Trap (free, exile Brain Freeze and its copies), Flusterstorm, Red Elemental Blast on Brain Freeze |
| Kinnan + Basalt Monolith | The Monolith (artifact), or Kinnan himself | Force of Negation, Fierce, Force of Will, Pact (Swan Song can't hit artifacts); Red Elemental Blast destroys Kinnan; the even power-up |
| Dack Fayden (6-mana creature) | Dack, on the cast | Force of Will, Pact, Mindbreak Trap. Fierce, Force of Negation and Swan Song can't counter a creature. After it resolves: Deadly Rollick or the even power-up |
| Sisay's activation (an ability) | Nothing counters it: remove Sisay first | Deadly Rollick, odd power-up, Snapback, Into the Flood Maw, Chain of Vapor |

- **Swan Song** ({U}) only hits enchantments, instants and sorceries, and gives them a 2/2 flier. Spend it on the
  Consultation, Breach or Ad Nauseam that wins, not on a draw spell.
- **Force of Negation** is your best counter on their turn (free, exiles). Hold it for a kill, not a Rhystic Study.
- **Mindbreak Trap** is the answer to storm and Breach loops: by the time they loop it's free, and it exiles any
  number of spells, including ones that can't be countered.
- **Silence or Orim's Chant at their upkeep** stops their whole turn of spells. Use it when they're the one about to
  win and you can't win first.
- **Red Elemental Blast** also destroys blue permanents: Kinnan, Thrasios, Kraum, Silas Renn, an opposing Teferi.
- Pact of Negation on an opponent's turn means paying {3}{U}{U} in your coming upkeep: only to stop a win you'd lose
  to anyway.

### 8. Answering stax

Bounce is enough on the kill turn: once Oracle resolves, it doesn't matter that the piece comes back. Chain of Vapor
({U}): any nonland permanent, but the owner may sacrifice a land to copy it and bounce one of yours, so cast it before
the combo starts. Into the Flood Maw ({U}): a creature, or any nonland permanent if you gift the opponent a tapped
1/1 Fish. Snapback: a creature, free by exiling a blue card. Boseiju channel ({1}{G}, {1} less per legendary creature
you control): destroys an artifact, enchantment or nonbasic land. Otawara channel ({3}{U}, same discount): bounces an
artifact, creature, enchantment or planeswalker. Teferi's -3: an artifact, creature or enchantment, and draws.
Boseiju and Otawara work from hand and are abilities, not spells.

| Stax piece (opponent's) | What it stops for us | Answer | Line that still works |
|---|---|---|---|
| Drannith Magistrate | Breach escapes, recasting Thanos from the command zone, Beseech's free cast | Deadly Rollick, Snapback, Chain, Flood Maw, Otawara, Teferi -3, even power-up | Oracle |
| Rest in Peace | Breach (graveyards are exiled), Noxious Revival | Chain, Flood Maw (gift), Boseiju, Otawara, Teferi -3 | Oracle |
| Grafdigger's Cage | Breach (no casting from graveyards) | Chain, Flood Maw (gift), Boseiju, Otawara, Teferi -3 | Oracle |
| Dauthi Voidwalker | Breach (our cards are exiled instead of going to the graveyard) | Deadly Rollick, Snapback, Chain, Flood Maw, Otawara, Teferi -3, even power-up | Oracle |
| Null Rod / Collector Ouphe | All artifact mana: Sol Ring, Mana Vault, Grim Monolith, the Moxen, Lotus Petal, LED, Treasures | Null Rod: Chain, Flood Maw (gift), Boseiju, Otawara, Teferi -3. Ouphe: Deadly Rollick, Snapback, Chain, Flood Maw, Otawara, Teferi -3, even power-up | Oracle (off lands, rituals, mana creatures); Breach is dead without LED |
| Cursed Totem | Birds and Hierarch's mana, the power-up, Ranger-Captain's sacrifice | Chain, Flood Maw (gift), Boseiju, Otawara, Teferi -3 | Both (spirit guides still work from hand) |
| Archon of Emeria | One spell per player per turn; your fetched duals enter tapped | Abilities: Otawara, the odd power-up. Or spend the turn's spell on Deadly Rollick, Snapback, Chain or Flood Maw and win next turn | Oracle only with Necropotence out: Consultation or Pact at an opponent's end step, then Oracle on your turn (Necro skips the draw step that would otherwise lose you the game). If Oracle is countered, your library is gone. Breach is dead |
| Deafening Silence | One noncreature spell per turn | Boseiju, Otawara (abilities); Teferi -3 (Teferi uses the turn's spell) | Oracle (a creature) + one Consultation or Pact; Breach is dead |
| Blood Moon / Magus of the Moon | Every land taps only for {R} (no basics); fetches, Cavern and Gemstone lose their abilities | Boseiju from hand ({1}{G}: G from Birds, Hierarch, Elvish Spirit Guide, a Mox, Petal, or Manamorphose paid with {R}), Otawara from hand, Chain, Flood Maw (gift), Teferi -3. Magus: Deadly Rollick, Snapback, odd power-up | Breach (Breach is red; LED makes blue). Oracle needs {U}{U} from Manamorphose, Moxen, Petal or mana creatures |
| Chalice of the Void (Dack) | X=0: LED, Moxen, Petal, Pact. X=1: Consultation, Dark Ritual, Silence, Orim's Chant, the 1-mana tutors and counters | Boseiju, Otawara, Teferi -3, Hexing Squelcher; Chain or Flood Maw if X isn't 1 | X=1: Oracle with Tainted Pact. X=0: Oracle; Breach is dead without Squelcher |
| Trinisphere (Dack) | Every spell costs at least 3: free spells and rituals stop working | Boseiju, Otawara, Teferi -3 (Chain and Flood Maw cost 3) | Oracle at 6 mana |
| Damping Sphere (Dack) / Defense Grid (Dack, Rograkh/Silas) | Sphere: each spell costs {1} more per other spell you cast this turn, Ancient Tomb makes {C}. Grid: {3} more on opponents' turns, so your free counters cost 3 there | Boseiju, Otawara, Chain, Flood Maw (gift), Teferi -3 | Oracle (two spells); Breach is dead under Sphere |
| Vexing Bauble (Rograkh/Silas, Dack) / Lavinia (Sisay) | Spells cast with no mana are countered: alternative costs, Pact, the free spells, LED, Moxen, Petal, Phyrexian mana paid with life. Lavinia also stops noncreature spells with mana value above your land count | Hexing Squelcher; pay mana. Bauble: Chain, Flood Maw (gift), Boseiju, Otawara, Teferi -3. Lavinia: Snapback or Deadly Rollick hard-cast, Chain, Flood Maw, Otawara, Teferi -3, even power-up | Oracle (paid with mana) |
| Opposition Agent | Your tutors and fetches: they take the card | Deadly Rollick, Snapback, Chain, Flood Maw, Otawara, Teferi -3, odd power-up | Oracle from hand (Consultation and Pact don't search) |
| Opposing Teferi, Time Raveler (Sisay) | You can cast spells only at sorcery speed, so you can't cast Consultation with the Oracle trigger on the stack | Chain, Flood Maw (gift), Otawara, Red Elemental Blast (on your main phase) | Breach (each escape at sorcery speed); Oracle only with the risky order |

### 9. Matchups

The Tier 6 field from the design doc's meta model. The sim (10/30 list, loose profile) has Kraum/Tymna landing the
finishing blow in 14.8% of games, Sisay 13.7%, Kinnan 11.2% and Dack Fayden 11.1%. Its stopped lines were mostly
stopped with no card named (a cost, a ward or a lock: 80 times), then about 20 times each by Force of Will, Swan
Song, Mental Misstep, Fierce Guardianship and Force of Negation.

| Opponent | Race or hold? | Hold up | Parity | Cards that matter |
|---|---|---|---|---|
| Kraum/Tymna (blue midrange, Oracle and Breach) | Go first; they wait for you to tap out. Lead with a lock | A free counter for their Consultation or Breach | Odd (Kraum 5, Tymna 3) | Silence, Grand Abolisher, Voice of Victory, Veil of Summer, Hexing Squelcher; their Red Elemental Blast hits Oracle and Brain Freeze: cast Oracle off Cavern of Souls, or behind Veil |
| Kinnan (fast mana combo, turn 3-5) | Race, but hold one counter for Basalt Monolith or Kinnan | Force of Negation, Fierce Guardianship; Red Elemental Blast for Kinnan | Even (Kinnan 2) | Orcish Bowmasters on their mana creatures; their Endurance can undo a Breach loop; Hexing Squelcher against their free counters |
| Rograkh/Thrasios (creature mana loops) | Race; they need Gaea's Cradle and creature loops | A free counter for the loop piece | Even (Rograkh 0, Thrasios 2) | Orcish Bowmasters kills Rograkh; the even power-up breaks their board; their Veil of Summer beats your counters, so answer the creature instead |
| Rograkh/Silas (turbo Oracle and Breach, turn 1-3) | The true race. Keep a free counter up from turn 1 | Force of Negation, Mindbreak Trap, Flusterstorm | Rarely worth the mana; odd kills Silas | Their Vexing Bauble and Pyroblast punish free spells and Oracle; Hexing Squelcher; Mental Misstep on their Consultation and rituals |
| Sisay (legends toolbox) | Race; its win threat appears around turn 3-4 | Removal for Sisay (Deadly Rollick, Snapback, Flood Maw), not counters: the legends arrive without being cast | Odd (Sisay 3), even for Lavinia | Its Lavinia, Teferi, Deafening Silence and Esper Sentinel tax you; Hexing Squelcher against Lavinia |
| Dack Fayden (white stax) | Race before the stax builds | Force of Will, Pact or Mindbreak Trap for Dack himself | Even (Dack 6) | Its Silence and Orim's Chant come before Dack; Chalice, Trinisphere, Damping Sphere, Defense Grid, Cursed Totem and Vexing Bauble call for Boseiju, Otawara and Hexing Squelcher |

### 10. After a stopped attempt

| What happened | Rebuild |
|---|---|
| Oracle countered into the graveyard | Noxious Revival ({G} or 2 life) puts it on top: draw it next turn, or this turn with Manamorphose, Gitaxian Probe or Borne Upon a Wind. With Breach out, escape it. Force of Negation and Mindbreak Trap exile instead: no Revival |
| Consultation or Pact countered | The other one is still in the deck: tutor for it, or Revival the countered one |
| Oracle resolved, exile half countered | Oracle stays: next turn Mockingbird for X=1 ({1}{U}) copies it, and you cast the exile half in response to the copy's trigger |
| Library exiled, trigger countered | Win this turn or lose at your draw: Mockingbird, or bounce and recast Oracle (section 1) |
| Breach countered | Revival on Breach, or switch to Oracle |
| LED cracked and the loop stopped | The hand is gone: Necropotence, Ad Nauseam, Rhystic Study and the remaining tutors rebuild. Escaped cards stay in the graveyard for the next Breach |
| Thanos removed | Recast him for {R}{W}{B} plus {2} per previous cast; his new object can power up again |

- **Switch lines** when the hate fits one line: graveyard hate, Drannith or Dauthi means Oracle; artifact hate (Null
  Rod, Ouphe) or a key Oracle piece exiled means Breach. Thassa's Oracle exiled ends the Oracle line (Mockingbird
  needs an Oracle on the battlefield); Breach then has to finish by milling the opponents.
- A Pact of Negation cast in the failed turn must be paid at your next upkeep. Count that before the attempt.
- After a stop, the next try needs more protection than the last: the table has seen your plan.

### 11. Life management

| Source | Cost |
|---|---|
| Fetch lands | 1 each |
| Ancient Tomb | 2 per tap |
| Mana Confluence | 1 per tap |
| Mana Vault | 1 per draw step while tapped |
| Vampiric Tutor, Imperial Seal | 2 each |
| Grim Tutor | 3 |
| Gitaxian Probe, Mental Misstep, Noxious Revival | 2 each if paid with life |
| Force of Will | 1 |
| Necropotence | 1 per card |
| Ad Nauseam | Each card's mana value (about 1.3 on average; Force of Will 5) |
| Sylvan Library | 4 per extra card kept |
| Lotho, Corrupt Shirriff | 1 each time any player casts their second spell in a turn (that's also a Treasure) |

**Rough budget for a turn-3 kill at 40 life:** the fixed costs (two fetches, a few Tomb or Confluence taps, one
life-paid tutor, a Phyrexian spell) come to about 10-15. That leaves about 25 for Necropotence or Ad Nauseam. Don't
drop below about 10 before your kill turn: Ragavan, Kraum and Tymna's attacks and your own later costs eat the rest.
Thanos's lifelink gets some back. Lotho can quietly cost 5 or more a round at a busy table; it pays you a Treasure
each time, so keep it unless life is short.

### 12. Common mistakes

1. Casting Chain of Vapor mid-combo: the opponent sacrifices a land, copies it and bounces your Breach or Oracle.
2. Casting Demonic Consultation before Thassa's Oracle.
3. Cracking Lion's Eye Diamond with cards you need still in hand, before Breach has resolved, or with Necropotence
   out (Necro exiles everything discarded). Discarded lands can never be escaped.
4. Choosing even with your own Grand Abolisher or Voice of Victory as this turn's lock, or with Oracle still on the
   battlefield for a Mockingbird copy.
5. Counting Force of Negation or Mindbreak Trap as free on your own turn.
6. Casting Pact of Negation without five mana ({3}{U}{U}) for your next upkeep, on a turn you might not win.
7. Paying Necropotence for more cards than you can keep: cleanup discards down to 7, and Necro exiles the rest. It
   also switches off Sylvan Library (no draw step).
8. Casting Gamble with only the found card in hand.
9. Tutoring or cracking fetches while Opposition Agent is on the battlefield.
10. Tapping out for Thanos on the turn the kill was available, or wiping instead of winning with six mana and the
    combo in hand.
11. Milling yourself out with Brain Freeze before Thassa's Oracle is safe in hand or the graveyard.
12. Using a top-of-library tutor on your own main phase with no way to draw the card this turn.

## Bracket and Rule 0

Bracket 5 (cEDH): 23 Game Changers, Thassa's Oracle and Underworld Breach combos, free counters, Silence effects.

## Decklist by type (100)
**Commander (1).** Thanos, the Mad Titan

**Creatures (15).** Birds of Paradise, Elvish Spirit Guide, Esper Sentinel, Grand Abolisher, Hexing Squelcher, Lotho, Corrupt Shirriff, Mockingbird, Noble Hierarch, Orcish Bowmasters, Ragavan, Nimble Pilferer, Ranger-Captain of Eos, Simian Spirit Guide, Thassa's Oracle, Valley Floodcaller, Voice of Victory

**Enchantments (5).** Mystic Remora, Necropotence, Rhystic Study, Sylvan Library, Underworld Breach

**Artifacts (7).** Chrome Mox, Grim Monolith, Lion's Eye Diamond, Lotus Petal, Mana Vault, Mox Diamond, Sol Ring

**Sorceries (10).** Beseech the Mirror, Demonic Tutor, Diabolic Intent, Gamble, Gitaxian Probe, Grim Tutor, Imperial Seal, Jeska's Will, Merchant Scroll, Rite of Flame

**Instants (34).** Ad Nauseam, Borne Upon a Wind, Brain Freeze, Cabal Ritual, Chain of Vapor, Dark Ritual, Deadly Rollick, Deflecting Swat, Demonic Consultation, Desperate Ritual, Eladamri's Call, Enlightened Tutor, Fierce Guardianship, Flusterstorm, Force of Negation, Force of Will, Into the Flood Maw, Intuition, Manamorphose, Mental Misstep, Mindbreak Trap, Mystical Tutor, Noxious Revival, Orim's Chant, Pact of Negation, Red Elemental Blast, Seething Song, Silence, Snapback, Swan Song, Tainted Pact, Vampiric Tutor, Veil of Summer, Worldly Tutor

**Lands (27).** Ancient Tomb, Arid Mesa, Badlands, Bayou, Bloodstained Mire, Boseiju, Who Endures, Cavern of Souls, Command Tower, Flooded Strand, Gemstone Caverns, Mana Confluence, Marsh Flats, Misty Rainforest, Otawara, Soaring City, Plateau, Polluted Delta, Savannah, Scalding Tarn, Scrubland, Taiga, Tropical Island, Tundra, Underground Sea, Verdant Catacombs, Volcanic Island, Windswept Heath, Wooded Foothills

**Planeswalkers (1).** Teferi, Time Raveler

## Import list (100)

```
1 Ad Nauseam
1 Ancient Tomb
1 Arid Mesa
1 Badlands
1 Bayou
1 Beseech the Mirror
1 Birds of Paradise
1 Bloodstained Mire
1 Borne Upon a Wind
1 Boseiju, Who Endures
1 Brain Freeze
1 Cabal Ritual
1 Cavern of Souls
1 Chain of Vapor
1 Chrome Mox
1 Command Tower
1 Dark Ritual
1 Deadly Rollick
1 Deflecting Swat
1 Demonic Consultation
1 Demonic Tutor
1 Desperate Ritual
1 Diabolic Intent
1 Eladamri's Call
1 Elvish Spirit Guide
1 Enlightened Tutor
1 Esper Sentinel
1 Fierce Guardianship
1 Flooded Strand
1 Flusterstorm
1 Force of Negation
1 Force of Will
1 Gamble
1 Gemstone Caverns
1 Gitaxian Probe
1 Grand Abolisher
1 Grim Monolith
1 Grim Tutor
1 Hexing Squelcher
1 Imperial Seal
1 Into the Flood Maw
1 Intuition
1 Jeska's Will
1 Lion's Eye Diamond
1 Lotho, Corrupt Shirriff
1 Lotus Petal
1 Mana Confluence
1 Mana Vault
1 Manamorphose
1 Marsh Flats
1 Mental Misstep
1 Merchant Scroll
1 Mindbreak Trap
1 Misty Rainforest
1 Mockingbird
1 Mox Diamond
1 Mystic Remora
1 Mystical Tutor
1 Necropotence
1 Noble Hierarch
1 Noxious Revival
1 Orcish Bowmasters
1 Orim's Chant
1 Otawara, Soaring City
1 Pact of Negation
1 Plateau
1 Polluted Delta
1 Ragavan, Nimble Pilferer
1 Ranger-Captain of Eos
1 Red Elemental Blast
1 Rhystic Study
1 Rite of Flame
1 Savannah
1 Scalding Tarn
1 Scrubland
1 Seething Song
1 Silence
1 Simian Spirit Guide
1 Snapback
1 Sol Ring
1 Swan Song
1 Sylvan Library
1 Taiga
1 Tainted Pact
1 Teferi, Time Raveler
1 Thanos, the Mad Titan
1 Thassa's Oracle
1 Tropical Island
1 Tundra
1 Underground Sea
1 Underworld Breach
1 Valley Floodcaller
1 Vampiric Tutor
1 Veil of Summer
1 Verdant Catacombs
1 Voice of Victory
1 Volcanic Island
1 Windswept Heath
1 Wooded Foothills
1 Worldly Tutor
```
