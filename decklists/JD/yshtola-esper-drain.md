# Y'shtola, Night's Blessed — Esper Drain

Drain the table a little with every spell, then all at once. Y'shtola turns your ordinary noncreature spells into damage to every opponent and life for you, Sanguine Bond and Marauding Blight-Priest turn that life into more damage, and Exsanguinate, Debt to the Deathless and Torment of Hailfire finish the game, with Cabal Coffers and Urborg paying for a big X. Exquisite Blood with Sanguine Bond is the two-card kill the tutors look for. The deck wins slowly, so it is weak to fast creature pressure and to fast combo decks that win before the drain gets there.

| | |
|---|---|
| **Commander** | Y'shtola, Night's Blessed ({1}{W}{U}{B}, 2/4, vigilance; white-blue-black) |
| **Bracket** | 4 (14 Game Changers: Necropotence, Notion Thief, Consecrated Sphinx, Smothering Tithe, Rhystic Study, Demonic Tutor, Mystical Tutor, Vampiric Tutor, Grand Arbiter Augustin IV, Force of Will, Farewell, Teferi's Protection, Orcish Bowmasters, Serra's Sanctum) |
| **Simulator** | deck key `yshtola`; results in `decklists/pool/pool-results.md`, history in `decklists/CHANGELOG.md` |


## Strategy

### The commander

**Y'shtola, Night's Blessed** ({1}{W}{U}{B}, 2/4, vigilance). *At the beginning of each end step, if a player lost 4 or more life this turn, you draw a card. Whenever you cast a noncreature spell with mana value 3 or greater, Y'shtola deals 2 damage to each opponent and you gain 2 life.*

She does three jobs in this deck:

- **She drains the table on every big spell.** Each trigger is 2 damage to each opponent (6 across three opponents) and 2 life for you. The life is a lifegain event, so Sanguine Bond and Marauding Blight-Priest turn it into more damage.
- **She draws cards.** Any player counts, you included, and she checks at *every* end step, yours and the opponents'.
- **She is the thing the table attacks.** A 2/4 dies to most removal, so most of the guide below is about keeping her alive or winning without her.

**Thirty-three spells trigger her.** The trigger is on cast, so a countered spell still drains.

| Type | Spells of mana value 3 or more |
|---|---|
| Artifacts (4) | Champion's Helm, Coalition Relic, Conjurer's Closet, Palantír of Orthanc |
| Enchantments (12) | Arrest, Bound in Silence, Exquisite Blood, Necropotence, Polluted Bonds, Prayer of Binding, Propaganda, Rhystic Study, Sanguine Bond, Smothering Tithe, Sphere of Safety, Teleportation Circle |
| Instants (5) | Anguished Unmaking, Force of Will, Generous Gift, Teferi's Protection, Your Fate Ends Here |
| Sorceries (12) | Austere Command, Cast Away Doubt, Debt to the Deathless, Diabolic Tutor, Exsanguinate (X of 1 or more), Farewell, Idyllic Tutor, Read the Bones, Ringsight, Tezzeret's Gambit, Torment of Hailfire (X of 1 or more), Toxic Deluge |

**Spells that don't trigger her:** every creature, and every spell of mana value 2 or less: Counterspell, Dovin's Veto, Demonic Tutor, Fatal Push, Go for the Throat, Path to Exile, Mystical Tutor, Vampiric Tutor, Mystic Remora, the rocks (Sol Ring, Arcane Signet, both Talismans), Expedition Map, and Profane Tutor (mana value 0, even cast from suspend). Zur's search puts an enchantment onto the battlefield without casting it, so it doesn't trigger her either.

**Her draw needs 4 life lost by one player in one turn.** One trigger is only 2 to each opponent, so a draw needs a second source in the same turn:

- **Two triggers**, such as a removal spell on an opponent's turn plus your own main-phase spell. Cheap instants with mana value 3 (Anguished Unmaking, Generous Gift, Your Fate Ends Here) on an opponent's turn are the easiest second trigger. Cast Away Doubt does it alone: its 2 damage plus her 2 is 4 to each opponent.
- **One trigger plus a drain** from Sanguine Bond, Blight-Priest, Palantír of Orthanc or Urborg Syphon-Mage. Sheoldred, the Apocalypse adds 2 to each opponent's every draw.
- **Your own life.** Read the Bones costs 2, Anguished Unmaking 3, and Necropotence as much as you like.

**When to cast her.** Turn 3 off a turn-2 rock, otherwise turn 4, with a spell of mana value 3 or more to follow. In the plan file she is the top priority from turn 2 on. Check your colours first: she needs {W}, {U} and {B} from three different sources (Command Tower, a rock or a fetch land can be the third). She is a creature, so Dovin's Veto can't protect her cast. Into open blue mana, activate Mistrise Village first ({U}, {T}: the next spell can't be countered) or accept that the first Force of Will eats her; she can be recast for {2} more per previous cast.

### How the deck wins

The damage comes mostly from Y'shtola's triggers and drains (Sanguine Bond, Blight-Priest, the X spells, Syphon-Mage), and only a little from combat. The sims show about a third of her wins as the Bond + Exquisite Blood combo and the rest as damage, and her plan online in about two games in three. There are four finishing lines and a grind that sets them up.

**The drain pieces:**

| Piece | What it does | Fires on |
|---|---|---|
| Sanguine Bond ({3}{B}{B}) | Whenever you gain life, target opponent loses that much | Every Y'shtola trigger (2 life), every X drain, Prayer of Binding (2), Sheoldred's draws |
| Marauding Blight-Priest ({2}{B}) | Whenever you gain life, each opponent loses 1 | The same events, to every opponent |
| Exquisite Blood ({4}{B}) | Whenever an opponent loses life, you gain that much | Any drain, ping or life payment by an opponent |
| Polluted Bonds ({3}{B}{B}) | Whenever an opponent's land enters, they lose 2 and you gain 2 | Each opponent's land drop: a drain, a gain for Bond and the Priest, and a start for Blood |
| Urborg Syphon-Mage ({2}{B}) | {2}{B}, {T}, discard a card: each other player loses 2, you gain the total | A spare land; the gain is one event |
| Palantír of Orthanc ({3}) | At your end step, scry 2; the opponent you target lets you draw, or you mill X and they lose life equal to the cards' total mana value | Every end step (and it triggers her when cast) |
| Cast Away Doubt ({2}{B}) | Draw two; 2 damage to each player | With her trigger, each opponent loses 4 |
| Sheoldred, the Apocalypse ({2}{B}{B}) | Opponents lose 2 per draw; you gain 2 per draw | Each draw step, and every extra draw (Loran, Rhystic, Tithe, cantrips) |

#### Line 1: Exquisite Blood + Sanguine Bond

Ten mana for the pair ({3}{B}{B} and {4}{B}), over one turn or two. Once both are on the battlefield, any life change starts a loop that kills the table.

1. Cast **Sanguine Bond** first. It works alone: every gain drains one opponent for the same amount.
2. Cast **Exquisite Blood** when you can follow it with a trigger the same turn (5 + 3 = 8 mana), or when no opponent can answer it before your next turn. It is the card the table's Cyclonic Rift, Swan Song and Force of Will target.
3. **Start the loop.** The cast trigger of the second enchantment does not start it: Y'shtola's trigger resolves while that enchantment is still on the stack. You need one more life change *after both are on the battlefield*:
   - any spell of mana value 3 or more with Y'shtola out (Cast Away Doubt at three mana is the cheapest);
   - Exsanguinate for X = 1 ({1}{B}{B}) or a Syphon-Mage activation (both drain an opponent and gain you life);
   - on an opponent's turn, a mana value 3 instant (Generous Gift, Your Fate Ends Here), their land drop under Polluted Bonds, any draw under Sheoldred, or any life they pay themselves (a fetch land, a shock land, Necropotence).
4. **The loop.** An opponent loses N, Blood gains you N, Bond makes an opponent lose N, Blood gains you N again. Each Bond trigger targets one opponent; when he dies, the next trigger targets the next one. The loop ends only when every opponent is dead.

What stops it: your own **Teferi's Protection** (your life can't change), a counter on the second piece, **Cyclonic Rift** or a removal spell on either enchantment before the loop starts, and **Phyrexian Unlife** on an opponent (he is at 0 or less and still alive: remove the Unlife with Anguished Unmaking, Generous Gift, Loran or Skyclave Apparition and he dies).

Tutor for Bond first, then Blood (see the tutor table). Zur can't fetch either (mana value 5).

#### Line 2: Debt to the Deathless or Exsanguinate with Bond

The X drains gain you everything they drain, and with Bond out that gain hits one opponent again. A table of three opponents at 20 life, Y'shtola and Bond out, **Debt to the Deathless** ({X}{W}{W}{B}{B}) for X = 5 (nine mana):

1. Y'shtola's cast trigger: each opponent 18, you gain 2, Bond: the target 16.
2. Debt resolves: each opponent loses 10 (A at 6, B and C at 8). You gain 30 as one event.
3. Bond: B loses 30 and dies. The Blight-Priest adds 1 to every opponent for each gain event.
4. The other two are at 6 and 8: the next trigger, X drain or Syphon-Mage finishes them, or Exquisite Blood does it at once.

Debt drains twice as hard per mana (2X, for X + 4 mana) and is the better card from about seven mana up. **Exsanguinate** ({X}{B}{B}) needs no white and starts cheaper (X = 1 is a three-mana trigger). Cast the X spell last in the turn, after your trigger spells have set up the kill. Count before you cast: each opponent loses X (or 2X), plus 2 from Y'shtola, plus 1 per gain event from the Priest, and the whole gain goes to one opponent through Bond.

#### Line 3: Cabal Coffers + Urborg for the big X

**Urborg, Tomb of Yawgmoth** makes every land a Swamp, and **Cabal Coffers** ({2}, {T}: add {B} for each Swamp you control) counts them. With L lands the pair makes 2L - 3 mana: 11 at seven lands, 13 at eight, 15 at nine.

- **Debt at eleven mana:** X = 7, so each opponent loses 14 and you gain 42 (three opponents); Bond sends 42 to one of them. Two {W} must come from lands (Plains, a dual, Serra's Sanctum): Coffers makes only {B}.
- **Exsanguinate at eleven mana:** X = 9. **Torment of Hailfire** at eleven: X = 9 (Line 4).
- **Two spells in one turn:** Debt for X = 4 (eight mana) plus a three-mana trigger spell. Each cast triggers her.
- **Finding the missing half:** **Expedition Map** ({2}, {T}, sacrifice: a land card to your hand) and **Urza's Cave** ({3}, {T}, sacrifice: a land card onto the battlefield tapped) fetch Urborg or Coffers. When both are out, the Cave finds the land that fixes the hand's colour. Coffers alone counts only real Swamps: eight Swamps and the four Swamp-type duals (Underground Sea, Watery Grave, Scrubland, Godless Shrine).
- **Mystical Tutor** ({U}) puts Exsanguinate or Debt on top at the end of an opponent's turn so you draw it for the Coffers turn.

The Coffers turn is a sorcery-speed turn into a table that holds counters: activate Mistrise Village first and keep a counter of your own behind the X spell.

#### Line 4: Torment of Hailfire

**Torment of Hailfire** ({X}{B}{B}, sorcery): repeat X times; each opponent loses 3 life unless he sacrifices a nonland permanent or discards a card. The opponents choose, so it is a table-wide strip first and a drain second. It is best when the table has few permanents and few cards, or when a big X outlasts their board (X of 9 asks for nine sacrifices or discards each). With X of 1 or more it triggers Y'shtola. In the sims Torment was cast in only about 39% of the games it was drawn, at round 9, and stuck in hand the rest: treat it as a late mana sink, not a card to plan the turn around.

#### Line 5: The grind with her triggers

The fair game. When the combo and the X turn are not ready, keep the engine running:

- **One spell of mana value 3 or more every turn** (the table above), preferring the ones that also do a job: a lock Aura, a tutor, a rock, Rhystic Study.
- **Two triggers on a turn draw a card** at the end step. A removal spell on an opponent's turn plus your own main-phase spell is the usual pair.
- **Sheoldred** turns every opponent draw into 2 life lost; Loran's tap, Rhystic Study, Smothering Tithe, Consecrated Sphinx and Notion Thief turn the same draws into cards and mana.
- **Palantír of Orthanc** gives a scry 2 and a card or a drain at every end step.
- Three opponents at 40 is 120 life, and 2 per trigger is slow, so the grind's real job is to land Bond, Blood and the Priest, keep Y'shtola alive, and hold the table off you (see Game plan) until Line 1, 2 or 3 is ready.

### Game plan

**Turns 1 to 3: mana.** Turn 2 Sol Ring, Arcane Signet or Talisman of Progress; Coalition Relic on three (a rock *and* a trigger once she is out). Suspend Profane Tutor early: it resolves two turns later for no mana. Smothering Tithe or Rhystic Study on turn 3 to 4 if you have no commander yet. Y'shtola on turn 3 or 4 with a spell to follow. Play lands that keep W, U and B open before you worry about anything else.

**Turns 4 to 6: the engine.** Every turn cast a spell of mana value 3 or more. Prefer trigger spells first, while the opponents' life is high and blockers don't matter. Look for Sanguine Bond first, then Exquisite Blood. Lock or remove the biggest creature and put Propaganda out so the table can't attack you. Against a combo table keep a counter up instead of tapping out for the 5-drop.

**Late game.** The first trigger usually lands by turn 6 to 8. Count before you cast an X spell. If the table is not close, keep drawing and taxing.

**Go now or hold?**

- **Go now** when the kill is on the table (Line 1 with a start ready, an X drain that finishes the opponents), when the blue players are tapped low, or when you hold two pieces of protection (Counterspell, Dovin's Veto, Force of Will, Mistrise Village).
- **Hold** when a combo deck (Kinnan, Urza, Zur) has the mana for its kill next turn: keep Counterspell, Dovin's Veto or Force of Will up, and spend the rest on instant-speed trigger spells (Generous Gift, Anguished Unmaking, Prayer of Binding) at their end step.
- **Hold** the second piece of Line 1 until you can start the loop the same turn (8 mana with Bond out and Y'shtola on the battlefield). Every turn a lone piece sits on the table is a turn for a Cyclonic Rift or a removal spell to find it.
- Mystic Remora's cumulative upkeep grows by {1} each turn. Pay it once or twice, then let it go.

#### Keeping the table off you

Y'shtola is a 2/4 and the deck wins slowly, so the table's creature decks are the main danger.

- **Pillowfort:** Propaganda taxes every attacker {2}. **Sphere of Safety** ({4}{W}) taxes every attacker {X}, X being the number of enchantments you control (the list has 13, and each Aura you put on an opponent's creature counts). Cast them early against creature decks.
- **Locks:** Arrest and Bound in Silence shut down a threat (each triggers Y'shtola as you cast it). Arrest also stops activated abilities.
- **Wipes:** Austere Command, Farewell and Toxic Deluge ({2}{B}: pay X life, all creatures get -X/-X until end of turn; a cheap answer to a wide board that also triggers Y'shtola).
- **Farewell** ({4}{W}{W}, sorcery) exiles, as you choose, all artifacts, all creatures, all enchantments and/or all graveyards. Choose carefully: creatures exiles Y'shtola, enchantments exiles your own locks, Sanguine Bond, Exquisite Blood and Smothering Tithe, artifacts exiles your rocks. Cast Teferi's Protection first and your permanents phase out and survive (she is phased out, so Farewell itself won't trigger her).

### Engine cards

**Orcish Bowmasters punishes the table's extra cards.** *Flash. When it enters, and whenever an opponent draws a card except the first one they draw in each of their draw steps, it deals 1 damage to any target, then you amass Orcs 1.* Flash it in at the end of an opponent's turn, or in answer to their draw spell, and aim the ping at a mana creature or a 1-toughness combo piece (Bloom Tender, Birds of Paradise, the elves in Kinnan's list; Walking Ballista at X = 1 in Urza's). Each ping grows the Orc Army.

**Mana, then cards.** The deck usually holds more spells than it can cast, so **Smothering Tithe** comes first: each opponent who draws without {2} to spare gives you a Treasure, and casting it triggers Y'shtola. **Consecrated Sphinx** draws two whenever an opponent draws; it's a creature, so it doesn't trigger her, and it's best cast when the hand is running low. **Necropotence** skips your draw step and turns life into cards (cast it with a healthy life total; the drains refill it; your maximum hand size is seven, so spend what you draw). **Mystic Remora** draws a card when an opponent casts a noncreature spell unless they pay {4}. **Esper Sentinel** draws a card when an opponent casts their first noncreature spell each turn, unless they pay {X} (its power). **Notion Thief** (flash) turns each extra card an opponent draws into a card for you. **Loran of the Third Path** ({2}{W}, vigilance) taps for "you and target opponent each draw a card"; with Notion Thief out you get both cards, with Sheoldred out you gain 2 and they lose 2, and with Bond that gain drains again. **Rhystic Study** ({2}{U}) draws a card whenever an opponent casts a spell unless they pay {1}. The other draw spells are Read the Bones and Tezzeret's Gambit.

**Blink.** **Displacer Kitten** ({3}{U}) exiles and returns a nonland permanent you control whenever you cast a noncreature spell: each lock, drain or draw spell also re-buys an enters trigger (Loran destroys another artifact or enchantment, Orcish Bowmasters pings again, Esper Sentinel comes back fresh) or untaps a used rock. Loran is the best target: every blink is removal on the rocks, engines and stax pieces that beat this deck. **Skyclave Apparition** blinks badly: its old exiled card stays exiled, but its owner gets an X/X Illusion token each time it leaves, so blink it only when the new exile is worth a token. **Teleportation Circle** ({3}{W}) does the same to one creature or artifact at each of your end steps, and **Conjurer's Closet** ({5}) to one creature. With both out, two enters triggers come back every turn.

**Discount and cards for legends.** Grand Arbiter Augustin IV makes your white and blue spells cost {1} less each (a white-and-blue spell {2} less) and the opponents' spells {1} more. **Venat, Heart of Hydaelyn** draws a card the first time each turn you cast a legendary spell (Y'shtola, Augustin, Zur, Sheoldred, Loran, Palantír of Orthanc); its {7} ability exiles a nonland permanent and flips it to Hydaelyn, the Mothercrystal, which gives another creature a +1/+1 counter and indestructible each turn (and draws a card if it is legendary, such as Y'shtola).

### Mulligans

The first mulligan is free (you see two 7-card hands). After that it's a London mulligan: draw 7, put one card on the bottom for each mulligan after the free one.

| Hand | Keep when |
|---|---|
| 7 cards | 2 to 5 lands (2 lands only with a rock, Demonic Tutor or another way to find land; 5 lands only with two spells that do something); {W}, {U} and {B} all reachable by turn 4 from lands, a fetch land, Command Tower or a rock; and a reason to go: Y'shtola, a tutor, an engine (Rhystic Study, Smothering Tithe, Necropotence, Orcish Bowmasters, Consecrated Sphinx) or a rock that casts her on turn 3 |
| 6 or 5 cards | 2 to 4 lands, plus Y'shtola, a rock or a tutor |
| 4 cards | Keep |

- **Counters, removal and wipes are not a keep requirement.** A hand of only those never threatens to win, and the draws bring them.
- **Count colours.** Y'shtola needs three colours. Counterspell needs {U}{U}; Sanguine Bond and Necropotence need {B}{B} and {B}{B}{B}.
- **The finishers are late cards.** Exsanguinate, Debt to the Deathless and Torment of Hailfire in a seven-card hand are not a reason to keep; Austere Command and Farewell cost six.
- **Bottom first:** the six-drops, then the second finisher, then lands past the fourth. Keep Y'shtola, tutors, rocks and Smothering Tithe.

| # | Hand | Call | Why |
|---|---|---|---|
| 1 | Command Tower, Godless Shrine, Plains, Sol Ring, Exquisite Blood, Sanguine Bond, Demonic Tutor | Keep | Y'shtola from the command zone on turn 3: Plains for {W}, Godless Shrine for {B}, Command Tower for {U} and Sol Ring for the rest. Both combo pieces are already in hand, and Demonic Tutor finds protection or a rock |
| 2 | Tundra, Underground Sea, Swamp, Talisman of Progress, Rhystic Study, Propaganda, Vampiric Tutor | Keep | Three lands in all three colours and a turn-2 rock. Rhystic Study and Propaganda are early engine and defence; Vampiric Tutor at an end step puts Sanguine Bond or Exquisite Blood on top for the next draw |
| 3 | Scrubland, Hallowed Fountain, Fatal Push, Path to Exile, Dovin's Veto, Counterspell, Teferi's Protection | Ship | Two lands, no mana and no engine. Counterspell needs {U}{U} and there is one {U}. Every card is reactive |
| 4 | Plains, Plains, Plains, Swamp, Arrest, Austere Command, Farewell | Ship | No blue for Y'shtola, one early play and two six-drops. The wipes win a late game, not this one |
| 5 | Plains, Island, Swamp, Underground River, Flooded Strand, Smothering Tithe, Diabolic Tutor | Keep (close) | Five lands, but both spells are real: Tithe on turn 4 starts making Treasures, and Diabolic Tutor ({2}{B}{B}, from Swamp and Underground River) finds Sanguine Bond. A flood is not fatal, since Coffers, the X drains and Syphon-Mage soak up lands |
| 6 | Swamp, Underground Sea, Plains, Cast Away Doubt, Read the Bones, Exsanguinate, Torment of Hailfire | Keep (close) | Three lands in three colours and two three-mana draw spells (both trigger her) that find rocks, lands and the combo pieces. The two X spells are dead for a while, but going to six cards costs more than playing the draws |
| 7 | Swamp, Rhystic Study, Propaganda, Smothering Tithe, Sphere of Safety, Sanguine Bond, Exquisite Blood | Ship | One land and nothing that costs under three. It needs two more lands before it does anything, and two colours it does not have |
| 8 | Command Tower, Marsh Flats, Swamp, Coalition Relic, Talisman of Hierarchy, Austere Command, Exsanguinate | Keep, bottom Austere Command | The 6-card rule: three lands and two rocks, so Y'shtola comes down from the command zone on turn 3. Austere Command is the slowest card; Exsanguinate wins late |

### What to tutor for

Y'shtola is the commander and starts in the command zone, so no tutor finds her. What the tutors do find falls into four groups: a drain piece (Sanguine Bond, Exquisite Blood, Debt to the Deathless, Exsanguinate), the mana or colours to cast it, protection for it, and an answer to whatever stops it. Read the board first (what is out, what the table can do, everyone's life, your mana, the turn), then pick the tutor whose speed and cost fit the card you need.

#### The tutors and tutor-like cards

| Card | Cost | The card goes | Finds | Speed | The catch |
|---|---|---|---|---|---|
| Demonic Tutor | {1}{B} | Hand | Any card | Sorcery | Mana value 2, so no Y'shtola trigger. The cheapest way to put any card in hand |
| Diabolic Tutor | {2}{B}{B} | Hand | Any card | Sorcery | Mana value 4: with Y'shtola out it is a tutor and a drain in one card (2 to each opponent, 2 life for you, a gain for Bond) |
| Vampiric Tutor | {B} | Top of library | Any card | Instant | You lose 2 life. You get the card at your next draw, or at once if you cast it in response to a draw you control (see below). Mana value 1: no trigger |
| Mystical Tutor | {U} | Top of library | An instant or sorcery | Instant | Cannot find a creature, enchantment, artifact or land, so Bond, Blood, Loran and the lands are out of reach. No life cost. Mana value 1: no trigger |
| Idyllic Tutor | {2}{W} | Hand | An enchantment | Sorcery | Reveals the card. Mana value 3: triggers her. The 13 enchantments: Arrest, Bound in Silence, Exquisite Blood, Mystic Remora, Necropotence, Polluted Bonds, Prayer of Binding, Propaganda, Rhystic Study, Sanguine Bond, Smothering Tithe, Sphere of Safety, Teleportation Circle |
| Ringsight | {1}{U}{B} | Hand | A card that shares a colour with a legendary creature you control | Sorcery | The Ring tempts you first. With Y'shtola out that is any white, blue or black card (Bond, Blood, Debt, Exsanguinate, Counterspell, Force of Will, Teferi's Protection, Kitten, Loran); it cannot find a colourless card (Sol Ring, the rocks, Palantír, Champion's Helm, Conjurer's Closet) or a land. Without her, the Ring-bearer you choose becomes legendary, so a coloured creature you control (Orcish Bowmasters, Esper Sentinel) works; with no creature it finds nothing. Mana value 3: triggers her |
| Profane Tutor | Suspend 2: {1}{B} | Hand | Any card | Cast from exile in your upkeep | It has no mana cost, so it can only be suspended. It is cast for free in your second upkeep after you suspend it, before your draw, and you pick the card when it resolves, with the board of that turn. Mana value 0: no trigger. It is a cast, so it can be countered, and Drannith Magistrate stops it |
| Zur the Enchanter | {1}{W}{U}{B} | Battlefield | An enchantment of mana value 3 or less | Whenever Zur attacks | Six targets: Arrest, Bound in Silence, Mystic Remora, Necropotence, Propaganda, Rhystic Study. Put onto the battlefield, not cast, so no trigger and no mana paid; an Aura goes around hexproof and ward. Zur must attack and survive. Bond, Blood (mana value 5) and Smothering Tithe (4) are out of reach |
| Necropotence | {B}{B}{B} | Hand, at your next end step | The top card of the library, one card per life paid | Any time you have priority | Not a search. It skips your draw step. Mana value 3: triggers her. A top-of-library tutor feeds it: the tutored card is the first one you exile |
| Read the Bones | {2}{B} | Hand | The top four cards (scry 2, then draw 2) | Sorcery | You lose 2 life. Mana value 3: triggers her. Use it to dig for a piece, not to find a named card |
| Cast Away Doubt, Tezzeret's Gambit | {2}{B}, {3}{U/P} | Hand | The top two cards (draw 2) | Sorcery | Cast Away Doubt deals 2 damage to each player (you too); Tezzeret's Gambit costs 2 life if {U/P} is paid with life. Both are mana value 3 or more and trigger her |
| Palantír of Orthanc | {3} | Scry 2 at your end step | The top two cards | Your end step | At your end step the target opponent either lets you draw or you mill X cards and they lose life equal to the total mana value. The mill can take a card you just put on top. Mana value 3: triggers her when cast |
| Expedition Map | {1}, then {2}, {T}, sacrifice | Hand | A land | Instant | Mana value 1: no trigger. Finds Urborg, Cabal Coffers or the missing colour; play it as your land drop |
| Urza's Cave | {3}, {T}, sacrifice | Battlefield, tapped | A land | Instant | Any land. The fetched land enters tapped, so it cannot tap this turn (Urborg's Swamp effect still applies at once) |
| Fetch lands (Flooded Strand, Polluted Delta, Marsh Flats) | {T}, 1 life, sacrifice | Battlefield | A basic or dual with the listed types | Instant | Strand finds Plains, Island, Tundra or Hallowed Fountain; Delta finds Island, Swamp, Underground Sea or Watery Grave; Flats finds Plains, Swamp, Scrubland or Godless Shrine. No Underground River, Adarkar Wastes, Caves of Koilos, Command Tower, Coffers or Urborg. Cracking one turns on Fatal Push's revolt |

Generous Gift finds nothing: it is an instant that destroys a permanent (its controller gets a 3/3), and its value here is a mana value 3 trigger on an opponent's turn.

**Choosing between them**

- **Speed.** Vampiric Tutor and Mystical Tutor are the only instant-speed tutors for an engine or kill piece. Cast them at the end step of the player before you and untap with the card as your draw. On your own turn the sorcery-speed ones put the card in hand now.
- **Draw into it.** A top-of-library tutor cast in response to a draw you control gives you that card as the draw: Y'shtola's end-step draw trigger, Loran's tap, Read the Bones, Cast Away Doubt, Tezzeret's Gambit, or a Rhystic Study trigger (if the opponent does not pay {1}).
- **Prefer the tutor that triggers her.** With Y'shtola out and the mana to spare, Diabolic Tutor, Ringsight and Idyllic Tutor do the same job as Demonic Tutor and also drain. Demonic Tutor is right when mana is tight, when you still have to cast the card you find, or when you want cheap bait for a counter.
- **A countered tutor still drains.** Her trigger is on cast, so a countered Diabolic Tutor, Ringsight or Idyllic Tutor has already dealt its 2 and gained its 2.
- **Palantír and Necropotence.** At your own end step, Palantír's mill can take the card you just put on top, so cast the tutor after its trigger resolves or at an opponent's end step. Under Necropotence you have no draw step: the tutored card comes only through a Necropotence activation, at your next end step.

#### What to fetch for the board

Life totals matter in three ways. Y'shtola draws a card at an end step when *any* player lost 4 or more life that turn, you included. Your tutors cost life (Vampiric Tutor 2, Anguished Unmaking 3, Read the Bones 2, a fetch land 1, Necropotence 1 per card). And an opponent at 4 or less dies to a cast-trigger plus one more drain. The tables below assume a four-player game with three opponents.

**Y'shtola not yet cast**

| Situation | Tutor | Target | Why, and what to watch for |
|---|---|---|---|
| Turn 1 to 3. The lands in hand cannot make {W}, {U} and {B} from three sources, no rock | Expedition Map (cast turn 1, crack it for {2}); Demonic Tutor | Command Tower, or an untapped dual in the missing pair (Tundra, Underground Sea, Scrubland); Arcane Signet if no land fits | She needs all three colours. Cast a rock from hand first if you hold one. Shock lands cost 2 life to enter untapped |
| Turn 3, three lands, no rock in hand, nothing else to do | Demonic Tutor ({1}{B}) with {1} left | Sol Ring | The two spells use all three mana, and turn 4 is four lands plus {C}{C}: she and two more mana. A rock in hand is better cast than tutored for |
| Turn 2 with spare mana and no play | Profane Tutor (suspend for {1}{B}) | Chosen when it resolves in your turn-4 upkeep: the rock or land that fixes colours, otherwise the answer or engine the table needs then | The search is on resolution, so decide with the turn-4 board. Do not suspend it into Drannith Magistrate or Opposition Agent |
| Turn 3 to 4, no land drop, two or three lands, no rock | Expedition Map, then Demonic Tutor | A land that fixes colours at two lands (Command Tower); Sol Ring at three | Mana first: the engine is useless if she comes down on turn 7 |
| Turn 4 to 5, she is castable, the hand has no noncreature spell of mana value 3 or more | Cast her first, then Diabolic Tutor, Ringsight or Idyllic Tutor with what is left | Rhystic Study or Smothering Tithe (Ringsight and Idyllic can fetch either); Propaganda if creatures are attacking | Cast her before the tutor so the tutor is her first trigger. Into open blue mana, use Mistrise Village first on her |
| She was countered or killed, recast costs {2} more each time | Diabolic Tutor, Ringsight (without her it needs a coloured creature), Demonic Tutor | Sanguine Bond | Bond works alone, so it is the first piece whatever the recast cost. Spend the turn on the engine and recast her when the mana is spare |

**Y'shtola out: building the drain**

| Situation | Tutor | Target | Why, and what to watch for |
|---|---|---|---|
| Neither piece, 5 to 8 mana, the table is not threatening | Diabolic Tutor now; or Vampiric Tutor at an opponent's end step | Sanguine Bond | Bond drains alone: every gain is a drain. With 9 mana, Diabolic Tutor then Bond in one turn is two triggers (4 to each opponent), so she draws at the end step |
| Bond out, no Blood; you will have 8 mana next turn | Vampiric Tutor at an opponent's end step | Exquisite Blood | You draw it and untap: Blood (5) plus a 3-mana trigger spell to start the loop. Mystical Tutor cannot find it. Do not tutor and cast Blood on the same turn unless you can still pay for the start |
| Blood out, no Bond | Diabolic Tutor, Ringsight or Vampiric Tutor | Sanguine Bond | Bond is the second piece, so it needs a life change after it resolves: keep a second 3-mana spell (Cast Away Doubt) for the same turn |
| Both out, no start in hand | Cast Diabolic Tutor, Ringsight or Idyllic Tutor as the start | A back-up: Exsanguinate or Teferi's Protection | The tutor's own cast trigger is the life change that starts the loop, so the spell you cast is the one that wins. Even a countered tutor drains |
| Both out, Y'shtola gone and not castable this turn | Demonic Tutor ({1}{B}) | Exsanguinate, cast for X = 1 ({1}{B}{B}) | A drain and a gain is a life change, so 5 mana starts the loop without her |
| Bond and Marauding Blight-Priest out, no Blood; each of two opponents at 10 or less, the Bond target at 30 or less, 8 mana | Mystical Tutor at the end step before | Exsanguinate for X = 6 | Her cast trigger and the Priest take 3 from each opponent; Exsanguinate takes 6 and the Priest 1 more (10 in all); the 18 gained goes to the Bond target (30 in all). Count your own table before you cast |
| 9 mana, no Bond, each opponent at 12 or less | Vampiric Tutor or Mystical Tutor at the end step before | Debt to the Deathless for X = 5 (nine mana: 10 to each opponent, plus 2 from her cast trigger) | Without Bond or Blood Debt only drains what you count, so cast it only when the total kills every opponent; otherwise land a piece first |

**Mana: Cabal Coffers, Urborg and Swamps**

Count lands (L), then check which pieces are out. With Urborg and Coffers both out you make 2L - 3 mana: 9 at six lands, 11 at seven, 13 at eight, 15 at nine. With Coffers alone it is L + S - 3, S being the real Swamps (the eight Swamps and Underground Sea, Watery Grave, Scrubland, Godless Shrine); Coffers is not itself a Swamp without Urborg. Debt costs X + 4 and takes 2X from each opponent; Exsanguinate costs X + 2 and takes X.

| Situation | Tutor | Target | Why, and what to watch for |
|---|---|---|---|
| Five or more lands; Urborg out, Coffers missing (or the reverse) | Expedition Map, Urza's Cave | The missing half | With both out, seven lands is 11 mana. Map's land is your land drop and can tap at once; Cave's land enters tapped, so Coffers fetched by Cave waits a turn |
| Seven or eight lands, Urborg and Coffers out (11 to 13 mana), Bond out, Y'shtola out | Mystical Tutor at the end step before; Demonic Tutor if you can still pay for the spell | Debt to the Deathless: X = 7 at 11 mana (14 to each opponent), X = 9 at 13 mana (18 to each) | Two {W} must come from lands (Coffers makes only {B}). Without two white sources, cast Exsanguinate instead (X = 9 at 11 mana). Mistrise Village first, a counter behind it |
| Ordinary mana only (no Coffers or Urborg), 7 to 10 mana | Mystical Tutor or Vampiric Tutor at the end step before | Debt (better from about seven mana) or Exsanguinate if you lack {W}{W} | Debt gets 2 per X at 4 extra mana; Exsanguinate gets 1 per X at 2 extra. Count what you actually kill |
| Hand colours do not fit: {W}{W} for Debt, {B}{B}{B} for Necropotence | Urza's Cave, Expedition Map, Demonic Tutor | The missing coloured land: a white source for Debt; for Necropotence any land once Urborg is out, otherwise a Swamp or black dual | With Urborg out every land taps for {B}, so Necropotence is easy; Debt still wants two white lands |
| Eight or more lands and few spells | Diabolic Tutor, Ringsight | Exsanguinate or Debt; the Coffers lands if they are missing | The deck is built to turn a flood into one big spell. Urborg Syphon-Mage also turns a spare land into 2 damage and 2 life |

**Blink engine: Displacer Kitten, Conjurer's Closet, Teleportation Circle, Restoration Angel, Loran**

| Situation | Tutor | Target | Why, and what to watch for |
|---|---|---|---|
| Displacer Kitten out with Loran (or Skyclave Apparition, Orcish Bowmasters) on the battlefield and an opposing artifact or enchantment worth removing | Mystical Tutor, Vampiric Tutor, Demonic Tutor | Whatever the board needs | Kitten blinks on *any* noncreature spell, so a one-mana tutor is a free blink: Loran destroys another artifact or enchantment. Blinking Skyclave Apparition gives its owner a token each time, so blink it only when the second exile is worth a token |
| Loran out, no Kitten, a stax piece to keep removing | Ringsight, Demonic Tutor; Idyllic Tutor for the Circle | Displacer Kitten; or Teleportation Circle ({3}{W}) | The Circle blinks one creature or artifact at each of your end steps (a trigger spell as well). Conjurer's Closet ({5}) blinks one creature and Ringsight cannot find it; take it only with a spare five mana |
| Skyclave Apparition or Orcish Bowmasters on the battlefield, one blink wanted at instant speed | Ringsight, Demonic Tutor | Restoration Angel ({3}{W}, flash) | It blinks a non-Angel creature at the end of an opponent's turn, or in response to removal on Y'shtola |
| The table has an artifact or enchantment that decides the game (Basalt Monolith, Winter Orb, Isochron Scepter, Rule of Law) and neither Loran nor Skyclave is out | Demonic Tutor, Diabolic Tutor, Ringsight | Loran of the Third Path (any artifact or enchantment); Skyclave Apparition (any nonland, nontoken permanent of mana value 4 or less) | Both are creatures cast at sorcery speed (neither has flash). For instant speed use Generous Gift or Anguished Unmaking, which Mystical Tutor can find |

**An opponent combo about to win**

| Situation | Tutor | Target | Why, and what to watch for |
|---|---|---|---|
| A combo is one piece from winning on their next turn, and you hold a counter | None: pass with the mana up (Counterspell {U}{U}, Dovin's Veto {W}{U}, Force of Will) | Vampiric Tutor or Mystical Tutor only after they pass the danger point, at their end step | Mana spent on a tutor is mana not spent on the counter. Dovin's Veto stops noncreature spells only; no counter stops an ability (Walking Ballista, Urza's activation) |
| The same, and your hand has no counter | Demonic Tutor on your turn with the counter mana left over | Dovin's Veto against a noncreature piece; Counterspell or Force of Will against a creature piece | The tutor must leave the counter's mana untapped: {1}{B} plus {W}{U} is 4 mana, or {1}{B} plus {U}{U}. Keep a second blue card in hand to pitch to Force of Will |
| A combo spell is on the stack, a Rhystic Study trigger is waiting above it, one blue mana is open | Mystical Tutor in response to the trigger | Dovin's Veto or Counterspell | If the opponent does not pay {1}, you draw the counter before the combo resolves and can cast it (3 mana in all). If they pay, you have only moved a card to the top |
| The combo piece is on the battlefield and the table cannot kill it by destroying (Heliod, Sun-Crowned; undying creatures; a piece with indestructible) | Mystical Tutor, Idyllic Tutor, Demonic Tutor | Anguished Unmaking (exile, costs 3 life), Prayer of Binding ({3}{W}, flash, exile and 2 life for you), Skyclave Apparition (mana value 4 or less) | Generous Gift destroys, which indestructible and undying pieces survive. Anguished Unmaking and Prayer are mana value 3 or more: they trigger her on an opponent's turn |
| A creature carries the combo (Kinnan, a mana creature) before their turn | Demonic Tutor, Vampiric Tutor | Fatal Push (mana value 2 or less, or 4 with revolt), Go for the Throat, Path to Exile; Orcish Bowmasters for a 1-toughness piece | These are mana value 1 or 2 and do not trigger her, but they cost one or two mana and leave a counter up. Fatal Push cannot hit mana value 3 or more without revolt, and Go for the Throat cannot hit a Spellskite or Walking Ballista |
| A sweeper or Cyclonic Rift is coming and Bond and Blood are on your board | Mystical Tutor, Demonic Tutor | Teferi's Protection | It phases your permanents out and keeps your life from changing; hold it for a turn you are not trying to win (Bond and the Priest fire on nothing under it) |

**Wide boards and the race**

| Situation | Tutor | Target | Why, and what to watch for |
|---|---|---|---|
| Many small creatures (tokens, Orcs, elves) across the table, little of yours | Demonic Tutor; Mystical Tutor at the end step before | Toxic Deluge ({2}{B}, pay X life) | X = 3 kills toughness 3 or less and leaves Y'shtola (2/4) alive; X = 4 kills her. It triggers her, and an X of 0 is a 3-mana trigger spell. Watch your life total |
| A few large creatures (Dragons) and your board is Y'shtola and enchantments | Mystical Tutor at the end step before | Austere Command ({4}{W}{W}) | Choose two modes. The "mana value 4 or greater" mode destroys Y'shtola (mana value 4), so cast Teferi's Protection first or accept the recast; the "mana value 3 or less" mode destroys Esper Sentinel and the Priest. Farewell exiles instead (see Keeping the table off you) |
| Their attackers deal lethal to you next turn and there is no tax out | Idyllic Tutor; Demonic Tutor | Propaganda or Sphere of Safety (taxes every attacker); Teferi's Protection with {2}{W} open | The tax stops a wide attack when they lack the mana; Teferi's Protection stops all of it for one turn but locks your life total |
| Their attackers kill you in two turns; yours kill them in one | Mystical Tutor at the end step before | Exsanguinate or Debt to the Deathless | Count: each opponent loses X (2X for Debt), plus 2 from her, plus 1 per gain from the Priest, and one opponent loses the gain through Bond. Do not hold the wipe for a bigger board |
| One big creature is the only attacker | Zur's attack; Idyllic Tutor | Arrest or Bound in Silence (by Zur: no targeting) | Arrest also stops activated abilities. Both are mana value 3 and trigger her if cast (not if Zur puts them in) |

**Stax and hate pieces**

| Situation | Tutor | Target | Why, and what to watch for |
|---|---|---|---|
| Winter Orb or Static Orb out: one land (or two permanents) untaps | One-mana tutors (Vampiric, Mystical), Expedition Map; the rocks keep untapping | The removal that your untapped sources can pay: Generous Gift or Loran (both 3 mana) | Under Winter Orb Sol Ring and the Talismans still untap. Do not spend the turn's one land on a tutor for a five-drop |
| Rule of Law or Eidolon of Rhetoric: one spell per player per turn | Vampiric or Mystical Tutor at an opponent's end step | Loran of the Third Path first, then the kill piece | Each player gets one spell each turn, theirs included, so an instant on their turn does not use your own turn's spell. Remove the lock with Loran before using the one spell on a piece that cannot resolve |
| Drannith Magistrate out and Y'shtola is in the command zone | Demonic Tutor, Vampiric Tutor | Fatal Push (the Magistrate has mana value 2) | Do not suspend Profane Tutor: it would sit in exile uncastable |
| Opposition Agent out | None. Do not search at all | Answer the Agent from hand: Path to Exile, Go for the Throat | Every search, fetch lands, Expedition Map, Urza's Cave and Zur's attack included, gives the Agent's controller the card |
| Narset, Parter of Veils out: you cannot draw more than one card each turn | Demonic Tutor, Diabolic Tutor, Ringsight, Idyllic Tutor (cards go to hand) | The piece you need | Your draw step is your one draw on your own turn, so Loran, Rhystic Study and the draw spells give nothing then; on an opponent's turn you still have one draw. Cards in hand beat cards on top |
| Grand Abolisher on the table | Sorcery-speed tutors on your turn; instant ones at another opponent's end step | The piece you need | Abolisher stops your spells and the activated abilities of your artifacts, creatures and enchantments on its controller's turn only. Do not plan an end-step tutor in that player's turn |

**Graveyards**

| Situation | Tutor | Target | Why, and what to watch for |
|---|---|---|---|
| The table recurs from graveyards (Animate Dead, Reanimate, Unearth, Entomb, undying) | Mystical Tutor, Demonic Tutor | Farewell, graveyard mode alone, or Skyclave Apparition or Anguished Unmaking on the returned creature | Farewell exiles every graveyard, yours too; choose only the modes you need so it does not exile Y'shtola, Bond or your rocks. Cast Teferi's Protection first if you choose more |
| You want a card from your own graveyard | None | Nothing in the list returns a card from your graveyard | Do not discard a key card to Syphon-Mage, and note that Necropotence exiles anything you discard |

**Low life**

| Situation | Tutor | Target | Why, and what to watch for |
|---|---|---|---|
| Your life is 10 or less and the table can hit you | Mystical Tutor, Demonic Tutor, Diabolic Tutor, Ringsight, Idyllic Tutor (none costs life) | Sheoldred, the Apocalypse (2 life per draw, deathtouch); Propaganda or Sphere of Safety; Debt or Exsanguinate for a large gain | Do not use Vampiric Tutor, Necropotence, a fetch land, Read the Bones or Anguished Unmaking at this life total. Teferi's Protection saves a turn, not the race |
| Life above about 30 and no card flow | Vampiric Tutor, Demonic Tutor | Necropotence (or Zur's attack for it) | Cast it with a healthy life total: the drains refill you. Keep above the table's burn range (Yawgmoth's drain, a Dragon table) |
| One opponent is at 4 or less, she is out | Demonic Tutor, Diabolic Tutor | Cast Away Doubt | Her trigger 2 plus its 2 damage is 4 to every opponent, in 3 mana. An opponent at 2 or less dies to any noncreature spell of mana value 3 or more. It also deals 2 to you |

**Turn windows and the end step**

| Situation | Tutor | Target | Why, and what to watch for |
|---|---|---|---|
| End of the opponent's turn right before yours, mana open | Vampiric Tutor (any card), Mystical Tutor (instant or sorcery) | The kill piece: Bond, Blood, Debt, Exsanguinate, or the missing Coffers land | You untap and draw it with a full turn of mana. The cheapest time to tutor; a tapped-out turn can still do it |
| Your own turn, tapped low, and next turn is the kill | Vampiric Tutor, Mystical Tutor at your own end step, after Palantír's trigger resolves | Debt, Exsanguinate or Torment of Hailfire | Palantír's mill takes the top cards of your library at your end step. Cast it after the trigger, or at an opponent's end step |
| Her draw trigger is on the stack at an end step (a player lost 4 or more life) | Vampiric Tutor or Mystical Tutor in response | The card you need next | The draw gets the tutored card. Cast it before the trigger resolves |
| Necropotence out (no draw step) | Vampiric Tutor, then pay 1 life | The kill piece | The tutored card is the first one the next activation exiles, and it arrives at your next end step. Tutor and activate a turn early |
| Turn 7 or later, 9 or more mana, the table is not close | Mystical Tutor at the end step before | Debt, Exsanguinate or Torment of Hailfire, whichever your mana and colours can pay | Torment is a late mana sink: opponents choose a sacrifice or a discard, so it works best against a table with few permanents and cards. Count before you commit |

#### What to fetch first

1. **Turns 1 to 3, Y'shtola not yet out:** the colour or rock that lets you cast her on turn 3 or 4 (Command Tower, Arcane Signet, a Talisman, Sol Ring). Rhystic Study or Smothering Tithe if the mana is fine.
2. **A game-ending threat on the table:** the answer to it before any engine piece. Dovin's Veto or Counterspell for a spell, Anguished Unmaking for a permanent, Teferi's Protection for a wipe or a lethal attack.
3. **Y'shtola out, neither piece:** Sanguine Bond, then Exquisite Blood.
4. **Bond out, Blood missing:** Exquisite Blood, from an opponent's end step, with a 3-mana trigger spell for the same turn.
5. **Both out:** do not fetch a start: cast Diabolic Tutor, Ringsight or Idyllic Tutor as the start. Without her, Exsanguinate for X = 1.
6. **Five or more lands:** Urborg or Cabal Coffers, whichever is missing, then Debt to the Deathless (with {W}{W}) or Exsanguinate.
7. **Creatures attacking you:** Propaganda or Sphere of Safety, Toxic Deluge or Austere Command, then Teferi's Protection to survive a turn.
8. **Nothing above applies:** an engine (Rhystic Study, Smothering Tithe, Necropotence), in that order of safety.

#### Tutor mistakes

1. Looking for Y'shtola. She is in the command zone; nothing in the library finds her.
2. Using Mystical Tutor for Bond, Blood, Loran or a land. It finds only instants and sorceries.
3. Casting a top-of-library tutor at your own end step with Palantír out, then letting its mill take the card; or casting one under Necropotence and waiting for a draw step that does not come.
4. Casting Vampiric Tutor, Read the Bones, Anguished Unmaking or a fetch land at a life total the table can kill, or casting Necropotence in front of Yawgmoth's drain.
5. Casting Exquisite Blood or Sanguine Bond on the turn you tutor for it when you cannot also pay for a start. Tutor at the end step and cast it next turn with the start in hand.
6. Using Demonic Tutor (mana value 2, no drain) when Y'shtola is out and you have the mana for Diabolic Tutor, Ringsight or Idyllic Tutor.
7. Tutoring for Ringsight's target with no Y'shtola and no coloured creature, or for a colourless card (Sol Ring, Palantír, Closet) or a land.
8. Expecting Zur to find Bond, Blood, Polluted Bonds or Smothering Tithe (mana value 4 or more), or expecting the search without an attack that survives.
9. Tutoring into open counter mana for a card you cannot cast that turn or the next. A sorcery tutor is useful as bait, not as the plan.
10. Searching under Opposition Agent: fetch lands, Map, Cave, Zur and every tutor give the Agent's controller the card. Remove the Agent first.
11. Suspending Profane Tutor into Drannith Magistrate or Opposition Agent, or forgetting it can only be suspended, not cast from hand.
12. Planning Debt to the Deathless without counting {W}{W}. Coffers makes only {B}, so the two white must come from lands; count them first.

### Protecting the win turn

Y'shtola is the engine, but the win turn is Bond and Blood on the battlefield and Debt or Exsanguinate resolving. Order of protection:

1. **Mistrise Village first** ({U}, {T}: the next spell you cast this turn can't be countered). It beats Force of Will, Counterspell, Mana Drain and Swan Song on the spell that matters. It is one tap, so it costs the Village's mana too.
2. **A counter behind it:** Dovin's Veto (noncreature, can't be countered itself), Counterspell ({U}{U}), Force of Will (pay 1 life and exile a blue card). Bait with a cheaper spell first if the opponent has one counter and you have two threats.
3. **Cast the kill when the counters are tapped.** Mana Drain and Counterspell need {U}{U}, so the best window is the turn after the blue players tapped out on their own turn.
4. **Her trigger resolves even if the spell is countered.** A countered Debt still drains 2 from each opponent and gains 2 for Bond.
5. **Against Cyclonic Rift** (Kinnan, Urza, Zur and Miirym all run it; overload at seven mana bounces your whole board): don't rely on a long board. Cast the second piece of Line 1 when the loop can start at once.

**Protecting Y'shtola herself:**

- **Dovin's Veto and Force of Will** stop the removal spell (Veto only a noncreature spell, not an ability).
- **Restoration Angel** (flash, {3}{W}) blinks her out of removal.
- **Displacer Kitten:** cast any noncreature spell in response to removal, and the Kitten blinks her; the removal fizzles (a mana value 3 spell also triggers her).
- **Champion's Helm** ({3}, equip {1}): +2/+2 and hexproof on a legendary creature. It makes her 4/6 and untargetable. It doesn't stop a wipe.
- **Teferi's Protection** ({2}{W}, instant): until your next turn your life total can't change, you have protection from everything, and all your permanents phase out. It saves her and the board from a wipe or a lethal attack. While it lasts you gain nothing, so Bond and the Priest don't fire: use it to survive a turn, not to win one.
- **Wipes** are the real danger (Damnation and The Meathook Massacre in Yawgmoth's list, Supreme Verdict, Toxic Deluge and Winds of Rath in Zur's, Blasphemous Act in Miirym's). Only Teferi's Protection saves her: Restoration Angel and the Helm do not.

### Interaction on opponents' turns

Any noncreature spell of mana value 3 or more cast on an opponent's turn triggers her. Two of them, or one plus a drain, draw you a card at that end step.

| Card | Cost | Use it on |
|---|---|---|
| Force of Will | {3}{U}{U}, or 1 life and a blue card | The combo piece or the wipe that wins. A mana value 5 spell, so it triggers her even when pitched |
| Counterspell | {U}{U} | Any spell: a combo piece, a tutor for the combo, a wipe, a creature. It cannot stop an ability (Walking Ballista, Urza's activation) |
| Dovin's Veto | {W}{U} | Any noncreature spell (Basalt Monolith, a Freed from the Real, a removal spell on Y'shtola, Cyclonic Rift). Can't be countered, so a Mana Drain or Counterspell cannot stop it |
| Path to Exile | {W} | A creature that decides the game (Kinnan, Urza, Yawgmoth, Zur, a Dragon; Miirym has ward {2}); gives the opponent a basic land |
| Fatal Push | {B} | Mana value 2 or less (Kinnan, a mana creature, Spellskite); mana value 4 or less with revolt (a fetch land or Treasure sacrificed, Expedition Map, Urza's Cave) |
| Go for the Throat | {1}{B} | A nonartifact creature, such as Yawgmoth or Zur. Not Spellskite or Walking Ballista |
| Generous Gift | {2}{W} | Any permanent (Winter Orb, Basalt Monolith, a lock enchantment, a creature); the owner gets a 3/3. Mana value 3, so it triggers her |
| Anguished Unmaking | {1}{W}{B} | Any nonland permanent, exiled (undying, indestructible, or a loop piece); costs you 3 life. Triggers her |
| Your Fate Ends Here | {2}{W} | A creature or planeswalker with mana value 3 or more (Urza, Yawgmoth, Narset, a Dragon). Triggers her; surveil 1 |
| Prayer of Binding | {3}{W}, flash | Exile any nonland permanent until it leaves; you gain 2 (a gain for Bond and the Priest). Cast it at an end step for a trigger |
| Orcish Bowmasters | {1}{B}, flash | A mana creature or 1-toughness piece, in response to the combo, or at their end step |
| Teferi's Protection | {2}{W} | A wipe, or a lethal attack on you |
| Restoration Angel | {3}{W}, flash | Blink Y'shtola in response to removal |

Path to Exile and Fatal Push don't trigger her: save them for a threat that decides the game. When a trigger helps, use Generous Gift, Anguished Unmaking or Your Fate Ends Here, which are a removal spell and a trigger in one.

### Matchups (Tier 5)

The sims show her losing about a third of t5 games to the Kinnan and Urza combos, and about half to those two, Zur's combo and Miirym's combat together. Games last about nine rounds, so the combo decks win before the Coffers turn. She wins about a quarter of t5 games (26.5% before the latest swaps; the current list is not measured). The sims do not break Yawgmoth out.

| Opponent | Their threat | Your plan | Cards that matter |
|---|---|---|---|
| Kinnan, Bonder Prodigy (Simic mana combo) | Kinnan + Basalt Monolith (unbounded mana), Freed from the Real or Pemmin's Aura on a mana creature; the mana goes into Walking Ballista or Thrasios. Turn 3 to 5 | You cannot race it. Kill the mana creatures early and hold a counter for the Monolith, an Aura or Kinnan. Their free counters (Force of Will, Force of Negation, Fierce Guardianship, Pact of Negation) can answer your key spell: bait with a lesser one first | Orcish Bowmasters on Bloom Tender or a mana elf; Fatal Push, Go for the Throat or Path on Kinnan (mana value 2); Dovin's Veto, Counterspell, Force of Will on Monolith or an Aura; Loran or Skyclave Apparition on the Monolith. Spellskite redirects targeted removal; Cyclonic Rift and Swan Song hit Bond and Blood |
| Urza, Lord High Artificer (mono-blue artifacts, stax) | Isochron Scepter + Dramatic Reversal, Basalt Monolith + Rings of Brighthearth or Forensic Gadgeteer, Hullbreaker Horror; Winter Orb, Static Orb and Back to Basics hold your mana while its artifacts untap. The fastest deck in the pool, turns 4 to 6 | Rocks and cheap plays: under Winter Orb you untap one land, and under Static Orb only two permanents. Get the Orb or the Monolith off the table before the combo. Keep a counter for the combo piece; Hullbreaker Horror can't be countered, so kill it or the pieces that feed it | Loran of the Third Path (destroy an artifact on entry and again on every blink), Skyclave Apparition (Monolith, Orb, Rings, Scepter all have mana value 4 or less), Generous Gift, Austere Command or Farewell in artifacts mode (they ignore Padeem's hexproof but hit your own rocks), Counterspell and Force of Will on Scepter. Padeem gives its artifacts hexproof, which stops every targeted answer here: kill Padeem (Go for the Throat, Path) or use a wipe. Narset stops her extra draws |
| Zur the Enchanter (Esper control) | Enchantment locks (Rule of Law, Eidolon of Rhetoric: one spell per turn; Drannith Magistrate; Opposition Agent; Narset; Grand Abolisher), a counter wall (Force of Will, Mana Drain, Fierce Guardianship, Counterspell, Dovin's Veto, Swan Song) and Heliod, Sun-Crowned + Walking Ballista as the kill. Phyrexian Unlife with Solemnity | Treat each spell as a trigger and a counter bait. Kill Zur early (Arrest or Bound in Silence stops the attack and the search). Remove the locks with Loran, Skyclave, Anguished Unmaking | Skyclave Apparition, Prayer of Binding or Anguished Unmaking on Heliod (indestructible: Generous Gift won't kill it); Fatal Push on a Walking Ballista (mana value 0 on the battlefield; it pings in response); Loran on the Rule of Law; Drannith Magistrate stops the commander and suspended Profane Tutor from being cast; Grand Abolisher shuts off your instants on their turn; Opposition Agent takes your tutors |
| Miirym, Sentinel Wyrm (Temur Dragons) | Each Dragon is copied by Miirym (flying, ward {2}). Terror of the Peaks, Scourge of Valkas and Twinflame Tyrant turn each arrival into burn; Wrathful Red Dragon and Blasphemous Act. Wins in combat | Fort up early. Propaganda and Sphere of Safety tax every copy. Wipe the Dragons with Toxic Deluge, Austere Command (creatures with mana value 4 or more) or Farewell: Terror of the Peaks makes spells that target it cost 3 more life, and a wipe doesn't target. Counter the first big Dragon | Propaganda, Sphere of Safety, Arrest and Bound in Silence on a Dragon; Austere Command, Farewell, Toxic Deluge; Counterspell and Force of Will on Miirym or Terror (Cavern of Souls makes its Dragons uncounterable). Cyclonic Rift, Chaos Warp, Beast Within, Blasphemous Act answer her |
| Yawgmoth, Thran Physician (mono-black aristocrats) | Yawgmoth + two undying creatures (Geralf's Messenger, Butcher Ghoul) or Gravecrawler + Phyrexian Altar loops into cards and drain via Blood Artist, Zulaport Cutthroat, Meathook Massacre. Damnation, Toxic Deluge and Meathook Massacre as wipes | The only t5 deck with no counterspells: you can tap out for Bond, Blood and the Priest. Exile pieces instead of destroying them (undying brings creatures back). Farewell's graveyard mode answers Animate Dead, Reanimate, Unearth and Entomb | Anguished Unmaking, Path and Skyclave Apparition (exile), Go for the Throat on Yawgmoth (protection from Humans only), Loran on Bitterblossom or Grave Pact, Counterspell and Force of Will on Yawgmoth; Y'shtola's lifegain offsets their drain |

- **The Kinnan and Urza tables end the game before the drain:** hold up Counterspell, Veto or Force of Will from turn 3 on rather than tapping out for Smothering Tithe or a rock.
- **Against Zur's deck,** Rule of Law and Eidolon of Rhetoric allow one spell a turn: make it your biggest trigger spell, and remove the lock with Loran or Skyclave Apparition first.
- **Against Miirym,** the table's creature total decides it. Do not hold the wipe for a bigger board: Miirym copies each Dragon before you can answer it.

### After a stopped attempt

| What happened | Rebuild |
|---|---|
| Bond or Blood countered | Neither card has a second copy or any recursion. If the other is out, Line 2 (an X drain) with the Priest is the kill; tutor for Debt to the Deathless or Exsanguinate |
| Bond or Blood bounced (Cyclonic Rift) | Recast it. Cast the one that works alone (Bond) first. Don't cast both into a Rift overload |
| Bond or Blood removed for good | The X drains and Y'shtola still win; use Palantír, Polluted Bonds and the grind |
| Debt or Exsanguinate countered | Y'shtola's cast trigger still drained. Torment of Hailfire and the other X drain are still in the library: tutor for them |
| Y'shtola killed or exiled | Recast her for {1}{W}{U}{B} plus {2} per previous cast. Bond, Blood and the X drains win without her, so a third recast can wait: spend the turn on the engine instead |
| Creature wipe on your board | Enchantments (Bond, Blood, Propaganda, Tithe) survive it; the creatures die (the Priest, Sphinx, Sheoldred, Loran, Kitten). Recast Y'shtola and the Priest first. Keep Teferi's Protection up next time |

- After a stopped attempt the table has seen your plan: keep a counter or Teferi's Protection up on their turn before you try again.

### Life management

| Source | Cost |
|---|---|
| Fetch lands (Flooded Strand, Polluted Delta, Marsh Flats) | 1 each |
| Shock lands (Hallowed Fountain, Watery Grave, Godless Shrine) | 2 to enter untapped |
| Caves of Koilos, Adarkar Wastes, Underground River | 1 per coloured tap |
| Talisman of Progress, Talisman of Hierarchy | 1 per coloured tap |
| Vampiric Tutor | 2 |
| Read the Bones | 2 |
| Cast Away Doubt | 2 (to each player) |
| Tezzeret's Gambit | 2 if {U/P} is paid with life |
| Anguished Unmaking | 3 |
| Force of Will (alternative cost) | 1 |
| Toxic Deluge | X (your choice) |
| Necropotence | 1 per card |
| Terror of the Peaks (opponent's) | 3 on every spell that targets it |

| Gain | Amount |
|---|---|
| Y'shtola trigger | 2 per spell of mana value 3 or more |
| Debt to the Deathless, Exsanguinate, Urborg Syphon-Mage | Whatever they drain (one event) |
| Exquisite Blood | Whatever an opponent loses |
| Sheoldred, the Apocalypse | 2 per card you draw |
| Prayer of Binding | 2 |
| Polluted Bonds | 2 per opponent land drop |

**Budget.** Pay life for tempo early (fetches, a shock, Vampiric Tutor) but keep your life above about 25 before you cast Necropotence, and above the table's burn range against Miirym (Scourge of Valkas, Terror of the Peaks) and Yawgmoth (Blood Artist, Zulaport Cutthroat, Meathook Massacre). Every Y'shtola trigger refills 2; with Bond the same life is damage. Teferi's Protection stops your life from changing, so don't use it in a turn where you need the gain.

### Common mistakes

1. Casting the second piece of Bond + Exquisite Blood and expecting Y'shtola's cast trigger to start the loop. It resolves while the piece is on the stack. You need a second life change after both are out.
2. Casting Exquisite Blood or Sanguine Bond into open blue mana without Mistrise Village or a counter behind it, when waiting a turn is cheap.
3. Casting a spell of mana value 1 or 2 and expecting a trigger (Counterspell, Dovin's Veto, Orcish Bowmasters, Go for the Throat, the tutors at 1 or 2). Spend the turn's mana on a three-plus spell when you can.
4. Casting Teferi's Protection to win a turn. Your life can't change under it, so Bond, the Priest and the X drains' gain do nothing.
5. Choosing the wrong Farewell modes: creatures with Y'shtola out, enchantments with Bond and Blood out, artifacts with the rocks out.
6. Blinking Skyclave Apparition for value without noticing it hands the owner an X/X token each time.
7. Casting X drains without counting: each opponent loses X (or 2X), plus 2 from her, plus 1 per gain from the Priest, and the gain goes to one opponent through Bond. Don't cast Debt at a low X when Cabal Coffers is a turn away.
8. Tutoring or cracking Expedition Map or Urza's Cave under Opposition Agent.
9. Pinging a face with Orcish Bowmasters when a mana creature or combo piece (Bloom Tender, Birds of Paradise, a Walking Ballista at X = 1) is on the table.
10. Casting Necropotence at a low life total with Yawgmoth's drain or a Dragon table in front of you.

## Mana

The list is exactly 100 cards.

37 lands (18 nonbasic, 6 Plains, 5 Island, 8 Swamp) plus five rocks: Sol Ring, Arcane Signet, Talisman of Progress, Talisman of Hierarchy, Coalition Relic. **Serra's Sanctum** taps for {W} per enchantment you control, and that includes the removal Auras on opponents' creatures (you control the Aura), so Arrest or Bound in Silence also adds a mana. **Urborg, Tomb of Yawgmoth** makes every land a Swamp in addition to its other types, so every land can tap for {B}: it fixes Necropotence and Sheoldred, the Apocalypse and replaces a basic Swamp at no cost (the fetches can't find it, but they still have 8 Swamps). **Cabal Coffers** ({2}, {T}: add {B} for each Swamp you control): with Urborg out every land is a Swamp, so L lands make 2L - 3 mana for Exsanguinate, Debt to the Deathless, Torment of Hailfire or two big spells in a turn. **Urza's Cave** taps for {C}, or for {B} with Urborg out; {3}, {T}, sacrifice it to fetch Urborg, Coffers or the colour the hand is missing. **Mistrise Village** taps for {U}, enters tapped here (the list has no Mountain or Forest), and its {U}, {T} ability makes the next spell uncounterable.

| Measure | Count | Note |
| --- | --- | --- |
| White lands | 14 | 6 Plains plus Serra's Sanctum, Adarkar Wastes, Caves of Koilos, Command Tower, Tundra, Scrubland, Hallowed Fountain, Godless Shrine |
| Blue lands | 13 | 5 Island plus Mistrise Village, Adarkar Wastes, Command Tower, Underground River, Tundra, Underground Sea, Hallowed Fountain, Watery Grave |
| Black lands | 17 (all 37 with Urborg out) | 8 Swamp plus Urborg, Tomb of Yawgmoth, Cabal Coffers (with a Swamp out), Caves of Koilos, Command Tower, Underground River, Underground Sea, Scrubland, Watery Grave, Godless Shrine |
| Fetch lands | 3 | Flooded Strand (W/U), Polluted Delta (U/B), Marsh Flats (W/B): each finds a basic or any dual or shock of its two types, for 1 life |
| Colourless only | 1 | Urza's Cave ({B} with Urborg out) |
| Always enter tapped | 1 | Mistrise Village |
| Sometimes enter tapped | 3 | the shocks (Hallowed Fountain, Watery Grave, Godless Shrine), unless you pay 2 life |

Black is the main colour (Necropotence's {B}{B}{B}, the X drains' {B}{B}), so it has the most sources. Blue has the fewest. Most blue spells cost a single {U}; Counterspell's and Force of Will's {U}{U} are the exceptions, and Force of Will can be cast for free. Each fetch finds the colour the hand is short of, and cracking one turns on Fatal Push's revolt (destroy a creature with mana value 4 or less).

**Curve.** Profane Tutor at 0 (suspended for {1}{B}), one-drops 8, two-drops 10, three-drops 22, four-drops 13 (counting Y'shtola), five 6, six 3. Average mana value about 3.1, with the X spells counted at X = 0. It is a slow curve by design: Y'shtola pays you for mana value 3 and up.

**Card types.** 14 creatures (plus Y'shtola; Esper Sentinel is an artifact creature, counted here), 13 enchantments, 9 artifacts, 12 instants, 14 sorceries, no planeswalkers.

**Finding cards.**
- **Engines:** Necropotence, Consecrated Sphinx, Mystic Remora, Esper Sentinel, Notion Thief, and her own end-step draw (Orcish Bowmasters turns the opponents' extra draws into damage and Orcs).
- **Mana:** Smothering Tithe's Treasures, on top of the rocks.
- **Draw spells:** Read the Bones, Tezzeret's Gambit, Cast Away Doubt, Loran's tap; Rhystic Study as the engine.
- **Tutors:** Demonic Tutor, Diabolic Tutor, Ringsight, Profane Tutor (suspend 2), Vampiric Tutor (any card, on top), Idyllic Tutor (an enchantment), Mystical Tutor (an instant or sorcery, on top), and Zur's attacks.
- **Land tutors:** Expedition Map and Urza's Cave (Urborg or Cabal Coffers, or the missing colour).

**Interaction.**
- **Counterspells (three):** Counterspell, Dovin's Veto, Force of Will.
- **Spot removal spells (six):** Path to Exile, Fatal Push, Go for the Throat, Generous Gift, Anguished Unmaking, Your Fate Ends Here.
- **Removal on permanents:** Skyclave Apparition, Loran of the Third Path (an artifact or enchantment, again on every blink), Prayer of Binding.
- **Removal Auras (two):** Arrest, Bound in Silence.
- **Board wipes (three):** Austere Command, Farewell, Toxic Deluge.
- **Protection:** Teferi's Protection, Restoration Angel, Champion's Helm.

## Bracket and Rule 0

**Bracket 4,** with fourteen Game Changers: **Necropotence**, **Notion Thief**, **Consecrated Sphinx**, **Smothering Tithe**, **Rhystic Study**, **Demonic Tutor**, **Mystical Tutor**, **Vampiric Tutor**, **Grand Arbiter Augustin IV**, **Force of Will**, **Farewell**, **Teferi's Protection**, **Orcish Bowmasters** and **Serra's Sanctum**. It has one two-card infinite combo (Exquisite Blood + Sanguine Bond), no mass land destruction and no extra turns.

Disclose before the game:

- **Necropotence**, **Notion Thief**, **Consecrated Sphinx**, **Smothering Tithe**, **Rhystic Study**, **Demonic Tutor**, **Mystical Tutor**, **Vampiric Tutor**, **Grand Arbiter Augustin IV**, **Force of Will**, **Farewell**, **Teferi's Protection**, **Orcish Bowmasters** and **Serra's Sanctum** (the fourteen Game Changers).
- **Free counterspell and board wipe:** Force of Will (pitch a blue card) and Farewell (exiles creatures, artifacts, enchantments and graveyards as you choose).
- **Seven tutors** (Demonic, Diabolic, Ringsight, Profane, Idyllic, Mystical, Vampiric), Zur's enchantment search, and Expedition Map / Urza's Cave for the Coffers lands.
- **The win is a drain:** Y'shtola pings the table on every big spell, and Exsanguinate or Debt to the Deathless with Sanguine Bond can take out two players in one turn.
- **Two-card combo:** Exquisite Blood + Sanguine Bond drains the whole table from any life change.

Re-check the official list at https://commanderbrackets.com/faq before an event.

## Decklist by type (100)

**Commander (1).** Y'shtola, Night's Blessed

**Creatures (14).** Consecrated Sphinx, Displacer Kitten, Esper Sentinel, Grand Arbiter Augustin IV, Loran of the Third Path, Marauding Blight-Priest, Notion Thief, Orcish Bowmasters, Restoration Angel, Sheoldred, the Apocalypse, Skyclave Apparition, Urborg Syphon-Mage, Venat, Heart of Hydaelyn // Hydaelyn, the Mothercrystal, Zur the Enchanter

**Enchantments (13).** Arrest, Bound in Silence, Exquisite Blood, Mystic Remora, Necropotence, Polluted Bonds, Prayer of Binding, Propaganda, Rhystic Study, Sanguine Bond, Smothering Tithe, Sphere of Safety, Teleportation Circle

**Artifacts (9).** Arcane Signet, Champion's Helm, Coalition Relic, Conjurer's Closet, Expedition Map, Palantír of Orthanc, Sol Ring, Talisman of Hierarchy, Talisman of Progress

**Instants (12).** Anguished Unmaking, Counterspell, Dovin's Veto, Fatal Push, Force of Will, Generous Gift, Go for the Throat, Mystical Tutor, Path to Exile, Teferi's Protection, Vampiric Tutor, Your Fate Ends Here

**Sorceries (14).** Austere Command, Cast Away Doubt, Debt to the Deathless, Demonic Tutor, Diabolic Tutor, Exsanguinate, Farewell, Idyllic Tutor, Profane Tutor, Read the Bones, Ringsight, Tezzeret's Gambit, Torment of Hailfire, Toxic Deluge

**Lands (37).** Adarkar Wastes, Cabal Coffers, Caves of Koilos, Command Tower, Flooded Strand, Godless Shrine, Hallowed Fountain, Marsh Flats, Mistrise Village, Polluted Delta, Scrubland, Serra's Sanctum, Tundra, Underground River, Underground Sea, Urborg, Tomb of Yawgmoth, Urza's Cave, Watery Grave, 6 Plains, 5 Island, 8 Swamp

## Import list (100)

```
1 Adarkar Wastes
1 Anguished Unmaking
1 Arcane Signet
1 Arrest
1 Austere Command
1 Bound in Silence
1 Cabal Coffers
1 Cast Away Doubt
1 Caves of Koilos
1 Champion's Helm
1 Coalition Relic
1 Command Tower
1 Conjurer's Closet
1 Consecrated Sphinx
1 Counterspell
1 Debt to the Deathless
1 Demonic Tutor
1 Diabolic Tutor
1 Displacer Kitten
1 Dovin's Veto
1 Esper Sentinel
1 Expedition Map
1 Exquisite Blood
1 Exsanguinate
1 Farewell
1 Fatal Push
1 Flooded Strand
1 Force of Will
1 Generous Gift
1 Go for the Throat
1 Godless Shrine
1 Grand Arbiter Augustin IV
1 Hallowed Fountain
1 Idyllic Tutor
1 Loran of the Third Path
1 Marauding Blight-Priest
1 Marsh Flats
1 Mistrise Village
1 Mystic Remora
1 Mystical Tutor
1 Necropotence
1 Notion Thief
1 Orcish Bowmasters
1 Palantír of Orthanc
1 Path to Exile
1 Polluted Bonds
1 Polluted Delta
1 Prayer of Binding
1 Profane Tutor
1 Propaganda
1 Read the Bones
1 Restoration Angel
1 Rhystic Study
1 Ringsight
1 Sanguine Bond
1 Scrubland
1 Serra's Sanctum
1 Sheoldred, the Apocalypse
1 Skyclave Apparition
1 Smothering Tithe
1 Sol Ring
1 Sphere of Safety
1 Talisman of Hierarchy
1 Talisman of Progress
1 Teferi's Protection
1 Teleportation Circle
1 Tezzeret's Gambit
1 Torment of Hailfire
1 Toxic Deluge
1 Tundra
1 Underground River
1 Underground Sea
1 Urborg Syphon-Mage
1 Urborg, Tomb of Yawgmoth
1 Urza's Cave
1 Vampiric Tutor
1 Venat, Heart of Hydaelyn // Hydaelyn, the Mothercrystal
1 Watery Grave
1 Y'shtola, Night's Blessed
1 Your Fate Ends Here
1 Zur the Enchanter
5 Island
6 Plains
8 Swamp
```
