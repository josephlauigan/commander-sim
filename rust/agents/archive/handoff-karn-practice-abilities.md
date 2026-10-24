# Handoff: karn-practice-abilities

Done: `human/karn.rs` lists every Oracle activated ability of the 23 audit cards (+ Batterskull's {3}, Walking Ballista's
counter removal) for the person; `abilities_of` / `abilities::land_abilities` call it; `owns_perm` / `owns_land` drop the
AI hook options for those cards (`alt::land_alts`, hook branch of `abilities_of`) so nothing shows twice.
`tests/play_karn_audit.rs` is a real test (conditions set per card); `tests/play_karn_abilities.rs` has 26 use tests.
Next: nothing outstanding except what the report lists as not modelled.
