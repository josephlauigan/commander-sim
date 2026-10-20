# Task: thrasios-ai, close the model gaps in JD's Tymna/Thrasios deck and make its AI play it right

Read `rust/agents/RULES.md` (skip its `git merge rust-port` step). Worktree:
`/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/thrasios-ai`, branch `thrasios-ai` (from
`thrasios-deck`, which added the deck: read `rust/agents/report-thrasios-deck.md` first).

The first real run (`audit/thrasios/2026-10-19-vs-kraum/thrasios-t5.txt`, 2,000 games vs Tier 5, look-ahead, loose)
put Tymna/Thrasios at 24.6% against Tymna/Kraum's 29.9% (`kraum-t5.txt`, same seeds). Part of that gap is the AI
not playing the cards. JD wants every model gap closed and the deck played correctly before we compare again.

## Gaps the run shows (fix each, with a test)
1. **Blossoming Defense is never cast** ("never cast by the sim: untagged"). It is cast in the Omnath deck, so the
   protection path is gated by deck key or tag. It should protect Tymna or Thrasios from targeted removal (hexproof
   in response), or pump an attacker when nothing needs protecting.
2. **Walking Ballista is cast in only 20.6% of the games it's drawn.** Outside the combo it is an X creature: cast it
   with spare mana (X >= 2 by mid-game), use the counters to kill X/1 utility creatures and planeswalker loyalty
   (opponents' mana dorks, hatebears), or ping the last points of a player. With Basalt + Rings out (or castable
   this turn) it is the win: hold it for the combo turn only if that turn is near.
3. **Thrasios activates only 0.52 times per game** although he is out in 86% of games ("ty_th_turns_eot 2.69 per
   game", "ty_th_spare4_eot 0.20"). Check the end-of-opponent's-turn sink: with {4} untapped and nothing at instant
   speed worth holding, activate (repeat while {4} remains). On your own turn, after the main plays, spend spare
   mana on him. Check the mana-creature and Talisman mana is counted for the sink.
4. **The combo assembles rarely** (plan online 2.7% of games). Check: cast priority for Basalt Monolith and Rings
   of Brighthearth (Basalt is also plain ramp: cast early); Diabolic Tutor and Worldly Tutor pick the missing combo
   piece once two of the three (Basalt, Rings, Ballista or Thrasios as the outlet) are in hand or on the battlefield;
   the deck's plan reports "online" when the combo goes off or Tymna's plan is online (the plan label is empty in the
   report: give the `thrasios` deck a plan line like the kraum seat's, or a combo milestone).
5. **Dark Endurance is cast in 5.2% of the games it's drawn** (Kraum too). Find out why (the rule may be too strict)
   and fix it for both partner seats if it's a real miss.
6. **Everything else stuck in hand under 50%** in `thrasios-t5.txt` (Avenge 30%, Sun Titan, Sphinx of New Prahv,
   Austere Command, Elspeth, Sunblast Angel, Nightveil Predator, Fact or Fiction, Diabolic Tutor): check each is a
   fair "expensive card, game ended first" case and not a priority or tag bug. Fix real bugs.
7. **The green cards new to this deck** (Faeburrow Elder, Avacyn's Pilgrim, Coiling Oracle, Paradox Surveyor,
   Witherbloom Charm, Dragonlord Dromoka, Bite Down, Worldly Tutor, Talisman of Unity, Talisman of Curiosity, the
   new lands): check they're cast and used as their Oracle text says (Faeburrow's mana = number of colours among
   your permanents; Bite Down needs a creature of yours; Witherbloom Charm picks the best of its three modes).

## Then watch it play
Trace at least 10 games (`commander-sim --deck thrasios --pool t5 --trace N --ai lookahead`), including games where
Basalt, Rings or Ballista is drawn. Look for wrong plays: holding mana with nothing to do, tapping out into
Thrasios's sink before casting a creature, attacking Tymna into bad blocks, not protecting a commander when
protection is in hand. List what you found and fixed in the report.

Fixes shared with the `kraum` seat are fine (say which), since I rerun both decks. Don't change the decklists.

## Checks
`cargo test --release -j 6` passes; `cargo build --release -j 6` has no warnings; a 200-game check
`--deck thrasios --pool t5 --games 200 --ai lookahead --analyze --jobs 6`: Blossoming Defense cast, Ballista cast
in more of its draws, Thrasios activations up. Don't run longer sims: I will.

Limits: at most 6 build threads, `nice -n 19`, memory cap per RULES.md, no Python. Handoff
`rust/agents/handoff-thrasios-ai.md`; commit after each piece (standalone, no Co-Authored-By); don't push. Report in at
most 15 lines; details in `rust/agents/report-thrasios-ai.md`.
