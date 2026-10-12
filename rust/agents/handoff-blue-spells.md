# Handoff: blue-spells

## For elrond-core (Mindseeker Oculus)
`sim_core::impls::alela::empower(g, p, n)` (pub, already existed for Keeper of the Quiet Hour) adds n loyalty to p's
Jace token, making it first. Jace, Reality Sculptor (done, bulk_blue.rs) uses it. Keeper unchanged.

## Done
- Jace, Reality Sculptor (+ override in data/card_overrides.toml, tests).

## Next (in order)
Relm's Sketching, Echocasting Symposium, Phantom Interference, Whoosh!, Amazing Acrobatics, Icy Reception,
Sudden Setback, Vanish from Sight, Kasmina's Transmutation, Retrieve the Esper, Rousing Read, Unfathomable Truths,
West Coast Expansion, Sleight of Hand.

## Gotchas
- Audit status comes from data/card_overrides.toml (`dsl = []`, `tags`, `status = "Full"`, `note`), then `cardgen export`.
- Spells: `r.card(db, name)?.resolve = Some(f)` (f returns "gy"/"exile"/"handled"); `prio = zero_prio` keeps the generic AI
  from casting it; own plays via `hand_options` -> `Opt` with `Action::Plan{f,arg}` (see t2.rs value_blink).
