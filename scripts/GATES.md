# Gate tools

Scripts that judge a PR by machine, the same way every time. None of them plays a game except `trace-record.sh`, which only
the coordinator runs.

All cargo calls go through `flock /tmp/cargo-heavy.lock systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 env
CARGO_TARGET_DIR=<tree>/rust/target cargo ...`, run from `<tree>/rust`. Every tree uses its own target directory; the main
checkout is refused (`gates-lib/common.sh`). `trace-record.sh` does not take the lock itself: wrap it the same way.

## stage-gate.sh: the L0 and L1 gates

    scripts/stage-gate.sh <worktree> --base <commit> --pr <name> --base-tree <path> \
        [--declared-deletions FILE] [--review FILE] [--only GATE,...] [--gates-dir DIR] [--dry-run]

Writes one `PASS`/`FAIL` line per gate, with an evidence path, to `/tmp/handoff/gates/<pr>.md` and ends with
`RESULT: PASS | FAIL | INCOMPLETE`. Exit 0 only if every gate run in the call passed. `--base-tree` is a checkout of `--base`
(the coordinator keeps one); the base's test lists are cached per commit in the gates dir. `--only` re-runs some gates and keeps
the other lines. Commit before running: every gate except `review` refuses a tree with uncommitted changes (the gates judge the working tree and record the head), and the export gate restores the tree afterwards. A line kept from an earlier run counts only when it was judged on the same head and base; after a new commit the other gates show as not run (INCOMPLETE) until they are run again.

| gate | passes when |
|---|---|
| suite | `cargo test --release --workspace --no-fail-fast`: cargo exit 0 and 0 failed |
| ignored | the branch's `-- --list --ignored` names are a subset of the base's; the `#[ignore` count in `rust/crates` is not above the base's |
| names | the `-- --list` names deleted from the base equal the `--declared-deletions` list exactly (`binary::name` or the bare name, one per line, `#` comments); no declaration, no deletions |
| diffcheck | `git diff --check <base>` is clean |
| fmt | `rustfmt --check --edition 2024` on every touched or new `.rs` file |
| audit-mine | every deck of `cardgen audit --mine` has 0 not Full |
| audit-paths | the four `double:` rows of `cardgen audit --paths` are 0 (declared `paths_ok` splits are fine) |
| validate | `cardgen validate` says `all decks passed` |
| export | `cardgen export --offline` leaves `git status --porcelain` clean (otherwise FAIL and the tree is restored) |
| allowlists | the frozen allow-lists (below) hold no new entry and no changed reason |
| newconsts | no new `const`/`static` named like `FIX\|ALARM\|EXCEPT\|GAP\|SKIP\|EXEMPT\|ALLOW\|QUIET\|NO_` in `rust/crates` |
| review | the first line of `--review FILE` is exactly `VERDICT: commit` and a line `Reviewed head: <hash>` (8 or more hex digits) is a prefix of the head, so a verdict for an earlier commit does not pass a later one |

Frozen allow-lists (`gates-lib/gatecheck.py`, table `FROZEN`): `TO_FIX` and `FALSE_ALARMS` in the four `practice_*_probe.rs`
files (choice, trigger, ability, opponent), `EXCEPTIONS` in `practice_rings_probe.rs`, `KNOWN_GAPS` in `spellbook.rs`,
`NO_CHOICE` and `QUIET` in `model_gaps_a4.rs`, all under `rust/crates/sim-core/tests/`. The key of an entry is its first string
literal (`KNOWN_GAPS`: the first two, since a deck key repeats); the branch's keys must be a subset of the base's, a changed
reason fails; a frozen const the base has and the branch cannot find (renamed, or `const` changed to `static`) fails too. A new entry is a caveat to report, never a fix. A "new" const in `newconsts` is a `(file, name)` the base lacks, so
moving one to another file also trips it.

`--dry-run` prints the cargo commands instead of running them and writes `<pr>.dry-run.md`. Test seam: `GATES_LIST_DIR=DIR`
reads `base.list`, `branch.list`, `base.ignored` and `branch.ignored` from DIR instead of cargo.

## red-green.sh: proof that a fix row is a fix

    scripts/red-green.sh --tree <scratch worktree> --red <commit> --green <commit> --test <name> --ledger <file> \
        [--cargo-args "-p sim-core --test cards_x"] [--dry-run]
    scripts/red-green.sh --ledger <file> --test <name> --coverage

Checks out R in the clean scratch worktree and runs the test: it must FAIL by assertion or panic (a compile error is not red:
exit 3), then checks out G, where it must pass. A row `| test | R | G | red evidence | green evidence |` goes into the ledger
(`/tmp/handoff/gates/<pr>-ledger.md`); the full outputs sit beside it as `<ledger>.<test>.red.log` and `.green.log`. `--coverage`
writes the row `coverage, no red/green` for a test that covers rather than fixes. Exit 0 = proven, 1 = not red or not green,
3 = a compile error, 2 = bad usage. `--test` is matched against the end of the test path (`a::b::name` or `name`).

## trace-record.sh and trace-compare.py: L2, L3 and goldfish (coordinator only)

    flock /tmp/cargo-heavy.lock systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 \
        scripts/trace-record.sh --mode l2|l3|goldfish --tree <worktree> --out <new dir> [--seeds N] [--decks LIST] \
        [--cards 'A;B' | --cards-file FILE] [-P N]

The tree must be built (`rust/target/release/commander-sim`) and clean. The binary is copied into the out dir and its sha256
recorded; each game is `env -i PATH HOME COMMANDER_SIM_ROOT=<tree> <copy> [--deck D] --pool T --seed 500000 --trace k`, one process
per game. Deck-tiers come from the tree's `data/my_decks.toml`. Output: `records.jsonl`, `meta.json` (commit, hashes, timing),
`logs/` (l2 only). `l2`: 20 seeds x (each deck-tier + the within-tier pods t1..t6), full records. `l3`: 200 seeds per deck-tier, only
exit code, panic text, timeout flag, winner-present flag, last round and the `casts <card>` flag per card (no winner names, no logs).
`goldfish`: `--goldfish <deck> --games 500 --seed 500000` into `goldfish.txt` and `goldfish.json` (median, mean, kills, route shares).

    scripts/trace-compare.py <base dir> <branch dir> [--rewrite 'old -> new' ...] [--expect-missing deck/tier ...] \
        [--allow deck ...] [--changed-cards 'A;B'] [--explained 'A;B']

l2 classes per record: SAME (`sha_full` equal), DIAG (only `sha_play` equal: the `[X decides: ...]` lines changed), PLAY (same
result, different actions: first differing line and both lines), RESULT (winner, wintype or last round differ), ERROR (a game
exited non-zero), MISSING (base only; a failure unless declared with `--expect-missing deck/tier` or a bare deck), NEW (branch
only). `--rewrite` applies a regex to every base log line before hashing. Exit non-zero if a record of a deck not in `--allow` is
not SAME. For l3 recordings the comparer reports panics, timeouts and no-winner games per deck-tier against the base on the same
seeds, lists each seed that newly times out, and prints cast counts per card; it fails on any branch panic, a timeout or no-winner
count above the base's, and a `--changed-cards` card (`Card`, or `deck:Card` to look at that deck's seat only: use it for a card that is in more than one deck) that is never cast in the branch by a deck's own seat (or no longer cast where the base cast it); casts are matched by exact card name (`Twinflame` does not match `Twinflame Tyrant`), a recording made before own-seat counts existed is refused against a newer one, and an `ERROR` record (the branch game exited non-zero) is never allowed by `--allow`, while a base-only crash that the branch fixes is a `RESULT` record; `--explained` is matched against the full `--changed-cards` text, so a `deck:Card` is explained as `deck:Card`
unless named in `--explained`. It prints no win rates.

## Tests

    bash scripts/tests/run.sh

Plays no game and runs no cargo: `fakebin/` holds stand-ins for `cargo`, `flock` and `systemd-run` that print canned output from
`fixtures/`, and the recorder tests use a stub `commander-sim` that prints a made-up log. Covers every comparer class, the
allow-list parser, the ignored-name subset, deletion-list equality, the VERDICT line, the red/green outcomes and each gate's
failure path.
