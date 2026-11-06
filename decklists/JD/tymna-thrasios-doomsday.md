# Tymna the Weaver & Thrasios, Triton Hero — Four-Colour Doomsday (W/U/B/G, cEDH)

> Built 2026-11-05 from JD's Thanos turbo deck and the Tymna/Thrasios partners deck. Thanos's cEDH core moves under
> Tymna/Thrasios, the red cards and the Breach line come out, and Doomsday plus three cantrips go in. The card moves are
> in `documents/doomsday-thanos-rebuild-swaps.md`. Not simulated yet; Doomsday has card code and pile logic since 2026-11-05 (`rust/crates/sim-core/src/impls/doomsday.rs`).

Thassa's Oracle wins the game three ways here: with **Demonic Consultation** or **Tainted Pact** (two cards, three or
four mana), out of a five-card **Doomsday** pile, or after Thrasios digs through the library on unlimited mana. Fast
mana, tutors and the free counters from the Thanos deck make each try cheap and well protected. The two commanders
cost two and three mana, so they're on the battlefield early. That turns on Fierce Guardianship and Deadly Rollick and
gives the deck card draw that doesn't depend on any one spell resolving. It is weak to a resolved Silence before its
own kill turn, to opposing Mindbreak Trap and Red Elemental Blast on Thassa's Oracle, and to artifact hate when the
hand leans on fast mana.

| | |
|---|---|
| **Commanders** | Tymna the Weaver ({1}{W}{B}, 2/2 lifelink) and Thrasios, Triton Hero ({G}{U}, 1/1); white-blue-black-green |
| **Bracket** | 5 (cEDH): 22 Game Changers, Thassa's Oracle combos, free counters, Silence effects |
| **Game Changers (22)** | Ad Nauseam, Ancient Tomb, Chrome Mox, Demonic Tutor, Enlightened Tutor, Fierce Guardianship, Force of Will, Gifts Ungiven, Grim Monolith, Imperial Seal, Intuition, Lion's Eye Diamond, Mana Vault, Mox Diamond, Mystical Tutor, Necropotence, Orcish Bowmasters, Rhystic Study, Smothering Tithe, Thassa's Oracle, Vampiric Tutor, Worldly Tutor |
| **Simulator** | Deck key `thrasios`, tier T6 (cEDH), played from `data/plans/thrasios.toml`; findings in `audit/thrasios/FINDINGS.md` (no sims yet). Replaced the old `thrasios` list (T5) on 2026-11-05. Doomsday has card code, a pile builder and the play of the pile (`impls/doomsday.rs`, 2026-11-05); none of it simulated yet |
| **Buys** | Doomsday, Brainstorm, Ponder, Preordain (none are in the binder or bulk lists) |

## Strategy

### The commanders

**Tymna the Weaver** ({1}{W}{B}, 2/2 lifelink). *At the beginning of each of your postcombat main phases, you may pay X
life, where X is the number of opponents that were dealt combat damage this turn. If you do, draw X cards.*

**Thrasios, Triton Hero** ({G}{U}, 1/1). *{4}: Scry 1, then reveal the top card of your library. If it's a land card, put
it onto the battlefield tapped. Otherwise, draw a card.*

Both have plain Partner and share the command zone. In this deck they do four jobs:

- **They turn on the free spells.** Fierce Guardianship (counter a noncreature spell) and Deadly Rollick (exile a
  creature) cost nothing *if you control a commander*: either one counts. With two cheap commanders, one of them is
  usually out from turn 2. Each is checked when you cast the spell, so if an opponent kills your last commander at
  your upkeep, both cost full price that turn.
- **Thrasios is a mana sink and a dig engine.** Each spare {4} at an opponent's end step becomes a land or a card.
  With **Basalt Monolith + Rings of Brighthearth** (unlimited colourless mana) he draws the whole library (see Line 4).
- **Thrasios adds 1 to your devotion to blue** ({G}{U} has one blue symbol). Thassa's Oracle counts it, so each
  Doomsday pile needs one draw fewer while he's out.
- **Tymna draws off small attacks.** A 1/1 mana creature or Tymna herself hitting two opponents is two cards for two
  life. At a cEDH table the boards are small and blockers rare, so on a turn with nothing better, attack with whatever
  can get through and let her draw. Don't attack into a deathtouch or first-strike blocker you need to keep alive.

**When to cast them.** Thrasios on turn 1 off a land plus a Mox or Lotus Petal ({G}{U} between them), otherwise on
turn 2. Mana creatures can't tap on the turn they arrive, so a turn-1 Birds or Hierarch means a turn-2 commander with
mana to spare. Tymna on turn 2-3. Never cast a commander instead of holding the mana for a kill or for protection on a
turn when the table is about to win. Each recast costs {2} more per previous cast (commander tax), so a commander that
has died twice is a luxury.

### How the deck wins

#### Line 1: Thassa's Oracle + Demonic Consultation or Tainted Pact

Oracle {U}{U} plus Consultation {B} (3 mana total) or Tainted Pact {1}{B} (4 mana).

1. Cast **Thassa's Oracle** and let it resolve. Its enter trigger goes on the stack.
2. With the trigger on the stack, cast **Demonic Consultation** or **Tainted Pact** (both instants).
   - **Consultation:** name a card that isn't in your library. Thassa's Oracle is a safe name: it's on the
     battlefield. Consultation exiles the top six, then reveals until it finds the name, so it exiles the whole
     library.
   - **Tainted Pact:** decline every card. The deck is singleton, so two cards with the same name never come up, and it
     exiles the whole library.
3. The trigger resolves: X (your devotion to blue) is at least 0, the number of cards in your library, and you win.
   Removing Oracle in response doesn't stop it: X can be 0, and 0 ≥ 0 still wins.

**Why Oracle goes first:** if Oracle is countered, Consultation is still in hand and you've lost one card, not the
game. Cast Consultation first and a counter on Oracle leaves you with no library, losing at your next draw.

| Piece | Stopped by (opponents' cards) | Not stopped by |
|---|---|---|
| Thassa's Oracle (blue creature, mana value 2) | Force of Will, Pact of Negation, Mindbreak Trap (exiles it), Red Elemental Blast / Pyroblast | Swan Song, Flusterstorm, Fierce Guardianship, Force of Negation, Mental Misstep (noncreature-only, or mana value 1) |
| Demonic Consultation (black instant, mana value 1) | Force of Will, Pact of Negation, Mindbreak Trap, Swan Song, Flusterstorm, Fierce Guardianship, Force of Negation, Mental Misstep | Red Elemental Blast / Pyroblast (it's black) |
| Tainted Pact (black instant, mana value 2) | The same as Consultation, except Mental Misstep | Red Elemental Blast / Pyroblast, Mental Misstep |
| The trigger | Stifle-type effects (counter an ability) | Every counterspell |

- **Cavern of Souls naming Wizard** makes Oracle uncounterable (Mindbreak Trap still exiles it). Pay {U}{U} with
  Cavern's coloured mana plus one other blue source. Thrasios and Valley Floodcaller are Wizards too.
- **If the trigger is countered after the library is gone,** you lose at your next draw. Outs this turn: Mockingbird
  with X=1 ({1}{U}) enters as a copy of the Oracle still on the battlefield and gives a new trigger; or bounce your own
  Oracle with Snapback (free by exiling a blue card from hand) or Chain of Vapor and recast it for {U}{U}. Don't use
  Teferi's −3 here: it draws a card from the empty library and you lose.

#### Line 2: Doomsday

*Doomsday* ({B}{B}{B}, **sorcery**): search your library and graveyard for five cards and exile the rest. Put the
chosen cards on top of your library in any order. You lose half your life, rounded up.

After it resolves, your library is exactly the five cards you chose, in the order you chose. Your graveyard is exiled
too. Thassa's Oracle wins when your devotion to blue is at least the number of cards left in the library, so the pile's
job is to draw you down to that number and then cast Oracle with protection.

**Counting devotion.** Oracle alone is 2 ({U}{U}). Add one for each {U} symbol in the mana costs of your other
permanents: Thrasios, Valley Floodcaller, Mockingbird, Mystic Remora, Rhystic Study and Teferi, Time Raveler are 1 each.
Count before you build the pile. Every extra point is one draw you don't need.

**Doomsday is a sorcery, and the timing matters.** Casting it at the end step of the player before you means your
draw step takes the first card off the pile, and you start your turn with all your mana. The deck has three ways to
do that:

- **Teferi, Time Raveler +1:** until your next turn you may cast sorceries as though they had flash.
- **Valley Floodcaller** ({2}{U}, flash): you may cast noncreature spells as though they had flash while it's out.
- **Borne Upon a Wind** ({1}{U}, instant): spells this turn have flash, and you draw a card.

Without one of them, cast Doomsday in your own main phase and use a pile that starts with a draw spell from your hand
(Pile C).

**Lion's Eye Diamond pays for Doomsday.** Cast Doomsday, and while you pay for it crack LED (discard your hand,
sacrifice LED) for {B}{B}{B}. Doomsday is already on the stack, so it isn't discarded, and Doomsday searches your
graveyard too: the cards LED just discarded can still go into the pile. Use it at the end step (Pile A): an emptied hand
in your own main phase has nothing to draw the pile with. LED can also pay for a spell inside the pile once the hand
holds nothing the line still needs. Not with Necropotence out (it exiles the discards), and not if a tax makes Doomsday
cost more than {B}{B}{B}.

**Life:** Doomsday takes half your life, rounded up: 40 → 20, 25 → 12 (lose 13). It's an all-or-nothing move: the
pile wins this turn or you lose, so your life total and the opponents' boards don't stop it. Two limits only: at 1 life
it takes your last point, and a Gitaxian Probe paid in life needs at least 3 life when you cast it (2 for the Probe, 1 to stay alive).

**The piles.** Cards are listed top to bottom. "Lock" means **Silence** ({W}: opponents can't cast spells this turn),
cast before Thassa's Oracle. **Veil of Summer** ({G}: your spells can't be countered this turn) can stand in for the
lock: cast it before Oracle, or in response to the counter. It stops every counter that turn, red blasts included, but
not Mindbreak Trap (it exiles, it doesn't counter): against a likely Trap, keep Silence as the lock (Trap is a spell, so
Silence stops it).

| Pile | When | Top → bottom | How it goes | Cost after Doomsday |
|---|---|---|---|---|
| **A** | End step of the player before you; **Oracle not in hand** | Brainstorm, Thassa's Oracle, Gitaxian Probe, Silence, any card | Draw step: Brainstorm (4 left). Cast Brainstorm: draw Oracle, Probe and Silence (1 left), put two other cards from hand back (3 left). Cast Silence. Cast Probe for 2 life: draw one (2 left). Cast Oracle: devotion 2 ≥ 2, you win | {U}, {W}, {U}{U}, 2 life |
| **B** | End step of the player before you; **Oracle in hand** | Gitaxian Probe, Preordain, Silence, any card, any card | Draw step: Probe (4 left). Probe for 2 life: draw Preordain (3 left). Preordain: scry 2, keep Silence on top, draw it (2 left). Cast Silence, then Oracle: 2 ≥ 2, you win | {U}, {W}, {U}{U}, 2 life |
| **C** | Your own main phase; **Oracle and Gitaxian Probe (or Borne Upon a Wind) in hand** | Preordain, Ponder, Silence, any card, any card | Probe draws Preordain (4 left). Preordain: scry 2, draw Ponder (3 left). Ponder: look at the top three, put Silence on top, draw it (2 left). Silence, then Oracle: 2 ≥ 2, you win | {B}{B}{B} for Doomsday, then {U}, {U}, {W}, {U}{U} and 2 life (Borne costs {1}{U} instead of 2 life) |

- **With Thrasios out (devotion 3), each pile needs one draw fewer.** In Pile A, skip the Probe: Oracle wins with 3
  cards left. In Pile B, skip Preordain and put Silence second, so the Probe draws it. In Pile C, put Silence second,
  so Preordain draws it, and Oracle wins with 3 left.
- **Pile A needs two other cards in hand for Brainstorm's put-back.** If Oracle, Probe and Silence are the only
  cards in your hand, you have to put back two of them. Put back Silence and Oracle (Oracle on top). The Probe draws
  Oracle, and you win without the lock.
- **The "any card" slots are never drawn.** Fill them with cards you can afford to lose for good, such as lands or
  spent tutors. The rest of your library is exiled anyway.
- **Under Necropotence you skip your draw step,** so Piles A and B don't work. Use Pile C.
- **Your own Esper Sentinel can force a draw.** When an opponent casts their first noncreature spell in a turn, you
  draw unless they pay; they can let you draw, and that card comes off the pile. That never hurts a pile you finish
  this turn, but don't hold a pile across an opponent's turn. An opponent's Esper Sentinel is a tax, not a draw: your
  first noncreature spell each turn (in the main phase, Doomsday itself) gives its controller a card unless you pay
  {X}, X being its power.

**When Doomsday is the right line.** Line 1 is cheaper. Doomsday is for when you hold Thassa's Oracle but neither
Consultation nor Pact (Pile B or C), or neither half but Doomsday plus a flash enabler (Pile A). In effect it is a
fourth tutor for the combo that also wins.

**What can go wrong:**

- **Doomsday is countered:** you lose the card and {B}{B}{B}. Your library and graveyard are untouched. It's a
  sorcery, so Swan Song, Flusterstorm, Fierce Guardianship and Force of Negation all hit it.
- **A cantrip in the pile is countered:** the pile stalls with too many cards left. Each cantrip is a spell an
  opponent can answer, which is why the lock comes first where the pile allows it. Hold Force of Will or Pact of
  Negation for the step that matters most, and don't start a pile into two untapped blue players with no lock or
  counter of your own.
- **Oracle is countered into the graveyard after Doomsday:** you're left with a tiny library. **Noxious Revival**
  ({G} or 2 life, instant) puts Oracle back on top. Draw it with the pile's cantrips or next turn and recast it. Force
  of Negation and Mindbreak Trap exile Oracle instead, and then the pile is dead. With 2 cards left you survive two more
  draw steps.
- **Oracle's trigger is countered (Stifle):** Mockingbird with X=1 copies the Oracle on the battlefield for a new
  trigger.

#### Line 3: Necropotence and Ad Nauseam into Line 1

| Card | Cost | Life per card | When | Watch out |
|---|---|---|---|---|
| Necropotence | {B}{B}{B} (turn 1 with Dark Ritual) | 1 per card | Main phase 1; the cards arrive at the beginning of this turn's end step | You discard down to 7 at cleanup, and Necro exiles the discards. It also skips your draw step, which switches off Sylvan Library and Doomsday Piles A and B |
| Ad Nauseam | {3}{B}{B}, instant | Each card's mana value | End step of the player before you, or your own turn with the combo mana up | Force of Will costs 5, Mindbreak Trap 4, Beseech the Mirror 4. 27 lands cost 0 |

- **Ad Nauseam rough budget:** 10 cards cost about 13-14 life. Stop around 10 life on an opponent's end step. On your
  kill turn, keep only what the rest of the kill costs.
- Never cast Ad Nauseam after Doomsday: it reveals only your pile.

#### Line 4: Basalt Monolith + Rings of Brighthearth with Thrasios or Walking Ballista

- **Basalt Monolith** ({3}, taps for {C}{C}{C}, {3}: untap) and **Rings of Brighthearth** ({3}: whenever you
  activate a non-mana ability, pay {2} to copy it) make unlimited colourless mana. Tap the Monolith for three, activate
  its untap ({3}), and pay {2} to copy it. The copy resolves first and untaps the Monolith, so tap it again with the
  original still on the stack. Then the original untaps it once more. That's {6} for {5}, one extra mana each loop. The
  first loop needs five mana: the Monolith's own three plus two more.
- **Finish A: Walking Ballista** ({X}{X}). Cast it for a huge X and remove the counters, one damage each, at each
  opponent.
- **Finish B: Thrasios.** Each {4} draws a card or puts a land onto the battlefield. Dig until the library is at or
  below your devotion to blue, then cast Thassa's Oracle (it needs {U}{U} from coloured sources). **Don't activate
  Thrasios with an empty library:** he would draw from it and you'd lose. Stop when the Oracle or the Ballista is in
  hand.
- The loop is two artifacts that do nothing alone. Hold them for a turn when an opponent can't answer one in response.
  Enlightened Tutor finds either piece.

### Game plan

**Turns 1-2.** Land, fast mana (Sol Ring, Mana Vault, Chrome Mox, Mox Diamond, Arcane Signet), then one of: a mana
creature (Birds of Paradise, Noble Hierarch, Avacyn's Pilgrim) for a turn-2 commander; Dark Ritual into Necropotence;
Mystic Remora, Rhystic Study or Esper Sentinel; or Thrasios. Hold Lotus Petal until you need it: it's one use. Crack
fetches at an opponent's end step.

**Turn 2-3: the usual kill turn.** Lock first (Silence, Grand Abolisher, Voice of Victory, Teferi), then the combo with
protection behind it. A commander on the battlefield keeps Fierce Guardianship free.

**Go now or wait?** A stopped attempt costs about two turns, and often your hand.

- **Go now** when a lock has resolved, the blue players are tapped low, you hold two or more pieces of protection, or a
  fast opponent (Rograkh/Silas, Kinnan) is likely to win before your next turn.
- **Wait a turn** when you have the combo but no protection, two or more blue players are untapped with full hands, and
  nobody is about to win. Spend the turn on a commander, a lock creature or Teferi, and keep a free counter up for their
  turn.
- Mystic Remora's cumulative upkeep grows by {1} each turn. Pay it once or twice, then let it go.

### Mulligans

The first mulligan is free (you see two 7-card hands). After that it's a London mulligan: draw 7, put one card on the
bottom for each mulligan after the free one.

| Hand | Keep when |
|---|---|
| 7 cards | 1-4 lands (0 lands only with 3+ fast mana; 5+ lands go back); 3+ mana by turn 2 from lands, rocks, rituals and Elvish Spirit Guide (Mox Diamond needs a spare land, Chrome Mox a spare nonland card); and a reason to go: a win-line card (Thassa's Oracle, Consultation, Tainted Pact, Doomsday), a tutor, or Necropotence, Ad Nauseam, Borne Upon a Wind, Gitaxian Probe, Brainstorm, Ponder, Preordain or Sylvan Library |
| 6 or 5 cards | 1-5 lands, 3+ cards that make mana, and any of the cards above |
| 4 cards | Keep |

- **Protection isn't a keep requirement.** It's in hand on most kill turns anyway, and a hand with only protection
  never threatens to win.
- **Count mana and colours.** A hand with no coloured source for its spells goes back even if the count passes.
  Doomsday needs {B}{B}{B}: Dark Ritual turns one {B} into three.
- **Bottom first:** lands past the third, then the disruption creatures (Esper Sentinel, Lotho, Orcish Bowmasters),
  then the flex cards (Diabolic Intent, Into the Flood Maw). Keep win-line cards and tutors.
- **Gemstone Caverns:** if it's in your kept hand and you aren't the starting player, you may start with it on the
  battlefield with a luck counter (taps for any colour) by exiling a card from your hand. Exile what you'd bottom.

| # | Hand | Call | Why |
|---|---|---|---|
| 1 | Underground Sea, Polluted Delta, Sol Ring, Thassa's Oracle, Demonic Consultation, Force of Will, Brainstorm | Keep | Both halves of Line 1 and protection; Oracle can come down on turn 2 |
| 2 | Bayou, Scalding Tarn, Dark Ritual, Doomsday, Valley Floodcaller, Gitaxian Probe, Preordain | Keep | Floodcaller lets Doomsday go at an end step; Dark Ritual makes the {B}{B}{B}; the cantrips are Pile B or C pieces |
| 3 | Tundra, Mox Diamond, Chrome Mox, Silence, Force of Will, Pact of Negation, Fierce Guardianship | Ship | All protection, no line card or tutor, and Mox Diamond has no spare land |
| 4 | Ancient Tomb, Sol Ring, Mana Vault, Grim Monolith, Ad Nauseam, Doomsday, Tainted Pact | Ship | Lots of mana, but none of it coloured: Ad Nauseam needs {B}{B}, Doomsday {B}{B}{B} |
| 5 | Savannah, Tropical Island, Misty Rainforest, Windswept Heath, Command Tower, Vampiric Tutor, Thrasios | Ship | Five lands go back, whatever the tutor |
| 6 | Mana Confluence, Flooded Strand, Birds of Paradise, Rhystic Study, Sylvan Library, Esper Sentinel, Swan Song | Keep (close) | 3 mana on turn 2, and Sylvan Library counts as dig. No line card, so expect a slow start |
| 7 | Lotus Petal, Chrome Mox, Elvish Spirit Guide, Dark Ritual, Demonic Tutor, Thassa's Oracle, Brainstorm | Keep (close) | 0 lands but four fast-mana cards. Chrome Mox imprinting Brainstorm is a blue source. It wants a land soon: the one-shot mana runs out after one big turn |
| 8 | Scrubland, Marsh Flats, Dark Ritual, Vampiric Tutor, Silence, Esper Sentinel, Lotho (on 6) | Keep, bottom Lotho | 6-card rule: 3 mana cards and a tutor |

### What to tutor for

Instant tutors that put the card on top (Vampiric Tutor, Mystical Tutor, Enlightened Tutor, Worldly Tutor) go at the
end step of the player before you, so you draw the card next. Imperial Seal is a sorcery that does the same: cast it on
your turn for next turn, or at instant speed with Teferi's +1, Valley Floodcaller or Borne Upon a Wind. On your own
turn, a top-of-library tutor followed by Gitaxian Probe, Brainstorm or Borne Upon a Wind draws the card at once. Under
Necropotence you skip the draw step, so top-of-library tutors are slow.

| Board state | Tutor | Target | Why |
|---|---|---|---|
| Oracle in hand, need the exile half, your turn | Demonic Tutor ({1}{B}) or Grim Tutor ({1}{B}{B}, 3 life) | Tainted Pact into Mental Misstep decks, otherwise Demonic Consultation | Pact dodges Misstep; Consultation is a mana cheaper |
| Oracle in hand, need the exile half, opponent's end step | Mystical Tutor ({U}) or Vampiric Tutor ({B}, 2 life) | Consultation or Pact | On top for your draw |
| Exile half in hand, need Oracle | Eladamri's Call ({G}{W}, instant, to hand) at an end step; Worldly Tutor ({G}, to the top); Beseech the Mirror bargained (cast it free) | Thassa's Oracle | Call is the cleanest: to hand, instant |
| Oracle in hand, no exile half, a flash enabler out (Teferi +1, Floodcaller) | Mystical Tutor at the end step, or Demonic Tutor on your turn | Doomsday (Pile B next turn) | Doomsday is the exile half in effect; Consultation or Pact are cheaper if the tutor can find them just as easily |
| Neither half, 3+ mana, sorcery speed | Demonic Tutor | Necropotence on turns 1-2; otherwise Thassa's Oracle | Necro finds the rest for life |
| Both halves in hand, opponents hold free counters | Mystical Tutor, Merchant Scroll, Worldly Tutor, Eladamri's Call | Silence or Orim's Chant (Mystical); Force of Will or Pact (Scroll); Grand Abolisher or Voice of Victory (Call, Worldly) | One lock wins more games than a spare combo piece |
| A stax piece blocks your line | Mystical Tutor, Merchant Scroll; an any-card tutor for the lands | Snapback, Chain of Vapor, Into the Flood Maw; Boseiju or Otawara | See "Answering stax" |
| Opposition Agent on the battlefield | Don't tutor or crack fetches | Answer it first (Deadly Rollick, Snapback, Chain, Flood Maw, Otawara, Teferi −3) | They take every card you search for |

**Intuition piles** ({2}{U}, instant; an opponent picks one card for your hand, the other two go to the graveyard).
Name three cards where any one does the job:

| You hold | Pile | Why it works |
|---|---|---|
| Thassa's Oracle | Demonic Consultation, Tainted Pact, Mystical Tutor | Any of the three gets the exile half |
| Consultation or Pact | Thassa's Oracle, Eladamri's Call, Worldly Tutor | Any of the three gets Oracle |
| Both halves, need a lock | Silence, Grand Abolisher, Voice of Victory | Each stops every opponent on your turn |

**Gifts Ungiven** ({3}{U}, instant; up to four cards with different names, an opponent puts two in your graveyard and
two in your hand). Holding Oracle: Demonic Consultation, Tainted Pact, Doomsday and Mystical Tutor (any two give you an
exile half or a way to one; Doomsday needs a flash enabler or Pile C). Noxious Revival can put a card the opponent
sent to the graveyard back on top.

**Merchant Scroll** ({1}{U}, a blue instant to hand): Mystical Tutor (a second step to Consultation, Pact, Doomsday or
Silence), Intuition, Gifts Ungiven, Brainstorm, Borne Upon a Wind (flash for Doomsday), Force of Will or Pact of
Negation, Snapback, Chain of Vapor or Into the Flood Maw, Fierce Guardianship.

**Beseech the Mirror** ({1}{B}{B}{B}): bargain by sacrificing an artifact, enchantment or token as you cast it, and you
may cast the found card free if its mana value is 4 or less. Fodder: a Treasure (from Lotho or Smothering Tithe),
Lotus Petal, a Mox, Sol Ring, an old Mystic Remora, the Orc Army token. A Mox or Sol Ring can tap for mana while you
cast Beseech and then be sacrificed to bargain. Best targets: Thassa's Oracle (holding the exile half), Necropotence,
or Doomsday (mana value 3). Cast free, Doomsday still works as a sorcery on your turn: use Pile C. Never Demonic
Consultation: it would resolve with no Oracle trigger.

**Diabolic Intent** ({1}{B}, sacrifice a creature): tap Birds or Noble Hierarch for mana first, then sacrifice it.
Don't sacrifice a commander you need for Fierce Guardianship this turn.

**Eladamri's Call and Worldly Tutor** find Thassa's Oracle, a lock creature (Grand Abolisher, Voice of Victory,
Ranger-Captain of Eos), Valley Floodcaller (flash for Doomsday), or Walking Ballista for Line 4. Ranger-Captain then
fetches a 1-drop: Birds, Hierarch, Avacyn's Pilgrim, Esper Sentinel or Mockingbird.

### Protecting the win turn

Order:

1. **The lock, before you commit a combo piece.** Silence ({W}) stops every opponent. Orim's Chant ({W}) stops one: aim
   it at the player with the most blue mana or cards. Grand Abolisher and Voice of Victory stop everyone on your turn
   and are best cast a turn early. Teferi, Time Raveler stops everyone from casting outside their own main phase.
   Ranger-Captain of Eos (sacrifice it) stops noncreature spells, and as an ability it can't be countered by a spell.
2. **Veil of Summer in response to a counter** ({G}): your spells can't be countered this turn, so the counter on the
   stack does nothing. It doesn't stop Mindbreak Trap (exile, not counter), a Silence cast before yours, or Stifle.
3. **Then the counter war:** Force of Will (any spell), Pact of Negation (any spell), Fierce Guardianship (noncreature,
   free with a commander out), Mental Misstep (their 1-mana counters, or a Silence aimed at you), Flusterstorm (their
   instant counters), Swan Song (their instant or sorcery).

**Timing traps:**

- **Force of Negation** is free only when it isn't your turn. On your kill turn it costs {1}{U}{U}.
- **Mindbreak Trap** is free only once an opponent has cast three or more spells this turn. On your turn, plan on
  {2}{U}{U}.
- **Pact of Negation:** at your next upkeep you pay {3}{U}{U} or lose. Use it on the turn you win, or when you're sure of
  five mana with {U}{U} next upkeep.
- **Fierce Guardianship and Deadly Rollick** need a commander on the battlefield when you cast them.
- **Mental Misstep** hits only mana value 1 spells: Swan Song, Flusterstorm, Silence, Orim's Chant, Consultation. Not
  Force of Will, Pact or Fierce.
- **Free spells against Vexing Bauble or Lavinia** (spells cast with no mana spent are countered): pay mana where you
  can (Fierce for {2}{U}, Snapback for {1}{U}, Gitaxian Probe for {U}).

### Interaction on opponents' turns

Keep one free counter for their kill. On their turn Force of Negation is free (exile a blue card), Force of Will is
free, Fierce Guardianship is free with a commander out, and Mindbreak Trap becomes free once they cast their third
spell.

| Their combo | Counter the... | With |
|---|---|---|
| Thassa's Oracle + Consultation/Pact (Kraum/Tymna, Rograkh/Silas) | The instant, cast in response to the Oracle trigger | Force of Negation, Swan Song, Flusterstorm, Fierce Guardianship, Mental Misstep (Consultation only), Force of Will, Pact |
| The same, if they cast Consultation first | Thassa's Oracle (a creature) | Force of Will, Pact, Mindbreak Trap |
| Underworld Breach loops (Kraum/Tymna, Rograkh/Silas) | Breach when it's cast; mid-loop, Brain Freeze | Breach: Swan Song, Force of Negation, Fierce, Force of Will. Mid-loop: Mindbreak Trap (free by then; exiles Brain Freeze and its copies), Flusterstorm |
| Kinnan + Basalt Monolith | The Monolith (artifact), or Kinnan himself | Force of Negation, Fierce, Force of Will, Pact (Swan Song can't hit artifacts); Deadly Rollick or Swords to Plowshares on Kinnan |
| Dack Fayden (6-mana creature) | Dack, on the cast | Force of Will, Pact, Mindbreak Trap. Fierce, Force of Negation and Swan Song can't counter a creature. After it resolves: Deadly Rollick or Swords to Plowshares |
| Sisay's activation (an ability) | Nothing counters it: remove Sisay first | Deadly Rollick, Swords to Plowshares, Snapback, Into the Flood Maw, Chain of Vapor |

- **Swan Song** ({U}) only hits enchantments, instants and sorceries, and gives the caster a 2/2 flier. Spend it on
  the Consultation, Breach or Ad Nauseam that wins, not on a draw spell.
- **Force of Negation** is your best counter on their turn (free, and it exiles). Hold it for a kill, not a Rhystic
  Study.
- **Mindbreak Trap** answers storm and Breach loops: by the time they loop it's free, and it exiles any number of
  spells, including ones that can't be countered.
- **Silence or Orim's Chant at their upkeep** stops their whole turn of spells. Use it when they're about to win and
  you can't win first.
- **Orcish Bowmasters** (flash) punishes extra draws and kills X/1s: Rograkh, Birds, Hierarch, an opposing Esper
  Sentinel.

### Answering stax

Bounce is enough on the kill turn: once Oracle's trigger resolves, it doesn't matter that the piece comes back.

- **Chain of Vapor** ({U}): any nonland permanent, but the owner may sacrifice a land to copy it and bounce one of
  yours, so cast it before the combo starts.
- **Into the Flood Maw** ({U}): a creature, or any nonland permanent if you gift the opponent a tapped 1/1 Fish.
- **Snapback:** a creature, free by exiling a blue card from hand.
- **Boseiju** (channel {1}{G}, {1} less per legendary creature you control; your commanders count): destroys an
  artifact, enchantment or nonbasic land.
- **Otawara** (channel {3}{U}, same discount): bounces an artifact, creature, enchantment or planeswalker.
- **Teferi's −3:** an artifact, creature or enchantment, and you draw a card.
- **Swords to Plowshares** and **Deadly Rollick** exile a creature for good.

Boseiju and Otawara work from hand, and they're abilities, not spells.

| Stax piece (opponent's) | What it stops for us | Answer | Line that still works |
|---|---|---|---|
| Drannith Magistrate | Recasting commanders, Beseech's free cast | Deadly Rollick, Swords, Snapback, Chain, Flood Maw, Otawara, Teferi −3 | Every line |
| Rest in Peace / Grafdigger's Cage | Noxious Revival after a countered Oracle | Chain, Flood Maw (gift), Boseiju, Otawara, Teferi −3 | Every line: none of them use the graveyard |
| Null Rod / Collector Ouphe | All artifact mana, LED, Treasures, Basalt + Rings | Null Rod: Chain, Flood Maw (gift), Boseiju, Otawara, Teferi −3. Ouphe: Deadly Rollick, Swords, Snapback, Chain, Flood Maw, Otawara, Teferi −3 | Lines 1-2 off lands, rituals and mana creatures |
| Cursed Totem | Mana creatures, Ranger-Captain's sacrifice, Thrasios's ability | Chain, Flood Maw (gift), Boseiju, Otawara, Teferi −3 | Lines 1-2 |
| Archon of Emeria | One spell per player per turn; your fetched duals enter tapped | Otawara (an ability), or spend the turn's spell on Deadly Rollick, Swords, Snapback, Chain or Flood Maw and win next turn | Line 1 with Necropotence out: Consultation or Pact at an opponent's end step, then Oracle on your turn (Necro skips the draw that would lose you the game). Doomsday piles need several spells: dead |
| Deafening Silence | One noncreature spell per turn | Boseiju, Otawara (abilities); Teferi −3 (Teferi uses the turn's spell) | Line 1 (Oracle is a creature, plus one Consultation or Pact) |
| Blood Moon / Magus of the Moon | Every nonbasic land taps only for {R} | Boseiju from hand ({1}{G} from Birds, Hierarch, Elvish Spirit Guide, a Mox or Petal), Otawara, Chain, Flood Maw (gift), Teferi −3; Magus: Deadly Rollick, Swords, Snapback | Line 1 off Moxen, Petal, mana creatures and Dark Ritual |
| Chalice of the Void | X=0: LED, Moxen, Petal, Pact. X=1: Consultation, Dark Ritual, Silence, the cantrips, the 1-mana tutors and counters | Boseiju, Otawara, Teferi −3; Chain or Flood Maw if X isn't 1 | X=1: Oracle + Tainted Pact. X=0: Lines 1-2 |
| Trinisphere | Every spell costs at least 3: free spells, rituals and cantrips stop working | Boseiju, Otawara, Teferi −3 | Line 1 with six mana |
| Vexing Bauble / Lavinia | Spells cast with no mana spent are countered: Pact, the free spells, LED, Moxen, Petal, Probe for life | Pay mana. Bauble: Chain, Flood Maw (gift), Boseiju, Otawara, Teferi −3. Lavinia: Swords, Snapback hard-cast, Chain, Flood Maw, Otawara, Teferi −3 | Line 1 paid with mana |
| Opposition Agent | Your tutors and fetches: they take the card | Deadly Rollick, Swords, Snapback, Chain, Flood Maw, Otawara, Teferi −3 | Line 1 from hand. Doomsday searches too, so the Agent's controller would choose your pile: never cast it under Agent |
| Opposing Teferi, Time Raveler | You can cast spells only at sorcery speed: no Consultation with the trigger on the stack | Chain, Flood Maw (gift), Otawara | Doomsday Pile C (sorcery-speed throughout) |

### Matchups (Tier 6)

The cEDH field from the opponent pool: Kraum/Tymna, Kinnan, Rograkh/Thrasios, Rograkh/Silas, Sisay and Dack Fayden.

| Opponent | Race or hold? | Hold up | Cards that matter |
|---|---|---|---|
| Kraum/Tymna (blue midrange, Oracle and Breach) | Go first; they wait for you to tap out. Lead with a lock | A free counter for their Consultation or Breach | Silence, Grand Abolisher, Voice of Victory, Veil of Summer. Their Red Elemental Blast hits Oracle: cast it off Cavern of Souls or behind Veil |
| Kinnan (fast mana combo, turns 3-5) | Race, but hold one counter for Basalt Monolith or Kinnan | Force of Negation, Fierce Guardianship; Deadly Rollick or Swords for Kinnan | Orcish Bowmasters on their mana creatures |
| Rograkh/Thrasios (creature mana loops) | Race; they need Gaea's Cradle and creature loops | A free counter for the loop piece, or removal for a creature | Bowmasters kills Rograkh. Their Veil of Summer beats your counters, so answer the creature instead |
| Rograkh/Silas (turbo Oracle and Breach, turns 1-3) | The true race. Keep a free counter up from turn 1 | Force of Negation, Mindbreak Trap, Flusterstorm | Their Vexing Bauble and Pyroblast punish free spells and Oracle; Mental Misstep on their Consultation and rituals |
| Sisay (legends toolbox) | Race; its win threat appears around turns 3-4 | Removal for Sisay (Deadly Rollick, Swords, Snapback, Flood Maw), not counters: the legends arrive without being cast | Its Lavinia, Teferi, Deafening Silence and Esper Sentinel tax you |
| Dack Fayden (white stax) | Race before the stax builds | Force of Will, Pact or Mindbreak Trap for Dack himself | Its Silence and Orim's Chant come before Dack; Chalice, Trinisphere, Damping Sphere and Cursed Totem call for Boseiju, Otawara and Teferi |

### After a stopped attempt

| What happened | Rebuild |
|---|---|
| Oracle countered into the graveyard | Noxious Revival puts it on top: draw it next turn, or this turn with Gitaxian Probe, Brainstorm or Borne Upon a Wind. Force of Negation and Mindbreak Trap exile instead: no Revival |
| Consultation or Pact countered | The other one, or Doomsday, is still in the library: tutor for it, or Noxious Revival the countered one |
| Oracle resolved, exile half countered | Oracle stays out: next turn Mockingbird with X=1 copies it, and you cast the exile half in response to the copy's trigger |
| Library exiled, trigger countered | Win this turn or lose at your draw: Mockingbird, or bounce and recast Oracle |
| Doomsday countered | Library and graveyard are untouched. Go back to Line 1 |
| Doomsday resolved, the pile stopped partway | You have as many draw steps left as cards in the library. Noxious Revival, Mockingbird and Snapback are the outs; otherwise you lose at the draw from the empty library |
| A commander removed | Recast it for its cost plus {2} per previous cast. Fierce Guardianship and Deadly Rollick stay free while the other one is out |

- A Pact of Negation cast in the failed turn must be paid at your next upkeep. Count that before the attempt.
- After a stop, the next try needs more protection than the last: the table has seen your plan.

### Life management

| Source | Cost |
|---|---|
| Fetch lands | 1 each |
| Ancient Tomb | 2 per tap |
| Mana Confluence | 1 per tap |
| Mana Vault | 1 per draw step while it's tapped |
| Vampiric Tutor, Imperial Seal | 2 each |
| Grim Tutor | 3 |
| Gitaxian Probe, Mental Misstep, Noxious Revival | 2 each if paid with life |
| Force of Will | 1 |
| Necropotence | 1 per card |
| Ad Nauseam | Each card's mana value |
| Sylvan Library | 4 per extra card kept |
| Tymna | 1 per card drawn |
| Lotho, Corrupt Shirriff | 1 each time any player casts their second spell in a turn (you get a Treasure) |
| **Doomsday** | **Half your life, rounded up** |

**Rough budget at 40 life:** the fixed costs of a turn-3 kill (two fetches, a few Tomb or Confluence taps, one
life-paid tutor, a Phyrexian spell) come to about 10-15. Doomsday then takes half of what's left. Life is no bar to a
Doomsday turn (the pile wins or you lose either way) beyond the 3 life a life-paid Probe in the pile needs (pay 2, keep 1). Line 1
costs no life of its own.

### Common mistakes

1. Casting Demonic Consultation before Thassa's Oracle.
2. Casting Doomsday at sorcery speed with Pile A or B: those piles rely on your draw step. Without a flash enabler,
   use Pile C.
3. Building a pile without counting devotion. Thrasios, Floodcaller, Mockingbird, Remora, Rhystic Study and Teferi each
   save a draw.
4. Activating Thrasios with an empty library during Line 4.
5. Casting Doomsday or a tutor while an opponent controls Opposition Agent.
6. Cracking Lion's Eye Diamond with cards you need still in hand, or with Necropotence out (Necro exiles the discards).
7. Counting Force of Negation or Mindbreak Trap as free on your own turn.
8. Casting Pact of Negation without five mana ({3}{U}{U}) for your next upkeep, on a turn you might not win.
9. Paying Necropotence for more cards than you can keep, or casting Doomsday Piles A and B under it.
10. Casting Chain of Vapor mid-combo: the opponent sacrifices a land, copies it, and bounces your Oracle.
11. Using a top-of-library tutor in your own main phase with no way to draw the card this turn.
12. Losing your last commander at your upkeep and then counting Fierce Guardianship as free.

## Mana

27 lands. Other mana: Lotus Petal, Chrome Mox, Mox Diamond, Sol Ring, Mana Vault, Grim Monolith, Basalt Monolith,
Arcane Signet, Lion's Eye Diamond, Dark Ritual, Cabal Ritual, Elvish Spirit Guide, three mana creatures (Birds of
Paradise, Noble Hierarch, Avacyn's Pilgrim), and Treasures from Lotho and Smothering Tithe.

- **Original duals:** Underground Sea, Bayou, Scrubland, Tundra, Tropical Island, Savannah.
- **Shock lands:** Overgrown Tomb, Godless Shrine, Hallowed Fountain, Breeding Pool.
- **Fetch lands (10):** Flooded Strand, Polluted Delta, Misty Rainforest, Verdant Catacombs, Windswept Heath, Marsh Flats,
  Arid Mesa, Scalding Tarn, Bloodstained Mire, Wooded Foothills. Each fetch also finds a dual with one of its land
  types; the ones naming Mountain still find the Plains, Island, Swamp or Forest half.
- **Others:** Command Tower, Mana Confluence, Gemstone Caverns, Cavern of Souls (name Wizard), Ancient Tomb, Boseiju,
  Otawara.
- **{B}{B}{B} for Doomsday:** Underground Sea, Bayou, Scrubland, Overgrown Tomb and Godless Shrine are Swamps. Dark
  Ritual turns one {B} into three, and Cabal Ritual does the same once you have seven cards in your graveyard.

## Bracket and Rule 0

Bracket 5 (cEDH): 22 Game Changers, Thassa's Oracle with Demonic Consultation and Tainted Pact (two-card wins), Doomsday,
Basalt Monolith + Rings of Brighthearth (unlimited mana), free counters and Silence effects. Tell the table it's a cEDH
deck.

## Decklist by type (100)

**Commanders (2).** Tymna the Weaver, Thrasios, Triton Hero

**Creatures (14).** Avacyn's Pilgrim, Birds of Paradise, Elvish Spirit Guide, Esper Sentinel, Grand Abolisher, Lotho, Corrupt Shirriff, Mockingbird, Noble Hierarch, Orcish Bowmasters, Ranger-Captain of Eos, Thassa's Oracle, Valley Floodcaller, Voice of Victory, Walking Ballista

**Planeswalkers (1).** Teferi, Time Raveler

**Artifacts (10).** Arcane Signet, Basalt Monolith, Chrome Mox, Grim Monolith, Lion's Eye Diamond, Lotus Petal, Mana Vault, Mox Diamond, Rings of Brighthearth, Sol Ring

**Enchantments (5).** Mystic Remora, Necropotence, Rhystic Study, Smothering Tithe, Sylvan Library

**Instants (31).** Ad Nauseam, Borne Upon a Wind, Brainstorm, Cabal Ritual, Chain of Vapor, Dark Ritual, Deadly Rollick, Demonic Consultation, Eladamri's Call, Enlightened Tutor, Fierce Guardianship, Flusterstorm, Force of Negation, Force of Will, Gifts Ungiven, Into the Flood Maw, Intuition, Mental Misstep, Mindbreak Trap, Mystical Tutor, Noxious Revival, Orim's Chant, Pact of Negation, Silence, Snapback, Swan Song, Swords to Plowshares, Tainted Pact, Vampiric Tutor, Veil of Summer, Worldly Tutor

**Sorceries (10).** Beseech the Mirror, Demonic Tutor, Diabolic Intent, Doomsday, Gitaxian Probe, Grim Tutor, Imperial Seal, Merchant Scroll, Ponder, Preordain

**Lands (27).** Ancient Tomb, Arid Mesa, Bayou, Bloodstained Mire, Boseiju, Who Endures, Breeding Pool, Cavern of Souls, Command Tower, Flooded Strand, Gemstone Caverns, Godless Shrine, Hallowed Fountain, Mana Confluence, Marsh Flats, Misty Rainforest, Otawara, Soaring City, Overgrown Tomb, Polluted Delta, Savannah, Scalding Tarn, Scrubland, Tropical Island, Tundra, Underground Sea, Verdant Catacombs, Windswept Heath, Wooded Foothills

## Import list (100)

```
1 Ad Nauseam
1 Ancient Tomb
1 Arcane Signet
1 Arid Mesa
1 Avacyn's Pilgrim
1 Basalt Monolith
1 Bayou
1 Beseech the Mirror
1 Birds of Paradise
1 Bloodstained Mire
1 Borne Upon a Wind
1 Boseiju, Who Endures
1 Brainstorm
1 Breeding Pool
1 Cabal Ritual
1 Cavern of Souls
1 Chain of Vapor
1 Chrome Mox
1 Command Tower
1 Dark Ritual
1 Deadly Rollick
1 Demonic Consultation
1 Demonic Tutor
1 Diabolic Intent
1 Doomsday
1 Eladamri's Call
1 Elvish Spirit Guide
1 Enlightened Tutor
1 Esper Sentinel
1 Fierce Guardianship
1 Flooded Strand
1 Flusterstorm
1 Force of Negation
1 Force of Will
1 Gemstone Caverns
1 Gifts Ungiven
1 Gitaxian Probe
1 Godless Shrine
1 Grand Abolisher
1 Grim Monolith
1 Grim Tutor
1 Hallowed Fountain
1 Imperial Seal
1 Into the Flood Maw
1 Intuition
1 Lion's Eye Diamond
1 Lotho, Corrupt Shirriff
1 Lotus Petal
1 Mana Confluence
1 Mana Vault
1 Marsh Flats
1 Mental Misstep
1 Merchant Scroll
1 Mindbreak Trap
1 Misty Rainforest
1 Mockingbird
1 Mox Diamond
1 Mystical Tutor
1 Mystic Remora
1 Necropotence
1 Noble Hierarch
1 Noxious Revival
1 Orcish Bowmasters
1 Orim's Chant
1 Otawara, Soaring City
1 Overgrown Tomb
1 Pact of Negation
1 Polluted Delta
1 Ponder
1 Preordain
1 Ranger-Captain of Eos
1 Rhystic Study
1 Rings of Brighthearth
1 Savannah
1 Scalding Tarn
1 Scrubland
1 Silence
1 Smothering Tithe
1 Snapback
1 Sol Ring
1 Swan Song
1 Swords to Plowshares
1 Sylvan Library
1 Tainted Pact
1 Teferi, Time Raveler
1 Thassa's Oracle
1 Thrasios, Triton Hero
1 Tropical Island
1 Tundra
1 Tymna the Weaver
1 Underground Sea
1 Valley Floodcaller
1 Vampiric Tutor
1 Veil of Summer
1 Verdant Catacombs
1 Voice of Victory
1 Walking Ballista
1 Windswept Heath
1 Wooded Foothills
1 Worldly Tutor
```
