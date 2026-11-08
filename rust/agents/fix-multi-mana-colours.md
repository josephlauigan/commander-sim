# Brief: one source of several mana pays only the colours it can make

JD approved this on 2026-11-08. Branch `fix-multi-mana-colours`, cut from `fix-combo-colour-pay` at c20155c2 or later
(PR #305, not merged yet: it makes ritual mana typed).

## The gap
In the mana-unit model (`engine/mana.rs`: `mana_units`, `plan_pay`, `float_leftover`; and how `can_pay`/`pay` consume
units), one permanent that makes several mana can pay a split of colours it could never make:
- **"N mana of any one colour"** (Gilded Lotus, Lotus Field, Black Lotus): three units of `Any` can pay {W}{U}{B}. It must
  be N mana of a single colour chosen at payment.
- **A fixed pair** (Karoo bounce lands such as Golgari Rot Farm {B}{G}, and the Signets such as Golgari Signet {B}{G}): it
  must pay exactly that pair, not {B}{B} or {G}{G}. Two-mana colourless sources stay colourless.

JD's decks run **Gilded Lotus (Thanos)**. The rest are opponents' cards: grep `decklists/pool` and `data/cards.json`
for the full list, and handle every card with this shape, not only the ones named here.

## What to do
1. Tests first:
   - Gilded Lotus can't pay {W}{U}{B}, but can pay {U}{U}{U} or {2}{U};
   - a Karoo can pay {B}{G} but not {B}{B};
   - Golgari Signet the same.
   Each must fail on the base branch.
2. Fix the unit model at its root: a unit group with a "single colour for all" or "exact colour set" constraint,
   respected by `plan_pay` (the AI's planner), by `pay`, and by the floating of leftovers (`float_leftover`, so a Lotus
   tapped for {U}{U}{U} floats U).
   - Keep it general: tag-, DSL- or data-driven rather than a list of names where the data allows. Where the data has
     no field for it, add one in cardgen the way `add_mana` got `color` on this branch.
   - Keep performance in mind: `plan_pay` runs in the look-ahead's inner loops. Time a goldfish smoke run before and
     after (`--goldfish thanos --games 50 --ai adaptive`) and report both times.
3. **Practice mode:** when the person gets mana from Carpet of Flowers, The Aetherspark or any "N mana of any one
   colour" source (Gilded Lotus ...), ask them which colour (a choice of their identity's colours), then float N of
   that colour. `engine/mana.rs::choose_one_colour` currently returns `Kind::Any` for a person; change that path. Look
   at how other practice choices are asked (`human/choices.rs`) and add a practice test for it.

## Rules
- Follow `rust/agents/RULES.md` where it still applies. Ignore its Python, difftest and `rust-port` parts. **No Python.**
- Memory cap on every cargo/test/binary command: `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 <cmd>`.
- This changes game behaviour on purpose. For each test that relied on the old leniency, fix the test with a comment
  and list it in the report.
- No sims beyond the two goldfish timing smoke runs.
- Before finishing: `cargo test --release` passes and `cargo build --release --all-targets` has no warnings.
- Commit (no Co-Authored-By line, don't push). The full report goes in `rust/agents/report-fix-multi-mana-colours.md`. If
  writing it is refused, put up to 25 lines in your reply. Otherwise your final reply is at most 15 lines.
