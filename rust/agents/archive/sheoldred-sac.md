# Task: sheoldred-sac — sacrifice outlets for a Sephiroth package in Sheoldred

Read `rust/agents/SAURON-RULES.md` first, with these changes: your branch was made from `cloud-midgar` (PR #60, the new
baseline), not `sauron-weakness`; **other sessions share the machine**: build and test with `nice -n 19` and `-j 4`,
smoke runs with `--jobs 4`. Don't change decklists (the lead tests with `--swap`).

JD wants to test a Sephiroth package in Sheoldred (mono-black): Sephiroth, Fabled SOLDIER (Full, `impls/sheoldred.rs`
`register_seph`), Phyrexian Altar, Carrion Feeder, Common Crook, Callous Inspector, Phyrexian Tower (for a Swamp), with
Carnage Altar and Blood Artist already in the list. The package only works if the AI actually sacrifices for value.

## 1. Phyrexian Tower (bug)
It has no code or override: the autotag makes it a land that taps for {B}{B} for free (`amt=2`, `c=B`). Oracle:
"{T}: Add {C}. {T}, Sacrifice a creature: Add {B}{B}." Model it properly (a colourless land whose second ability
sacrifices a creature for {B}{B}, used when the extra mana matters or the sacrifice is worth it on its own), Full, with
tests. Five pool decks run it (Teysa, Meren, Korvold, Tergrid, Yawgmoth): note in your report that their mana drops.

## 2. The AI sacrificing to free outlets (Phyrexian Altar, Carrion Feeder, Phyrexian Tower, Carnage Altar)
Find how sacrifice outlets are used today (grep `Tag::Sac`, Ashnod's Altar, Viscera Seer, Altar of Dementia, Carnage
Altar, the aristocrats pool decks' AI, `sac_in_response`). Make sure, for any deck holding them:
- a creature about to die anyway (removal, a wipe, combat) is sacrificed in response for value;
- spare fodder (tokens, Common Crook, Callous Inspector, Undercity Dire Rat-type bodies, worth less than about 2.2 by
  the existing `sac_worth`) is sacrificed when a death payoff is out (Sephiroth's drain, Blood Artist, Sephiroth's flip
  count: the fourth drain in a turn transforms him) or when the mana/card is needed now;
- never the commander, Sephiroth himself, or a creature worth more than the payoff.
Carnage Altar and Common Crook are compiled from Scryfall text without an audit verdict: check them and mark Full.

## Tests and smoke
Tests for Tower, each outlet, and the AI rules above (a test where Sephiroth flips from four sacrifices in one turn).
Smoke: `--deck sheoldred --pool t3 --profile loose --games 96 --ai lookahead --jobs 4 --analyze --swap
"Dark Deal=>Sephiroth, Fabled SOLDIER" --swap "Charcoal Diamond=>Phyrexian Altar" --swap "Stir Up Trouble=>Carrion
Feeder" --swap "Swamp=>Phyrexian Tower"`: report how often Sephiroth flipped and how many sacrifices per game (add a
cheap stat if none exists).
