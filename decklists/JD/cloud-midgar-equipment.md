# Cloud, Midgar Mercenary — Mono-White Equipment

A mono-white Equipment deck. **Cloud, Midgar Mercenary** costs {W}{W} and finds an Equipment when he enters; with
Equipment on him, every Equipment trigger happens twice. The deck wins by hanging Swords and big Equipment on Cloud
(or on a backup carrier) and hitting. It is weak to board wipes and to artifact removal, and it needs Cloud, or
another carrier, to survive.

| | |
|---|---|
| **Commander** | Cloud, Midgar Mercenary ({W}{W}, 2/1, white) |
| **Bracket** | 3 (3 Game Changers: Smothering Tithe, Enlightened Tutor, Teferi's Protection) |
| **Simulator** | deck key `cloud`; results in `decklists/pool/pool-results.md`, history in `decklists/CHANGELOG.md` |


## Strategy

### The commander

**Cloud, Midgar Mercenary** ({W}{W}, 2/1 Human Soldier Mercenary, legendary). *When Cloud enters, search your library
for an Equipment card, reveal it, put it into your hand, then shuffle. As long as Cloud is equipped, if a triggered
ability of Cloud or an Equipment attached to it triggers, that ability triggers an additional time.*

- **A tutor every time he enters.** The deck has 10 Equipment, so the search always finds one. He is cheap to cast and
  to recast from the command zone (the commander tax adds {2} for each earlier cast from there), and a recast replaces
  a lost Equipment at once.
- **The doubling is narrow.** It applies only while Cloud himself is equipped, and only to triggers of Cloud or of an
  Equipment attached to *Cloud*. Equipment on another creature never doubles, and a static bonus (+X/+X, hexproof)
  does not trigger at all.
- **He is the plan and the weak point.** A 2/1 for {W}{W} dies to targeted removal, blockers and wipes. Equipped with
  a Sword he is a 4/3 with two protections, so the first Sword is also his first piece of defence.
- **When to cast him.** Turn 2 ({W}{W}), before anything else. A turn-1 Sol Ring does not delay him: it pays for the
  Sword the turn after. After a removal, recast him when you have the mana to equip him the same turn (Swiftfoot Boots
  and Sting, the Glinting Dagger give haste, so he can attack at once) and something to defend him; do not recast him
  into the same removal with nothing behind him.
- **Cloud is a Soldier**, so Auriok Steelshaper gives him +1/+1 while the Steelshaper is equipped, and Elspeth's
  Soldier tokens share that bonus. Path of Ancestry scries when a creature that shares a type with Cloud (Human or
  Soldier) is cast with its mana.

**Doubled triggers.** With Cloud equipped, these Equipment gain from the extra trigger:

- **Sword of Fire and Ice** (+2/+2, protection from red and blue): 2 damage to any target twice and two cards. A hit by
  a 4/3 Cloud is 4 combat damage, 4 more from the triggers and two cards.
- **Sword of Feast and Famine** (+2/+2, protection from black and green): two discards from the player you hit; your
  lands untap (the second untap adds nothing). Cast spells before combat, hit, then use the untapped lands after.
- **Sword of Hearth and Home** (+2/+2, protection from green and white): two lands and up to two creatures blinked.
  Point one trigger at a creature with a useful enter effect (Solemn Simulacrum, Knight of the White Orchid, Loyal
  Warhound, Kor Outfitter) or at Cloud himself for another search (his Equipment falls off when he leaves, so only when
  you can re-equip). Do not aim both at Cloud: the second would fizzle and lose its land.
- **Mask of Memory**: two triggers of draw two, then discard one: net two cards a hit.
- **Buster Sword** (+3/+2): two triggers of draw a card, then cast a spell from your hand free with mana value up to
  the damage dealt.
- **Andúril, Narsil Reforged**: whenever the equipped creature attacks, it puts a +1/+1 counter on each creature you
  control (two with the city's blessing), so Cloud's attack puts two counters on each (four with the blessing).

**Swiftfoot Boots**, **Colossus Hammer** and **Fireshrieker** have no triggers: they are plain bonuses (hexproof and
haste, +10/+10, double strike). **Sting, the Glinting Dagger** (+1/+1 and haste) has an untap trigger at the start of
each combat; doubling it adds nothing.

### How the deck wins

The deck wins by combat damage from an equipped Cloud, and by commander damage when it can. Everything below is a way
to get the right Equipment onto him and keep him alive for the swing.

#### Line 1: Cloud with a Sword

Cloud ({W}{W}) plus one Sword (cast {3}, equip {2}): 5 mana beyond Cloud.

1. **Turn 2:** cast Cloud. His search finds the Sword that fits the table (see "What to tutor for").
2. **Turn 3 with Sol Ring** (turn 1 Sol Ring, turn 2 Cloud): cast the Sword and equip it the same turn, and attack with
   Cloud as a 4/3 with protection from two colours. Without Sol Ring, cast the Sword on turn 3 and equip on turn 4,
   with 2 mana spare for Swiftfoot Boots or Mask of Memory.
3. **Attack only when it is safe.** A bare 2/1 does not attack into blockers. With a Sword he is a 4/3, and creatures
   of the Sword's two colours cannot block him.
4. **Each hit pays twice.** Sword of Fire and Ice: 4 damage and two cards on top of the hit. Sword of Feast and Famine:
   two discards and a second main phase of untapped lands. Sword of Hearth and Home: two lands and a blink.
5. **Add a second Sword.** Two Swords give four protections (for example black, green, red and blue), and each hit
   triggers both of them twice.

#### Line 2: Double strike

Fireshrieker (cast {3}, equip {2}) gives double strike. Cloud with a Sword and Fireshrieker triggers the Sword at
first-strike damage and again at regular damage, and each trigger is doubled by Cloud: four Sword triggers per
attack. Sword of Fire and Ice then deals 8 damage and draws four cards on top of 8 combat damage. It also deals
commander damage twice.

- **Silverblade Paladin** ({1}{W}{W}, soulbond): when it enters, pair it with Cloud and both have double strike. It
  needs no equip, and it works the turn Cloud is paired even if every Equipment is gone.
- **Akroma's Will** ({3}{W}, instant): creatures you control gain flying, vigilance and double strike until end of
  turn. With Cloud (a commander) out you may choose both modes and add lifelink, indestructible and protection from
  each colour. Cast it in the beginning-of-combat or declare-attackers step: flying gets Cloud past ground blockers
  (after blockers it no longer unblocks him). Hold it for its second mode if a wipe or removal is coming instead.
- **Balan, Wandering Knight** has double strike on his own while two or more Equipment are attached to him.
- **Commander damage.** Cloud as a 4/3 with double strike hits for 8, so three swings kill one opponent. With Buster
  Sword and a Sword (7/5) double strike deals 14: two swings.

#### Line 3: Colossus Hammer, the one-swing commander kill

Colossus Hammer costs {1}, gives +10/+10 and loses flying, and equip costs {8}. Cloud with the Hammer is a 12/11. With
double strike that is 24 commander damage in one attack, and 21 kills.

1. **Get the Hammer on the battlefield early.** It costs {1}: cast it as soon as you have a spare mana, or fetch it with
   Cloud's search or a tutor once you can afford to equip it.
2. **Never pay {8} if you can attach it free.** **Ardenn** attaches any number of Equipment to Cloud at the beginning of
   combat on your turn, even the turn she is cast. **Kor Outfitter** attaches one Equipment when it enters.
   **Puresteel Paladin** makes equip {0} with three artifacts.
3. **Example kill turn:** Cloud (not summoning sick) and the Hammer on the battlefield, six mana. Cast Ardenn ({2}{W})
   and Fireshrieker ({3}) in the first main phase. At the beginning of combat, Ardenn attaches both to Cloud: a 12/11
   with double strike. Attack one opponent: 24 damage if nothing blocks or prevents it.
4. **Mind the blockers.** The Hammer gives no evasion. Pro-colour from a Sword stops creatures of those colours; Rogue's
   Passage (Line 4) stops all of them; **Helitrooper** gives another attacker flying when it attacks.

#### Line 4: The Rogue's Passage push

**Rogue's Passage** ({4}, {T}: target creature can't be blocked this turn) puts a loaded Cloud through a wall of
blockers. It taps itself as part of the cost, so it needs five lands: four other mana sources, plus Passage.

- Use it in your own turn, before blockers are declared (the beginning of combat or the declare-attackers step).
- Best with the Hammer (12 damage unblocked, 24 with double strike) or with a Sword and Fireshrieker when the table
  has a lot of blockers (Marwyn's Elves, Lathril's tokens, Sythis's wall).
- Equip first, then pay for Passage with what is left. With seven lands: equip a Sword ({2}) and activate Passage.

#### Line 5: Backup carriers (Cloud is gone)

When Cloud is dead or removed, the Equipment stays on the battlefield and a carrier takes it. **Balan** moves every
Equipment onto himself for {1}{W} and has double strike with two attached. **Danitha Capashen** has first strike,
vigilance and lifelink, and makes Aura and Equipment spells cost {1} less. **Adelbert Steiner** (lifelink, +1/+1 per
Equipment), **Thalia, the Survivor** (lifelink; opponents' noncreature spells cost {1} more), **Armored Skyhunter**
(flying), **Exalted Angel** (cast face down for {3}, flip for {2}{W}{W}; flying, and you gain life equal to the damage
it deals) and **Jocasta, Automaton Avenger** (flying; grows when your commander connects, and comes back from the
graveyard when you attack with him) all carry the Swords. **Zack Fair** hands his counters and Equipment to another
creature when he is sacrificed, which also makes that creature indestructible. Then recast Cloud: his search replaces
what was lost. **Swiftfoot Boots** and **Sting** give a recast Cloud haste.

#### The fair win: a wide board

When Equipment is scarce or Cloud keeps dying, the deck grinds. **Elspeth, Sun's Champion** makes three 1/1 Soldiers a
turn; **Andúril** puts a counter on every creature you control each attack; **Archangel of Tithes**, **Windborn Muse**,
**Baird**, **Archon of Absolution** and **Norn's Annex** make the table pay to attack; **Court of Ardenvale** (monarch)
and **Smothering Tithe** supply cards and Treasures. **Sram, Senior Edificer** draws a card for every Aura, Equipment or
Vehicle spell. Fliers (Archangel, Archon, Muse, Skyhunter, Cid, Jocasta) finish the game.

#### Free and cheap equips (the engine)

Every Equipment costs mana twice (cast and equip), so these matter more than
extra Equipment.

- **Ardenn, Intrepid Archaeologist** moves any number of Equipment onto one permanent at the beginning of combat.
- **Kor Outfitter** attaches one Equipment when it enters.
- **Forge Anew** ({2}{W}, enchantment) returns an Equipment from your graveyard to the battlefield when it enters; on
  your turn you equip at instant speed, and the first equip each turn costs {0}.
- **Armory Automaton** attaches any number of Equipment to itself when it enters or attacks.
- **Armored Skyhunter** flies; whenever it attacks it looks at the top six cards and puts an Equipment onto the
  battlefield, attached to a creature (Cloud) if you like.
- **Balan** moves every Equipment onto himself for {1}{W}.
- **Puresteel Paladin** draws a card whenever an Equipment enters and, with three or more artifacts, gives your
  Equipment equip {0}: Colossus Hammer becomes practical.
- **Auriok Steelshaper** reduces equip costs by {1}; **Helitrooper** makes equipping it cheaper by {2} and gives another
  attacker flying; **Cid, Freeflier Pilot** makes Equipment spells cost {1} less, flies on your turn and returns an
  Equipment from the graveyard to your hand.
- **Stoneforge Mystic** ({1}{W}, {T}): puts an Equipment from your hand onto the battlefield. Do it at the end of an
  opponent's turn.
- **Marvin, Murderous Mimic** ({2}, colorless 2/2) has the activated abilities of your other creatures: with Stoneforge
  Mystic out it is a second {1}{W}, {T}: an Equipment onto the battlefield; with Giver of Runes out it is a second
  {T}: protection for Cloud. It needs one of them on the board; alone it is a 2/2 (and a body against edicts).

#### Making Cloud enter again

**Court of Ardenvale** returns a permanent with mana value 3 or less from your graveyard at
each upkeep (to the battlefield while you are the monarch, which Court makes you). Every Equipment in the deck has
mana value 3 or less, and so does Cloud: if he died and you left him in the graveyard instead of the command zone, Court
can return him with no tax and his search triggers again. **Kor Outfitter**, **Solemn Simulacrum**, **Knight of the
White Orchid** and **Loyal Warhound** are worth a second enter if Sword of Hearth and Home blinks them.

### Game plan

Deployment is the deck's bottleneck: an equipped Cloud attacking is often not online by turn 4, or even turn 6, and cards
get stuck in hand. So spend all your mana every turn, and slow the table down so the plan has time to come online.

**Turns 1-3.**

- Turn 1: Plains and Sol Ring (the best opening), or Giver of Runes or Zack Fair. Hold Steelshaper's Gift ({W}) for
  the turn Cloud is gone. A rock (Mind Stone, Arcane Signet, Fellwar Stone) is fine on turn 2 if you cannot make
  {W}{W}.
- Turn 2: Cloud. If you cannot make {W}{W}, play a two-drop that works the Equipment (Stoneforge Mystic, Sram, Auriok
  Steelshaper, Cid) or a rock.
- Turn 3: cast the first Sword and equip it if you have the mana, with Giver of Runes, Swiftfoot Boots or Teferi's
  Protection ready (see "Protecting Cloud"). Otherwise cast the Sword and keep Cloud home.
- Knight of the White Orchid and Loyal Warhound fetch a Plains when an opponent controls more lands than you:
  a free ramp if the table ramps.

**Turns 4-6.**

- Equip the Sword, attack, and cast a second piece: Mask of Memory, Buster Sword, Fireshrieker or Swiftfoot Boots.
- Add a free-equip piece (Ardenn, Kor Outfitter, Puresteel Paladin) before the big Equipment.
- Add defence: a tax creature (Archangel of Tithes at four mana, Windborn Muse, Baird, Archon of Absolution) or Court of
  Ardenvale. Defence matters most here: slowing the race that kills Cloud is worth more than another Equipment.
- Open the Armory and Enlightened Tutor find the next piece.

**Late (turn 7+).**

- Colossus Hammer with Ardenn or Puresteel Paladin; Fireshrieker or Silverblade Paladin for double strike; Rogue's
  Passage for the final swing.
- Elspeth, Sun's Champion at six mana; Austere Command or Winds of Abandon overload to clear the board when you are
  behind.
- Keep a way to protect Cloud on the swing turn.

**Hold up or go?**

- **Go** when opponents are tapped low, when a Sword on the battlefield can be equipped this turn, or when a swing now
  gets ahead of a table that is about to wipe.
- **Hold up {2}{W}** (Judgment of Alexander or Teferi's Protection) when an opponent has a lethal-looking attack
  (Isshin, Aurelia, Marwyn with Craterhoof, Kaalia) or removal for Cloud on an empty mana board. Do not tap out for a
  second Equipment while Cloud is the only creature.
- **Equip last, with protection up.** Equip is sorcery speed (Forge Anew aside), so an opponent can answer the
  activation. Equip with Giver of Runes untapped, Swiftfoot Boots on Cloud, or opponents tapped out.
- **Keep the hand small.** Cheap cards that sit in hand do nothing: cast Stoneforge Mystic, Sram and the rock early
  even if it delays the Sword by a turn.

**What beats the deck.**

- **Board wipes** (Toxic Deluge, Damn, Blasphemous Act, Farewell): do not overcommit creatures when you already have
  enough power; keep Stoneforge Mystic, Steelshaper's Gift and the backup carriers in hand.
- **Artifact and enchantment removal** (Aura Shards, Reclamation Sage, Collector Ouphe): spread Equipment across a few
  carriers (Balan and Ardenn re-attach) and recur with Cid or Forge Anew.
- **A faster combat deck** (Isshin, Aurelia, Kaalia): you lose the race, not the board. Taxes and Judgment of Alexander
  are your answers.
- **Cloud in the command zone:** remember the {2} tax per earlier cast; use backup carriers first.

### Mulligans

The first mulligan is free (you see two 7-card hands). After that it is a London mulligan: draw 7, put one card on the
bottom for each mulligan after the free one.

| Hand | Keep when |
|---|---|
| 7 cards | 3-5 lands, or 2 lands plus a rock or a cheap creature (Sol Ring, Mind Stone, Arcane Signet, Fellwar Stone, Coalition Relic, Thought Vessel, Giver of Runes, Zack Fair, Knight of the White Orchid, Loyal Warhound, Puresteel Paladin, Kor Outfitter, Auriok Steelshaper, Grand Abolisher, Ardenn); and {W}{W} by turn 3 from Plains, Path of Ancestry or Plaza of Heroes |
| 6 or 5 cards | 2-4 lands that make {W}{W}, and any play (a rock, a cheap creature, a tutor, removal) |
| 4 cards | Keep |

- **The hand needs mana, not Equipment.** Cloud is always castable from the command zone and his search finds the
  Equipment. A hand of lands, rocks and removal is a better keep than a hand of Equipment and two lands.
- **Count the white.** Ash Barrens, Terminal Moraine and Rogue's Passage make only {C}. Evolving Wilds, Ash Barrens
  (basic landcycling {1}) and Terminal Moraine (costs {2}) fetch a Plains but take a turn. Plaza of Heroes makes any
  colour only for a legendary spell (Cloud is legendary), so it counts as a {W} source for Cloud.
- **Bottom first:** Colossus Hammer (equip {8}) without Puresteel Paladin or Ardenn, the six-drops (Elspeth, Austere
  Command) in a slow hand, a second wipe, then extra lands past four.
- **Do not keep one-landers**, even with rocks: the rocks need a land first.

| # | Hand | Call | Why |
|---|---|---|---|
| 1 | Plains, Plains, Plains, Sol Ring, Sword of Fire and Ice, Swiftfoot Boots, Swords to Plowshares | Keep | Turn 1 Sol Ring, turn 2 Cloud and Boots, turn 3 cast and equip the Sword and attack |
| 2 | Plains, Plains, Evolving Wilds, Mind Stone, Giver of Runes, Steelshaper's Gift, Buster Sword | Keep | Three lands, a rock, protection (Giver) and a tutor; Cloud on turn 2 with Giver ready for turn 3 |
| 3 | Plains, Terminal Moraine, Rogue's Passage, Sword of Fire and Ice, Fireshrieker, Colossus Hammer, Elspeth, Sun's Champion | Ship | One {W} source and no way to make {W}{W} before turn 4; every spell costs three or more |
| 4 | Plains, Sol Ring, Mind Stone, Arcane Signet, Fellwar Stone, Coalition Relic, Thought Vessel | Ship | One land and six rocks, no spell to spend the mana on; the rocks need land drops |
| 5 | Plains, Plains, Mind Stone, Stoneforge Mystic, Giver of Runes, Sword of Hearth and Home, Austere Command | Keep (close) | Two lands plus Mind Stone; plays at one and two mana; Austere Command is the one dead card early |
| 6 | Plains, Plains, Plains, Plains, Evolving Wilds, Colossus Hammer, Sword of Fire and Ice | Keep (close) | Five lands is the top of the range, but Cloud on turn 2 finds a third Equipment and the hand has a Sword and the Hammer; it wants Ardenn or Puresteel Paladin for the Hammer |
| 7 | Plains, Plains, Colossus Hammer, Buster Sword, Fireshrieker, Austere Command, Elspeth, Sun's Champion | Ship | Two lands, no rock, no cheap play: after Cloud, nothing costs less than three |
| 8 | Plains, Plains, Plains, Sol Ring, Archangel of Tithes, Elspeth, Sun's Champion, Teferi's Protection | Keep, bottom Elspeth | 6-card rule: Sol Ring makes Archangel a turn-3 play behind Cloud, and Protection backs her up; Elspeth is the slowest card |

### What to tutor for

Cloud's search is the deck's main tutor. **Stoneforge Mystic** finds an Equipment when it enters and puts one from your
hand onto the battlefield for {1}{W}, {T} (do it at an opponent's end step). **Steelshaper's Gift** ({W}) finds any
Equipment, and **Open the Armory** ({1}{W}) finds an Aura or Equipment; the list has no Auras, so it finds Equipment
only. **Enlightened Tutor** ({W}, instant) finds an artifact or enchantment and puts it on top: cast it at the end of an
opponent's turn.

| Situation | Tutor | Target |
|---|---|---|
| Turn 2, no Equipment yet | Cloud | A Sword that fits the table (see the table below); Sword of Fire and Ice if nothing stands out |
| A Sword on Cloud, want more value | Cloud, Stoneforge, Gift | Mask of Memory (cast {2}, equip {1}): two cards net a hit |
| Opponents have spot removal | Cloud, Gift | Swiftfoot Boots (hexproof and haste, equip {1}) |
| Puresteel Paladin or Ardenn out | Cloud, Gift, Enlightened Tutor | Colossus Hammer: attached for free |
| Hand full of spells to deploy | Cloud, Gift | Buster Sword: a hit casts a spell free |
| Lethal setup | Cloud, Gift | Fireshrieker (double strike) if Cloud already carries the Hammer or a Sword |
| Cloud dead or removed | Steelshaper's Gift, Stoneforge, Open the Armory | The Equipment the next carrier needs; a Sword |
| Equipment destroyed | Enlightened Tutor | Forge Anew, then return the Equipment |
| Opponents draw a lot | Enlightened Tutor | Smothering Tithe |
| Need cards | Enlightened Tutor | Court of Ardenvale |
| Stuck on mana | Enlightened Tutor | Sol Ring |

**Which Sword?** Protection stops targeting, blocking by creatures of that colour, and damage from sources of that
colour. It does not stop wipes or edicts.

| Sword | Protection from | Best against |
|---|---|---|
| Sword of Fire and Ice | red, blue | Korvold, Aurelia, Isshin, Kaalia (red burn and attackers); Brago, Tatyova (blue) |
| Sword of Feast and Famine | black, green | Lathril, Meren, Tergrid, Marwyn, Korvold, Lord Windgrace |
| Sword of Hearth and Home | green, white | Light-Paws, Sythis, Marwyn; white removal and white creatures (Teysa, Brago, Aurelia, Isshin, Kaalia) |

Two Swords stack. Sword of Feast and Famine plus Sword of Hearth and Home gives black, green and white: no targeted
removal from Teysa's white-black deck can touch Cloud.

### Protecting Cloud

Cloud is the key permanent: the Equipment is only as good as its carrier. Order of defence:

1. **Prevent.** Swiftfoot Boots (hexproof), the Sword's protection colours, Grand Abolisher (opponents cannot cast
   spells or activate abilities of artifacts, creatures or enchantments during your turn), Bastion Protector (commander
   creatures get +2/+2 and indestructible: wipes that destroy and damage do not kill him), Giver of Runes (protection
   from a colour for another creature, at instant speed).
2. **Answer in response.**
   - **Teferi's Protection** ({2}{W}, instant): until your next turn your life total can't change, you have protection
     from everything, and all your permanents phase out. Cloud keeps his Equipment through removal, a wipe or a lethal
     attack. Cast it on an opponent's turn.
   - **Galadriel's Dismissal** ({W}, instant): phases Cloud out with his Equipment until your next turn. Kicked
     ({2}{W} more), all of a player's creatures phase out.
   - **Plaza of Heroes** ({3}, {T}, exile it): a legendary creature gains hexproof and indestructible until end of turn.
   - **Zack Fair** ({1}, sacrifice): another creature gains indestructible, and Zack's counters and Equipment move to
     it.
   - **Akroma's Will**, second mode: lifelink, indestructible and protection from each colour for the turn.
3. **Protect the team.** Elspeth makes three Soldiers a turn as fodder for edicts and wall for attackers. Her -3
   destroys creatures with power 4 or more, and Cloud with any Sword has power 4: use it only when Cloud is not
   carrying one, or when he is not on the board.
4. **Judgment of Alexander** ({2}{W}, instant): prevents all damage that would be dealt to you this turn by sources your
   opponents control, and each commander creature you control deals damage equal to its power to each creature whose
   damage was prevented. Cast it once attackers are declared: with Cloud equipped it is a fog that kills the attackers.

**Timing rules.**

- Removal in response to an equip activation is the usual loss. Hold up Giver, Boots or Protection before you pay.
- Wrath-type sweeps (Toxic Deluge, Damn, Blasphemous Act) kill a lone Cloud. Phase him out or keep him in the command
  zone until the table has shown its hand.
- Exile (Swords to Plowshares, Path to Exile) goes round indestructible: hexproof, protection and phasing are the answers.
- Edicts (Plaguecrafter, Fleshbag Marauder, Sheoldred's Edict, Grave Pact, Dictate of Erebos) take Cloud when he is the
  only creature. Keep tokens or another cheap creature (Marvin, Giver of Runes, Zack Fair) around.

### Interaction on opponents' turns

The deck's answers, most of them at instant speed:

| Card | Cost | Use it on |
|---|---|---|
| Swords to Plowshares | {W} | The one creature that is winning: Kaalia, Light-Paws, Meren, Korvold. Its controller gains life equal to its power |
| Path to Exile | {W} | Same; the controller may fetch a basic land tapped. Marit Lage (indestructible) |
| Stroke of Midnight | {2}{W} | Any nonland permanent: a planeswalker (Windgrace, Atraxa's walkers), Grave Pact, Aura Shards, Rhystic Study. Its controller gets a 1/1 Human |
| Judgment of Alexander | {2}{W} | A big attack: Isshin's team, Aurelia's combats, Craterhoof, Kaalia's cheat. Each creature whose damage is prevented takes damage equal to Cloud's power |
| Teferi's Protection | {2}{W} | A wipe or lethal attack; phases out your board and your life total |
| Galadriel's Dismissal | {W} | Removal aimed at Cloud, or a wipe (kicked) |
| Akroma's Will | {3}{W} | An attack that would kill Cloud, or a swing back of your own |
| Giver of Runes, Zack Fair, Plaza of Heroes | {T} / {1} / {3} | Targeted removal on Cloud (Giver cannot tap the turn it enters) |
| Exalted Angel | {2}{W}{W} to flip | Cast face down for {3}, flip at instant speed mid-combat: a flying blocker that gains you life |
| Enlightened Tutor | {W} | End of an opponent's turn: the Sword or Equipment you need next |
| Skyclave Apparition | {1}{W}{W} | Mana value 4 or less on your turn: Kaalia, Meren, Teysa, Rhystic Study, Aura Shards, Grave Pact |
| Winds of Abandon | {1}{W} / {4}{W}{W} | One creature (sorcery speed); overloaded it exiles all your opponents' creatures and leaves yours. Each controller fetches a basic land tapped |
| Lay Down Arms | {W} | A creature with mana value up to your Plains. Its controller gains 3 life. Light-Paws, Sythis, Marwyn at two or three Plains |
| Wrath of God, Fumigate | {2}{W}{W} / {3}{W}{W} | A wide opposing board (Lathril, Marwyn, Tatyova's Avenger). Fumigate gains 1 life per creature. Equipment survives |
| Austere Command | {4}{W}{W} | Choose two modes. "Creatures with mana value 4 or more" plus "enchantments" leaves Cloud (mana value 2) and the artifacts; "artifacts" kills your own Equipment |

Passive defence:

- **Archangel of Tithes** ({1}{W}{W}{W}, 3/5 flier): while untapped, opponents pay {1} for each creature that attacks you;
  while she attacks, they pay {1} for each blocker. She taps when she attacks, so the tax turns off then.
- **Windborn Muse** (a 2/3 flier, {2} per attacker), **Baird, Steward of Argive** (a 1/4 vigilance, {1} per attacker),
  **Norn's Annex** ({W/P} per attacker; costs {3}{W/P}{W/P}) and **Archon of Absolution** ({1} per attacker, a 3/2 flier with
  protection from white). They stack.
- **Thalia, the Survivor** makes noncreature spells cost opponents {1} more. Smothering Tithe turns their draws into
  Treasures for you.

### Matchups (Tiers 1-3)

Baseline (before Rogue's Passage and Sting): 29.6% at t1, 20.6% at t2, 22.4% at t3, 24.2% over t1-t3 (2,000 games each,
loose profile). In earlier sims Light-Paws caused about 18% of Cloud's eliminations (commander damage) and Isshin about
15% (combat). Sword choice below uses the table in "What to tutor for".

**Tier 1**

| Opponent | Their threat | Your plan | Cards that matter |
|---|---|---|---|
| Isshin (Mardu attackers) | Isshin doubles attack triggers: Hero of Bladehold, Adeline, Hellrider swing for ten or more on turn four. Swords to Plowshares, Path to Exile, Anguished Unmaking, Blasphemous Act, Damn | Slow the swing, then race. Kill Isshin (mana value 3). Sword of Fire and Ice (protection from red) | Archangel of Tithes, Windborn Muse, Baird, Norn's Annex, Judgment of Alexander, Skyclave Apparition, Wrath of God |
| Lathril (Golgari Elves) | Elves flood the board; Lathril taps ten Elves: you lose 10. Trophy, Putrefy, Beast Within, Reclamation Sage | Sweep before turn 5. Sword of Feast and Famine: green creatures cannot block him, and black or green removal cannot target him | Wrath of God, Fumigate, Winds of Abandon (overload), Archangel, Skyclave Apparition on Lathril (mana value 4) |
| Light-Paws (Aura voltron) | One huge Aura'd creature, commander damage, instant protection (Blacksmith's Skill, Flickerform). The worst t1 matchup | Kill her (mana value 2) before the Aura chain: Lay Down Arms, Skyclave, Swords to Plowshares, Path to Exile. Sword of Hearth and Home: white creatures cannot block him, and white damage to him is prevented | Elspeth's -3 (when Cloud has no Sword), Judgment of Alexander, Winds of Abandon, Soldier tokens as chump blockers |
| Tatyova (Simic landfall) | Ramp, then Avenger of Zendikar or Ancient Greenwarden; Counterspell, Negate, Deprive, Aetherize | Race: she rarely attacks. Resolve Cloud before counters. Wrath if Avenger lands | Smothering Tithe (her draws pay or feed you), Thalia, Grand Abolisher (no counters on your turn), Stroke of Midnight on Tatyova |
| Teysa (Orzhov aristocrats) | Blood Artist, Zulaport, Cruel Celebrant drain; Teysa doubles them; Grave Pact and Dictate of Erebos edicts; Plunderer + Skeleton + altar | Do not wipe: their creatures dying drains you. Exile the drain creatures and Teysa; keep a fodder creature for edicts. Sword of Feast and Famine plus Sword of Hearth and Home: no targeted removal can hit Cloud | Skyclave on Teysa or Grave Pact, Stroke of Midnight on Dictate of Erebos, Austere Command (enchantment mode), Elspeth's tokens |

**Tier 2**

| Opponent | Their threat | Your plan | Cards that matter |
|---|---|---|---|
| Brago (Azorius blink control) | Counterspell, Mana Drain, An Offer You Can't Refuse; Aven Mindcensor (Cloud searches only the top four); Farewell; Agent of Treachery | Land Cloud and the first Sword before counters. Do not overextend into Farewell; keep Stoneforge Mystic and Steelshaper's Gift in hand. Rogue's Passage past the fliers | Grand Abolisher (no spells on your turn), Thalia, Swords to Plowshares or Path to Exile on Agent of Treachery (mana value 7) |
| Kaalia (Mardu cheat) | Kaalia attacks and puts an Angel, Demon or Dragon into play attacking (Master of Cruelties, Terror of the Peaks, Lyra) | Kill Kaalia on sight (mana value 4). Attack taxes make Kaalia pay to attack but not the cheated attackers. Judgment of Alexander when one is already attacking | Skyclave Apparition, Swords to Plowshares, Path to Exile, Lay Down Arms at four Plains, Judgment of Alexander, Teferi's Protection |
| Lord Windgrace (Jund lands) | Landfall board (Avenger, Field of the Dead, Baloths), Marit Lage, Constant Mists, Strip Mine. No counters | Race and attack the planeswalkers. Exile Marit Lage (indestructible, so Wrath and Elspeth miss it). Sword of Feast and Famine or Fire and Ice: Trophy, Putrefy and Beast Within cannot target him | Swords to Plowshares, Path to Exile, Winds of Abandon, Lay Down Arms (the token has mana value 0), Stroke of Midnight on a planeswalker |
| Meren (Golgari recursion) | Meren returns a creature every end step; Massacre Wurm (-2/-2), Plaguecrafter and Fleshbag edicts, Grave Pact, Toxic Deluge | Win before the grind. Keep Cloud equipped: a 4/3 survives Massacre Wurm and a bare 2/1 does not. Keep a fodder creature. Kill Meren (mana value 4) | Skyclave Apparition, Teferi's Protection (against Deluge and Wurm), Elspeth's tokens, Sword of Feast and Famine |
| Sythis (Selesnya enchantress) | Aura Shards destroys an Equipment each time a creature enters; Ghostly Prison and Archon of Sun's Grace taxes; Banishing Light, Oblivion Ring, Journey to Nowhere; Winds of Rath | Kill Aura Shards and the engines early. Free equips (Ardenn, Kor Outfitter) re-attach cheaply. Pay the tax only for a kill. Sword of Hearth and Home: white and green removal cannot target Cloud | Skyclave Apparition on Aura Shards, Stroke of Midnight, Austere Command (enchantment mode), Forge Anew, Cid, Rogue's Passage |

**Tier 3**

| Opponent | Their threat | Your plan | Cards that matter |
|---|---|---|---|
| Atraxa (4-colour proliferate) | Planeswalkers (Oko, Teferi, The Wandering Emperor), poison creatures, Rhystic Study, Narset (you draw one card a turn), Farewell | Attack the planeswalkers. Oko can turn Cloud or an Equipment into an Elk: Sword of Feast and Famine or Hearth and Home blocks the target. Pay Rhystic Study's {1} on key spells | Stroke of Midnight on a planeswalker, Skyclave on Rhystic Study or Narset, Swords to Plowshares on poison creatures |
| Aurelia (Boros combats) | Aurelia (flying, vigilance, haste, extra combat), Hero of Bladehold, Hellrider, Embercleave; Helm of the Host + Combat Celebrant late | Taxes and Judgment of Alexander at the first combat. Exile Aurelia (mana value 6) when she lands. Sword of Fire and Ice plus Sword of Hearth and Home: red and white removal cannot target Cloud | Archangel of Tithes, Windborn Muse, Baird, Winds of Abandon (overload), Swords to Plowshares, Path to Exile, Wrath of God |
| Korvold (Jund sacrifice) | Blood Artist, Zulaport, Mayhem Devil, Goblin Bombardment, Impact Tremors, Terror of the Peaks; Toxic Deluge; Decay, Trophy, Terminate, Bolt | Race. Sword of Fire and Ice plus Sword of Feast and Famine cover red, blue, black and green: nearly all their removal. Kill Korvold (mana value 5). Lifelink for the drain | Stroke of Midnight, Skyclave on Mayhem Devil or Bombardment, Teferi's Protection, Danitha, Adelbert Steiner, Thalia |
| Marwyn (mono-green Elves) | Elves into Craterhoof Behemoth (Natural Order, Chord, Finale); Gaea's Cradle; Collector Ouphe (no equip); Beast Within | Every creature is green: a Sword with protection from green (Feast and Famine or Hearth and Home) makes Cloud unblockable by them. Hold Judgment of Alexander and taxes for Craterhoof; sweep before they go wide. Free equips beat Ouphe | Judgment of Alexander, Archangel of Tithes, Wrath of God, Fumigate, Ardenn, Kor Outfitter, Armory Automaton |
| Tergrid (mono-black disruption) | Edicts and discard; Tergrid takes the nontoken permanents you sacrifice or discard; Liliana of the Veil; Painful Quandary; Go for the Throat, Infernal Grasp | Sword of Feast and Famine: protection from black means no creature of theirs can block him or target him. Keep fodder against edicts. Play your hand out before Liliana | Elspeth's tokens, Teferi's Protection, Sword of Feast and Famine, Wrath of God |

Against Tergrid, do not sacrifice Evolving Wilds, Terminal Moraine, Zack Fair or Mind Stone, cycle Ash Barrens, or
discard a permanent card to Mask of Memory: she puts the card onto the battlefield under her control.

### After a wipe or losing Cloud

| What happened | Rebuild |
|---|---|
| Cloud died or was exiled | Recast from the command zone for {W}{W} plus {2} per earlier cast (4, 6, 8 mana). His search replaces any lost Equipment. The third cast costs six: prefer a backup carrier |
| Board wipe, Equipment survived | Cloud is in the command zone and the Equipment stays. Recast Cloud and equip, or Balan ({1}{W}) / Ardenn / Kor Outfitter move it for free |
| Equipment destroyed or exiled (Aura Shards, Farewell) | Forge Anew returns one from the graveyard; Cid returns one to hand; Court of Ardenvale returns any Equipment (all have mana value 3 or less). Steelshaper's Gift, Stoneforge Mystic or Open the Armory find more |
| Cloud in the graveyard, Court in play | If you left him there, Court returns him at upkeep (to the battlefield while you are the monarch): his search triggers again with no tax |
| Opponent stole the monarch | Kill the attacker or tax it; Archangel of Tithes and the tax creatures protect it |
| Hand empty after a wipe | Court of Ardenvale or Mask of Memory and Sram refill; Elspeth makes bodies; Smothering Tithe gives mana |
| A backup carrier survived | Move the Swords onto it (Balan, Ardenn, Armory Automaton) and keep attacking; recast Cloud behind it |

- Do not recast Cloud into the same removal. If an opponent kept the answer for him, bring Swiftfoot Boots or Giver of
  Runes first.
- Save one tutor for after the wipe: Steelshaper's Gift and Stoneforge Mystic keep the deck going when Cloud is gone.

### Life management

| Source | Cost |
|---|---|
| Norn's Annex ({3}{W/P}{W/P}) | 2 life for each {W/P} paid with life (up to 4) |

The deck has no other life costs. Teferi's Protection freezes your life total until your next turn: you cannot pay
life and you cannot gain it. The deck gains life through:

| Source | Gain |
|---|---|
| Danitha Capashen, Adelbert Steiner, Thalia, the Survivor | Lifelink on the attack or block |
| Exalted Angel | Life equal to the damage it deals |
| Akroma's Will, second mode | Lifelink for your creatures this turn |
| Fumigate | 1 life per creature destroyed |

Judgment of Alexander prevents damage instead of gaining life.

Pay {W} for Norn's Annex when you have the mana. Pay life only when it saves a turn of Cloud's plan and no opponent is
about to swing (not against Isshin, Aurelia, Kaalia, Lathril's drain or Craterhoof).

### Common mistakes

1. Equipping into open removal at sorcery speed with no Giver, Boots or Protection behind it.
2. Putting the Equipment on another creature and expecting the doubling: it works only on Cloud.
3. Using Elspeth's -3 while Cloud carries a Sword: a 4/3 has power 4 and dies with the rest.
4. Casting Wrath of God or Fumigate over a board you built, when Austere Command ("mana value 4 or greater" plus
   "enchantments") or Winds of Abandon's overload would clear theirs and keep Cloud. Against Teysa, Korvold and Meren do
   not wipe at all unless you must: their deaths feed them.
5. Sending a bare 2/1 Cloud into blockers. Wait for the Sword.
6. Aiming both Sword of Hearth and Home triggers at Cloud, or blinking Cloud with an Equipment you cannot re-equip.
7. Paying {8} to equip Colossus Hammer instead of attaching it with Ardenn, Kor Outfitter, Armory Automaton or Puresteel
   Paladin.
8. Tapping out for the second Equipment while Judgment of Alexander or Teferi's Protection would have saved the game
   against Isshin, Aurelia, Kaalia or Craterhoof.
9. Activating Rogue's Passage after blockers are declared; it must come before.
10. Feeding Tergrid (sacrificing Evolving Wilds or Terminal Moraine, discarding a permanent to Mask of Memory, cycling
    Ash Barrens).

## Mana

33 lands: 27 Plains plus Ash Barrens (basic landcycling {1}), Evolving Wilds, Path of Ancestry (enters tapped; scry 1
when you cast a creature that shares a type with Cloud), Plaza of Heroes, Rogue's Passage and Terminal Moraine. Ash
Barrens, Terminal Moraine and Rogue's Passage make only {C}; Plaza of Heroes makes any colour for a legendary spell.
Cloud, Kor Outfitter, Puresteel Paladin, Knight of the White Orchid and Grand Abolisher need {W}{W}; Archangel of Tithes
needs {W}{W}{W}.

- **Rocks (6):** Sol Ring, Mind Stone, Arcane Signet, Fellwar Stone, Coalition Relic, Thought Vessel.
- **Other ramp:** Solemn Simulacrum; Knight of the White
  Orchid and Loyal Warhound, which fetch a Plains when an opponent controls more lands than you; Smothering Tithe makes
  Treasures when opponents draw and don't pay {2}.
- **Card draw:** Mask of Memory, Court of Ardenvale, Puresteel Paladin, Sram, Buster Sword, Sword of Fire and Ice and
  Mind Stone.
- **Removal and defence:** Wrath of God, Fumigate, Austere Command, Winds of Abandon, Lay Down Arms, Swords to
  Plowshares, Path to Exile, Stroke of Midnight, Skyclave Apparition, Judgment of Alexander, Archangel of Tithes,
  Archon of Absolution, Norn's Annex, Windborn Muse, Baird, Elspeth, Sun's Champion.
- **10 Equipment** in the deck: Andúril, Buster Sword, Colossus Hammer, Fireshrieker, Mask of Memory, Sting, Swiftfoot
  Boots, and the three Swords.

## Bracket and Rule 0

**Bracket 3,** with three Game Changers: **Smothering Tithe**, **Enlightened Tutor** and **Teferi's Protection**. The deck has no combo that wins
on its own. Tell the table before the game that:

- the deck has Equipment tutors (Cloud, Stoneforge Mystic, Steelshaper's Gift and Open the Armory) and Enlightened
  Tutor, but no game-winning combo;
- Austere Command can wipe the table, and Winds of Abandon (overloaded) and Elspeth, Sun's Champion can clear opposing creatures.

## Decklist by type (100)

**Commander (1).** Cloud, Midgar Mercenary

**Creatures (30).** Adelbert Steiner, Archangel of Tithes, Archon of Absolution, Ardenn, Intrepid Archaeologist, Armored Skyhunter, Armory Automaton, Auriok Steelshaper, Baird, Steward of Argive, Balan, Wandering Knight, Bastion Protector, Cid, Freeflier Pilot, Danitha Capashen, Paragon, Exalted Angel, Giver of Runes, Grand Abolisher, Helitrooper, Jocasta, Automaton Avenger, Knight of the White Orchid, Kor Outfitter, Loyal Warhound, Marvin, Murderous Mimic, Puresteel Paladin, Silverblade Paladin, Skyclave Apparition, Solemn Simulacrum, Sram, Senior Edificer, Stoneforge Mystic, Thalia, the Survivor, Windborn Muse, Zack Fair

**Enchantments (3).** Court of Ardenvale, Forge Anew, Smothering Tithe

**Artifacts (17).** Andúril, Narsil Reforged, Arcane Signet, Buster Sword, Coalition Relic, Colossus Hammer, Fellwar Stone, Fireshrieker, Mask of Memory, Mind Stone, Norn's Annex, Sol Ring, Sting, the Glinting Dagger, Swiftfoot Boots, Sword of Feast and Famine, Sword of Fire and Ice, Sword of Hearth and Home, Thought Vessel

**Sorceries (7).** Austere Command, Fumigate, Lay Down Arms, Open the Armory, Steelshaper's Gift, Winds of Abandon, Wrath of God

**Instants (8).** Akroma's Will, Enlightened Tutor, Galadriel's Dismissal, Judgment of Alexander, Path to Exile, Stroke of Midnight, Swords to Plowshares, Teferi's Protection

**Lands (33).** Ash Barrens, Evolving Wilds, Path of Ancestry, Plaza of Heroes, Rogue's Passage, Terminal Moraine, 27 Plains

**Planeswalkers (1).** Elspeth, Sun's Champion

## Import list (100)

```
1 Adelbert Steiner
1 Akroma's Will
1 Andúril, Narsil Reforged
1 Arcane Signet
1 Archangel of Tithes
1 Archon of Absolution
1 Ardenn, Intrepid Archaeologist
1 Armored Skyhunter
1 Armory Automaton
1 Ash Barrens
1 Auriok Steelshaper
1 Austere Command
1 Baird, Steward of Argive
1 Balan, Wandering Knight
1 Bastion Protector
1 Buster Sword
1 Cid, Freeflier Pilot
1 Cloud, Midgar Mercenary
1 Coalition Relic
1 Colossus Hammer
1 Court of Ardenvale
1 Danitha Capashen, Paragon
1 Elspeth, Sun's Champion
1 Enlightened Tutor
1 Evolving Wilds
1 Exalted Angel
1 Fellwar Stone
1 Fireshrieker
1 Forge Anew
1 Fumigate
1 Galadriel's Dismissal
1 Giver of Runes
1 Grand Abolisher
1 Helitrooper
1 Jocasta, Automaton Avenger
1 Judgment of Alexander
1 Knight of the White Orchid
1 Kor Outfitter
1 Lay Down Arms
1 Loyal Warhound
1 Marvin, Murderous Mimic
1 Mask of Memory
1 Mind Stone
1 Norn's Annex
1 Open the Armory
1 Path of Ancestry
1 Path to Exile
1 Plaza of Heroes
1 Puresteel Paladin
1 Rogue's Passage
1 Silverblade Paladin
1 Skyclave Apparition
1 Smothering Tithe
1 Sol Ring
1 Solemn Simulacrum
1 Sram, Senior Edificer
1 Steelshaper's Gift
1 Sting, the Glinting Dagger
1 Stoneforge Mystic
1 Stroke of Midnight
1 Swiftfoot Boots
1 Sword of Feast and Famine
1 Sword of Fire and Ice
1 Sword of Hearth and Home
1 Swords to Plowshares
1 Teferi's Protection
1 Terminal Moraine
1 Thalia, the Survivor
1 Thought Vessel
1 Windborn Muse
1 Winds of Abandon
1 Wrath of God
1 Zack Fair
27 Plains
```
