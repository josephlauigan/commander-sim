# Yoshimaru and Tymna (key `yoshimaru`): findings

The deck's living list of strengths and weaknesses and what was tested.

**List:** `decklists/JD/yoshimaru-tymna-blink.md` (Yoshimaru, Ever Faithful and Tymna the Weaver, plain Partner, white-black blink and
enter-effect midrange; 100 cards). **Tiers:** t4 and t5 (`data/my_decks.toml`). **No sims yet.**
The findings of the retired Tymna/Thrasios Doomsday deck are in `audit/retired/thrasios/`.

## Baseline 2026-11-23 (main f1ffd896, Stage 3; look-ahead, loose, 1,500 games per tier, seed 500000)
t4 **27.5%** (about ±2.3 at 95%, binomial arithmetic), t5 **24.5%** (±2.2): **26.0% over t4, t5**. 4-player baseline 25%; average rounds survived 8.3 (t4), 8.4 (t5). Raw (`--analyze`):
`audit/stage3-baseline-2026-11-23/yoshimaru-analyze.txt`.
This is the first simulation of this list (the intro above, written before it, still says "No sims yet"). It is a baseline from the new engine (exact X and alternative-cost pricing, hybrid mana, the
Stage 2 filters); no older number in this file is for this list or build, so none is comparable. It is also the first measurement of the Equipment package: the claims under "What to check in the first runs"
below stay untested except where named here.

- **How it wins,** t4 / t5: combo 64.6% / 62.2% of wins, damage 35.4% / 37.5% (t5 timeout 0.3%). It eliminates 0.78 / 0.71 opponents a game. t4: combo 56.4%, combat 25.3%, drain 6.6%, commander damage 6.2%,
  triggers 5.6%. t5: combo 51.4%, combat 25.6%, drain 8.1%, commander damage 7.5%, triggers 7.4%. Damage dealt 26.2 / 30.3 a game, combat 22.6 (86%) / 26.7 (88%). Tymna: 2.57 / 3.26 cards drawn a game, 3.26 / 3.38
  Tymna turns, 0.78 / 0.90 opponents hit per Tymna turn; she lands on turn 3.71 / 3.72 (in 94% / 93% of games).
- **How it loses:** t4 eliminated in 72.5% of games (round 8.1): Krenko combat 11.3% and triggers 6.1%, Edgar Markov combat 8.9% and combo 3.6%, Yuriko triggers 8.3% and combat 3.4%, Chulane combo 8.3%, Prosper
  triggers 4.2%. t5 75.5% (round 8.0): Kinnan combo 14.8% and combat 3.2%, Urza combo 14.1%, Miirym combat 10.1% and triggers 5.7%, Zur combo 9.9% and commander damage 2.1%, Yawgmoth combo 8.7%.
- **Plan timing** (the plan line prints blank), t4 / t5: online by T4 1% / 1%, T5 4% / 3%, T6 7% / 4%, T8 14% / 10%, T12 19% / 16%; at all 19.5% / 17.5%. The commander lines disagree: "resolves on turn" gives median 1.0
  (mean 1.5 / 1.6) over 99.1% / 99.5% of games, but "resolved by turn" gives T4 33% / 35% and T8 35% / 38%. The output does not explain it: unmeasured, and worth a check before anyone uses either line.
- **Often stuck in hand** (cast in under half the games drawn), t4 / t5: Silence 1.4% / 0.7%, Eerie Interlude 5.5% / 5.3%, Solitude 13.8% / 7.6%, Restoration Angel 15.1% / 16.6% (reactive), Teferi's Protection 17.1% / 16.7%
  (reactive), Conjurer's Closet 21.1% / 19.2%, Teleportation Circle 24.6% / 20.3%, Forge Anew 26.0% / 24.7%, Flawless Maneuver 37.9% / 33.2%, The Wandering Emperor 39.5% / 40.4%, Gandalf the White 43.1% / 35.0%; t4 only:
  Court of Ardenvale 42.1%; t5 only: Wrath of God 34.0%.
- **Most removed by opponents (per game),** t4 / t5: Yoshimaru 0.47 / 0.48, Tymna 0.14 / 0.18, Sword of Fire and Ice 0.08 / 0.05, Sting 0.05 / 0.02, Sword of Hearth and Home 0.04 / 0.02, Skrelv 0.04 / 0.04, Mana Vault 0.04 (t4),
  Exquisite Blood 0.04 (t4), Adeline 0.03 (t5), Elesh Norn, Mother of Machines 0.02 (t5).

What the numbers say (correlation, not causation):
- **t4 is 3.0 points above t5.** 27.5% against 24.5%, borderline against the intervals (a difference of two independent runs needs about ±3.1) and between different opponents, so not a paired result. The t5 losses are the
  fast artifact and mana combos (Kinnan 14.8%, Urza 14.1%, Zur 9.9%, Yawgmoth 8.7% combos: 47.5% of games); at t4 the combos are Chulane 8.3% and Edgar Markov 3.6%, and combat and triggers decide more.
- **Combo is most of the wins.** About 17.8% (t4) and 15.2% (t5) of games are won by combo (arithmetic from the win rate and the combo share), close to the 19.5% / 17.5% the plan line calls online; what that line counts is not
  named in the output, so read it as a similarity of numbers only: unmeasured.
- **The Ballista and Heliod line is the best-performing.** Walking Ballista wins 69.9% / 66.3% of the games it is cast in (cast in 20.8% / 20.2% of games) and Heliod, Sun-Crowned 68.1% / 57.9% (18.8% / 19.3%); Exquisite Blood
  55.6% / 51.5%, Vito 48.9% / 48.2%. Archangel of Thune wins 45.1% / 39.2% (cast in 12.3% / 11.7%): the guide's Archangel + Ballista line is one the plan file leaves out (above). A high win rate when cast can also mean a card is
  only cast in games already going well.
- **The blink pieces sit in hand.** Conjurer's Closet is cast in 21.1% / 19.2% of the games drawn, Teleportation Circle 24.6% / 20.3%, Forge Anew 26.0% / 24.7%; Silence 1.4% / 0.7% and Eerie Interlude 5.5% / 5.3% are not flagged
  reactive and are nearly never cast. Why is not in the output: unmeasured. Whether the plan file's Equipment rows put Swords on the right creatures (the first item under "What to check in the first runs") is also not in
  `--analyze`: unmeasured.
- **The commanders are the targets and add nothing alone.** Yoshimaru is removed 0.47 / 0.48 times a game and Tymna 0.14 / 0.18, far above any other card (Sword of Fire and Ice is next at 0.08 / 0.05). Both are cast in 94-99% of
  games and win at the deck's own rate (Yoshimaru 27.4% / 24.5%, Tymna 26.9% / 24.7%). Stoneforge Mystic is the lowest card at t4 (23.2% when cast, -4.2, cast in 14.1% of games), the only one clearly below the deck.

## What the simulator cannot do yet
- Every card is Full or Full-auto except Delney, Streetwise Lookout (Approximate: the extra instance for leaves that run at once or inline in the
  engine is left out; see her note in `data/audit.toml`). Full-auto means compiled from Oracle text, not hand-written card code: Stroke of Midnight and
  Sword of Hearth and Home. **Steelshaper's Gift is rated Full-auto but is wrong** (defect 9 below).
- `data/plans/yoshimaru.toml` is provisional: priorities come from the guide, unmeasured; the keep rule is the generic lands 2 to 5.
  It leaves out the Archangel of Thune + Walking Ballista line and the guide's Restoration Angel + Felidar Guardian line (Spellbook
  1090-2781, listed in `KNOWN_GAPS` in `tests/spellbook.rs`).

## 2026-11-22: Zack Fair => Skrelv, Defector Mite (JD's choice, not simulated)
Skrelv ({W}, a legendary artifact creature 1/1 with toxic 1 that can't block; {W/P}, {T}: another creature we control gains toxic 1 and hexproof from a
chosen colour and can't be blocked by creatures of that colour until end of turn; model status Full; a proxy) takes Zack Fair's slot. Opus recommended it over
Giada, Selfless Spirit, Mother of Runes and Karlov of the Ghost Council as the closest slot-for-slot card; JD chose it. Both are one-mana legendary white creatures:
the count of legendary permanents that give Yoshimaru a counter stays at 21, Mox Amber still turns on with it, and Elas, Haliya, Ranger-Captain of Eos
and Recruiter of the Guard find or reward it the same way. What it gains: protection against targeted removal of one colour (exile, bounce and -X/-X as
well as destroy), and unblockability plus toxic for the creature it helps, which can feed Tymna's draws; the AI uses it in response to removal on a commander or a creature worth 4 or more, and before combat on the best attacker against the colour of the blockers that matter. What it gives up: Zack Fair's {1} sacrifice worked the
turn he came down and saved one creature from a destroy wrath, and moved one Equipment (Skrelv needs {T}, so it does nothing the turn it lands, can't
block and moves no Equipment). Counts after: artifacts 18 (17), so the chance of metalcraft (three artifacts) in the first 12 cards goes from 34.5% to
38.4%; Chrome Mox can imprint 46 cards (47), because Skrelv is an artifact; legendary permanents 21 (21); one-drops, protection (6) and creatures (28)
unchanged. The plan file gives Skrelv Zack Fair's cast priority (66 once a commander is out, else 54). Not measured. To check in the first run: how often
Skrelv's ability is used and what it answers, whether the creature it makes unblockable connects for Tymna, and whether the missing Equipment mover and
destroy-wrath save matter against sweeper tables.

## 2026-11-21: the Equipment package (22 swaps, not simulated)
What changed and why is in `decklists/CHANGELOG.md`; the guide's "The Equipment package" describes the cards. **Nothing has been simulated:
every statement below is a claim to test, not a result.**

### What to check in the first runs
- **Where the AI puts Equipment.** `data/plans/yoshimaru.toml` has Cloud's Equipment rows (priorities, `hand:equipment` conditions) but not Cloud's
  `[equip] holder = "commander"`, which would load the Swords on Yoshimaru or Tymna. The guide wants fliers (Restoration Angel, Flickerwisp, Archangel of Thune,
  Sunpearl Kirin) and only Swiftfoot Boots on Tymna, and Sword of Hearth and Home never on Walking Ballista. Look at the traces for who wears what and
  whether Tymna ends up with +power (Delney's doubling and evasion switch off at power 3).
- **Flawless Maneuver in response to our own Wrath of God.** The guide's one-sided wipe needs it; the generic policy may not do it.
- **Court of Ardenvale and Walking Ballista.** Court returns a permanent card of mana value 3 or less to the battlefield while we are the monarch; Ballista
  would enter with X = 0 and die. Check what the AI returns.
- **Mana.** JD's report (11/21) was 3 to 4 mana for most of the game with the old list. The outside goldfish estimate for 35 lands, 6 rocks and Wayfarer's Bauble is a mean of
  4.1 mana on turn 4, 4.8 on turn 5, 5.3 on turn 6 and 6.3 on turn 8 (on the play; not the simulator). Compare with the simulator's turn-by-turn mana and with
  `--goldfish yoshimaru,t5`.
- **Delney conflicts.** Cloud, Stoneforge Mystic, Kor Outfitter and Puresteel Paladin have power 2 or less, so Delney doubles their enter and combat
  triggers (Skrelv has no triggered ability); any Sword on them removes that. A Sword on Cloud (equipped, so his own doubling applies) is the intended exception.
- **Protection from white.** Sword of Hearth and Home's carrier cannot be targeted by our white spells and abilities (Ephemerate, Restoration Angel, Felidar Guardian,
  Flickerwisp, Teleportation Circle, Eerie Interlude, Giver of Runes, Heliod's lifelink, Skrelv's ability, Kor Outfitter's and Ardenn's attach); Equipment's own
  equip abilities, Puresteel Paladin's equip {0}, Forge Anew and Mithril Coat's attach trigger still work.

### Costs accepted by JD with the list (11/21)
Combo stops are down to Silence and Ranger-Captain of Eos; answers to noncreature permanents are Stroke of Midnight, Skyclave Apparition and Witch Enchanter; Line C is
a variant of Line B or A (Sorin is out); Line D's payoffs are Elas il-Kor, Haliya, Vito and Archangel of Thune; Wrath of God cannot touch opposing indestructible
creatures; Stroke of Midnight gives the opponent a 1/1 Human; the deck's black is oversupplied (no card costs {B}{B}).

### Engine defects found by the interaction tests (fixed by the engine change that lands with this list)
Tests of the package's card interactions (`rust/crates/sim-core/tests/cards_yoshimaru_equipment.rs`, 128 tests) found twelve engine defects; all twelve are fixed
by the engine change, which also found and fixed more of the same class (Giver of Runes and the AI's other protectors, Eerie Interlude, Teleportation Circle and
the other end-step flickers, Momentary Blink, the person's own Ephemerate, the practice-mode Kor Outfitter and Ardenn choices, Zack Fair's practice-mode choice of
Equipment, Secure the Scene's token). Each line: the card, what the engine did, what the Oracle text says.
1. Court of Ardenvale never returned a land card; a land card is a permanent card with mana value 0 (now returned through the real land-entry path).
2. Zack Fair's sacrifice moved every Equipment attached to him; the text attaches "an Equipment" (now one, the most valuable).
3. Zack Fair's sacrifice could target a creature with protection from white (a Sword of Hearth and Home carrier), though Zack is white.
4. Kor Outfitter's enter trigger attached an Equipment to the protected carrier, though Kor Outfitter is white.
5. Ardenn's beginning-of-combat attach did the same.
6. Restoration Angel's enter trigger blinked the protected carrier, though the Angel is white.
7. Felidar Guardian's enter trigger blinked the protected carrier, though Felidar is white.
8. The AI offered Ephemerate on the protected carrier, though Ephemerate is white.
9. Steelshaper's Gift was compiled with an empty search filter and fetched any card (it found Sol Ring); it now searches only for an Equipment card.
10. Stroke of Midnight's token was a plain 1/1 of the controller's colour, not a white Human.
11. Wrath of God had no "can't be regenerated" tag, so a regeneration shield saved a creature.
12. Metalcraft counted only artifact permanents, not Treasure, Food, Clue and Blood tokens (Lotho makes Treasures).
Fixed through the existing `protected_from` helper at the call sites that lacked it (equip abilities are colourless and unchanged; Deflecting Swat, Siren Stormtamer and
Not of This World target the spell, so they are exempt). `best_holder`, `zack_sacrifice`, `metalcraft` and `resto_etb` are shared with the retired Cloud card code, so
its numbers can shift.
