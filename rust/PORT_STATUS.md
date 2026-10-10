# Port status

What's ported, what's waiting, and where each Python test went. The plan is `documents/rust-rewrite-scope.md`
(section 10); the design is `documents/rust-design.md`.

## Placeholders left

Code the Rust calls but hasn't ported yet is a placeholder in the module it will live in, marked with the
milestone that ports it. To list them: `grep -rn "PORT(\|HUMAN(" rust/crates/sim-core/src`.

| Marker | M2 | M3 | What |
|---|---|---|---|
| `PORT(M3)` | 18 | 0 | the ability language, the turn loop |
| `PORT(M4)` | 18 | 26 | the AI's choices (`ai.rs`): the main phase, defender choice, attack filters, tutor targets, protection, wipe answers, spell importance, scry |
| `PORT(M5)` | 58 | 110 | card code (`cardcode.rs`): Auras, regeneration, Marchesa, emblems, the Ring, Blood Moon, Plaza of Heroes ... |
| `HUMAN(phase 9)`, `PORT(phase 9)` | 29 | 48 | a person's choices in practice mode (the AI's choice is used until then) |

## Python tests

| Python | Rust | Status |
|---|---|---|
| test_rules.Mana (6) | tests/rules.rs | ported |
| test_rules.StateChecks (5) | tests/rules.rs | ported |
| test_rules.Counterspells (3) | tests/rules.rs | ported |
| test_rules.Removal (5) | tests/rules.rs | ported |
| test_rules.TeferisProtection (5) | tests/rules.rs | 3 ported (with "until your next turn"); lethal combat damage and the pool deck's wipe answer with M4 |
| test_rules.Mulligan (1), Combat (4) | tests/rules.rs | ported |
| test_rules.Tutors | | M4 (tutor targets) |
| test_rules.DeathriteShaman, GameEndsMidEffect | | M5 (card code) |
| test_stack.Stack (5) | tests/stack.rs | 3 ported; the two with a person at the table in phase 9 |
| test_stack.Abilities (5) | tests/stack.rs | 1 ported; Azorius Guildmage with M5, a person's answers in phase 9 |
| test_triggers (12) | tests/stack.rs | 4 ported; card code (Tidebinder, Tithe Taker, Light-Paws, Moon-Circuit Hacker) with M5, a person ordering triggers in phase 9 |
| test_game_changers.Mana (4) | tests/rules.rs | 2 ported (Ancient Tomb, Mox Diamond entering); Gaea's Cradle with M5, Mox Diamond's priority with M4 |
| test_land_entry (11) | tests/land_entry.rs, tests/rules.rs | 9 ported (the entry rules, the AI's shock lands, gain lands); a person's shock land and Temples' scry in phase 9 |
| test_dsl.Interpreter (1) | tests/dsl.rs | ported, with 6 more interpreter tests on real cards; the compiler tests stay in Python (the compiler isn't ported) |
| (whole games) | tests/game.rs | Rust-only: every deck at every tier finishes, a seed replays the same game, a copied game plays on alone |

Rust-only tests: the unit tests in each module (cards, tags, random numbers, game state, the hook registry, life),
`a_pact_is_affordable_from_everything_untapped`, `archons_trigger_drains_the_biggest_threat`.
