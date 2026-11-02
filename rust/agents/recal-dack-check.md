# Brief: why does Dack Fayden (Tier 6) win 14 points more with its hand AI off?

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/overnight-stack` on
branch `overnight-stack` (origin/main + #185 + #186 + card models 2 and 3). Run every command from there. Read
`CLAUDE.md` and `rust/agents/RULES.md` first: memory cap, no Python, loose profile.

## The finding

`audit/recal/pairs-2026-10-31.md` section 4: on `--realism --pool t6`, 2000 games, seeds 500000-501999, Dack's share
of decisive games is 38.4% with its hand AI off (the default: generic policy plus `data/plans/dack-fayden-white-stax.toml`)
and 24.5% with `AI_HAND=dack-fayden-white-stax` (its hand AI), -13.9 [-17.4, -10.4]. Sisay's hand AI on/off moved
nothing (+0.4). With the hand AI off, Tier 6's deck win spread is 20.3 points (target 0-8). The raw runs:
`/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/recal/runs/base-t6.txt` and
`ha-dack-t6.txt` (built before card models 3; rebuild from this branch, don't reuse that binary).

## Work

1. Decide which it is: (a) the generic policy plays Dack's list genuinely better (which cards, which lines, which
   turns); (b) a rules or card-model bug the generic policy reaches and the hand AI never did (something that
   shouldn't work or is free); (c) the hand AI was a handicap (a bug or a bad rule in it, e.g. the known "Dack
   Chalice" gap in `audit/phase4/hand-off-handoff.md`). Read Dack's hand AI and plan file, compare how Dack wins in
   both arms (`--analyze`, game logs of a sample of seeds, the per-card cast and win-line numbers), and check every
   card the generic policy leans on against its Oracle text in `data/scryfall_cache.json`.
2. If (b): write a failing unit test that shows the bug and a fix, with the full cargo suite passing. If (a) or (c):
   say so with the evidence; no code change.
3. Re-run both arms on this branch (2000 games, same seeds) after any fix and report the new delta.

## Sims

You are the only sim agent: `--jobs 22`, one sim at a time, each under
`systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0`, in the background; check rc=0 on every run you cite.
Another agent runs cargo at `-j 6` in a different worktree. Wait on a task's completion notice or an output file,
never `until ! pgrep -f ...`; don't leave wait loops behind. Keep to about 10 sim runs.

## Rules

- No deck list changes; no style values; no recalibration; don't change any switch default.
- Commit often (findings in `audit/recal/dack-check.md`), no Co-Authored-By line, never push. Scratch in
  `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/dack-check/`.
- Report in 15 lines or fewer, then the full report as text (the harness refuses report files from subagents): which
  of (a)/(b)/(c), the evidence, any fix and its test, the new numbers.
