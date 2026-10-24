# Report: cardgen

Branch `worktree-agent-a81b136543e20d594`, final commit see `git log` (message prefix "Rust port: cardgen").

## Done
- One-time extraction (previous agent, `rust/tools/extract_card_data.py`) plus `rust/tools/extract_audit_data.py`
  (this run): `data/carddb.txt`, `data/card_overrides.toml`, `data/my_decks.toml`, `data/python_hooks.txt`,
  `data/audit.toml`. All human-editable; the Python is no longer needed to add a card or deck.
- New crate `rust/crates/cardgen` (lib + `cardgen` binary): `dsl_parse.rs` (port of cards/dsl_parse.py), `dsl.rs`
  (hints, build_cd), `autotag.rs` (previous agent), `cd.rs` (CD), `sources.rs` (sources.py + pool_cards.apply + the
  tag edits / keyword data the card modules ran), `cardinfo.rs` (phyrexian, land_etb_fx, enters_rule, land_types),
  `scryfall.rs` (Python's cache format; client with 100 ms spacing, batches of 75, fuzzy fallback; ureq),
  `decks.rs` (deck files, pool, structural/card problems = pools.py), `export.rs`, `update_deck.rs`, `audit.rs`.
  Commands: `cardgen export | audit | validate | deck`.
- Equivalence: `cardgen export` output is byte-identical to the Python exporter's `data/cards.json` (1,767 cards) and
  `data/decks.json` (32 decks). Test `tests/equivalence.rs` compares card by card, field by field, then bytes.
  `audit` (summary, --mine, --deck) and `validate` output is identical to pool_audit / pools --validate.
  `deck` stdout and the rewritten deck file are identical to update_deck.py on a pool deck and on a deck of yours
  (dry run, real run with --log, error list); only the final "Next:" hint line differs on purpose.
- Tests: `tests/workflow.rs` (add a hand-tagged card, add an override, missing card offline, deck update + fixture,
  audit) on a scratch copy; `deck_guard_fixture_is_current`. README: `rust/crates/cardgen/README.md`; `rust/README.md`
  updated.

## Checks
`cargo test --release` passes (0 failures); `cargo build --release --all-targets` has no warnings;
`all_decks adaptive 3` and `all_decks lookahead 1` finish (no panic). No difftest needed.

## Shared files touched
`rust/Cargo.toml` (member added, previous agent), `rust/README.md`. No sim-core change.
`cardgen audit --hooks` uses sim-core's public `impls::registry`, `CardDb::from_raw`, `hooks::Event::from_name`.
The first `cargo build` needs the network to fetch `ureq`/`chrono`/`toml`/`fancy-regex` etc. (Cargo.lock updated).

## Python behaviour ported as is / bugs found
- `pool_audit --md` crashes in Python (`__import__("pools")` should be `commander_sim.pools`); Rust writes the table
  with the same intent (commander short name).
- `dsl_parse.CTX['line']` is a module global that keeps its value between cards (loyalty / activated / modal parsing
  uses the previous card's line for the "on top" search check). Ported as a thread-local that also persists, so the
  result can depend on the order cards are compiled (sorted by name here; the export is identical to Python's today).
- update_deck's Game Changers regex `\s*(.*)$` with `re.M` can swallow the newline after an empty list; ported as is.
- `data/cards_dsl.json` (Python's hand-written ability file, never present in the repo) is not read; the same job is
  done by `card_overrides.toml` (types, cost, tags, dsl). `data/cards_dsl.example.json` left in place (Python docs
  point at it).

## Notes for the merge
`data/audit.toml` is new; `data/*.json` are unchanged (byte-identical to the Python export). `cardgen deck` writes
`tests/fixtures/my_decks_parsed.json` like update_deck.py did.
