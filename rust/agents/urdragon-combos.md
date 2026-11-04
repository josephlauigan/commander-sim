# Task: add The Ur-Dragon deck's three infinite combos to the combo catalogue

Worktree: `/home/joseph/projects/commander-sim/.claude/worktrees/urdragon-combos`, branch `urdragon-combos`, based on
origin/main (51d4d31a: Garth One-Eye, Cursed Mirror, Astral Dragon and the new Ur-Dragon plan are all merged). `cd`
into it for every command. Commit there; do not push, do not open a PR.

## Which rules apply
Read the repo's `CLAUDE.md` and `AGENTS.md`, then `rust/agents/RULES.md` only for its conventions (impls layout,
hooks, registration, commit often, handoff file, short report). **Ignore the parts of RULES.md that are out of date:**
there is no Python reference and no `rust-port` branch, so don't merge `rust-port` and don't run `difftest`. Never run
Python, not even a version check: a hook blocks it. This task is already approved by JD; work through it without
asking.
- **Memory cap** on every cargo or binary command: `systemd-run --user --scope -q -p MemoryMax=12G -p MemorySwapMax=0
  <cmd>`. Exit 137 means the cap was hit: treat it as a memory bug. The shell is zsh, so write the cap out in full;
  don't store it in a variable.
- **Run no simulations** (no `commander-sim` runs, no `all_decks`). The coordinator runs the sims afterwards.
- **Formatting:** run `rustfmt --edition 2024` only on files you create. Existing files aren't rustfmt-clean, so in
  them hand-format just your own lines.
- No `Co-Authored-By:` line in commits.
- **Stay in your area.** Another agent is editing `decklists/` on another branch, so don't touch it.

## The combos
Commander Spellbook finds these three in the rebuilt deck (`data/spellbook/urdragon.json`, `included`; read each
variant's `cards`, `results`, `prerequisites` and `requires` there). Work out each loop's exact sequence from the
cards' Oracle text (`data/scryfall_cache.json`) and check it against the Spellbook entry:

1. **Astral Dragon + Cursed Mirror:** infinite copies of noncreature permanents you control, and infinite hasty
   creature tokens.
2. **Miirym, Sentinel Wyrm + Astral Dragon + Doubling Season:** near-infinite creature tokens.
3. **Garth One-Eye + Twinflame + Spark Double + Doubling Season:** infinite card draw, infinite coloured mana,
   infinite creature tokens.

## Work
1. **The engine's catalogue.** Read how a combo is defined (`impls/combos.rs`: `Combo`, `Ready`, `Step`, `StepKind`)
   and copy the closest existing pattern. Good models are `impls/t6_combos_godo.rs` and the Helm + Combat Celebrant
   entry in `combos.rs`. Each catalogue entry has:
   - `groups`: the pieces;
   - `ready`: when the pieces are in the right zones and the mana and other prerequisites are there; never earlier;
   - `mana`;
   - `steps`, each with its answer window, so opponents can respond the way the existing combos allow;
   - `finish`: how the loop resolves as its result.
   
   A loop of hasty tokens wins the way the other infinite-attack combos do. Infinite draw and mana should end in a
   win the same way as an existing combo with that result. Say in your report which existing combo each finish copies.
   
   Put the three entries in a new file `rust/crates/sim-core/src/impls/t45_urdragon_combos.rs`, registered with
   `.chain(...)` in `combos.rs` and in `impls/mod.rs`.
2. **The plan file.** Add three `[[line]]` entries to `data/plans/urdragon.toml`, one per combo, with
   `combo = "<exact catalogue name>"`, so the tutors hunt the first missing piece. Put them ahead of the value lines,
   best first. Add `[[wish]]` or `[[prio]]` rules only where a piece needs them (e.g. hold Cursed Mirror until Astral
   Dragon can follow). Also update the comment in that file that names the deck document: it is being renamed
   `decklists/JD/ur-dragon-equipment.md` → `decklists/JD/ur-dragon-dragon-copies.md` on another branch.
3. **Tests**, in a new `rust/crates/sim-core/tests/combos_urdragon.rs`. For each combo:
   - not ready with a piece missing, or too little mana;
   - ready with everything in place;
   - running it produces its result or the win, and a counterspell or removal at a step stops it where the existing
     combos allow that.
   
   The existing guards must still pass: `plans` (including `every_plan_file_fits_its_deck`, and the catalogue-lines
   report if relevant) and `spellbook` (plan lines vs Spellbook coverage).

## Finish
- **Tests:** `cargo test --release --no-fail-fast` (from `rust/`, under the cap) passes with **no failures**, since
  main is green. `cargo build --release --all-targets` has no warnings.
- **Commits:** everything committed, with `rust/agents/handoff-urdragon-combos.md` kept current while you work.
- **Report:** if you can't write `rust/agents/report-urdragon-combos.md`, put the full report in your final message.
  It covers each combo's sequence and how it's modelled, which existing combo each finish follows, any simplification
  and why, and every shared file touched.
- **Final message:** at most 15 lines plus the full report if needed: commit hash, done or not done, test result,
  anything the coordinator must act on. Then stop.
