# Task: wire JD's two new decks into the engine, retire karn and urdragon (JD approved, 2026-11-05)

Worktree: `/home/joseph/projects/commander-sim/.claude/worktrees/deck-rewire` (branch deck-rewire, from origin/main a5a68b6b).
Task name: `deck-rewire`.

**JD's rule: no gap may be left quietly.** Don't drop test coverage, leave a stale reference, or add a "left out" note on
your own judgement. Anything you can't do: stop that part, commit, and list it at the top of your reply under `NEEDS JD`
(exact gap, effect, what it would take).

## Background (already merged: PRs #268, #269)
- New lists: `decklists/JD/tymna-thrasios-doomsday.md` (Tymna/Thrasios rebuilt around Doomsday, cEDH) and
  `decklists/JD/thanos-artifacts-and-dragons.md` (Thanos commander, The Ur-Dragon in the 99, bracket 4, meant for t5).
- Swap guide: `documents/doomsday-thanos-rebuild-swaps.md` (every card per box and pile; the source for the binder).
- Printings: `data/printings/tymna-thrasios-doomsday.txt`, `data/printings/thanos-artifacts-and-dragons.txt`.
- Retirement notes (2026-11-05) are at the top of `karn-colorless-artifacts.md`, `ur-dragon-dragon-copies.md`,
  `tymna-thrasios-partners.md`, `thanos-five-colour-turbo.md`, plus CHANGELOG entries. `data/my_decks.toml` is unchanged.
- Precedent: the Ghave retirement, commit 34ca1d03 (`git show 34ca1d03 --stat`, then the diffs). Follow it.
- `cardgen audit --cards` on the new lists: every card Full/Full-auto except **Doomsday** (Unmodeled, not in the card
  database). Doomsday is NOT this task (a separate PR 2 models it); leave it unmodelled, but the export must still work
  with it in the list (see how other unmodelled cards are handled; if export refuses, NEEDS JD).

## Items (JD approved each)
1. `data/my_decks.toml`: `thrasios` -> `file = "JD/tymna-thrasios-doomsday.md"`, `tiers = ["t6"]` (name, commanders, ident:
   check against the new list). `thanos` -> `file = "JD/thanos-artifacts-and-dragons.md"`, `tiers = ["t5"]`. Remove the
   `karn` and `urdragon` keys. Then `./rust/target/release/cardgen export --offline` (decks.json, cards.json), and
   regenerate `tests/fixtures/my_decks_parsed.json` the way the deck guard expects (find its generator / test).
2. Move the four old lists to `decklists/pool/retired/` (git mv), fix paths in their notes, `decklists/CHANGELOG.md`, and
   `decklists/pool/README.md` (add a retirement line as for the others), `documents/testing-decks.md`, READMEs.
3. Printings: `git mv` the two new files to `data/printings/thrasios.txt` and `data/printings/thanos.txt` (replacing the
   old ones); remove `karn.txt` and `urdragon.txt`. Check practice mode reads them (catalog / printings loader + its tests).
4. Plans and Spellbook:
   - Remove `data/plans/karn.toml`, `data/plans/urdragon.toml`, `data/spellbook/karn.json`, `data/spellbook/urdragon.json`.
   - Rewrite `data/plans/thanos.toml` for the new list at t5. Its lines: Basalt Monolith + Rings of Brighthearth into
     Walking Ballista / Mystic Forge / Thrasios-style mana sinks in the list; Astral Dragon + Cursed Mirror with Terror of
     the Peaks; Thanos's power-up wipes. Read the pilot guide for the lines, and the existing plan-file format and
     loaders (`ai/plan_data.rs`, `ai/plans.rs`, other plan files). Every card a line names must be in the new list.
     Note: every t5 commander has an even mana value (Kinnan 2; Urza, Yawgmoth, Zur 4; Miirym 6); Rings can copy Thanos's
     power-up to hit both parities.
   - `data/plans/thrasios.toml`: remove every line whose cards left the deck; keep only lines the new list can still play.
     The Doomsday lines are PR 2: don't add them.
   - Regenerate the Spellbook files for both new lists (find how they were made: git log on data/spellbook, sim-cli or
     cardgen commands; if it needs the internet and that is unavailable, NEEDS JD).
5. `audit/`: `git mv audit/karn audit/retired/karn` and `audit/urdragon audit/retired/urdragon` (JD's choice); add a
   "retired 2026-11-05" line at the top of each FINDINGS. For thrasios and thanos: `git mv` the old FINDINGS.md to
   `FINDINGS-old-list.md` in the same folder and start a new `FINDINGS.md` (header: which list, tiers, "no sims yet").
   Fix any references to the moved paths.
6. Code: remove code that only runs for the retired deck keys (`karn`, `urdragon`), as Ghave's was (JD's choice). Keep every
   card model (the new Thanos and pool decks use many of these cards). Settings/knobs/CLI/report/practice catalog
   references to the old keys go. `grep -rn '"karn"\|"urdragon"' rust/` lists ~45 files.
7. Tests: about 45 test files seat `karn` / `urdragon` (and the old `thanos` / `thrasios` lists). Re-seat each so it still
   tests the same card or engine behaviour: on the new `thanos` (it has 27 Karn and 67 Ur-Dragon cards) or the new
   `thrasios` where the cards are there, else on a pool deck with them, else a `testkit::retired_decks` entry (it builds
   commander + basics only; extend it if a test needs a fuller list). No test may be deleted or weakened without a
   NEEDS JD line, except tests of code removed in item 6 (list each in the report).
8. `binder-cards.txt`: add the cards the swap guide sends to the binder (12 from old Thanos, 33 from Ur-Dragon, the rest of
   Karn and old Tymna), in the file's format; remove any binder line for a card now in either new list.
9. Rules files (JD approved this edit): in `CLAUDE.md` ("Opponent tiers" paragraph) and `AGENTS.md` (wherever tiers or the
   owned decks are listed), make the tier line: thrasios t6 (cEDH; new Doomsday list 11/05), thanos t5 (new artifacts and
   Dragons list 11/05; no longer Scion's t6 seat), karn and urdragon retired 11/05. Keep the rest of each file unchanged.
   Check `git show origin/main:CLAUDE.md` first so you edit the current text.

## Checks
- `cargo build --release --all-targets`: no warnings.
- Run every test file you changed or that seats the changed decks, plus `practice` crate tests, deck-guard test, plan
  loader tests (`plans`, `plan_files_3`), `generic_paths`, `realism` (sim-cli), with `--no-fail-fast`.
- `cardgen audit --mine`: report its output (expect only Doomsday below Full).
- Don't run the full suite or sims; the coordinator does.

## Report
The harness may refuse an agent's write of a report file. Return the full report as text in your reply, after a summary of
at most 15 lines, under a line `---- FULL REPORT ----`: per item what changed, files, tests re-seated (old deck -> new
deck), code removed, every NEEDS JD. The coordinator saves it to `rust/agents/report-deck-rewire.md`.

## Ground rules (these override the older rust/agents/RULES.md, which is from the Python-port era)
- Work only in your worktree (path above); `cd` into it for every command. Don't merge other branches. Don't push.
  Toolchain: `export PATH=$HOME/.cargo/bin:$PATH`; build from `rust/`.
- **Never run Python, not even a version check** (a hook refuses it). Use jq, shell tools, or Rust.
- **Memory cap on every cargo command and every binary you run, no exceptions**: `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 <cmd>`
  (cargo from `rust/`). The shell is zsh: write the prefix out in full, don't store it in a variable. Exit 137 = cap hit = a bug.
- No sims and no full suite. Run tests in the foreground with a long timeout; no background jobs; leave nothing running.
- Never let an unsigned counter go below zero (release builds have overflow checks).
- `rustfmt --edition 2024 <your new files>` only; never `cargo fmt` on the workspace.
- Oracle text: `jq '.["<lowercased name>"].oracle_text' data/scryfall_cache.json`.
- Commit often (`git add -A && git commit -q -m "..."`, only inside the worktree); **no Co-Authored-By line**. Keep
  `rust/agents/handoff-deck-rewire.md` current. If your context gets long (~80+ tool calls), stop at a clean point with
  everything committed and end your reply with a first line `HANDOFF`.
- Finish with everything committed and the reply as above. Then stop.
