# Report: Phase 4 PR 6, rule checks in the policy

Brief: `rust/agents/phase4-rules.md`. Design and survey: `audit/phase4/rules-design.md`. Handoff:
`audit/phase4/rules-handoff.md`. No sims (JD, 2026-10-31): ships off by default.

## What it does

Three generic guards in `ai/rules.rs`, behind `AI_RULES` (unset or `0` off, the default; `1` all; or a list of
`followup`, `tutored`, `combo`), plus `AI_RULES_DECKS=a,b` to guard only those decks' seats (for half-tier tests).
People in practice mode are never guarded.

- **followup:** a spell is refused when its upkeep payment (Pact of Negation's {3}{U}{U}, plus every Pact debt already
  owed) can't be paid on the next upkeep, or when a tagged discard / sacrifice or compiled additional cost can't be
  paid. Next upkeep's mana: a copy of the game with p's permanents untapped and not summoning-sick, through `can_pay`
  (lands, rocks, dorks, Treasures; not the land drop, hand or rituals).
- **tutored:** a card a tutor found is marked for this turn and the next; hand-size discard, discard costs, Kefka's
  discard and scry / surveil set it aside while another card is available.
- **combo:** a spent combo piece (Demonic Consultation, Tainted Pact, Dramatic Reversal, Brain Freeze, the Dualcaster
  copies) is refused unless each other piece is held or findable by a tutor in hand p can pay for now.

## Findings

- A real bug, fixed only with the guard on: outside Tier 6, Pact of Negation is cast as a counter when its upkeep
  can't be paid (its {0} cost passes the payment check), and the AI loses at its next upkeep.
- Deadly Dispute's sacrifice cost is never asked for by the engine (`fill = "dispute"`): left for card models.

## Checks (coordinator, 2026-10-31)

- Read the hooks: two one-line calls in `ai/brain.rs` (`main_options`, `end_of_turn_window`), the counter pick, the
  discard / scry sites and five tutor sites; every entry (`filter_options`, `followup_ok`, `mark_tutored`,
  `spare_tutored`) returns before reading or changing the game when its guard is off.
- `cargo build --release --all-targets` 0 warnings; `cargo test --release --no-fail-fast` 3748 passed, 0 failed
  (14 new tests in `tests/rules_guards.rs`).

## For the recalibration

The byte-identity check with `AI_RULES` unset (t1-t5, 300 games), the half-tier pairs, JD's decks on their own tiers
(urdragon, thanos, kefka, sheoldred, ghave most affected), and one guard at a time if a pair moves: design section 5.
