# Yoshimaru, Ever Faithful & Tymna the Weaver — Blink and Enter-the-Battlefield Value (W/B)

Put creatures with enter effects on the battlefield, then use them again, and back them with an Equipment package that protects the board, gives a second way to connect and tutors for itself. Solitude, Skyclave Apparition, Ranger-Captain of Eos, Recruiter of the Guard, Cloud and Stoneforge Mystic each do their job every time they come back, and seven blink cards bring them back. A life layer (Elas il-Kor, Haliya, Vito) turns every enter and every blink into drain, and Tymna with Delney draws twice as many cards. Four short lines can end the game: Vito with Exquisite Blood, Heliod with Walking Ballista, Archangel of Thune with Walking Ballista and a lifelink source, and Restoration Angel with Felidar Guardian and a payoff. Yoshimaru's counters are a bonus, not the engine: about a 4/4 to 5/5 by turn 6 (an estimate). The deck is weak to effects that stop enter triggers, to fast combo (its answers are Silence, Ranger-Captain of Eos and instant removal, it has no counterspell and no board shield for its own win turn), to graveyard and stax effects, to a sweeper when it taps out, to noncreature permanents it cannot remove, and to life loss. **This list has not been simulated; every number below is a count from the list or a labelled estimate.**

| | |
|---|---|
| **Commanders** | Yoshimaru, Ever Faithful ({W}, 1/1; white) and Tymna the Weaver ({1}{W}{B}, 2/2 lifelink; white-black). Both have Partner; the colour identity is white-black |
| **Bracket** | 4 (8 Game Changers: Mana Vault, Demonic Tutor, Vampiric Tutor, Enlightened Tutor, Imperial Seal, The One Ring, Orcish Bowmasters, Chrome Mox) |
| **Simulator** | Deck key `yoshimaru`, tiers t4-t5; findings in `audit/yoshimaru/FINDINGS.md`, history in `decklists/CHANGELOG.md`; plan file `data/plans/yoshimaru.toml`. No sims yet |

## Strategy

### The commanders

**Yoshimaru, Ever Faithful** ({W}, 1/1). *Whenever another legendary permanent you control enters, put a +1/+1 counter on Yoshimaru. Partner.*

**Tymna the Weaver** ({1}{W}{B}, 2/2). *Lifelink. At the beginning of each of your postcombat main phases, you may pay X life, where X is the number of opponents that were dealt combat damage this turn. If you do, draw X cards. Partner.*

Both have Partner and share the command zone. Each costs {2} more for every time it has been cast from there before.

- **Yoshimaru is a one-mana body that grows.** He counts every legendary permanent we play: 21 in the 98 (4 lands, 12 creatures, 4 artifacts and 1 planeswalker; Heliod is one of the creatures; all listed under Decklist by type), and every blink of one of them is another enter. Mithril Coat and Sting are legendary Equipment, so each is a counter when it enters. He has no evasion and no protection of his own, so he is a bonus, not the plan (see Yoshimaru's counters below).
- **Tymna is the card engine, but only if she connects.** X is the number of opponents hit by combat damage from any of our creatures that turn, so a flier, a token, a creature that cannot be blocked or an equipped creature that connects draws her cards. With Delney she draws twice (see Doubling triggers). Her lifelink is a life gain on every hit, which feeds Vito, Heliod and Archangel of Thune. Equipment on Tymna herself conflicts with Delney (see The Equipment package).

**The order rule.** Yoshimaru must be on the battlefield first. His trigger is for *another* legendary permanent that enters while he is there, so Tymna entering after him is his first counter. If Tymna is out first, Yoshimaru's own entry gets nothing, and neither does anything that came down before him. With four mana and neither commander out, cast Yoshimaru, then Tymna.

**When to cast each.**

- **Yoshimaru on turn 1** with a white land that enters untapped. Twenty-four lands in the list qualify: 14 Plains, Scrubland, Caves of Koilos, Command Tower, Eiganjo, Vault of Champions (untapped with two or more opponents), Plaza of Heroes (any colour for a legendary spell), and Godless Shrine (2 life), Witch Enchanter // Witch-Blessed Meadow (3 life), Marsh Flats and Arid Mesa (fetch an untapped Plains) at a small life cost. Minas Tirith (tapped without a legendary creature) and Isolated Chapel (tapped unless we control a Plains or a Swamp) enter tapped as a first land, and Urborg, Takenuma and the Swamps make no white. Chance of at least one of the 24 in the opening hand: 87.0% on the play (7 cards), 90.4% on the draw (8 cards).
- **Only Chrome Mox lets Yoshimaru come down before the land drop.** Chrome Mox ({0}): when it enters we may exile a nonartifact, nonland card from our hand, and it taps for one mana of a colour of that card, so with a white card imprinted it pays {W}: Mox, Yoshimaru, then the land. That order also lets a legendary land (Eiganjo, Takenuma, Urborg, Minas Tirith) come down after Yoshimaru as his first counter, and Minas Tirith then enters untapped. Sol Ring and Mana Vault make colourless mana and cannot pay {W}, and Mox Amber makes mana only of a colour among our legendary creatures and planeswalkers, so it needs a legend already on the battlefield. Without Chrome Mox the turn-1 order is land, then Yoshimaru, and his first counter comes on turn 2 from a legendary land, a legendary rock, a legendary creature or Tymna.
- **Chrome Mox costs a card.** 46 of the 98 cards can be imprinted (every nonland, nonartifact card; Witch Enchanter counts, since in hand it is a creature card; 39 are white and 9 black, and none is colourless), so there is almost always a card to give, but the card is gone for good. Imprint a white or black card we will not cast soon (a five-drop, a second payoff or a spare protection spell) and pick the card whose colour the hand is missing; with no such card in hand the Mox makes no mana. With an untapped white land on turn 1 it is two mana: Yoshimaru ({W}) and a one-drop ({W}) such as Giver of Runes or Skrelv. It is an artifact, so Enlightened Tutor can find it.
- **Mox Amber right after Yoshimaru** is a free legendary permanent entering (a counter) and then taps for {W}, because Yoshimaru is a white legendary creature. That {W} can cast Giver of Runes, Skrelv, Enlightened Tutor, Steelshaper's Gift or Ephemerate, or pay for Sol Ring on turn 1.
- **Tymna on turn 3**, normally. With Mox Amber and a black source she can come down on turn 2: {W} from Mox Amber (Yoshimaru), {B} from a land, {1} from the other. Chrome Mox does the same job with a white or black card imprinted: with Yoshimaru out on turn 1, two lands and the Mox on turn 2 pay {1}{W}{B} when the lands and the imprint cover both colours. Plaza of Heroes can also supply one coloured mana for her, since she is a legendary spell.
- **Tymna can't attack the turn she comes down, but she can still draw that turn.** Her trigger needs only that she is on the battlefield at the start of the postcombat main phase, and X counts the opponents dealt combat damage by any creature of ours. Cast her before combat and she draws for Yoshimaru's, Adeline's or a flier's hits. Cast Delney before her, so that the doubling is already there. With Swiftfoot Boots on her ({1} to equip) she has haste and can attack herself the turn she is cast.

**Which to protect.** Yoshimaru costs {W} to replace and loses his counters whenever he leaves; Tymna costs {1}{W}{B} plus {2} per recast and carries the card engine. Spend Giver of Runes, Ephemerate, Restoration Angel, Swiftfoot Boots and Mithril Coat on Tymna first, unless Yoshimaru is already large.

### How the deck plays

**The engine has three parts.**

1. **Enter effects reused by blink.** Sixteen cards do something worth repeating when they enter:

| Card | Mana value | When it enters |
|---|---|---|
| Cloud, Midgar Mercenary | 2 | Search for an Equipment card, reveal it, put it into hand. Legendary. As long as he is equipped, a triggered ability of Cloud or of an Equipment attached to him triggers an additional time |
| Stoneforge Mystic | 2 | We may search for an Equipment card, reveal it, put it into hand. Later, {1}{W}, {T}: we may put an Equipment card from hand onto the battlefield |
| Kor Outfitter | 2 | We may attach target Equipment we control to target creature we control |
| Orcish Bowmasters | 2 | Flash. 1 damage to any target, then amass Orcs 1; again for each extra card an opponent draws |
| Sunpearl Kirin | 2 | Flash, flying (a 2/1). Return up to one other target nonland permanent we control to its owner's hand; if it was a token, draw a card |
| Skyclave Apparition | 3 | Exile up to one target nonland, nontoken permanent we don't control with mana value 4 or less |
| Ranger-Captain of Eos | 3 | We may search for a creature card with mana value 1 or less. Sacrifice it: opponents can't cast noncreature spells this turn |
| Recruiter of the Guard | 3 | We may search for a creature card with toughness 2 or less |
| Flickerwisp | 3 | Flying. Exile another target permanent; it returns at the beginning of the next end step |
| Mithril Coat | 3 | Legendary Equipment with flash and indestructible: attach it to target legendary creature we control (it stays unattached if we control none). A blink of it is a new attach, and another counter for Yoshimaru |
| Forge Anew | 3 | Return target Equipment card from our graveyard to the battlefield |
| Felidar Guardian | 4 | We may blink another target permanent we control |
| Restoration Angel | 4 | Flash, flying. We may blink a target non-Angel creature we control |
| Witch Enchanter // Witch-Blessed Meadow | 4 | As a creature spell: destroy target artifact or enchantment an opponent controls. It is also one of the 35 lands |
| Court of Ardenvale | 4 | We become the monarch. Each of our upkeeps it returns a permanent card with mana value 3 or less from our graveyard to hand, or to the battlefield while we are the monarch |
| Solitude | 5 | Flash, lifelink. Exile up to one other target creature (its controller gains life equal to its power). Evoke: exile a white card from hand, so it can be free |

Puresteel Paladin reacts to these instead of having an enter effect of its own: it draws a card (a "may") whenever an Equipment we control enters, so a cast, Stoneforge's put, Forge Anew and a blink of an Equipment each draw. Ardenn has no enter effect: it moves Equipment (see The Equipment package).

2. **The life-and-drain layer.** Every enter and every blink is a point of life or a counter for these:

| Piece | Cost | What it does |
|---|---|---|
| Elas il-Kor, Sadistic Pilgrim | {W}{B} | Whenever another creature we control enters, we gain 1 life. Whenever another creature we control dies, each opponent loses 1 life |
| Haliya, Guided by Light | {2}{W} | Whenever Haliya or another creature or artifact we control enters, we gain 1 life. At our end step, draw a card if we gained 3 or more life. Warp {W} |
| Vito, Thorn of the Dusk Rose | {2}{B} | Whenever we gain life, target opponent loses that much. {3}{B}{B}: our creatures gain lifelink until end of turn |
| Exquisite Blood | {4}{B} | Whenever an opponent loses life, we gain that much |
| Heliod, Sun-Crowned | {2}{W} | Whenever we gain life, a +1/+1 counter on target creature or enchantment we control. {1}{W}: another target creature gains lifelink until end of turn |
| Archangel of Thune | {3}{W}{W} | Flying, lifelink. Whenever we gain life, a +1/+1 counter on each creature we control |

Haliya gains 1 life for every artifact too, so each Equipment, rock, Bauble or Treasure entering is a life point for the layer.

3. **Tymna with Delney, and the cards that come with the board.** Delney, Streetwise Lookout makes Tymna's trigger fire twice (see Doubling triggers). Puresteel Paladin, Haliya, Court of Ardenvale's monarchy, Buster Sword, Sword of Fire and Ice and The One Ring add cards around her. Puresteel Paladin draws once per Equipment entering; Buster Sword draws a card each time its carrier deals combat damage to a player and then lets us cast a spell from hand for free if its mana value is at most that damage; Sword of Fire and Ice draws a card on the same hit.

**Seven blink pieces:** Ephemerate, Restoration Angel, Flickerwisp, Felidar Guardian, Eerie Interlude, Teleportation Circle and Conjurer's Closet. Sword of Hearth and Home is a blink on a hit (see The Equipment package). **Three doublers:** Delney, Gandalf the White and Elesh Norn, Mother of Machines.

**Conjurer's Closet** ({5}, a colourless artifact, not legendary): *At the beginning of your end step, you may exile target creature you control, then return that card to the battlefield under your control.* It blinks only creatures (not artifacts or other permanents), only at the beginning of our own end step, one target, and it is a "may". It costs 5 and does nothing until a creature is on the battlefield, so it is a late-game engine and a reason the deck wants good creatures (Solitude, Skyclave Apparition, Ranger-Captain of Eos, Recruiter of the Guard, Cloud, Stoneforge Mystic) to repeat. Cast in one of our main phases it already blinks at that turn's end step; Gandalf's flash for artifact spells lets us cast it during combat, or at an opponent's end step so that it is ready for our next turn. It is an artifact entering, not a legend, so it gives Yoshimaru no counter, and it is not an Equipment, so Puresteel Paladin does not draw for it. It does set off cards that look for an artifact or any permanent entering (Haliya gains 1 life, and Gandalf and Elesh Norn each add a trigger, see Doubling triggers). Its own end-step trigger is not caused by anything entering and is not a creature's trigger, so no doubler adds a trigger to it; the blinked creature's enter effect is doubled as usual. It stacks with Teleportation Circle (see The end-step engine below).

**The One Ring** gives protection from everything for a turn when cast, then a card engine that costs life every upkeep (see Mana, life costs). **Lotho, Corrupt Shirriff** ({W}{B}, a legendary 2/1): *Whenever a player casts their second spell each turn, you lose 1 life and create a Treasure token.* He is a legendary creature, so he gives Yoshimaru a counter when he enters after him, and every blink of Lotho is a legendary permanent entering again. He has no enter effect of his own, so he does nothing for blink value: he is a two-drop that grows Yoshimaru and makes mana. The trigger is for any player's second spell each turn, opponents included, so at a four-player table it fires often, and the life loss is always ours and cannot be declined. With Delney out it fires twice (Lotho has power 2 and the trigger is a triggered ability): 2 life and 2 Treasures for each second spell. A Treasure is one mana of any colour (ramp and fixing, including for a second spell of our own) and an artifact token entering under our control, so Haliya's "Whenever Haliya or another creature or artifact you control enters, you gain 1 life" fires for it, and Gandalf (an artifact entering) and Elesh Norn (a permanent entering) each add one more Haliya trigger: 1 life per Treasure, 2 with one of them, 3 with both. That gain is a life gain like any other for Vito, Heliod and Archangel of Thune. A Treasure is also an artifact for Puresteel Paladin's metalcraft. Elas il-Kor triggers only on creatures, so a Treasure does nothing for her, and a Treasure is not legendary, so it gives Yoshimaru nothing. Gandalf and Elesh Norn do not double Lotho himself, since his trigger is caused by a spell being cast and not by anything entering. **The cost is real:** this deck already pays life (Tymna, fetch and shock lands, the tutors, The One Ring), and Lotho adds a loss that the whole table sets off, so keep an eye on life with him out. With Haliya out each Treasure also gains 1 life; without her the loss comes out of the same life total as everything else.

#### Yoshimaru's counters

Be honest about this part: he is a bonus. All numbers are estimates and the deck has not been simulated.

- **Inputs.** The library is 98 cards (the commanders are in the command zone) with 21 legendary permanents. By turn 6 on the play we have seen 12 cards (7 + 5 draws; 13 on the draw). Tymna's draws and the tutors are ignored.
- **Legends seen.** The chance of seeing at least 2, 3, 4 and 5 of the 21 in 12 cards is 78.2%, 49.8%, 23.4% and 7.9% (hypergeometric, exact), a mean of 2.57.
- **Legends played.** Not every legend seen is cast by turn 6 (cost, colours, a land drop used on something else). Assuming 75% are played gives 1.93 counters on average, and the chance of at least 2, 3 and 4 played becomes 60.8%, 29.8% and 10.3%. Playing 50% gives 1.29 and 100% gives 2.57.
- **Tymna** gives one counter only when Yoshimaru is already out. Assuming that is true in 90% of games: +0.9.
- **Blinks.** The 7 blink pieces give a mean of 0.86 in 12 cards. Assuming half are cast with a legend on the battlefield by turn 6, and each makes 1.5 legendary re-entries on average (Ephemerate rebounds, Eerie Interlude takes several, Teleportation Circle repeats): +0.64.
- **Total:** about 1.93 + 0.9 + 0.64 = 3.5 counters, a range of about 2.6 to 4.8 counters (low: half of the legends seen are played and half of the blink pieces are cast, one re-entry each, +0.43; high: all are played and all blink pieces are cast, 1.5 re-entries each, +1.29). Yoshimaru is a 1/1 base, so about a 4/4 to 5/5 by turn 6.
- **Doublers.** The chance of at least one of Delney, Gandalf and Elesh Norn in 12 cards is 32.7%; each adds one counter per legend entering after it lands (Delney only while Yoshimaru's power is 2 or less).
- **Reading.** A big ground body with no evasion that the table will remove if it can. Commander damage (21) from a 4/4 or 5/5 needs four or five hits and is not a plan. The deck does not win by his size.

#### Doubling triggers: Gandalf, Elesh Norn and Delney

Card texts. **Gandalf the White** ({3}{W}{W}, 4/5): *Flash. You may cast legendary spells and artifact spells as though they had flash. If a legendary permanent or an artifact entering or leaving the battlefield causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time.* **Elesh Norn, Mother of Machines** ({4}{W}, 4/7): *Vigilance. If a permanent entering causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time. Permanents entering don't cause abilities of permanents your opponents control to trigger.* **Delney, Streetwise Lookout** ({2}{W}, 2/2): *Creatures you control with power 2 or less can't be blocked by creatures with power 3 or greater. If a triggered ability of a creature you control with power 2 or less triggers, that ability triggers an additional time.*

Each doubler adds one extra trigger; they do not multiply each other (the usual ruling for effects of this kind, a rules point and not card text: two such effects give three triggers).

**Yoshimaru's counters** (his trigger is caused by a legendary permanent entering):

| Doublers on the battlefield | Counters per legendary permanent that enters |
|---|---|
| None | 1 |
| Gandalf only | 2 (a legendary permanent is always legendary) |
| Elesh Norn only | 2 (a permanent entering) |
| Delney only | 2 while Yoshimaru's power is 2 or less, otherwise 1 |
| Gandalf and Elesh Norn | 3 |
| All three, Yoshimaru with 0 or 1 counter | 4 |
| All three, Yoshimaru with 2 or more counters | 3 |

Yoshimaru is a 1/1, so 0 counters is power 1 and 1 counter is power 2. At 2 counters his power is 3 and Delney stops doubling him; Delney's evasion for him stops too. A blinked legend is a legend entering again, so the same table applies each time.

**Tymna with Delney.** Tymna is a 2/2, so her trigger fires twice at each postcombat main phase. Each copy asks for its own payment, so with X opponents hit we pay X life and draw X cards, twice: 2X cards for 2X life. It works only while Tymna's power is 2 or less when the trigger happens. What turns it off:

- A counter on Tymna: Archangel of Thune, whose trigger puts a counter on every creature each time we gain life (Tymna's own lifelink hit gains life, so with Thune out she grows past power 2 before her postcombat trigger); Heliod, if we aim its counter at her; The Wandering Emperor's +1.
- Equipment that adds power: Sword of Fire and Ice, Sword of Feast and Famine, Sword of Hearth and Home (+2/+2 each), Buster Sword (+3/+2) and Sting (+1/+1). Swiftfoot Boots and Mithril Coat add no power and leave her at 2.
- Removal on Delney, or Delney blinked out or phased out when the trigger would happen.

Gandalf and Elesh Norn do not double Tymna or Lotho: their triggers are caused by a phase beginning (Tymna) and by a spell being cast (Lotho), not by anything entering.

**Delney and Archangel of Thune mostly cancel each other.** Each life gain puts a counter on every creature, so a power-2 creature goes past the limit on the first gain and a power-1 creature on the second. With Thune out, one Elas or Haliya gain before combat turns the doubling off for the whole team that turn. Treat Thune as a finisher (Line C) or a Line D payoff, not as something to cast on a turn where Tymna needs Delney. With Heliod aim the counter at Heliod himself (he is an enchantment we control) or at a large creature, not at Tymna or the small creatures.

**What else Delney doubles.** These creatures have power 2 or less and a triggered ability, so each trigger fires twice with Delney out. The power comes from the card data.

| Creature | Power | Doubled |
|---|---|---|
| Tymna | 2 | Two postcombat draws (above) |
| Yoshimaru | 1 | Counters, while power 2 or less (above) |
| Recruiter of the Guard | 1 | Two searches (the second finds a different card) |
| Cloud, Midgar Mercenary | 2 | Two searches on entering: two Equipment cards. His own text adds a further trigger only while he is equipped (see The Equipment package) |
| Stoneforge Mystic | 1 | Two Equipment searches on entering (its put ability is not a trigger) |
| Kor Outfitter | 2 | Two attach triggers on entering (two Equipment or two targets) |
| Puresteel Paladin | 2 | Two draws (each a "may") for each Equipment that enters |
| Ardenn, Intrepid Archaeologist | 2 | Two attach triggers at the beginning of combat |
| Sunpearl Kirin | 2 | Two bounces on entering (two different targets; each returned token draws a card) |
| Felidar Guardian | 1 | Two blinks (two targets) |
| Lotho, Corrupt Shirriff | 2 | 2 life lost and 2 Treasures for each second spell any player casts in a turn |
| Witch Enchanter // Witch-Blessed Meadow | 2 | Two destroys when cast as a creature (the second needs a second target) |
| Skyclave Apparition | 2 | Two exiles on entering (its leave trigger: see the open points below) |
| Orcish Bowmasters | 1 | Two pings and two Orc amasses on entering, and again for each extra opposing draw |
| Elas il-Kor | 2 | Two gains per other creature entering; two drains per death |
| Vito | 1 | Two drains per life gain |

Not counted: Giver of Runes, Skrelv, Delney and Walking Ballista have power 2 or less but no triggered ability, so Delney does nothing for them. The creatures with power 3 or more (Restoration Angel, Flickerwisp, Haliya, Solitude, Ranger-Captain of Eos, Archangel of Thune, Gandalf, Elesh Norn) are not doubled by Delney. Adeline's power is the number of creatures we control, so Delney doubles her attack trigger only while we control two or fewer.

**Elesh Norn covers the rest:** every enter trigger of ours fires an extra time, including Solitude, Ranger-Captain, Restoration Angel, Flickerwisp and Court of Ardenvale's monarchy trigger (which does nothing extra). With Elesh Norn and Delney both out, a power-2 creature with an enter trigger fires three times (Cloud: three Equipment searches). With Gandalf too, a legendary creature of power 2 or less fires four times (Cloud: four Equipment cards). Puresteel Paladin with all three doublers draws up to four cards for each Equipment that enters (Equipment is an artifact, so Gandalf counts, and Elesh Norn counts any permanent; each draw is a "may").

**Elas with Delney, Elesh Norn and Vito.** Elas has power 2 and Vito power 1. Each other creature entering gives Elas 1 + 1 (Elesh Norn) + 1 (Delney) = 3 gains of 1 life. Each gain triggers Vito twice (1 + 1 for Delney; Elesh Norn does not apply, since a life gain is not an entering), so 3 gains lose an opponent 6 life per creature that enters. If the creature is legendary or an artifact, Gandalf adds a fourth Elas trigger: 8 life. Four cards, so this is something to build toward and not a requirement.

**Gandalf also doubles:** Elas for a legendary or artifact creature entering or dying, Haliya for a legendary creature or an artifact entering (a Lotho Treasure or an Equipment is an artifact entering), and Puresteel Paladin for an Equipment. Gandalf's flash for legendary and artifact spells lets Heliod, Walking Ballista, Mox Amber, Tymna, the Equipment, Conjurer's Closet and the rocks come down at an opponent's end step. **Conjurer's Closet** entering is an artifact entering: Haliya's trigger fires for it (1 life), and Gandalf (an artifact entering) and Elesh Norn (a permanent entering) each add one trigger to it. It is not a legendary permanent, so it does not trigger Yoshimaru, and Delney does nothing for it (it is not a creature).

### The Equipment package

Seven Equipment, three cards that search for them (Cloud, Stoneforge Mystic, Steelshaper's Gift) and four that attach them for free or cheaply (Kor Outfitter, Ardenn, Forge Anew, Puresteel Paladin). The package does three jobs for a deck that is otherwise blink value: it protects a legend, it gives a second way to connect (which also draws Tymna's cards), and it is a tutor chain that finds the piece the board needs.

**The pieces** (texts from the card data):

| Equipment | Cost / equip | Equipped creature | On combat damage to a player, or on entering |
|---|---|---|---|
| Sword of Fire and Ice | {3} / {2} | +2/+2, protection from red and from blue | 2 damage to any target and draw a card |
| Sword of Feast and Famine | {3} / {2} | +2/+2, protection from black and from green | That player discards a card and we untap all lands |
| Sword of Hearth and Home | {3} / {2} | +2/+2, protection from green and from white | Exile up to one target creature we own, then search for a basic land; both go onto the battlefield |
| Buster Sword | {3} / {2} | +3/+2 | Draw a card, then we may cast a spell from hand with mana value at most the damage, free |
| Sting, the Glinting Dagger | {2} / {2} | +1/+1 and haste; untapped at the beginning of each combat; first strike when blocking or blocked by a Goblin or Orc | Legendary: a counter for Yoshimaru when it enters |
| Swiftfoot Boots | {2} / {1} | Hexproof and haste | Adds no power (like Mithril Coat); the equip cost is the cheapest |
| Mithril Coat | {3}, flash / {3} | Indestructible | When it enters, attach it to target legendary creature we control. Legendary: a counter for Yoshimaru |

**Who carries what.**

- **Not on Tymna, with one exception.** Any added power takes Tymna above power 2 and turns off Delney's doubling and evasion for her (see Doubling triggers). Swiftfoot Boots is the carrier piece for her: haste lets her attack and draw the turn she is cast, and hexproof protects her. Mithril Coat also leaves her at 2 and makes her indestructible. Tymna draws when *any* creature of ours connects, so the Swords belong on another creature.
- **Fliers or a creature that connects.** The fliers are Restoration Angel, Flickerwisp, Archangel of Thune and Sunpearl Kirin (the chance of at least one of the four in the first 12 cards is 41.2%). Put Swords, Buster Sword and Sting on them or on any creature the table cannot block. Sting (+1/+1) on a power-1 creature (Recruiter of the Guard, Stoneforge Mystic) leaves it at power 2, so Delney's evasion still applies.
- **Sword of Hearth and Home** gives protection from white as well as green, so the carrier cannot be targeted by white sources, ours included: Ephemerate, Restoration Angel, Felidar Guardian, Flickerwisp, Teleportation Circle, Eerie Interlude, Giver of Runes, Heliod's lifelink, Skrelv's ability, and the attach triggers of Kor Outfitter and Ardenn (these are rules readings of colour and protection, not card text). Equip abilities, Puresteel Paladin's equip {0}, Forge Anew and Mithril Coat's attach trigger still work, because the Equipment is colourless. It also stops white creatures from blocking the carrier. Never put it on Walking Ballista: Heliod's lifelink could no longer target it, which breaks Line B. Its hit trigger exiles up to one creature we own and fetches a basic land, and both come back onto the battlefield (the land untapped): a free blink that re-runs an enter effect, and ramp. Best carriers: Cloud (his doubling gives two blinks and two basics per hit) or Adeline.
- **Cloud, Midgar Mercenary** is a normal creature in the 98, not a commander. His enter trigger searches for an Equipment, and Delney doubles it (power 2) for two. His own doubling is a static ability that works while he is equipped: a triggered ability of Cloud or of an Equipment attached to him triggers an additional time. A Sword or Buster Sword attached to him is itself what makes him equipped, so its combat-damage trigger fires twice on his hits (Sword of Hearth and Home: two blinks and two basics). Mithril Coat can attach to him when it enters (he is legendary) and makes him indestructible, but it has no trigger for the doubling to double. A Sword takes him above power 2, which turns Delney off for him but not his own doubling.

**Blink the Equipment, not the carrier.** A blinked creature loses its Equipment, and a blinked Equipment comes back unattached unless it attaches itself on entering (Mithril Coat does; Kor Outfitter attaches only when Kor Outfitter itself enters). Teleportation Circle (an artifact or creature), Felidar Guardian and Flickerwisp can blink an Equipment; Conjurer's Closet, Restoration Angel, Ephemerate and Eerie Interlude take creatures only. Each blink of a legendary Equipment (Mithril Coat, Sting) gives Yoshimaru a counter, and Puresteel Paladin draws a card for any Equipment entering (a cast, Stoneforge's put, Forge Anew, a blink), but not when an Equipment is merely moved (equip, Ardenn).

**The free-attach engine.**

- **Kor Outfitter** attaches an Equipment to a creature when it enters. A blink of Kor Outfitter is another attach.
- **Ardenn, Intrepid Archaeologist** attaches any number of Equipment to a target permanent or player at the beginning of combat on our turn, before attackers are declared, for no mana.
- **Forge Anew** lets us equip at instant speed during our turn and pay {0} for the first equip each turn (Swiftfoot Boots onto Tymna in response to removal). Its enter trigger returns an Equipment card from the graveyard to the battlefield, but the deck rarely has one there (Mithril Coat is indestructible, the tutors put Equipment in hand, and Eiganjo's channel is the only discard outlet), so the value is the equip rule. Felidar Guardian and Flickerwisp re-use it; Teleportation Circle cannot target an enchantment; Enlightened Tutor finds it and Steelshaper's Gift does not.
- **Puresteel Paladin** gives Equipment equip {0} while we control three or more artifacts. The list has 18 artifacts (6 rocks, 7 Equipment, Wayfarer's Bauble, Conjurer's Closet, The One Ring, Walking Ballista and Skrelv), and Lotho's Treasures count; the chance of at least 3 of the 18 in the first 12 cards is 38.4%, so metalcraft is a mid-game state.

**The tutor chain.** Steelshaper's Gift ({W}, a sorcery) finds only an Equipment. Cloud and Stoneforge Mystic find one on entering (Delney doubles both), and Stoneforge's {1}{W}, {T} puts an Equipment from hand onto the battlefield at instant speed, which counts as an enter (Puresteel draws, Sting or Mithril Coat gives Yoshimaru a counter). Recruiter of the Guard finds the enablers: Cloud, Stoneforge Mystic, Kor Outfitter, Puresteel Paladin and Ardenn (toughness 2 or less); it can also find Skrelv, which is not an enabler, and Ranger-Captain of Eos finds Skrelv (mana value 1). Enlightened Tutor finds any Equipment, Forge Anew, Court of Ardenvale and Skrelv (an artifact). The chance that the first 10 cards (turn 4 on the play) hold an Equipment or a card that finds one (7 Equipment plus Cloud, Stoneforge Mystic and Steelshaper's Gift) is 67.8%, and 80.3% when Enlightened Tutor, Demonic Tutor, Vampiric Tutor and Imperial Seal count too.

**Mithril Coat and Skrelv as protection.**

- **Mithril Coat** has flash. In response to a destroy effect, or to damage that would kill a legend, cast it: its trigger attaches it to the legend and resolves before the removal (a rules point, not card text), and indestructible saves the legend. It does not stop exile, -X/-X, bounce or sacrifice.
- **Skrelv, Defector Mite** ({W}, a legendary artifact creature 1/1 with toxic 1 that can't block; {W/P}, {T}: another creature we control gains toxic 1 and hexproof from a chosen colour, and can't be blocked by creatures of that colour, until end of turn; {W/P} is {W} or 2 life). The protection is hexproof from one colour, chosen when the ability resolves: it stops targeted removal of that colour (exile, bounce, -X/-X as well as destroy; a multicolour spell that includes the colour too), but not a sweeper, an effect that does not target, or a removal spell of another colour. It needs {T}, so it does nothing the turn it comes down, and it can protect only another creature. With nothing to protect, use it before combat so a Sword carrier or a creature that should connect for Tymna can't be blocked by creatures of one colour. It is one of the 21 legendary permanents (a counter for Yoshimaru) and an artifact, but it moves no Equipment and gives no counter itself.
- **Wrath of God can be one-sided.** Cast it, hold priority, and cast Flawless Maneuver (free with a commander) in response; Mithril Coat and Plaza of Heroes keep a legend alive; Equipment stays on the battlefield; Court of Ardenvale rebuilds. It does not touch opposing indestructible creatures.

**What it costs.**

- **Mana.** A Sword is {3} to cast and {2} to equip, so the first hit takes 5 mana before Kor Outfitter, Ardenn, Forge Anew or Puresteel Paladin help; equip costs and Court of Ardenvale use the mana the 35 lands and 6 rocks provide.
- **A Delney conflict for Tymna.** Every Equipment except Swiftfoot Boots and Mithril Coat turns off her doubling, so the Swords go on other creatures.
- **A targeting limit.** Sword of Hearth and Home's protection from white blocks most of the deck's own blink and protection spells on its carrier.
- **A weaker anti-combo suite.** Nothing in the package stops an opponent's combo: the stops are Silence, Ranger-Captain of Eos and instant removal, and the win turn has no board shield beyond Silence cast that turn.

### How the deck wins

Four lines can end the game, and the grind sets them up. **The numbers are mana counts from the list, not win rates.** A trigger that is "may" can be declined; Vito's and Exquisite Blood's triggers are not.

#### Line A: Vito, Thorn of the Dusk Rose + Exquisite Blood

Two cards, 8 mana ({2}{B} and {4}{B}), over one turn or two. Vito: *Whenever you gain life, target opponent loses that much life.* Exquisite Blood: *Whenever an opponent loses life, you gain that much life.*

1. Cast **Vito**. It works alone: every life gain is a drain.
2. Cast **Exquisite Blood** when a start is ready, or when no opponent can answer it before our next turn.
3. **Start the loop.** The cast of the second piece does not start it. We need one life gain, or one point of life lost by an opponent, after both are on the battlefield: Elas's gain for a creature entering, Haliya's gain for a creature or artifact entering, The Wandering Emperor's -2 (it gains 2), a lifelink hit from Tymna, Solitude or Archangel of Thune, any combat damage to an opponent, Sword of Fire and Ice's 2 damage to an opponent, an Orcish Bowmasters or Walking Ballista ping at an opponent, or an opponent paying life for a fetch land or a shock land. A cheap start with Elas out is a one-drop creature such as Giver of Runes or Skrelv ({W}): Elas gains 1 life when it enters.
4. **The loop.** An opponent loses N, Blood gains us N, Vito makes an opponent lose N, Blood gains us N again. Both triggers are mandatory, and each Vito trigger targets one opponent; when that opponent dies, the next trigger targets another. The loop ends only when every opponent is dead.

**What stops it:** a counterspell on the second piece; removal on Vito or on Blood (a trigger already on the stack still resolves, so one more cycle can happen before the loop stops); an effect that stops life changes; an opponent who can't lose life or can't be targeted.

**Finding it:** Enlightened Tutor finds Exquisite Blood; Demonic Tutor, Vampiric Tutor and Imperial Seal find either piece. Nothing else finds Vito: he is a creature that is neither an artifact nor an enchantment, and his toughness of 3 is out of Recruiter of the Guard's range.

#### Line B: Heliod, Sun-Crowned + Walking Ballista

Two cards, 9 mana: Heliod 3, Walking Ballista with X = 2 for 4, and {1}{W} for lifelink. Heliod: *Whenever you gain life, put a +1/+1 counter on target creature or enchantment you control. {1}{W}: Another target creature gains lifelink until end of turn.* Walking Ballista: *This creature enters with X +1/+1 counters on it. {4}: Put a +1/+1 counter on this creature. Remove a +1/+1 counter from this creature: It deals 1 damage to any target.*

1. Heliod and Ballista (X = 2) on the battlefield.
2. {1}{W}: Ballista gains lifelink until end of turn.
3. Remove a counter (Ballista is a 1/1): 1 damage to an opponent. Lifelink: we gain 1. Heliod's trigger: a +1/+1 counter on Ballista (back to 2 counters).
4. Repeat. Each cycle costs no mana and takes 1 life from an opponent (with Vito out each gain is also a drain). The table's life totals are the only limit.

**Why X = 2:** at X = 1, removing the counter makes Ballista a 0/0 that dies, so Heliod's counter has no Ballista to land on and the loop ends after one cycle.

**What stops it:** removal on Ballista before it has lifelink (answer by pinging with every counter in response); removal on Ballista in response to Heliod's trigger (the counter has no target); a life-gain prevention effect; a counterspell. Heliod himself is indestructible, so only exile or bounce stops him. Do not put Sword of Hearth and Home on Ballista: its protection from white stops Heliod's lifelink from targeting it.

**Finding it:** Enlightened Tutor finds either (both are artifact or enchantment cards); Ranger-Captain of Eos finds Ballista (mana value 0 in the library); Demonic Tutor, Vampiric Tutor and Imperial Seal find either.

#### Line C: Archangel of Thune + Walking Ballista + a lifelink source

Three pieces. Archangel of Thune: *Whenever you gain life, put a +1/+1 counter on each creature you control.* Lifelink on Ballista can come from Heliod ({1}{W}) or Vito's {3}{B}{B}.

1. Thune and Ballista (X = 2) on the battlefield, and Heliod's lifelink (or Vito's activation) so that Ballista has lifelink on our turn.
2. Remove a counter: 1 damage with lifelink, we gain 1, Thune puts a counter on each of our creatures (Ballista is back to 2).
3. Repeat: the same loop as Line B with Thune in Heliod's place.

Costs 5 + 4 + 3 = 12 mana with Heliod, plus {1}{W} for the lifelink: 14 over two turns; with Vito it is 5 + 4 + 3 + 5 = 17. Both lifelink sources are cards that already make Line B or Line A work, so this is close to those lines with Thune as the counter payoff: it is the way to run the Ballista loop when Heliod is not there and Vito is, and it grows the whole team (watch Tymna: her power goes past 2, which turns Delney's doubling off).

#### Line D: Restoration Angel + Felidar Guardian

Two cards that blink each other. Restoration Angel: *When this creature enters, you may exile target non-Angel creature you control, then return that card to the battlefield under your control.* Felidar Guardian: *When this creature enters, you may exile another target permanent you control, then return that card to the battlefield under its owner's control.*

1. Restoration Angel on the battlefield, then cast Felidar Guardian (or the reverse).
2. Felidar's trigger exiles Restoration Angel and returns it. Restoration Angel's trigger exiles Felidar (a Cat Beast, not an Angel) and returns it. Felidar's trigger exiles Restoration Angel again. Each trip makes both creatures enter, and both triggers are "may", so we choose when to stop.
3. **The payoff is separate.** Without one the loop does nothing. With a payoff on the battlefield:
   - **Elas il-Kor or Haliya alone** gives infinite life.
   - **Add Vito** and the life becomes a kill.
   - **Add Archangel of Thune** instead and every creature gets infinite counters, and Restoration Angel and Thune fly.
4. **Flash timing.** Restoration Angel has flash. With Felidar already on the battlefield, cast Restoration Angel at an opponent's end step and run the loop there, with the payoff already out.
5. **Ending the loop.** Aim the last Felidar trigger at Recruiter of the Guard or Cloud (a search), Solitude or Skyclave Apparition (removal), or decline the "may".

Cost: 8 mana for the two cards, then the payoff already on the battlefield. The worst case is 13 mana and four cards (Restoration Angel 4, Felidar 4, Elas 2, Vito 3); it is a late finisher or a finish when two of the pieces are already on the table. **What stops it:** removal on either creature in response to a trigger (the trigger has no target), an effect that stops enter triggers, or removal on the payoff.

#### The grind

Expect most games to be won here. Keep the engine running: one enter effect per turn, a blink for the best one, an Equipment on a creature that connects, Tymna connecting with Delney out, a drain on every creature that enters, and the monarchy from Court of Ardenvale. Solitude and Skyclave Apparition clear blockers; Restoration Angel, Flickerwisp, Sunpearl Kirin and Archangel of Thune fly; Delney makes every creature with power 2 or less (Tymna, Elas, Vito, Adeline's Human tokens) unblockable by creatures with power 3 or more. Three opponents at 40 is a lot of life: the grind's job is to land Vito, Elas and Haliya, keep both commanders alive, and set up a line. Yoshimaru's size is a side effect.

**Court of Ardenvale** ({2}{W}{W}) makes us the monarch when it enters, and at each of our upkeeps it returns a permanent card with mana value 3 or less from the graveyard to hand, or to the battlefield while we are the monarch. Good targets: Stoneforge Mystic, Recruiter of the Guard, Ranger-Captain of Eos, Skyclave Apparition, Vito, Cloud, Skrelv and any Equipment. Never return Walking Ballista to the battlefield: it enters with X = 0 and dies. An Equipment returned this way is a normal enter (Puresteel draws; a legendary one is a counter for Yoshimaru). Blinking Court (Felidar Guardian, Flickerwisp) makes us the monarch again, and an enchantment survives a creature sweeper. **The exposure:** the monarch draws an extra card at the beginning of their end step, and a creature that deals combat damage to the monarch makes its controller the monarch (both rules points, not card text). At a four-player table all three opponents attack the monarch, and the deck has few blockers, so an opponent who connects takes the card each turn. Our own Stroke of Midnight gives its target's controller a 1/1 Human, which can block or, if it connects, take the monarchy. Keep blockers back (Felidar Guardian, Ranger-Captain, Solitude) before casting Court, or cast it when the table is tapped out.

**Enter effects and blink work like this:**

| Piece | Target | What to know |
|---|---|---|
| Ephemerate ({W}, instant) | A creature we control | Rebound: cast from hand, it comes back at the next upkeep for free: two blinks for one card. Also a response to removal |
| Restoration Angel ({3}{W}, flash) | A non-Angel creature we control | The enter effect itself, at instant speed. The creature returns without its Equipment |
| Sunpearl Kirin ({1}{W}, flash) | Up to one other nonland permanent we control | Not a blink: the permanent goes to hand and must be recast, so it costs mana. Use it on a creature with a strong enter effect that we can pay for again, on a creature about to be removed (at instant speed), on a legendary Equipment (a recast is a counter and a Puresteel draw), or on a token (draw a card). Its own enter effect is doubled by Delney or Elesh Norn |
| Flickerwisp | Any other permanent | Mandatory. Ours is a delayed enter effect: an Equipment returns unattached (Puresteel draws), Court of Ardenvale makes us the monarch again. An opposing permanent is gone until the end step, and an opposing token is gone for good (a rules point). Elesh Norn stops its owner's enter trigger when it returns |
| Felidar Guardian | Any other permanent we control | Works on Equipment (Puresteel draws), Court of Ardenvale, planeswalkers (a blinked planeswalker is a new object with its starting loyalty and can use an ability again, a rules point), rocks |
| Eerie Interlude ({2}{W}, instant) | Any number of our creatures | They return at the beginning of the next end step: cast it before the end step you want them back at. It is a mass enter effect and the sweeper answer. Equipment is left unattached |
| Teleportation Circle ({3}{W}) | An artifact or creature | At each of our end steps. Solitude, Recruiter, Cloud, Stoneforge Mystic, a legend for Yoshimaru, or an Equipment (Mithril Coat attaches itself again and is a counter) |
| Conjurer's Closet ({5}, artifact) | One target creature we control (creatures only) | At the beginning of each of our end steps, a "may". A late engine: it needs a creature on the battlefield. Stacks with Teleportation Circle |
| Sword of Hearth and Home | Up to one creature we own, not its carrier (a blinked carrier comes back without its Equipment) | On a combat hit by its carrier: the creature returns at once with its enter effect, and a basic land comes with it |

**The end-step engine.** Two permanents blink at the beginning of our own end step: Teleportation Circle (up to one target artifact or creature we control) and Conjurer's Closet (one target creature we control, a "may"). They trigger at the same time, each is a separate trigger, and we choose the order they go on the stack (a rules point, not card text). They stack by blinking two different permanents in one end step. Aim each trigger at a different target: a creature that an earlier trigger has already returned is a new object, and a later trigger aimed at the old one does nothing (a rules point, not card text). The Closet takes only creatures; the Circle also reaches artifacts.

**What to blink at the end step:** Recruiter of the Guard or Ranger-Captain of Eos (a search every turn), Cloud or Stoneforge Mystic (an Equipment card to hand every turn), Kor Outfitter (a free attach), Solitude (an exile), Skyclave Apparition (see below), a legendary Equipment (Mithril Coat or Sting: a counter, a draw with Puresteel Paladin), and a creature that makes life or counters with the life layer out. The cases below are the exceptions. A creature whose own effect returns things "at the beginning of the next end step" behaves differently when it is blinked at our end step: that end step's beginning has already passed, so the return waits for the next turn's end step (a rules point, not card text). Flickerwisp blinked this way keeps the opposing permanent it exiled gone through that opponent's whole turn, which is a strong use.

**What not to blink:**

- **Walking Ballista.** It re-enters with X = 0 and dies (a rules point: with no cast, X is 0).
- **Skyclave Apparition**, unless the exiled card was cheap. Blinking it makes the exiled card's owner create an X/X Illusion token (X is that card's mana value) and gives us a new exile. The old card stays exiled.
- **Yoshimaru himself.** He re-enters with no counters. Blink him only to answer removal, and protect him with Giver of Runes, Mithril Coat or Plaza of Heroes first.
- **A creature that carries an Equipment.** It comes back bare (a rules point). Blink the Equipment instead, or let the carrier stay. Cloud blinked loses his Equipment and his doubling, though his enter trigger finds another Equipment.
- **The One Ring is a neutral case, not a mistake.** Blinking it gives no protection (the protection needs a cast), and it comes back with no burden counters (a rules point): that stops the life loss and also resets the card count, so it is a way to stop the life loss when it gets too high.
- **Solitude** is the best target for a blink: with the evoke cast (exile a white card, free), a blink in response to the sacrifice trigger leaves it on the battlefield as a new object that was not evoked, with a second exile. Ephemerate, Restoration Angel and Eerie Interlude all work (rules points, not card text).

**Haliya's warp.** For {W}, Haliya enters (a legend for Yoshimaru, and 1 life), is exiled at the beginning of the next end step, and can be cast from exile on a later turn (another legend entering). It is a cheap extra enter, and a cheap blink-like play on turn 1 or 2.

### Game plan

**Turn 1.** A white land that enters untapped, then Yoshimaru; Mox Amber right after him if drawn (counter and a free {W}). With Chrome Mox in hand, play it first (imprinting a white card) and spend its {W} and the land's on Yoshimaru and a one-drop (Giver of Runes, Skrelv). **Turn 2.** A two-drop that does something when it enters (Cloud or Stoneforge Mystic for an Equipment, Kor Outfitter, Puresteel Paladin; Orcish Bowmasters or Sunpearl Kirin at an opponent's end step), or Elas, Lotho ({W}{B}, a legendary two-drop that is a counter for Yoshimaru if he is already out), a rock, Swiftfoot Boots, Sting or Demonic Tutor. **Turn 3.** Tymna, or Delney first if both are in hand and a blinker is ready; or a Sword if the carrier is ready. **Turns 4 to 6.** An enter-effect creature a turn, with a blink to repeat the best one, and an Equipment on a creature that connects. Hold flash plays (Restoration Angel, Solitude, Bowmasters, The Wandering Emperor, Gandalf, Mithril Coat) for an opponent's end step or for a removal spell. **Late.** The 8 to 13 mana lines, and Elesh Norn, Gandalf and Conjurer's Closet as the late plays.

**Go now or hold?**

- **Go now** when a line is complete (Line A with a start in hand, Line B with 9 mana, Line D at an opponent's end step), or when the table has tapped out.
- **Hold** the second piece of Line A until a start is ready; every turn Vito or Blood sits alone is a turn for an answer to find it.
- **Hold** a blink spell for a removal spell or a sweeper when a creature that matters is on the table: the spell is worth more as an answer than as a second enter.
- **Hold** Eerie Interlude and Akroma's Will when the table plays sweepers, and keep Mithril Coat's mana up (or Skrelv untapped and not summoning sick) when a legend is the target.
- **Hold** Wrath of God until a legend can be protected (see Protecting the commanders).

### Protecting the commanders and the table

**Blink spells and protection on the commanders.**

| Card | Cost | Answers | Does not stop |
|---|---|---|---|
| Ephemerate | {W} | Removal aimed at one creature: exile it in response; it returns and rebounds | A sweeper that hits the other creatures |
| Restoration Angel | {3}{W}, flash | Removal on a non-Angel creature, at instant speed | The same |
| Sunpearl Kirin | {1}{W}, flash | Removal aimed at one nonland permanent: return it to hand in response (and recast it) | A sweeper that hits the other creatures; the recast costs mana |
| Eerie Interlude | {2}{W} | Any removal or sweeper that hits our creatures; they return at the next end step | Tokens (they are gone for good); a sweeper that also hits artifacts, enchantments or planeswalkers; Equipment is left unattached |
| Giver of Runes | {W} | A targeted removal spell on another creature (protection from the spell's colour, or from colorless). She cannot protect herself | Sweepers, which do not target |
| Mithril Coat | {3}, flash | A destroy effect or lethal damage on a legendary creature: cast it in response and it attaches (indestructible) | Exile, -X/-X, bounce, sacrifice; a creature that is not legendary |
| Skrelv, Defector Mite | {W}, then {W/P} and {T} | Targeted removal aimed at another creature: in response, it gains hexproof from the spell's colour until end of turn, so the spell is countered on resolution; it also makes that creature unblockable by that colour | Sweepers, combat damage, anything that does not target, a spell of another colour; nothing the turn it lands (it needs {T}) |
| Swiftfoot Boots | {1} to equip | Removal that targets the carrier on our own turn: hexproof, equipped in response with Forge Anew (equip is a sorcery otherwise) | Sweepers; anything on an opponent's turn (Forge Anew lets us equip at instant speed only during our turn); removal already on the stack without Forge Anew |
| A Sword, with Forge Anew | {2} to equip, or {0} for the first | Removal of the Sword's protected colours aimed at the carrier during our own turn: equip in response (Feast and Famine: black and green; Fire and Ice: red and blue; Hearth and Home: green and white) | Other colours, sweepers, and anything on an opponent's turn |
| Flawless Maneuver | {2}{W}, free with a commander | Creatures gain indestructible until end of turn: a destroy sweeper | -X/-X (indestructible does not save a creature whose toughness reaches 0, a rules point), exile |
| Akroma's Will | {3}{W} | Creatures we control gain indestructible and protection from each colour (and lifelink) until end of turn: a destroy sweeper, a damage sweeper, targeted removal. With a commander on the battlefield (Yoshimaru or Tymna) we may choose both modes, so they also gain flying, vigilance and double strike | -X/-X, exile and bounce sweepers (Farewell, Toxic Deluge); it covers creatures only, not our other permanents |
| Plaza of Heroes | {3}, {T}, exile | A legendary creature gains hexproof and indestructible until end of turn | Anything that is not a legendary creature |

**Akroma's Will, protection and finisher.** Cast in response to a destroy or damage sweeper or to removal, it gives our creatures indestructible and protection from each colour. With Yoshimaru or Tymna on the battlefield (one is enough), choose both modes and it is also a finisher: double strike doubles the combat triggers of Sword of Fire and Ice, Sword of Feast and Famine and Sword of Hearth and Home, and double strike with lifelink feeds the life-gain kills (Vito with Exquisite Blood, Heliod, Archangel of Thune). It is an instant, so cast it before combat damage and after blockers.

A commander in the command zone can be recast for its tax. Yoshimaru is cheap to recast but loses his counters.

**Sweepers, and what each answer does not stop.** A destroy sweeper (a Wrath of God type) is stopped by Flawless Maneuver, Mithril Coat on a legend, Plaza on a legend, Eerie Interlude and Akroma's Will. A -X/-X sweeper is stopped only by Eerie Interlude (an instant blink does not help: the creature returns before the sweeper resolves, so it still gets -X/-X). Exile and bounce effects are stopped by Eerie Interlude too. Nothing in the deck gives total protection any more, so Akroma's Will does nothing against exile or -X/-X. After a wipe the deck rebuilds with its noncreature permanents: The Wandering Emperor, Teleportation Circle, The One Ring, Heliod (not a creature unless our devotion to white is five or more), Exquisite Blood, Conjurer's Closet (it needs a creature to blink), every Equipment (they stay on the battlefield), Court of Ardenvale (it returns a permanent card with mana value 3 or less each upkeep and survives a creature sweeper), and Takenuma's channel (returns a creature or planeswalker from the graveyard to hand). Most draw engines are creatures, so a wipe also costs card flow, apart from The One Ring and Court.

**Our own Wrath of God can be one-sided.** Cast it, hold priority, and cast Flawless Maneuver (free with a commander) in response; Mithril Coat and Plaza of Heroes keep a single legend alive; Equipment stays on the battlefield; Court of Ardenvale rebuilds. It does not touch opposing indestructible creatures.

**Against combo.** The deck has no counterspell. Its stops are:

- **Silence** ({W}, instant): *Your opponents can't cast spells this turn.* Cast it in an opponent's upkeep and they cast nothing that turn, or cast it on our own turn once the win line is set up. It does not stop abilities (Walking Ballista, a planeswalker, a land) or spells already on the stack.
- **Ranger-Captain of Eos** ({1}{W}{W}, 3/3): *Sacrifice this creature: Your opponents can't cast noncreature spells this turn.* It works at instant speed on their turn, but it stops only noncreature spells (not creature spells, so a combo that runs on a creature spell is not stopped, and not abilities). Blink makes it reusable.
- **Instant removal:** Swords to Plowshares, Path to Exile, Deadly Rollick, Stroke of Midnight ({2}{W}: destroy target nonland permanent; its controller creates a 1/1 white Human creature token, so it also answers a noncreature combo piece), and the flash creatures Solitude, Orcish Bowmasters (an X/1 piece) and The Wandering Emperor (-2 on a tapped creature).

**This is a weakness.** The deck has no board shield for its own win turn: Silence cast that turn is the only protection against an opponent's instant-speed answer, and abilities (an opponent's equip, a planeswalker, a land) are not stopped. Flawless Maneuver and Akroma's Will do not stop an opponent's combo.

### Mulligans

The first mulligan is free (see two 7-card hands). After that it is a London mulligan: draw 7, bottom one card per mulligan after the free one.

| Hand | Keep when |
|---|---|
| 7 cards | 2 to 5 lands (2 lands only with a rock or Mox Amber; 5 lands only with two spells that do something); a white source that enters untapped by turn 1 or 2, and white and black reachable by turn 3 or 4 (a dual, Command Tower, Plaza of Heroes, a fetch, a Signet, or Chrome Mox with a white or black card to imprint); and a reason to go: an enter-effect creature, a blink piece with a target, an Equipment or something that finds one, a draw engine, a tutor or Delney. The commanders are real plays, so Yoshimaru on turn 1 and Tymna on turn 3 are a start |
| 6 or 5 cards | 2 to 4 lands, a white source, and a play before turn 4 |
| 4 cards | Keep |

- **Count colours.** Most of the list is white (52 white pips against 9 black); 13 cards need {W}{W}. Tymna needs {W}{B}; no card in the 98 costs {B}{B} (Vito's {3}{B}{B} activation does).
- **Payoffs are late cards.** Gandalf, Elesh Norn, Archangel of Thune, Solitude, Exquisite Blood and Conjurer's Closet cost 5; a hand of those and lands has no play before turn 5.
- **Bottom first:** the five-drops, then reactive cards, then lands past the fourth.

| # | Hand | Call | Why |
|---|---|---|---|
| 1 | Plains; Scrubland; Swamp; Sol Ring; Giver of Runes; Recruiter of the Guard; Demonic Tutor | Keep | Plains and Yoshimaru on turn 1. Turn 2: Scrubland, Sol Ring from the Plains, then Sol Ring's {C}{C} and Scrubland's {W} cast Recruiter (it finds Cloud or Stoneforge Mystic). Turn 3: Swamp, Tymna ({W} Plains, {B} Swamp, {1} Sol Ring) and Giver of Runes ({W} Scrubland). Demonic Tutor finds the rest |
| 2 | Arid Mesa; Godless Shrine; Plains; Elas il-Kor, Sadistic Pilgrim; Lotho, Corrupt Shirriff; Stoneforge Mystic; Ephemerate | Keep | Yoshimaru turn 1, Elas on turn 2 ({W} Plains, {B} Godless Shrine for 2 life), Tymna on turn 3 with Arid Mesa fetching Scrubland. Tymna entering after Yoshimaru is his counter and Elas gains 1 life. Lotho (a legend, so a counter), Stoneforge Mystic (an Equipment to hand) and Ephemerate follow as the fourth land and more mana arrive; Lotho costs us 1 life each time a player casts a second spell in a turn |
| 3 | Plains; Swamp; Isolated Chapel; Sol Ring; Vampiric Tutor; Enlightened Tutor; Giver of Runes | Keep | Three lands in both colours (Isolated Chapel enters untapped once Plains or Swamp is out), a rock, a one-drop and two instant tutors for a combo piece. Giver of Runes protects Tymna |
| 4 | Plains; Plains; Scrubland; Swamp; Marsh Flats; Ranger-Captain of Eos; Sunpearl Kirin | Keep (close) | Five lands, but both spells are real: Ranger-Captain on turn 3 ({W}{W} from the Plains) finds Giver of Runes, Skrelv or Walking Ballista, and Sunpearl Kirin comes down at an opponent's end step and returns Ranger-Captain to hand for another search, or saves it from removal |
| 5 | Plains; Command Tower; Caves of Koilos; Orzhov Signet; Gandalf the White; Exquisite Blood; Ephemerate | Keep (close) | The commanders are the early plays: Yoshimaru turn 1, Orzhov Signet turn 2, Tymna turn 3. Gandalf and Blood are late, but a fourth land or a tutor makes them castable; Ephemerate needs a target |
| 6 | Plains; Swamp; Swords to Plowshares; Path to Exile; Stroke of Midnight; Deadly Rollick; Wrath of God | Ship | Two lands and only reactive cards: nothing here wins or makes cards |
| 7 | Plains; Plains; Plains; Eiganjo, Seat of the Empire; Gandalf the White; Elesh Norn, Mother of Machines; Archangel of Thune | Ship | Three five-drops and no black for Tymna; no play before turn 5 except Yoshimaru |
| 8 | Plains; Plains; Swamp; Scrubland; Command Tower; Takenuma, Abandoned Mire; Elesh Norn, Mother of Machines | Ship | Six lands and one five-drop: a flood. The rule is five lands only with two spells that do something |
| 9 | Plains; Swamp; Mox Amber; Sol Ring; Giver of Runes; Cloud, Midgar Mercenary; Delney, Streetwise Lookout | Keep (close) | Two lands, but Mox Amber and Sol Ring make mana. Turn 1: Plains, Yoshimaru, Mox Amber (a counter), then its {W} casts Giver of Runes. Turn 2: Swamp and Tymna ({W} from Mox Amber, {B} Swamp, {1} Plains). Cloud ({W}{W}) is castable on turn 2 from the Plains and Mox Amber instead of Tymna (both want the same mana), so the hand has a choice of plays; without a third land by turn 4 it stalls, so it is close |
| 10 | Plains; Plains; Swamp; Scrubland; Cloud, Midgar Mercenary; Kor Outfitter; Swiftfoot Boots | Keep | Four lands in both colours. Yoshimaru turn 1; Cloud on turn 2 ({W}{W} from the Plains), fetching a Sword or Mithril Coat; turn 3 is Tymna or the Equipment, not both (they want the same three mana); Kor Outfitter attaches an Equipment that is already on the battlefield when it enters, so it comes after the Sword (turn 4 or 5); Swiftfoot Boots go on Tymna once she is out |

### What to tutor for

Yoshimaru and Tymna start in the command zone, so no tutor finds them. The tutors find a piece of a line (Vito, Exquisite Blood, Heliod, Walking Ballista, Restoration Angel, Felidar Guardian), an engine (Delney, Elas, Elesh Norn, Cloud), an Equipment, a stop for a combo (Silence, Ranger-Captain), or an answer to a stax piece. Read the board first (what is out, what the table can do, life totals, our mana), then choose the tutor whose speed and cost fit.

#### The tutors and tutor-like cards

| Card | Cost | The card goes | Finds | Speed | The catch |
|---|---|---|---|---|---|
| Demonic Tutor | {1}{B} | Hand | Any card | Sorcery | The cheapest way to put any card in hand |
| Vampiric Tutor | {B} | Top of library | Any card | Instant | We lose 2 life. Cast it at the end step before our turn to draw the card with all our mana |
| Enlightened Tutor | {W} | Top of library | An artifact or enchantment card: Sol Ring, Mana Vault, Mox Amber, Arcane Signet, Orzhov Signet, Chrome Mox, Wayfarer's Bauble, Conjurer's Closet, The One Ring, Walking Ballista, the seven Equipment (Mithril Coat, Sword of Fire and Ice, Sword of Feast and Famine, Sword of Hearth and Home, Buster Sword, Sting, Swiftfoot Boots), Court of Ardenvale, Exquisite Blood, Forge Anew, Teleportation Circle, Heliod (22 cards) | Instant | Cannot find a creature that is not an artifact or enchantment, an instant, a sorcery or a land |
| Imperial Seal | {B} | Top of library | Any card | Sorcery | We lose 2 life |
| Steelshaper's Gift | {W} | Hand | An Equipment card (7 cards) | Sorcery | Does not find Forge Anew or Court of Ardenvale (enchantments) |
| Cloud, Midgar Mercenary | {W}{W} | Hand | An Equipment card (7 cards) | On entering (blink reuses it; Delney doubles it) | A creature that must survive to be useful; he is also a legend for Yoshimaru |
| Stoneforge Mystic | {1}{W} | Hand | An Equipment card (7 cards); later puts one from hand onto the battlefield for {1}{W}, {T} | On entering (blink reuses it) | A "may" search |
| Ranger-Captain of Eos | {1}{W}{W} | Hand | A creature card with mana value 1 or less: Walking Ballista, Giver of Runes, Skrelv (3 cards) | On entering (blink reuses it) | A "may" search. Also the combo stop |
| Recruiter of the Guard | {2}{W} | Hand | A creature card with toughness 2 or less: Giver of Runes, Flickerwisp, Delney, Solitude, Skyclave Apparition, Orcish Bowmasters, Sunpearl Kirin, Walking Ballista (toughness 0), Elas il-Kor, Lotho, Witch Enchanter, Cloud, Stoneforge Mystic, Kor Outfitter, Puresteel Paladin, Ardenn, Skrelv (17 cards; the library holds only Witch Enchanter's creature face, a rules point) | On entering (blink reuses it) | A "may" search. It cannot find Vito (toughness 3), Restoration Angel or Felidar Guardian. Delney or Elesh Norn doubles it |

#### What to fetch for the board

| Situation | Tutor | Target | Why, and what to watch for |
|---|---|---|---|
| Turn 1 to 3, no white or no black source for the commanders | Demonic Tutor | Command Tower or an untapped dual (Scrubland); Orzhov Signet if no land fits | Plaza of Heroes only gives colour for a legendary spell until Yoshimaru or Tymna is out |
| Both commanders out, no engine | Demonic Tutor; Recruiter of the Guard | Delney | Delney doubles Tymna and lets most enter effects fire twice; cast it before Tymna's first postcombat main |
| Vito or Exquisite Blood missing (Line A) | Enlightened Tutor at the end step before; Demonic Tutor | Exquisite Blood (Enlightened); Vito (Demonic Tutor) | Cast the second piece only when a start is ready |
| Heliod or Walking Ballista missing (Line B) | Enlightened Tutor; Ranger-Captain | Either piece; Ballista (Ranger-Captain) | Ballista needs X = 2 and {1}{W} for lifelink: 9 mana in all |
| Restoration Angel or Felidar Guardian missing (Line D) | Demonic Tutor, Vampiric Tutor, Imperial Seal | The missing one | Recruiter can't find either (toughness 4); pick the one that can be flashed in at an end step (Restoration Angel) |
| An opponent combo is about to win | Demonic Tutor on our turn; Vampiric Tutor at their end step | Silence; instant removal for the piece (Stroke of Midnight reaches a noncreature piece) | Mana spent on a tutor is mana not spent on the stop. Ranger-Captain's sacrifice works on their turn if it is already out |
| A creature that decides the game | Recruiter of the Guard | Solitude or Skyclave Apparition (mana value 4 or less) | Solitude can be evoked for free |
| An artifact or enchantment stax piece | Recruiter of the Guard; Demonic Tutor | Witch Enchanter (artifact or enchantment only); Stroke of Midnight (any nonland permanent) | Witch Enchanter is a creature cast at sorcery speed; Stroke of Midnight is an instant but leaves its controller a 1/1 Human. Enlightened Tutor cannot find either |
| A sweeper is coming | Demonic Tutor, Vampiric Tutor; Enlightened Tutor or Steelshaper's Gift for Mithril Coat | Eerie Interlude, Akroma's Will, Flawless Maneuver or Mithril Coat | Match the answer to the sweeper (see Protecting the commanders) |
| Equipment in hand, nothing that connects | Recruiter of the Guard; Demonic Tutor | A flier (Flickerwisp, Restoration Angel) or Cloud | Swords on Tymna turn off Delney; Swiftfoot Boots and Mithril Coat are the exceptions |
| Equipment on the battlefield, equip cost too high | Recruiter of the Guard; Enlightened Tutor | Puresteel Paladin or Kor Outfitter; Forge Anew | Puresteel needs three artifacts; Ardenn also attaches for free |
| A legend needs a shield | Steelshaper's Gift; Enlightened Tutor | Mithril Coat | Flash, so it can wait for the removal spell |
| Mana or fixing | Enlightened Tutor | Sol Ring, Mana Vault or a rock that fits the hand | Orzhov Signet fixes both colours; Chrome Mox gives one colour of the card imprinted |
| Late, 7 or more mana | Demonic Tutor; Vampiric Tutor at the end step before | Gandalf, Elesh Norn or the missing line piece | Count mana: Line D from scratch costs 13 |

**Tutor mistakes:** asking for a commander; using Enlightened Tutor for a creature that is not an artifact or enchantment (Vito, Delney); asking Steelshaper's Gift for Forge Anew; casting a top-of-library tutor on our own turn when we cannot spend the mana on the card (cast it at the end step before); and casting Vampiric Tutor or Imperial Seal at a life total the table can punish.

### Rules points not verified, and texts not read in full

The deck has not been simulated. These points are marked so that nobody relies on them:

- **UNVERIFIED: Delney and Skyclave Apparition's leave trigger.** Skyclave has power 2, so Delney should double its enter trigger (two exiles), which follows from the text. Whether its "leaves the battlefield" trigger also doubles, and what the second trigger does with two exiled cards, is not checked. Do not rely on it: do not blink or sacrifice Skyclave under Delney unless the exiled card was cheap.
- **UNVERIFIED: a doubler doubling the trigger for its own entry.** Delney, Gandalf and Elesh Norn enter the battlefield and are on it when the entering is checked, so they should double Yoshimaru's trigger for their own arrival (as the same kind of effect does for a permanent entering beside it). This is a rules reading, not checked against a ruling. The doubling table assumes it only where the text says so (a doubler already on the battlefield).
- **UNVERIFIED: the end-step triggers of Conjurer's Closet and Teleportation Circle.** The texts are read in full from the data (above). Two points are rules readings and not card text: that we choose the order of our own simultaneous triggers, and that a trigger aimed at a creature already returned by an earlier trigger has an illegal target (the creature is a new object). Do not rely on them without a check.
- **UNVERIFIED: Lotho and the Treasure interactions.** The texts are read from the card data: Lotho's trigger fires for any player's second spell each turn and is mandatory ("you lose 1 life and create a Treasure token"); Delney should double it (power 2, a triggered ability); a Treasure token is an artifact entering under our control, so Haliya's trigger fires for it and Gandalf and Elesh Norn each add a trigger. These are readings of the text, not checked against rulings or a simulation.
- **UNVERIFIED: Equipment and attachment.** That a blinked creature returns without its Equipment and a blinked Equipment returns unattached; that Mithril Coat's trigger attaches it again on every entering (a blink, Forge Anew, Court of Ardenvale, Stoneforge Mystic's put); that Stoneforge Mystic's put, Forge Anew and Court of Ardenvale each count as an Equipment entering (Puresteel Paladin draws, Sting and Mithril Coat give Yoshimaru a counter) while equip and Ardenn only move it; and that Mithril Coat cast in response to removal attaches before the removal resolves. All are rules readings, not card text.
- **UNVERIFIED: Cloud's doubling.** The text is read from the data (as long as Cloud is equipped, a triggered ability of Cloud or of an Equipment attached to it triggers an additional time). That it adds one trigger on top of Delney, Gandalf or Elesh Norn (three triggers with two effects, as in the doubling rule above) and that his own enter trigger is not doubled by it (he is not yet equipped when he enters) are readings, not checked.
- **UNVERIFIED: protection from white.** Sword of Hearth and Home's protection should stop every white source from targeting the carrier (Ephemerate, Restoration Angel, Flickerwisp, Kor Outfitter's and Ardenn's triggers, Skrelv's ability, Heliod's lifelink) while colourless Equipment abilities, Mithril Coat's trigger and Forge Anew's equip rule still work. This is a rules reading of colour and protection, not checked against a ruling.
- **UNVERIFIED: the monarch.** Court of Ardenvale's text is read from the data (we become the monarch when it enters; at our upkeep it returns a permanent card with mana value 3 or less to hand, or to the battlefield while we are the monarch). The extra card at the end step and the transfer of the monarchy on combat damage are rules, not in the card data.
- **Forge Anew.** Its text is read from the data (equip at instant speed during our turn; {0} for the first equip each turn). How it combines with Puresteel Paladin's equip {0} is not checked and not relied on.
- **Card texts.** Every card text above is read from the card data in `data/scryfall_cache.json`; none was cut off in the cards the guide uses.
- **Estimates:** the Yoshimaru totals, the 75% and 90% assumptions, the half of blink cards cast and the 4 to 5 mana through turn 5 (including the mean of about 5.3 mana on turn 6 on the play for 35 lands and Wayfarer's Bauble, from an outside goldfish model and not the simulator) are judgement calls with no simulation of the real list behind them.

## Mana

The list is exactly 100 cards.

35 lands (13 nonbasic, one modal land, 14 Plains, 7 Swamp), 6 rocks (Sol Ring, Mana Vault, Mox Amber, Arcane Signet, Orzhov Signet and Chrome Mox) and Wayfarer's Bauble. **Chrome Mox** ({0}, a Game Changer): when it enters we may exile a nonland, nonartifact card from hand, and it taps for one mana of a colour of that card: it can make white or black but only as the imprinted card allows, and it costs a card, not life. **Wayfarer's Bauble** ({1}) is not a rock: {2}, {T}, sacrifice it to put a basic land from the library onto the battlefield tapped, so it is three mana in all for a land (it can fetch the Swamp or Plains the hand is missing) and an artifact until then. **Plaza of Heroes** taps for {C}, for one mana of any colour that can be spent only on a legendary spell (it casts Yoshimaru on turn 1), and for one mana of any colour among legendary permanents we control (white once Yoshimaru is out, black once Tymna is out). **Urborg, Tomb of Yawgmoth** makes every land a Swamp, so every land taps for {B}; it is symmetrical, so opposing lands tap for {B} too. **Command Tower** makes either colour. **Witch Enchanter // Witch-Blessed Meadow** is a land that enters tapped unless we pay 3 life and taps for {W}, or a 2/2 creature for {3}{W} later. **Marsh Flats** finds a Plains or Swamp card (Plains, Swamp, Scrubland or Godless Shrine); **Arid Mesa** finds a Mountain or Plains card (Plains, Scrubland or Godless Shrine).

| Measure | Count | Note |
| --- | --- | --- |
| White lands | 26 | 14 Plains, Witch Enchanter, Eiganjo, Minas Tirith, Scrubland, Godless Shrine, Caves of Koilos, Isolated Chapel, Vault of Champions, Command Tower, Marsh Flats, Arid Mesa, and Plaza of Heroes |
| Black lands | 18 (all 35 with Urborg out) | 7 Swamp, Takenuma, Urborg, Scrubland, Godless Shrine, Caves of Koilos, Isolated Chapel, Vault of Champions, Command Tower, Marsh Flats, Arid Mesa, and Plaza of Heroes. Arid Mesa and Marsh Flats count for both because they fetch Scrubland or Godless Shrine |
| Both colours | 9 | Scrubland, Godless Shrine, Caves of Koilos, Isolated Chapel, Vault of Champions, Command Tower, Marsh Flats, Arid Mesa, Plaza of Heroes |
| Colour sources with the rocks | 28 white, 20 black | Rocks: Arcane Signet and Orzhov Signet add 2 white and 2 black; Mox Amber and Chrome Mox are conditional and not counted |
| Legendary lands | 4 | Urborg, Takenuma, Eiganjo, Minas Tirith |
| Sometimes enter tapped | 5 | Minas Tirith (tapped without a legendary creature), Isolated Chapel (tapped without a Plains or Swamp), Vault of Champions (tapped only with fewer than two opponents), Godless Shrine (2 life or tapped), Witch Enchanter (3 life or tapped) |
| Fetch lands | 2 | Marsh Flats and Arid Mesa, 1 life each |

**Pips.** The 63 nonland cards in the 98 have 52 white pips (39 cards with a white pip) and 9 black pips (9 cards with a black pip). 13 cards cost {W}{W}: Adeline, Archangel of Thune, Cloud, Court of Ardenvale, Flickerwisp, Gandalf the White, Kor Outfitter, Puresteel Paladin, Ranger-Captain of Eos, Skyclave Apparition, Solitude, The Wandering Emperor and Wrath of God. No card costs {B}{B}; the 9 black cards each have one black pip: Deadly Rollick, Demonic Tutor, Elas il-Kor, Exquisite Blood, Imperial Seal, Lotho, Orcish Bowmasters, Vampiric Tutor and Vito (Vito's {3}{B}{B} activation is the only {B}{B} in the deck). The deck is white with a light black for Tymna, Vito, Elas, Lotho, the tutors, Exquisite Blood and Deadly Rollick: black is oversupplied (18 black lands and a Signet for 9 black cards and Tymna), not short.

**Land count and early mana** (hypergeometric, exact, from the 98-card library):

- Three or more lands by turn 3 on the play (9 cards): 69.0% (76.7% on the draw). Four or more by turn 4 (10 cards): 50.9% (60.3% on the draw). A 2-to-4-land opening hand: 73.4%.
- 35 lands, 6 rocks and Wayfarer's Bauble smooth this out. 16 of the 63 nonland cards cost 0 or 1 (four of them rocks), 15 cost 4 or more and the payoffs cost 5; most of the curve is 3 and 4 (28 cards), and the Equipment (cast {3}, equip {2} or {3}) and Court of Ardenvale use the extra mana.
- **Mana through turn 5 (an estimate).** The deck typically has 4 to 5 mana through turn 5 even with 35 lands. An outside goldfish model (an estimate, not the simulator) gave a mean of about 5.3 mana on turn 6 on the play for 35 lands plus Wayfarer's Bauble. Do not count on more than that: the mana is spent on equip costs, Court and the second spell each turn.
- **Yoshimaru turn 1:** 87.0% on the play and 90.4% on the draw (an untapped white land among the 24, lands only; see The commanders for Chrome Mox).
- **Tymna turn 3 (on the play, 9 cards):** the chance that two different lands among the 9 cards make white and black is 79.1%, and 83.8% on the draw (10 cards). This counts Plaza of Heroes for Tymna (she is a legendary spell). It is necessary but not sufficient: the third land is also needed (at least three lands in the 9 cards with a white and a black: 64.3%; 72.1% on the draw), and a Signet, Chrome Mox or Mox Amber can supply it.
- **White double pips:** at least 2 white lands by turn 3 (9 cards): 74.8%; by turn 4 (10 cards): 80.4%; by turn 6 (12 cards): 88.4%. White rocks add to this. **Black:** no card needs {B}{B}, only Vito's activation; at least 2 black lands by turn 4: 58.4%; by turn 6: 69.4%. Urborg turns every land into a black source.

**Life costs.** Caves of Koilos 1 per coloured tap; Marsh Flats and Arid Mesa 1; Godless Shrine 2 (or tapped); Witch Enchanter's land side 3 (or tapped); Mana Vault 1 damage in the draw step while tapped; Vampiric Tutor and Imperial Seal 2; Tymna X per trigger (and per doubled trigger); Lotho 1 per trigger, on any player's second spell each turn (2 per spell under Delney); The One Ring 1 per burden counter each upkeep. Offsets: Tymna's lifelink, Elas, Haliya (also for each Treasure and each Equipment), Solitude's lifelink and Heliod's lifelink on a creature of our choice. Keep an eye on life at a 40-life table. Lotho's Treasures are one-shot mana of any colour: they fix white and black and can pay for a second spell, but they come from a trigger the whole table can set off, so they are not counted among the colour sources above.

**Curve** (63 nonland cards in the 98; Walking Ballista counts 0, Deadly Rollick 4 and Flawless Maneuver 3 at their printed costs, Solitude 5; Witch Enchanter is counted as a land). Average mana value 2.57 (162 over 63). 29 of 63 cost 0 to 2.

- **0 (3):** Chrome Mox, Mox Amber, Walking Ballista
- **1 (13):** Enlightened Tutor, Ephemerate, Giver of Runes, Imperial Seal, Mana Vault, Path to Exile, Silence, Skrelv, Defector Mite, Sol Ring, Steelshaper's Gift, Swords to Plowshares, Vampiric Tutor, Wayfarer's Bauble
- **2 (13):** Arcane Signet, Cloud, Demonic Tutor, Elas il-Kor, Kor Outfitter, Lotho, Orcish Bowmasters, Orzhov Signet, Puresteel Paladin, Sting, Stoneforge Mystic, Sunpearl Kirin, Swiftfoot Boots
- **3 (19):** Adeline, Ardenn, Buster Sword, Delney, Eerie Interlude, Flawless Maneuver, Flickerwisp, Forge Anew, Haliya, Heliod, Mithril Coat, Ranger-Captain of Eos, Recruiter of the Guard, Skyclave Apparition, Stroke of Midnight, Sword of Feast and Famine, Sword of Fire and Ice, Sword of Hearth and Home, Vito
- **4 (9):** Akroma's Will, Court of Ardenvale, Deadly Rollick, Felidar Guardian, Restoration Angel, Teleportation Circle, The One Ring, The Wandering Emperor, Wrath of God
- **5 (6):** Archangel of Thune, Conjurer's Closet, Elesh Norn, Exquisite Blood, Gandalf the White, Solitude

**Card types.** As listed under Decklist by type: 28 creatures (counting Witch Enchanter's creature face, Heliod as an enchantment creature, and Walking Ballista and Skrelv as artifact creatures), 1 planeswalker, 4 enchantments, 16 artifacts, 11 instants, 4 sorceries and 34 lands; counted as a land, Witch Enchanter // Witch-Blessed Meadow makes it 35 lands and 27 creatures. Counting Walking Ballista and Skrelv, the list has 18 artifacts and 7 Equipment.

**Finding cards.**
- **Tutors:** Demonic Tutor, Vampiric Tutor, Enlightened Tutor, Imperial Seal (4 any-card or type tutors), plus Ranger-Captain of Eos and Recruiter of the Guard (creatures) and Cloud, Stoneforge Mystic and Steelshaper's Gift (Equipment).
- **Draw:** Tymna, Puresteel Paladin, Haliya, The One Ring, Court of Ardenvale's monarchy, Buster Sword and Sword of Fire and Ice on a hit, Orcish Bowmasters (when an opponent draws extra), and Minas Tirith.

**Interaction.**
- **Spot removal spells (4):** Swords to Plowshares, Path to Exile, Deadly Rollick, Stroke of Midnight (the only one that also hits a noncreature permanent).
- **Removal on a body:** Solitude and Skyclave Apparition (creatures and permanents with mana value 4 or less); Witch Enchanter (artifacts and enchantments).
- **Answers to noncreature permanents are thin:** Stroke of Midnight, Skyclave Apparition (mana value 4 or less), Witch Enchanter (artifact or enchantment).
- **Other removal:** The Wandering Emperor (-2 exiles a tapped creature), Eiganjo's channel ({2}{W}, discard: 4 damage to an attacking or blocking creature, {1} less per legendary creature we control), Sword of Fire and Ice's 2 damage, Walking Ballista and Orcish Bowmasters pings.
- **Sweepers (1):** Wrath of God, which can be one-sided (see Protecting the commanders).
- **Protection (6):** Akroma's Will, Flawless Maneuver, Giver of Runes, Mithril Coat, Skrelv, Swiftfoot Boots; plus the blink spells, the Swords' protection (with Forge Anew in response) and Plaza of Heroes.
- **Stops for the win turn:** Silence, Ranger-Captain of Eos; no board shield.
- **No counterspells and no graveyard hate.**

## Bracket and Rule 0

**Bracket 4,** with eight Game Changers: **Mana Vault**, **Demonic Tutor**, **Vampiric Tutor**, **Enlightened Tutor**, **Imperial Seal**, **The One Ring**, **Orcish Bowmasters** and **Chrome Mox**. It has no free counterspell, no mass land destruction and no extra turns.

Disclose before the game:

- **The eight Game Changers** above.
- **Four infinite or game-ending lines:** Vito with Exquisite Blood; Heliod with Walking Ballista (with {1}{W} for lifelink); Archangel of Thune with Walking Ballista and a lifelink source; and the Restoration Angel and Felidar Guardian loop (infinite life with Elas il-Kor or Haliya, and a kill with Vito).
- **Tutors:** four any-card or type tutors (Demonic, Vampiric, Enlightened, Imperial Seal), two creature searchers (Ranger-Captain of Eos, Recruiter of the Guard) and three Equipment searchers (Cloud, Stoneforge Mystic, Steelshaper's Gift).
- **Free spells:** Deadly Rollick and Flawless Maneuver are free with a commander on the battlefield, and Solitude can be evoked by exiling a white card. Fast mana: Sol Ring, Mana Vault, Mox Amber and Chrome Mox (exile a card from hand).
- **Court of Ardenvale** makes us the monarch. The monarch draws an extra card each turn, and an opponent who deals combat damage to us takes the monarchy (rules points, not card text); the deck has few blockers, so at a four-player table the table will attack us for it.
- **The doubling:** Elesh Norn, Gandalf and Delney make enter and life-gain triggers fire extra times, and Elesh Norn also shuts off opponents' enter triggers.

Re-check the official list at https://commanderbrackets.com/faq before an event.

## Decklist by type (100)

**Commanders (2).** Yoshimaru, Ever Faithful; Tymna the Weaver

**Creatures (28).** Adeline, Resplendent Cathar; Archangel of Thune; Ardenn, Intrepid Archaeologist; Cloud, Midgar Mercenary; Delney, Streetwise Lookout; Elas il-Kor, Sadistic Pilgrim; Elesh Norn, Mother of Machines; Felidar Guardian; Flickerwisp; Gandalf the White; Giver of Runes; Haliya, Guided by Light; Heliod, Sun-Crowned; Kor Outfitter; Lotho, Corrupt Shirriff; Orcish Bowmasters; Puresteel Paladin; Ranger-Captain of Eos; Recruiter of the Guard; Restoration Angel; Skrelv, Defector Mite; Skyclave Apparition; Solitude; Stoneforge Mystic; Sunpearl Kirin; Vito, Thorn of the Dusk Rose; Walking Ballista; Witch Enchanter // Witch-Blessed Meadow

**Planeswalkers (1).** The Wandering Emperor

**Enchantments (4).** Court of Ardenvale; Exquisite Blood; Forge Anew; Teleportation Circle

**Artifacts (16).** Arcane Signet; Buster Sword; Chrome Mox; Conjurer's Closet; Mana Vault; Mithril Coat; Mox Amber; Orzhov Signet; Sol Ring; Sting, the Glinting Dagger; Swiftfoot Boots; Sword of Feast and Famine; Sword of Fire and Ice; Sword of Hearth and Home; The One Ring; Wayfarer's Bauble

**Instants (11).** Akroma's Will; Deadly Rollick; Eerie Interlude; Enlightened Tutor; Ephemerate; Flawless Maneuver; Path to Exile; Silence; Stroke of Midnight; Swords to Plowshares; Vampiric Tutor

**Sorceries (4).** Demonic Tutor; Imperial Seal; Steelshaper's Gift; Wrath of God

**Lands (34).** Arid Mesa; Caves of Koilos; Command Tower; Eiganjo, Seat of the Empire; Godless Shrine; Isolated Chapel; Marsh Flats; Minas Tirith; Plaza of Heroes; Scrubland; Takenuma, Abandoned Mire; Urborg, Tomb of Yawgmoth; Vault of Champions; 14 Plains; 7 Swamp

## Import list (100)

```
1 Adeline, Resplendent Cathar
1 Akroma's Will
1 Arcane Signet
1 Archangel of Thune
1 Ardenn, Intrepid Archaeologist
1 Arid Mesa
1 Buster Sword
1 Caves of Koilos
1 Chrome Mox
1 Cloud, Midgar Mercenary
1 Command Tower
1 Conjurer's Closet
1 Court of Ardenvale
1 Deadly Rollick
1 Delney, Streetwise Lookout
1 Demonic Tutor
1 Eerie Interlude
1 Eiganjo, Seat of the Empire
1 Elas il-Kor, Sadistic Pilgrim
1 Elesh Norn, Mother of Machines
1 Enlightened Tutor
1 Ephemerate
1 Exquisite Blood
1 Felidar Guardian
1 Flawless Maneuver
1 Flickerwisp
1 Forge Anew
1 Gandalf the White
1 Giver of Runes
1 Godless Shrine
1 Haliya, Guided by Light
1 Heliod, Sun-Crowned
1 Imperial Seal
1 Isolated Chapel
1 Kor Outfitter
1 Lotho, Corrupt Shirriff
1 Mana Vault
1 Marsh Flats
1 Minas Tirith
1 Mithril Coat
1 Mox Amber
1 Orcish Bowmasters
1 Orzhov Signet
1 Path to Exile
1 Plaza of Heroes
1 Puresteel Paladin
1 Ranger-Captain of Eos
1 Recruiter of the Guard
1 Restoration Angel
1 Scrubland
1 Silence
1 Skrelv, Defector Mite
1 Skyclave Apparition
1 Sol Ring
1 Solitude
1 Steelshaper's Gift
1 Sting, the Glinting Dagger
1 Stoneforge Mystic
1 Stroke of Midnight
1 Sunpearl Kirin
1 Swiftfoot Boots
1 Sword of Feast and Famine
1 Sword of Fire and Ice
1 Sword of Hearth and Home
1 Swords to Plowshares
1 Takenuma, Abandoned Mire
1 Teleportation Circle
1 The One Ring
1 The Wandering Emperor
1 Tymna the Weaver
1 Urborg, Tomb of Yawgmoth
1 Vampiric Tutor
1 Vault of Champions
1 Vito, Thorn of the Dusk Rose
1 Walking Ballista
1 Wayfarer's Bauble
1 Witch Enchanter // Witch-Blessed Meadow
1 Wrath of God
1 Yoshimaru, Ever Faithful
14 Plains
7 Swamp
```
