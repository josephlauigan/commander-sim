# Task: practice-shell, the command line, the iPad shell and the cutover docs (phases 12-13)

Read `rust/agents/RULES.md`, then `documents/rust-practice-design.md` (sections 9, 11) and `ios-shell/README.md`.
**Waits for** practice-session and practice-server (and practice-cards for the full decks); runs last. Merge `rust-port` first and check
that `practice` builds and its HTTP tests pass.

## Files owned
`rust/crates/practice/src/{lib,main}.rs`, `ios-shell/*` (including `include/practice.h`, `build-rust.sh`, `App/`), `documents/practice-mode.md`,
`documents/architecture.md` practice parts, `rust/README.md`, `rust/crates/practice/README.md` (new).

## Do
- `main.rs` = `__main__.py`: `commander-practice [--text] [--port 8765] [--no-browser] [--lan] [--deck] [--tier] [--seed] [--seat] [--opponents]
  [--profile] [--ai] [--step] [--data DIR] [--static DIR]`; opens the browser unless `--no-browser`; prints LAN addresses as Python does.
- `lib.rs`: `practice_start(data_dir, port) -> u16` starts `server::start(Config{app: true, lan: false, saves: <data_dir>/saves})`;
  add `practice_stop()` and `practice_set_lan(bool)` (the hidden two-player option); delete the M6 proof page and its code.
- `ios-shell`: copy `play/static/` and card data into the app's writable folder on first launch (keep saves across updates), *Continue* after iOS
  closes the app via the autosave, the hidden two-player gesture of the Briefcase app, true-black palette (already in the page); build for the
  simulator with `ios-shell/run.sh` only if a Mac is available - otherwise state in the report that the Swift part is unbuilt.
- Docs: how to run, test and package practice mode in Rust; mark the Python `play/` as retired once the user approves phase 13 (do not delete it).

## Checks
RULES.md checks; `commander-practice --no-browser` serves the unchanged page and a full game works in the browser (describe what you tried; the manual
checklist of the scope doc, section 7 item 6, is written into `documents/practice-mode.md` for the user to run); the HTTP suite passes on it.

Commit message: "Rust port: commander-practice command line, iPad library entry points and practice docs"
