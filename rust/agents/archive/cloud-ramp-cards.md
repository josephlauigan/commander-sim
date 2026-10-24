# Task: cloud-ramp-cards — model the Cloud deck's ramp and equipment candidates

Read `rust/agents/LASTDECK-RULES.md` first: its hard rules, modeling method and checks apply, with these changes:
- Base branch is `cloud-midgar` (not elrond-cloud-decks); your worktree's branch `cloud-ramp-cards` starts there.
- One other agent (retire-elrond) shares the machine with other sessions: every build, test and run under
  `nice -n 19`, cargo with `-j 4`, sims with `--jobs 8`.
- The mechanic owners table no longer applies: you may extend the equipment machinery the cloud deck uses
  (`impls/cloud.rs`, `impls/bulk_white.rs`), with small, listed changes. The Elrond deck is being removed by another
  agent: don't touch `impls/elrond.rs`, and don't use the `elrond` deck key in tests.

JD wants to shore up the mana of the mono-white Cloud, Midgar Mercenary equipment deck (key `cloud`,
`decklists/JD/cloud-midgar-equipment.md`). These cheap candidates are not fully modeled yet (`Partial` or
`Approximate`). Make each one **Full**, with a test per card, so they can be screened in paired swap tests.

## Cards
- **Catch-up ramp (opponents with more lands):** Keeper of the Accord, Loyal Warhound, Gift of Estates,
  Archaeomancer's Map, Oreskos Explorer. Model the land-count conditions exactly (Warhound and Oreskos count
  opponents' lands at resolution, Keeper and the Map check each opponent on their trigger). Remove the old regex-tag
  approximations they have now so they don't count twice.
- **Mana rocks:** Thran Dynamo ({T}: {C}{C}{C}), Prismatic Lens ({T}: {C}; {1},{T}: one mana of any color). Both
  should be plain rocks the mana engine understands like Arcane Signet or Hedron Archive.
- **Equipment helpers:** Kor Outfitter (ETB: you may attach target Equipment you control to target creature you
  control: the AI attaches the best unattached or moved Equipment onto Cloud when Cloud is out); Auriok Steelshaper
  (equip costs you pay cost {1} less; while it is equipped, your Soldiers and Knights get +1/+1: Cloud is a Soldier);
  Brass Squire ({T}: attach target Equipment you control to target creature you control, sorcery speed is not
  required: the AI uses it like a free equip onto Cloud). Reuse the existing equip-cost and free-attach code (grep
  `equip_cost`, `attach`, Puresteel Paladin, Cid, Helitrooper, Armory Automaton in `impls/cloud.rs`/`bulk_white.rs`).

## Checking a card's audit status
`cardgen audit --mine` only looks at deck lists. Do it on a scratch copy so no deck list changes in git: copy
`data/`, `decklists/`, `tests/` and the root `*.txt` files to a scratch dir, append your cards (`1 <name>` lines)
under `31 Plains` in its `decklists/JD/cloud-midgar-equipment.md`, then
`rust/target/release/cardgen --root <scratch> audit --mine | sed -n '/== cloud/,$p'`. Every card above must show as
Full there. Don't change the real deck list.

## Done when
The LASTDECK-RULES checks pass (use the cloud smoke run), every card above is Full in the scratch audit, and
`--deck cloud --pool t3 --profile loose --games 240 --seed 500000 --ai lookahead --jobs 8` gives exactly the same
result before and after your change (none of these cards is in the list yet, so nothing may move). Report in at most
15 lines (line 1 DONE or HANDOFF, line 2 your model ID); full report in `rust/agents/report-cloud-ramp-cards.md`.
