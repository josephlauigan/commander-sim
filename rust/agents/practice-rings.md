# Task: practice-rings — Rings of Brighthearth and the Basalt Monolith loop for the person in practice mode

Read `rust/agents/RULES.md` for the working rules (commit often, handoff file, 15-line report), but these override it:
your worktree is `.claude/worktrees/practice-rings` (branch `practice-rings`, already on origin/main; don't merge
`rust-port`, it is gone), there is no Python (never run it), and the memory cap is
`systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 <command>`. Build with `cargo build --release`
and run only the targeted tests you add or touch (`cargo test --release -p practice --test <name>`,
`cargo test --release -p sim-core --test <name>`). Don't run the full suite or any sim: the coordinator does.

## The bug (JD, 11/03)

JD played Tymna/Thrasios in practice mode with Basalt Monolith, Rings of Brighthearth and Walking Ballista and could
not combo. Basalt's `{3}: untap` works for the person (it is a compiled ability: `dsl::activate`, `dsl/mod.rs` ~2279),
but nothing ever offers Rings' copy, and clicking the Rings says "has no ability you can activate right now". The
bots do the combo only as an abstract readiness check (`impls/combos.rs` `power_artifact` ~1176,
`impls/karn_engines.rs` `infinite_mana` ~872). Both of JD's decks `karn` and `thrasios` run the combo.

## Do both

1. **Rings of Brighthearth, by the rules, for the person.** Oracle: "Whenever you activate an ability, if it isn't a
   mana ability, you may pay {2}. If you do, copy that ability. You may choose new targets for the copy." When a
   human seat that controls Rings (on the battlefield, not phased out; one ask per Rings) activates a non-mana
   ability, ask "Rings of Brighthearth: pay {2} to copy <ability>?"; if yes and payable from the pool/sources the way
   other human costs are paid, the copy goes on the stack above it and resolves first. Machinery already exists:
   `engine/stack.rs` `copy_trigger` / `copyable` / `ability_window_ex` (copies counted in `AbilityRes.copies`,
   resolved by the caller, e.g. `dsl::activate`), used by Peter Parker's Camera (`impls/t6_copies.rs`
   `camera_copy`). Cover at least: compiled abilities (`dsl::activate`, incl. Basalt's untap and Walking Ballista),
   loyalty abilities (`dsl::use_loyalty` / `human/cards.rs` ~395: Karn −3, Ugin +2 in JD's Karn notes), and the
   practice abilities in `human/karn.rs` and `human/abilities.rs` that use plain `ability_window` (Grim Monolith
   untap, Staff of Domination, Thrasios's ability, Walking Ballista's ping, fetch lands if they're abilities there).
   Where an ability can't be copied without a big rework, list it in your report rather than faking it. Mana
   abilities are never copied. A countered original still lets its copy resolve (they're separate objects).
   The AI does not change: gate everything on `crate::human::is_human`, so sims are byte-identical.
2. **A table shortcut for the loop**, like Sephiroth's (`human/abilities.rs` `loop_abilities`, `impls/mine.rs`
   `run_loop`): when the person controls Basalt Monolith and Rings of Brighthearth and can start it (the same test as
   `power_artifact`: {5} in all, i.e. Basalt untapped plus {2} more, or {5} from elsewhere when Basalt is tapped),
   Basalt and the Rings each list "start the loop: infinite colourless mana". It pays the start-up mana, gives the
   opponents one window to answer (as an ability on the stack: a Stifle-type answer or removal on a piece in
   response stops it; reuse what `run_loop` / `combo_interrupted` do), then asks the person how much {C} they want
   (a number, with a large default such as 999) and adds it to their mana pool, tapping Basalt at the end as the
   real loop leaves it. In the Karn deck colourless mana from artifacts goes to `ManaPool::art` rules
   (`impls/karn.rs` header): follow what Basalt's own tap does. The pool empties as usual at the end of the step.
   Then the person spends it with the normal actions, so make the outlets practical with a huge pool:
   - Walking Ballista: casting it asks for X (check it accepts a big X), and its "remove a counter: 1 damage" must
     let the person remove several counters at once (ask how many, and the target, e.g. "each opponent: 40 each" is
     not needed, but one ask per target with a count is).
   - Staff of Domination: `{1}: untap`, `{5},{T}: draw` — offer "draw N cards" (untap + draw N times, asks N), or
     at least each step works and costs right.
   - Thrasios ({4}: scry 1, then reveal the top card; a land goes onto the battlefield tapped, else into the hand):
     check it can be activated repeatedly by the person.
   Log each step in the game log the way other human actions are logged.

## Tests

Add a practice-mode test (look at `crates/practice/tests/session_basic.rs` / `session_common` and the human tests in
`crates/sim-core/tests/` that drive a human seat, e.g. grep `human` in tests) that sets up Thrasios's seat with
Basalt, Rings, lands and Walking Ballista in hand and checks: (a) activating Basalt's untap asks to copy with Rings
and the copy untaps it (net +1 {C} per full loop), (b) the shortcut adds the chosen {C}, (c) Ballista cast for a big
X and pinged kills the table. One for the Karn seat (artifact mana pool) too.

## Report

`rust/agents/report-practice-rings.md`: what each ability does now, every human ability path you hooked Rings into
and the ones you couldn't, shared files you touched. Short report back (≤15 lines). Commit; don't push.
