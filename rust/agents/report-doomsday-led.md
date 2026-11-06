# Report: doomsday-led (2026-11-06)

Branch `doomsday-led` (worktree `.claude/worktrees/doomsday-led`). Brief `rust/agents/doomsday-led.md`. Not pushed, nothing merged.

## Rules checked against the Oracle text
- Lion's Eye Diamond: "{0} artifact. Discard your hand, Sacrifice Lion's Eye Diamond: Add three mana of any one color. Activate only as an
  instant." A mana ability (rule 605): it can be activated while paying for a spell that is already on the stack (601.2g), so
  that spell is not discarded. The sim cracks it exactly there: `mine::crack_led` discards every hand card except the spell
  being cast, sacrifices the diamond (to the graveyard) and adds three mana of one colour to the pool.
- Doomsday searches the library *and* graveyard, so the discards are in reach of the pile. Under Necropotence the discards are
  exiled (`discard_cards` already does this; the guide's mistake 6), so the builder leaves them out.
- Without a draw the pile cannot start: after a main-phase Doomsday the hand is empty, so the LED-paid cast is only offered at
  the end step (the draw step takes the first pile card), or when Thrasios's dig can draw.

## What was done
1. **LED pays Doomsday** (`impls/doomsday.rs`): `plan_cast` = `pre_checks` + `plan_mode(.., led)`; with the diamond on the table both
   the plain and the LED plan are built and the LED plan is taken only on a strictly better score (lock, backup, fewer spells), or
   when the plain cast has no pile. `build_led` builds the pile for the position it leaves (empty hand, the old hand in the graveyard,
   no {B} owed, lands free). The table-risk check is made with the hand gone (no held Silence, Chant, counter or Veil as cover).
   `go_doomsday` casts the enabler, then Doomsday, cracking the diamond for BBB with Doomsday already on the stack.
2. **LED inside the pile**: the search state has `led` (available) and `bonus` (colour of a crack's mana, counted once in the running
   mana total via `Env::okb`). `dfs_inner`/`try_card` crack it at the first spell (or Thrasios dig) the lands cannot pay,
   in {U}, {W} or {G}, discarding the rest of the hand; `run_pile` performs it. The Oracle in hand is only castable with the crack
   when it is the spell being cast; a held Veil / Revival goes with the hand.
3. **Cast from hand**: `data/plans/thrasios.toml`: set `led_fuel` (Doomsday and the tutors that can fetch it) and a prio row, value 90
   with one in hand, else 0.
4. `t6stats::makes_mana_now` -> `led_use_open` -> `led_uses` -> `doomsday::led_use` counts a Doomsday the diamond wins with as a use.
5. `audit/thrasios/FINDINGS.md`: one entry. The pilot guide says nothing the model contradicts (mistake 6 matches); not edited.

## Tests
`tests/doomsday_led.rs` (10): LED pays Doomsday with no black and the pile takes the discarded Oracle (end step, wins next turn);
the main phase has no such cast; no diamond, no cast; Necropotence blocks it; a crack inside the pile pays the Oracle after Silence
(as late as possible); no crack when the Oracle would go with the hand (and the crack that is fine); one crack pays a cantrip and
the Oracle; the AI casts LED with Doomsday or a fetching tutor in hand and holds it otherwise; the held statistic; the control
(plain cast is taken, the diamond stays).

## Checks run (release, memory cap, no sims)
`cargo build --release --all-targets`: no warnings. `cargo test --release --no-fail-fast` for doomsday_led, doomsday (incl. the
builder timing tests), thrasios_ai, t6_ai, t6_ai_b, c1_t6_lines, c1b_engines, ai_t6_*, engine_t6_*, veil_oracle, plans,
plan_files_3, generic_paths: all pass, 0 failed. The old hang repro (`--deck thrasios --pool t6 --ai lookahead --seed 500400
--trace 24`) finishes in 6 s.

## Not done / for JD
- The pilot guide has no line saying LED makes Doomsday's {B}{B}{B}; I did not edit JD's guide (its mistake 6 agrees with the model).
- Not modelled: LED mana for anything else on the Doomsday turn, a crack in response to something, LED's mana in the "Line 1"
  (Consultation / Pact) turns, and a Doomsday tax (Esper Sentinel / Thalia: the LED plan declines a cast with a cost above BBB).
- No sim was run: how often the sim now plays LED + Doomsday is unmeasured.
