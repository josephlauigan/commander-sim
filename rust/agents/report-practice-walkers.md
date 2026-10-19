# Report: practice-walkers

## Done (step 1): the person's creatures can attack planeswalkers
- `human/combat.rs::human_attack`: after the attackers are picked, if an opponent controls a planeswalker (alive,
  on the battlefield, not phased out) the person is asked "Attack which player or planeswalker with N creature(s)?"
  (kind `target`): players first, then walkers labelled like `Vraska, Betrayal's Sting (Alice) (planeswalker, 6 loyalty)`
  (describe_target's text plus the loyalty, so `table_refs` still finds it on the page). Split gives each creature its
  own player or walker. With no walker on the table the flow and questions are exactly as before.
- Walker attackers are put in the group of the walker's controller (so that player blocks them, attack triggers work)
  and recorded in the new `Game::human_walker_atk` (state.rs, additive). Cleared at every human attack declaration.
- `engine/combat.rs` (shared, person-only branches): `to_walker` = `g.human_walker_atk` for a person instead of an
  empty list, so the existing walker-damage code applies (loyalty drops, walker dies to the graveyard, lost_names);
  plus one added guard: if a person's attacker's walker is gone by damage time, no damage goes to the player.
  The AI branch (`walker_attacks`) is untouched.
- Page and text client need no change: they render `target` choices as given (refs by prefix match).
- Tests: `tests/play_walker_attacks.rs` (attack walker, attack player with walker present, split, kill walker,
  no walker -> no extra question).

## Not done (steps 2 and 3): several blockers on one attacker, damage assignment order
The engine's damage step (`resolve_combat_inner`) is built on one blocker per attacker: `blocker_of` takes the first
`(a, b)` pair and the ~90-line pairing block (first strike, deathtouch, trample, Kaldra, Cecil, Hawkeye ...) works on
a single `b`. The AI never makes multi-blocks either (menace spends a second blocker that deals no damage). Making
several blockers deal and take damage properly means restructuring that block (and ordering/trample assignment), which
the brief says not to do. Left as is; the module doc in human/combat.rs still states the limitation.
