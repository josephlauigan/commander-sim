# Task: comet-storm — model Comet Storm for Sauron

Read `rust/agents/SAURON-RULES.md` first, with these changes: you are the only agent; build and test with
`nice -n 19` and `-j 8`, smoke runs with `--jobs 8`. Your branch was made from `sauron-weakness`.

JD wants Comet Storm (cheap) in place of Torment of Hailfire (expensive) in the proposed Sauron list, if it plays as
well. The lead will test `--swap "Torment of Hailfire=>Comet Storm"` on the final list.

## Card (Full, from Oracle text, with tests)
Comet Storm {X}{R}{R} instant: "Multikicker {1}. Choose any target, then choose another target for each time this
spell was kicked. Comet Storm deals X damage to each of them." It is in the Scryfall cache (unparsed: Multikicker).
Grep how other X spells and kickers are modeled (Torment of Hailfire in `impls/threats.rs`, Exsanguinate in
`impls/yshtola.rs` `x_option`, kicker / multikicker elsewhere) and reuse what exists.

## AI (Sauron's deck, gated on the card; make it work for any deck that runs it if that's easy)
A finisher like Torment of Hailfire: spend the mana on X and kicks to hit as many opponents (or planeswalkers /
creatures worth killing) as useful. Cast it when it kills at least one opponent, or late when the damage to every
opponent is worth the card. It's an instant: if it fits the existing end-of-turn / response windows cheaply, allow it
at the end of the turn before yours; otherwise main phase is fine.

## Tests and smoke
Tests: damage to each chosen target, kicks counted and paid, X chosen by the AI when lethal. Smoke: `--deck sauron --pool
t3 --games 96 --ai lookahead --jobs 8 --analyze --swap "Torment of Hailfire=>Comet Storm"` is not possible on the
current list (no Torment); use `--swap "Brain Freeze=>Comet Storm"` and report how often it was cast when drawn.
