# Tests

Run every test from the repository root:

```
python3 -m unittest discover -s tests -t .
```

That's 137 tests in about a minute. To run one file, or one test:

```
python3 -m unittest tests.test_my_cards
python3 -m unittest tests.test_my_cards.Sephiroth.test_massacre_wurm
```

The tests need no network access. The card data they use is in `data/scryfall_cache.json`.

## What each file covers

| File | Tests | What it checks |
|---|---|---|
| `test_rules.py` | 25 | Core rules on hand-built positions: mana colours and payment order, Sol Ring, Talisman pain, commander tax, the state-based losses (life, 21 commander damage, empty library, 10 poison), what each counterspell can hit and what it costs, Swords / Path / Bolt / token removal, Toxic Deluge, tutors, mulligans, commander damage, flying, deathtouch, lifelink. |
| `test_my_cards.py` | 50 | Key cards of your three decks against their Oracle text, and Sephiroth's four loops (Mikaeus + Triskelion, Mikaeus or Melira + Kitchen Finks, Nim Deathmantle + Ashnod's Altar + Grave Titan): when each kills, what stops it, and how the AI finds and assembles the pieces. Also Kefka, Brush Off, Melira, Avacyn's Pilgrim, Unsummon, Champion's Helm, Slaughter Pact, Urabrask, and the Sauron AI's priorities for Sheoldred and Consecrated Sphinx. |
| `test_search.py` | 7 | The look-ahead AI: game copies are independent, hidden hands are re-dealt correctly, a win always outscores a board, and a whole decision picks one of the options, leaves the real game untouched, and is reproducible. |
| `test_dsl.py` | 5 | The ability language: Oracle text compiles to the expected abilities, unreadable text is reported, and a compiled card works in a game. |
| `test_cli.py` | 14 | The Wilson interval and the paired difference, and every command run the way you run it (`python3 -m ...`) with a few games: a tier run, `--swap`, `--analyze`, `--trace`, `--calibrate`, `--cards`, the validator, the audits and `tools.swaptest`. Also checks that `--jobs 1` and `--jobs 3` give the same result. |
| `test_update_deck.py` | 10 | The deck updater: list formats from deck sites, name matching, a swap rewriting every section, bad lists refused with nothing written, a pool deck's Game Changer rule and header, and `--dry-run`. |
| `test_pool_games.py` | 2 | Seeded games from every tier, alone and with each of your decks, play to the end; one game with the look-ahead AI. |
| `test_pool_sampling.py` | 8 | Seating: opponents drawn without replacement, seeded and uniform. Pairing: a changed list faces the same opponents, seats and draws; games replay exactly. |
| `test_validator.py` | 15 | The decklist checks: size, singleton, commander, name resolution, colour identity, bans, Game Changers per tier. |
| `test_my_decks.py` | 1 | Your deck files parse to the lists recorded in `fixtures/my_decks_parsed.json`. |

## When a test fails

- **`test_my_decks` fails after you edit a deck.** That's expected: it guards against accidental edits. Once the
  new list is what you want, rewrite the fixture:

  ```
  python3 tests/test_my_decks.py --record
  ```

  `python3 -m commander_sim.update_deck` records it for you when it updates a list.

- **A card test fails.** A card test failing means the simulator no longer does what the card's Oracle text says.
  Fix the card's implementation, not the test, unless the test itself misread the card.

## Coverage

```
python3 -m commander_sim.tools.linecov                     # per-module table and total
python3 -m commander_sim.tools.linecov --missing engine    # also the lines no test runs, for matching modules
```

The tool runs the whole suite while recording which lines of `commander_sim` execute. That includes the
subprocesses the command-line tests start, and their worker processes. It takes about a minute, uses only the
standard library, and exits with an error if a test fails.

Line coverage was 86% in September 2026. It shows which code the tests reach, not whether they check its result:
the end-to-end games reach most of the game code, but only the rule and card tests check what it does.

## Writing a rule or card test

`tests/table.py` builds a position without playing a game:

```python
from tests.table import table, hand, lands, perm, token
from commander_sim import engine as E

g = table('seph', 'veyran')                 # seats, in turn order; hands empty, life 40, first seat active
s, v = g.players
lands(v, 'Island', 2)                       # untapped lands
hand(v, 'Counterspell')                     # cards into a hand (taken from that deck's library if it runs them)
perm(g, s, 'Sheoldred, the Apocalypse')     # onto the battlefield: its enter effects happen
E.draw(g, v, 1)                             # then drive the engine directly
assert v.life == 38
```

Useful engine entry points:

- `E.draw`, `E.on_cast` (cast triggers), `E.magecraft`;
- `E.apply_removal`, `E.apply_wipe`, `E.die`, `E.check_state`;
- `E.pay` and `E.can_pay`;
- `ais.resolve_combat` (attackers against one defender), `ais.attack_triggers`, `ais.upkeep`.

Take the expected numbers from the card's Oracle text. Prefer situations where the AI has no real choice, so
the result doesn't depend on its randomness. Blocks, for example, are only random for chump blocks.
