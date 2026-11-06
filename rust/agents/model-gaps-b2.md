# Task: a person's Rings of Brighthearth (and Peter Parker's Camera) can copy every activated ability (JD, 2026-11-05)

Worktree: `/home/joseph/projects/commander-sim/.claude/worktrees/model-gaps-b2` (branch model-gaps-b2, from model-gaps-b1).
Task name: `model-gaps-b2`. Read `rust/agents/report-model-gaps-b1.md` (the Rings row) and the doc comments of
`ability_window`, `ability_window_ex`, `ability_window_rc`, `rings_offer`, `ability_run`, `ability_run_w` in
`rust/crates/sim-core/src/engine/stack.rs` first.

**JD's rule: no gap may be left quietly.** Do not leave a site unconverted, add a "left out" note, or leave a caveat on your
own judgement. Anything you truly can't do, or a site you think should not be converted: list it at the top of your reply
under `NEEDS JD` (file:line, card, why, what converting it would take). JD decides.

## The gap
A person is offered Rings of Brighthearth's copy ("Whenever you activate an ability, if it isn't a mana ability, you may pay
{2}. If you do, copy that ability.") only for abilities whose code goes through `stack::ability_run` / `ability_run_w` /
`ability_window_ex` / `ability_window_rc` / `ability_window_card_ex` / `ability_window_card_rc`, which carry out the copies.
Person abilities that use plain `ability_window` or `ability_window_card` get no offer (Cid, Mind Stone, fetch lands, many
karn lands ...), and Peter Parker's Camera can't copy them either. In `rust/crates/sim-core/src/human/` there are 44
`ability_window(` and 23 `ability_window_card(` calls (cards.rs, karn.rs, gaps_c.rs, cloud.rs, urdragon.rs, gaps_a.rs,
abilities.rs, combos.rs, fetch.rs, model_gaps.rs).

## Do
1. Classify every one of those call sites first (a table in the report): card, ability, copyable activated ability (rule
   602 / 707.10) or not (a mana ability per rule 605, a special action, a trigger, a cost-only step...). Also check the
   person ability paths that call into `impls/` code which itself uses plain `ability_window` (grep from the `human/`
   dispatch): those count too.
2. Convert each copyable one to `ability_run` (or `ability_run_w` when the effect needs to know it is a copy): the cost is
   paid before, as today; the effect moves into the closure, which runs once per resolved copy and then for the original
   (if not countered). A copy pays no cost and **chooses its targets anew** (the person is asked again; rule 707.10c), and
   modes/X are copied as chosen. Effects that read a target chosen at activation must re-pick it in the closure for a copy
   (look at how A4's Siege-Gang person path and `ability_run_w` callers do it). Keep the original's behaviour and its
   questions identical.
3. AI paths stay unchanged: these are the person's paths (`human/`). If an `impls/` function shared with the AI must change,
   keep the AI's behaviour identical (`if is_human` split) and list it in the report.

## Tests
- `rust/crates/sim-core/tests/practice_ability_probe.rs` (or a new `practice_rings_probe.rs`): a sweep that, for every
  non-mana activated ability a person can use in the karn and thrasios decks (the decks with Rings), seats a person with
  Rings of Brighthearth and {2} spare, activates it, and checks the Rings question is asked and that a "yes" copy
  resolves. A new card that isn't offered fails the test; a listed exception needs a reason (and goes under NEEDS JD).
- `rust/crates/sim-core/tests/play_rings.rs`: spot tests: Cid, Mind Stone, a fetch land, a karn land ability (e.g. Urza's
  Saga or Mishra's Factory's pump), one ability with a target (the copy asks for a new one), one where the original is
  countered and the copy still resolves, and a Camera copy of a converted ability.
- Run: your test files, `play_rings`, `play_cards`, `play_abilities`, `play_karn_*`, every `practice_*` file, `model_gaps_*`,
  `play_cloud_karn_gaps`, and the test files covering any `impls/` code you touched.

## Done when
- Every site converted or listed under `NEEDS JD`; tests pass; `cargo build --release --all-targets` has no warnings.
- `rust/agents/report-model-gaps-b2.md`: the classification table, what changed per site, files, tests, every shared-file
  change.

## Ground rules (these override the older rust/agents/RULES.md, which is from the Python-port era)
- Work only in your worktree (path above); `cd` into it for every command. Don't merge other branches. Don't push.
  Toolchain: `export PATH=$HOME/.cargo/bin:$PATH`; build from `rust/`.
- **Never run Python, not even a version check** (a hook refuses it). Use jq, shell tools, or Rust.
- **Memory cap on every cargo command, no exceptions**: `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 cargo ...`
  (run from `rust/`). The shell is zsh: write the prefix out in full, don't store it in a variable. Exit 137 = cap hit = a bug.
- **Don't run the full suite** (`cargo test --release` with no filter) and don't run sims: the coordinator does that. Build and
  run only the test files named above (`cargo test --release -q -p sim-core --test <name>`). Run test commands in the
  foreground with a long timeout; don't start background jobs and don't leave anything running when you finish.
- Practice mode: the person's seat is `crate::human::is_human(g, p)`; questions go through `human::choices::{choose,
  choose_one, pick_cards, pick_any, yes_no, search, ...}`. AI and person code stay separate (`if is_human`).
- Never let an unsigned counter go below zero (release builds have overflow checks).
- `rustfmt --edition 2024 <your new files>` only; never `cargo fmt` on the workspace (the repo isn't fmt-clean).
- Oracle text: `jq '.["<lowercased name>"].oracle_text' data/scryfall_cache.json`.
- Commit often (`git add -A rust/ data/ && git commit -q -m "..."`); **no Co-Authored-By line**. Keep
  `rust/agents/handoff-model-gaps-b2.md` current (done, next, gotchas). If your context gets long (~60+ tool calls), stop at a
  clean point with everything committed and end your reply with a first line `HANDOFF`.
- Finish with: everything committed; full details in the report file (committed); a reply of at most 15 lines (commit hash,
  sites converted / listed, test results, anything the coordinator must act on). Then stop.
