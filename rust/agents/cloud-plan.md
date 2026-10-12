# Task: cloud-plan — make the Cloud deck's AI play its plan

Read `rust/agents/LASTDECK-RULES.md` first (two agents share the machine this round: `-j 8`, `--jobs 8`). Deck:
`decklists/JD/cloud-midgar-equipment.md` (key `cloud`). Every card is now Full; the plan is what's broken: the deck
wins **0-1%** against Tier 3 (`--games 96`, lookahead and adaptive alike). Your files: `impls/cloud.rs` (the plan:
`cloud_prio`, `cloud_special`, `cloud_protect`, `attack_filter`, the equip choice), `impls/bulk_white.rs` (only its
target choices), `tests/cards_cloud.rs`; shared files (the search AI in `ai/`, `engine/zones.rs`) only for small
changes gated to `key == "cloud"`. Read `rust/agents/report-cloud-core.md` and `report-white-rest.md` first.

## What the traces show (`commander-sim --deck cloud --pool t3 --profile loose --ai lookahead --trace 0`)
1. **The look-ahead search doesn't cast Cloud.** On turns 2-6 his priority says "Cloud, Midgar Mercenary 99%" and the
   search picks Tome of Legends, Memory Trap, Citizen's Arrest instead; he first comes down on turn 7. Find why the
   search values the cast so low (rollouts where he dies at once? his search not seen in rollouts? the commander cast
   priced wrong?) and fix it, gated to cloud. With the adaptive AI he comes down on turn 2 and dies to the first
   removal; decide with measurements whether to wait for protection mana (Shield of Kaldra, Bastion Protector,
   Shardmage's Rescue up) after his first death.
2. **Equipment is cast but not equipped.** Sword of Kaldra (turn 8), Mask of Memory, Colossus Hammer sit unattached
   while Cloud is out; Medic's Kitesail ends up on a token. Each turn, spend mana to get the best Equipment onto Cloud
   (cast + equip in the same turn when it fits; Puresteel's {0}; Wrecking Ball Arm's {3} on a legend; Veteran's
   Powerblade's {W} on a Soldier; Beatrix and Armory Automaton). An equipped Cloud doubles triggers and attacks.
3. **Stormfront Riders is recast every turn** and bounces Cloud each time; the bounced commander goes to the command
   zone, so his tax grows. The rules let the owner leave a commander in hand when it would go there (rule 903.9b): for
   cloud keep him in hand (no tax), and bounce him only when the new search is worth what falls off.
4. Mana: games with 3 lands on turn 6. Check the mulligan rule keeps 3+ lands (35 lands, few rocks).
5. Add a plan milestone `cloud` (first attack by an equipped Cloud) the way `sheoldred` does
   (`engine/zones.rs` milestone entry; `sim-cli/src/report.rs` `plan_turn` and the note), so the report shows when
   the plan comes online.

## Measuring
Baseline first, then after each change: `--deck cloud --pool t1`, `t3`, `t5`, `--games 240 --ai lookahead --jobs 8`
(same default seeds, so runs are paired), plus `--analyze` on t3 for the plan turn. Keep a change only if it helps;
note every measurement in your report (a table: change, t1/t3/t5, overall).

## Done when
The LASTDECK-RULES checks pass, the milestone shows in the report, Cloud is cast by turn 3 in most games where he can
be, and the report holds the before/after table. Full report in `rust/agents/report-cloud-plan.md`.
