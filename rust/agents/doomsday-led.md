# Task: Lion's Eye Diamond as Doomsday mana in JD's thrasios deck (JD approved, 2026-11-06)

Worktree: `/home/joseph/projects/commander-sim/.claude/worktrees/doomsday-led` (branch doomsday-led, from main). Task
name: `doomsday-led`. JD's `thrasios` (Tymna/Thrasios Doomsday, Tier 6; list `decklists/JD/tymna-thrasios-doomsday.md`)
plays Lion's Eye Diamond (LED) but no wheel or Underworld Breach, so the sim never uses it. Its real job in a Doomsday
deck is mana for Doomsday and the pile. Read `rust/crates/sim-core/src/impls/doomsday.rs` (header first: the pile
builder `build`, the exact search `solve`/`dfs`, `Env::ok`, `doomsday_cost`, `plan_cast`, `run_pile`) and
`impls/mine.rs` (`led_options`, `led_uses`, `led_wheel_options`: how LED is cracked today).

## The rules (check each against the Oracle text and rulings; say so in the report)
LED: "{0} artifact. Discard your hand, Sacrifice Lion's Eye Diamond: Add three mana of any one color. Activate only as an
instant." It is a mana ability, so it can be activated while casting a spell, after the spell is on the stack: the spell
is not discarded. Doomsday: "Search your library and graveyard for five cards and exile the rest. Put the chosen cards on
top of your library in any order. You lose half your life, rounded up."

## Do
1. **LED pays for Doomsday.** Casting Doomsday with LED's {B}{B}{B}: Doomsday is on the stack, the rest of the hand is
   discarded to the graveyard, and Doomsday (which searches library *and* graveyard) can put those discarded cards into
   the pile. Let the cast (`doomsday_cost` / `plan_cast` / `go_doomsday`) use LED's mana when it is on the battlefield,
   and build the pile for the position it leaves: an empty hand, the discards available to the pile. Prefer it when it
   wins where the plain cast doesn't (no other {B}{B}{B}, or the hand's mana is needed for the pile), or when it frees
   mana the pile needs; not when the hand holds cards the line needs that the pile can't take (e.g. a held Veil or
   Noxious Revival that only work from hand: count that).
2. **LED inside the pile.** In the exact search (`dfs`), add an action: crack LED while casting a pile spell (the spell
   is on the stack), discarding the rest of the hand and adding three mana of one colour. The search already knows the
   hand, so it only takes this when the discarded cards aren't needed (the discard empties the hand: Thassa's Oracle in
   hand is lost unless it is the spell being cast). The mana lasts the phase (it is spent within the same pile turn).
3. **LED on the battlefield first.** It must be cast (free) before it can be cracked: make sure the AI casts LED from
   hand when a Doomsday line is live (Doomsday in hand or wanted by a tutor) and keeps it rather than cracking it for
   nothing; the plan file `data/plans/thrasios.toml` gets a cast-priority row if needed (no change to the deck list).
4. `t6stats::makes_mana_now` and `led_uses`: count the Doomsday use as a use.
5. Update the pilot guide's LED lines if they say something the model now contradicts (JD's guide, keep the style), and
   `audit/thrasios/FINDINGS.md` (one short entry: what LED now does in the sim).

## Tests
New `rust/crates/sim-core/tests/doomsday_led.rs`: LED pays Doomsday with no other black mana and the pile uses a card
the LED discarded (from the graveyard); LED cracked inside a pile pays a spell and the line still wins; LED is not
cracked when it would discard the Oracle the line needs; the AI casts LED from hand with Doomsday in hand; a control
where LED is not used because the plain cast is better. Run your file, `doomsday`, `thrasios_ai`, `t6_ai`, `t6_ai_b`,
`c1_t6_lines`, `c1b_engines`, `ai_t6_*`, `engine_t6_*`, `veil_oracle`, `plans`, `plan_files_3`, `generic_paths`. Check
the look-ahead stays fast: the old hang repro `commander-sim --deck thrasios --pool t6 --ai lookahead --seed 500400
--trace 24` must finish well under a minute (single game, allowed), and the builder's timing tests in `doomsday.rs` pass.

## Ground rules (these override the older rust/agents/RULES.md, which is from the Python-port era)
- **JD's rule: no shortcoming is accepted quietly.** Anything you can't make right (a card below Full, a case you skip, a
  caveat): commit what you have, list it at the top of your reply under `NEEDS JD`, and carry on.
- Work only in your worktree; `cd` into it for every command. Don't merge other branches. Don't push.
  Toolchain: `export PATH=$HOME/.cargo/bin:$PATH`; build from `rust/`.
- **Never run Python, not even a version check** (a hook refuses it). Use jq, shell tools, cardgen or Rust.
- **Never use `git stash`** in any form (the stash is shared with other sessions). Use a WIP commit.
- **Memory cap on every cargo command and every binary you run, no exceptions**:
  `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 <cmd>` (zsh: write the prefix out in full).
  Exit 137 = cap hit = a bug.
- No sims (no `--realism`, `--pool`, `--calibrate` or multi-game runs) and no full `cargo test` suite: the coordinator
  runs those. Single `--trace N` games are fine. Leave nothing running when you stop.
- Loose profile only. Other agents work in other worktrees at the same time: never touch files outside yours.
- Never let an unsigned counter go below zero. AI and person code stay separate (`if crate::human::is_human(g, p)`).
- `rustfmt --edition 2024 <the files you changed>` only; never `cargo fmt` on the workspace.
- Commit often inside the worktree (`git commit`, **no Co-Authored-By line**). Keep `rust/agents/handoff-<task>.md`
  current. If your context gets long (~80+ tool calls), stop at a clean point, everything committed, reply's first line
  `HANDOFF`.
- Write the full report to `rust/agents/report-<task>.md` and commit it (if the harness refuses the write, put the full
  report in your reply after a `---- FULL REPORT ----` line). Reply in at most 15 lines.
- `cargo build --release --all-targets` with no warnings before you finish; run the test files the brief names with
  `cargo test --release --no-fail-fast --test <name> ...`.
