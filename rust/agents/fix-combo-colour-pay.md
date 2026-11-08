# Brief: combo lines must pay coloured costs with the right colours

JD approved this on 2026-11-08. Branch `fix-combo-colour-pay`, cut from origin/main.

## The bug (found by the coordinator, reproduced)
A goldfish game of Tymna/Thrasios (`thrasios`) with the opening hand Tundra, Watery Grave, Underground Sea, Sol Ring,
Vampiric Tutor, Thassa's Oracle, Demonic Consultation, kept, adaptive AI, plays like this:
- Turn 1: Watery Grave.
- The goldfish's end step: Vampiric Tutor for Dark Ritual (paid with Watery Grave's {B}).
- Turn 2: Tundra; "casts Dark Ritual for Thassa's Oracle + Demonic Consultation / Tainted Pact"; Dark Ritual; Thassa's
  Oracle; Demonic Consultation; it wins.

That is illegal. On turn 2 there are two lands, Watery Grave and Tundra, and Sol Ring was never cast. Watery Grave's {B}
pays for Dark Ritual and makes {B}{B}{B}, which leaves Tundra's {U}. Thassa's Oracle costs {U}{U} (`data/cards.json`:
pips "UU"), and Demonic Consultation {B}. So {U}{U}{B} was paid with {B}{B}{B}{U}.

The playout setup to reproduce it: `sim_core::mull_oracle::setup_playout` with `Arm::Keep { hand, tries: 0 }`, the deck
plus `sim_core::goldfish::inert_seats`, adaptive AI, name `trace:0`, diag seed 0. Then `engine::turn::run_rounds(g, 10)`
with `g.log = Some(vec![])`. Building the game: see `practice/tests/mulligan_hints.rs::game_with` or the practice advisor's
`oracle_uncached`.

## Where to look
- `impls/combos.rs`: `cast_rituals_for` (~L495), which loops on `can_pay(g, p, gn, &pips, false)` after `mana_need`.
  Also `mana_need` itself, and the step runner that then casts each spell of the line (how is each step paid?).
- `engine/mana.rs`: `can_pay`, `pay`, `plan_pay` and `mana_units`. How is floating mana (a ritual's {B}{B}{B}) represented
  for an AI player, and can it pay a {U} pip? Is any unit "any colour" when it shouldn't be?
- How Dark Ritual's resolution adds mana for an AI player.

## What to do
1. **Test first.** Write a failing test (sim-core tests) for this exact case: either this hand on turn 2 must not win,
   or, more direct, a unit test that the line can't be paid with {B}{B}{B} floating plus Tundra.
2. **Find the root cause and fix it there.** Don't special-case Thassa's Oracle. A correct fix makes every coloured cost
   in combo lines and ritual-fuelled casts respect colours.
3. **Audit the same gap elsewhere:**
   - the other places that check a line's mana with floating ritual mana or "fast" mana: `affordable`, the T6 line
     modules `impls/t6_combos_*`, `impls/breach_line.rs` (it tracks red vs black ritual mana: `line_red`),
     `impls/doomsday.rs` (Doomsday's {B}{B}{B} and its pile);
   - the planner/look-ahead's mana estimates, if they gate a "wins this turn" decision.
   Fix what's wrong, with a test each. List anything suspicious you didn't fix.
4. **Keep the rules faithful:** real Magic mana rules (a ritual adds coloured mana; a land taps for one of its colours).
   Don't break legitimate lines, for example Lotus Petal any colour, Chrome Mox an imprinted colour, Mana Confluence or
   Command Tower any of the identity's colours, Birds and Noble Hierarch.

## Rules
- Follow `rust/agents/RULES.md` where it still applies. Ignore its Python, difftest and `rust-port` parts. **No Python.**
- Memory cap on every cargo/test/binary command: `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 <cmd>`.
- This changes game behaviour on purpose, so existing tests that relied on the bug may fail. For each one, decide whether
  the test was wrong (the bug) and fix the test with a comment, or whether your fix is too strict. List each in the report.
- **No sims or measurement runs.** A smoke run of `--goldfish thrasios --games 50 --ai adaptive` before and after is
  fine, to show the kill-turn shift (report both).
- Before finishing: `cargo test --release` passes and `cargo build --release --all-targets` has no warnings.
- Commit (no Co-Authored-By line, don't push). The full report goes in `rust/agents/report-fix-combo-colour-pay.md`. If
  writing that file is refused, put up to 30 lines in your reply. Otherwise your final reply is at most 15 lines: the
  commit hash, the worktree path, the root cause in one or two sentences, what you fixed, the tests changed and why, the
  goldfish before/after, and anything left.
