# Task: cand-torbran — model the Torbran candidate deck and write its AI plan

Base branch: `rework-candidates`. Deck: `decklists/JD/candidates/torbran.md` (key `torbran`): mono-red tokens, pingers
and burn under Torbran, Thane of Red Fell. The deck is already wired into every deck list; your module is
`rust/crates/sim-core/src/impls/torbran.rs` (stub in place), tests in `rust/crates/sim-core/tests/cards_torbran.rs`.

## Cards to model (until `cardgen audit --mine` shows each as Full)
- **Torbran, Thane of Red Fell:** *if a red source you control would deal damage to an opponent or a permanent an
  opponent controls, it deals that much damage plus 2 instead.* Every red source: spells, abilities, combat damage
  from red creatures, and red tokens. Not damage to you or your own permanents. Find the engine's single damage path
  and add it there.
- Impractical Joke (unmodeled), Borderland Marauder, Goblin Medics, Godo's Irregulars, Karplusan Wolverine, Rift Bolt
  (suspend).

## AI plan for torbran
- Count Torbran's +2 when choosing burn targets: send burn at a player when it kills or nearly kills, at creatures
  when it clears a blocker or a threat.
- Pingers (Deadeye Duelist, Viashino Fangtail, Goblin Medics, Bomber Corps) aim at opponents' faces with Torbran out.
- Go wide, then attack: tokens are 3-damage hits each with Torbran out.
- Cast Torbran early (turn 4 on curve) and keep him protected (Champion's Helm, Darksteel Plate).

Smoke run: `--deck torbran`.
