# Sheoldred, the Apocalypse — Mono-Black Drain

**Added 2026-10-11.** JD's mono-black deck around Sheoldred, the Apocalypse, taken from Sauron (who measured −2.1 points
without her; Bedevil from the binder took her slot). Built from the bulk and binder lists with no buys, plus Memory Jar
from Jodah. Fully modeled; measured 2026-10-11 at **about 23.8% over all tiers** with Memory Jar (see "Simulator status").
**Updated 2026-10-11.** Out: Carnage Altar. In: Memory Jar. JD's choice: Memory Jar moves from Jodah (Szadek, Lord of
Secrets refills it there); with Sheoldred out it makes each opponent draw seven, 14 life each. Measured +2.9 points over
all tiers (look-ahead, loose, 1,000 paired games per tier; `audit/sheoldred/`). Carnage Altar goes back to bulk.

## Strategy

Sheoldred, the Apocalypse ({2}{B}{B}, 4/5 deathtouch): *Whenever you draw a card, you gain 2 life. Whenever an
opponent draws a card, they lose 2 life.* Three opponents' draw steps drain 6 life a round; every card you draw is
2 life back.

- **Protect her.** Champion's Helm (hexproof on a legend), Darksteel Plate, Masterful Flourish, Battle-Rage Blessing,
  Dark Endurance and Perigee Beckoner (she comes back if she dies).
- **Draw cards.** Each card is 2 life with her out: Phyrexian Rager, Skyscanner, Deadly Dispute, Cast Away Doubt,
  Seymour Flux, Tome of Legends, Diary of Dreams, Ribskiff, Callous Inspector, Locust Spray, Bonders' Enclave (she has
  power 4).
- **Make opponents draw.** Memory Jar is the big one: with Sheoldred out, crack it at once and each opponent draws
  seven (14 life each). Vault Plunderer and Cost of Brilliance hand an opponent cards at 2 life each.
- **Remove threats.** 17 removal spells, from Terror and Last Gasp to Invoke Despair; Melancholic Poet drains each
  opponent for every one that targets a creature.
- **Gain life, use it.** Sheoldred gains life on every one of your draws, which turns on Ulna Alley Shopkeep, Foolish
  Fate, Moseo, Restless Bloodseeker (Blood tokens) and Leech Collector (Bloodletting: each opponent loses 2).

## Key lines

- **Turns 1-3:** a rock (Arcane Signet, Fellwar Stone, Potioner's Trove) and a cheap blocker; Sheoldred on turn 3 or 4.
- **With Sheoldred out:** equip Champion's Helm or Darksteel Plate first, then draw and remove threats; hold a
  protection instant for removal aimed at her. Crack Memory Jar the turn she's safe.
- **Without her:** removal and drain creatures (Blood Artist, Al Bhed Salvagers, Sneering Shadewriter, Screeching
  Soulbreaker) keep the pressure on until she comes back. Hold Memory Jar: without her it refills the table.

## Consistency

36 lands: 34 Swamp, Command Tower and Bonders' Enclave, no tapped lands. Ramp is thin: Arcane Signet, Fellwar Stone,
Potioner's Trove, Undercity Dire Rat (a Treasure when it dies).

## Simulator status

**Measured 2026-10-11** (look-ahead AI, loose profile, 1,000 paired games per tier, `audit/sheoldred/`). The bulk-only
list (Carnage Altar in Memory Jar's slot) won 16.8%, 21.7%, 20.5%, 23.5% and 22.0% against Tiers 1-5, **20.9% over
all tiers**. Memory Jar added +2.5, +6.8, +5.2, +0.6 and −0.6 (clear against Tiers 2 and 3): **about 23.8%** with it.
These come from paired runs, not the section 0d seeds of `decklists/pool/pool-results.md`, so they are close to but not
directly comparable with that table (Titania 22.8%, Veyran 23.7%).

Two other Jodah blocks were tried and not taken, both within noise: Toxic Deluge + Bitter Triumph (+0.7) and Korlash,
Heir to Blackblade (+0.4).

Modeling: every card is Full. Sheoldred's drain fires on every opponent draw (draw steps, extra draws, wheels) and her
lifegain on every card you draw.

## Bracket

Bracket 3, no Game Changers.

## Where the cards come from

**Commander:** Sheoldred, the Apocalypse, from Sauron. **From Jodah:** Memory Jar. **Basics:** 34 Swamp, your own.
Every other card comes off JD's lists (2026-10-11), one copy each: 2 from the binder (Lash of the Balrog, Phyrexian
Delver), 62 from bulk. Set codes are the printings the bulk list shows; where it names more than one, any of them.
Bulk's Arcane Signet and Fellwar Stone are spare copies (other decks keep their own).

### Creatures (31)

| Card | From | Set |
|---|---|---|
| Accursed Marauder | Bulk | mh3 |
| Al Bhed Salvagers | Bulk | fin |
| Apex Witchstalker | Bulk | fra |
| Blood Artist | Bulk | ema |
| Burglar Rat | Bulk | j22 |
| Burrog Banemaker | Bulk | sos |
| Callous Inspector | Bulk | tla |
| Canyon Crawler | Bulk | tla |
| Fathom Fleet Cutthroat | Bulk | xln |
| Fetid Imp | Bulk | m21 |
| Gravblade Heavy | Bulk | eoe |
| Hullcarver | Bulk | eoe |
| Leech Collector // Bloodletting | Bulk | sos |
| Mabel, Bitter Recluse | Bulk | fra |
| Melancholic Poet | Bulk | sos |
| Moseo, Vein's New Dean | Bulk | sos |
| Necrosquito | Bulk | one |
| Perigee Beckoner | Bulk | eoe |
| Phyrexian Delver | Binder |  |
| Phyrexian Rager | Bulk | dd3 |
| Rampart Hunter | Bulk | fra |
| Restless Bloodseeker // Bloodsoaked Reveler | Bulk | vow |
| Rooftop Assassin | Bulk | otj |
| Screeching Soulbreaker | Bulk | fra |
| Seymour Flux | Bulk | fin |
| Sheoldred, the Apocalypse | Sauron |  |
| Skyscanner | Bulk | list |
| Sneering Shadewriter | Bulk | sos |
| Ulna Alley Shopkeep | Bulk | sos |
| Undercity Dire Rat | Bulk | fin |
| Vault Plunderer | Bulk | otj |

### Artifacts (9)

| Card | From | Set |
|---|---|---|
| Arcane Signet | Bulk | fic, msc |
| Champion's Helm | Bulk | fic |
| Darksteel Plate | Bulk | fic |
| Diary of Dreams | Bulk | sos |
| Fellwar Stone | Bulk | msc, S.H.I.E.L.D. Spy Satellite printing |
| Memory Jar | Jodah |  |
| Potioner's Trove | Bulk | sos |
| Ribskiff | Bulk | one |
| Tome of Legends | Bulk | msc |

### Enchantments (1)

| Card | From | Set |
|---|---|---|
| Soul Enervation | Bulk | mkm |

### Instants (15)

| Card | From | Set |
|---|---|---|
| Battle-Rage Blessing | Bulk | dmu |
| Dark Endurance | Bulk | eoe |
| Deadly Dispute | Bulk | j25 |
| Depressurize | Bulk | eoe |
| Disembowel | Bulk | rvr |
| Foolish Fate | Bulk | sos |
| Last Gasp | Bulk | fra, sos |
| Locust Spray | Bulk | soa |
| Masterful Flourish | Bulk | sos |
| Multiply by Zero | Bulk | fra |
| Overkill | Bulk | fin |
| Sephiroth's Intervention | Bulk | fin |
| Terror | Bulk | dmr |
| Vanquish the Weak | Bulk | xln |
| Wander Off | Bulk | sos |

### Sorceries (8)

| Card | From | Set |
|---|---|---|
| Cast Away Doubt | Bulk | fra |
| Cost of Brilliance | Bulk | sos |
| End of the Hunt | Bulk | sos |
| Invoke Despair | Bulk | ump |
| Lash of the Balrog | Binder |  |
| Lethal Throwdown | Bulk | mh3 |
| Silence the Echo | Bulk | fra |
| Stir Up Trouble | Bulk | hob |

### Lands (2, plus 34 Swamp of your own)

| Card | From | Set |
|---|---|---|
| Bonders' Enclave | Bulk | fic |
| Command Tower | Bulk | khc, fic |

## Decklist by type (100)

**Commander (1).** Sheoldred, the Apocalypse

**Lands (36).** Bonders' Enclave, Command Tower, 34 Swamp

**Creatures (30).** Accursed Marauder, Al Bhed Salvagers, Apex Witchstalker, Blood Artist, Burglar Rat, Burrog Banemaker, Callous Inspector, Canyon Crawler, Fathom Fleet Cutthroat, Fetid Imp, Gravblade Heavy, Hullcarver, Leech Collector // Bloodletting, Mabel Bitter Recluse, Melancholic Poet, Moseo Vein's New Dean, Necrosquito, Perigee Beckoner, Phyrexian Delver, Phyrexian Rager, Rampart Hunter, Restless Bloodseeker // Bloodsoaked Reveler, Rooftop Assassin, Screeching Soulbreaker, Seymour Flux, Skyscanner, Sneering Shadewriter, Ulna Alley Shopkeep, Undercity Dire Rat, Vault Plunderer

**Enchantments (1).** Soul Enervation

**Artifacts (9).** Arcane Signet, Champion's Helm, Darksteel Plate, Diary of Dreams, Fellwar Stone, Memory Jar, Potioner's Trove, Ribskiff, Tome of Legends

**Sorceries (8).** Cast Away Doubt, Cost of Brilliance, End of the Hunt, Invoke Despair, Lash of the Balrog, Lethal Throwdown, Silence the Echo, Stir Up Trouble

**Instants (15).** Battle-Rage Blessing, Dark Endurance, Deadly Dispute, Depressurize, Disembowel, Foolish Fate, Last Gasp, Locust Spray, Masterful Flourish, Multiply by Zero, Overkill, Sephiroth's Intervention, Terror, Vanquish the Weak, Wander Off

## Import list (100)

```
1 Accursed Marauder
1 Al Bhed Salvagers
1 Apex Witchstalker
1 Arcane Signet
1 Battle-Rage Blessing
1 Blood Artist
1 Bonders' Enclave
1 Burglar Rat
1 Burrog Banemaker
1 Callous Inspector
1 Canyon Crawler
1 Cast Away Doubt
1 Champion's Helm
1 Command Tower
1 Cost of Brilliance
1 Dark Endurance
1 Darksteel Plate
1 Deadly Dispute
1 Depressurize
1 Diary of Dreams
1 Disembowel
1 End of the Hunt
1 Fathom Fleet Cutthroat
1 Fellwar Stone
1 Fetid Imp
1 Foolish Fate
1 Gravblade Heavy
1 Hullcarver
1 Invoke Despair
1 Lash of the Balrog
1 Last Gasp
1 Leech Collector // Bloodletting
1 Lethal Throwdown
1 Locust Spray
1 Mabel, Bitter Recluse
1 Masterful Flourish
1 Melancholic Poet
1 Memory Jar
1 Moseo, Vein's New Dean
1 Multiply by Zero
1 Necrosquito
1 Overkill
1 Perigee Beckoner
1 Phyrexian Delver
1 Phyrexian Rager
1 Potioner's Trove
1 Rampart Hunter
1 Restless Bloodseeker // Bloodsoaked Reveler
1 Ribskiff
1 Rooftop Assassin
1 Screeching Soulbreaker
1 Sephiroth's Intervention
1 Seymour Flux
1 Sheoldred, the Apocalypse
1 Silence the Echo
1 Skyscanner
1 Sneering Shadewriter
1 Soul Enervation
1 Stir Up Trouble
1 Terror
1 Tome of Legends
1 Ulna Alley Shopkeep
1 Undercity Dire Rat
1 Vanquish the Weak
1 Vault Plunderer
1 Wander Off
34 Swamp
```
