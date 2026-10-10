# Porting card code

How Python's hand-written card code (`commander_sim/cards/impl/*.py`) becomes Rust (`crates/sim-core/src/impls/`).
The Python is the reference and is frozen during the port: read it, don't change it.

## Where things live

| Python | Rust |
|---|---|
| `@on('Card', 'event')` in `cards/impl/x.py` | a fn in `impls/x.rs`, set in its `register`: `r.card(db, "Card")?.event = Some(f);` |
| `CI.SPELL_PRIO`, `CI.SPELL_IMP`, `CI.PVAL` | the `prio`, `spell_imp`, `pval` slots |
| `CI.DYN_MANA`, `ON_TAP`, `LAND_ETB`, `LAND_COLS`, `AS_ENTERS`, `SELF_CAST`, `SELF_REGEN` | the slots of the same names (`dyn_mana_land` / `dyn_mana_perm`, `on_tap_land` / `on_tap_perm` ...) |
| `IC.spell(name, prio=..)` (a hand-written instant or sorcery) | the `resolve` slot (returns where the card goes) and `prio` |
| the hook signatures (`etb(g, src, p, m)` ...) | `hooks.rs`: one type per event (`EtbFn`, `AttackFn` ...) |
| `CI.fire`, `hooked`, `total`, `hand_cards`, `gy_cards` | `engine/hooks.rs` and `cardcode.rs` (already ported: register a slot and the engine calls it) |
| shared functions the engine calls by name (`common.aura_fall`, `rules.transform_away` ...) | placeholders in `cardcode.rs` marked `PORT(M5): module.function`, or stubs in `impls/x.rs`; replace the body with the port |

`impls/t1.rs` (Isshin's attack package) is the pattern to follow.

## Translating

- `src.owner is p` is `g.perm(src).owner == p`; `src in atk` is `atk.contains(&src)`; `m in p.perms` is
  `g.perm(m).on_bf && g.perm(m).owner == p`.
- `trigger_window(g, p, src, 'text', imp=6)` is `trigger_window(g, p, Some(src), "text", Some(6.0))?`: it returns
  `Res<bool>`, and the `?` matters (the engine probes a pending hook by running it until its trigger window).
- Python exceptions that end a game or a playout are `Err(Stop::..)` in Rust: propagate with `?`, never swallow them.
- `id(m)`-keyed dicts on the game (`g.eot_pt[id(m)]`) are fields on the permanent (`eot_pt`); per-card state Python
  keeps in `m.data` is `PermData` (`DataKey`); per-player attributes added with `getattr(p, 'x', default)` are
  `Player` fields (add one if it's missing, with Python's default).
- Once-per-turn keys built with `id(src)` use the permanent id: `intern(&format!("key_{}", src.0))`.
- Random choices use `g.rng` exactly where the Python does (the same number of draws keeps games comparable).
- `E.human_choice(g, p)` branches (practice mode) are not ported now: leave a `// HUMAN(phase 9): ...` comment and
  port the AI branch.
- Python's `sum()` of floats is `.psum()` (`pysum::PySum`); `max(xs, key=..)` / `min` keep the first best: use
  `engine::zones::max_by` / `min_by`.
- Keep Python's names, and say where each function comes from in its doc comment (`/// t1.fresh: ...`). Card notes
  from `note(..)` become the doc comment of the card's function.
- A card defined with `card('Name', 'tags')` in Python already has its tags in data/cards.json: nothing to port.

## Checking

- `cargo test` from `rust/`; tests for ported cards go in `crates/sim-core/tests/cards_<module>.rs`, built with
  `testkit` (`table`, `hand`, `lands`, `perm`, `token`), ported from the Python tests where they exist.
- `python3 rust/tools/difftest.py --gen 1000` from the repository root: the same positions in both engines. A card
  you port should leave the "card code" bucket and show no AI or build differences.
- Whole games: `cargo run --release -p sim-core --example all_decks` plays every deck.
