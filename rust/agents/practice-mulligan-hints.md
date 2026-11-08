# Brief: practice mode's mulligan hints teach JD how to mulligan

JD approved this on 2026-11-07. Branch `practice-mulligan-hints`, cut from origin/main (0177d217 or later).

## Goal
When JD looks at an opening hand in practice mode, the hint should explain how to decide, not just give a verdict. It
says which rule actually decided, what the hand does in this deck's terms, and, for Tymna/Thrasios, what the
mulligan oracle measures for this exact hand.

## The bug to fix first
`rust/crates/practice/src/advisor/mulligan.rs::hint` describes the wrong rule for three of JD's decks.
- `engine::turn::keeps_hand` uses, in this order: the Tier 6 rule, then the deck's plan-file `[mulligan]` table
  (`sim_core::ai::plan_data::keeps_hand`, the tables in `data/plans/<deck>.toml`), then 2-5 lands.
- Tymna/Thrasios, Aragorn and Thanos have their own tables, for example Thrasios keeps a no-lander with 3+ fast mana and a
  tutor. But the hint says "keep 2 to 5 lands" and "the AI's rule does not look at key cards". Only Cloud's
  description is right (Y'shtola and Sheoldred: check whether they have a table).
- Fix: the hint must name the table row that decided, in plain English.
  - The row conditions are a small language: `hand-lands`, `fast`, `hand:tutors`, `hand:payoffs`, `hand:keep2` ...;
    see `plan_data.rs` around `Mulligan`, `keeps_hand` and the `hand:` set definitions in each plan file.
  - Add a function in `plan_data.rs` that returns which row fired (index plus its `when` text). Keep it additive, and
    don't change `keeps_hand`'s result.
  - Translate each condition into words: "no lands", "3 or more fast-mana pieces (Moxen, Sol Ring, rituals...)",
    "at least one tutor (Vampiric Tutor, ...)", naming the cards in this hand that count.
  - If no row fires, say what the default (`otherwise`) is.

## What the hint should show (keep it short and scannable, most important first)
1. **Verdict and the rule that decided it**, in one or two lines.
2. **The hand in the deck's terms**, naming the cards:
   - lands;
   - mana by turn 2, using `t6_keep`-style honest counting (Mox Diamond needs a spare land, Chrome Mox a spare card,
     rituals only fuel one spell);
   - tutors, win-line pieces (the plan file's payoff / combo sets), protection (free counters) and draw engines.
3. **Colours the plan needs.** Whether the lands and mana can make what the hand's key spells need. Examples: Doomsday's
   {B}{B}{B}, Ad Nauseam's {B}{B}, the commander's colours. Replace the generic "commander's colours are covered" line
   where a better one exists.
4. **Speed.** Does the hand hold a line: "both halves of Thassa's Oracle + Demonic Consultation", "one tutor away from
   the line", "no line and no tutor". Plus the turn the commander and the line can come online if every land drop hits.
5. **Tymna/Thrasios only: the oracle's numbers for this hand.**
   - Run the mulligan oracle's KEEP and MULLIGAN arms for this exact hand: `sim_core::mull_oracle::setup_playout` and
     the scoring in `sim-cli/src/mull_oracle.rs`. Move the shared playout/scoring code into sim-core so practice can
     call it, additively.
   - 40 playouts per arm, adaptive AI, paired seeds, run in parallel if that's easy. JD accepts about 1-2 seconds.
   - Show it as: "Kept, this hand kills on turn X on average in 40 goldfish games; mulliganed, turn Y. The oracle says
     keep / mulligan / too close to call."
   - Add one line on the limit: goldfish games have no opponents, so counters, removal and stax count for nothing.
   - Use the same seed for the same hand, so the numbers don't change when JD looks again.
   - Don't run it for the other decks yet: their goldfish kill is about turn 10 of a 10-turn cap, so it carries no
     signal. A later change (C5) gives them a real-game measure.
6. **After a mulligan, the bottom choice.** Say why the AI's pick goes to the bottom (the card the hand's plan needs
   least, and why), not only which one.

## Rules
- Follow `rust/agents/RULES.md` where it still applies. Ignore its Python, difftest and `rust-port` parts. **No Python.**
- Memory cap on every cargo/test/binary command: `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 <cmd>`.
- Files: `practice/src/advisor/mulligan.rs` (main), `sim-core/src/ai/plan_data.rs` (an additive "which row fired"
  function), `sim-core/src/mull_oracle.rs` (additive: the playout and scoring moved from sim-cli, which then calls it),
  and tests. Another agent is editing `sim-cli/src/mull_oracle.rs` at the same time to add a real-game arm. Keep your
  change to that file to moving the shared code out and calling it, so the merge stays simple.
- **The verdict (`choice`) must not change.** It is still `keeps_hand`. Only the explanation changes. `shadow.rs`
  uses `verdict()`, so keep it working.
- Tests (in `practice/tests/` or the module):
  - for each of JD's six decks, a hand where a table row decides, and the hint names that row and the counted cards;
  - a Thrasios hand whose hint includes the oracle numbers, deterministic (same output twice);
  - the existing `hint_every_decision.rs` still passes.
- No sims. Smoke-run only. The checks before finishing are `cargo test --release` (all pass) and
  `cargo build --release --all-targets` (no warnings).
- Commit (no Co-Authored-By line, don't push). Put the full report in `rust/agents/report-practice-mulligan-hints.md`,
  including three example hints, one each for Thrasios, Aragorn and Cloud, as text. Your final reply is at most 15 lines.
