# Yoshimaru and Tymna (key `yoshimaru`): findings

The deck's living list of strengths and weaknesses and what was tested.

**List:** `decklists/JD/yoshimaru-tymna-blink.md` (Yoshimaru, Ever Faithful and Tymna the Weaver, plain Partner, white-black blink and
enter-effect midrange; 100 cards). **Tiers:** t4 and t5 (`data/my_decks.toml`). **No sims yet.**
The findings of the retired Tymna/Thrasios Doomsday deck are in `audit/retired/thrasios/`.

## 2026-11-24: paired test of the PR #394 package (new Akroma's Will list against the old Teferi's Protection list)
One paired run on main 654db206 (the merged PR #394), t4 and t5, loose, 1,500 games per list per tier, seed 500000, same opponents and draws. Raw: `audit/arzakon-package-paired-2026-11-24/yoshimaru-paired.txt` (read its README: the run swaps the package back, so the harness prints "current" for the NEW list and "swapped" for the OLD list, and its `delta` is old minus new; everything below is new against old).
**The numbers:** t4 new **27.3%**, old **26.6%**, new minus old **+0.7** (paired noise ±1.5; the table's band for the win-rate row ±3.2). t5 new **23.7%**, old **24.1%**, new minus old **-0.3** (±1.5; table band ±3.1). About 25.5% against 25.4% over t4, t5 (arithmetic from the two tier numbers each). Both tiers: "no clear effect", no axis with a star.
**The card (the "Why" table):**
- **Akroma's Will (new),** t4 / t5: drawn 17.9% / 19.1% of games, cast in 33.6% / 30.3% of the games drawn, first cast in round 6.6 / 7.4, stuck in hand 66.4% / 69.7%, wins 52.2% / 49.4% of the games it is cast in, removed 0.00 a game.
- **Teferi's Protection (old),** t4 / t5: drawn 17.6% / 19.3%, cast 14.4% / 20.0%, round 8.9 / 9.7, stuck 85.6% / 80.0%, win when cast 28.9% / 25.9%, removed 0.00 a game. (The 11-23 baseline had it cast in 17.1% / 16.7%: the same range.)
**Game shape (new -> old, t4 then t5; none beyond its band):** plan (first Tymna draw, or a combo going off) online at all 82.9% -> 83.1% and 86.3% -> 86.4%; rounds survived 8.35 -> 8.29 and 8.50 -> 8.52; eliminated by the opponents in 72.7% -> 73.4% and 76.1% -> 75.9% of games; damage dealt per game 27.5 -> 26.8 (new minus old +0.72, band ±2.26) and 32.0 -> 32.3; commander removals per game 0.60 -> 0.60 and 0.68 -> 0.65; mana on turn 8 7.35 -> 7.31 and 7.53 -> 7.57; wins by combo 57% -> 60% and 55% -> 54%, damage 43% -> 40% and 43% -> 45%.
**Reading (correlation, not causation):** a tie on both tiers at ±1.5, so the output supports no gain and no loss from the swap; it does not show that either card is neutral for a smaller effect. Akroma's Will is cast in about a third of the games it is drawn and sits in hand about two thirds of the time, more than twice as often cast as Teferi's Protection on t4 (33.6% against 14.4%) and 1.5 times on t5 (30.3% against 20.0%). Its win rate when cast (52.2%, 49.4%) is about double the deck's, which a card cast in games that are going well would show anyway, and the output does not say which mode it was cast in or whether the double-strike kills (the Swords, Vito, Heliod) happened: unmeasured. The 11-23 entry's engine limit stands: the AI casts it only on its own turn after blockers, so a sim measures it as a finisher. What the swap costs, total protection against exile or -X/-X wipes, is not a measured axis: the table has no wipe-loss row. Unmeasured.
**What to check next:** how often Akroma's Will is cast in which mode and what turn it is cast; why it is stuck in hand two thirds of the games drawn (the plan has no row for it, so it uses the generic rating); and, for the Teferi's Protection side, how many losses came right after a wipe or an exile in the games it was held. Four times the games halve the noise. No swap is proposed here.

## 2026-11-23: Teferi's Protection => Akroma's Will (JD's choice, not simulated when written; paired test in the 2026-11-24 entry above)
Akroma's Will ({3}{W} instant: choose one, or both if you control a commander; flying, vigilance and double strike, or lifelink, indestructible and protection from each colour, for your creatures until end of turn; model status Full; owned, from the Arzakon deck) takes Teferi's Protection's slot (a proxy, a Game Changer). Opus called the pair sound with a job lost.
**What it gives:** with Yoshimaru or Tymna on the battlefield both modes are live, so it is an on-plan finisher: double strike doubles the combat triggers of Sword of Fire and Ice, Sword of Feast and Famine and Sword of Hearth and Home, and double strike with lifelink feeds the life-gain kills (Vito with Exquisite Blood, Heliod, Archangel of Thune). Its second mode (indestructible and protection from each colour) answers a destroy or damage sweeper and coloured removal at instant speed. **What it costs:** Teferi's Protection's total protection (permanents phased out, protection from everything, life frozen): it stopped exile and -X/-X wipes, targeted removal and a drain or damage kill on the life total. Akroma's Will protects creatures only and does nothing against Farewell or Toxic Deluge. Teferi's Protection was cast in 17.1% (t4) and 16.7% (t5) of the games it was drawn (before, reactive only). The anti-wipe cover that stays: Flawless Maneuver (free with a commander out), Giver of Runes, Mithril Coat, Eerie Interlude, and Plaza of Heroes on a legend.
**Counts after (before):** Game Changers 8 (9); instants 11 (11); mana value 3 cards 19 (20), mana value 4 cards 9 (8); the average mana value of the 63 nonland cards in the 98 goes from 2.56 to 2.57 (162 over 63); legendary permanents 21 (21); protection cards still 6.
**Engine limit to know:** the AI casts Akroma's Will only on its own turn after blockers, never in response to a sweeper or removal, so a sim measures it as a finisher, not as protection; its both-modes text needs a commander on the battlefield, which Yoshimaru and Tymna usually provide. Plan file: the `[[hold]]` row for Teferi's Protection is removed and Akroma's Will has no row (generic rating).
**To check in the first run:** how often it is cast and in which mode; whether the double strike kills show up (Sword combat triggers, Vito, Heliod); and what sweepers cost the deck now that total protection is gone (Farewell, Toxic Deluge and other exile or -X/-X effects have only Eerie Interlude).

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
- **Often stuck in hand** (cast in under half the games drawn), t4 / t5: Silence 1.4% / 0.7%, Eerie Interlude 5.5% / 5.3%, Solitude 13.8% / 7.6% (undercounted: evokes were not recorded as casts; fixed in the solitude-fix PR), Restoration Angel 15.1% / 16.6% (reactive), Teferi's Protection 17.1% / 16.7%
  (reactive), Conjurer's Closet 21.1% / 19.2%, Teleportation Circle 24.6% / 20.3%, Forge Anew 26.0% / 24.7%, Flawless Maneuver 37.9% / 33.2%, The Wandering Emperor 39.5% / 40.4% (undercounted: flash casts were not recorded as casts; fixed in the solitude-fix PR), Gandalf the White 43.1% / 35.0%; t4 only:
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

## Silence knob, paired test 2026-11-23 (branch stage4-protect-knob 223d437b against main f1ffd896; same seeds, loose, 1,500 games per tier)
`[knobs] protect_before_go = true` (Silence cast before a catalogue line goes). t4 **27.5% -> 27.6%**, t5 **24.5% -> 24.8%**: inside the noise (standard error about 1.1 points), as predicted. The mechanism moved: Silence cast in **1.4% -> 4.7%** (t4) and **0.7% -> 4.1%** (t5) of the games it is drawn; combo share of wins 64.6% -> 64.7% (t4), 62.2% -> 63.2% (t5). Raw: `audit/stage3-paired-2026-11-23/knob-yoshimaru-analyze.txt`.
- The knob reaches only lines that go through `combos::attempt`: Heliod + Walking Ballista and Vito + Exquisite Blood. Archangel + Ballista and Restoration Angel + Felidar Guardian are not in the combo catalogue, so it does nothing for them.
- Not measured: how often the protect step fired (the `t6_protect` counter is not printed; ticket T-S4-3); whether Silence was countered more or less.

## Equipment carrier rule, paired tests 2026-11-23 (arms on seed 500000 x 1,500 games and seed 700000 x 3,000 games per tier, loose)
Arm A = main with the Silence knob; arm B = the full carrier package (flier +2, unblocked +1.5, Yoshimaru -4, Delney -4, Boots +4 on Tymna, plus the Ballista veto and Tymna -6, free-attach pair scoring); arm C = veto only (Sword of Hearth and Home never on Walking Ballista, no +power Equipment on Tymna, nothing else scored). Raw: `audit/stage3-paired-2026-11-23/big-*`.
- Win rate, seed 700000 x 3,000: t4 A 26.3, B 25.7, C 25.7; t5 A 22.0, B 23.0, C 22.3. Seed 500000 x 1,500: t4 A 27.6 -> B 26.5; t5 A 24.8 -> B 23.1. All inside the noise (about 0.8 per tier per arm at 3,000 games).
- **Commander-damage eliminations by this deck, share of its eliminations (t4 / t5):** A 5.4 / 6.8, B 2.9 / 4.2, **C 5.6 / 6.7**. The halving in B replicated on both seeds and is gone in C: the guessed weights (Yoshimaru -4 and the flier stacking), not the veto or the Tymna rule, cause it. The guide says commander damage "is not a plan" (line 96), from an unsimulated list; a 5-7% share of eliminations does not contradict that, but the weights cost it for nothing measurable in return.
- Equipment worn per game (t4 / t5): B commander 0.22 / 0.26, flier 0.15 / 0.18, other 0.54 / 0.65; C commander 0.43 / 0.48, flier 0.12 / 0.15, other 0.31 / 0.45; protection from white on Ballista 0.00 in both. Tymna cards drawn and opponents hit per Tymna turn are flat in all arms.
- Decision: ship arm C (the guide-backed rules) with the report and Court fixes; the other weights stay out until a measured reason. No arm shows a win-rate gain, so this is rule correctness, not a measured improvement.

## Solitude fixes, paired test 2026-11-23 (branch solitude-fix against main 47146456; seed 700000, 3,000 games per tier, loose)
Win rate t4 25.7 -> 25.8, t5 22.3 -> 22.5 (baseline = the carrier-rule arm): no change. Mechanisms: Solitude no longer appears in the "often stuck in hand" list (it read 16.2% and 11.5% of draws; the figure was an artifact: evokes were not recorded as casts), so its counted cast rate is now above half of draws (the exact value is not printed: ticket); The Wandering Emperor cast when drawn 42.9 -> 43.3 (t4), 39.1 -> 44.0 (t5). Heliod cast 17.7 -> 18.0 / 17.6 -> 17.9. Raw: `audit/stage3-paired-2026-11-23/big-solitude-yoshimaru-analyze.txt`.
- In 60 traces (`solitude-trace-counts.txt`): evokes that ended in `ward counters the effect` 3 -> 0; Heliod, Sun-Crowned pitched 1 -> 0. A Deflecting Swat redirect (opposing play) is not fixed.
- Commander traces for the other decks are identical to main (sheoldred, urdragon, yshtola and the within-tier pods: 160 of 160); 4 of 80 yoshimaru games differ, as expected when evokes and flash casts are counted and the look-ahead plays the changed code.

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
