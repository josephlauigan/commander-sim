# Brief: AI foundations step 2, a goal-directed line planner for Tier 6 decks

Work in the worktree `.claude/worktrees/ai-foundations` (branch `ai-foundations`, after step 1). It is your whole
world. Read `CLAUDE.md` first. Don't use `rust/agents/RULES.md`.

## Why

The audit (`audit/ai-foundations/REPORT.md` and `architecture-review.md`, section 8) found that nothing in the AI
plans toward a win. The heuristic and the look-ahead pick one play at a time, from a cast-order priority, over a
horizon that ends after the deck's next turn. Step 1 (`step1-fixes.md`) fixed the plain bugs and bought about a turn.
Scion's goldfish median is now 9, against a crude best case of about 6 on the same draws (`loss-accounting.md`). JD
approved a planner as the real fix. The Tier 6 table's calibration is step 3, not this brief.

## What to build

A new module, `sim-core/src/ai/planner.rs`. It is used only for decks in `t6_plans::T6_KEYS`, which includes `scion`,
and can be switched off with a settings knob (env `PLANNER=0` and `[ai] planner = false` in a knobs file).

1. **The goals** are the catalogued lines (`impls/combos.rs`): their pieces, `ready_lines`, `audit_ready`,
   affordability (with the step 1 ritual accounting), and the table checks (`t6_table::go_ok`,
   `protect_before_go`). Don't invent new lines.
2. **`plan_this_turn(g, p)`.** A depth-limited search over own-turn actions, run on game clones (`search::clone`).
   - The actions: mana (lands, rocks, moxen including Mox Diamond and Chrome Mox, Lotus Petal, rituals, untapped
     non-sick dorks, Spirit Guides, LED as the line uses it), tutors for a chosen target, and casting a line's pieces.
   - It succeeds when a line is ready, affordable and cleared to go (`go_ok`). It returns the action sequence.
   - Only those action kinds are expanded, so branching stays small. Cap the depth (about 6) and the nodes per call,
     and say what you picked.
   - Tutors need a **forced-target hook**: the planner picks the card, and `engine/tutors.rs` takes it (to hand or to
     top, as the card says). This must be legal: the target meets the tutor's real restriction, and the step 1 record
     of known top cards is kept. A tutor to the top wins only next turn (or after a draw this turn), and the plan has
     to account for that.
3. **`plan_next_turn(g, p)`.** Used when nothing wins this turn. It picks the play this turn, including an
   end-of-turn instant tutor, that makes a line ready and affordable on the deck's next turn.
   - Next turn's mana is the untapped sources, plus a land drop if a land is in hand, plus the rituals and one-shot
     mana in hand.
   - The pieces are the hand plus the known top card. Don't count random draws.
   - Rank the candidate plays by: ready next turn, then most protection kept, then most cards left.
   - Keep it simple and deterministic. Say how it's scored.
4. **Integration in `brain::main`,** before `t6_table::must_option`:
   - When `plan_this_turn` returns a sequence, take its first action, then re-plan at the next decision (a receding
     horizon, so the plan stays honest when a draw or a trigger changes things).
   - Otherwise, when `plan_next_turn` finds a play, take it as a must-do.
   - With no plan, the existing AI (step 1 behaviour) runs unchanged.
   - Respond to the table as now: hold-ups, counters and protection before going stay with `go_ok` and
     `protect_before_go`. In the end-of-turn window, `plan_next_turn` may choose the instant tutor.
   - Log a planned action in traces as `[plan: ...]` with the sequence.
5. **Leave alone:** the utility AI for every other deck (Tiers 0-5 and JD's other decks), the Tier 6 table's
   interaction AI, the lists, and the realism targets.

## Tests (one per behaviour)

- A turn-2 Oracle kill through a ritual: two lands plus Dark Ritual, with Thassa's Oracle and Demonic Consultation.
- Tutor into the line this turn: Demonic Tutor for the missing piece, with mana for both.
- A tutor that fetches mana when the line lacks only mana.
- A to-top tutor at end of turn, then the kill next turn.
- No plan in a hand with no line: the old AI's choice is unchanged.
- The forced tutor target obeys the tutor's restriction: Enlightened Tutor can't take Oracle.
- `PLANNER=0` gives the step 1 behaviour.
- With an opponent's counter visible and `go_ok` false, the planner doesn't go.

## Measure

1. **The goldfish**, `--goldfish t6,scion --games 2000 --seed 500000`, with the planner off (that's step 1) and on.
   Report the median, the shares by turn 2, 3 and 4, the loss-accounting buckets, and the gap to the crude best case
   (`loss-accounting.md`). The target is to close most of that gap. Explain what is left, with 10 traced games for
   scion and two other decks.
2. **The table:** `--realism --pool t6 --games 4000`, planner off and on. Also `--deck scion --pool t6 --games 2000`.
   Expect faster Tier 6 games and a still higher combo share. Report it, but don't tune the table (that's step 3).
3. **The gate check:** `--realism --pool t3 --games 1000` and `--deck cloud --pool t1 --games 500`, planner off and
   on. The output must be identical: the planner may consume no RNG for decks that don't use it. If it isn't
   identical, find out why and fix it.

Write it all up in `audit/ai-foundations/step2-planner.md`, with the raw output in `audit/ai-foundations/raw/step2/`.

## Rules

- **Commands.** Run every build, test and sim from `rust/`, with `run_in_background`, under
  `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 ...`. Put anything longer than about 10 minutes
  in a detached `systemd-run --user --unit=step2-<name> ...` unit and poll its output files. Never use one blocking
  command longer than 10 minutes.
- **Threads.** Use `--jobs 16` and `-j 16`, and run one sim or cargo command at a time.
- **No Python, ever.** Not even `python3 --version`, and not after a `cd` either: the hook only catches commands
  that *start* with python, and step 1 slipped one through that way.
- **Shared files.** Mark each change in a shared file `// --- ai-planner`. Comments explain why, briefly, in the
  surrounding style.
- **Formatting.** Format only the files you write (`rustfmt --edition 2024 <file>`), never `cargo fmt`. Lines go up
  to 120 characters.
- **Commits.** Commit at each working milestone: the this-turn planner, the next-turn planner, integration, and the
  measurements. No Co-Authored-By line. Never push. Keep `rust/agents/handoff-ai-planner.md` up to date and commit it
  with your work.
- **Before you finish:** `cargo test --release --no-fail-fast` passes, `cargo build --release --all-targets` has no
  warnings, and nothing of yours is still running (`pgrep -af 'commander-sim|cargo'`).
- **Decisions.** JD may be away. Make the conservative call and record it, with the alternative, in a "Decisions"
  list in `step2-planner.md`.
- **Report** in your final reply, in 15 lines or fewer: the goldfish before and after, the Tier 6 table before and
  after, the gate check, and anything JD must decide.
