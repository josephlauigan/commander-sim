# Dack Fayden — flip-trace rules sheet (shared by the trace agents)

Written by the lead 2026-10-17 from Scryfall Oracle text (below). Every trace agent uses these facts; if you think one
is wrong, say so in your file's notes instead of silently changing it.

## Setup for every pile

- Pod of 4: you (Dack's controller) and three opponents. Dack Fayden, Helping Hand has just resolved and its ETB
  flips exactly three creature cards ("the pile"). Before Dack: nothing else of yours on the battlefield that matters
  (lands and mana rocks only), your hand holds no relevant cards, your graveyard is empty, and you have **2 untapped
  {W} available** after casting Dack (so 2 mana left over), unless a row says otherwise.
- **Opponents do not interact** (goldfish trace). Weakness to interaction is handled elsewhere.
- Our seven creatures: Abdel Adrian (Ab), All-Fates Stalker (St), Felidar Guardian (Fe), Fiend Hunter (FH),
  Preston, the Vanisher (Pr), Radiant Solar (So), Werefox Bodyguard (Wf). The library contains all seven; after the
  flip it is shuffled, so later flips (a second Dack ETB) are random among the four not flipped.

## Rules facts to use

1. **The three flipped creatures enter the battlefield simultaneously under YOUR control**, then, still during the
   resolution of Dack's ability, each opponent gains control of one of them. Control has changed before any triggered
   ability is put on the stack.
2. **Triggered abilities are controlled by the player who controlled the source when it triggered** (CR 603.3a), so
   every ETB of the flipped creatures, and Preston's trigger, are YOURS, even though the creatures now belong to
   opponents' control. "You" in those abilities means you. Targets are chosen when the trigger is put on the stack
   (after the control change).
3. **Enter-the-battlefield abilities see permanents entering at the same time** (CR 603.6a): if Preston or Radiant
   Solar is in the pile, it triggers for the other two pile creatures (and Solar for itself).
4. **Preston** triggers on "another NONTOKEN creature you control enters, if it wasn't cast": the token copy is a 0/1
   white Illusion under your control with all the copied creature's abilities (including its ETB, which triggers when
   the token enters). Tokens entering never trigger Preston. Preston needs to be under your control when the creature
   enters; a donated Preston (opponent controls it) does nothing for you after this first batch.
5. **Felidar Guardian's ETB** targets "another target permanent you control" and returns it "under its owner's
   control". You own all these cards, so a flickered card comes back under YOUR control, as a new object, not cast
   (it triggers Preston and Solar again if you control them).
6. **Banishers** (Fiend Hunter: "another target creature"; All-Fates Stalker: "non-Assassin creature" until it
   leaves; Werefox Bodyguard: "other target non-Fox creature" until it leaves) CAN target opponent-controlled
   creatures. When the banisher leaves the battlefield, the exiled card returns under its OWNER's control (you), not
   cast. A token copy of a banisher (from Preston) works the same way and leaves when it dies, is flickered (a token
   flickered ceases to exist) or is exiled. Fiend Hunter's return is a leaves trigger; Stalker and Werefox are
   "until" durations (immediate return when they leave).
7. **Abdel Adrian** exiles any number of other nonland permanents YOU control until Abdel leaves, making a 1/1
   Soldier per card; they return under their owner's control when Abdel leaves.
8. **Radiant Solar** ventures into the dungeon whenever it or another nontoken creature you control enters. A Preston
   token copy of Solar has the same trigger. Choose **Lost Mine of Phandelver**: room 1 Cave Entrance (scry 1),
   room 2 Goblin Lair or Mine Tunnels, room 3 **Dark Pool (each opponent loses 1 life, you gain 1)**, room 4 Temple of
   Dumathoin (draw). After room 4 the dungeon is completed and the next venture starts a new one. So **every 4
   ventures drain each opponent for 1**; 40 life each = about 160 ventures = needs an unbounded loop.
9. **Dack's own ETB** retriggers if Dack (a nontoken creature you control) is flickered: it flips three MORE
   creatures from the shuffled library, donating them too. A Preston token copy of Dack also has the ETB (flips three
   more). Each opponent must receive one; if fewer than three creature cards remain, flip what remains.
10. **Win conditions to look for:** an unbounded loop of a nontoken creature card you control re-entering (each entry
    giving a Preston token whose ETB re-flickers it), with Solar or a Solar token under your control to venture each
    time = drain the table. Or the two-banisher loop (two banishers alternately exiling each other's targets) with a
    Solar trigger. Finite but large damage is NOT a win unless it reaches 40 per opponent.
11. Our flicker spells (only if a row says they are in hand): **Ephemerate** {W} (exile target creature you control,
    return under its owner's control; rebound), **Cloudshift** {W} (exile target creature you control, return under
    your control), **Touch the Spirit Realm** channel {1}{W} (exile target artifact or creature, return at the next end
    step under its owner's control) or cast {2}{W} (exile until it leaves).

## Outcome labels

- **WIN-NOW**: unbounded drain this turn with no other cards (only the 2 spare {W}).
- **WIN+1**: wins this turn if you also hold ONE of Ephemerate / Cloudshift / Touch the Spirit Realm (say which).
- **NEEDS-2ND-FLIP**: only through a second Dack ETB (give the chance that the second flip completes a win).
- **NO-WIN**: no win this turn from the pile.

## Oracle text
### Lost Mine of Phandelver  — Dungeon

Cave Entrance — Scry 1. (Leads to: Goblin Lair, Mine Tunnels)
Goblin Lair — Create a 1/1 red Goblin creature token. (Leads to: Storeroom, Dark Pool)
Mine Tunnels — Create a Treasure token. (Leads to: Dark Pool, Fungi Cavern)
Storeroom — Put a +1/+1 counter on target creature. (Leads to: Temple of Dumathoin)
Dark Pool — Each opponent loses 1 life and you gain 1 life. (Leads to: Temple of Dumathoin)
Fungi Cavern — Target creature gets -4/-0 until your next turn. (Leads to: Temple of Dumathoin)
Temple of Dumathoin — Draw a card.

### Tomb of Annihilation  — Dungeon

Trapped Entry — Each player loses 1 life. (Leads to: Veils of Fear, Oubliette)
Veils of Fear — Each player loses 2 life unless they discard a card. (Leads to: Sandfall Cell)
Sandfall Cell — Each player loses 2 life unless they sacrifice a creature, artifact, or land of their choice. (Leads to: Cradle of the Death God)
Oubliette — Discard a card and sacrifice a creature, an artifact, and a land. (Leads to: Cradle of the Death God)
Cradle of the Death God — Create The Atropal, a legendary 4/4 black God Horror creature token with deathtouch.

### Dungeon of the Mad Mage  — Dungeon

Yawning Portal — You gain 1 life. (Leads to: Dungeon Level)
Dungeon Level — Scry 1. (Leads to: Goblin Bazaar, Twisted Caverns)
Goblin Bazaar — Create a Treasure token. (Leads to: Lost Level)
Twisted Caverns — Target creature can't attack until your next turn. (Leads to: Lost Level)
Lost Level — Scry 2. (Leads to: Runestone Caverns, Muiral's Graveyard)
Runestone Caverns — Exile the top two cards of your library. You may play them. (Leads to: Deep Mines)
Muiral's Graveyard — Create two 1/1 black Skeleton creature tokens. (Leads to: Deep Mines)
Deep Mines — Scry 3. (Leads to: Mad Wizard's Lair)
Mad Wizard's Lair — Draw three cards and reveal them. You may cast one of them without paying its mana cost.

### Touch the Spirit Realm {2}{W} — Enchantment

When this enchantment enters, exile up to one target artifact or creature until this enchantment leaves the battlefield.
Channel — {1}{W}, Discard this card: Exile target artifact or creature. Return it to the battlefield under its owner's control at the beginning of the next end step.

### Ephemerate {W} — Instant

Exile target creature you control, then return it to the battlefield under its owner's control.
Rebound (If you cast this spell from your hand, exile it as it resolves. At the beginning of your next upkeep, you may cast this card from exile without paying its mana cost.)

### Cloudshift {W} — Instant

Exile target creature you control, then return that card to the battlefield under your control.

### Static Prison {W} — Enchantment

When this enchantment enters, exile target nonland permanent an opponent controls until this enchantment leaves the battlefield. You get {E}{E} (two energy counters).
At the beginning of your first main phase, sacrifice this enchantment unless you pay {E}.

### Portable Hole {W} — Artifact

When this artifact enters, exile target nonland permanent an opponent controls with mana value 2 or less until this artifact leaves the battlefield.

### Soul Partition {1}{W} — Instant

Exile target nonland permanent. For as long as that card remains exiled, its owner may play it. A spell cast by an opponent this way costs {2} more to cast.

### Abdel Adrian, Gorion's Ward {4}{W} — Legendary Creature — Human Warrior 4/4

When Abdel Adrian enters, exile any number of other nonland permanents you control until Abdel Adrian leaves the battlefield. Create a 1/1 white Soldier creature token for each permanent exiled this way.
Choose a Background (You can have a Background as a second commander.)

### All-Fates Stalker {3}{W} — Creature — Drix Assassin 2/3

When this creature enters, exile up to one target non-Assassin creature until this creature leaves the battlefield.
Warp {1}{W} (You may cast this card from your hand for its warp cost. Exile this creature at the beginning of the next end step, then you may cast it from exile on a later turn.)

### Felidar Guardian {3}{W} — Creature — Cat Beast 1/4

When this creature enters, you may exile another target permanent you control, then return that card to the battlefield under its owner's control.

### Fiend Hunter {1}{W}{W} — Creature — Human Cleric 1/3

When this creature enters, you may exile another target creature.
When this creature leaves the battlefield, return the exiled card to the battlefield under its owner's control.

### Preston, the Vanisher {3}{W} — Legendary Creature — Rabbit Wizard 2/5

Whenever another nontoken creature you control enters, if it wasn't cast, create a token that's a copy of that creature, except it's a 0/1 white Illusion.
{1}{W}, Sacrifice five Illusions: Exile target nonland permanent.

### Radiant Solar {5}{W} — Creature — Angel 3/6

Flying, lifelink
Whenever this creature or another nontoken creature you control enters, venture into the dungeon. (Enter the first room or advance to the next room.)
{W}, Discard this card: Venture into the dungeon and you gain 3 life.

### Werefox Bodyguard {1}{W}{W} — Creature — Elf Fox Knight 2/2

Flash
When this creature enters, exile up to one other target non-Fox creature until this creature leaves the battlefield.
{1}{W}, Sacrifice this creature: You gain 2 life.

### Dack Fayden, Helping Hand {4}{W}{W} — Legendary Creature — Human Advisor 4/6

When Dack Fayden enters, reveal cards from the top of your library until you reveal X creature cards, where X is the number of opponents you have. Put those creature cards onto the battlefield, then shuffle. They're goaded for the rest of the game. For each of those permanents, choose a different opponent. Each opponent gains control of the permanent for which they were chosen.

