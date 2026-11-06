# Handoff: doomsday (updated as the work goes)

Branch `doomsday` (worktree `.claude/worktrees/doomsday`). Brief: `rust/agents/doomsday.md`.

Done (committed): `impls/doomsday.rs` (card resolve, exact pile search `solve`/`dfs`, `build`, `run_pile`, end-step and
main-phase cast `doomsday_options`, catalogue combo "Doomsday + Thassa's Oracle", Teferi +1 `Player::sorc_flash`);
person paths: Ponder, Brainstorm (`put_back` now returns Res), Gitaxian Probe, Doomsday; pile hooks in `topdeck.rs`,
`ai/topdeck.rs` (scry), `partials.rs` (Probe); `data/plans/thrasios.toml` Doomsday section; overrides for Doomsday and
Teferi; cards.json exported; `cardgen audit --mine` shows 0 not Full.
Cherry-picked 810a2370 (probe list) from deck-rewire.

To do: tests/doomsday.rs passing; take "thrasios: Doomsday" and "thrasios: Ponder" off TO_FIX in
tests/practice_choice_probe.rs; run the listed test files; `cargo build --release --all-targets` warning-free;
FINDINGS.md / decklist wording; final report in the reply.
