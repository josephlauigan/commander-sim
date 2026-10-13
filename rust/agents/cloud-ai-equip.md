# Task: cloud-ai-equip — diagnose how the Cloud AI uses its equipment (diagnosis only, no fixes yet)

Read `rust/agents/LASTDECK-RULES.md` first for the hard rules (no Python, memory cap, worktree, commit, handoff,
short report). Changes for this task:
- Your worktree's branch `cloud-ai-equip` starts from `cloud-c3-list`, whose last commit puts a **test-only** Cloud
  list (the "C3" list) in `decklists/JD/cloud-midgar-equipment.md`. Never edit that list.
- **JD has stopped all simulation runs.** Don't run batch sims (no `--games`, no `--analyze`, no `all_decks`). You may
  run single-game traces (`commander-sim --deck cloud --pool t1|t3 --profile loose --ai lookahead --trace N`, any N)
  and unit tests. Build with `nice -n 19` and `-j 4`.
- **Diagnose only.** Don't change engine or AI code. You may add tests that demonstrate a problem (mark them
  `#[ignore]` if they fail) and you must write the report.

## The problem
The mono-white Cloud, Midgar Mercenary equipment deck (key `cloud`) wins 7% at Tier 1 with the C3 list. A 480-game
analysis of C3 shows:
- an equipped Cloud attacks in only 63% of games;
- cards drawn but left in hand in most games: Champion's Helm (cast 43% of the games it was drawn: {2} to cast, equip
  {1}, gives a legend hexproof), Solemn Simulacrum (37%), Shield of Kaldra (37%), Sword of Kaldra (47%), Court of
  Ardenvale (42%), Shardmage's Rescue (22%).

Traces show equipment being cast and then not equipped for several turns (Sword of Hearth and Home, Shield/Sword of
Kaldra, Wrecking Ball Arm), while the AI casts more creatures or equipment instead.

## Questions to answer (with trace evidence: round, mana available, the options and their scores)
1. Why are cast equipment left unattached? Is equipping scored too low against casting, is the mana planned badly
   (casting first, then no mana to equip), or is the equip option missing?
2. Why do Champion's Helm, Solemn Simulacrum and Court of Ardenvale stay in hand? Find their cast priority / search
   value (`impls/cloud.rs` cloud_prio, `ai/decks.rs`, `ai/search.rs` and `search::strength`, the cast utility for
   equipment).
3. Equip targets: when Cloud is dead or in the command zone, does the AI suit up another creature (Bronze Guardian,
   Exalted Angel, Jocasta, Armory Automaton...)? When Cloud comes back, does it move equipment onto him, and at what
   cost?
4. Cloud's tutor pick on entering: which equipment does it fetch, and is that the best pick for the board?
5. Anything else in the plan that wastes turns (attacking into blockers for 0, holding back, recasting Cloud late).

## Deliverable
`rust/agents/report-cloud-ai-equip.md`: each problem, the evidence (trace excerpts: seed/game number and round), the
code responsible (file:function), and a proposed fix ranked by expected impact. Commit it. Final reply: at most 15
lines (line 1 DONE or HANDOFF, line 2 your model ID, then the top problems and fixes).
