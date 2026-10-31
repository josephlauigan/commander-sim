# Brief: Phase 4, PR 7: each deck's hand-written AI behind a switch, off by default

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/phase4-hand-off` on
branch `phase4-hand-off`, from origin/main. Run every command from there. Read `CLAUDE.md` and `rust/agents/RULES.md`
first: memory cap, no Python. **Another agent works in parallel** (worktree `phase4-rules`, the rule checks): use
`-j 12` for cargo, in the background.

## No sims (JD, 2026-10-31)

**Run no `commander-sim` of any kind**, not even a short byte-identity or timing check, and no Rust example that
plays games. Checks are cargo tests only. The recalibration after Phase 4 will measure each deck with its hand AI on
and off.

Waiting on background work: wait for the task's completion notice, or on an output file. Never wait with
`until ! pgrep -f ...` (it matches its own command line and never ends), and don't leave a wait loop running when its
job is cancelled.

## JD's decision (2026-10-31)

Phase 4's last step deletes the per-deck AI ("the per-deck priority functions, wish lists and plan files, deck by
deck, each replacement checked with a paired test... Keep a deck's hand code only where its paired test shows the
generic policy is worse"). With no sims, JD chose: **put each deck's hand AI behind a switch that defaults to the
generic policy now; delete it later**, after the recalibration's on/off test for that deck. Nothing is deleted in
this PR.

## Work

1. **Survey** (`audit/phase4/hand-off-design.md`, commit before code): every deck with hand-written AI, JD's and the
   pool's. JD's: veyran (`ai/veyran.rs`), thanos and the Tier 6 table (`ai/t6_table.rs`, `impls/t6_plans.rs`,
   `t6_combos_*`), and any deck-keyed AI in `impls/` (alela, cloud, galadriel, karn, kefka, sheoldred, sauron,
   yshtola, thrasios / `tymna_partners`, urdragon); the pool's (Sythis in `t12_kaalia_sythis.rs`, and others). Find
   them by deck-key matches (`key == "..."`, `KEY` constants, `ai/decks.rs` dispatch, `is_main`, `jd_deck`, `pl.t6`)
   and by per-deck priority functions, wish lists, mulligan and hold rules. For each site, class it: **deck AI** (a
   choice the generic policy also makes: what to cast, hold, tutor, mulligan, attack, target) or **card code** (how a
   card works: stays as it is). List them per deck.
2. **Switch** `AI_HAND`: unset or `0` = the generic policy for every deck (**the new default**), `1` = every deck's
   hand AI as today, or a list of deck keys that keep their hand AI. One function (e.g. `ai::hand_ai(g, p)`) answers
   it, and every deck-AI site asks it. With the hand AI off for a deck:
   - the deck is a generic-policy deck for `values::generic_deck` (so derived values, the learned evaluation and
     ISMCTS apply to it; that file is Phase 4's, but this one-line change is yours: list it);
   - it plays from its plan file (`data/plans/<key>.toml`); a deck with no plan file gets one, written from what its
     hand AI encodes (lines, key cards, holds, mulligan pieces), so that knowledge isn't lost.
   - **Tier 6:** the Tier 6 table code (`t6_table`, `pl.t6`) is the table's; decide in the design whether its pool
     decks move to their plan files too (PR 2 wrote plan files for them) and say why.
3. **Tests:** tests of a deck's hand AI keep passing with the hand AI forced on in that test (a thread-local
   override, as `ai/ismcts.rs` does with `FORCE`); tests of card behaviour stay as they are. Add one test per deck that
   with the switch off, the deck's hand-AI entry points are not reached (or the deck decides through the generic
   policy), and that with `AI_HAND=1` they are.
4. **For the recalibration**, list in the design doc, per deck, the on/off paired test to run (JD's decks on their own
   tiers; pool decks by the half-tier method in `audit/phase4/values-handoff.md`) and the rule: delete a deck's hand
   AI where off is level or better; otherwise improve its plan file, and keep the code only if the gap can't be closed.

## Checks

- `cargo test --release --no-fail-fast -j 12` passes (all crates, practice included); `cargo build --release
  --all-targets -j 12` 0 warnings; `cardgen validate` passes; the plan-file test passes.
- JD's decks seated only in their own tiers in new tests; don't seat `omnath` or `godo` (the pool deck).

## Rules

- Stay out of `ai/search.rs`, `ai/ismcts.rs`, `ai/learned.rs`, `data/eval/` (one-line changes forced by the switch:
  make them and list them). Keep edits to `ai/brain.rs` to the switch checks (the other agent edits it too).
- No deck list changes, no recalibration, no style values. Delete nothing.
- Format only new files (`rustfmt --edition 2024 <file>`), lines up to 120. Commit often, no Co-Authored-By line,
  never push. Scratch files in
  `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/phase4-hand-off/`.
- Keep `audit/phase4/hand-off-handoff.md` current.
- Report in 15 lines or fewer, then the full report as text in your reply (the harness refuses report files from
  subagents): the per-deck survey (deck AI vs card code), the switch, the plan files written, the tests, and the
  recalibration list.
