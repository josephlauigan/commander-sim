# Task: kefka-breach — can the Underworld Breach package move to Kefka?

Read `rust/agents/SAURON-RULES.md` first. Sauron runs an Underworld Breach + Brain Freeze line fuelled by Dark Ritual
and Lotus Petal (see `decklists/JD/sauron-grixis-amass.md`, "Key lines"). It wins about half the games it is cast in
but is rarely cast. JD wants to know if the package works better in Kefka (`kefka`, `decklists/JD/kefka-grixis-discard.md`),
which already has an Underworld Breach: the test moves Brain Freeze, Dark Ritual and Lotus Petal into Kefka.

The line is Sauron-only today: `impls/mine.rs` `breach_line` returns early unless `key == "sauron"`, and the tutor
and hold logic in `ai/decks.rs` (`breach_candidates`, the "held for the Breach line" branch near line 236, the
sauron branch of the tutor picks) is keyed to Sauron too.

## What to do
1. Make the Breach line work for any deck that holds the pieces (Underworld Breach, a storm card such as Brain Freeze
   or Grapeshot, rituals), not only Sauron. Sauron's games must not change: with a fixed seed, `--deck sauron --pool t3
   --games 96 --seed 500000 --profile loose --ai adaptive --brief` gives the identical result before and after.
2. Kefka's AI: holds Brain Freeze / the rituals for the line when Breach is reachable (as Sauron's does), and its
   tutors and Breach pick the line when it's live. Find out what Kefka uses Underworld Breach for today (grep Kefka's
   module and `ai/decks.rs`) and keep that working when the line isn't available.
3. Check that Kefka's own draw and discard (wheels, discard outlets) don't throw the pieces away when they're needed.

## Tests
A test position for Kefka with Breach, Brain Freeze, Dark Ritual, Lotus Petal and a few graveyard cards that mills
out the table; a test that Kefka without the storm card still uses Breach the way it did before.

## Smoke run
`./target/release/commander-sim --deck kefka --pool t3 --profile loose --games 96 --ai lookahead --jobs 6 --analyze
--swap "<cut 1>=>Brain Freeze" --swap "<cut 2>=>Dark Ritual" --swap "<cut 3>=>Lotus Petal"`, using three of Kefka's
weakest cards from that analysis (the lead picks the real cuts). Report how often the line was attempted and won.
