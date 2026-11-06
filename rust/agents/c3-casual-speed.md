# Task: Phase C3, casual-tier games end too late and the deck spread is wide (JD approved, 2026-11-05)

Worktree: `/home/joseph/projects/commander-sim/.claude/worktrees/c3-casual-speed` (branch c3-casual-speed, from origin/main
336b44f5). Task name: `c3-casual-speed`. **No sims** (the coordinator runs the scorecards; you may trace single games).

**JD's rule: no gap may be left quietly.** Anything you can't make right: commit, list it at the top of your reply under
`NEEDS JD` (exact gap, effect, what it would take), and carry on with the rest.

## The measured problem (2026-11-05, on main before this branch)
Files in `audit/realism/c3-2026-11-06/`: `t3-adaptive-8200.txt`, `t5-adaptive-8200.txt` (`--realism --pool tN --ai adaptive
--games 8200 --seed 900000`), `t3-cal-4000.txt`, `t5-cal-4000.txt` (`--calibrate within`). Read `documents/realism.md`,
`audit/realism/t3-trace-2026-10-24.md`, `audit/realism/t3-lines-2026-10-26.md`, `audit/realism/t45-targets-2026-10-24.md`
and `audit/realism/casual-combos-2026-10-26.md` first (what was already tried: knob fits don't move speed or combo share).
- **Tier 3** (Atraxa, Aurelia, Korvold, Marwyn, Tergrid): 2/9 targets. Median win round 10 against 8 (band 7-9); seat 1
  28.6% against 32.3%; seat 4 21.3% against 18.0%; combo/alt 13.3% against 16.5%; damage 53.4% against 59.8%; commander
  damage 9.5% against 14.4%; **poison 23.8% against 9.3%** (Atraxa infect wins 37.7% of its games; Korvold 10.6%).
- **Tier 5** (Kinnan, Miirym, Urza, Yawgmoth, Zur): 1/5. Median win round 9 against 7 (6-8); seats 29.9/26.0/23.7/20.4
  against 31.8/27.8/22.4/18.0. Miirym 36.5%, Zur 18.0%, Yawgmoth 19.3%.
- These tiers matter most to JD: his decks play t1-t5 (and thrasios t6).

## Items
1. **Why are casual games two rounds slow?** Trace games (`commander-sim --calibrate within --pool t3 --ai adaptive
   --games 4000 --seed 900000 --trace N`, also t5) and find what the decks do on turns 3-8 that real players wouldn't:
   creatures not attacking into open boards, wasted mana, cards held, removal used too freely, attacks spread instead of
   focused on a beatable player, commanders not cast or not attacking. Real casual pods: damage and commander damage win
   most games around turn 8. For each cause: rules bug, AI decision bug, or the list (lists are JD's call: report them
   separately). Fix the rules and AI causes, generally (the attack/block policy for every casual deck, not one per deck).
2. **Tier 3 poison (Atraxa)**: 23.8% of wins by poison against 9.3%. Check infect/toxic/proliferate rules (poison counters
   from infect damage, toxic, proliferate adding poison, the 10-counter loss) against the Oracle and the rules; check the
   opponents' AI treats an infect attacker as the threat it is (blocks, removal priority). Fix what is wrong. If the rules
   and AI are right and it's the list, report it (JD's 10-24 decision allows pool lists to be updated toward real lists,
   but list changes need his approval: report, don't change).
3. **Korvold weak (10.6%) and Miirym strong (36.5%)**: find whether anything in the rules or AI under- or over-rates them
   (Korvold's sacrifice trigger and draws, its growing size; Miirym's Dragon copies). Rules/AI fixes only.
4. **Commander damage low (9.5% vs 14.4%)**: check commanders are cast, recast and attack as players do.
5. Write `audit/realism/c3-2026-11-06/REPORT.md`: each cause (with trace game numbers), the fix, what you expect it to
   move, and list-level findings separately.

## Tests
New file `rust/crates/sim-core/tests/c3_casual_speed.rs`: a test per fix (a scripted position where the old AI or rule was
wrong). Run with `--no-fail-fast`: your file, `ai`, `ai_flow`, `ai_eval`, `ai_planner`, `cards_t3*`, `cards_t45_*`,
`cards_t4`, `cards_t5`, `dsl_poison`, `combo_outlets`, `generic_paths`, `plans`, and the test files of anything you touch.
**JD's own decks** (`data/my_decks.toml`) also use the general attack/block AI: run their card test files too
(`cards_sauron*`, `cards_sheoldred*`, `cards_cloud*`, `cards_kefka*`, `cards_yshtola`, `cards_alela`, `cards_galadriel`,
`cards_thanos`) and list in the report which of JD's decks a change can affect.

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
  with `--trace N` and run test binaries. Don't leave anything running.
- Loose profile only (the default). Other agents work in other worktrees at the same time: never touch files outside yours.
- Never let an unsigned counter go below zero (release builds have overflow checks).
- AI and person code stay separate (`if crate::human::is_human(g, p)`).
- `rustfmt --edition 2024 <your new files>` only; never `cargo fmt` on the workspace.
- Commit often (`git add -A && git commit -q -m "..."`, inside the worktree); **no Co-Authored-By line**. Keep
  `rust/agents/handoff-c3-casual-speed.md` current. If your context gets long (~80+ tool calls), stop at a clean point with
  everything committed and end your reply with a first line `HANDOFF`.
- Finish with everything committed and the reply as above. Then stop.
