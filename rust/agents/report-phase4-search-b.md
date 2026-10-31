# Report: Phase 4 PR 5b, blocks, protection and wipe responses by search

Brief: `rust/agents/phase4-search-b.md`. Design: `audit/phase4/search-design.md` ("Part (b)"). Handoff:
`audit/phase4/search-b-handoff.md`. JD (2026-10-31): no sims of any kind until the recalibration, so this PR ships
**off by default** and its paired tests are listed for the recalibration.

## What it does

- Blocks, protection (responding to removal aimed at a generic seat's permanent) and wipe responses can be decided by
  PR 5's ISMCTS root search, behind `AI_SEARCH_B` (unset or `0` = PR 5's heuristics; `1` = every generic-policy deck
  on ISMCTS; a key list = those decks).
  - Blocks are searched when the unblocked attack reaches half the defender's life; options: the heuristic's blocks,
    no blocks, kill-and-survive only, chump until not lethal, every attacker blocked.
  - Protection: let it resolve, or up to three protectors that would save the target.
  - Wipes: no answer, Golgari Charm's regeneration, or up to three board-wide protectors.
- The engine code was split at each decision point so a copy can play on from mid-step: `combat::resolve_blocks`,
  `removal::apply_removal_rest`, `removal::wipe_victim` / `wipe_finish`, `search::play_on_resume`.
- Ablation knobs `AI_SEARCH_OPP` and `AI_SEARCH_ATK_MIN`, both defaulting to PR 5's behaviour.

## Results (all before the 10/31 stop)

- **Survey** (300 games a tier): blocks come up 4.5-8 times a game; protection and wipe responses with a real
  option are rare (protection 1.3 a game on t2, at most 0.25 elsewhere).
- **Stage A:** with `AI_SEARCH_B` unset, output equalled PR 5's head byte for byte on t1, t2, t4 and t5 (300 games);
  `AI_SEARCH_B=1` twice on t1 gave the same output. The two knobs came after; checked by code and a unit test.
- **Cost:** on t1 with every seat on part (b), +3.7% playout steps a game.
- **The lean (PR 5's 7 of 10 pairs down):** not fixed. Slower widening, t2 half B: -1.1 [-5.0, +2.8] against PR 5's
  ISMCTS; not kept. Meren's decision logs: ISMCTS departs from the policy less in the main phase (14.5% vs 19.6%)
  with about 30% fewer steps a decision, which points at lost main-phase look-ahead; `AI_SEARCH_MIN=6` would test it.
- **Not run:** Stage B, Stage C, the other three ablations; listed at the end of the design doc.

## Coordinator's checks (2026-10-31)

- Read the diff: each part (b) entry (`choose_blocks`, `choose_protect`, `choose_wipe`) returns at once unless
  `AI_SEARCH_B` is on for the deciding seat; the survey hook only writes `SearchCost` counters, which only the reports
  read; the engine splits keep the original order of calls.
- Merged origin/main (through #182) into the branch: clean. `cargo build --release --all-targets` 0 warnings;
  `cargo test --release --no-fail-fast` 3707 passed, 0 failed. Test-merged with #183 (card models): no conflicts.

## Open (for the recalibration)

1. Stage B on each tier, PR 5's head against `AI_SEARCH_B=<half keys>` (`stageB.sh` in the scratch dir).
2. The lean ablations on t2 half B (`AI_SEARCH_MIN=6`, `AI_SEARCH_OPP=1`, `AI_SEARCH_ATK_MIN=6`), then the best on
   t1 half B.
3. Stage C, then the byte-for-byte check of the head against the chosen arm on each tier, and `all_decks`.
