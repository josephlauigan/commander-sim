# Cloud, Midgar Mercenary — Mono-White Equipment

A mono-white Equipment deck. **Cloud, Midgar Mercenary** costs {W}{W} and finds an Equipment when he enters; with
Equipment on him, every Equipment trigger happens twice. The deck wins by hanging Swords and big Equipment on Cloud
(or on a backup carrier) and hitting. It is weak to board wipes and to artifact removal, and it needs Cloud, or
another carrier, to survive.

| | |
|---|---|
| **Commander** | Cloud, Midgar Mercenary ({W}{W}, 2/1, white) |
| **Bracket** | 3 (2 Game Changers: Smothering Tithe, Enlightened Tutor) |
| **Simulator** | deck key `cloud`; results in `decklists/pool/pool-results.md`, history in `decklists/CHANGELOG.md` |

## Strategy

### The commander
**Cloud, Midgar Mercenary** ({W}{W}, 2/1 Human Soldier Mercenary) reads: *When Cloud enters, search your library for an
Equipment card, reveal it, put it into your hand, then shuffle. As long as Cloud is equipped, if a triggered ability
of Cloud or an Equipment attached to it triggers, that ability triggers an additional time.*

- **A tutor every time he enters.** He is cheap to cast and to recast from the command zone (the commander tax adds
  {2} for each earlier cast from there). The deck has several ways to make him enter again.
- **The doubling is narrow.** It applies only while Cloud himself is equipped, and only to triggers of Cloud or of an
  Equipment attached to *Cloud*. Equipment on another creature never doubles, and a static bonus (+X/+X, hexproof)
  does not trigger at all. See "Doubled triggers" below for which Equipment actually gains.
- **What breaks him.** He is a 2/1 for {W}{W}: targeted removal, blockers and wipes. Protection and backup carriers
  are part of the plan, not an afterthought.

### Game plan
- **Turns 1-2.** Cast Cloud on turn two ({W}{W}) or play a cheap mana rock (Sol Ring, Mind Stone, Arcane Signet) and
  then Cloud. His search finds a cheap Sword or Champion's Helm. Cheap creatures (Kor Duelist, Zack Fair, Giver of
  Runes, Forum Familiar) are fine early plays.
- **Turns 3-4.** Cast and equip a Sword (equip {2}) on Cloud and attack. Use the free-equip pieces (Ardenn, Kor
  Outfitter, Beatrix) to save mana, and keep protection up. Enlightened Tutor finds the next piece.
- **Turns 5+.** Add the big Equipment (Sword of Kaldra, Wrecking Ball Arm, Colossus Hammer), the Kaldra set, and
  the removal. If Cloud dies, a backup carrier takes the Swords.

### Ramp and mana
Sol Ring, Mind Stone, Arcane Signet, Fellwar Stone, Coalition Relic and Thought Vessel; Page, Loose Leaf (a 0/2
artifact creature that taps for {C}); Solemn Simulacrum; Knight of the White Orchid and Loyal Warhound, which fetch
a Plains when an opponent controls more lands than you; and Smothering Tithe, which makes Treasures when opponents
draw and don't pay {2}. The deck wants Cloud out on turn two and a Sword equipped by turn four.

### Doubled triggers
With Cloud equipped, these Equipment gain from the extra trigger:
- **Sword of Fire and Ice**: 2 damage to any target twice and two cards.
- **Sword of Feast and Famine**: two discards from the player you hit; your lands untap (the second untap adds
  nothing).
- **Sword of Hearth and Home**: two lands and up to two creatures blinked. Point one trigger at a creature with a
  useful enter effect (Solemn Simulacrum, Knight of the White Orchid, Loyal Warhound, Kor Outfitter) or at Cloud
  himself for another search (his Equipment falls off when he leaves, so only when you can re-equip); do not aim both
  at Cloud, as the second would fizzle and lose its land.
- **Mask of Memory**: two triggers of draw two then discard one.
- **Explorer's Scope**: two looks at your top card whenever Cloud attacks, putting each land you find onto the battlefield tapped.
- **White Mage's Staff**: it gives Cloud *Whenever this creature attacks, you gain 1 life*, which is Cloud's own
  trigger, so you gain 2.

The doubling does nothing useful for **Sword of Kaldra** (the second exile has no target left), **Nim Deathmantle**
(the card is returned once) or **Hero's Blade** (it can't attach twice). **Champion's Helm**, **Swiftfoot Boots**,
**Colossus Hammer**, **Veteran's Powerblade**, **Shield of Kaldra**, **Helm of Kaldra** and **Wrecking Ball Arm** have
no triggers: they are plain bonuses (hexproof, first strike, haste, +X/+X, indestructible).

### Making Cloud enter again
**Ennis, Debate Moderator** blinks Cloud (he returns at the next end step, so do it after combat; his Equipment falls
off and has to be re-equipped, so it suits a turn with Ardenn or Beatrix out), **Forum Familiar**
and **Unyielding Gatekeeper** do it when turned face up from disguise, and **Stormfront Riders** returns two creatures
to your hand (and makes a 1/1 Soldier for each). **Recommission** and **Unfinished Business** bring him back from the
graveyard, Unfinished Business with up to two Equipment attached. **Court of Ardenvale** returns a permanent with mana
value 3 or less from the graveyard each upkeep (to the battlefield while you are the monarch, which Court makes you).

### Free equips
- **Ardenn, Intrepid Archaeologist** moves any number of Equipment onto one permanent at the start of each combat.
- **Beatrix, Loyal General** does the same for a creature you control (she is a 4/4 vigilance).
- **Kor Outfitter** attaches one Equipment when it enters.
- **Armory Automaton** attaches any number of Equipment to itself when it enters or attacks.
- **Armored Skyhunter** flies; whenever it attacks it looks at the top six cards and puts an Equipment onto the
  battlefield, attached to a creature (Cloud) if you like.
- **Balan, Wandering Knight** moves every Equipment onto himself for {1}{W}; with two or more attached he has double
  strike.
- **Puresteel Paladin** draws a card whenever an Equipment enters and, with three or more artifacts, gives your
  Equipment equip {0}. This is what makes Colossus Hammer (equip {8}) and Sword of Kaldra (equip {4}) practical.
- **Auriok Steelshaper** reduces equip costs by {1}, and gives Soldiers and Knights (Cloud is a Soldier) +1/+1 while
  it is equipped; **Helitrooper** makes equipping it cheaper by {2} and gives another attacker flying; **Cid, Freeflier
  Pilot** makes Equipment spells cost {1} less, flies on your turn and returns an Equipment from the graveyard.
- **Veteran's Powerblade** equips a Soldier for {W}; Cloud is one.
- **White Mage's Staff** makes its own 1/1 Hero token and attaches itself to it.

### Backup carriers
When Cloud is dead or removed, **Balan, Wandering Knight**, **Kor Duelist** (double strike when equipped),
**Armored Skyhunter**, **Bronze Guardian** (double strike; power equal to your artifacts; ward {2}, and other
artifacts get ward {2}), **Exalted Angel** (cast face down for {3}, flip for {2}{W}{W}), **Adelbert Steiner** (lifelink,
+1/+1 for each Equipment), **Thalia, the Survivor** (lifelink, taxes noncreature spells) and **Jocasta, Automaton Avenger**
(flying; grows when your commander connects, and comes back from the graveyard when you attack with him) all carry the
Swords. **Zack Fair** gives his counters and Equipment to another creature when he is sacrificed, which also makes
that creature indestructible.

### Making a 2/1 lethal
- **Wrecking Ball Arm** equips a legendary creature for {3}: base 7/7, and it can't be blocked by creatures with power
  2 or less. Cloud with it and a Sword of Fire and Ice is a 9/9.
- **Sword of Kaldra** gives +5/+5 and exiles any creature it damages. **Colossus Hammer** gives +10/+10.
- **Hero's Blade** (+3/+2) attaches itself when a legendary creature enters.
- **Helm of Kaldra** gives first strike, trample and haste. With all three Kaldra pieces out, pay {1}: a legendary
  Kaldra token wears all three (a 9/9 indestructible haste first striker).
- **Champion's Helm** (+2/+2; hexproof on a legendary creature) and **Swiftfoot Boots** (hexproof and haste) both
  protect and push.

### Keeping Cloud alive
- **Prevent.** Champion's Helm and Swiftfoot Boots (hexproof), Giver of Runes (protection from a colour for another
  creature), Grand Abolisher (opponents can't cast spells or activate abilities of artifacts, creatures or enchantments
  during your turn), Plaza of Heroes ({3}, exile it: a legendary creature gains hexproof and indestructible),
  Bastion Protector (commander creatures get +2/+2 and indestructible) and Shield of Kaldra (indestructible).
- **Protect the team.** Unbreakable Formation (creatures indestructible; in your main phase also a +1/+1 counter and
  vigilance for each), Rootborn Defenses (creatures indestructible, populate) and Ultimate Magic: Holy (every
  permanent you control indestructible). Cast Holy in response to your own Austere Command to keep your side.

### Removal
- **Single targets:** Lay Down Arms ({W}: exiles a creature with mana value up to the number of Plains you control, so
  it gets better later), Stand Up for Yourself (power 3 or more), Citizen's Arrest, Secure the Scene, Fate of the
  Sun-Cryst (costs {2} less against a tapped creature), Skyclave Apparition (mana value 4 or less) and Unyielding
  Gatekeeper.
- **Artifacts and enchantments:** Disenchant.
- **Board wipe:** Austere Command (choose two modes). Choose "destroy creatures with mana value 4 or greater" with
  "destroy all enchantments" to leave Cloud (mana value 2) and the artifacts (it also kills your own larger creatures),
  or cast Holy in response so that none of your permanents are destroyed.

### How it wins
- **Equipped Cloud connects.** A Sword of Fire and Ice on Cloud draws two cards and deals 4 damage; with Sword of Feast
  and Famine the opponent discards two cards and your lands untap. Add Wrecking Ball Arm or Sword of Kaldra for a
  9/9 or larger evasive attacker.
- **The Kaldra token.** All three Kaldra pieces and {1} make a 9/9 indestructible first striker with trample.
- **Wide Equipment.** Balan with two Equipment has double strike; Bronze Guardian with a pile of artifacts hits for
  7 or more twice.

### Weak spots and how to play around them
- **Board wipes.** Don't overcommit creatures when you already have enough power. Hold Holy, Rootborn Defenses or
  Unbreakable Formation up when the table has wiped before.
- **Artifact and enchantment removal.** Spread the Equipment across several carriers (Balan, Beatrix and Ardenn can
  re-attach after a loss) and recur them with Cid or Unfinished Business.
- **Cloud dead, command zone.** Remember the {2} commander tax; use the backup carriers first.
- **Small bodies.** Giver of Runes, Grand Abolisher and Bastion Protector keep the board safe; Thalia and
  Skyclave Apparition slow the table down.

## Key lines

- **Turn 2 Cloud:** search a Sword or Champion's Helm; turn three cast and equip it and attack with protection up
  (Swiftfoot Boots or Giver of Runes).
- **Best opening:** Sol Ring on turn one, Cloud on turn two, then cast and equip a Sword on turn three.
- **Mulligan:** keep hands with two lands (or a land and Sol Ring) and a play of two mana or less. Throw back hands that
  can not make {W}{W} by turn three.
- **Big turn:** Puresteel Paladin out and three artifacts means equip {0}: cast Colossus Hammer, Sword of Kaldra
  and a Sword, attach all, and attack.
- **Holy trick:** if you expect a wipe, cast Austere Command with Ultimate Magic: Holy in response to wipe their board
  and keep yours.
- **Removal in response to an equip:** have Giver of Runes, Zack Fair or Swiftfoot Boots ready.

## Tutor targets by board state

- **Cloud's search, early:** a cheap Sword (Fire and Ice, Feast and Famine, Hearth and Home) or Champion's Helm.
- **Cloud's search, Puresteel Paladin out:** Colossus Hammer or Sword of Kaldra.
- **Cloud's search, many blockers:** Wrecking Ball Arm.
- **Cloud's search, facing removal:** Swiftfoot Boots or Champion's Helm.
- **Enlightened Tutor** (puts the card on top, so cast it at the end of an opponent's turn; it finds an artifact or
  enchantment): a Sword if you are missing one, Smothering Tithe when opponents draw a lot, Court of Ardenvale
  when you need card advantage, Helm of Kaldra once you hold the other two Kaldra pieces.

## Mana and consistency

33 lands: 28 Plains plus Ash Barrens (basic landcycling {1}), Evolving Wilds, Path of Ancestry (enters tapped; scry 1
when you cast a creature that shares a type with Cloud), Plaza of Heroes and Terminal Moraine. Ramp: Sol Ring, Mind Stone,
Arcane Signet, Fellwar Stone, Coalition Relic, Thought Vessel, Page, Loose Leaf, Solemn Simulacrum, Knight of the
White Orchid and Loyal Warhound. Card draw: Mask of Memory, Court of Ardenvale, Puresteel Paladin and the Swords.
Interaction: Lay Down Arms, Stand Up for Yourself, Citizen's Arrest, Secure the Scene, Fate of the Sun-Cryst, Skyclave
Apparition, Austere Command and Disenchant. 16 Equipment in the deck, so Cloud's search always finds one.

## Bracket and Rule 0

**Bracket 3,** with two Game Changers: **Smothering Tithe** and **Enlightened Tutor**. The deck has no combo that wins
on its own. Tell the table before the game that:

- the deck has two tutors (Cloud and Enlightened Tutor) but no game-winning combo;
- Austere Command can wipe the table, and Ultimate Magic: Holy lets you keep your own board.

## Decklist by type (100)

**Commander (1).** Cloud, Midgar Mercenary

**Creatures (29).** Adelbert Steiner, Ardenn Intrepid Archaeologist, Armored Skyhunter, Armory Automaton, Auriok Steelshaper, Balan Wandering Knight, Bastion Protector, Beatrix Loyal General, Bronze Guardian, Cid Freeflier Pilot, Ennis Debate Moderator, Exalted Angel, Forum Familiar, Giver of Runes, Grand Abolisher, Helitrooper, Jocasta Automaton Avenger, Knight of the White Orchid, Kor Duelist, Kor Outfitter, Loyal Warhound, Page Loose Leaf, Puresteel Paladin, Skyclave Apparition, Solemn Simulacrum, Stormfront Riders, Thalia the Survivor, Unyielding Gatekeeper, Zack Fair

**Enchantments (3).** Citizen's Arrest, Court of Ardenvale, Smothering Tithe

**Artifacts (22).** Arcane Signet, Champion's Helm, Coalition Relic, Colossus Hammer, Explorer's Scope, Fellwar Stone, Helm of Kaldra, Hero's Blade, Mask of Memory, Mind Stone, Nim Deathmantle, Shield of Kaldra, Sol Ring, Swiftfoot Boots, Sword of Feast and Famine, Sword of Fire and Ice, Sword of Hearth and Home, Sword of Kaldra, Thought Vessel, Veteran's Powerblade, White Mage's Staff, Wrecking Ball Arm

**Sorceries (5).** Austere Command, Lay Down Arms, Recommission, Secure the Scene, Unfinished Business

**Instants (7).** Disenchant, Enlightened Tutor, Fate of the Sun-Cryst, Rootborn Defenses, Stand Up for Yourself, Ultimate Magic: Holy, Unbreakable Formation

**Lands (33).** Ash Barrens, Evolving Wilds, Path of Ancestry, Plaza of Heroes, Terminal Moraine, 28 Plains

## Import list (100)

```
1 Adelbert Steiner
1 Arcane Signet
1 Ardenn, Intrepid Archaeologist
1 Armored Skyhunter
1 Armory Automaton
1 Ash Barrens
1 Auriok Steelshaper
1 Austere Command
1 Balan, Wandering Knight
1 Bastion Protector
1 Beatrix, Loyal General
1 Bronze Guardian
1 Champion's Helm
1 Cid, Freeflier Pilot
1 Citizen's Arrest
1 Cloud, Midgar Mercenary
1 Coalition Relic
1 Colossus Hammer
1 Court of Ardenvale
1 Disenchant
1 Enlightened Tutor
1 Ennis, Debate Moderator
1 Evolving Wilds
1 Exalted Angel
1 Explorer's Scope
1 Fate of the Sun-Cryst
1 Fellwar Stone
1 Forum Familiar
1 Giver of Runes
1 Grand Abolisher
1 Helitrooper
1 Helm of Kaldra
1 Hero's Blade
1 Jocasta, Automaton Avenger
1 Knight of the White Orchid
1 Kor Duelist
1 Kor Outfitter
1 Lay Down Arms
1 Loyal Warhound
1 Mask of Memory
1 Mind Stone
1 Nim Deathmantle
1 Page, Loose Leaf
1 Path of Ancestry
1 Plaza of Heroes
1 Puresteel Paladin
1 Recommission
1 Rootborn Defenses
1 Secure the Scene
1 Shield of Kaldra
1 Skyclave Apparition
1 Smothering Tithe
1 Sol Ring
1 Solemn Simulacrum
1 Stand Up for Yourself
1 Stormfront Riders
1 Swiftfoot Boots
1 Sword of Feast and Famine
1 Sword of Fire and Ice
1 Sword of Hearth and Home
1 Sword of Kaldra
1 Terminal Moraine
1 Thalia, the Survivor
1 Thought Vessel
1 Ultimate Magic: Holy
1 Unbreakable Formation
1 Unfinished Business
1 Unyielding Gatekeeper
1 Veteran's Powerblade
1 White Mage's Staff
1 Wrecking Ball Arm
1 Zack Fair
28 Plains
```
