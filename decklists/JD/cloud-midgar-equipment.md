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
  then Cloud. His search finds a cheap Sword or Champion's Helm. Cheap creatures (Zack Fair, Giver of
  Runes, Forum Familiar) are fine early plays.
- **Turns 3-4.** Cast and equip a Sword (equip {2}) on Cloud and attack. Use the free-equip pieces (Ardenn, Kor
  Outfitter, Balan) to save mana, and keep protection up. Enlightened Tutor finds the next piece.
- **Turns 5+.** Add the big Equipment (Sword of Kaldra, Colossus Hammer), Open the Armory (search for a Sword or any Aura or Equipment), and
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
- **Buster Sword**: two triggers of draw a card, then cast a spell from your hand free with mana value up to that
  damage.

The doubling does nothing useful for **Sword of Kaldra** (the second exile has no target left). **Champion's Helm**, **Swiftfoot Boots**,
**Colossus Hammer** and **Fireshrieker** have
no triggers: they are plain bonuses (hexproof, double strike, +X/+X).

### Making Cloud enter again
**Forum Familiar** returns Cloud (or any creature) to your hand when turned face up from disguise, and
**Court of Ardenvale** returns a permanent with mana value 3 or less from the
graveyard each upkeep (to the battlefield while you are the monarch, which Court makes you). **Kor Outfitter**,
**Solemn Simulacrum**, **Knight of the White Orchid** and **Loyal Warhound** are worth a second enter if you have a way
to blink them.

### Free equips
- **Ardenn, Intrepid Archaeologist** moves any number of Equipment onto one permanent at the start of each combat.
- **Kor Outfitter** attaches one Equipment when it enters.
- **Forge Anew** ({2}{W}, enchantment) returns an Equipment from your graveyard to the battlefield when it enters; on
  your turn you equip at instant speed, and the first equip each turn costs {0}.
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
- **Fireshrieker** equips for {2} and gives double strike; Akroma's Will gives it for a turn, and Silverblade Paladin gives it to itself and its soulbond partner.

### Backup carriers
When Cloud is dead or removed, **Balan, Wandering Knight**, **Danitha Capashen, Paragon** (first strike, vigilance, lifelink;
Aura and Equipment spells cost {1} less), **Armored Skyhunter**, **Exalted Angel** (cast face down for {3}, flip for {2}{W}{W}), **Adelbert Steiner** (lifelink,
+1/+1 for each Equipment), **Thalia, the Survivor** (lifelink, taxes noncreature spells) and **Jocasta, Automaton Avenger**
(flying; grows when your commander connects, and comes back from the graveyard when you attack with him) all carry the
Swords. **Zack Fair** gives his counters and Equipment to another creature when he is sacrificed, which also makes
that creature indestructible.

### Making a 2/1 lethal
- **Buster Sword** ({3}, equip {2}) gives +3/+2: Cloud is a 5/3. Each time he hits a player you draw a card, then may
  cast a spell from your hand free with mana value up to the damage dealt, so a 5-damage hit casts most of the deck,
  and getting the hand onto the battlefield is the deck's bottleneck.
- **Sword of Kaldra** gives +5/+5 and exiles any creature it damages. **Colossus Hammer** gives +10/+10.
- **Andúril, Narsil Reforged** ({2}, equip {3}): whenever the equipped creature attacks, put a +1/+1 counter on each creature you control, or two with the city's blessing (ten or more permanents).
- **Double strike** makes every combat hit count twice: Fireshrieker (equip {2}) gives it to the equipped creature, so
  Cloud with a Sword triggers it at first-strike and regular damage (four Sword triggers per attack with his doubling)
  and deals commander damage twice. **Silverblade Paladin** ({1}{W}, soulbond) gives itself and the creature it pairs with double strike: pair it with
  Cloud and he deals both Sword triggers and commander damage twice. Akroma's Will ({3}{W}, instant) gives your creatures flying, vigilance and
  double strike until end of turn; with a commander out you may also choose lifelink, indestructible and protection from
  each color.
- **Champion's Helm** (+2/+2; hexproof on a legendary creature) and **Swiftfoot Boots** (hexproof and haste) both
  protect and push.

### Keeping Cloud alive
- **Prevent.** Champion's Helm and Swiftfoot Boots (hexproof), Giver of Runes (protection from a colour for another
  creature), Grand Abolisher (opponents can't cast spells or activate abilities of artifacts, creatures or enchantments
  during your turn), Plaza of Heroes ({3}, exile it: a legendary creature gains hexproof and indestructible),
  Bastion Protector (commander creatures get +2/+2 and indestructible).
- **Answer in response.** **Galadriel's Dismissal** ({W}, instant) phases Cloud out in response to removal; his
  Equipment phases out with him and comes back still attached at your next turn.
- **Protect the team.** Kicked ({2}{W} more), Galadriel's Dismissal phases out every creature you control, an answer
  to a board wipe. Elspeth, Sun's Champion makes three 1/1 Soldiers each turn (+1) as a wall of blockers and carriers; her -3 destroys creatures with
  power 4 or greater, so use it only when Cloud is not carrying a big Equipment.

### Removal
- **Single targets:** Winds of Abandon ({1}{W}, exile; overload {4}{W}{W} exiles all your opponents' creatures), Lay Down Arms ({W}: exiles a creature with mana value up to the number of Plains you control, so
  it gets better later), Stand Up for Yourself (power 3 or more), Fate of the
  Sun-Cryst (costs {2} less against a tapped creature), Skyclave Apparition (mana value 4 or less) and Stroke of Midnight ({2}{W} instant: destroys any nonland permanent, and its controller gets a 1/1 Human).
- **Artifacts and enchantments:** Disenchant.
- **Attack limits and taxes:** **Archangel of Tithes** ({1}{W}{W}{W}, a 3/5 flier: while untapped, opponents pay {1} for each creature that attacks you, and while it attacks, they pay {1} for each creature that blocks). With her too,
  creatures cannot attack you unless their controller pays per attacker: Windborn Muse {2} (a 2/3 flier), Baird,
  Steward of Argive {1} (a 1/4 vigilance), Norn's Annex {W/P}, and Archon of Absolution {1} (a 3/2 flier with
  protection from white).
- **Board wipes:** Wrath of God, Day of Judgment and Fumigate (gain 1 life per creature destroyed). Equipment survives them, and Cloud comes back from the command zone.
- **Austere Command** (choose two modes): choose "destroy creatures with mana value 4 or greater" with
  "destroy all enchantments" to leave Cloud (mana value 2) and the artifacts (it also kills your own larger creatures).
  Winds of Abandon's overload is the cleaner one-sided wipe when you need opposing creatures gone and want to keep your own.

### How it wins
- **Equipped Cloud connects.** A Sword of Fire and Ice on Cloud draws two cards and deals 4 damage; with Sword of Feast
  and Famine the opponent discards two cards and your lands untap. Add Sword of Kaldra or Colossus Hammer for a
  9/9 or larger attacker, or Buster Sword to cast a spell free with every hit.
- **Wide Equipment.** Balan with two Equipment has double strike; Danitha makes every Equipment {1} cheaper, and
  **Sram, Senior Edificer** draws a card whenever you cast an Aura, Equipment or Vehicle spell.

### Weak spots and how to play around them
- **Board wipes.** Don't overcommit creatures when you already have enough power. Keep Elspeth's Soldiers
  and the backup carriers in hand when the table has wiped before.
- **Artifact and enchantment removal.** Spread the Equipment across several carriers (Balan and Ardenn can
  re-attach after a loss) and recur them with Cid or Forge Anew.
- **Cloud dead, command zone.** Remember the {2} commander tax; use the backup carriers first.
- **Small bodies.** Giver of Runes, Grand Abolisher and Bastion Protector keep the board safe; Thalia and
  Skyclave Apparition slow the table down.

## Key lines

- **Turn 2 Cloud:** search a Sword or Champion's Helm; turn three cast and equip it and attack with protection up
  (Swiftfoot Boots, Giver of Runes or Galadriel's Dismissal).
- **Best opening:** Sol Ring on turn one, Cloud on turn two, then cast and equip a Sword on turn three.
- **Mulligan:** keep hands with two lands (or a land and Sol Ring) and a play of two mana or less. Throw back hands that
  can not make {W}{W} by turn three.
- **Big turn:** Puresteel Paladin out and three artifacts means equip {0}: cast Colossus Hammer, Sword of Kaldra
  and a Sword, attach all, and attack.
- **Winds of Abandon:** {1}{W} exiles one creature that is not yours (its controller fetches a basic land); at six mana
  overload it ({4}{W}{W}) to exile every creature you do not control, a one-sided wipe that leaves your Equipment and carriers.
- **Removal in response to an equip:** have Giver of Runes, Zack Fair, Galadriel's Dismissal or Swiftfoot Boots ready.

## Tutor targets by board state

- **Cloud's search, early:** a cheap Sword (Fire and Ice, Feast and Famine, Hearth and Home) or Champion's Helm.
- **Cloud's search, Puresteel Paladin out:** Colossus Hammer or Sword of Kaldra.
- **Cloud's search, a full hand to deploy:** Buster Sword (each hit casts a spell from hand free).
- **Cloud's search, facing removal:** Swiftfoot Boots or Champion's Helm.
- **Enlightened Tutor** (puts the card on top, so cast it at the end of an opponent's turn; it finds an artifact or
  enchantment): a Sword if you are missing one, Smothering Tithe when opponents draw a lot, Court of Ardenvale
  when you need card advantage, or Forge Anew to bring back an Equipment that was destroyed.

## Mana and consistency

33 lands: 28 Plains plus Ash Barrens (basic landcycling {1}), Evolving Wilds, Path of Ancestry (enters tapped; scry 1
when you cast a creature that shares a type with Cloud), Plaza of Heroes and Terminal Moraine. Ramp: Sol Ring, Mind Stone,
Arcane Signet, Fellwar Stone, Coalition Relic, Thought Vessel, Page, Loose Leaf, Solemn Simulacrum, Knight of the
White Orchid and Loyal Warhound. Card draw: Mask of Memory, Court of Ardenvale, Puresteel Paladin and the Swords.
Interaction: Wrath of God, Day of Judgment, Fumigate, Archangel of Tithes, Archon of Absolution, Norn's Annex, Lay Down Arms, Stand Up for Yourself, Stroke of Midnight, Fate of the Sun-Cryst, Skyclave
Apparition, Winds of Abandon, Austere Command, Elspeth, Sun's Champion and Disenchant. 11 Equipment in the deck, so Cloud's search always finds one.

## Bracket and Rule 0

**Bracket 3,** with two Game Changers: **Smothering Tithe** and **Enlightened Tutor**. The deck has no combo that wins
on its own. Tell the table before the game that:

- the deck has two tutors (Cloud and Enlightened Tutor) but no game-winning combo;
- Austere Command can wipe the table, and Winds of Abandon (overloaded) and Elspeth, Sun's Champion can clear opposing creatures.

## Decklist by type (100)

**Commander (1).** Cloud, Midgar Mercenary

**Creatures (30).** Adelbert Steiner, Archangel of Tithes, Archon of Absolution, Ardenn, Intrepid Archaeologist, Armored Skyhunter, Armory Automaton, Auriok Steelshaper, Baird, Steward of Argive, Balan, Wandering Knight, Bastion Protector, Cid, Freeflier Pilot, Danitha Capashen, Paragon, Exalted Angel, Forum Familiar, Giver of Runes, Grand Abolisher, Helitrooper, Jocasta, Automaton Avenger, Knight of the White Orchid, Kor Outfitter, Loyal Warhound, Page, Loose Leaf, Puresteel Paladin, Silverblade Paladin, Skyclave Apparition, Solemn Simulacrum, Sram, Senior Edificer, Thalia, the Survivor, Windborn Muse, Zack Fair

**Enchantments (3).** Court of Ardenvale, Forge Anew, Smothering Tithe

**Artifacts (18).** Andúril, Narsil Reforged, Arcane Signet, Buster Sword, Champion's Helm, Coalition Relic, Colossus Hammer, Fellwar Stone, Fireshrieker, Mask of Memory, Mind Stone, Norn's Annex, Sol Ring, Swiftfoot Boots, Sword of Feast and Famine, Sword of Fire and Ice, Sword of Hearth and Home, Sword of Kaldra, Thought Vessel

**Sorceries (7).** Austere Command, Day of Judgment, Fumigate, Lay Down Arms, Open the Armory, Winds of Abandon, Wrath of God

**Instants (7).** Akroma's Will, Disenchant, Enlightened Tutor, Fate of the Sun-Cryst, Galadriel's Dismissal, Stand Up for Yourself, Stroke of Midnight

**Lands (33).** Ash Barrens, Evolving Wilds, Path of Ancestry, Plaza of Heroes, Terminal Moraine, 28 Plains

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
1 Champion's Helm
1 Cid, Freeflier Pilot
1 Cloud, Midgar Mercenary
1 Coalition Relic
1 Colossus Hammer
1 Court of Ardenvale
1 Danitha Capashen, Paragon
1 Day of Judgment
1 Disenchant
1 Elspeth, Sun's Champion
1 Enlightened Tutor
1 Evolving Wilds
1 Exalted Angel
1 Fate of the Sun-Cryst
1 Fellwar Stone
1 Fireshrieker
1 Forge Anew
1 Forum Familiar
1 Fumigate
1 Galadriel's Dismissal
1 Giver of Runes
1 Grand Abolisher
1 Helitrooper
1 Jocasta, Automaton Avenger
1 Knight of the White Orchid
1 Kor Outfitter
1 Lay Down Arms
1 Loyal Warhound
1 Mask of Memory
1 Mind Stone
1 Norn's Annex
1 Open the Armory
1 Page, Loose Leaf
1 Path of Ancestry
1 Plaza of Heroes
1 Puresteel Paladin
1 Silverblade Paladin
1 Skyclave Apparition
1 Smothering Tithe
1 Sol Ring
1 Solemn Simulacrum
1 Sram, Senior Edificer
1 Stand Up for Yourself
1 Stroke of Midnight
1 Swiftfoot Boots
1 Sword of Feast and Famine
1 Sword of Fire and Ice
1 Sword of Hearth and Home
1 Sword of Kaldra
1 Terminal Moraine
1 Thalia, the Survivor
1 Thought Vessel
1 Windborn Muse
1 Winds of Abandon
1 Wrath of God
1 Zack Fair
28 Plains
```
