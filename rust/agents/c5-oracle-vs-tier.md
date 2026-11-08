# Brief: C5, the mulligan oracle against real opponents

JD approved this on 2026-11-07. Branch `c5-oracle-vs-tier`, cut from origin/main (0177d217 or later).

## Goal
`commander-sim --mulligan-oracle` (merged in #304, `sim-cli/src/mull_oracle.rs` + `sim-core/src/mull_oracle.rs`)
judges a hand by goldfish kill turn. Its first run (`audit/realism/c4-2026-11-07/REPORT.md`) showed two limits:
- it ignores interaction;
- the fair decks almost never kill within 10 goldfish turns.

Add a **real-game arm**: KEEP and MULLIGAN each play real pod games against the deck's own opponent tiers, and the hand
is judged by **win rate**.

## What to build
`commander-sim --mulligan-oracle KEYS --vs-tier [--pool tN|all] [--hands N] [--playouts M] [--seed S] [--ai ...]`
- **KEYS:** JD's deck keys only, with `--vs-tier`.
- **Tiers:** only the deck's own tiers from `data/my_decks.toml` (`poolmode::deck_tiers` / `select_tiers`).
  `--pool all` is the default and means all of them. A tier outside them is refused, as `--deck X --pool tN` is.
  `--any-tier` is not supported here.
- **Profile:** loose only (the CLI default).
- **Hands:** for each of N hands (default 100), deal the opening seven and, when the rule mulligans it, the second seven,
  exactly as the goldfish oracle does now (`opening_tries`).
- **Playouts:** M per arm (default 40), each a real pod game against three opponents drawn from the tier, the way
  pool mode seats them (see `poolmode.rs`; reuse its seating, don't copy it).
  - KEEP: the deck keeps the given hand, its library reshuffled.
  - MULLIGAN: the normal rule from the next draw.
  - Opponents mulligan and play normally.
  - Both arms of one playout share the opponents, their libraries and the play seed (paired).
- **Score:** a win is 1, anything else 0. Report the KEEP and MULLIGAN win rates. The verdict is keep or mulligan when the
  paired difference beats 2 standard errors, otherwise unclear.
- **Report:** the same layout as the goldfish oracle (agreement tables, by lands, the worst disagreements with card
  names, turns or win-points lost per hand), one block per tier, plus a combined line over the deck's tiers labelled
  e.g. "over t1-t3".
  - Also report, for the hands kept: the win rate split by land count and by fast-mana count. C6 will fit rules to it.
  - Write a machine-readable CSV next to the report (`--out FILE`): one row per hand × tier with the seed, try, card
    names, lands, the effective mana, tutors, payoffs, the rule's call, both win rates and the verdict. C6 will read it.
- **AI:** adaptive by default for speed, with `--ai lookahead` available. Add `--recheck K`, which reruns K random hands
  with the look-ahead AI and prints how often the verdicts agree.

## Rules
- Follow `rust/agents/RULES.md` where it still applies. Ignore its Python, difftest and `rust-port` parts. **No Python.**
- Memory cap on every cargo/test/binary command: `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 <cmd>`.
- **Trace only:** no game's behaviour changes. A plain `--deck X --pool tN` run and `--goldfish` must give identical
  output before and after; add a test for the pool run.
- Another agent is editing at the same time: it moves the goldfish playout/scoring code from `sim-cli/src/mull_oracle.rs`
  into `sim-core/src/mull_oracle.rs` for practice mode. Put your new code in new functions or a new file
  (`sim-cli/src/mull_oracle_tier.rs` is fine) and keep edits to the existing oracle functions minimal, so the merge
  stays simple.
- Tests: the tier refusal; KEEP and MULLIGAN pairing (same opponents and seeds); a small run is deterministic; the CSV
  has the expected columns.
- **No measurement runs.** Smoke-run only (N≤5, M≤4, one deck). Time it, and give an estimate for N=100, M=40 over each
  of JD's six decks' own tiers. The coordinator runs the real ones.
- Before finishing: `cargo test --release` passes and `cargo build --release --all-targets` has no warnings.
- Commit (no Co-Authored-By line, don't push). Put the full report in `rust/agents/report-c5-oracle-vs-tier.md`. Your
  final reply is at most 15 lines: the commit hash, the worktree path, the exact command for the real run, the time
  estimate per deck, and anything left undone.
