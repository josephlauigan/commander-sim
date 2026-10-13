# Task: sauron-threats — cards for Sauron's speed and backup-win packages

Read `rust/agents/SAURON-RULES.md` first. Sauron's Army starts late (a six-mana commander, an Army that starts at 0)
and the deck has one way to win. The lead will test packages in the four Underworld Breach slots (Underworld Breach,
Brain Freeze, Dark Ritual, Lotus Petal) and in Sauron's slow slots:
- **Backup wins:** Dualcaster Mage + Twinflame (an infinite combo), Exsanguinate, Torment of Hailfire.
- **More creatures:** Grishnákh, Brash Instigator, Lazotep Reaver, Grim Initiate, Inferno Titan (Inferno Titan is
  sauron-sweepers' card; don't model it).
- **Speed:** Talisman of Indulgence, Rakdos Signet, Fellwar Stone, Dreadhorde Invasion, Lazotep Reaver.

## Cards (Full, from Oracle text, each with a test)
- Not modeled: **Lazotep Reaver**, **Grim Initiate**, **Torment of Hailfire** (fetch Oracle text, see the rules).
- Partial (one unparsed clause each): **Dualcaster Mage**, **Dreadhorde Invasion**, **Grishnákh, Brash
  Instigator**. Finish them.
- Check Full and working in Sauron's deck: **Twinflame**, **Exsanguinate**, **Talisman of Indulgence**, **Rakdos
  Signet**, **Fellwar Stone**.
- Amass with a type (Zombies or Orcs) puts counters on the Army you already control whatever its type (rule 701.47a,
  the Army gains the type); check the engine does that, so Sauron's Orc Army gets the Zombie counters.

## Dualcaster Mage + Twinflame
Twinflame on a creature you control; with it on the stack, flash in Dualcaster Mage copying it and target Dualcaster
Mage: each token copy of Dualcaster copies Twinflame again, so you get as many hasty 2/2 Dualcasters as you like.
Grep how other infinite combos are modeled (`milestone "combo"`, `pl.stat("combo"...)`, `impls/mine.rs` combo
lines, Kiki-Jiki or Splinter Twin style lines in the pool decks) and model it the same way: the AI recognizes the
two cards and the mana ({1}{R} + {1}{R}{R}), goes for it on its own turn before combat when it can win (enough
attackers to kill every opponent through their untapped blockers; when in doubt make a large finite number), and
it can be countered or responded to like any spell. Both cards also do their normal job when the combo isn't there.

## AI (Sauron's deck, gated on the card)
- Early amass creatures and Dreadhorde Invasion are cast early (turn 1-3 plays), rocks on curve before them.
- Exsanguinate and Torment of Hailfire: finishers, X as large as the mana allows, cast when lethal or late.

## Smoke run
`./target/release/commander-sim --deck sauron --pool t3 --profile loose --games 96 --ai lookahead --jobs 6 --analyze
--swap "Underworld Breach=>Dualcaster Mage" --swap "Brain Freeze=>Twinflame" --swap "Dark Ritual=>Exsanguinate"
--swap "Lotus Petal=>Torment of Hailfire"`, and one with the speed cards (`Deepglow Skate=>Talisman of Indulgence`,
`Conqueror's Flail=>Rakdos Signet`, `Sword of Fire and Ice=>Dreadhorde Invasion`, `Flux Channeler=>Lazotep
Reaver`). Report how often each new card was cast when drawn and how often the Dualcaster combo went off.
