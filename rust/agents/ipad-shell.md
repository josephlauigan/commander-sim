# Task: ipad-shell, the iPad app on the Rust engine (phase 12)

Read `rust/agents/RULES.md` (skip the `git merge rust-port` step: the port is merged, your worktree is on main),
then `ios-shell/README.md`, `documents/rust-practice-design.md` (its iPad section), and this brief. Your worktree:
`/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/ipad-rust`, branch `ipad-rust`.

The Rust iPad shell in `ios-shell/` is still the M6 proof: `practice_start` in `rust/crates/practice/src/lib.rs`
serves a test page that plays one game. Make it the real practice app, with everything the old Briefcase app
(`ios/`, Python: read it, never run it or edit it) does. This machine is Linux: the Swift part can't be built here.
JD builds it on a Mac with `ios-shell/run.sh`, so the README steps must be exact.

## CPU and memory (shared machine: hard limits)
- At most **4 threads**: every cargo command gets `-j 4` and runs under
  `nice -n 19 systemd-run --user --scope -q -p MemoryMax=8G -p MemorySwapMax=0 ...`. Test binaries: also
  `-- --test-threads=4`. Never run sims or `all_decks`; the RULES.md all_decks check is waived for this task.
- Build only the `practice` crate and what it needs: `cargo test --release -j 4 -p practice` (not the workspace).

## Files owned
`rust/crates/practice/src/lib.rs` (and a new module for the data copy if you like), `rust/crates/practice/Cargo.toml`
(crate-type already set? check), `ios-shell/*`, `rust/agents/{handoff,report}-ipad-shell.md`. Anything else: the
smallest change, listed in your report.

## Do
1. **lib.rs entry points** (C ABI, `#[unsafe(no_mangle)] extern "C"`), replacing the M6 proof page and its code:
   - `practice_start(bundle_dir, app_dir, port) -> u16`: `bundle_dir` is read-only (the app bundle's `Resources`),
     `app_dir` is writable (Application Support). Copy the bundled data into `app_dir/data` (step 2), load the
     engine from there, and start the real server like `main.rs` does:
     `server::start_with(Config { data, saves: app_dir/saves, static_dir: <bundle>/static, lan: false, app: true, port }, RealSessions { engine })`.
     Keep the `ServerHandle` in a static. Return the port, 0 on failure (log the error to stderr).
   - `practice_stop()` and `practice_set_lan(bool)` (the hidden two-player option). Find out first what `app: true`
     already hides in the hub and the page, and how the Briefcase app reached two-player; if `set_lan` must restart
     the server on 0.0.0.0, do that, and return the new port. If the option is fully hidden and nothing in the page
     can turn it on, say so in the report and keep `set_lan` minimal but working.
   - Keep the `start(...)` Rust function `main.rs` uses working, or update main.rs to match.
2. **First-launch data copy, in Rust** (port of `ios/src/commander_ipad/bootstrap.py::sync_data`): copy when the
   bundle's `bundle.json` stamp differs from `app_dir/data/bundle.json`; plain files are replaced; inside folders,
   files are copied only when missing, except the images `index.json`, which is merged; **saves are never touched**
   (they live outside data/ anyway). Write it so it is testable on Linux.
3. **Tests** (cargo, Linux): the copy (first launch, same stamp = no copy, new stamp replaces files but keeps
   downloaded images and merges the index, saves survive); `practice_start` on temp dirs serves `/` (the real
   index.html) and the API the page calls first; an autosaved game shows up for *Continue* after
   `practice_stop` + `practice_start` again (that is what happens when iOS closes the app); `set_lan`.
4. **ios-shell**:
   - `include/practice.h`: the new signatures, with comments.
   - `App/CommanderPracticeApp.swift`: pass `Bundle.main.resourceURL` and the Application Support folder (create it);
     keep true black, full screen, status bar hidden; the hidden two-player gesture if the Briefcase app had one.
   - `build-rust.sh`: copy into `Resources/` everything the engine and page need: work out from `Engine::load` and the
     server which data files (cards.json, decks.json, printings/, card images and their index? check what Briefcase
     `ios/stage.py` bundled and why) plus the page (`rust/crates/practice/static/` to `Resources/static`), and write
     `Resources/data/bundle.json` (git commit + time).
   - `project.yml`: the resources; the app icon: no Python here, so reuse a committed PNG (look in `ios/` for the icon
     Briefcase used) in an asset catalog. Bundle id / name as Briefcase had them, so the app replaces the old one if
     possible (say in the report whether it does, and that the old app's saves won't carry over if they don't).
   - `README.md`: no more "M6 proof"; `git checkout main` (not rust-port); one-time setup, `run.sh` for the
     simulator, and exact steps to put it on JD's own iPad from Xcode (signing with a free Apple ID, trusting the
     developer on the iPad, the 7-day re-sign limit). A short manual checklist for JD: start a game, play a few
     turns, close the app from the app switcher, reopen, *Continue*; touch, long press to enlarge, AI speed.
   - If `rustup target add aarch64-apple-ios` works here, try `cargo check -j 4 -p practice --lib --target aarch64-apple-ios`
     (no linker needed) to catch iOS-only compile errors; report what happened.
5. Do **not** delete `ios/` or `tools/app_icon.py`: that happens after JD has it working on the device.

## Checks
`cargo test --release -j 4 -p practice` passes (no ignored tests you added), `cargo build --release -j 4 -p practice`
has no warnings, `commander-practice --no-browser` still starts. Say plainly in the report that the Swift part is
unbuilt and what JD must check.

## Working
Write `rust/agents/handoff-ipad-shell.md` after your first concrete step and keep it current; commit after each
piece (standalone `git add -A && git commit -m "..."`, no Co-Authored-By line; don't push). Report in at most 15 lines;
details in `rust/agents/report-ipad-shell.md`. Final commit message: "iPad app on the Rust engine: real practice
server, first-launch data copy, ios-shell build and README".
