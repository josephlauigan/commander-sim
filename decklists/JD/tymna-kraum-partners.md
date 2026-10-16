# Tymna the Weaver & Kraum, Ludevic's Opus — Four-Colour Partners (W/U/B/R)

Two commanders who draw cards without spending mana: Tymna draws one card for each opponent your creatures hit in
combat, and Kraum draws whenever an opponent casts their second spell in a turn. The deck is full of cheap creatures
that are hard to block (fliers, menace, protection, "can't be blocked by"), Equipment that makes more of them
unblockable, and a wide spread of removal and counterspells. It wins by chipping every opponent every turn, drawing far
more cards than they do, and finishing with Terror of the Peaks, Lyra, Gilgamesh and a pumped team. It is weak to
board wipes, slow to set up its four-colour mana, and pays life for every Tymna draw.

| | |
|---|---|
| **Commanders** | Tymna the Weaver ({1}{W}{B}, 2/2) and Kraum, Ludevic's Opus ({3}{U}{R}, 4/4); white-blue-black-red |
| **Bracket** | 3 (1 Game Changer: Force of Will) |
| **Simulator** | deck key `kraum`; history in `decklists/CHANGELOG.md`, findings in `audit/kraum/FINDINGS.md` |

## Strategy

### The commanders

**Tymna the Weaver** ({1}{W}{B}, 2/2 lifelink). *At the beginning of each of your postcombat main phases, you may pay X
life, where X is the number of opponents that were dealt combat damage this turn. If you do, draw X cards.*

**Kraum, Ludevic's Opus** ({3}{U}{R}, 4/4 flying, haste). *Whenever an opponent casts their second spell each turn,
draw a card.*

Both have plain Partner, so they share the command zone and either can be cast at any time. Tymna is the engine: with
three opponents, hitting all of them is three cards a turn for 3 life. What she counts is *opponents* hit, not
creatures, so spreading attackers across the table matters more than hitting one player hard. Kraum is a hasty 4/4
flier, so he hits someone (and turns on Tymna) the turn he lands, and his draw trigger needs no work from you: in a
four-player game someone casts two spells most turns, and Kraum taxes the spell-heavy decks the most.

### Game plan

**Early turns (1–3).** A cheap evasive creature on turn 2 (The Wasp, Exit Specialist, Spectral Lynx, Moseo,
Nightwind Glider), then **Tymna on turn 3**. A Talisman on turn 2 is just as good: Kraum on turn 4 off a Talisman is a
strong start. Keep the hand's removal for real threats.

**Middle turns (4–6).** Kraum comes down and attacks at once. Attack a different opponent with each creature where you
can, and draw with Tymna after combat. Add Equipment to the creatures that can't get through on their own. Hold up a
counterspell when the table has a threat you can't remove.

**Late game.** Tymna and Kraum keep the hand full, so the late game is spending the cards: Terror of the Peaks turns
every creature you cast into damage, Lyra and Gilgamesh are big threats, Firebird pumps the whole attack, and Conqueror's
Flail stops the table from casting spells while you swing.

### Getting through: evasion for Tymna

Most creatures in the deck are hard to block, because each one that connects is a card:

- **Fliers:** The Wasp (flash; taps a creature when it attacks), Falcon and Redwing (makes a flying Bird for each
  damage it deals), Nightwind Glider (protection from black), Moseo, Abigale, Aven Mindcensor (flash), Elemental
  Mascot, Firebird, Photon (adds mana when it connects), Nightveil Predator (hexproof, deathtouch), Sphinx of New Prahv,
  Lyra, Terror of the Peaks, and Kraum.
- **Ground evasion:** Exit Specialist (can't be blocked by power 3 or more), Lightning, Security Sergeant and Black
  Widow (menace), Spectral Lynx (protection from green, regenerates).
- **Removing blockers for a turn:** The Wasp taps a creature whenever it attacks and Vibrant Outburst taps one at
  instant speed; Marchesa's dethrone grows every attacker that goes at the player with the most life.

### Gilgamesh and the Equipment

**Gilgamesh, Master-at-Arms** ({4}{R}{R}, 6/6) looks at the top six cards whenever he enters or attacks and puts every
Equipment among them onto the battlefield for free, then may attach one to a Samurai (himself, or the 1/1 Hero token
from **Samurai's Katana**). With eleven Equipment in the deck, each trigger finds at least one about 52% of the
time, and the expensive pieces (Shield of Kaldra, The Aetherspark, Helm of Kaldra) come in for free.

The Equipment is chosen to get creatures through, so it feeds Tymna even without Gilgamesh:

- **Trailblazer's Boots** (nonbasic landwalk): unblockable against almost every Commander deck.
- **Medic's Kitesail** and **Dragoon's Lance** give flying (the Lance only on your turn, and it comes with a 1/1 Hero
  to carry it). **Nim Deathmantle** gives intimidate and +2/+2, and later returns a nontoken creature that died for
  {4}.
- **Conqueror's Flail** gives +1/+1 for each colour among your permanents (+4/+4 with all four out), and while it is
  attached your opponents can't cast spells during your turn: no combat tricks, no instant-speed removal, no fog.
- **Hero's Heirloom** gives a legendary creature trample and haste; **Hero's Blade** (+3/+2) attaches itself for free
  whenever a legendary creature enters (both commanders, Gilgamesh, Lyra, Elenda, Marchesa, Laelia and others).
- **Samurai's Katana** makes a 1/1 Samurai Hero with +2/+2, trample and haste, a target for Gilgamesh.
- **The Aetherspark** attaches itself to a creature (+1: and a +1/+1 counter) and grows from its combat damage.
- **Shield of Kaldra** (equip {4}) makes the equipped creature indestructible: put it on Kraum or the best threat, so
  it survives destroy effects and damage wipes. **Helm of Kaldra** (equip {2}) gives first strike, trample and haste:
  put it on Gilgamesh, Lyra or the biggest attacker. Helm's {1} ability makes the 9/9 Kaldra token only if you also
  control Sword of Kaldra, which this deck doesn't have, so ignore it.

Equipment costs mana twice (cast, then equip), so in the early turns play creatures first, and equip when you have
spare mana or when one creature is stopped by blockers.

### Removal

Thirteen answers, most of them instants or cheap:

- **Creatures:** Harsh Annotation (2 mana, leaves a 1/1 flier), Scorching Dragonfire (3 damage, exiles), Vibrant
  Outburst (3 damage to any target and taps a creature), Silverquill Charm (exile power 2 or less, or drain each
  opponent 3), Wander Off (exile), Darksteel Mutation (turns a creature or commander into a 0/1 with no abilities).
- **Any nonland permanent:** Bedevil (artifact, creature or planeswalker), Fate of the Sun-Cryst (2 less on a tapped
  creature), Memory Trap, Vanish from Sight (top or bottom of the library, owner's choice).
- **Artifacts and enchantments:** Disenchant, Make Your Move (also a creature with power 4 or more).
- **Sweeper:** Toxic Deluge, the only one. Pay just enough life to kill their creatures; your fliers are small, so it
  usually kills yours too: cast it when you are behind on board, not to clean up.

### Counterspells and protection

**Force of Will** (free by exiling a blue card), **Arcane Denial** and **Dissipate** stop the spell that would beat you:
a wipe, a combo piece, a big threat. **Aven Mindcensor** (flash) blunts tutors and land searches.
**Bastion Protector** gives both commanders +2/+2 and indestructible, which keeps them alive through destroy effects and
damage wipes, and **Shield of Kaldra** does the same for any one creature. **Conqueror's Flail** also protects your
turn: nobody can cast spells while you attack.

### Card advantage beyond the commanders

**Black Widow** draws whenever an opponent draws their second card in a turn (in Commander, most turns).
**Tocasia's Welcome** draws once a turn when a cheap creature enters. **Laelia** and **Lightning** play cards off the
top of the library when they attack or connect, and **Caparocti Sunborn** discovers 3 when it attacks (casts a free
spell of mana value 3 or less). **Quick Study**, **Fact or Fiction** and **Expressive Iteration** are the draw spells,
**Urza, Powerstone Prodigy** loots every turn, and **Diabolic Tutor** finds whichever card the game needs: Toxic
Deluge when behind, Terror of the Peaks or Conqueror's Flail to finish, Bastion Protector to keep the commanders
safe.

### How it wins

By combat damage spread over many turns, while the commanders bury the table in cards. A typical win: three or four
evasive creatures hitting every opponent, Tymna drawing three a turn, and a finisher on top: **Terror of the Peaks**
(each creature you cast deals its power to any target), **Lyra Dawnbringer** (5/5 flying, first strike, lifelink),
**Gilgamesh** with a free Equipment or two, **Firebird** (+X/+0 to every other attacker), **Elenda** (grows with
every creature death), **Garland** (makes an opponent the monarch, then steals their best creature while they keep the
crown), and **Falcon and Redwing** making a flock of Birds. Marchesa's dethrone puts counters on everything that
attacks the life leader.

### Weak spots and how to play around them

- **Board wipes.** The creatures are small and cheap, and the deck has only Bastion Protector (commanders only),
  Shield of Kaldra (one creature) and counterspells to protect them. Keep a counter up once you have a board, don't
  play more creatures than you need to keep Tymna drawing, and keep a creature or two in hand to rebuild.
- **Slow, tapped mana.** Twelve lands always enter tapped, and four colours with demanding costs (Nightveil Predator
  {U}{U}{B}{B}, Sphinx of New Prahv {W}{W}{U}{U}, Bedevil {B}{B}{R}, Lyra {3}{W}{W}, Terror of the Peaks {3}{R}{R},
  Gilgamesh {4}{R}{R}) can strand cards. Play the tapped lands in the early turns when nothing else needs the mana;
  keep Talismans and Chromatic Lantern as top priorities.
- **Life as a resource.** Tymna, Toxic Deluge, the pain lands, the Talismans and Force of Will all cost life. Tymna's
  lifelink, Lyra, Medic's Kitesail and Silverquill Charm gain some back. Tymna is all or nothing (X is the number of
  opponents hit), so against aggressive decks skip the draw on a turn you can't afford the life.
- **Large blockers and ground decks.** Tymna's draw stops when nothing gets through. Equipment (Boots, Kitesail, Lance)
  and the tap effects (The Wasp, Vibrant Outburst) are the answer; save them for that.
- **Commander removal.** Kraum costs 5 and each recast costs 2 more. Bastion Protector and Hero's Heirloom's haste help;
  don't send Kraum into a board where he will just trade.

## Key lines

- **Turn 3 Tymna, turn 4 Kraum** off a Talisman, then attack each opponent with a different creature and draw.
- **Kraum the turn he lands:** haste, so he connects and counts for Tymna immediately.
- **Trailblazer's Boots on a creature with nothing else:** almost every opponent plays nonbasic lands, so it is a card
  every turn.
- **Conqueror's Flail before the big attack:** nobody can cast a fog, trick or removal on your turn.
- **Terror of the Peaks, then creatures:** each creature you cast deals its power to any target.
- **Gilgamesh attacking:** each attack looks at six more cards for free Equipment.

**Mulligan:** keep 3+ lands that make at least three of the four colours, with a two-drop creature or a Talisman. A
hand that can't cast Tymna by turn 3 or 4 is a mulligan.

## Mana and consistency

36 lands: 25 nonbasic and 11 basic (3 Island, 3 Plains, 3 Swamp, 2 Mountain). Mana sources besides lands:
**Talisman of Hierarchy** and **Talisman of Conviction** (two mana, colour fixing), **Chromatic Lantern** (every land
taps for any colour), **Star Compass** and **Potioner's Trove**; **Photon** adds mana when it connects.

- **Any colour:** Command Tower, Exotic Orchard, Path of Ancestry (tapped), Vivid Creek (tapped, two uses), Unclaimed
  Territory (name **Human**: 15 of the creatures are Humans, Tymna and Gilgamesh among them).
- **Three colours (tapped):** Arcane Sanctum (W/U/B), Crumbling Necropolis (U/B/R), Mystic Monastery (U/R/W), Nomad
  Outpost (R/W/B).
- **Two colours, untapped:** Battlefield Forge, Caves of Koilos and Underground River (pain lands), Rugged Prairie
  (filter), Clifftop Retreat and Glacial Fortress (with a basic of the right type), Furycalm Snarl and Port Town (reveal
  a basic), Spectator Seating (untapped with two or more opponents), Shattered Sanctum (with two other lands).
- **Two colours, tapped:** Temple of Deceit, Temple of Epiphany, Temple of Malice (scry 1), Irrigated Farmland
  (cycling {2}), Spectacle Summit (surveil for {2}{U}{R}), Boiling Rock Prison (sacrifice to draw for {4}).

**Card draw:** Tymna, Kraum, Black Widow, Tocasia's Welcome, Quick Study, Fact or Fiction, Expressive Iteration,
Urza (loot), and the cards played off the top (Laelia, Lightning, Caparocti, Elemental Mascot). **Tutor:** Diabolic
Tutor.

**Interaction:** thirteen removal spells, three counterspells, Aven Mindcensor, and Bastion Protector. The deck has 25
creatures plus the two commanders, and eleven Equipment.

## Bracket and Rule 0

**Bracket 3,** with one Game Changer: **Force of Will**. No two-card infinite combos, no mass land destruction, no
extra turns. Mention Aven Mindcensor (it stops tutors and fetches) and Conqueror's Flail (no spells on your turn)
before the game.

## Decklist by type (100)

**Commanders (2).** Tymna the Weaver, Kraum, Ludevic's Opus

**Creatures (33).** Abigale, Poet Laureate, Adeline, Resplendent Cathar, Archon of Absolution, Aven Mindcensor, Baird, Steward of Argive, Baneslayer Angel, Bastion Protector, Black Widow, Agile Avenger, Callous Inspector, Cloudfin Raptor, Dauthi Voidwalker, Elemental Mascot, Elenda, the Dusk Rose, Falcon and Redwing, Firebird, Blazing Ranger, Gilgamesh, Master-at-Arms, Hero of Bladehold, Laelia, the Blade Reforged, Lightning, Security Sergeant, Lyra Dawnbringer, Marchesa, the Black Rose, Moseo, Vein's New Dean, Nightveil Predator, Nightwind Glider, Rescue, Pepper Potts, Skyclave Apparition, Spectral Sailor, Sphinx of New Prahv, Terror of the Peaks, The Wasp, Winsome Avenger, Thrummingbird, Vampire Nighthawk, Windborn Muse

**Planeswalkers (1).** Elspeth, Sun's Champion

**Artifacts (9).** Conqueror's Flail, Crawlspace, Helm of Kaldra, Lightning Greaves, Potioner's Trove, Shield of Kaldra, Star Compass, Talisman of Conviction, Talisman of Hierarchy

**Enchantments (5).** Darksteel Mutation, Ghostly Prison, Memory Trap, Propaganda, Tocasia's Welcome

**Instants (10).** Arcane Denial, Bedevil, Disenchant, Fact or Fiction, Force of Will, Harsh Annotation, Masterful Flourish, Path to Exile, Scorching Dragonfire, Swords to Plowshares

**Sorceries (4).** Diabolic Tutor, Expressive Iteration, Rip Apart, Toxic Deluge

**Lands (36).** Battlefield Forge, Caves of Koilos, Clifftop Retreat, Command Tower, Dragonskull Summit, Drowned Catacomb, Exotic Orchard, Furycalm Snarl, Glacial Fortress, Isolated Chapel, Port Town, Rugged Prairie, Shattered Sanctum, Spectator Seating, Sulfur Falls, Unclaimed Territory, Underground River, 5 Island, 6 Plains, 5 Swamp, 3 Mountain

## Import list (100)

```
1 Abigale, Poet Laureate
1 Adeline, Resplendent Cathar
1 Arcane Denial
1 Archon of Absolution
1 Aven Mindcensor
1 Baird, Steward of Argive
1 Baneslayer Angel
1 Bastion Protector
1 Battlefield Forge
1 Bedevil
1 Black Widow, Agile Avenger
1 Callous Inspector
1 Caves of Koilos
1 Clifftop Retreat
1 Cloudfin Raptor
1 Command Tower
1 Conqueror's Flail
1 Crawlspace
1 Darksteel Mutation
1 Dauthi Voidwalker
1 Diabolic Tutor
1 Disenchant
1 Dragonskull Summit
1 Drowned Catacomb
1 Elemental Mascot
1 Elenda, the Dusk Rose
1 Elspeth, Sun's Champion
1 Exotic Orchard
1 Expressive Iteration
1 Fact or Fiction
1 Falcon and Redwing
1 Firebird, Blazing Ranger
1 Force of Will
1 Furycalm Snarl
1 Ghostly Prison
1 Gilgamesh, Master-at-Arms
1 Glacial Fortress
1 Harsh Annotation
1 Helm of Kaldra
1 Hero of Bladehold
1 Isolated Chapel
1 Kraum, Ludevic's Opus
1 Laelia, the Blade Reforged
1 Lightning Greaves
1 Lightning, Security Sergeant
1 Lyra Dawnbringer
1 Marchesa, the Black Rose
1 Masterful Flourish
1 Memory Trap
1 Moseo, Vein's New Dean
1 Nightveil Predator
1 Nightwind Glider
1 Path to Exile
1 Port Town
1 Potioner's Trove
1 Propaganda
1 Rescue, Pepper Potts
1 Rip Apart
1 Rugged Prairie
1 Scorching Dragonfire
1 Shattered Sanctum
1 Shield of Kaldra
1 Skyclave Apparition
1 Spectator Seating
1 Spectral Sailor
1 Sphinx of New Prahv
1 Star Compass
1 Sulfur Falls
1 Swords to Plowshares
1 Talisman of Conviction
1 Talisman of Hierarchy
1 Terror of the Peaks
1 The Wasp, Winsome Avenger
1 Thrummingbird
1 Tocasia's Welcome
1 Toxic Deluge
1 Tymna the Weaver
1 Unclaimed Territory
1 Underground River
1 Vampire Nighthawk
1 Windborn Muse
5 Island
3 Mountain
6 Plains
5 Swamp
```
