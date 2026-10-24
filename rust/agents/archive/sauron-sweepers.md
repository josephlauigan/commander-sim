# Task: sauron-sweepers — one-sided board control for Sauron

Read `rust/agents/SAURON-RULES.md` first. Sauron's symmetric wipes (Toxic Deluge, Blasphemous Act) are among his worst
cards in the sim: they kill his own Orc Army, his one engine. JD wants to test one-sided board control in their place.
The lead will test three two-card packages (Toxic Deluge and Blasphemous Act out):
A) Ral and the Implicit Maze + Massacre Wurm, B) Widespread Brutality + Crippling Fear, C) Chandra's Ignition +
Inferno Titan.

## Cards (Full, from Oracle text, each with a test)
- **Crippling Fear**, **Widespread Brutality**, **Chandra's Ignition**, **Inferno Titan**: not modeled yet (not in the
  Scryfall cache either; see the rules file for fetching them).
- **Ral and the Implicit Maze**, **Massacre Wurm**: already modeled (pool / manual). Confirm they are Full and that
  the one-sided part really spares Sauron's side (test).
Notes: Crippling Fear's creature type is chosen as it resolves: Sauron's AI names Orc (the Army is an Orc Army) unless
a different type saves more of its own value. Widespread Brutality: read the Oracle text for which Army it amasses;
damage to each non-Army creature (Sauron's other creatures too). Chandra's Ignition: the source is a creature you
control (normally the Army); it also hits each opponent, so it doubles as a finisher.

## AI (Sauron's deck only, gated on the card)
- Cast a sweeper when it kills clearly more opposing value than own value (reuse the existing wipe valuation; grep
  `Tag::Wipe` and the "wipes that hit nothing of the opponents' are not cast" rule), and at instant speed only where
  the card allows.
- Chandra's Ignition: pick the creature with the most power; also cast it when the damage to each opponent is lethal
  for at least one of them or wins the game, even if the board part is small.
- Inferno Titan: the 3 damage goes to the best opposing creatures/planeswalkers (or a player when lethal).

## Smoke run
Each package once: `./target/release/commander-sim --deck sauron --pool t3 --profile loose --games 96 --ai lookahead
--jobs 6 --analyze --swap "Toxic Deluge=><card 1>" --swap "Blasphemous Act=><card 2>"`. In the report: how often
each new card was cast when drawn (the card report shows it), so the lead knows the AI uses them.
