# Task: elrond-plan — make the Elrond deck's AI play its plan

Read `rust/agents/LASTDECK-RULES.md` first (two agents share the machine this round: `-j 8`, `--jobs 8`). Deck:
`decklists/JD/elrond-ring-tempts.md` (key `elrond`). Every card is now Full; the plan is weak: the deck wins
**11.5%** against Tier 3 (`--games 96`, lookahead). Your files: `impls/elrond.rs` (the plan: `elrond_prio`,
`elrond_special`, `elrond_protect`, the Ring-bearer pick), `impls/bulk_blue.rs` (only its target and timing choices),
`tests/cards_elrond.rs`; shared files (the search AI in `ai/`) only for small changes gated to `key == "elrond"`. Read
`rust/agents/report-elrond-core.md` and `report-blue-spells.md` first.

## What the traces show (`commander-sim --deck elrond --pool t3 --profile loose --ai lookahead --trace 0`, `1`, `2`)
1. **The Ring almost never tempts:** 0, 0 and 1 tempts in three games. The deck plays one creature a turn. The plan
   is two creature entries per turn cycle: pair a cheap creature (Ornithopter, Cloudfin Raptor, Rescue, The Wasp)
   with another on the same turn; cast Coral Barrier and token spells (each makes two entries or a second one); hold
   a flash creature for an opponent's turn only when another entry is coming that same turn, or for the end step
   before your own turn; warp for the second entry. Count Ring tempts per game before and after.
2. **Elrond is cast 3-5 times a game:** he dies to everything. Protect him (counter the removal, bounce him in
   response) and don't recast him into an open removal-heavy board when a creature-pair turn is available.
3. The Ring-bearer: check the choice in traces (an evasive creature, not Gandalf) and that it attacks when the Ring's
   level makes it safe or lethal (level 4: each opponent loses 3).
4. Add a plan milestone `elrond` (first Ring tempt from Elrond) the way `sheoldred` does (`engine/zones.rs` milestone
   entry; `sim-cli/src/report.rs` `plan_turn` and the note).

## Measuring
Baseline first, then after each change: `--deck elrond --pool t1`, `t3`, `t5`, `--games 240 --ai lookahead --jobs 8`
(same default seeds, so runs are paired), plus `--analyze` on t3 for the plan turn. Keep a change only if it helps;
note every measurement in your report (a table: change, tempts per game, t1/t3/t5, overall).

## Done when
The LASTDECK-RULES checks pass, the milestone shows in the report, the Ring tempts in most games where Elrond lives
two turns, and the report holds the before/after table. Full report in `rust/agents/report-elrond-plan.md`.
