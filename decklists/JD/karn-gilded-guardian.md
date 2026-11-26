# Karn, Gilded Guardian — Colour-Draw Blink (WUBRG, High Bracket 4)

Karn, Gilded Guardian draws a card for each colour among the *other* artifacts you control, up to five, every time he
enters. The deck fills the table with coloured artifacts, casts Karn, and then makes him enter again and again: six
blink effects (Ephemerate, Conjurer's Closet, Restoration Angel, Flickerwisp, Teleportation Circle and the
harnessed The Mind Stone) and two doublers (Gandalf the White and Elesh Norn, Mother of Machines) turn each entry into
more draws. Around the engine sit four ways to win: Thassa's Oracle with Demonic Consultation or Doomsday, Exquisite
Blood with Sanguine Bond or Vito, Exsanguinate off Cabal Coffers and Urborg, and combat with Karn carrying Buster Sword
or Sword of Feast and Famine. Seven tutors and thirty-five lands with no basics keep the five colours and the lines
reachable. The deck is weak to artifact removal and sweepers (Farewell is in our own list too), to a Karn that is
exiled or killed with no way back, to Blood Moon effects (there are no basic lands), to fast combo (its answers are Force
of Will, Counterspell, Silence, Teferi's Protection and Teferi, Time Raveler), and to decking itself: Karn's draw is
mandatory. **This list has not been simulated; every number below is a count from the list or a labelled estimate.**

| | |
|---|---|
| **Commander** | Karn, Gilded Guardian ({2/W}{2/U}{2/B}{2/R}{2/G}, mana value 10, 5/5 artifact creature, vigilance and trample; all five colours). When Karn enters, draw a card for each colour among other artifacts you control |
| **Bracket** | 4: 15 Game Changers and four short winning lines. Played against Tier 4 and Tier 5 |
| **Game Changers (15)** | Consecrated Sphinx, Demonic Tutor, Enlightened Tutor, Farewell, Force of Will, Imperial Seal, Mystical Tutor, Necropotence, Orcish Bowmasters, Rhystic Study, Smothering Tithe, Teferi's Protection, Thassa's Oracle, The One Ring, Vampiric Tutor |
| **Simulator** | Deck key `karn_gilded`, tiers t4-t5; findings in `audit/karn_gilded/FINDINGS.md`, history in `decklists/CHANGELOG.md`; plan file `data/plans/karn_gilded.toml`. First runs and their limits are in the findings file; the current list is not simulated. Dread Tiller and Iron Man, Armored Avenger are Full |
| **Buys** | None: every card he does not own is proxied |

## Strategy

### The commander

**Karn, Gilded Guardian** ({2/W}{2/U}{2/B}{2/R}{2/G}, 5/5). *Vigilance, trample. When Karn enters, draw a card for each
color among other artifacts you control.*

Each hybrid symbol is paid with its colour or with {2}. With all five colours available Karn costs **five**; each colour
you cannot pay adds one more mana, so the worst case is ten. The commander tax adds {2} for every recast from the command
zone. The Fire Crystal makes red spells cost {1} less (Karn is every colour, red included), but it only reduces the generic part (a {2} paid for a hybrid
symbol, or the tax), so it does nothing for a clean five-colour cast. Herd Heirloom's mana pays for Karn (he is a
creature spell).

Karn counts colours among *other* artifacts: each of the five colours counts once however many artifacts have it, and
colourless artifacts (Sol Ring, Arcane Signet, Coalition Relic, Thought Vessel, Expedition Map, Buster Sword, Sword of
Feast and Famine, Conjurer's Closet, the Robot tokens, Treasures) count nothing. The deck's coloured artifacts:

| Artifact | Colours |
|---|---|
| The Mind Stone | W |
| Esper Sentinel | W |
| The Soul Stone | B |
| Cursed Mirror | R |
| The Fire Crystal | R |
| Twitching Doll | G |
| Herd Heirloom | G |
| Dread Tiller | B G |
| Iron Man, Armored Avenger | U |
| Gold-Forged Thopteryx | W U |
| Baleful Strix | U B |
| The Reaper, King No More | B R G |
| Breya, Etherium Shaper | W U B R (her two Thopter tokens are blue artifacts too) |
| Infinite Guideline Station | all five alone (WUBRG) |

**Infinite Guideline Station** ({W}{U}{B}{R}{G}) is the only card that is five colours by itself, so it makes every Karn
entry draw five. Breya is four, The Reaper and Gold-Forged Thopteryx and Baleful Strix and Dread Tiller add three or two, and
the rest one. Without Station the five colours come from several artifacts: white from The Mind Stone, Esper Sentinel,
Thopteryx and Breya, blue from Thopteryx, Strix, Breya and Iron Man, black from The Soul Stone, Strix, The Reaper, Breya and Dread
Tiller, red from Cursed Mirror, The Fire Crystal, The Reaper and Breya, green from Twitching Doll, Herd Heirloom, The Reaper and
Dread Tiller. Station needs exactly one of each colour, and red and green are the thin colours; it also makes a tapped 2/2 Robot
for each multicolored permanent we control when it enters.

### The packages

**Draw engine: coloured artifacts and Karn.** The fourteen artifacts above, plus Karn. Several do other work: Baleful
Strix draws a card and trades with an attacker, Esper Sentinel draws a card whenever an opponent casts their first
noncreature spell each turn unless that player pays {X}, where X is its power (1, more with an Equipment on it), The Reaper puts -1/-1 counters on two creatures and steals the ones that
die, Breya makes two Thopters and turns two artifacts into removal or a drain, Dread Tiller and Iron Man are creatures that
work with counters (below), Gold-Forged Thopteryx gives every legendary permanent we control ward {2} (Karn included).

**Blink.** Ephemerate (and its rebound), Conjurer's Closet ({5} artifact: at the beginning of our end step we may exile a creature
we control and return it, so it blinks Karn every turn it is out), Restoration Angel (flash, a non-Angel
creature), Flickerwisp (the enter trigger is mandatory and exiles another permanent until the next end step), Teleportation Circle (an artifact or creature at our end step)
and The Mind Stone once harnessed ({5}{W}, {T}; then it blinks one other nonland permanent at our end step). Skyclave
Apparition, Solitude, Orcish Bowmasters, Baleful Strix and Breya are the other things worth blinking.

**The Fire Crystal.** {2}{R}{R}, legendary: red spells cost {1} less (Karn is every colour, red included, but the discount takes off only
generic mana: a {2} paid for a hybrid symbol, or the tax), our creatures have haste, and {4}{R}{R}, {T} makes a token copy of a creature we control that is
sacrificed at the beginning of the next end step. Haste covers attacks and {T} abilities and ends with the Crystal: Karn attacks the turn he enters or
returns, so a blink in the precombat main phase no longer costs his attack, and Giver of Runes, Twitching Doll and the mana creatures work the turn they come
down, the Crystal's copies included. Any creature we control can be copied, a token too (Breya's Thopters, Twitching Doll's Spiders, the Station's Robots). A copy of Karn
enters and the legend rule comes first: we keep the real Karn, then the copy's trigger draws five (the real Karn is another artifact of all five colours). A copy of a
creature with an enter trigger (Flickerwisp, Skyclave Apparition, Solitude, Orcish Bowmasters, Ranger-Captain of Eos) repeats it.

**Doublers.** Gandalf the White doubles an enter trigger caused by a legendary permanent or an artifact entering (Karn is
both), and also gives artifacts and legendary spells flash, so Karn can be cast at an opponent's end step. Elesh Norn,
Mother of Machines doubles any permanent's enter trigger and turns off the opponents' enter triggers. Both out means three
Karn triggers per entry. With all five colours among the other artifacts as well, one entry is fifteen cards: watch the library.

**Drain (the life layer).** Haliya, Guided by Light (gain 1 for every creature or artifact entering), Vito, Thorn of the Dusk Rose and Sanguine Bond (an opponent loses what we gain), Exquisite Blood
(we gain what an opponent loses) and Exsanguinate.

**Tutors.** Demonic Tutor, Vampiric Tutor, Mystical Tutor, Enlightened Tutor (an artifact or enchantment), Imperial Seal,
Ringsight (a card that shares a colour with a legendary creature we control: with Karn out that is any coloured card;
without a legendary creature it finds nothing) and Expedition Map (a land: Cabal Coffers, Urborg or Reliquary Tower).
Ranger-Captain of Eos fetches one of the deck's three creatures with mana value 1 or less, Yoshimaru, Ever Faithful,
Giver of Runes or Esper Sentinel, and is also a blinkable body (each blink fetches again) and a sacrifice that stops the table's
noncreature spells for a turn. **Yoshimaru** ({W}, 1/1) gets a +1/+1 counter whenever another legendary permanent we
control enters (Karn, The Mind Stone, The Fire Crystal, The Soul Stone, Breya, The Reaper, Infinite Guideline
Station, Iron Man, Haliya, Vito, Gandalf, Elesh Norn), so he grows with the deck; he is a legendary creature that Ringsight can
match. **Giver of Runes** ({W}, 1/1) is protection for Karn: {T}, another target creature we control gains protection from
colorless or from a colour of our choice until end of turn.

**Counters: Dread Tiller and Iron Man.** **Dread Tiller** ({1}{B}{G}, 2/4 artifact creature, a Scarecrow): when it enters, put a -1/-1
counter on target creature (any creature, ours included); whenever a creature with a -1/-1 counter on it dies, we may put a land card from our hand or
graveyard onto the battlefield tapped (any creature's death counts, and a cracked fetchland in the graveyard comes back). On its own the
counter kills only a creature with toughness 1 or one already damaged; its job is to start the counters that **The Reaper, King No More** (also a Scarecrow)
cashes in: The Reaper steals an opponent's creature that dies with a -1/-1 counter on it (once each turn), so a Tiller counter on a creature that
then dies in combat or to our removal returns it to us under our control, and the Tiller puts a land into play from the same death. Every blink of Dread Tiller
or The Reaper (Ephemerate and its rebound, Conjurer's Closet, Restoration Angel, Flickerwisp, Teleportation Circle, The Mind Stone, The Fire Crystal's copy) puts
more counters on the table; Gandalf the White (the Tiller is an artifact) and Elesh Norn, Mother of Machines each add a trigger, so with both out one entry
is three counters, which kills a toughness-3 creature at once. **Iron Man, Armored Avenger** ({3}{U}, legendary 2/2 artifact creature, flying): whenever we draw a
card, put a +1/+1 counter on target creature; whenever it attacks, our other attacking modified creatures (a counter, an Equipment or an Aura on them) gain flying
until end of turn. Karn's five draws put five counters, ten or fifteen with the doublers, all on one creature or spread out (Karn himself, Iron Man, Yoshimaru); the
draw step, Rhystic Study, Consecrated Sphinx, Baleful Strix, Esper Sentinel and The One Ring count too. Karn with Buster Sword, Sword of Feast and Famine or a
counter on him attacks as a flying creature of at least 5/5 with trample while Iron Man attacks beside him; the AI puts each counter on its own creature it values most, a person chooses.
Iron Man is a blue artifact (blue rises to four coloured artifacts) and a legendary creature (Yoshimaru grows when it enters, Thopteryx gives it ward {2}).

**Protection and answers.** Force of Will, Counterspell, Silence, Teferi's Protection, Teferi, Time Raveler, Giver of Runes
(for Karn), Ephemerate and Restoration Angel (they blink Karn in response to a spell or ability that targets him; Conjurer's Closet works only at our end step, so it does not), Akroma's Will ("if you control a commander" gives both modes, so before Karn is out it is a single mode), Swords to Plowshares, Path to Exile, Anguished Unmaking (for a noncreature permanent), Skyclave
Apparition, Solitude, Farewell (a sweeper that also exiles our own artifacts: choose the modes).

**Card flow and mana.** Rhystic Study, Esper Sentinel, Smothering Tithe, Necropotence, Consecrated Sphinx, The One Ring,
Gleaming Splendor, Buster Sword. Mana: Sol Ring, Arcane Signet, Coalition Relic, Thought Vessel, The Mind Stone, The Soul Stone,
Cursed Mirror, Twitching Doll, Herd Heirloom and the 35 lands (see Mana).

### Rules facts that matter

- **The cost.** Five coloured mana when all five colours are on hand; every missing colour is {2} more (mana value 10 in
  the library and the command zone, a real cost of 5 to 10). Commander tax {2} per recast.
- **Blinking at your own end step draws before cleanup.** Conjurer's Closet, Teleportation Circle and the harnessed The Mind Stone blink Karn
  in our end step (the Closet says "may" and the Circle "up to one" target, so either can be declined when the library or the hand is the problem): he returns and draws then, and at cleanup everything above seven cards is discarded. **Thought Vessel
  and Reliquary Tower give no maximum hand size**, so keep one in play before an end-step blink. A blink in our own main
  phase or at an opponent's end step (Karn has flash with Gandalf out) avoids the problem.
- **Karn's draw is mandatory.** There is no "may". With Gandalf and Elesh Norn out and all five colours among the other artifacts, one entry is up to
  fifteen cards. Count the library before each blink. **Thassa's Oracle is the outlet**: with the library empty (or one
  card), Oracle wins instead of losing.
- **Flickerwisp on our own Karn** leaves him in exile until the beginning of the next end step: cast in our main phase he
  returns in our end step (and draws then, see above); if Flickerwisp itself is blinked in our end step, it exiles Karn
  until the *opponent's* end step. While he is in exile the commander rule lets us put him into the command zone: we
  decline.
- **An opponent's Flickerwisp, or any removal, exile or bounce on Karn,** can send him to the command zone (the
  simulator always does); the recast then costs {2} more.
- **The Fire Crystal's copy and the end step.** The copy is sacrificed at the beginning of the next end step to begin, and a copy made during an end
  step waits for the following one. Made in the end step of the player just before us, it stays through our whole turn (it can attack, and a Karn
  copy's cards arrive before our turn, so we can play them before our cleanup), is sacrificed in our end step, and the Crystal untaps with the rest of our permanents.
  Made in our own end step, it stays through the next player's turn (it can block there) and goes in that player's end step; a Karn copy's draw then
  comes before our cleanup, like an end-step blink. Made in a main phase, it is gone at that turn's end step. The {4}{R}{R} is mana we left open.
- **Instant-speed blinks resolve in one action** (Ephemerate, Restoration Angel), so Karn simply returns.
- **Akroma's Will** gives both modes only with a commander on the battlefield: before Karn is out it is a single mode.

### How the deck wins

**A. Thassa's Oracle with Demonic Consultation or Doomsday.** Consultation naming a card that is not in the library exiles
the whole library, then Thassa's Oracle wins with an empty library. Doomsday leaves five chosen cards as the library
(Oracle among them); once it is empty, Oracle wins. Oracle costs {U}{U}; Force of Will, Counterspell, Silence and
Teferi's Protection (with Thopteryx's ward and Ranger-Captain's sacrifice) are the backup for the turn it goes off.

**B. Exquisite Blood with Sanguine Bond or Vito.** Either pair loops: an opponent loses life, Exquisite Blood gains us
that much, Sanguine Bond or Vito makes an opponent lose it again. Any life gain starts it: Haliya's trigger for each
creature or artifact entering (Karn, a Thopter, a Treasure, a Robot), or a lifelink source (Thopteryx). Nothing in the list doubles
life gained, so each Haliya trigger is 1 life and the loop does the draining; Gandalf and Elesh Norn add extra Haliya triggers.

**C. Exsanguinate with Cabal Coffers and Urborg.** Urborg makes every land a Swamp, Cabal Coffers ({2}, {T}) adds {B} for
each Swamp, and Exsanguinate for a large X drains every opponent for X and gains us as much. (Urborg is symmetric: the
opponents' lands tap for black too.)

**D. Combat.** Karn is a 5/5 with vigilance and trample. Buster Sword (+3/+2, draws and casts a spell when it connects) or
Sword of Feast and Famine (+2/+2, protection from black and green, a discard and an untap) make him the clock, and he stays
untapped as a blocker. Breya, Skyclave Apparition, Solitude and the Swords clear the way.

**The grind.** Without a line in hand the deck draws with Karn, Rhystic Study, Necropotence and Consecrated Sphinx,
taxes the table with Smothering Tithe, and answers with its removal.

### Game plan

- **Turns 1-3:** mana (Sol Ring, Signet, Soul Stone, Mind Stone, Thought Vessel), a tutor or a cheap coloured artifact
  (Strix, Thopteryx, Cursed Mirror, Dread Tiller), Rhystic Study or Smothering Tithe if the hand allows.
- **Turns 4-6:** Breya, Iron Man or Station when the colours allow, then Karn. Cast him only when a draw
  is worth it (at least three colours among other artifacts) or when it cannot wait; Gandalf lets him come at an
  opponent's end step with mana up.
- **After Karn:** hold a blink spell and the mana for it, a Thought Vessel or Reliquary Tower before an end-step blink,
  and the counterspells for the turn the line goes off.

### Mulligans

Keep: a hand with two or three lands and a mana rock; a tutor with mana and a line piece in the library. A hand of
coloured artifacts with no mana, or a hand that needs five colours on turn 4 with two colours of lands, is a mulligan.
This is a plan, not a measured keep rule.

### What to tutor for

| Situation | Tutor | Find | Note |
|---|---|---|---|
| Mana short | Imperial Seal, Enlightened Tutor | Sol Ring, Signet, Thought Vessel | The Seal and Vampiric Tutor put the card on top: do it at an end step |
| Colours short for Karn | Enlightened Tutor, Demonic Tutor | Station, Breya, Iron Man, Dread Tiller | Station is five colours alone, Breya four; Enlightened Tutor finds all four (they are artifacts) |
| Karn out, no blink | Demonic Tutor, Vampiric Tutor (Enlightened Tutor finds Conjurer's Closet) | Ephemerate or Restoration Angel, or Conjurer's Closet for a blink every end step | Count the library and the hand size first |
| Ready to win | Demonic Tutor, Mystical Tutor | Consultation or Doomsday, Thassa's Oracle, Exquisite Blood | Hold a counter or Silence |

## Limits (what the simulator and the engine do not do)

These are the limits that matter to a player; the engine notes are in `audit/karn_gilded/FINDINGS.md`.

- **An opponent's Flickerwisp on Karn** sends him to the command zone without asking (the real rule lets the owner
  choose). Cursed Mirror (and Metamorph) never copies a commander in the engine, so there is no Karn copy line.
- **The AI does not cast Karn at instant speed under Gandalf.**
- **The Mind Stone is Approximate**: the harnessed ability picks Karn and creatures and tapped rocks, never Infinite
  Guideline Station or The One Ring.
- **Teferi's Protection is Approximate** (JD waived the Full requirement).
- **Infinite Guideline Station**: a copy that becomes it without entering (a Mirage Mirror) does not get its printed 7/15.
- **Breya's 3-damage mode** takes a player, not a planeswalker.
- **The Reaper's second ability** (named limit, same as Dread Tiller's): the simulator kills creatures one at a time, in list order (seat order, then the order the
  permanents entered), in a wipe, in combat damage and in state-based deaths, and runs the trigger only for a Reaper still on the battlefield, so a creature with a -1/-1 counter
  that dies in the same event after the Reaper is not heard, while one that dies before it is (a Farewell exiles, so it is no case).
- **The Fire Crystal and the AI:** the AI activates the copy only in the end-of-turn window before its own turn, with the mana it left unspent, for the
  creature whose copy is worth most (Karn when its hand has room for the draw, Restoration Angel, Solitude, Skyclave Apparition...); in a main phase it copies
  only a Thassa's Oracle that wins. In its main phase it keeps {4}{R}{R} up when that copy is worth three cards or more. Its numbers are provisional and unmeasured.
- **The AI never** activates the Crystal in response to removal, copies for a lethal hasty attack, makes a Ranger-Captain copy to sacrifice on a combo turn, or
  sacrifices a copy for value; it never copies a token with no card behind it; in the legend rule it keeps the real card over a copy; and its attack
  planning does not count a hasty creature that is still summoning sick (a copy) as an attacker, although the attack itself uses it.
- **The Fire Crystal and the commander tax (simulator):** the Crystal's {1} does not reach the commander tax in the engine until the cost-order change
  merges, so a Karn recast paid with all five colours costs {1} more there than at a table.
- **An animated land can't be copied** by the Crystal, for a person or the AI (no such land is in the list).
- **Dread Tiller's choices (simulator):** the AI puts the counter on the opposing creature it kills (the one worth most first), else the opposing creature worth most,
  else the Tiller itself, and never kills a creature of its own on purpose; an opposing creature whose ward it can't pay is worth nothing to it (the ward counters
  the trigger), so it aims there before shrinking its own creatures. It always puts a land when it has one in its hand or graveyard: first a land that adds a colour its
  lands don't make, then Cabal Coffers, Urborg, Reliquary Tower, Mistrise Village, Path of Ancestry and Command Tower, the graveyard copy before the hand copy (the hand
  copy when its hand holds more than seven cards); it does not put Mana Confluence or City of Brass at 10 life or less. An undying creature, or one under Mikaeus, the
  Unhallowed, that the counter would kill is not counted as a kill (it comes back bigger).
  These rankings are provisional and untuned by any sim. Named limit: the simulator kills creatures one at a time, in list order (seat order, then the order
  the permanents entered), in a wipe, in combat damage and in state-based deaths, so a creature with a -1/-1 counter that dies in the same event after the Tiller does not trigger it,
  while one that dies before it does (The Reaper has the same limit; a Farewell exiles, so it is no case).
- **Iron Man's counter is not a trigger on the stack** in the simulator: it is put as each card is drawn, so the table cannot respond to it; the AI puts it on its own creature
  it values most, and only when none of its own is a legal target (as under Akroma's Will) on an opposing one: it never pays a ward there, it aims at a creature with a ward,
  which counters the ability, else at the opposing creature worth least (a person chooses).
- **The AI numbers for Station, Breya, Herd Heirloom and Twitching Doll are provisional** and untuned by any sim.
- **Not weighed by the AI:** blinking Karn in the precombat main phase loses his attack (he is summoning sick on return) unless The Fire Crystal is out.

## Mana

The list is exactly 100 cards: Karn, 64 other spells and 35 lands. **There are no basic lands.**

**35 lands:** 10 original duals (Tundra, Underground Sea, Badlands, Taiga, Savannah, Scrubland, Volcanic Island, Bayou,
Plateau, Tropical Island), 9 shock lands (Hallowed Fountain, Watery Grave, Godless Shrine, Sacred Foundry, Overgrown Tomb,
Steam Vents, Blood Crypt, Temple Garden, Breeding Pool), 5 fetch lands (Polluted Delta, Marsh Flats, Flooded Strand, Arid
Mesa, Wooded Foothills), 3 pain lands (Adarkar Wastes, Caves of Koilos, Battlefield Forge), 4 lands that make any colour
(Command Tower, City of Brass, Mana Confluence, Path of Ancestry) and 4 utility lands: Urborg, Tomb of Yawgmoth, Cabal
Coffers, Mistrise Village and Reliquary Tower.

- With no basics every fetch can only find a dual or a shock, and each fetch's two land types reach duals of all five
  colours, so a fetch counts as a source of every colour.
- **Colour sources from lands:** white 20, blue 19, black 20 (including Urborg and Cabal Coffers, which needs {2}),
  red 17, green 16.
- **Other sources:** Arcane Signet, Coalition Relic, Twitching Doll and Herd Heirloom (creature spells only) make any
  colour; The Mind Stone (W), The Soul Stone (B) and Cursed Mirror (R) make one; Smothering Tithe's Treasures make any.
- **Enter tapped:** Path of Ancestry always, Mistrise Village unless a Mountain or Forest is out (the typed duals count),
  the nine shocks unless 2 life is paid. **Life costs:** the shocks, five fetches (1), three pain lands (1 per coloured
  tap), City of Brass and Mana Confluence, next to Necropotence, Doomsday and Imperial Seal: watch the life total.
- **Blood Moon** turns every nonbasic land into a Mountain and leaves the deck with red mana only: there is no basic to
  fetch or play.

**Pips.** The 64 spells have **white 34, blue 17, black 24, red 5 and green 4** coloured pips, plus The Reaper's {2/B}{2/R}{2/G}. Red comes
from Cursed Mirror, The Fire Crystal, Breya and Station; green from Twitching Doll, Herd Heirloom, Dread Tiller and Station. White is the heavy colour,
with 20 sources; red and green are thin, and almost all of that load is Station, The Reaper and The Fire Crystal's {R}{R}.

**Curve** (64 spells; The Reaper counted as 6, Station as 5). Average mana value 2.86 (183 over 64).

- **1 (14):** Demonic Consultation, Enlightened Tutor, Ephemerate, Esper Sentinel, Expedition Map, Giver of
  Runes, Imperial Seal, Mystical Tutor, Path to Exile, Silence, Sol Ring, Swords to Plowshares, Vampiric Tutor, Yoshimaru
- **2 (14):** Arcane Signet, Baleful Strix, Counterspell, Demonic Tutor, Exsanguinate (X), Gleaming Splendor,
  Gold-Forged Thopteryx, Herd Heirloom, Orcish Bowmasters, Thassa's Oracle, The Mind Stone, The Soul Stone, Thought
  Vessel, Twitching Doll
- **3 (17):** Anguished Unmaking, Buster Sword, Coalition Relic, Cursed Mirror, Doomsday, Dread Tiller, Flickerwisp, Haliya,
  Necropotence, Ranger-Captain of Eos, Rhystic Study, Ringsight, Skyclave Apparition, Sword of Feast and Famine,
  Teferi's Protection, Teferi, Time Raveler, Vito
- **4 (8):** Akroma's Will, Breya, Iron Man, Restoration Angel, Smothering Tithe, Teleportation
  Circle, The Fire Crystal, The One Ring
- **5 (8):** Conjurer's Closet, Elesh Norn, Exquisite Blood, Force of Will, Gandalf the White, Infinite Guideline Station, Sanguine Bond,
  Solitude
- **6 (3):** Consecrated Sphinx, Farewell, The Reaper, King No More

**Card types** (the 64 spells): 15 artifacts, 8 artifact creatures, 14 other creatures, 13 instants, 6 sorceries,
7 enchantments and 1 planeswalker; 23 artifacts in all and 22 creatures, without Karn.

## Bracket and Rule 0

**Bracket 4,** with fifteen Game Changers: **Consecrated Sphinx**, **Demonic Tutor**, **Enlightened Tutor**, **Farewell**,
**Force of Will**, **Imperial Seal**, **Mystical Tutor**, **Necropotence**, **Orcish Bowmasters**, **Rhystic Study**,
**Smothering Tithe**, **Teferi's Protection**, **Thassa's Oracle**, **The One Ring** and **Vampiric Tutor**. It has no
mass land destruction and no extra turns.

Disclose before the game:

- **The fifteen Game Changers** above.
- **Infinite or game-ending lines:** Thassa's Oracle with Demonic Consultation or Doomsday; Exquisite Blood with Sanguine
  Bond or Vito (with Haliya or a lifelink source as the starter).
- **Tutors:** Demonic, Vampiric, Mystical, Enlightened, Imperial Seal, Ringsight, Expedition Map, Ranger-Captain of Eos
  (a creature with mana value 1 or less: Yoshimaru, Giver of Runes or Esper Sentinel).
- **Free spells:** Force of Will (exile a blue card) and Solitude (exile a white card).
- **Fast mana:** Sol Ring and no more.

Re-check the official list at https://commanderbrackets.com/faq before an event.

## Decklist by type (100)

**Commander (1).** Karn, Gilded Guardian

**Creatures (22).** Baleful Strix, Breya, Etherium Shaper, Consecrated Sphinx, Dread Tiller, Elesh Norn, Mother of Machines, Esper Sentinel, Flickerwisp, Gandalf the White, Giver of Runes, Gold-Forged Thopteryx, Haliya, Guided by Light, Iron Man, Armored Avenger, Orcish Bowmasters, Ranger-Captain of Eos, Restoration Angel, Skyclave Apparition, Solitude, Thassa's Oracle, The Reaper, King No More, Twitching Doll, Vito, Thorn of the Dusk Rose, Yoshimaru, Ever Faithful

**Enchantments (7).** Exquisite Blood, Gleaming Splendor, Necropotence, Rhystic Study, Sanguine Bond, Smothering Tithe, Teleportation Circle

**Artifacts (15).** Arcane Signet, Buster Sword, Coalition Relic, Conjurer's Closet, Cursed Mirror, Expedition Map, Herd Heirloom, Infinite Guideline Station, Sol Ring, Sword of Feast and Famine, The Fire Crystal, The Mind Stone, The One Ring, The Soul Stone, Thought Vessel

**Instants (13).** Akroma's Will, Anguished Unmaking, Counterspell, Demonic Consultation, Enlightened Tutor, Ephemerate, Force of Will, Mystical Tutor, Path to Exile, Silence, Swords to Plowshares, Teferi's Protection, Vampiric Tutor

**Sorceries (6).** Demonic Tutor, Doomsday, Exsanguinate, Farewell, Imperial Seal, Ringsight

**Planeswalkers (1).** Teferi, Time Raveler

**Lands (35).** Adarkar Wastes, Arid Mesa, Badlands, Battlefield Forge, Bayou, Blood Crypt, Breeding Pool, Cabal Coffers, Caves of Koilos, City of Brass, Command Tower, Flooded Strand, Godless Shrine, Hallowed Fountain, Mana Confluence, Marsh Flats, Mistrise Village, Overgrown Tomb, Path of Ancestry, Plateau, Polluted Delta, Reliquary Tower, Sacred Foundry, Savannah, Scrubland, Steam Vents, Taiga, Temple Garden, Tropical Island, Tundra, Underground Sea, Urborg, Tomb of Yawgmoth, Volcanic Island, Watery Grave, Wooded Foothills

## Import list (100)

```
1 Adarkar Wastes
1 Akroma's Will
1 Anguished Unmaking
1 Arcane Signet
1 Arid Mesa
1 Badlands
1 Baleful Strix
1 Battlefield Forge
1 Bayou
1 Blood Crypt
1 Breeding Pool
1 Breya, Etherium Shaper
1 Buster Sword
1 Cabal Coffers
1 Caves of Koilos
1 City of Brass
1 Coalition Relic
1 Command Tower
1 Conjurer's Closet
1 Consecrated Sphinx
1 Counterspell
1 Cursed Mirror
1 Demonic Consultation
1 Demonic Tutor
1 Doomsday
1 Dread Tiller
1 Elesh Norn, Mother of Machines
1 Enlightened Tutor
1 Ephemerate
1 Esper Sentinel
1 Expedition Map
1 Exquisite Blood
1 Exsanguinate
1 Farewell
1 Flickerwisp
1 Flooded Strand
1 Force of Will
1 Gandalf the White
1 Giver of Runes
1 Gleaming Splendor
1 Godless Shrine
1 Gold-Forged Thopteryx
1 Haliya, Guided by Light
1 Hallowed Fountain
1 Herd Heirloom
1 Imperial Seal
1 Infinite Guideline Station
1 Iron Man, Armored Avenger
1 Karn, Gilded Guardian
1 Mana Confluence
1 Marsh Flats
1 Mistrise Village
1 Mystical Tutor
1 Necropotence
1 Orcish Bowmasters
1 Overgrown Tomb
1 Path of Ancestry
1 Path to Exile
1 Plateau
1 Polluted Delta
1 Ranger-Captain of Eos
1 Reliquary Tower
1 Restoration Angel
1 Rhystic Study
1 Ringsight
1 Sacred Foundry
1 Sanguine Bond
1 Savannah
1 Scrubland
1 Silence
1 Skyclave Apparition
1 Smothering Tithe
1 Sol Ring
1 Solitude
1 Steam Vents
1 Sword of Feast and Famine
1 Swords to Plowshares
1 Taiga
1 Teferi's Protection
1 Teferi, Time Raveler
1 Teleportation Circle
1 Temple Garden
1 Thassa's Oracle
1 The Fire Crystal
1 The Mind Stone
1 The One Ring
1 The Reaper, King No More
1 The Soul Stone
1 Thought Vessel
1 Tropical Island
1 Tundra
1 Twitching Doll
1 Underground Sea
1 Urborg, Tomb of Yawgmoth
1 Vampiric Tutor
1 Vito, Thorn of the Dusk Rose
1 Volcanic Island
1 Watery Grave
1 Wooded Foothills
1 Yoshimaru, Ever Faithful
```
