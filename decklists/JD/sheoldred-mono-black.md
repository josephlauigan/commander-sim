# Sheoldred, the Apocalypse — Mono-Black Drain

**Simulator, 2026-10-12:** **30.8% over all tiers** (±0.5; rank 3 of the ten decks): T1 26.5%, T2 39.7%, T3 32.5%, T4 30.9%, T5 24.4%. Look-ahead AI, loose profile, 6,720 games per tier, every deck on the same code; `decklists/pool/pool-results.md` section 0d.

**Added 2026-10-11.** JD's mono-black deck around Sheoldred, the Apocalypse, taken from Sauron (who measured −2.1 points
without her; Bedevil from the binder took her slot). First built from the bulk and binder lists with no buys (20.9%),
then given Memory Jar and Gix from Jodah and two upgrade packages (19 cards bought on 2026-10-11 for about $40, plus
Phyrexian Arena from Sauron). Fully modeled; measured at **about 32% over all tiers** (see "Simulator status").
**Updated 2026-10-11.** Out: Carnage Altar. In: Memory Jar. JD's choice: Memory Jar moves from Jodah (Szadek, Lord of
Secrets refills it there); with Sheoldred out it makes each opponent draw seven, 14 life each. Measured +2.9 points over
all tiers (look-ahead, loose, 1,000 paired games per tier; `audit/sheoldred/`). Carnage Altar goes back to bulk.
**Updated 2026-10-11 (second change).** Out: Ribskiff, Locust Spray, Gravblade Heavy, Multiply by Zero, Mabel, Bitter
Recluse, Moseo, Vein's New Dean, Burglar Rat, Fathom Fleet Cutthroat, Canyon Crawler, Necrosquito, Rampart Hunter,
Callous Inspector, Depressurize, End of the Hunt, Tome of Legends, Diary of Dreams, Dark Endurance, Lash of the Balrog,
Hullcarver, 2 Swamp. In: the two upgrade packages (see "Buy list"), and Gix, Yawgmoth Praetor from Jodah (Lagrella, the
Magpie refills it there). JD's choice: the packages measured +7.7 points (24.3% → 32.0%) and Gix +1.2 on top (within
noise, leaning positive); `audit/sheoldred/`. The cuts go back to bulk, Lash of the Balrog to the binder.
**Updated 2026-10-11 (third change).** No change to the list: Phyrexian Arena comes from Sauron instead of being bought
(Sauron measured +0.6 with Cost of Brilliance in its slot, within noise; `audit/sauron/2026-10-11-arena-cost-of-brilliance/`).
The buy list is 19 cards, $40.39.

## Strategy

Sheoldred, the Apocalypse ({2}{B}{B}, 4/5 deathtouch): *Whenever you draw a card, you gain 2 life. Whenever an
opponent draws a card, they lose 2 life.* Three opponents' draw steps drain 6 life a round; every card you draw is
2 life back. The deck makes the whole table draw and punishes every card.

- **Punish their draws.** Sheoldred (2 life a card), Underworld Dreams and Fate Unraveler (1 damage a card each): with
  all three out, every card an opponent draws costs them 4.
- **Make them draw.** Howling Mine (a card more in every draw step), Temple Bell and Geier Reach Sanitarium (each
  player draws, at will), Seizan (each player loses 2 and draws 2 in their upkeep), and the wheels: Memory Jar (seven
  cards each) and Dark Deal. Painful Lesson, Cost of Brilliance and Vault Plunderer hand one opponent cards; Peer into
  the Abyss makes one draw half their library, lethal with Sheoldred out. The AI holds the symmetric ones (the Mine,
  the Bell, the Sanitarium, Seizan, the wheels) until a punisher is out, so they don't just refuel the table.
- **Draw your own.** Each card is 2 life with her out: Phyrexian Arena, Read the Bones, Ambition's Cost, Syphon Mind,
  Phyrexian Rager, Skyscanner, Deadly Dispute, Cast Away Doubt, Seymour Flux, Castle Locthwain, Bonders' Enclave, and
  Gix (a card whenever one of your creatures hits an opponent).
- **Finish.** Exsanguinate (X life from each opponent) and Gray Merchant of Asphodel (devotion to black).
- **Protect her.** Champion's Helm (hexproof on a legend), Swiftfoot Boots, Darksteel Plate, Masterful Flourish,
  Battle-Rage Blessing and Perigee Beckoner (she comes back if she dies).
- **Remove threats.** 13 removal spells, from Terror and Last Gasp to Invoke Despair; Melancholic Poet drains each
  opponent for every one that targets a creature.
- **Gain life, use it.** Sheoldred's lifegain turns on Ulna Alley Shopkeep, Foolish Fate, Restless Bloodseeker (Blood
  tokens) and Leech Collector (Bloodletting: each opponent loses 2).

## Key lines

- **Turns 1-3:** a rock (Arcane Signet, Fellwar Stone, Mind Stone, Charcoal Diamond, Commander's Sphere, Potioner's
  Trove) and a cheap blocker; Sheoldred on turn 3 or 4. Underworld Dreams is a fine turn-3 play on its own.
- **With Sheoldred out:** equip Champion's Helm or Swiftfoot Boots first, then the draw engines: Howling Mine, Seizan,
  Temple Bell at each opponent's end step, Memory Jar the turn she's safe. Hold a protection instant for removal aimed at
  her.
- **Without her:** removal, Underworld Dreams, Fate Unraveler and the drain creatures (Blood Artist, Al Bhed Salvagers,
  Sneering Shadewriter, Screeching Soulbreaker) keep the pressure on until she comes back; hold the symmetric draw
  cards.
- **The kill:** Peer into the Abyss on an opponent with Sheoldred out, or Exsanguinate for the table.

## Consistency

36 lands: 32 Swamp, Command Tower, Bonders' Enclave, Castle Locthwain (untapped with a Swamp out) and Geier Reach
Sanitarium (colorless). Ramp: Arcane Signet, Fellwar Stone, Mind Stone, Charcoal Diamond, Commander's Sphere, Potioner's
Trove, Undercity Dire Rat (a Treasure when it dies).

## Simulator status

**Measured 2026-10-11** (look-ahead AI, loose profile, 1,000 paired games per tier; `audit/sheoldred/README.md`):

| List | T1 | T2 | T3 | T4 | T5 | All tiers |
|---|---|---|---|---|---|---|
| Bulk only (Carnage Altar for Memory Jar) | 16.8% | 21.7% | 20.5% | 23.5% | 22.0% | 20.9% |
| + Memory Jar | 18.1% | 27.1% | 24.9% | 27.5% | 23.9% | 24.3% |
| + both upgrade packages | 26.4% | 42.7% | 33.4% | 32.2% | 25.3% | **32.0%** |
| + Gix (on a second set of games: 30.7% → 31.9%) | | | | | | +1.2 |

These come from paired runs, not the section 0d seeds of `decklists/pool/pool-results.md`, so they are close to but not
directly comparable with that table (Sephiroth 35.9%, Kefka 26.6%). Traced games show the wins coming from the drain
plan and from Exsanguinate. Two other Jodah blocks were tried and not taken, both within noise: Toxic Deluge + Bitter
Triumph (+0.7) and Korlash, Heir to Blackblade (+0.4).

Modeling: every card is Full. Sheoldred's drain fires on every opponent draw (draw steps, extra draws, wheels) and her
lifegain on every card you draw. One known simplification: Seizan's and Underworld Dreams' triggers resolve at once,
so opponents can't respond to them.

## Bracket

Bracket 3, no Game Changers.

## Buy list (19 cards, about $40)

Bought for the two upgrade packages (cheapest printing on Scryfall, 2026-10-11). Each is a new copy for this deck.

| Card | Price | Role |
|---|---|---|
| Seizan, Perverter of Truth | $1.87 | Each player loses 2 and draws 2 in their upkeep |
| Underworld Dreams | $2.02 | 1 damage for every card an opponent draws |
| Dark Deal | $2.35 | Wheel: each player discards their hand and draws one fewer |
| Painful Lesson | $0.17 | Target player draws 2 and loses 2 |
| Howling Mine | $5.53 | Each player draws a card more in their draw step |
| Temple Bell | $6.69 | Each player draws a card, at will |
| Fate Unraveler | $3.41 | 1 damage for every card an opponent draws |
| Peer into the Abyss | $3.40 | Target player draws half their library and loses half their life |
| Geier Reach Sanitarium | $2.42 | Land: each player draws, then discards |
| Castle Locthwain | $4.31 | Land: draw a card |
| Mind Stone | $0.21 | Ramp |
| Commander's Sphere | $0.31 | Ramp |
| Charcoal Diamond | $0.38 | Ramp |
| Read the Bones | $0.30 | Draw 2 |
| Ambition's Cost | $0.34 | Draw 3 |
| Syphon Mind | $0.30 | Each other player discards; you draw for each |
| Exsanguinate | $0.40 | Finisher: X life from each opponent |
| Gray Merchant of Asphodel | $4.47 | Finisher: drain by devotion to black |
| Swiftfoot Boots | $1.51 | Hexproof for Sheoldred |
| **Total** | **$40.39** | |

## Where the cards come from

**From Sauron:** Sheoldred, the Apocalypse (the commander) and Phyrexian Arena. **From Jodah:** Memory Jar, Gix, Yawgmoth Praetor. **Bought:**
the 19 cards above. **Basics:** 32 Swamp, your own. The rest come off JD's lists (2026-10-11), one copy each: 1 from the
binder (Phyrexian Delver), 44 from bulk. Set codes are the printings the bulk list shows; where it names more than one,
any of them. Bulk's Arcane Signet and Fellwar Stone are spare copies.

The tables below list every card but the Swamps, grouped by color so you can pull them a color at a time; the "From"
column says where each comes from (Bulk, Binder, Jodah, Sauron, or a buy with its price).

### Black (51)

| Card | Type | From | Set |
|---|---|---|---|
| Accursed Marauder | Creature | Bulk | mh3 |
| Al Bhed Salvagers | Creature | Bulk | fin |
| Ambition's Cost | Sorcery | Buy, $0.34 |  |
| Apex Witchstalker | Creature | Bulk | fra |
| Battle-Rage Blessing | Instant | Bulk | dmu |
| Blood Artist | Creature | Bulk | ema |
| Burrog Banemaker | Creature | Bulk | sos |
| Cast Away Doubt | Sorcery | Bulk | fra |
| Cost of Brilliance | Sorcery | Bulk | sos |
| Dark Deal | Sorcery | Buy, $2.35 |  |
| Deadly Dispute | Instant | Bulk | j25 |
| Disembowel | Instant | Bulk | rvr |
| Exsanguinate | Sorcery | Buy, $0.40 |  |
| Fate Unraveler | Creature | Buy, $3.41 |  |
| Fetid Imp | Creature | Bulk | m21 |
| Foolish Fate | Instant | Bulk | sos |
| Gix, Yawgmoth Praetor | Creature | Jodah |  |
| Gray Merchant of Asphodel | Creature | Buy, $4.47 |  |
| Invoke Despair | Sorcery | Bulk | ump |
| Last Gasp | Instant | Bulk | fra, sos |
| Leech Collector // Bloodletting | Creature | Bulk | sos |
| Lethal Throwdown | Sorcery | Bulk | mh3 |
| Masterful Flourish | Instant | Bulk | sos |
| Melancholic Poet | Creature | Bulk | sos |
| Overkill | Instant | Bulk | fin |
| Painful Lesson | Sorcery | Buy, $0.17 |  |
| Peer into the Abyss | Sorcery | Buy, $3.40 |  |
| Perigee Beckoner | Creature | Bulk | eoe |
| Phyrexian Arena | Enchantment | Sauron |  |
| Phyrexian Delver | Creature | Binder |  |
| Phyrexian Rager | Creature | Bulk | dd3 |
| Read the Bones | Sorcery | Buy, $0.30 |  |
| Restless Bloodseeker // Bloodsoaked Reveler | Creature | Bulk | vow |
| Rooftop Assassin | Creature | Bulk | otj |
| Screeching Soulbreaker | Creature | Bulk | fra |
| Seizan, Perverter of Truth | Creature | Buy, $1.87 |  |
| Sephiroth's Intervention | Instant | Bulk | fin |
| Seymour Flux | Creature | Bulk | fin |
| Sheoldred, the Apocalypse | Creature | Sauron |  |
| Silence the Echo | Sorcery | Bulk | fra |
| Sneering Shadewriter | Creature | Bulk | sos |
| Soul Enervation | Enchantment | Bulk | mkm |
| Stir Up Trouble | Sorcery | Bulk | hob |
| Syphon Mind | Sorcery | Buy, $0.30 |  |
| Terror | Instant | Bulk | dmr |
| Ulna Alley Shopkeep | Creature | Bulk | sos |
| Undercity Dire Rat | Creature | Bulk | fin |
| Underworld Dreams | Enchantment | Buy, $2.02 |  |
| Vanquish the Weak | Instant | Bulk | xln |
| Vault Plunderer | Creature | Bulk | otj |
| Wander Off | Instant | Bulk | sos |

### Colorless (13)

| Card | Type | From | Set |
|---|---|---|---|
| Arcane Signet | Artifact | Bulk | fic, msc |
| Champion's Helm | Artifact | Bulk | fic |
| Charcoal Diamond | Artifact | Buy, $0.38 |  |
| Commander's Sphere | Artifact | Buy, $0.31 |  |
| Darksteel Plate | Artifact | Bulk | fic |
| Fellwar Stone | Artifact | Bulk | msc, S.H.I.E.L.D. Spy Satellite printing |
| Howling Mine | Artifact | Buy, $5.53 |  |
| Memory Jar | Artifact | Jodah |  |
| Mind Stone | Artifact | Buy, $0.21 |  |
| Potioner's Trove | Artifact | Bulk | sos |
| Skyscanner | Creature | Bulk | list |
| Swiftfoot Boots | Artifact | Buy, $1.51 |  |
| Temple Bell | Artifact | Buy, $6.69 |  |

### Lands (4, plus 32 Swamp of your own)

| Card | Type | From | Set |
|---|---|---|---|
| Bonders' Enclave | Land | Bulk | fic |
| Castle Locthwain | Land | Buy, $4.31 |  |
| Command Tower | Land | Bulk | khc, fic |
| Geier Reach Sanitarium | Land | Buy, $2.42 |  |

## Decklist by type (100)

**Commander (1).** Sheoldred, the Apocalypse

**Lands (36).** Bonders' Enclave, Castle Locthwain, Command Tower, Geier Reach Sanitarium, 32 Swamp

**Creatures (24).** Accursed Marauder, Al Bhed Salvagers, Apex Witchstalker, Blood Artist, Burrog Banemaker, Fate Unraveler, Fetid Imp, Gix Yawgmoth Praetor, Gray Merchant of Asphodel, Leech Collector // Bloodletting, Melancholic Poet, Perigee Beckoner, Phyrexian Delver, Phyrexian Rager, Restless Bloodseeker // Bloodsoaked Reveler, Rooftop Assassin, Screeching Soulbreaker, Seizan Perverter of Truth, Seymour Flux, Skyscanner, Sneering Shadewriter, Ulna Alley Shopkeep, Undercity Dire Rat, Vault Plunderer

**Enchantments (3).** Phyrexian Arena, Soul Enervation, Underworld Dreams

**Artifacts (12).** Arcane Signet, Champion's Helm, Charcoal Diamond, Commander's Sphere, Darksteel Plate, Fellwar Stone, Howling Mine, Memory Jar, Mind Stone, Potioner's Trove, Swiftfoot Boots, Temple Bell

**Sorceries (13).** Ambition's Cost, Cast Away Doubt, Cost of Brilliance, Dark Deal, Exsanguinate, Invoke Despair, Lethal Throwdown, Painful Lesson, Peer into the Abyss, Read the Bones, Silence the Echo, Stir Up Trouble, Syphon Mind

**Instants (11).** Battle-Rage Blessing, Deadly Dispute, Disembowel, Foolish Fate, Last Gasp, Masterful Flourish, Overkill, Sephiroth's Intervention, Terror, Vanquish the Weak, Wander Off

## Import list (100)

```
1 Accursed Marauder
1 Al Bhed Salvagers
1 Ambition's Cost
1 Apex Witchstalker
1 Arcane Signet
1 Battle-Rage Blessing
1 Blood Artist
1 Bonders' Enclave
1 Burrog Banemaker
1 Cast Away Doubt
1 Castle Locthwain
1 Champion's Helm
1 Charcoal Diamond
1 Command Tower
1 Commander's Sphere
1 Cost of Brilliance
1 Dark Deal
1 Darksteel Plate
1 Deadly Dispute
1 Disembowel
1 Exsanguinate
1 Fate Unraveler
1 Fellwar Stone
1 Fetid Imp
1 Foolish Fate
1 Geier Reach Sanitarium
1 Gix, Yawgmoth Praetor
1 Gray Merchant of Asphodel
1 Howling Mine
1 Invoke Despair
1 Last Gasp
1 Leech Collector // Bloodletting
1 Lethal Throwdown
1 Masterful Flourish
1 Melancholic Poet
1 Memory Jar
1 Mind Stone
1 Overkill
1 Painful Lesson
1 Peer into the Abyss
1 Perigee Beckoner
1 Phyrexian Arena
1 Phyrexian Delver
1 Phyrexian Rager
1 Potioner's Trove
1 Read the Bones
1 Restless Bloodseeker // Bloodsoaked Reveler
1 Rooftop Assassin
1 Screeching Soulbreaker
1 Seizan, Perverter of Truth
1 Sephiroth's Intervention
1 Seymour Flux
1 Sheoldred, the Apocalypse
1 Silence the Echo
1 Skyscanner
1 Sneering Shadewriter
1 Soul Enervation
1 Stir Up Trouble
1 Swiftfoot Boots
1 Syphon Mind
1 Temple Bell
1 Terror
1 Ulna Alley Shopkeep
1 Undercity Dire Rat
1 Underworld Dreams
1 Vanquish the Weak
1 Vault Plunderer
1 Wander Off
32 Swamp
```
