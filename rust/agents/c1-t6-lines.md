# Task: Phase C1, Tier 6 decks turn "one tutor away" into a win too slowly (JD approved, 2026-11-05)

Worktree: `/home/joseph/projects/commander-sim/.claude/worktrees/c1-t6-lines` (branch c1-t6-lines, from origin/main 336b44f5).
Task name: `c1-t6-lines`. **No sims** (see the ground rules: the coordinator runs the scorecards; you may trace single games).

**JD's rule: no gap may be left quietly.** Anything you can't make right: commit, list it at the top of your reply under
`NEEDS JD` (exact gap, effect, what it would take), and carry on with the rest.

## The measured problem (2026-11-05, on 996befd1 = main before this branch)
Files in `audit/realism/c1-2026-11-06/`: `t6-adaptive-8200.txt` and `t6-lookahead-2200.txt` (`--realism --pool t6`),
`t6-cal-4000.txt` (`--calibrate within --pool t6 --ai adaptive --games 4000 --seed 900000`). Read `documents/realism.md`
and `audit/realism/taxed-costs-2026-10-24/REPORT.md` (what was already fixed) first.
- Scorecard: 0/7 (adaptive) and 2/7 (look-ahead) targets. Median win round 8 (adaptive) / 7 (look-ahead) against 5 (band
  4-6); win round spread 5 against 2 (1-3); seat 1 26-29% against 31.8%; deck spread ~20 pts against 4 (0-8): Kinnan 14%,
  Sisay 34-35%.
- From the calibrate report, per deck (sections in order Dack, Kinnan, Kraum/Tymna, Rograkh/Silas, Rograkh/Thrasios, Sisay):
  - **Kinnan**: one tutor away in 53% of games (median round 5) but ready in only 13% (round 9). While one tutor away it
    chose Kinnan 841, Green Sun's Zenith 293, hold mana 258, Finale 252, Nature's Rhythm 242, Chord 227 ...; tutors held
    when it passed: GSZ 129, Nature's Rhythm 103, Finale 78. Combos stopped 38% (Snap, March of Otherworldly Light ...).
  - **Kraum/Tymna**: ready in 38% of games, but in 193 ready games it never went, and while ready it chose "stop (hold
    mana)" 418 and "stop" 263. While one tutor away: hold mana 524; tutors held when it passed: Mystical Tutor 251,
    Diabolic Intent 249, Enlightened Tutor 175, Demonic Tutor 62. Step-2 stops: "Demonic Consultation / Tainted Pact in
    response exiles the library" 13.
  - **Rograkh/Silas**: while one tutor away: hold mana 190, stop 107; tutors held: Mystical Tutor 188, Diabolic Intent 55,
    Demonic Counsel 53.
  - **Rograkh/Thrasios**: ready 19% at round 9; tutors held: Finale 461, GSZ 329 (the October report: Finale is held as
    the finisher); combo steps fail with no card against them: "Snap on Eternal Witness (untap two lands)" 27, "cast Oboro
    Breezecaller" 20+18, "cast Eternal Witness" 7, "cast Finale" 7.
  - **Sisay**: strongest (34-35%); combos stopped only 11%.
  - Every deck: Lion's Eye Diamond held at the end of turns 1-3 ~550-630 card-turns (check it's right to hold), Chrome Mox
    / Mox Diamond held with the mana to cast them.

## Items
1. **Find why each deck's tutors don't turn "one tutor away" into "ready"**, deck by deck, from traces
   (`commander-sim --calibrate within --pool t6 --ai adaptive --games 4000 --seed 900000 --trace N` prints game N). For
   each cause, decide: a rules bug, an AI decision bug (it holds a tutor it should cast, casts it at the wrong time, fetches
   the wrong card, or the tutor can't find the missing piece and the AI doesn't know), or the list. Real cEDH play: an
   instant tutor (Mystical, Enlightened, Vampiric) is cast at the end step of the player before you for the missing piece;
   a sorcery tutor on your own turn when the line can then go (or set up for next turn when it can't).
2. **Fix the AI and rules causes**: in plan files (`data/plans/<deck>.toml`) where the format can express it, else in the
   AI code (`ai/`, `impls/t6_*`, `impls/combos.rs`). Keep each fix general where possible (a rule for instant tutors at
   end of turn is better than one per deck). Kraum/Tymna and Rograkh/Silas holding while ready: find what they wait for
   (protection? a lock check?) and whether a real pilot would wait; the October report called part of it "the spec's
   waiting for protection, a style choice": quantify it, and fix whatever is not a deliberate choice.
3. **Rograkh/Thrasios's failing steps** ("no card" stops at steps 1-2 of its own lines): find and fix each.
4. **Kinnan** (14%, the weakest): find why one tutor away rarely becomes ready, and fix.
5. **Sisay** (34%, the strongest): check whether anything in the rules or AI over-favours it (lines that should be
   answerable but aren't: e.g. Stifle-type effects or hate pieces other decks run that never target abilities). Don't
   weaken it artificially: only rules or AI fixes.
6. Write `audit/realism/c1-2026-11-06/REPORT.md`: each cause found (with the trace evidence: game numbers), the fix, and
   what you expect it to move. List anything that is the list, not the engine, separately (JD decides lists).

## Tests
New file `rust/crates/sim-core/tests/c1_t6_lines.rs`: a test per fix (a scripted position where the old AI held / failed
and the new one acts). Run with `--no-fail-fast`: your file, `ai_t6_*`, `engine_t6_*`, `cards_t6_*`, `ai_planner`,
`plans`, `plan_files_3`, `ai`, `ai_flow`, `combo_outlets`, `generic_paths`, and the test files of anything you touch.

## Done when
- Items done (or under NEEDS JD); tests pass; `cargo build --release --all-targets` has no warnings; report written.
- Reply: summary of at most 15 lines, then the full report under a line `---- FULL REPORT ----`. The coordinator then
  runs the scorecards on your branch and may send you a short follow-up.

## Ground rules (these override the older rust/agents/RULES.md, which is from the Python-port era)
- Work only in your worktree (path above); `cd` into it for every command. Don't merge other branches. Don't push.
  Toolchain: `export PATH=$HOME/.cargo/bin:$PATH`; build from `rust/`.
- **Never run Python, not even a version check** (a hook refuses it). Use jq, shell tools, or Rust.
- **Memory cap on every cargo command and every binary you run, no exceptions**: `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 <cmd>`
  (cargo from `rust/`). The shell is zsh: write the prefix out in full, don't store it in a variable. Exit 137 = cap hit = a bug.
- **No sims and no full suite**: no `--realism`, no multi-game `--calibrate` or `--deck` runs. You may print single games
  with `--trace N` (one game each) and run test binaries. Don't leave anything running.
- Loose profile only (the default). Other agents work in other worktrees at the same time: never touch files outside yours.
- Never let an unsigned counter go below zero (release builds have overflow checks).
- AI and person code stay separate (`if crate::human::is_human(g, p)`).
- `rustfmt --edition 2024 <your new files>` only; never `cargo fmt` on the workspace.
- Commit often (`git add -A && git commit -q -m "..."`, inside the worktree); **no Co-Authored-By line**. Keep
  `rust/agents/handoff-c1-t6-lines.md` current. If your context gets long (~80+ tool calls), stop at a clean point with
  everything committed and end your reply with a first line `HANDOFF`.
- Finish with everything committed and the reply as above. Then stop.
