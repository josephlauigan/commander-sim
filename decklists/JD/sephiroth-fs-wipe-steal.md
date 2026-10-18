# Sephiroth, Fabled SOLDIER — Mono-Black Wipe, Drain and Steal the Dead

A mono-black control deck. Sephiroth, Fabled SOLDIER drains one life for every creature that dies, so the deck makes
creatures die on purpose: sacrifice outlets, edicts and board wipes. Once Sephiroth has flipped, the emblem keeps the
drain going for the rest of the game, and the deck takes opponents' dead creatures for itself with Grave Betrayal,
Necromantic Selection and the reanimation spells. It wins by draining the table, by Exsanguinate, or by a 5/5 flying
Sephiroth sacrificing a whole army as he attacks. It is weak to graveyard hate, to exile removal on Sephiroth before he
flips, and to opponents who simply have no creatures to die.

This is the Sephiroth, Fabled SOLDIER deck (sim key `sephfs`), not the Seph/Atraxa deck (`seph`).

| | |
|---|---|
| **Commander** | Sephiroth, Fabled SOLDIER ({2}{B}, 3/3, black) // Sephiroth, One-Winged Angel (5/5 flying) |
| **Bracket** | 4 (4 Game Changers: The One Ring, Demonic Tutor, Vampiric Tutor, Bolas's Citadel) |
| **Simulator** | deck key `sephfs`; results in `decklists/pool/pool-results.md`, history in `decklists/CHANGELOG.md` |

## Strategy

### The commander
**Sephiroth, Fabled SOLDIER** ({2}{B}, 3/3): *Whenever Sephiroth enters or attacks, you may sacrifice another
creature. If you do, draw a card. Whenever another creature dies, target opponent loses 1 life and you gain 1 life. If
this is the fourth time this ability has resolved this turn, transform Sephiroth.*

**Sephiroth, One-Winged Angel** (5/5, flying): *As he transforms you get an emblem: "Whenever a creature dies, target
opponent loses 1 life and you gain 1 life." Whenever Sephiroth attacks, you may sacrifice any number of other
creatures. If you do, draw that many cards.*

In practice: every death is a drain, and the fourth death in one turn flips him. The emblem is permanent, so it stays
even if Sephiroth dies, and it triggers on every creature, including tokens and Sephiroth himself. He also turns spare
bodies into cards: sacrifice one when he enters or attacks to draw, or a whole team when he attacks as the angel.
What breaks him: exile and bounce while he is still the 3/3, and a wipe that kills him before the fourth death (he has
to be on the battlefield when the fourth drain resolves to flip).

### Game plan
- **Turns 1-3:** play cheap creatures that are happy to die (Viscera Seer, Carrier Thrall, Undercity Dire Rat, Common
  Crook, Callous Inspector, Reassembling Skeleton, Woe Strider, Yahenni, Carrion Feeder) and a mana rock. Sephiroth on
  turn 3, ideally with a sacrifice outlet already on the board. **Dark Ritual** can speed the start.
- **Turns 3-5: flip him.** Get four deaths in one turn with Sephiroth alive. Sacrifice to Phyrexian Altar, Phyrexian
  Tower, Viscera Seer, Yahenni, Woe Strider or Carrion Feeder; or play an edict (**Plaguecrafter**, **Fleshbag
  Marauder**, **Accursed Marauder**: every player sacrifices at once, so up to four deaths in a four-player game).
- **Middle game:** with the emblem out, wipe the board when opponents are ahead. Every creature that dies drains, and
  yours come back from your reanimation and steal effects.
- **Late game:** keep draining, take their best dead creatures, and finish with Exsanguinate or an attacking angel.

### Sacrifice outlets and fodder
- **Outlets:** **Phyrexian Altar** (any color) and **Phyrexian Tower** (BB, a land) turn each sacrifice into mana;
  **Viscera Seer** (free, scry 1), **Woe Strider** (scry 1; also brings a Goat token; escapes from the graveyard),
  **Carrion Feeder** (+1/+1 counter, can't block), **Yahenni, Undying Partisan** (haste; gets bigger when an
  opponent's creature dies; sacrifice another creature for indestructible). Sephiroth himself is an outlet when he
  enters or attacks.
- **Fodder that pays you back:** **Carrier Thrall** (an Eldrazi Scion on death), **Undercity Dire Rat** and **Common
  Crook** (Treasure on death), **Callous Inspector** (menace; a Clue on death, 1 damage to you), **Dread Drone** (two
  Eldrazi Spawn tokens), **Pawn of Ulamog** (an Eldrazi Spawn when a nontoken creature of yours dies), **Eternal
  Student** ({1}{B}, exile it from the graveyard: two 1/1 flying Inklings), **Reassembling Skeleton** ({1}{B}: back from the
  graveyard tapped), **Grave Titan** (two 2/2 Zombies on entering and attacking, deathtouch), **Pitiless Plunderer** (a
  Treasure when another creature of yours dies).
- **Do the flip with an outlet:** the Spawn and Scion tokens are also fodder, and the Treasure and Scion mana helps pay
  for the next turn.

### Drain on every death
**Blood Artist**, **Zulaport Cutthroat**, **Al Bhed Salvagers** (also counts artifacts of yours dying),
**Bastion of Remembrance** (and a 1/1 token), **Butcher of Malakir** (each opponent sacrifices a creature whenever
yours die) and **Gray Merchant of Asphodel** (when it enters, each opponent loses life equal to your devotion to black and
you gain that much; in mono-black that is usually a big drain, and it is a body to sacrifice afterwards) add to every death on top of Sephiroth, so a single wipe can be game ending. **Moseo, Vein's New Dean** (a flying
body with a Pest token) uses the life you gain from the drains: at your end step it returns a creature card to the
battlefield with mana value up to the life you gained that turn.

### Wipes and edicts
With the emblem, every wipe drains once per creature:
- **Massacre Girl** (each other creature gets -1/-1, and again for each death that turn, so it chains), **Pestilence**
  (pings {B} at a time; also hits players, including you, but the drain gains it back), **Blood on the Snow**,
  **Crux of Fate**, **Toxic Deluge**, **Living Death** (wipes the board, then everyone's graveyard creatures come back;
  best when your graveyard is the fuller one) and **Necromantic Selection** (wipes and steals the best creature that died).
- **Extinction Event** exiles instead, so it drains nothing; choose odd or even so Sephiroth (mana value 3) and your
  best creatures stay and the opponents' lose. **End of the Hunt** exiles an opponent's biggest creature or
  planeswalker (also no drain).
- Only wipe **after the flip**, or when Sephiroth survives it. Massacre Girl shrinks Sephiroth too: cast it when he is
  already the 5/5. Pestilence spares a 3/3 Sephiroth for the first two points.

### Their dead fight for you
**Grave Betrayal** (a creature you don't control that dies comes back under your control at the next end step, with a
+1/+1 counter), **Dread Slaver** (a creature it damaged that dies comes back), **Necromantic Selection**, **Animate
Dead** and **Grave Researcher // Reanimate** (any graveyard), **Gonti, Lord of Luxury** (steals a card from the top four
of an opponent's library; deathtouch) and **Zombify**, **Rewrite Regrets**, **Pull from the Grave**, **Ral Zarek, Guest
Lecturer** (-2 returns a creature of mana value 3 or less) and **Braids's Frightful Return** (chapter II returns a
creature card to hand) for your own. Stolen creatures go to Phyrexian Tower or Altar for mana, to Sephiroth for cards,
and every death drains again.

### Protecting Sephiroth
**Kaya's Ghostform** (he comes back if he dies or is exiled, and his enter trigger fires again), **Presumed Dead** (a creature that dies comes back suspected: use it
on Sephiroth, or on a creature you are about to sacrifice, which can then be sacrificed again), and the sacrifice
outlets in response to removal. **The One Ring** gives protection from everything until your next turn when you cast it.

### Removal
**Last Gasp**, **Sephiroth's Intervention** (destroy, gain 2), **Wander Off** (exile) and **End of the Hunt** are the
spot answers.

### Card flow and mana
**The One Ring** (draw more each turn; the burden counters cost life, and the drain gains it back), **Call of the Ring**
(a Ring-bearer and a card for 2 life), **Bolas's Citadel** (play lands and cast spells off the top of the library by paying life
instead of mana; the drains refill the life, and every creature you cast from the top is more fodder), **Cast Away Doubt**
(draw two; 2 damage to every player), **Stargaze** (X cards to hand, the rest into the graveyard where reanimation uses
them), **Sol Ring**, **Arcane Signet**, **Fellwar Stone**, **Mind Stone** and **Dark Ritual**.

### Tutors
**Demonic Tutor** ({1}{B}, any card to hand), **Vampiric Tutor** ({B}, lose 2 life, any card on top of the library, a good
match for Bolas's Citadel) and **Rune-Scarred Demon** ({5}{B}{B}, 6/6 flier: search for any card when it enters) find the
piece the game needs: Pestilence or Massacre Girl for the wipe turn, Exsanguinate or Gray Merchant to finish, Phyrexian
Altar or Kaya's Ghostform to set up the flip. The life paid is refilled by the drains.

### How it wins
- **The wipe turn.** With the emblem out, a wipe in a four-player game drains once per creature on the board. A board of
  ten creatures is ten life across the table and ten gained, and Blood Artist, Zulaport, Bastion and Al Bhed Salvagers
  add to it.
- **Exsanguinate.** Sacrifice into Phyrexian Altar, Phyrexian Tower and Treasures first for the mana, then drain every
  opponent for X at once.
- **The angel.** The 5/5 flier sacrifices the army as he attacks, drawing a card each and draining as each dies.
- **A loop.** **Phyrexian Altar**, **Reassembling Skeleton** and **Pawn of Ulamog** make a loop: sacrifice the Skeleton to
  Altar for a mana, Pawn makes a Spawn that adds one more, and that pays {1}{B} to bring the Skeleton back. Each pass
  is a death, so with Sephiroth or any drain creature on board it is a chain of drains. See "Bracket and Rule 0".

### Weak spots and how to play around them
- **Graveyard hate** (effects that exile graveyards) stops the stealing and reanimating. The sacrifice
  and drain plan still works.
- **Exile or bounce on Sephiroth before the flip.** Play him with an outlet, keep Kaya's Ghostform or Presumed Dead
  ready, and flip as soon as possible. If he is removed, keep going: the drain creatures and Pestilence still work.
- **Few creatures on the table.** Edicts and wipes do little against a deck that plays no creatures; use removal and
  Exsanguinate instead.
- **A wipe before the flip** kills Sephiroth. Use an edict or four sacrifices first.
- **Lifegain decks.** The drain is slower against them; Exsanguinate and Butcher of Malakir (an edict for each of your
  creatures that dies) do not depend on life totals.

## Key lines

- **Turn 3 Sephiroth**, ideally with a sacrifice outlet already out; protect him with Kaya's Ghostform.
- **Flip before wiping:** a wipe that kills Sephiroth doesn't flip him. Use an edict or four sacrifices first.
- **Wipe when opponents are ahead on board**, after the flip; keep a steal effect (Grave Betrayal) out first when you
  can.
- **Turn 4 flip:** Sephiroth, Viscera Seer and three cheap creatures: sacrifice the three to the Seer, then the Seer
  itself, for four deaths and the emblem.
- **Mulligan:** keep hands with a mana rock or a Treasure source, two or three lands, and a cheap creature or two;
  send back hands with no creatures.
- **Weak spots:** graveyard hate stops the stealing; exile removal on Sephiroth before he flips.

## Mana and consistency

36 lands: 32 Swamp, Command Tower and Path of Ancestry (both make black here), Vivid Marsh (enters tapped, makes black)
and Phyrexian Tower (colorless, or sacrifice a creature for {B}{B}). Ramp: Sol Ring, Arcane Signet, Fellwar Stone,
Mind Stone and Phyrexian Altar, plus Dark Ritual, Treasure and the Eldrazi tokens. Card draw: The One
Ring, Call of the Ring, Bolas's Citadel, Cast Away Doubt, Stargaze, Sephiroth's own sacrifice. Tutors: Demonic Tutor,
Vampiric Tutor and Rune-Scarred Demon. Interaction: four spot
removal spells and seven wipes (Crux of Fate, Toxic Deluge, Blood on the Snow, Massacre Girl, Extinction
Event, Living Death, Necromantic Selection).

## Bracket and Rule 0

**Bracket 4 by Game Changer count:** four Game Changers, **The One Ring**, **Demonic Tutor**, **Vampiric Tutor** and
**Bolas's Citadel**; no extra turns and no mass land destruction. Say at the table that
the deck wipes the board and takes dead creatures, and that the Phyrexian Altar, Reassembling Skeleton and Pawn of
Ulamog loop makes unlimited deaths and drains (it needs three specific cards and a drain source). Name the four Game Changers, and that Rune-Scarred
Demon is a seven-mana tutor.

## Decklist by type (100)

**Commander (1).** Sephiroth, Fabled SOLDIER // Sephiroth, One-Winged Angel

**Lands (36).** Command Tower, Path of Ancestry, Phyrexian Tower, Vivid Marsh, 32 Swamp

**Creatures (28).** Accursed Marauder, Al Bhed Salvagers, Blood Artist, Butcher of Malakir, Callous Inspector, Carrier Thrall, Carrion Feeder, Common Crook, Dread Drone, Dread Slaver, Eternal Student, Fleshbag Marauder, Gonti Lord of Luxury, Grave Researcher // Reanimate, Grave Titan, Gray Merchant of Asphodel, Massacre Girl, Moseo Vein's New Dean, Pawn of Ulamog, Pitiless Plunderer, Plaguecrafter, Reassembling Skeleton, Rune-Scarred Demon, Undercity Dire Rat, Viscera Seer, Woe Strider, Yahenni Undying Partisan, Zulaport Cutthroat

**Planeswalkers (1).** Ral Zarek Guest Lecturer

**Artifacts (7).** Arcane Signet, Bolas's Citadel, Fellwar Stone, Mind Stone, Phyrexian Altar, Sol Ring, The One Ring

**Enchantments (7).** Animate Dead, Bastion of Remembrance, Braids's Frightful Return, Call of the Ring, Grave Betrayal, Kaya's Ghostform, Pestilence

**Instants (6).** Dark Ritual, Last Gasp, Presumed Dead, Sephiroth's Intervention, Vampiric Tutor, Wander Off

**Sorceries (14).** Blood on the Snow, Cast Away Doubt, Crux of Fate, Demonic Tutor, End of the Hunt, Exsanguinate, Extinction Event, Living Death, Necromantic Selection, Pull from the Grave, Rewrite Regrets, Stargaze, Toxic Deluge, Zombify

## Import list (100)

```
1 Accursed Marauder
1 Al Bhed Salvagers
1 Animate Dead
1 Arcane Signet
1 Bastion of Remembrance
1 Blood Artist
1 Blood on the Snow
1 Bolas's Citadel
1 Braids's Frightful Return
1 Butcher of Malakir
1 Call of the Ring
1 Callous Inspector
1 Carrier Thrall
1 Carrion Feeder
1 Cast Away Doubt
1 Command Tower
1 Common Crook
1 Crux of Fate
1 Dark Ritual
1 Demonic Tutor
1 Dread Drone
1 Dread Slaver
1 End of the Hunt
1 Eternal Student
1 Exsanguinate
1 Extinction Event
1 Fellwar Stone
1 Fleshbag Marauder
1 Gonti, Lord of Luxury
1 Grave Betrayal
1 Grave Researcher // Reanimate
1 Grave Titan
1 Gray Merchant of Asphodel
1 Kaya's Ghostform
1 Last Gasp
1 Living Death
1 Massacre Girl
1 Mind Stone
1 Moseo, Vein's New Dean
1 Necromantic Selection
1 Path of Ancestry
1 Pawn of Ulamog
1 Pestilence
1 Phyrexian Altar
1 Phyrexian Tower
1 Pitiless Plunderer
1 Plaguecrafter
1 Presumed Dead
1 Pull from the Grave
1 Ral Zarek, Guest Lecturer
1 Reassembling Skeleton
1 Rewrite Regrets
1 Rune-Scarred Demon
1 Sephiroth, Fabled SOLDIER // Sephiroth, One-Winged Angel
1 Sephiroth's Intervention
1 Sol Ring
1 Stargaze
1 The One Ring
1 Toxic Deluge
1 Undercity Dire Rat
1 Vampiric Tutor
1 Viscera Seer
1 Vivid Marsh
1 Wander Off
1 Woe Strider
1 Yahenni, Undying Partisan
1 Zombify
1 Zulaport Cutthroat
32 Swamp
```
