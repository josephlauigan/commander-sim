# Marchesa, the Black Rose — Grixis Sacrifice and Recursion

A Grixis (blue-black-red) sacrifice and recursion deck. Sacrifice outlets and death payoffs turn every creature that
dies into drain, and two returning effects bring the creatures back: Mikaeus, the Unhallowed gives the non-Humans undying,
and Marchesa returns any creature card that dies with a +1/+1 counter. Enter-the-battlefield creatures (Grave Titan,
Mulldrifter, Plaguecrafter, Gonti) pay out again each time they come back, and a few loops win on the spot: Mikaeus with
Triskelion, Yawgmoth with two undying creatures, Nim Deathmantle with Ashnod's Altar and Priest of Gix or Grave Titan.
It is weak to graveyard hate and exile effects, to a table that kills the outlets before the engine runs, and to
decks that punish the life it pays (Reanimate, the tutors, the shocklands).

| | |
|---|---|
| **Commander** | Marchesa, the Black Rose ({1}{U}{B}{R}, 3/3 Human Wizard) |
| **Bracket** | 4 (3 Game Changers: Bolas's Citadel, Demonic Tutor, Vampiric Tutor; infinite loops: Mikaeus + Triskelion, Yawgmoth with two undying creatures and a death payoff, Nim Deathmantle + Ashnod's Altar + Priest of Gix or Grave Titan, Gravecrawler + Phyrexian Altar + a Zombie + a death payoff) |
| **Simulator** | deck key `marchesa` (tiers t3 and t4); plan file `data/plans/marchesa.toml`; findings in `audit/marchesa/FINDINGS.md`, history in `decklists/CHANGELOG.md` |

## Strategy

### The commander

**Marchesa, the Black Rose** ({1}{U}{B}{R}, 3/3, Human Wizard). *Dethrone. Other creatures you control have dethrone.
Whenever a creature you control with a +1/+1 counter on it dies, return that card to the battlefield under your control at
the beginning of the next end step.*

- **Dethrone** puts a +1/+1 counter on a creature whenever it attacks the player with the most life or tied for most.
  Marchesa gives it to every creature you control, so each attack at the life leader makes counters.
- **She returns cards, not tokens.** A creature card that dies with a +1/+1 counter comes back at the next end step as a
  new object, with its enters trigger again and no counter. A token ceases to exist when it dies, so it never returns: the
  Zombies from Grave Titan, the Goat from Woe Strider, the Human Soldier from Bastion of Remembrance and the Orc Army
  are fodder only.
- **Counters and undying cover each other.** A creature with no +1/+1 counter dies and Mikaeus's undying returns it
  with one; the next time it dies it has a counter, so Marchesa returns it instead. Undying checks for "no +1/+1
  counters", so a creature that has a counter (Carrion Feeder after it eats, Triskelion and Walking Ballista while they
  hold counters) is Marchesa's to return, not Mikaeus's.
- **She is a Human.** Mikaeus gives her no undying. If she dies with a +1/+1 counter on her, leave her in the graveyard
  instead of putting her in the command zone: the trigger returns "that card", and a card in the command zone is not in the
  graveyard to return.
- **Wipes.** A creature that dies in the same wipe as Marchesa still triggers her (leaves-the-battlefield triggers look
  back), so with counters or undying on the board Blasphemous Act is a mostly one-sided sweeper.

Marchesa is the deck's engine, not its only win: the combo lines below don't need her. Cast her when the table can't
answer her cheaply, with an outlet already on the board. Lightning Greaves (equip {0}: shroud and haste) puts her
out of reach of targeted removal and lets her attack the life leader the turn she lands.

### The packages

Each package has a job. The 64 spells split as follows (Marchesa is the 100th card, 35 lands the rest).

| Package | Cards | Job |
|---|---|---|
| Mana (9) | Sol Ring, Arcane Signet, Dimir Signet, Izzet Signet, Rakdos Signet, Talisman of Creativity, Talisman of Dominance, Talisman of Indulgence, Jet Medallion | Eight mana rocks and a cost reducer ({1} off black spells). Priest of Gix (below) is a third mana source |
| Sacrifice outlets (6) | Viscera Seer, Carrion Feeder, Woe Strider, Ashnod's Altar, Phyrexian Altar, Altar of Dementia | Free, repeatable sacrifice (Seer, Feeder, Strider, the Altars). Phyrexian Tower (a land) is a seventh, and Yawgmoth, Sephiroth and Deadly Dispute sacrifice too |
| Death payoffs (7) | Blood Artist, Zulaport Cutthroat, Bastion of Remembrance, Syr Konrad, Mayhem Devil, Sephiroth, Grave Pact | Each death drains (Artist, Zulaport, Bastion, Sephiroth, Konrad), pings (Mayhem Devil on any sacrifice) or makes the table sacrifice (Grave Pact) |
| Undying, counters and combo pieces (8) | Mikaeus, Triskelion, Yawgmoth, Nim Deathmantle, The Ozolith, Walking Ballista, Vampire Socialite, Gravecrawler | Mikaeus gives undying; Triskelion and Ballista enter with counters; The Ozolith moves the counters of a creature that leaves onto another; Vampire Socialite adds counters to other Vampires; Nim returns creatures for {4}; Gravecrawler is endless fodder; Yawgmoth turns two undying creatures into a loop |
| Enters-the-battlefield creatures (9) | Accursed Marauder, Plaguecrafter, Gonti, Mulldrifter, Grave Titan, Phyrexian Delver, Perigee Beckoner, Priest of Gix, Sauron | The value that returning creatures replay: edicts (Marauder, Plaguecrafter), a stolen card (Gonti), two cards (Mulldrifter), two Zombies (Titan), a reanimation (Delver), BBB (Priest), a copied enters trigger (Sauron), a one-turn return for another creature (Beckoner) |
| Reanimation (3) | Entomb, Reanimate, Animate Dead | Entomb a creature, then Reanimate or Animate Dead it; they also take a creature from an opponent's graveyard |
| Tutors (4) | Demonic Tutor, Diabolic Tutor, Grim Tutor, Vampiric Tutor | Find the missing combo piece. Demonic and Vampiric Tutor are two of the three Game Changers (Bolas's Citadel is the third) |
| Interaction (12) | Bitter Triumph, Fatal Push, Lightning Bolt, Terminate, Bedevil, Chaos Warp, Blasphemous Act, Counterspell, Arcane Denial, Swan Song, Mindbreak Trap, Propaganda | Cheap removal, one wipe, four counters and an attack tax |
| Draw and finisher (5) | Fact or Fiction, Deadly Dispute, Skullclamp, Black Market Connections, Bolas's Citadel | Refill; Deadly Dispute and Skullclamp turn a death into cards; Bolas's Citadel plays cards off the top for life equal to their mana value (the deck averages under 3) and is an alternate finisher: {T}, sacrifice ten nonland permanents and each opponent loses 10 |
| Protection (1) | Lightning Greaves | Shroud and haste for Marchesa or the engine creature |

Plus 35 lands (see "Mana").

**Humans get no undying** (Mikaeus: "other non-Human creatures"): Marchesa, Yawgmoth, Zulaport Cutthroat, Plaguecrafter,
Priest of Gix, Syr Konrad and Sephiroth (the front face; the flipped Sephiroth, One-Winged Angel is not a Human). The
good fodder is non-Human and nontoken: Accursed Marauder, Blood Artist, Carrion Feeder, Gonti, Grave Titan, Gravecrawler,
Mayhem Devil, Mulldrifter, Perigee Beckoner, Phyrexian Delver, Sauron, Triskelion, Vampire Socialite, Viscera Seer,
Walking Ballista and Woe Strider.

### How the deck wins

Six lines. Lines 1 to 4 are infinite loops, 5 and 6 are value that wins by itself. The tutors find the missing half.

#### Line 1: Mikaeus + Triskelion

**Mikaeus, the Unhallowed** ({3}{B}{B}{B}, 5/5): *Other non-Human creatures you control get +1/+1 and have undying.*
**Triskelion** ({6}): *Enters with three +1/+1 counters. Remove a +1/+1 counter: it deals 1 damage to any target.*

1. Mikaeus out. Cast Triskelion (no tap symbol, so you can use it at once).
2. Remove all three counters. Aim two pings at Triskelion itself (with Mikaeus it is a 2/2 once the counters are gone)
   and one at an opponent. It dies with no +1/+1 counters, so undying returns it.
3. It comes back with three counters plus the undying counter: four. Remove all four: two at itself, two at the
   opponents. It dies, undying returns it again. Each cycle is 2 damage you choose, repeated without limit.
4. Stop when every opponent is dead. Every Triskelion death also drains with each death payoff on the board.

- **Cast Triskelion second.** If they answer Triskelion in response (Swords to Plowshares, Path to Exile), remove the
  counters in response and ping with them.
- **If Mikaeus is answered mid-loop,** Triskelion has no undying and the loop ends. Pings already dealt stay dealt.
- Mikaeus has intimidate, and destroys any Human that deals damage to you.

#### Line 2: Yawgmoth + two undying creatures

**Yawgmoth, Thran Physician** ({2}{B}{B}, Human, protection from Humans): *Pay 1 life, sacrifice another creature: put a
-1/-1 counter on up to one target creature and draw a card.* With Mikaeus out and **two other nontoken non-Human
creatures** (call them A and B, neither with a +1/+1 counter), the loop runs:

1. Sacrifice A: B gets a -1/-1 counter, you draw. A died with no +1/+1 counter, so it returns with one.
2. Sacrifice B: A gets a -1/-1 counter, which cancels the +1/+1 counter that undying gave it. B died with no +1/+1
   counter, so it returns with one.
3. Repeat. Every activation is a death (each payoff drains: Blood Artist and Zulaport Cutthroat return the 1 life
   Yawgmoth costs) and a card. Stop before you draw your library.

- **Mikaeus does not give himself undying and Yawgmoth is a Human**, so the loop needs two other nontoken non-Human
  creatures. With one, the loop stalls. Tokens do not return.
- Each creature must survive the other's -1/-1 counter: with Mikaeus's +1/+1, a creature with printed toughness 1 (Blood
  Artist, Viscera Seer) does.
- It needs a death payoff (Blood Artist, Zulaport Cutthroat, Bastion of Remembrance, Syr Konrad, Sephiroth) to win; the
  cards you draw are a bonus.

#### Line 3: Nim Deathmantle + Ashnod's Altar + Priest of Gix or Grave Titan

**Nim Deathmantle** ({2}): *Whenever a nontoken creature is put into your graveyard from the battlefield, you may pay {4}.
If you do, return that card to the battlefield and attach this Equipment to it.* **Ashnod's Altar**: *Sacrifice a
creature: add {C}{C}.* **Priest of Gix**: *When this creature enters, add {B}{B}{B}.*

1. Nim and the Altar out (Nim does not need to be equipped). Cast Priest of Gix: {B}{B}{B}.
2. Sacrifice Priest to the Altar: {C}{C}. Nim triggers: pay {4} out of the five mana in your pool. Priest returns and adds
   {B}{B}{B} again.
3. Each cycle nets one mana and one death, so it drains with every payoff and builds mana for a sink. Walking Ballista
   ({4}: put a +1/+1 counter; remove a counter: 1 damage) turns the mana into damage.

With **Grave Titan** in place of the Priest, each cycle returns the Titan, which makes two Zombies; sacrifice the Zombies
to the Altar too ({C}{C} each) and the loop nets two mana a cycle. Needs two mana to start.

#### Line 4: Gravecrawler + Phyrexian Altar + a Zombie + a payoff

**Gravecrawler** ({B}): *You may cast this card from your graveyard as long as you control a Zombie.* **Phyrexian Altar**:
*Sacrifice a creature: add one mana of any color.* With another Zombie on the battlefield (Carrion Feeder, Accursed
Marauder, Phyrexian Delver, Mikaeus, a Grave Titan Zombie token, or a creature wearing Nim Deathmantle), sacrifice Gravecrawler
to Phyrexian Altar for {B}, cast it from the graveyard for that {B}, repeat. Every cycle is a death for each payoff
on the board. The Altar's mana can be any colour, so it pays the {B}; Ashnod's Altar's {C}{C} can't.

#### Line 5: Sauron's attack replays an enters trigger

**Sauron, the Necromancer** ({3}{B}{B}, 4/4 menace): *Whenever Sauron attacks, exile target creature card from your
graveyard. Create a tapped and attacking token that's a copy of that card, except it's a 3/3 black Wraith with menace. At the
beginning of the next end step, exile that token unless Sauron is your Ring-bearer.* The copy enters, so its enters trigger
fires: Grave Titan's two Zombies, Mulldrifter's two cards, Priest of Gix's {B}{B}{B}, Plaguecrafter's and Accursed
Marauder's edicts, Gonti's stolen card, Phyrexian Delver's reanimation (which can return another creature card).
Pair it with Entomb to put the creature in the graveyard first. The token is exiled at the end step, not killed, so no
payoff and no Marchesa trigger.

#### Line 6: Entomb into Reanimate, and the aristocrats drain

- **Entomb + Reanimate** is two mana for a six-drop on turn 2: Entomb ({B}, instant) puts Grave Titan or Mikaeus in the
  graveyard, Reanimate ({B}) returns it and you lose life equal to its mana value (6 for Grave Titan or Mikaeus). Animate
  Dead ({1}{B}, an Aura) takes no life but the creature gets -1/-0 and is sacrificed if the Aura leaves. Both take the
  best creature from any graveyard, an opponent's included. Entomb is an instant: use spare mana at the end of an opponent's
  turn.
- **The drain grind.** With an outlet, a death payoff and fodder, every sacrifice is a drain: Blood Artist, Zulaport
  Cutthroat and Bastion of Remembrance take 1 and give 1 per death, Syr Konrad deals 1 per death (also when a creature
  card is milled, discarded or leaves your graveyard), Mayhem Devil pings on any sacrifice (Treasures included), Grave Pact makes each other player sacrifice a
  creature whenever one of yours dies. The undying and Marchesa returns refill the fodder.

### Game plan

**Turns 1-3.** Mana and cheap engine pieces. Turn 1: a tapped land (Crumbling Necropolis, Path of Ancestry), Sol Ring,
Carrion Feeder, Gravecrawler, Viscera Seer, Fatal Push held, Entomb at an opponent's end step. Turn 2: a Signet or
Talisman, Blood Artist, Zulaport Cutthroat, Vampire Socialite, Jet Medallion. Turn 3: a Priest of Gix (BBB), an Altar,
Woe Strider, Lightning Greaves, Nim Deathmantle, a tutor.

**Turns 4-6.** Marchesa (an outlet first, Greaves on her if the table has removal), then Mikaeus (6 mana, BBB), Grave
Titan, Sauron, Phyrexian Delver, the draw. Hold Counterspell or Mindbreak Trap mana on the turn that completes a loop.

**Late.** Take the line the board allows. If a loop piece is missing, a tutor finds it: Demonic Tutor and Diabolic Tutor
to hand, Vampiric Tutor to the top for the next draw, Grim Tutor for 3 life. Count the table's graveyard hate and
sacrifice effects first.

**Hold up or go.** Go when the table is tapped low, when the loop is two pieces and you hold both, or when someone else
is about to win. Hold Mikaeus or the loop piece when the table has open counters you can't match: Mindbreak Trap exiles any
number of spells (free if an opponent cast three or more spells this turn), Counterspell stops anything, Swan Song stops
an enchantment, instant or sorcery.

**Sequencing rules.** An outlet before Marchesa. Fodder before the loop piece. Attack the player with the most life so
dethrone makes counters. Never sacrifice Marchesa unless she carries a counter and you leave her in the graveyard.

### Mulligans

The first mulligan is free (you see two 7-card hands). After that it is a London mulligan: draw 7, put one card on the
bottom for each mulligan after the free one. Marchesa is in the command zone, so she is never in the hand.

| Hand | Keep when |
|---|---|
| 7 cards | 2-5 lands (a mana rock counts as half a land) including a source of {B}, with the other two colours in reach by turn 4 (the shocklands, fetchlands and Command Tower give all three); a play on turns 1-3; and a reason to go: an outlet with fodder, a payoff, a tutor, Entomb with a reanimation spell. 0-1 lands, and 6+ lands, go back |
| 6 or 5 cards | 2-4 lands including a {B} source, and two plays that cost 3 or less |
| 4 cards | Keep |

- **No {B} source goes back.** Almost every engine piece needs black; Mikaeus, Bolas's Citadel and Grave Pact need {B}{B}{B}.
- **Bottom first:** Mikaeus, Grave Titan, Triskelion, Bolas's Citadel (six mana each) and the reactive blue cards when the hand has no engine.
- **A hand of only six-drops is a ship**, whatever the lands.

| # | Hand | Call | Why |
|---|---|---|---|
| 1 | Swamp, Watery Grave, Badlands, Sol Ring, Viscera Seer, Blood Artist, Entomb | Keep | Three colours in the lands, Sol Ring on turn 1, Seer and Blood Artist for the first drain, Entomb for the graveyard |
| 2 | Command Tower, Swamp, Talisman of Indulgence, Carrion Feeder, Gravecrawler, Zulaport Cutthroat, Demonic Tutor | Keep | Two lands and a Talisman, three cheap engine pieces (Feeder, Gravecrawler and Zulaport), and a tutor to find the next |
| 3 | Swamp, Swamp, Badlands, Entomb, Reanimate, Animate Dead, Gravecrawler | Keep | Entomb into Reanimate or Animate Dead is a Grave Titan or Mikaeus on turn 2 or 3; Gravecrawler is cheap fodder. Mind the 6 life if you use Reanimate |
| 4 | Swamp, Swamp, Swamp, Mikaeus, Grave Titan, Triskelion, Mulldrifter | Ship | Three lands, no rock, and every spell costs 5 or 6 |
| 5 | Island, Island, Steam Vents, Counterspell, Arcane Denial, Swan Song, Fact or Fiction | Ship | No black source, and nothing in the hand advances the plan |
| 6 | Underground Sea, Swamp, Swamp, Priest of Gix, Ashnod's Altar, Nim Deathmantle, Walking Ballista | Keep | Priest, Altar, Nim and a Ballista are Line 3 with a sink; the lands are on colour and the curve is 3 or less |

### What to tutor for

There are four tutors (Demonic, Diabolic, Grim, Vampiric) and Entomb. Marchesa is in the command zone, so no tutor finds
her. Demonic Tutor ({1}{B}), Diabolic Tutor ({2}{B}{B}) and Grim Tutor ({1}{B}{B}, lose 3 life) put the card in your
hand; Vampiric Tutor ({B}, lose 2 life) puts it on top of your library, so you have it at your next draw.

| Situation | Tutor for | Why, and what to watch |
|---|---|---|
| Mikaeus out, no Triskelion, 6 mana for Triskelion | Triskelion | Line 1. Cast it second |
| Triskelion out, no Mikaeus | Mikaeus | He is a creature: bring him down when the table can't answer. {3}{B}{B}{B} |
| Mikaeus and Yawgmoth out, one other nontoken non-Human creature | A second one: Blood Artist, Viscera Seer, Carrion Feeder, Gravecrawler | Line 2 needs two. Pick a creature with toughness 1 or higher that survives a -1/-1 counter |
| Nim and an Altar on the board | Priest of Gix (3 mana) | Line 3 needs no outside mana. Grave Titan needs 2 spare mana to start |
| A payoff is missing in a loop | Blood Artist, Zulaport Cutthroat, Syr Konrad or Bastion of Remembrance | Any death payoff turns each cycle into a drain |
| The graveyard is the plan (no graveyard hate shown) | Entomb, then the creature | Grave Titan or Mikaeus, then Reanimate or Animate Dead |
| The table has a wipe or counters | Mindbreak Trap or Counterspell, kept up | Protect the turn that completes a loop |
| An opponent has Opposition Agent | Nothing, kill the Agent | It takes whatever you search for |
| Tergrid out | Terminate or Bitter Triumph for Tergrid | Tergrid takes each nontoken permanent you sacrifice or discard: remove her first, and sacrifice only tokens (Grave Titan's Zombies, the Goat, Bastion's Soldier) until she is gone |

### Protecting the engine

| Threat | Answer in the deck |
|---|---|
| Targeted removal on Marchesa or an engine creature | Lightning Greaves (equip {0}, sorcery speed: put it on before the table acts; the equip uses the stack, so it is too late once removal is on the stack); Counterspell, Arcane Denial and Swan Song against the removal spell |
| A wipe | Mikaeus's undying and Marchesa's counters bring the non-Humans back; keep cards in hand that don't rely on being on the board |
| Graveyard hate (Farewell, Bojuka Bog, Dauthi Voidwalker, Skullsnatcher) | Entomb and Animate Dead are off; Counterspell or Swan Song the exile; the loops on the battlefield (Lines 1 to 3) don't need the graveyard |
| Edicts | Keep tokens (Zombies, Goat, Soldier) and cheap fodder on the board, so the edict takes those; sacrifice a creature in response if it is the one that will be taken |
| Counters | Cast the loop piece with Counterspell or Mindbreak Trap mana up; Swan Song can't hit creatures |
| Exile effects, bounce | Recast from hand or the command zone (Marchesa costs {2} more for each earlier cast); a creature card in a graveyard is still fodder for Reanimate |

### Interaction on opponents' turns

| Card | Cost, speed | What to use it on |
|---|---|---|
| Counterspell | {U}{U} instant | Any spell; the key wipe or combo piece |
| Arcane Denial | {1}{U} instant | Any spell; its caster draws up to two cards at the next upkeep and you draw one |
| Swan Song | {U} instant | An enchantment, instant or sorcery spell; its controller gets a 2/2 Bird |
| Mindbreak Trap | {2}{U}{U} instant (free if an opponent cast three or more spells this turn) | A combo turn: exile any number of spells, the Aluren loop included |
| Fatal Push, Bitter Triumph, Lightning Bolt, Terminate | {B}, {1}{B}, {R}, {B}{R} | Cheap creature removal; Push kills a creature of mana value 2 or less (4 or less if a permanent left the battlefield under your control this turn: a sacrifice turns it on) |
| Bedevil, Chaos Warp | {B}{B}{R}, {2}{R} instant | Bedevil: an artifact, creature or planeswalker (Embercleave). Chaos Warp: any permanent, enchantments included (Aluren, Rhystic Study) |
| Entomb, Vampiric Tutor | {B} instant | At an opponent's end step, so the next piece is ready on your turn |
| Deadly Dispute | {1}{B} instant | In response to removal or an edict on a creature: sacrifice it, draw two and make a Treasure |
| Outlets: Viscera Seer, Carrion Feeder, Woe Strider, the Altars, Phyrexian Tower | Free | In response to a steal effect or removal, to get a drain from a creature that is about to die |

### Matchups (Tiers 3 and 4)

Your tiers are t3 (Atraxa, Aurelia, Korvold, Marwyn, Tergrid) and t4 (Chulane, Edgar Markov, Krenko, Prosper, Yuriko).
The list below is the cards and plans against each commander; it is not a simulated result.

**Against combat decks** (Aurelia, Krenko, Edgar Markov, Marwyn's Craterhoof, Atraxa's poison). **Propaganda** ({2}{U}:
creatures can't attack you unless their controller pays {2} for each attacking creature) is the first answer; **Blasphemous
Act** ({8}{R}, costs {1} less per creature on the battlefield, 13 damage to each creature) clears the board and your own
non-Humans come back (undying, or Marchesa's counters); the edicts (Accursed Marauder, Plaguecrafter, Grave Pact) take
one creature at a time, a token if the table has them, so they are weak against token decks; cheap removal (Bolt, Push,
Terminate) takes the commander.

**Against combo decks** (Chulane with Aluren, Edgar's life-drain loops, Prosper's Citadel combo, Korvold's sacrifice
engine). **Mindbreak Trap** exiles every spell on the stack and is free once they have cast three, which is when a loop
is cast; **Counterspell** and **Arcane Denial** stop a piece; **Swan Song** stops an enchantment, instant or sorcery
(Aluren, a tutor, a wipe) but gives them a Bird; Chaos Warp answers any permanent on the board (an Aluren), Bedevil an artifact.

| Opponent | Their threat | Your plan | Cards that matter |
|---|---|---|---|
| Atraxa, Praetors' Voice (t3, WUBG infect and planeswalkers) | Poison: Blighted Agent (unblockable), Plague Stinger and Skithiryx (fliers), then a proliferate every turn. Ten poison kills at any life total. Farewell exiles creatures, enchantments, artifacts and graveyards | Kill the poison creatures on sight (Fatal Push or Bolt kills a 1/1), race with the loops, and counter Farewell (a sorcery: Counterspell, Swan Song, Mindbreak Trap). With the loops on the battlefield, the graveyard is not needed | Fatal Push, Lightning Bolt, Terminate, Swan Song, Counterspell |
| Aurelia, the Warleader (t3, Boros extra combats) | Hasty flying attackers (Aurelia, Hero of Bladehold, Goblin Rabblemaster, Legion Warboss) with Embercleave and extra combats; Mother of Runes and Giver of Runes protect them; Blasphemous Act | Propaganda taxes every attacker. Kill Mother of Runes and Giver of Runes first. Terminate or Bitter Triumph on Aurelia. Bedevil or Chaos Warp for Embercleave. Blasphemous Act against the board, with your engine returning | Propaganda, Terminate, Bedevil, Fatal Push, Blasphemous Act |
| Korvold, Fae-Cursed King (t3, Jund sacrifice) | Korvold draws and grows off every sacrifice; Blood Artist, Zulaport Cutthroat, Mayhem Devil, Goblin Bombardment, Impact Tremors; Chatterfang and Pitiless Plunderer; Terror of the Peaks; Toxic Deluge and cheap removal | A mirror: your outlets and payoffs are faster and returning. Kill Korvold (Terminate). Hold a counter for Toxic Deluge or Terror of the Peaks. Edicts are weak against their tokens | Terminate, Counterspell, Swan Song, Grave Pact, Mindbreak Trap |
| Marwyn, the Nurturer (t3, mono-green elves) | A board of mana elves, then Craterhoof Behemoth once Gaea's Cradle and the elves are out (Natural Order, Green Sun's Zenith, Chord of Calling, Finale of Devastation find it). Killing Craterhoof after it enters doesn't stop its trigger | Kill the mana elves early and sweep before the swing: Blasphemous Act costs {1} less per elf. Propaganda taxes the swing. Counter Craterhoof or its tutors: Swan Song stops Natural Order, Green Sun's Zenith, Chord of Calling and Finale of Devastation, but not a creature | Blasphemous Act, Propaganda, Counterspell, Swan Song, Mindbreak Trap |
| Tergrid, God of Fright (t3, mono-black discard and edicts) | Tergrid takes every nontoken permanent you sacrifice or discard. Edicts (Sheoldred's Edict, Plaguecrafter, Fleshbag Marauder, Innocent Blood), Liliana of the Veil, Waste Not, Painful Quandary. Their deck has its own Sheoldred and Gray Merchant. Dauthi Voidwalker and Bojuka Bog hate the graveyard | The deck's own plan is the problem: every nontoken creature you sacrifice goes to Tergrid's controller. Kill Tergrid first (Terminate, Bitter Triumph), sacrifice tokens only while she is out (Zombies, Goat, Soldier). Reanimate and Animate Dead can take a creature from their graveyard too. Keep cheap tokens for their edicts | Terminate, Bitter Triumph, Reanimate, Animate Dead, Grave Titan, Counterspell |
| Chulane, Teller of Tales (t4, Bant value and Aluren) | Chulane draws and ramps off every creature spell. Aluren or Intruder Alarm with Shrieking Drake or Whitemane Lion loops creatures, draws the deck and wins with Thassa's Oracle, Laboratory Maniac or Craterhoof. Counterspell, Dovin's Veto, Swan Song, Fierce Guardianship; Cyclonic Rift; Drannith Magistrate | Mindbreak Trap against the loop (it exiles every spell and is free once three have been cast), Chaos Warp for Aluren or Intruder Alarm, Terminate for Chulane. Cast creatures into counterspell decks: Veto, Swan Song and Fierce can't hit creatures. Cast the loop piece when they are tapped | Mindbreak Trap, Chaos Warp, Terminate, Counterspell |
| Edgar Markov (t4, Mardu Vampires) | Eminence makes a 1/1 Vampire for every Vampire spell cast, from the command zone. Wide first-strike attacks (Edgar, Drana). Their Sanguine Bond, Vito and Exquisite Blood loops. Cheap removal (Swords, Path, Bolt, Terminate, Fatal Push, Anguished Unmaking), Damn, New Blood, Teferi's Protection | Propaganda and Blasphemous Act hold the board. Their Exquisite Blood and Bloodthirsty Conqueror gain life when you lose it: Reanimate, Grim Tutor, Bitter Triumph, Yawgmoth, Bolas's Citadel (the mana value of each card cast off the top), the Talismans and the shocklands start their loop, so don't pay life when their pieces are out | Propaganda, Blasphemous Act, Grave Pact, Terminate, Swan Song, Counterspell |
| Krenko, Mob Boss (t4, mono-red goblins) | Krenko doubles the Goblins each activation; Impact Tremors, Goblin Bombardment, Purphoros, Coat of Arms; haste from Lightning Greaves and Swiftfoot Boots. Thornbite Staff + an outlet is infinite. Blood Moon turns your nonbasic lands into Mountains | Kill Krenko before he untaps (Terminate, Lightning Bolt: he is a 3/3). Blasphemous Act is almost free against a Goblin board and your creatures return. Propaganda taxes the swarm. Under Blood Moon the 21 nonbasic lands become Mountains (Barad-dûr too) and only the 9 Swamps and 3 Islands make {B} and {U}, so hold the Swamps | Terminate, Lightning Bolt, Blasphemous Act, Propaganda, Counterspell |
| Prosper, Tome-Bound (t4, Rakdos exile and Treasure) | Exiles cards to play them and makes a Treasure each time; Opposition Agent (steals your tutors); Bolas's Citadel + Sensei's Divining Top + Aetherflux Reservoir; Underworld Breach; Toxic Deluge, Blasphemous Act, Delayed Blast Fireball | A wipe is weak against the undying and Marchesa returns. Kill Opposition Agent on sight and don't tutor under it. Counter the Citadel or Breach (Swan Song stops Underworld Breach, an enchantment) | Terminate, Counterspell, Swan Song, Mindbreak Trap, Fatal Push |
| Yuriko, the Tiger's Shadow (t4, Dimir ninjas) | Cheap evasive attackers (Ornithopter, Faerie Seer, Spectral Sailor), then ninjutsu Yuriko or another Ninja: each Ninja hit reveals a card and you lose life equal to its mana value. Counterspell, Force of Will, Fierce Guardianship, Flare of Denial, Misdirection; Cyclonic Rift; Rhystic Study | Propaganda taxes the attackers they need. Keep blockers back: Grave Titan's Zombies and the Goat are blockers. Kill the ninjas (Push, Bolt). Cast the creature pieces into their counters, and keep life high against the reveal | Propaganda, Fatal Push, Lightning Bolt, Terminate, Counterspell |

### Life management

The deck pays life: Reanimate (the creature's mana value), Grim Tutor (3), Vampiric Tutor (2), Bolas's Citadel (the mana value of
each card cast off the top), Bitter Triumph (3 unless you
discard), Yawgmoth (1 per activation, a Blood Artist or Zulaport trigger returns it), the shocklands (2 unless they
enter tapped), the fetchlands (1), the Talismans, Shivan Reef and Underground River (1 per coloured mana), Phyrexian Delver (the creature's mana value).
Keep your life above what the table can swing in a round, and don't pay life at all when an opponent has Exquisite Blood or
Bloodthirsty Conqueror with Sanguine Bond, Vito or Marauding Blight-Priest on the battlefield: your life loss starts
their loop.

### Common mistakes

1. Casting Marchesa or the loop piece into open counters. Hold Counterspell or Mindbreak Trap mana.
2. Sacrificing a creature with no outlet-and-payoff plan, or while Tergrid is out.
3. Counting on undying for a Human (Marchesa, Yawgmoth, Zulaport, Plaguecrafter, Priest of Gix, Syr Konrad, Sephiroth) or for a
   token.
4. Running the Yawgmoth loop with one undying creature, or with Mikaeus as one of the two.
5. Putting Marchesa in the command zone when she died with a counter and you want her back.
6. Casting Triskelion before Mikaeus is safe, or before the table is tapped.
7. Reanimate at low life: Grave Titan or Mikaeus costs 6.
8. Using Entomb or Animate Dead into graveyard hate.
9. Tutoring while an opponent controls Opposition Agent.
10. Overextending into a wipe when the recursion isn't online (no Mikaeus, no counters).

## What the simulator cannot do yet

- **The attack AI does not aim at the life leader for dethrone.** The counters work mechanically, but no attack logic
  picks the player with the most life to set up Marchesa's returns. In the simulator the counters come from Mikaeus's
  undying, Carrion Feeder, Triskelion, Walking Ballista and Vampire Socialite rather than from combat.
- **There is no Marchesa-specific AI.** The deck plays the generic policy plus `data/plans/marchesa.toml` (priorities for the
  outlets, payoffs and Marchesa). When to sacrifice a countered creature before the end step, when to cast Blasphemous Act and
  how to pay for Nim Deathmantle are not modelled as plans.
- **Commander return.** Marchesa's return (and Kaya's Ghostform's) takes a commander only from the graveyard, as the rules do:
  a commander put in the command zone is a new object and cannot be returned, so the simulator leaves her in the graveyard
  while her own return is pending.
- **The Yawgmoth loop check.** The simulator requires two nontoken undying creatures for the Yawgmoth, Mikaeus and fodder
  loop, as the rules do.
- **Walking Ballista + Mikaeus is not played as a line.** Ballista pings itself, dies with no +1/+1 counter and undying returns
  it with one; with Blood Artist, Zulaport Cutthroat, Syr Konrad or Bastion of Remembrance out it kills the table, but the
  simulator has no entry for it, so the AI does not set it up. Priest of Gix + Nim Deathmantle + Phyrexian Altar (mana-neutral,
  needs a payoff) is not modelled either.

## Mana

35 lands: Badlands, Barad-dûr, Blood Crypt, Bloodstained Mire, Command Tower, Crumbling Necropolis, Dragonskull Summit, Drowned
Catacomb, Luxury Suite, Morphic Pool, Path of Ancestry, Phyrexian Tower, Polluted Delta, Scalding Tarn, Shivan Reef,
Steam Vents, Sulfur Falls, Underground River, Underground Sea, Volcanic Island, Watery Grave, 3 Island, 2 Mountain and
9 Swamp. Twenty-four lands make {B} (25 counting Phyrexian Tower's {B}{B} when you sacrifice a creature), 17 make {U} and 15 make {R}. Eight mana rocks, seven of which add colour (Sol Ring makes {C}{C}): five can make {B} (Arcane Signet, Dimir Signet, Rakdos
Signet, Talisman of Dominance, Talisman of Indulgence), five {U}, five {R}; the Signets cost {1} to use and the Talismans
deal 1 damage to you for coloured mana, and so do Shivan Reef ({U} or {R}) and Underground River ({U} or {B}).

- **Heavy black:** Mikaeus ({3}{B}{B}{B}), Bolas's Citadel ({3}{B}{B}{B}) and Grave Pact ({1}{B}{B}{B}) are the hardest
  costs; the rest of the black cards need one or two {B}.
- **Blue is light:** Counterspell ({U}{U}) and Mindbreak Trap ({2}{U}{U}) are the only cards with two {U}; the other blue
  cards (Arcane Denial, Swan Song, Fact or Fiction, Mulldrifter, Propaganda) need one.
- **Red is light:** Bedevil ({B}{B}{R}) is the hardest; Lightning Bolt, Chaos Warp, Blasphemous Act, Terminate, Mayhem
  Devil and Vampire Socialite need one {R}.
- **Marchesa needs all three** ({1}{U}{B}{R}): count the sources before you cast her.
- **Tapped lands:** Crumbling Necropolis and Path of Ancestry always enter tapped; Luxury Suite, Morphic Pool,
  Dragonskull Summit, Drowned Catacomb and Sulfur Falls enter tapped under their conditions; Shivan Reef and Underground River never do; the shocklands
  cost 2 life to enter untapped. Phyrexian Tower makes {C}, or {B}{B} by sacrificing a creature. Barad-dûr is not a Swamp (the fetchlands cannot find it) and
  enters tapped unless you control a legendary creature; seven are in the list (Marchesa, Mikaeus, Yawgmoth, Sauron,
  Sephiroth, Syr Konrad, Gonti). Late in the game, {X}{X}{B}, {T}: amass Orcs X, only if a creature died this turn; the Army
  is a token, so Marchesa does not return it, but it is sacrifice fodder.

## Bracket and Rule 0

Bracket 4: three Game Changers (Bolas's Citadel, Demonic Tutor, Vampiric Tutor) and the infinite combos named here. There are no extra
turns and no mass land destruction, but there are infinite loops: Mikaeus + Triskelion (a two-card infinite-damage
combo), Yawgmoth + Mikaeus + two undying creatures with a death payoff, Nim Deathmantle + Ashnod's Altar + Priest of Gix or
Grave Titan, and Gravecrawler + Phyrexian Altar + a Zombie with a death payoff. Name these before the game. Tell the table
about the three Game Changers and the five tutors (Demonic, Diabolic, Grim, Vampiric and Entomb), that Marchesa returns
creatures that die with a +1/+1 counter, so a board wipe is weak against the deck, and that Propaganda, Counterspell and
Mindbreak Trap are the interaction.

## Decklist by type (100)
**Commander (1).** Marchesa, the Black Rose

**Creatures (23).** Accursed Marauder, Blood Artist, Carrion Feeder, Gonti Lord of Luxury, Grave Titan, Gravecrawler, Mayhem Devil, Mikaeus the Unhallowed, Mulldrifter, Perigee Beckoner, Phyrexian Delver, Plaguecrafter, Priest of Gix, Sauron the Necromancer, Sephiroth Fabled SOLDIER // Sephiroth One-Winged Angel, Syr Konrad the Grim, Triskelion, Vampire Socialite, Viscera Seer, Walking Ballista, Woe Strider, Yawgmoth Thran Physician, Zulaport Cutthroat

**Enchantments (5).** Animate Dead, Bastion of Remembrance, Black Market Connections, Grave Pact, Propaganda

**Artifacts (17).** Altar of Dementia, Arcane Signet, Ashnod's Altar, Bolas's Citadel, Dimir Signet, Izzet Signet, Jet Medallion, Lightning Greaves, Nim Deathmantle, Phyrexian Altar, Rakdos Signet, Skullclamp, Sol Ring, Talisman of Creativity, Talisman of Dominance, Talisman of Indulgence, The Ozolith

**Sorceries (5).** Blasphemous Act, Demonic Tutor, Diabolic Tutor, Grim Tutor, Reanimate

**Instants (14).** Arcane Denial, Bedevil, Bitter Triumph, Chaos Warp, Counterspell, Deadly Dispute, Entomb, Fact or Fiction, Fatal Push, Lightning Bolt, Mindbreak Trap, Swan Song, Terminate, Vampiric Tutor

**Lands (35).** Badlands, Barad-dûr, Blood Crypt, Bloodstained Mire, Command Tower, Crumbling Necropolis, Dragonskull Summit, Drowned Catacomb, Luxury Suite, Morphic Pool, Path of Ancestry, Phyrexian Tower, Polluted Delta, Scalding Tarn, Shivan Reef, Steam Vents, Sulfur Falls, Underground River, Underground Sea, Volcanic Island, Watery Grave, 3 Island, 2 Mountain, 9 Swamp

## Import list (100)

```
1 Accursed Marauder
1 Altar of Dementia
1 Animate Dead
1 Arcane Denial
1 Arcane Signet
1 Ashnod's Altar
1 Badlands
1 Barad-dûr
1 Bastion of Remembrance
1 Bedevil
1 Bitter Triumph
1 Black Market Connections
1 Blasphemous Act
1 Blood Artist
1 Blood Crypt
1 Bloodstained Mire
1 Bolas's Citadel
1 Carrion Feeder
1 Chaos Warp
1 Command Tower
1 Counterspell
1 Crumbling Necropolis
1 Deadly Dispute
1 Demonic Tutor
1 Diabolic Tutor
1 Dimir Signet
1 Dragonskull Summit
1 Drowned Catacomb
1 Entomb
1 Fact or Fiction
1 Fatal Push
1 Gonti, Lord of Luxury
1 Grave Pact
1 Grave Titan
1 Gravecrawler
1 Grim Tutor
1 Izzet Signet
1 Jet Medallion
1 Lightning Bolt
1 Lightning Greaves
1 Luxury Suite
1 Marchesa, the Black Rose
1 Mayhem Devil
1 Mikaeus, the Unhallowed
1 Mindbreak Trap
1 Morphic Pool
1 Mulldrifter
1 Nim Deathmantle
1 Path of Ancestry
1 Perigee Beckoner
1 Phyrexian Altar
1 Phyrexian Delver
1 Phyrexian Tower
1 Plaguecrafter
1 Polluted Delta
1 Priest of Gix
1 Propaganda
1 Rakdos Signet
1 Reanimate
1 Sauron, the Necromancer
1 Scalding Tarn
1 Sephiroth, Fabled SOLDIER // Sephiroth, One-Winged Angel
1 Shivan Reef
1 Skullclamp
1 Sol Ring
1 Steam Vents
1 Sulfur Falls
1 Swan Song
1 Syr Konrad, the Grim
1 Talisman of Creativity
1 Talisman of Dominance
1 Talisman of Indulgence
1 Terminate
1 The Ozolith
1 Triskelion
1 Underground River
1 Underground Sea
1 Vampire Socialite
1 Vampiric Tutor
1 Viscera Seer
1 Volcanic Island
1 Walking Ballista
1 Watery Grave
1 Woe Strider
1 Yawgmoth, Thran Physician
1 Zulaport Cutthroat
3 Island
2 Mountain
9 Swamp
```
