# Brief: recalibration, stage 1: the paired tests for the Phase 4 switches

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/overnight-stack` on
branch `overnight-stack`: origin/main plus the open PRs #185 (rule checks, `AI_RULES`), #186 (hand AI behind
`AI_HAND`, generic by default) and card models part 2 (PR to come). Run every command from there. Read `CLAUDE.md`
and `rust/agents/RULES.md` first: memory cap, no Python, loose profile, opponent tiers.

**Sims are allowed again** (JD, 2026-10-31 night: "start sims once there is no more non-sim work", then "full
recalibration"). You are the only sim agent: `--jobs 22` for sims, one sim at a time, each under
`systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0`, in the background; check rc=0 on every run you
cite. Another agent may run cargo builds at `-j 4` in a different worktree. **Waiting:** wait on the task's completion
notice or an output file, never `until ! pgrep -f ...`; keep a single queue script per batch, and don't leave wait
loops behind.

## The method

The half-tier paired method in `audit/phase4/values-handoff.md` (the halves are listed there): the shares in a tier
sum to a constant, so each tier's generic decks are split into fixed halves A and B and the switch goes on one half
only; 2000 games per arm, seeds 500000-501999, `--realism --pool tN`, look-ahead AI, loose. "Level" = the CI of the
half's summed-share delta includes 0, or the new arm is better. For one JD deck: `--deck KEY --pool tN` on each of its
own tiers, both arms on the same seeds; report the average over its tiers. All arms of one comparison use the same
binary and `--jobs`. The base arm of every comparison is the stack's default (no env set).

## Work, in this order (commit results and a handoff after each)

1. **Identity checks (300 games, t1-t5):** the stack binary with no env against origin/main's binary built from a
   clean `git archive` — these are *expected to differ* (#186 changes JD's decks' play; card models change cards), so
   instead check: with `AI_HAND=1 AI_RULES=0`, does the stack match origin/main byte for byte on the `--realism` pool
   runs t1-t5? If not, find which change moves pool decks (card models part 2's fizzle rule is expected to; say how
   much) and report it; don't fix behaviour.
2. **PR 5b (`AI_SEARCH_B`):** Stage B per `audit/phase4/search-design.md` "Part (b)" and the scratch
   `.../scratchpad/phase4-search-b/stageB.sh` (rebuild its binaries from this branch): each tier t1-t5, both halves,
   base vs `AI_SEARCH_B=<half keys>`. Then the three lean ablations on t2 half B (`AI_SEARCH_MIN=6`,
   `AI_SEARCH_OPP=1`, `AI_SEARCH_ATK_MIN=6`, each against PR 5's ISMCTS default), then the best on t1 half B.
3. **Rule checks (`AI_RULES`):** per `audit/phase4/rules-design.md` section 5: generic halves with
   `AI_RULES=1 AI_RULES_DECKS=<half>`, and JD's decks the guards reach most (urdragon, thanos, kefka, sheoldred, ghave)
   on their own tiers with `AI_RULES=1 AI_RULES_DECKS=<key>`.
4. **Hand AI (`AI_HAND`):** per `audit/phase4/hand-off-design.md`: for each of the 14 decks, `AI_HAND=<key>` (its hand
   AI on) against the default (off), on its own tiers (JD's decks) or by half-tier pair (brago t2, aurelia t3) or on
   t6 (sisay, dack: the Tier 6 pool, `--realism --pool t6`, read each deck's share).
5. **Results table** in `audit/recal/pairs-2026-10-31.md`: every comparison (arms, delta, CI, games/s, rc), and a
   recommended default for each switch: `AI_SEARCH_B` on where level or better; `AI_RULES` on where level or better;
   for each deck, hand AI off where level or better (its hand AI can then be deleted), else "keep, improve the plan
   file" with what the paired breakdown suggests (`--analyze` on the deck if needed).
6. **Don't change defaults or delete code**: the coordinator decides from your table.

## Rules

- No deck list changes; no style values; no recalibration fits (later stages).
- Commit often (results and handoff `audit/recal/handoff.md`), no Co-Authored-By line, never push. Scratch in
  `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/recal/`.
- Keep the handoff current so another agent can continue if you run out of time. Post short progress in the handoff
  after each step.
- Report in 15 lines or fewer, then the full report as text (the harness refuses report files from subagents): the
  identity result, each comparison's numbers, and the recommended defaults.
