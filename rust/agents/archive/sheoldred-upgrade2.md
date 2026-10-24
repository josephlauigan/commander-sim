# Task: sheoldred-upgrade2 — model the second, bigger Sheoldred upgrade package so it can be swap-tested

Read `rust/agents/OVERNIGHT-RULES.md` first: its hard rules (never run Python, memory cap, edit only your files,
commit often, handoff, 15-line report) apply here. Where it says "Kefka/Zada", read "Sheoldred".
The machine is yours: `cargo -j 16`, `--jobs 16` for smoke runs.

Base branch: `sheoldred-deck`. Deck: `decklists/JD/sheoldred-mono-black.md` (key `sheoldred`), fully modeled; module
`impls/sheoldred.rs` (the first package is `register_upgrade`: Seizan, Underworld Dreams, Dark Deal, Painful Lesson),
tests `tests/cards_sheoldred.rs`. Read `rust/agents/report-sheoldred-upgrade.md` for how the first package was added
(Scryfall cache records, overrides, `cardgen export --offline`; and the gotcha: an override `tags = ""` wipes the
autotagged pow/tgh/leg).

The package cards are **not in the deck list** (JD may buy them); they are tested with `--swap`. Don't change the list.

## Cards to model (whole Oracle text, a test each, verdict Full)
Not in the card data yet: **Howling Mine**, **Temple Bell**, **Fate Unraveler**, **Peer into the Abyss**, **Geier Reach
Sanitarium** (a land: {T}: {C}; {2},{T}: each player draws a card, then discards a card), **Ambition's Cost**,
**Syphon Mind**.

Already in the data, but **prove each works with a cast/play test in a sheoldred game** (verdicts have been stale
before: 18 Sheoldred cards said Full and did nothing): Mind Stone, Commander's Sphere, Charcoal Diamond, Castle
Locthwain, Phyrexian Arena, Read the Bones, Exsanguinate, Gray Merchant of Asphodel (devotion to black counts the
mana symbols of your black permanents), Swiftfoot Boots. Fix any that don't.

Every opponent draw these cards cause must fire Sheoldred, Underworld Dreams and Fate Unraveler (tests).

## AI plan (gated to `key == "sheoldred"`)
- **Howling Mine:** cast while Sheoldred or another draw punisher (Underworld Dreams, Fate Unraveler) is out;
  otherwise hold it.
- **Temple Bell:** cast any time; tap it (at the end of an opponent's turn, or in your main phase when it kills) only
  while a draw punisher is out.
- **Geier Reach Sanitarium:** activate under the same rule, with spare mana at an opponent's end step.
- **Peer into the Abyss:** with Sheoldred out, target the opponent it kills (they draw half their library: 2 life each,
  plus half their life); without her, target yourself only when your library is big and life is high, else hold.
- **Swiftfoot Boots:** on Sheoldred when she has no Champion's Helm; Helm first if both are in hand.
- **Exsanguinate:** as a finisher (X that kills, or late with spare mana). Gray Merchant: normal cast.
- **Syphon Mind, Ambition's Cost, Read the Bones, Phyrexian Arena:** normal card draw.

## Check the swap works
These swaps must run as one block (the first package plus the second):
```
--swap "Ribskiff=>Seizan, Perverter of Truth" --swap "Locust Spray=>Painful Lesson"
--swap "Gravblade Heavy=>Underworld Dreams" --swap "Multiply by Zero=>Dark Deal"
--swap "Mabel, Bitter Recluse=>Howling Mine" --swap "Moseo, Vein's New Dean=>Temple Bell"
--swap "Burglar Rat=>Fate Unraveler" --swap "Fathom Fleet Cutthroat=>Peer into the Abyss"
--swap "Canyon Crawler=>Mind Stone" --swap "Necrosquito=>Commander's Sphere"
--swap "Rampart Hunter=>Charcoal Diamond" --swap "Callous Inspector=>Phyrexian Arena"
--swap "Depressurize=>Read the Bones" --swap "End of the Hunt=>Ambition's Cost"
--swap "Tome of Legends=>Syphon Mind" --swap "Diary of Dreams=>Exsanguinate"
--swap "Dark Endurance=>Gray Merchant of Asphodel" --swap "Lash of the Balrog=>Swiftfoot Boots"
--swap "Swamp=>Geier Reach Sanitarium" --swap "Swamp=>Castle Locthwain"
```
Check that two `Swamp=>` swaps replace two Swamps (34 → 32); if `--swap` can't do that, say so and propose the
smallest fix. Smoke: `--deck sheoldred --pool t3 --profile loose --games 24 --ai lookahead --jobs 16 --brief` with
those swaps.

## Done when
The OVERNIGHT-RULES checks all pass (full `cargo test --release`, no warnings, `all_decks adaptive 3` and
`lookahead 1`, audit still `sheoldred: 0 not Full`, a 48-game smoke run with `--deck sheoldred`), plus the swap smoke
above. Report as those rules say: line 1 DONE or HANDOFF, line 2 your model ID. Full report in
`rust/agents/report-sheoldred-upgrade2.md`.
