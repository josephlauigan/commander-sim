# Tymna the Weaver & Thrasios, Triton Hero — Four-Colour Doomsday (W/U/B/G, cEDH)

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
| **Simulator** | Deck key `thrasios`, tier T6 (cEDH), played from `data/plans/thrasios.toml`; findings in `audit/thrasios/FINDINGS.md`, history in `decklists/CHANGELOG.md`. Doomsday has card code, a pile builder and the play of the pile (`impls/doomsday.rs`) |
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
| 5 | Watery Grave, Tropical Island, Misty Rainforest, Windswept Heath, Command Tower, Vampiric Tutor, Thrasios | Ship | Five lands go back, whatever the tutor |
| 6 | Mana Confluence, Flooded Strand, Birds of Paradise, Rhystic Study, Sylvan Library, Esper Sentinel, Swan Song | Keep (close) | 3 mana on turn 2, and Sylvan Library counts as dig. No line card, so expect a slow start |
| 7 | Lotus Petal, Chrome Mox, Elvish Spirit Guide, Dark Ritual, Demonic Tutor, Thassa's Oracle, Brainstorm | Keep (close) | 0 lands but four fast-mana cards. Chrome Mox imprinting Brainstorm is a blue source. It wants a land soon: the one-shot mana runs out after one big turn |
| 8 | Scrubland, Marsh Flats, Dark Ritual, Vampiric Tutor, Silence, Esper Sentinel, Lotho (on 6) | Keep, bottom Lotho | 6-card rule: 3 mana cards and a tutor |

### What to tutor for

Where a tutor puts the card decides how soon you can use it. **To hand:** Demonic Tutor, Grim Tutor, Diabolic Intent,
Eladamri's Call, Merchant Scroll, Intuition, Gifts Ungiven, Beseech the Mirror, Tainted Pact, Demonic Consultation.
**On top of the library:** Vampiric Tutor, Imperial Seal, Mystical Tutor, Enlightened Tutor, Worldly Tutor, Noxious
Revival. A top-of-library card needs a draw before it counts: your draw step, Gitaxian Probe, Brainstorm, Preordain,
Borne Upon a Wind, a Rhystic Study or Mystic Remora trigger, or {4} on Thrasios. Every search (the tutors that
say "shuffle", and fetch lands) shuffles the library, and a shuffle buries a card you put on top.

Shorthand below: **Oracle** is Thassa's Oracle, **the exile half** is Demonic Consultation or Tainted Pact, **the lock**
is Silence, Orim's Chant, Grand Abolisher, Voice of Victory or Teferi, Time Raveler, and **any-card tutor** is Demonic
Tutor, Vampiric Tutor, Grim Tutor, Imperial Seal, Diabolic Intent, Beseech the Mirror, Intuition or Gifts Ungiven. There
is no Laboratory Maniac or Jace, Wielder of Mysteries in the list: Oracle is the only "library is empty" win, so a
countered or exiled Oracle ends Lines 1 and 2 for the turn.

#### The tutors

| Card | Cost | Speed | Goes to | Finds | Besides mana | Catch |
|---|---|---|---|---|---|---|
| Demonic Tutor | {1}{B} | Sorcery | Hand | Any card | Nothing | Cast it before Oracle: with Oracle's trigger on the stack only instants work |
| Grim Tutor | {1}{B}{B} | Sorcery | Hand | Any card | 3 life | Demonic Tutor with a mana and 3 life added; use it when {B}{B} is spare and life is high |
| Diabolic Intent | {1}{B} | Sorcery | Hand | Any card | Sacrifice a creature | Tap Birds, Hierarch or Pilgrim for mana first. Don't sacrifice a commander you need for Fierce Guardianship |
| Vampiric Tutor | {B} | Instant | Top | Any card | 2 life | Not revealed. Needs a draw before you can use the card |
| Imperial Seal | {B} | Sorcery | Top | Any card | 2 life | Not revealed. Sorcery: cast it on your turn for next turn's draw, or at instant speed with Teferi's +1, Valley Floodcaller or Borne Upon a Wind |
| Mystical Tutor | {U} | Instant | Top | Instant or sorcery | Nothing | Revealed. The exile half, Doomsday, Silence, Veil of Summer, every counter and the cantrips |
| Enlightened Tutor | {W} | Instant | Top | Artifact or enchantment | Nothing | Revealed. Necropotence, Rhystic Study, Mystic Remora, the mana rocks, Basalt Monolith, Rings of Brighthearth, Lion's Eye Diamond, Esper Sentinel and Walking Ballista (both artifact creatures) |
| Worldly Tutor | {G} | Instant | Top | Creature | Nothing | Revealed. Oracle, the lock creatures, Valley Floodcaller, Walking Ballista, mana creatures |
| Eladamri's Call | {G}{W} | Instant | Hand | Creature | Nothing | Revealed. The cleanest way to Oracle: instant speed, straight to hand. Needs both {G} and {W} |
| Merchant Scroll | {1}{U} | Sorcery | Hand | Blue instant | Nothing | Revealed. Reaches Mystical Tutor, Intuition, Gifts Ungiven, Brainstorm, Borne Upon a Wind, Force of Will, Pact of Negation, Fierce Guardianship, Force of Negation, Mental Misstep, Flusterstorm, Swan Song, Mindbreak Trap, Snapback, Chain of Vapor, Into the Flood Maw. Not Silence, Veil of Summer, the exile half or Doomsday: those take Mystical Tutor, a second step |
| Intuition | {2}{U} | Instant | Hand (one card) | Three cards | Nothing | You choose the opponent; they pick one for your hand and the other two go to the graveyard. Name three that each do the job |
| Gifts Ungiven | {3}{U} | Instant | Two to hand, two to graveyard | Up to four cards with different names | Nothing | The opponent you choose picks which two go to the graveyard. Name four where any two are enough. Graveyard cards stay reachable with Noxious Revival and Doomsday |
| Beseech the Mirror | {1}{B}{B}{B} | Sorcery | Exile face down, then hand or cast free | Any card | Bargain: sacrifice an artifact, enchantment or token | Free cast only if bargained and the card's mana value is 4 or less (Oracle 2, Necropotence 3, Doomsday 3, Teferi 3, Floodcaller 3). Otherwise it goes to hand. A free spell can still be countered, and a spell cast with no mana spent is countered by Vexing Bauble or Lavinia |
| Tainted Pact | {1}{B} | Instant | Hand (one card) | The first card you take, going down from the top | Nothing | Every card above it is exiled for good, and you can't aim it. Its job is the Oracle win; as a tutor it only works on a top you already know |
| Demonic Consultation | {B} | Instant | Hand | A card you name | Nothing | The top six are exiled before it looks, so a named card in the top six is lost and the rest of the library follows it. Its job is the Oracle win |
| Doomsday | {B}{B}{B} | Sorcery | Top (a five-card pile) | Any five cards from library and graveyard | Half your life, rounded up | Exiles everything else, including your graveyard. See Line 2 |
| Necropotence | {B}{B}{B} | Enchantment | Hand, at the beginning of your next end step | The top card, whatever it is | 1 life per card | Not a tutor, a dig. Skips your draw step. Discards are exiled |
| Ranger-Captain of Eos | {1}{W}{W} | Creature | Hand | Creature with mana value 1 or less: Birds of Paradise, Noble Hierarch, Avacyn's Pilgrim, Esper Sentinel, Mockingbird, Walking Ballista | Nothing | A creature spell, counterable. Its sacrifice ability is a lock against noncreature spells |
| Noxious Revival | {G} or 2 life | Instant | Top | Any card in any graveyard | 2 life if you don't pay {G} | Puts back a countered Oracle (not an exiled one) or a card Intuition or Gifts sent to the graveyard. Needs a draw after |

#### Cards that dig or draw

| Card | Cost | What it does | Catch |
|---|---|---|---|
| Brainstorm | {U}, instant | Draw three, put two back on top | Put the two junk cards back, then crack a fetch or cast a tutor: the shuffle buries them. It draws a tutored top card at once |
| Ponder | {U}, sorcery | Look at the top three, reorder, optional shuffle, draw | Don't shuffle if a tutored card is on top |
| Preordain | {U}, sorcery | Scry 2, draw | Keep the tutored card on top, then draw it |
| Gitaxian Probe | {U} or 2 life, sorcery | Look at a hand, draw | Free in mana. Sorcery: not with a trigger on the stack. Starts Pile C |
| Borne Upon a Wind | {1}{U}, instant | Spells this turn have flash, draw | Flash for Doomsday and the sorcery tutors, and a draw. Starts Pile C |
| Sylvan Library | {1}{G}, enchantment | At your draw step, draw two more, then put two cards drawn this turn back on top or pay 4 life each | A free look at three cards. Off under Necropotence (no draw step) |
| Thrasios, Triton Hero | {G}{U}; {4} ability | Scry 1, reveal the top card: a land goes onto the battlefield tapped, anything else is drawn | Draws a tutored top card at instant speed, but a tutored land lands on the battlefield, not in hand. Never on an empty library |
| Mystic Remora | {U}, enchantment | Draw when an opponent casts a noncreature spell, unless they pay {4} | You may decline the draw. Cumulative upkeep: pay it once or twice |
| Rhystic Study | {2}{U}, enchantment | Draw when an opponent casts a spell, unless they pay {1} | You may decline the draw. A table that always pays {1} gives you nothing |
| Esper Sentinel | {W}, artifact creature | Draw when an opponent casts their first noncreature spell each turn, unless they pay {X} | The draw isn't optional, so don't hold a Doomsday pile through an opponent's turn |
| Tymna the Weaver | {1}{W}{B} | After combat, pay X life, X being the opponents dealt combat damage this turn, and draw X | Needs an attacker to connect. Draws from the top, so it also pulls a tutored card |
| Ad Nauseam | {3}{B}{B}, instant | Reveal the top card, put it in hand, lose life equal to its mana value, repeat as you like | Never after Doomsday: it reveals only your pile |
| Thassa's Oracle | {U}{U}, creature | Look at the top X cards (X is devotion to blue), keep one on top, the rest go to the bottom in random order | A tutored top card survives only if you choose it |
| Mockingbird | {X}{U}, creature | Copies a creature already out | Copying Oracle gives a second trigger, and the copy has Oracle's {U}{U} cost, so it adds 2 to devotion |
| Teferi, Time Raveler | {1}{W}{U} | -3 bounces an artifact, creature or enchantment and draws | With an empty library the draw loses you the game |

#### What each target needs to find it

| Target | Found by |
|---|---|
| Thassa's Oracle, Valley Floodcaller, Grand Abolisher, Voice of Victory, Orcish Bowmasters | Any-card tutors, Worldly Tutor, Eladamri's Call |
| Walking Ballista | Any-card tutors, Worldly Tutor, Eladamri's Call, Enlightened Tutor, Ranger-Captain of Eos |
| Birds of Paradise, Noble Hierarch, Avacyn's Pilgrim, Esper Sentinel, Mockingbird | Any-card tutors, Worldly Tutor, Eladamri's Call, Ranger-Captain of Eos |
| Demonic Consultation, Tainted Pact, Doomsday, Silence, Orim's Chant, Veil of Summer, Noxious Revival | Any-card tutors, Mystical Tutor |
| Gitaxian Probe, Preordain, Ponder | Any-card tutors, Mystical Tutor (not Merchant Scroll: they are sorceries) |
| Force of Will, Pact of Negation, Fierce Guardianship, Force of Negation, Swan Song, Flusterstorm, Mental Misstep, Mindbreak Trap, Snapback, Chain of Vapor, Into the Flood Maw, Brainstorm, Borne Upon a Wind | Any-card tutors, Mystical Tutor, Merchant Scroll |
| Necropotence, Rhystic Study, Mystic Remora, Sylvan Library, Smothering Tithe, mana rocks, Moxen, Lotus Petal, Lion's Eye Diamond, Basalt Monolith, Rings of Brighthearth | Any-card tutors, Enlightened Tutor |
| Teferi, Time Raveler | Any-card tutors only |
| Cavern of Souls, Boseiju, Otawara, Ancient Tomb | Any-card tutors only (a fetch land finds only a land with a basic land type) |

#### What each card does to the library

| Card | What leaves the library | Where it goes | What that means |
|---|---|---|---|
| Demonic Consultation | The top six cards, then every card revealed until the named one appears | The named card to hand; everything else is exiled | Name a card that isn't in the library and the whole library is exiled: the Oracle win. Name a card in the library and it is lost if it was in the top six |
| Tainted Pact | One card at a time from the top, until you take one | The card you take to hand; the rest are exiled | Decline every card and the whole library is exiled (the deck is singleton, so no name repeats). Take a card and stop, and only the cards above it are lost |
| Doomsday | Everything except five cards you choose, from library and graveyard | Five cards on top in your order; the rest are exiled | Library is five cards, so Oracle needs devotion to blue to cover what is left after your draws |
| Necropotence | The top card per life paid | To hand at the beginning of your next end step | Nothing is lost, but you discard down to 7 at cleanup and the discards are exiled. No draw step |
| Ad Nauseam | The top card each time you repeat | To hand | You pay its mana value in life each time; lands are free |
| Thassa's Oracle (trigger) | Looks at the top X | One card on top, the rest on the bottom in random order | You win if X is at least the cards left in the library |
| Thrasios | The top card per {4} | Hand, or the battlefield tapped if it is a land | Each activation shrinks the library by one |
| Fetch lands, tutors that shuffle | One card | Battlefield (fetch), hand or top (tutor) | Then the library is shuffled, which moves a tutored top card |

**If Oracle is still in the library when the library is exiled, the win is gone.** Cast the exile half only with
Oracle on the battlefield and its trigger on the stack (Line 1).

#### Count before you tutor

**Free mana (counts for one turn unless noted):** Dark Ritual ({B} into {B}{B}{B}: +2), Cabal Ritual ({1}{B} into
{B}{B}{B}: +1, or {B}{B}{B}{B}{B}: +3 with seven cards in your graveyard), Lotus Petal (+1, once), Lion's Eye Diamond
(+3 of one colour, discards your hand), Mox Diamond (+1 for a discarded land, stays), Chrome Mox (+1 for an exiled
nonartifact, nonland card, stays), Elvish Spirit Guide ({G} from your hand, also on opponents' turns) and Treasures from Lotho or
Smothering Tithe. Mana creatures need a turn out first.

**Free interaction:** Force of Will (1 life and a blue card, any spell), Force of Negation (a blue card, noncreature,
only on opponents' turns), Fierce Guardianship (noncreature, a commander out), Pact of Negation (any spell, owe
{3}{U}{U} at your next upkeep), Mental Misstep ({U} or 2 life, mana value 1 only), Snapback (a blue card, a creature)
and Deadly Rollick (a commander out, a creature). Each pitch cost is a blue card, so count spare blue cards (cantrips,
Mental Misstep, Intuition, Merchant Scroll) as well as the counters. Never pitch Oracle.

| Plan | Mana |
|---|---|
| Oracle with Demonic Consultation | 3 ({U}{U}, {B}) |
| Oracle with Tainted Pact | 4 ({U}{U}, {1}{B}) |
| Add a tutor first | Demonic Tutor 2, Grim Tutor 3, Mystical Tutor 1, Vampiric Tutor 1, Eladamri's Call 2 |
| Add a lock | Silence, Orim's Chant or Veil of Summer 1; Grand Abolisher 2; Voice of Victory 2; Teferi 3 |
| Doomsday, Pile A or B | {B}{B}{B}, then 4 ({U}, {W}, {U}{U}) and 2 life (or {U} instead of the life) |
| Doomsday, Pile C | {B}{B}{B}, then 5 ({U}, {U}, {W}, {U}{U}) and 2 life (or {U}) |
| Necropotence | {B}{B}{B} |
| Basalt Monolith + Rings of Brighthearth | About 8 from other sources if both are cast this turn; 5 if one is already out |

**What the Doomsday pile needs.** (1) {B}{B}{B}: Swamp-type lands, Dark Ritual, Cabal Ritual or Lion's Eye Diamond. (2)
Oracle in hand (Piles B and C) or in the pile (Pile A). (3) A way to cast it at the end step of the player before you:
Teferi's +1, Valley Floodcaller or Borne Upon a Wind. With none of them, cast it in your own main phase and use Pile C,
which also needs Oracle and Gitaxian Probe or Borne Upon a Wind in hand. (4) Library after the draws no larger than
devotion to blue: the pile leaves five, so you need five minus devotion net draws, which is three with Oracle alone and
two with Thrasios out. (5) The lock in the pile (Silence), and a free counter or Pact of Negation for Doomsday itself,
which Fierce Guardianship, Flusterstorm, Swan Song and Force of Negation can all hit. (6) Life: Doomsday leaves half your
life rounded down. At 1 life it kills you; a Probe paid in life afterwards needs the life left to be 3 or more, so
pay {U} for it below that.

#### Board-state decisions

Read a row as: the window, what you have, what the table has, which tutor, which card, and why. "Free counters" means the
count from the list above.

**No win line yet**

| Window | Your side | Opponents | Tutor and target | Why, and what to watch |
|---|---|---|---|---|
| Turn 1-2, your main phase | Five mana from a land, two fast-mana cards and Dark Ritual ({B}{B}{B} ready), 30+ life, no line card | Empty boards, no counters seen | Demonic Tutor, then Necropotence | It digs for both halves at 1 life a card. Pay no more than you can hold at 7: the cards are exiled face down and reach your hand at your next end step, unseen until then, and cleanup discards are exiled. This turn is setup. It skips your draw step (Piles A and B and Sylvan Library are off) |
| Any turn, your main phase | Two or more tutors, mana for them, none of Oracle, the exile half or Doomsday | Any | Narrow tutors take what only they reach: Eladamri's Call or Worldly Tutor for Oracle, Mystical Tutor for the exile half. An any-card tutor takes what is left | Mystical can't find a creature and Call can't find an instant. Run tutors that go to hand first. A top-of-library tutor goes last, and only at the end step before your turn or with a draw after it |
| Opponent's end step | Teferi's +1 used on your turn or Valley Floodcaller already out, 5 mana open (tutor 2, {B}{B}{B}), 6+ life, no Oracle, no exile half. With Borne Upon a Wind as the enabler it is 7 ({1}{U} more) | Tapped low | Demonic Tutor (made instant by the enabler) for Doomsday, cast at once | Pile A holds Oracle, Silence and its own draws, so one tutor and one enabler is a kill on your turn. The tutor must go to hand: a card tutored to the top comes after this end step |
| Your main phase, mana to spare | Brainstorm, Preordain or Ponder next to a tutor or fetch land | Any | Cantrip first, tutor or fetch second | Brainstorm puts back two cards and the shuffle buries them. Keep a tutored top card out of Ponder's shuffle |

**Win line in hand**

| Window | Your side | Opponents | Tutor and target | Why, and what to watch |
|---|---|---|---|---|
| Your turn | Oracle, 6 mana (tutor 2, Oracle 2, exile half 1, lock 1), 0 or 1 free counters | No blue open | Demonic Tutor for Demonic Consultation (3 mana with Oracle), or Tainted Pact (4) | Sorcery-speed tutor first, then Oracle, then the exile half. Pact over Consultation when an opponent can Mental Misstep: Consultation is mana value 1, Pact is 2. Grim Tutor costs a mana and 3 life more |
| Opponent's end step before your kill turn | Oracle, 1-2 mana open | Tapped low | Mystical Tutor ({U}) or Vampiric Tutor ({B}, 2 life) for the exile half | It lands on top for your draw step, and you untap with all your mana for Oracle, the exile half and the lock. Mystical reveals the card to the table; Vampiric doesn't |
| Opponent's end step | The exile half, {G}{W} open | Tapped low | Eladamri's Call for Oracle, or Worldly Tutor ({G}) for Oracle on top | Call puts Oracle in hand now; Worldly needs a draw. Call needs both colours: Birds, Hierarch, Command Tower, Chrome Mox or Mana Confluence can supply them |
| Your turn | Both halves, 4+ mana, no free counter | Blue mana up, cards in hand | Intuition or Demonic Tutor for Silence now, or Mystical Tutor for it at the end step before; Eladamri's Call for Grand Abolisher or Voice of Victory (Worldly Tutor at the end step before) | A lock wins more games than a spare combo piece. Silence is one mana and covers every opponent. The creatures stop spells on your turn but can be countered: cast them a turn early |
| Your turn | Both halves, 3-4 mana, two or more free counters (for example Force of Will with a spare blue card, Pact of Negation, Fierce Guardianship with a commander out) | Blue mana up | No tutor: go | Each tutor spends mana you need for Oracle and the exile half. Cast Oracle first, then the exile half with the trigger on the stack |
| Your turn | Oracle and Doomsday in hand, no way to cast it at an end step, no Probe or Borne | Any | Merchant Scroll or Demonic Tutor for Borne Upon a Wind; Demonic Tutor for Gitaxian Probe (Mystical Tutor at the end step before) | Pile C needs a draw spell in hand to start. Borne also gives flash, so with it you can cast Doomsday at an end step instead (Pile B) |
| Opponent's end step | Oracle in hand, Consultation and Pact no longer in the library, Teferi's +1 used on your turn or Valley Floodcaller out, 5 mana open (tutor 2, Doomsday {B}{B}{B}) | Tapped low | Demonic Tutor for Doomsday, then Doomsday | With either exile half still in the library, tutor for it instead: it is cheaper and costs no life. Doomsday with Oracle in hand is Pile B (Probe, Preordain, Silence) and costs half your life |
| Your turn, before a Doomsday | Oracle in hand, devotion 2, spare mana | Any | Cast Thrasios ({G}{U}) first; Eladamri's Call for Valley Floodcaller; Demonic Tutor for Rhystic Study or Mystic Remora (Worldly or Enlightened Tutor at the end step before) | Each point of devotion is one draw fewer in the pile. Count devotion when the trigger resolves, not when you cast: a blue permanent killed in response lowers it. Mockingbird as a copy of Oracle adds {U}{U} |
| Your turn | Basalt Monolith or Rings of Brighthearth missing, 5-8 mana from other sources | Few artifact counters | Demonic Tutor for the missing piece (Enlightened Tutor at the end step before); Eladamri's Call for Walking Ballista | Finish with Ballista or Thrasios (Line 4). Basalt and Rings are noncreature artifacts: Force of Negation and Fierce Guardianship stop them, Swan Song doesn't. Null Rod and Collector Ouphe stop the loop |

**Opponents hold counterspells or interaction**

| Window | Your side | Opponents | Tutor and target | Why, and what to watch |
|---|---|---|---|---|
| Your turn | Both halves, one free counter | One blue player, 2+ untapped, cards in hand | Intuition or Demonic Tutor for Silence or Orim's Chant (Mystical Tutor at the end step before) | Silence first, then Oracle, and keep the free counter for what is left. Orim's Chant stops only one player |
| Your turn | Both halves, one or two free counters | Two or more blue players untapped | Intuition (Silence, Orim's Chant, Veil of Summer), or Mystical Tutor at the end step before | Silence is the only lock that stops every opponent. A Chant on one leaves the other free to counter |
| Your turn | Both halves, mana for Veil of Summer ({G}) | A known Force of Will or unknown counters | Mystical Tutor at the end step before for Veil of Summer; Merchant Scroll now for Pact of Negation or Force of Will as backup | Veil in response to the counter: your spells can't be countered this turn, Force of Will included. It doesn't stop Mindbreak Trap, a Silence cast first, or a Stifle-type effect |
| Your turn | Oracle in hand, Cavern of Souls in library | Red mana up (Red Elemental Blast, Pyroblast) | Any-card tutor for Cavern of Souls the turn before, then play it naming Wizard; or Mystical Tutor at the end step before for Veil of Summer | The blasts counter a blue spell. Oracle with Cavern's mana can't be countered, and Consultation and Pact are black, so the blasts miss them. Cavern pays only one {U}: bring a second blue source |
| Your turn | Oracle, any halves | A Mindbreak Trap or a three-spell turn | Intuition or Demonic Tutor for Silence or Orim's Chant (Mystical Tutor at the end step before) | Trap is a spell, so a lock stops it. Veil of Summer doesn't, because Trap exiles |
| Your turn | Free counters held: Mental Misstep, Force of Will, Fierce Guardianship | White mana up (their Silence or Orim's Chant) | Merchant Scroll for Mental Misstep (Mystical Tutor at the end step before) | Silence and Orim's Chant have mana value 1. Keep the counter for their lock, not for your Oracle. Cast your own lock first |
| Your turn | Both halves | A stax piece (see Answering stax) | Merchant Scroll for Snapback, Chain of Vapor or Into the Flood Maw (Mystical Tutor at the end step before); an any-card tutor for Boseiju or Otawara | Bounce is enough on the kill turn. Archon of Emeria and Deafening Silence limit how many spells you cast: tutor the answer, not the combo |
| Your turn | Anything | Opposition Agent | No tutor, no fetch | They take every card you search for. Answer the Agent, or go from the hand you hold |

**An opponent is about to win**

| Window | Your side | Opponents | Tutor and target | Why, and what to watch |
|---|---|---|---|---|
| Their turn, you can't win first | 3-4 mana, a spare blue card | A spell combo (Oracle with Consultation, Underworld Breach, Kinnan with Basalt Monolith) | Intuition: Force of Negation, Fierce Guardianship, Swan Song (or Flusterstorm, Mental Misstep) | Instant, to hand, in time. A top-of-library tutor arrives too late unless Brainstorm, Borne Upon a Wind or a Rhystic Study trigger draws it. Name three that all answer the spell |
| Their turn | {G}{W} and {1}{B} open (Call 2, Bowmasters 2) | A combo built on X/1 creatures (Rograkh, Kinnan's mana creatures) | Eladamri's Call for Orcish Bowmasters | Fierce Guardianship and Force of Negation can't counter a creature. Bowmasters is a flash creature that deals 1 damage. Snapback, Deadly Rollick or Swords to Plowshares work too if you already hold one |
| Your turn, can't win | {W} to spare for their upkeep | They plan to win next turn | Demonic Tutor or Intuition for Silence or Orim's Chant, held and cast at their upkeep | It has to be in hand: a top-of-library tutor won't arrive in time. Silence stops their spells for the turn, not abilities or spells already on the stack |

**Timing, engines, mana and commanders**

| Window | Your side | Opponents | Tutor and target | Why, and what to watch |
|---|---|---|---|---|
| Opponent's end step | Instant tutor, mana open | Tapped for their turn | Top-of-library tutor | They have used their mana, and you untap with everything. Cast it after their last spell, before your untap |
| Your main phase | Top-of-library tutor, no way to draw | Any | Don't: use a hand tutor, or wait for the end step | A top tutor with no draw sits there while you pass the turn |
| Their turn | Mystic Remora or Rhystic Study out | Casting spells | A top tutor, cast just before their next spell | The trigger can draw the card sooner, but a table that pays {1} or {4} gives you nothing: don't count on it |
| After Doomsday | Mystic Remora or Rhystic Study out, pile in the library | Casting spells | No tutor: decide each trigger | Both triggers say "may": decline the draw when the pile doesn't need it, because it takes the next pile card and an early draw can strand a pile built for a draw step |
| Any turn | Thrasios out, {4} spare | Any | Top tutor, then Thrasios's {4} | Draws the tutored card at instant speed. A tutored land goes onto the battlefield tapped instead |
| Any turn | No commander out, Fierce Guardianship or Deadly Rollick in hand | Any | No tutor first: cast Thrasios ({G}{U}) | Either commander turns the free spells on. Thrasios is cheaper and adds to devotion; Tymna is a lifelink 2/2 |
| Your turn, combat | Tymna out and an attacker that connects | Few blockers | Top tutor before combat | Tymna's draw after combat takes the top card. Pay X life for X opponents dealt damage |
| Turn 2-3, short on mana | Two mana or fewer, colours fine | Any | Demonic Tutor for Sol Ring or Mana Vault (Enlightened Tutor at the end step before) | More mana before the line. Mana Vault taps for {C}{C}{C} once and costs life when it stays tapped at your draw step |
| Turn 2-3, wrong colours | No source of {U} or {B} for Oracle ({U}{U}) or Doomsday ({B}{B}{B}) | Any | Eladamri's Call for Birds of Paradise or Noble Hierarch; Demonic Tutor for a dual land, Mox Diamond or Chrome Mox | Sol Ring, Mana Vault and the Monoliths make only {C} and fix nothing. Birds and Hierarch can't tap the turn they arrive. Mox Diamond needs a land to discard, Chrome Mox a blue or black card to exile |
| Turn 1-3 | Oracle, a tutor and fast mana for the cheapest line | Opponents have 1-3 lands | Demonic Tutor for Demonic Consultation | Few counters can be cast at that mana; the free ones (Force of Will, Pact of Negation, Mental Misstep, Fierce Guardianship) are the risk, so hold one free counter of your own |
| Turn 4 or later | Both halves, no lock, no free counter | 4+ lands with mana open, Rhystic Study, stax | The lock first, then the line; wait a turn if there is none | A stopped attempt costs about two turns. Spend the turn on a lock creature or Teferi, and keep a free counter up on theirs |

**Life totals**

| Window | Your side | Opponents | Tutor and target | Why, and what to watch |
|---|---|---|---|---|
| Any | 10 life or less | Any | Demonic Tutor, Worldly Tutor, Eladamri's Call, Mystical Tutor | Skip Grim Tutor (3), Vampiric Tutor and Imperial Seal (2 each) and a long Necropotence run |
| Any | 20 life | Any | Anything | Doomsday leaves half. After Vampiric Tutor (2) and a fetch (1), 20 becomes 17, then 8 after Doomsday |
| Any | 5 life or less | Any | No Doomsday unless Probe is paid with {U} | Doomsday leaves 2 or less, and a Probe paid in life needs 3 or more left. At 1 life it kills you |

**What to fetch first**

1. Both halves in hand: no tutor. Fetch a lock only if you hold no free counter.
2. One half in hand: fetch the other. Oracle missing: Eladamri's Call, then Demonic Tutor, then Worldly Tutor on top at the end step before.
   Exile half missing: Mystical Tutor at the end step before, or Demonic Tutor on your turn. Pact if they have Mental Misstep, otherwise Consultation.
3. Neither half: Necropotence on turns 1-2 if you can cast it this turn. With a flash enabler and {B}{B}{B}, Doomsday
   (Pile A), fetched with a tutor that goes to hand. Otherwise the piece your other tutors can't reach.
4. The lock, in this order: Silence, then Grand Abolisher or Voice of Victory a turn early, then Orim's Chant for one
   player, then Veil of Summer against red blasts and Force of Will.
5. Protection for the pieces themselves: Force of Will, Pact of Negation, Fierce Guardianship.
6. Mana and colours: the missing source.
7. Engines (Rhystic Study, Mystic Remora, Necropotence) only when nothing above applies.

**Tutor mistakes**

1. Casting the exile half before Oracle is on the battlefield, or while Oracle is still in the library: the library is
   exiled with Oracle in it.
2. Using Demonic Consultation as a tutor: a card in the top six is exiled and the library follows it.
3. Using Tainted Pact as a tutor on an unknown library: the cards above the one you take are gone.
4. Stacking top-of-library tutors: each search shuffles, and the second shuffles the first card away. Draw it first, or
   put the top tutor last.
5. Cracking a fetch land after a tutor to the top. The shuffle buries the card.
6. Casting a sorcery-speed tutor (Demonic Tutor, Grim Tutor, Diabolic Intent) with Oracle's trigger on the stack.
7. A top tutor in your main phase with no draw, so the card sits there until next turn.
8. Paying Necropotence for more cards than you can hold at 7. The cards arrive face down and unseen, you discard to 7 at cleanup, and the discards are exiled.
9. Pitching Oracle or the exile half to Force of Will, Force of Negation or Snapback.
10. Casting Demonic Consultation free off Beseech the Mirror. The free cast needs bargain and is optional (otherwise the card goes to hand), and a Consultation cast that way has no Oracle trigger to answer.
11. Diabolic Intent sacrificing the commander that makes Fierce Guardianship free.
12. Tutoring while an opponent controls Opposition Agent.
13. Paying life on tutors, fetches and Probe, and then having too little for the Doomsday that follows.
14. Using Gifts Ungiven or Intuition with piles where one choice leaves you without a line: the opponent picks the
    worst one.
15. Casting Pact of Negation on a turn when you may not win, with five mana ({3}{U}{U}) not certain next upkeep.

**Intuition and Gifts Ungiven piles** (Intuition: name three cards where any one does the job; Gifts: four where any
two are enough).

| You hold | Pile | Why it works |
|---|---|---|
| Thassa's Oracle | Demonic Consultation, Tainted Pact, Mystical Tutor (Intuition) | Any of the three gets the exile half |
| Demonic Consultation or Tainted Pact | Thassa's Oracle, Eladamri's Call, Worldly Tutor (Intuition) | Any of the three gets Oracle |
| Both halves, need a lock | Silence, Grand Abolisher, Voice of Victory (Intuition) | Each stops every opponent on your turn |
| Thassa's Oracle | Demonic Consultation, Tainted Pact, Doomsday, Mystical Tutor (Gifts Ungiven) | Any two give you an exile half or a path to one. Doomsday needs a flash enabler or Pile C. Noxious Revival can put a card the opponent sent to the graveyard back on top |
| Their combo coming | Force of Negation, Fierce Guardianship, Swan Song (Intuition, on their turn) | Each counters a noncreature spell |

**Beseech the Mirror.** Best targets are Oracle (holding the exile half), Necropotence, or Doomsday (mana value 3). Cast
free, Doomsday still works as a sorcery on your turn: use Pile C. Fodder for the bargain: a Treasure, Lotus Petal, a
Mox, Sol Ring, a Mystic Remora about to expire or the Orc Army token. A Mox or Sol Ring can tap for mana while you cast Beseech and
then be sacrificed.

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
9. Paying Necropotence for more cards than you can hold at seven, or casting Doomsday Piles A and B under it.
10. Casting Chain of Vapor mid-combo: the opponent sacrifices a land, copies it, and bounces your Oracle.
11. Using a top-of-library tutor in your own main phase with no way to draw the card this turn.
12. Losing your last commander at your upkeep and then counting Fierce Guardianship as free.

## Mana

27 lands. Other mana: Lotus Petal, Chrome Mox, Mox Diamond, Sol Ring, Mana Vault, Grim Monolith, Basalt Monolith,
Arcane Signet, Lion's Eye Diamond, Dark Ritual, Cabal Ritual, Elvish Spirit Guide, three mana creatures (Birds of
Paradise, Noble Hierarch, Avacyn's Pilgrim), and Treasures from Lotho and Smothering Tithe.

- **Original duals:** Underground Sea, Bayou, Scrubland, Tundra, Tropical Island.
- **Shock lands:** Overgrown Tomb, Godless Shrine, Hallowed Fountain, Watery Grave.
- **Basic:** Swamp (any black fetch finds it; safe against Blood Moon and Back to Basics).
- **Fetch lands (10):** Flooded Strand, Polluted Delta, Misty Rainforest, Verdant Catacombs, Windswept Heath, Marsh Flats,
  Arid Mesa, Scalding Tarn, Bloodstained Mire, Wooded Foothills. Each fetch also finds a dual with one of its land
  types; the ones naming Mountain still find the Plains, Island, Swamp or Forest half.
- **Others:** Command Tower, Mana Confluence, Gemstone Caverns, Cavern of Souls (name Wizard), Ancient Tomb, Boseiju,
  Otawara.
- **{B}{B}{B} for Doomsday:** Underground Sea, Bayou, Scrubland, Overgrown Tomb, Godless Shrine, Watery Grave and the
  basic Swamp are Swamps. Dark Ritual turns one {B} into three, and Cabal Ritual does the same once you have seven
  cards in your graveyard.

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

**Lands (27).** Ancient Tomb, Arid Mesa, Bayou, Bloodstained Mire, Boseiju, Who Endures, Cavern of Souls, Command Tower, Flooded Strand, Gemstone Caverns, Godless Shrine, Hallowed Fountain, Mana Confluence, Marsh Flats, Misty Rainforest, Otawara, Soaring City, Overgrown Tomb, Polluted Delta, Scalding Tarn, Scrubland, Swamp, Tropical Island, Tundra, Underground Sea, Verdant Catacombs, Watery Grave, Windswept Heath, Wooded Foothills

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
1 Scalding Tarn
1 Scrubland
1 Silence
1 Smothering Tithe
1 Snapback
1 Sol Ring
1 Swamp
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
1 Watery Grave
1 Windswept Heath
1 Wooded Foothills
1 Worldly Tutor
```
