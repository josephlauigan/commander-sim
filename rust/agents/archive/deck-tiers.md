# Task: deck-tiers — each of JD's decks simulates only against its matched power tiers

Read `rust/agents/OVERNIGHT-RULES.md` first (hard rules, checks, keeping context small). Base: `origin/main`. Your
worktree's branch: `deck-tiers`. Use `cargo -j 12`, `--jobs 12` (check /home/joseph/projects/mtg_sims/cpu-ledger.md).
Never use `git stash`. Wrap builds/tests/binaries in `nice -n 19 systemd-run --user --scope -q -p MemoryMax=16G -p
MemorySwapMax=0 ...`.

## The rule (JD, 2026-10-14)
Every simulation runs each of JD's decks only against the opponent tiers of roughly equal power:

| key | tiers |
|---|---|
| yshtola | t4, t5 |
| seph (Sephiroth / Atraxa reanimator) | t4, t5 |
| sheoldred | t3, t4 |
| sauron | t2, t3, t4 |
| kefka | t2, t3, t4 |
| sephfs (Sephiroth, Fabled SOLDIER) | t1, t2, t3 |
| veyran | t1, t2, t3 |
| cloud | t1, t2 |
| titania | t1, t2 |
| alela | t1 |
| galadriel | t1 |

## Build
1. **Data:** a `tiers = ["t4", "t5"]` line per deck in `data/my_decks.toml` (the values above); `cardgen export`
   carries it into `data/decks.json` (`mine[].tiers`); `cardgen validate` errors if one of JD's decks has no tiers or
   an unknown tier. Keep pool decks unchanged.
2. **commander-sim (`sim-cli`):**
   - `--deck KEY --pool all` (and swap runs, `--analyze`) runs only the deck's matched tiers, and the summary's
     "over all tiers" averages are over those tiers (label them, e.g. "over its tiers (t1-t3)").
   - `--deck KEY --pool tN` with tN outside the deck's tiers: refuse with a clear message naming the deck's tiers,
     unless a new explicit override flag `--any-tier` is given.
   - `--all-decks` / pool mode over JD's decks: each deck only in its own tiers. `--calibrate` (pool balance checks,
     no deck) is unchanged.
   - Find every place that loops over `TIERS` for one of JD's decks (`poolmode.rs`, `report.rs`, `swaptest`,
     `tools.rs`, `play.rs`, examples) and apply the rule; list them in the report.
   - `--help` text and examples updated.
3. **Tests:** unit tests for parsing, the refusal and `--any-tier`, the per-deck tier selection, the fixture
   (`tests/fixtures/my_decks_parsed.json`) and anything that counts decks or tiers.
4. **Rule in the repo:** add a short section to `commander-sim/CLAUDE.md` ("Opponent tiers": each deck sims only
   against its tiers in `data/my_decks.toml`; report averages over those tiers; `--any-tier` only when JD asks), and
   one line to `rust/agents/OVERNIGHT-RULES.md` so agents follow it.

## Checks
OVERNIGHT-RULES checks 1-3 (`http_play` / `test_replayed_history_is_marked` in practice are known timing flakes: rerun
alone if one fails); `commander-sim --deck sephfs --pool all --games 48 --ai lookahead --profile loose --brief` runs
t1-t3 only; `--deck yshtola --pool t1` is refused and runs with `--any-tier`; same-seed check: for one tier inside a
deck's range, results identical before vs after (sauron t3, cloud t1, seph t4, seed 7, 48 games). Report in
`rust/agents/report-deck-tiers.md`; final reply at most 15 lines (DONE/HANDOFF, model ID, commit, what changed, tests,
anything I must act on). Commit (no Co-Authored-By), never push.
