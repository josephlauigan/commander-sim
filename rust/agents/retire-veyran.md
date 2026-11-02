# Brief: retire the `veyran` deck

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/retire-veyran` on branch
`retire-veyran` (origin/main). Run every command from there. Read `CLAUDE.md` and `rust/agents/RULES.md` first:
memory cap, **no Python in any form** (the hook misses commands that start with `cd ... &&`; never run Python that way
either).

**A sim agent runs at the same time** (22 threads): use **`-j 6`** for cargo, in the background. **Run no
`commander-sim` and no game-playing example yourself**: cargo tests only. Wait on background work by its completion
notice or an output file, never `until ! pgrep -f ...`.

## Why

JD, 2026-11-01: "we should retire veyran". Its cards went to Kefka and Sauron in #195 (Veyran, Voice of Duality and the
magecraft package are in Kefka's list now); the `veyran` key stayed as a t1-t3 sim seat until retired. Its hand AI was
already deleted (#196). Precedent: seph's retirement in #182 (`git show` that PR's merge for what moved where).

## Work

1. Take `veyran` out of JD's decks: `data/my_decks.toml`, then regenerate `data/decks.json` and
   `tests/fixtures/my_decks_parsed.json` with `cardgen export` (never hand-edit generated files). Move its decklist to
   the retired folder the way seph's went (`decklists/.../retired/`), with a dated note at the top. Remove
   `data/plans/veyran.toml` and any veyran-only code left (key matches, `jd_deck`/`is_main` lists, `HAND_DELETED`,
   reports, the practice catalog, sim-cli deck lists) where nothing else needs it.
2. Tests: tests that seat veyran only as a table filler or a generic JD deck get another live deck of a fitting tier
   (cloud for t1-t3, or a pool deck); tests of a card's rules keep testing that card (seat Kefka where the card is in
   Kefka's list now). Delete only tests of veyran-only behaviour that no longer exists, and say which.
3. Docs: `CLAUDE.md`'s opponent-tier line (veyran out of band 4; cloud stays t1-t3), `README.md`, `decklists/CHANGELOG.md`
   (a dated entry), and any doc listing JD's decks. Fix the stale Galadriel tier in `CLAUDE.md` while there: it is
   t1-t2 per `data/my_decks.toml` (JD's data), not t1.
4. Keep the `audit/veyran/` folder (history); add a dated "Retired" line at the top of its FINDINGS.md.

## Checks

- `cargo test --release --no-fail-fast -j 6` passes; `cargo build --release --all-targets -j 6` 0 warnings;
  `cardgen export` leaves no diff after your regeneration; `cardgen validate` passes.
- `grep -rn '"veyran"'` over `rust/` and `data/` shows only what you kept on purpose (say what and why).

## Rules

- No other deck list changes. Stay out of `ai/search.rs`, `ai/ismcts.rs`, `ai/learned.rs`, `data/eval/`, `audit/recal/`.
- Format only new files, lines up to 120. Commit, no Co-Authored-By line, never push. Scratch in
  `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/retire-veyran/`.
- Report in 15 lines or fewer, then the full report as text (the harness refuses report files from subagents).
