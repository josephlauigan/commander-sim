# Report: Phase 2, PR 3: export refuses doubles; coverage per deck (branch `phase2-coverage`)

Brief: `rust/agents/phase2-coverage.md`. Handoff: `audit/phase2/coverage-handoff.md`. Base: origin/main 2bdfead0.
Commits: 440d23da (guard), faab5c92 (coverage, docs), 0cd7c4f1 (handoff), plus this report.

## The Phase 2 gate is not passed
There are 0 hard doubles. But the data had **25 shadowed, 21 competing and 161 undeclared splits** with no `paths_ok` (29 splits are declared). The brief assumed only the shadowed ones were left. Refusing every one would have meant writing new `paths_ok` notes for all of them before the current data could pass. So the 207 cards are frozen in `data/paths_known.txt`, one "class | card" line each, and that list is what is left to clear before "no double-modeled cards" holds. The session coordinator accepted this on 2026-10-29, on condition that the list only shrinks. JD has not been asked.

## 1. `cardgen export` refuses a new double
- `cardgen::paths::guard` runs in `export::run` after the cards are built and before anything is written. It reads the registry the same way `audit --paths` does: `audit::engine_db_from` builds sim-core's own `CardDb` and `impls::registry` from the cards.json text. It is a reuse, not a copy.
- It refuses any hard, shadowed or competing double, and any undeclared split, unless the card has `paths_ok` or is in `paths_known.txt` with the same class. A hard double is never accepted from that list. A known card that changes class counts as new.
- Ratchet: the test fails on a stale line (a card that is no longer that double, or no longer exists), and the message says to delete the line. `cardgen export` prints the same thing as a note.
- Tests (`rust/crates/cardgen/tests/paths.rs`):
  - `the_export_guard_passes_the_committed_data` runs the check on the committed data, so `cargo test` fails without a re-export.
  - `the_export_guard_refuses_an_undeclared_double` covers Hexing Squelcher without its `paths_ok`, a hard double named in the known list, and a class change.
- Proof: a scratch override on Danitha Capashen, Paragon (`dsl = [{ type = "static", static = "cost", ... }]`, on a card with a `cost` hook), then `cardgen export --offline --out <scratch>`. It exited rc=1, the out dir stayed empty, and the override was then restored:
```
cardgen: refused: 1 card(s) run through two paths (Phase 2, one card model); nothing was written
  Danitha Capashen, Paragon [hard]: cost: static:cost + cost
      dsl: static:cost | tags: - | rule: cost | ai: -
Fix each card one of three ways:
  - delete one path: the card-code slot (sim-core/src/impls) or the compiled ability / executing tag;
  - a `dsl` override in data/card_overrides.toml: `dsl = []` (card code alone) or the abilities without the doubled one;
  - if the two paths do different parts of the card on purpose: `paths_ok = "why"` on its override.
(`cardgen audit --paths` lists every card's paths; data/paths_known.txt only shrinks: don't add to it.)
```

## 2. Coverage per deck (`rust/crates/sim-cli/src/coverage.rs`)
- It prints before the results:
  - pool mode (`--deck`, `--pool`, `--all-decks`, `--analyze`, `--swap`, `--calibrate`): in `main.rs`, before the preflight;
  - `--realism`;
  - `--goldfish`.
- It uses `cardgen audit`'s `card_status`, the same call `explain.rs`'s swap model status makes. It counts all five statuses: Full, Full-auto, Approximate, Partial, Unmodeled. Each list counts 100 slots, the commander included.
- It names up to 5 cards, Unmodeled then Partial. Within each, the commander comes first, then the engine cards, then nonlands, then lands, then "(+N more)".
- What each run prints:
  - `--deck`: one line per JD deck;
  - each opponent tier: one summed line;
  - `--realism` and `--calibrate`: a line per deck of the tier.
- It is computed once per run from the card data, about 0.3 s, and plays no game.
- `--quiet` keeps it: it only turns off the stderr progress bar, and the block goes to stdout with the results. There is no `--no-coverage` flag.
- `--fit`, `--trace` and `--cards` print no block.

Realism `--pool t3 --games 1000 --jobs 12`, origin/main binary against the branch binary (both rc=0): the outputs differ only in the 7 lines this block adds. Everything else is byte-identical:
```
Model coverage (cardgen audit statuses of the 99 + commander; names: Unmodeled, then Partial, commander and engine cards first, at most 5):
  t3 Atraxa   100 cards: Full 78, Full-auto 10, Approximate 3, Partial 7, Unmodeled 2 | Unmodeled: Astral Cornucopia, Norn's Decree; Partial: Ajani, Sleeper Agent, Dreamtide Whale, Innkeeper's Talent (+4 more)
  t3 Aurelia  100 cards: Full 86, Full-auto 9, Approximate 5, Partial 0, Unmodeled 0
  t3 Korvold  100 cards: Full 88, Full-auto 8, Approximate 4, Partial 0, Unmodeled 0
  t3 Marwyn   100 cards: Full 79, Full-auto 12, Approximate 9, Partial 0, Unmodeled 0
  t3 Tergrid  100 cards: Full 85, Full-auto 13, Approximate 1, Partial 1, Unmodeled 0 | Partial: Aclazotz, Deepest Betrayal // Temple of the Dead
```

JD's decks: `--deck KEY --pool <first own tier> --games 50 --jobs 12 --quiet`, all 12 rc=0.

| Deck (tier) | Full / Full-auto / Approx / Partial / Unmodeled | Named |
|---|---|---|
| alela (t1) | 82 / 18 / 0 / 0 / 0 | |
| cloud (t1) | 89 / 10 / 0 / 1 / 0 | Partial: Buster Sword |
| galadriel (t1) | 93 / 7 / 0 / 0 / 0 | |
| karn (t4) | 88 / 0 / 12 / 0 / 0 | |
| kefka (t3) | 81 / 17 / 2 / 0 / 0 | |
| sauron (t3) | 100 / 0 / 0 / 0 / 0 | |
| scion (t6) | 89 / 2 / 4 / 5 / 0 | Partial: Orim's Chant, Redirect Lightning, Tinder Wall, Gemstone Caverns, Shifting Woodland |
| seph (t3) | 96 / 3 / 0 / 0 / 1 | Unmodeled: Glimpse the Unthinkable |
| sheoldred (t3) | 84 / 16 / 0 / 0 / 0 | |
| thrasios (t5) | 96 / 4 / 0 / 0 / 0 | |
| veyran (t1) | 97 / 3 / 0 / 0 / 0 | |
| yshtola (t5) | 94 / 5 / 0 / 1 / 0 | Partial: Venat, Heart of Hydaelyn // Hydaelyn, the Mothercrystal |

Example opponent line: `opponents Tier 6 (cEDH, B5)  6 decks, 600 cards: Full 510, Full-auto 20, Approximate 19, Partial 51, Unmodeled 0`. The goldfish (`scion,karn`) and `--calibrate within --pool t1` smoke runs were both rc=0 and printed the block.

## 3. Docs
`documents/realism.md` (Phase 2 status: hard doubles 0; the known list is what's left; the gate is not passed; coverage printed), `commander-sim --help`, `rust/crates/sim-cli/README.md` and the `cardgen` usage text.

## Checks (all under the memory cap)
- `cargo test --release --no-fail-fast`: 3589 passed, 0 failed, 2 ignored.
- `cargo build --release --all-targets`: 0 warnings.
- `cardgen export --offline`: no change to `data/`. `cardgen validate`: all decks passed.

## Risks
- A newly fetched card (for a swap test, say) that is a double now blocks `cardgen export` until it is fixed or declared. This is intended.
- `data/paths_known.txt` is a second place, besides `paths_ok`, where a double can be accepted. It only shrinks, and the test enforces that.
