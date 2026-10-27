# Handoff: scion-goldfish

## Done
- Part 1, goldfish mode: `commander-sim --goldfish KEYS` (sim-cli/src/goldfish.rs; sim-core/src/goldfish.rs holds the
  inert seats). Seats 2-4 are `goldfish-2..4`: 40 life, a vanilla commander (Goblin Roughrider) and 99 Wastes; they
  keep seven, take real turns (untap, upkeep, draw, end step) but skip main phases, combat, the land drop and the
  end-of-turn window, and never block. Gates marked `// --- scion` in engine/turn.rs, engine/combat.rs, t6stats.rs.
  Tests: sim-cli goldfish::tests (inert seats never act, same seed same report, keys, summary maths), args test.

## Next
- Part 2: the `scion` deck key (deck file, my_decks.toml, cardgen deck/export/audit, key wiring, t6_plan_scion.rs).
- Part 3: Scion's copy, Prismari's storm and ward, the Scion AI, the lines, tests.
- Measure, FINDINGS, report.

## Gotchas
- Log lines start with the round (`R12    Goldfish 2 ...`): strip it before matching an actor.
- Wishclaw Talisman hands itself to an opponent: an inert seat can "gain control" of it (passive; it never activates).
- An inert seat's hand stays at seven: it draws in its draw step and discards at cleanup.
