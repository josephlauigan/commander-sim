# Port status

What's ported, what's waiting, and where each Python test went. The plan is `documents/rust-rewrite-scope.md`
(section 10); the design is `documents/rust-design.md`.

## Placeholders left

Code the Rust calls but hasn't ported yet is a placeholder in the module it will live in, marked with the
milestone that ports it. To list them: `grep -rn "PORT(\|HUMAN(" rust/crates/sim-core/src`.

| Marker | M2 | M3 | M4 | What |
|---|---|---|---|---|
| `PORT(M3)` | 18 | 0 | 0 | the ability language, the turn loop |
| `PORT(M4)` | 18 | 26 | 0 | the AI's choices: the main phase, defender choice, attack filters, tutor targets, protection, wipe answers, spell importance, scry |
| `PORT(M5)` | 58 | 110 | 130 | card code (`cardcode.rs`): Auras, regeneration, Marchesa, emblems, the Ring, Blood Moon, Plaza of Heroes, the combos, the outside decks' card plays (evoke, adventures, aristocrats), Underworld Breach ... |
| `PORT(phase 6)` | | | 17 | your other decks' AI (Sephiroth, Veyran, Galadriel, Y'shtola, Alela, Jodah): they cast by the outside decks' tag priority until then |
| `HUMAN(phase 9)`, `PORT(phase 9)` | 29 | 48 | 52 | a person's choices in practice mode (the AI's choice is used until then) |

## Python tests

| Python | Rust | Status |
|---|---|---|
| test_rules.Mana (6) | tests/rules.rs | ported |
| test_rules.StateChecks (5) | tests/rules.rs | ported |
| test_rules.Counterspells (3) | tests/rules.rs | ported |
| test_rules.Removal (5) | tests/rules.rs | ported |
| test_rules.TeferisProtection (5) | tests/rules.rs, tests/ai.rs | ported (the AI's two: lethal combat damage only, a pool deck against a wipe) |
| test_rules.Mulligan (1), Combat (4) | tests/rules.rs | ported |
| test_rules.Tutors (1) | tests/ai.rs | ported, with Sauron tutoring for the missing combo half |
| test_rules.DeathriteShaman, GameEndsMidEffect | | M5 (card code) |
| test_stack.Stack (5) | tests/stack.rs | 3 ported; the two with a person at the table in phase 9 |
| test_stack.Abilities (5) | tests/stack.rs | 1 ported; Azorius Guildmage with M5, a person's answers in phase 9 |
| test_triggers (12) | tests/stack.rs | 4 ported; card code (Tidebinder, Tithe Taker, Light-Paws, Moon-Circuit Hacker) with M5, a person ordering triggers in phase 9 |
| test_game_changers.Mana (4) | tests/rules.rs, tests/ai.rs | 3 ported (Ancient Tomb, Mox Diamond entering, Mox Diamond's priority); Gaea's Cradle with M5 |
| test_search (7) | tests/ai.rs | 6 ported (copies, hidden information, position scores, a whole decision, reproducible); "card definitions are shared" is the type system's job in Rust (`Arc<CardDb>`) |
| test_land_entry (11) | tests/land_entry.rs, tests/rules.rs | 9 ported (the entry rules, the AI's shock lands, gain lands); a person's shock land and Temples' scry in phase 9 |
| test_dsl.Interpreter (1) | tests/dsl.rs | ported, with 6 more interpreter tests on real cards; the compiler tests stay in Python (the compiler isn't ported) |
| (whole games) | tests/game.rs | Rust-only: every deck at every tier finishes, a seed replays the same game, a copied game plays on alone |

Rust-only tests: the unit tests in each module (cards, tags, random numbers, game state, the hook registry, life,
Python's float sum), `a_pact_is_affordable_from_everything_untapped`, `archons_trigger_drains_the_biggest_threat`,
and in tests/ai.rs the heuristic AI's choices (main-phase options, removal targets, Farewell's modes, Cyclonic Rift,
Disruptor Flute, deck plans), scry and surveil, and a whole game with the look-ahead.

## The differential harness (M4)

`python3 rust/tools/difftest.py` builds the same positions in both engines and compares what the AI reads from them:
the board reading (`Situation`), removal risk, the card it holds mana for, each candidate's cast priority and utility,
the tutor target, every main-phase option with its utility, and every seat's position score (`evaluate`,
`strength`), down to each permanent's value and power. The positions are a few hand-built ones plus random ones from
Sauron and Tier 1 (decks, lands, hands, graveyards, boards, life, turn; the library's order is fixed so both engines
draw the same cards); the Rust half is the example `difftest`. Nothing in it is random, so the numbers must match.

Differences are put down to a cause: card code (Python's `cards/impl`, ported in M5: the card is in one of
cardimpl's tables, a combo piece, an Aura or a transforming removal, or an option label from code that is still a
placeholder), phase 6 (your other decks' AI), random (a land search picked a different basic), or else the AI or the
position's building, which are bugs.

At the end of M4, on 3,004 fresh positions (seed 5): 931 match in full, 1,915 differ only where card code is
involved, 157 only through a random basic, 1 is a phase-6 deck, and **0 have an AI or building difference**. What
the harness found and fixed on the way:

- **Colour identity**: Rust took a seat's identity from its commander's card, which records none for Sauron,
  Sephiroth, Veyran, Atraxa and Tergrid, so their Command Tower, Arcane Signet and Chromatic Lantern lands made
  nothing. decks.json now carries each seat's identity as the Python engine sets it (`ident`), and seats use it.
- **Float sums**: Python 3.12's `sum()` adds floats with compensation, so a threat of 2.1 came out 2.0999999999999996
  in Rust and broke a tie for the leader the other way. `pysum::psum` reproduces it where the Python calls `sum()`.
- **Scry** (the last `PORT(M4)`): ported as `ai::topdeck` (desire, arrange, scry, surveil); a Temple's scry now
  orders the library as the Python does.
