# Handoff: scion-goldfish

## Done
- Part 1, goldfish mode: `commander-sim --goldfish KEYS` (sim-cli/src/goldfish.rs; sim-core/src/goldfish.rs holds the
  inert seats). Seats 2-4 are `goldfish-2..4`: 40 life, a vanilla commander (Goblin Roughrider) and 99 Wastes; they
  keep seven, take real turns (untap, upkeep, draw, end step) but skip main phases, combat, the land drop and the
  end-of-turn window, and never block. Gates marked `// --- scion` in engine/turn.rs, engine/combat.rs, t6stats.rs.
  Tests: sim-cli goldfish::tests (inert seats never act, same seed same report, keys, summary maths), args test.

- Part 2, the `scion` key: decklists/JD/scion-ur-dragon-turbo.md (v1, 20 Game Changers), my_decks.toml (t6),
  cardgen deck + export (Prismari and Tinder Wall fetched), key lists 12 -> 13 (sim-cli decks.rs / args.rs / poolmode
  test / report plan text, practice catalog + setup + 4 tests, game.rs 91), T6_KEYS 7 -> 8 (t6_plans dispatch,
  t6stats, ai_t6_table test), ai/mod.rs `T6_JD_STYLES` (scion 1.0 / 0.6 / 0.35), impls/t6_plan_scion.rs (keep only).
  Not added on purpose: settings.rs cthresh (a fixed 6 replaces the pool formula, ~3.5 for this counter-heavy deck)
  and rules.rs CTHRESH (it switches off the Veil of Summer response); MAIN_STYLES / jd_deck (switch off combos).

- Part 3: impls/t6_scion.rs (Scion becomes a copy via alela::set_face, copy_revert at turn start, stacked
  activations, Prismari's storm cast hook and ward, the Scion AI), impls/t6_combos_scion.rs (Prismari + Breach +
  Manamorphose + Brain Freeze), tests/cards_scion.rs (13). Shared edits marked `// --- scion`: state.rs DataKey,
  alela.rs set_face pub(crate), zones.rs phys commander, turn.rs copy_revert, removal.rs ward, t45_urdragon.rs key
  gate, combos.rs chain, impls/mod.rs. data/audit.toml notes for Scion and Prismari.

- Measured: audit/scion/FINDINGS.md, raw in audit/scion/2026-10-26-baseline/ (goldfish look-ahead scion + t6,
  goldfish adaptive scion, scion vs t6 loose look-ahead).

## Next
- report-scion-goldfish.md: the details are in the agent's final reply (the tool refused the report file from a
  subagent); the coordinator writes it.
- JD: the Tier 6 AI speed (every T6 deck kills at median own turn 7-14 in the goldfish), Smothering Tithe in the
  goldfish, the keep rule (D11).

## Gotchas
- Log lines start with the round (`R12    Goldfish 2 ...`): strip it before matching an actor.
- Wishclaw Talisman hands itself to an opponent: an inert seat can "gain control" of it (passive; it never activates).
- An inert seat's hand stays at seven: it draws in its draw step and discards at cleanup.
- The Prismari loop escapes Manamorphose only when mana runs low: each Manamorphose copy draws, and the library is
  both the draw pile and the escape fuel (an earlier version decked itself of fuel at storm ~10).
- A new combo is chained at the END of `combos::all()` (indices of the existing ones stay the same).
