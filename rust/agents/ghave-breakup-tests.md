# Brief: plan files and tests after the Ghave breakup

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/ghave-breakup` on branch
`ghave-breakup`. Run every command from there. Read `CLAUDE.md` and `rust/agents/RULES.md` first: memory cap
(`systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 ...`), **no Python in any form**, cargo with
`-j 8`, in the background. **Run no `commander-sim` and no game-playing example**: cargo tests only.

## Why

JD retired the Ghave deck on 2026-11-02 and moved its cards into eight decks with `cardgen deck` (commit 6b36441f; see
the newest entries in `decklists/CHANGELOG.md` for every card in and out). The lists and `data/decks.json` are final:
**don't change any deck list**. Another session edits `decklists/JD/*.md` in this worktree at the same time: don't
touch those files, and **don't commit** (the coordinator commits).

## Work

1. **Plan files** (`data/plans/<deck>.toml`) for cloud, kefka, sauron, sheoldred, thrasios, urdragon, karn and yshtola
   still name cards that left those decks. `cargo test --release -p sim-core --test plans` currently reports some of
   them; also grep each plan file for every card that left (the CHANGELOG entries list them) and fix them all. Drop a
   card from a list where others remain; remove a rule whose only card left. Where a new card plainly takes the same
   role in the same rule (e.g. a mana-rock list, a protection hold, an equipment list), you may add it, but only if
   the schema clearly supports it; say what you added. Keep comments accurate.
2. **Three tests** now fail because the lists changed:
   - `crates/practice/tests/session_partners.rs:135` "no game cast Thrasios" (Thrasios's list changed).
   - `crates/sim-core/tests/audit_ysh.rs:80` `!g.player(P0).library.contains(&c)` (Y'shtola's list changed).
   - `crates/sim-core/tests/search_hold.rs:83` "the old search spends the mana": expected "stop (hold mana)", got
     "Sheoldred, the Apocalypse" (Y'shtola now runs Sheoldred, the Apocalypse).
   Find why each depends on the list (a seed, a card drawn, a card the AI now prefers) and make it list-independent
   the way earlier fixes did (e.g. commit dfedc7ba "two seed-dependent tests made list-independent": a fixed hand or
   library, an explicit card, more seeds). Keep each test testing what it was written to test. Don't weaken an
   assertion to make it pass.

## Checks

- `cargo test --release --no-fail-fast -j 8` passes in full (run it under the memory cap, in the background).
- `cargo build --release --all-targets -j 8`: 0 warnings.

## Rules

- Change only `data/plans/*.toml` and test files (plus a test helper if one truly needs it). Stay out of `ai/search.rs`,
  `ai/ismcts.rs`, `ai/learned.rs`, `data/eval/`, `audit/recal/`, `decklists/`.
- Lines up to 120. No commit. Scratch in
  `/tmp/claude-1000/-home-joseph-projects-mtg-sims-commander-sim/795ca8bc-d596-4718-8849-64dd08cfd4bf/scratchpad/agent/`.
- Report in 15 lines or fewer, then the full report as text: each plan-file change (card, rule, what you did), each
  test fix and why it was list-dependent, and the final test totals.
