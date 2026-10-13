# Task: cloud-equip-cards — model four free-equip and backup-carrier cards for the Cloud deck

Read `rust/agents/LASTDECK-RULES.md` first: its hard rules, modeling method and checks apply, with these changes:
- Base branch is `cloud-midgar`; your worktree's branch `cloud-equip-cards` starts there.
- **No batch sims** (JD paused them): no `--games`, `--analyze` or `all_decks` runs. Unit tests and single-game traces
  only. Build and test under `nice -n 19` with `-j 4`.
- Another agent (cloud-ai-fix) is changing `impls/cloud.rs`, `ai/search.rs` and the equip code in `dsl/` in
  parallel. Put your card code in `impls/bulk_white.rs` (Cloud's white cards) and keep edits to `cloud.rs` and
  shared files to small, listed hooks. Don't touch `impls/elrond.rs` (renamed `blue_cards.rs`).

JD wants the mono-white Cloud, Midgar Mercenary equipment deck (key `cloud`) to equip for free and to have backup
creatures that can carry the equipment when Cloud is dead. Make each card **Full**, with at least one test per card,
from its Oracle text (`jq '.["card name lowercased"]' data/scryfall_cache.json`).

## Cards
- **Ardenn, Intrepid Archaeologist** ({2}{W} 2/2 legend, partner): at the beginning of combat on your turn, you may
  attach any number of Auras and Equipment you control to target permanent or player. AI: move every Equipment onto
  the best attacker (Cloud when he is out and can attack; otherwise the best carrier). Partner can be noted as unused
  (single commander).
- **Balan, Wandering Knight** ({2}{W}{W} 3/3 first strike legend): double strike while two or more Equipment are
  attached to it; {1}{W}: attach all Equipment you control to it. AI: use it when Cloud is not out (or can't attack)
  and Balan can attack with the stack, and keep Cloud first when he is out.
- **Armored Skyhunter** ({3}{W} 3/3 flying): whenever it attacks, look at the top six; you may put an Aura or
  Equipment card from among them onto the battlefield; if it is an Equipment, attach it to a creature you control;
  the rest on the bottom in a random order.
- **Kor Duelist** ({W} 1/1 Kor Soldier): double strike as long as it is equipped.

Reuse the existing attach/equip machinery (grep `attach`, `equip_cost`, Kor Outfitter's and Brass Squire's code in
`impls/cloud.rs`, Puresteel Paladin, Wrecking Ball Arm) rather than writing new machinery.

## Checking a card's audit status
`cardgen audit --mine` only looks at deck lists. Do it on a scratch copy: copy `data/`, `decklists/`, `tests/` and the
root `*.txt` files to a scratch dir, append `1 <name>` lines under `31 Plains` in its
`decklists/JD/cloud-midgar-equipment.md`, then `rust/target/release/cardgen --root <scratch> audit --mine | sed -n
'/== cloud/,$p'`. Never use `cardgen audit --md` in the worktree (it rewrites the deck doc). Don't change the real
deck list.

## Done when
`cargo test --release -j 4` passes (the practice `http_lan` save test fails on the base commit too: note it, don't fix
it), the build has no warnings, all four cards are Full in the scratch audit, and one single-game trace of the cloud
deck finishes. Report in at most 15 lines (line 1 DONE or HANDOFF, line 2 your model ID); full report in
`rust/agents/report-cloud-equip-cards.md`, committed.
