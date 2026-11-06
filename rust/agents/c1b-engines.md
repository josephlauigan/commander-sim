# Task: Phase C1b, Tier 6 decks hold their draw engines and wheels (JD approved Phase C, 2026-11-05)

Worktree: `/home/joseph/projects/commander-sim/.claude/worktrees/c1b-engines` (branch c1b-engines, from c1-t6-lines 8a7650e8,
PR #271). Task name: `c1b-engines`. **No sims** (the coordinator runs scorecards; you may trace single games).

**JD's rule: no gap may be left quietly.** Anything you can't make right: commit, list it at the top of your reply under
`NEEDS JD`, and carry on.

## The measured problem
Files in `audit/realism/c1b-2026-11-06/` (on the C1 code): `t6-adaptive-8200.txt` (`--realism --pool t6 --ai adaptive
--games 8200 --seed 900000`; its "Card and mana flow" sections per deck) and `t6-cal-4000.txt`. Read
`audit/realism/c1-2026-11-06/REPORT.md` and `documents/realism.md` first. Tier 6 median win round 8 against 5 (band 4-6);
the October reports say the decks are draw- and mana-limited, not waiting. The flow tables show why: draw engines and
wheels sit in hand, payable, never cast. "Seen, never cast, at the end of turn 4 (where it is, games)", `hand_payable` =
in hand with the mana to cast it:
- Rograkh/Silas: Wheel of Fortune 342, Windfall 308, Necropotence 266, Timetwister 228, Ad Nauseam 171, Rhystic Study 155.
  Funnel: Wheel of Fortune seen 18% / cast 1% by T8; Windfall 18/1; Timetwister 17/9.
- Kraum/Tymna: Necropotence 354, Ad Nauseam 176, Faerie Mastermind 232, Esper Sentinel 102, Rhystic Study 97.
- Kinnan: Thrasios 279, Faerie Mastermind 241, Sylvan Library 138, The One Ring 117, Consecrated Sphinx 113, Rhystic Study 89.
- Dack: The One Ring hand_payable 125.
- Cantrips cast ~0.02 a turn in every deck; cards drawn ~1.0-1.3 a turn for most decks through T4.
Real cEDH: Rhystic Study, Mystic Remora, Sylvan Library, Esper Sentinel and The One Ring are cast early (turn 1-3);
Necropotence turn 1-3 paying 5-10 life; wheels cast to refill an empty hand (or with Notion Thief / a Breach line);
Faerie Mastermind and Thrasios are cast on curve.

## Items
1. For each card above, find why the AI holds it while payable (trace games: `commander-sim --calibrate within --pool t6
   --ai adaptive --games 4000 --seed 900000 --trace N`): its value/priority in the generic policy (`ai/values.rs`,
   `ai/brain.rs`, `ai/planner.rs`), a plan-file `[[prio]]`/`[[hold]]` that holds it, a Tier 6 rule, a hand-written hook
   that refuses it, a lock check, or a bug. Distinguish deliberate holds (Ad Nauseam as a finisher; a wheel held for an
   Underworld Breach line) from mistakes, and check the deliberate ones against how real pilots play these lists.
2. Fix the causes: engines cast early when the mana is spare (with protection or into a tapped-out table where it
   matters); Necropotence paid for the right number of cards (life budget like real play, not 0); wheels when the hand is
   small or as part of a line; Faerie Mastermind / Thrasios on curve. Prefer general rules (a draw-engine value that counts
   turns left) over per-deck tweaks; plan-file edits where a plan holds a card wrongly.
3. Keep the deliberate holds where real play agrees (say so with the reason).
4. Write `audit/realism/c1b-2026-11-06/REPORT.md` (causes with trace game numbers, fixes, expected effect, what's left).

## Tests
New `rust/crates/sim-core/tests/c1b_engines.rs`: a test per fix (a scripted hand where the old AI held and the new one
casts). Run `--no-fail-fast`: your file, `c1_t6_lines`, `ai_t6_*`, `engine_t6_*`, `cards_t6_*`, `ai_planner`, `ai`,
`ai_flow`, `ai_eval`, `plans`, `plan_files_3`, `generic_paths`, `thrasios_ai`, and the test files of what you touch. The
generic policy also runs JD's decks: run their card test files (`cards_sauron*`, `cards_sheoldred*`, `cards_cloud*`,
`cards_kefka*`, `cards_yshtola`, `cards_alela`, `cards_galadriel`, `cards_thanos`, `doomsday` if present) and say which
of JD's decks a change can affect.

## Done when / reply
Items done (or under NEEDS JD); tests pass; build without warnings; report written (the harness may refuse your write of
the report file: then return it in full in your reply). Reply: at most 15 lines, then `---- FULL REPORT ----`.

## Ground rules (these override the older rust/agents/RULES.md, which is from the Python-port era)
- Work only in your worktree; `cd` into it for every command. Don't merge other branches. Don't push.
  Toolchain: `export PATH=$HOME/.cargo/bin:$PATH`; build from `rust/`.
- **Never run Python, not even a version check.** Use jq, shell tools, or Rust.
- **Memory cap on every cargo command and every binary you run, no exceptions**: `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 <cmd>`
  (cargo from `rust/`). zsh: write the prefix out in full. Exit 137 = cap hit = a bug.
- No sims (no `--realism`, no multi-game runs) and no full suite; single `--trace N` games are fine. Leave nothing running.
- Loose profile only. Other agents work in other worktrees: never touch files outside yours.
- Never let an unsigned counter go below zero. AI and person code stay separate (`if crate::human::is_human(g, p)`).
- `rustfmt --edition 2024 <your new files>` only; never `cargo fmt` on the workspace.
- Commit often (inside the worktree); **no Co-Authored-By line**. Keep `rust/agents/handoff-c1b-engines.md` current. If your
  context gets long (~80+ tool calls), stop at a clean point, everything committed, reply's first line `HANDOFF`.
