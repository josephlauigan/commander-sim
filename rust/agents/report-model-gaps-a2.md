# Report: model-gaps-a2 (branch model-gaps-a2)

Statuses are from `cardgen audit --mine` (audit.toml [card_notes] wins over overrides).

| Card | Before | After | What changed |
|---|---|---|---|
| Hexing Squelcher | Approximate | Approximate | The life ward already existed (`t45_krenko::life_ward`, called from `removal::apply_removal_rest`); the brief's "mana only" note was stale. Added the person path (asked to pay 2 life; any life total >= 2 can pay; the AI still refuses at <= 4). Left out: ward against abilities and non-removal spells (the engine checks every ward, mana ones too, only when a removal spell resolves). |
| Thanos, the Mad Titan | Approximate | Full | Code was already right: once per Thanos object, and a token copy's mana value is the copied card's (`cd` is kept on copy tokens; plain tokens 0). Note was stale; test added. |
| Lotho, Corrupt Shirriff | Approximate | Full | The regex `treas` tag made a Treasure on entry (wrong). Override drops the tag (`tags = "leg pow=2 tgh=1"`, `dsl = []`); new cast hook `impls/model_gaps_a2.rs`: on any player's second spell each turn its controller loses 1 life and creates a Treasure. |
| Marvin, Murderous Mimic | Approximate | Approximate | Generalised: `dsl::borrowed_abilities/borrowed_options/activate_borrowed` now cover every compiled activated ability of the creature cards you control under another name (mana, discard, sacrifice costs; token copies; same per-turn cap; sorcery timing respected) instead of 4 named creatures with {T}-only abilities. Krenko, Kiki-Jiki, Ioreth hand-written copies unchanged. Left out: other creatures' hand-written (card-code) abilities. |
| Psychosis Crawler | Approximate | Full | Already compiled (`self_scaling` cards in hand, draw trigger drains each opponent); only the audit lacked a verdict (source "pool"). Override status/note added; test shows */* and the drain; the Kefka loop tests (cards_kefka) pass. |
| Astral Dragon | Approximate | Approximate | Only gap is a land target (engine keeps lands in `Player::lands`, a creature land can't attack) and Auras (copy isn't attached). Note made exact. No code change. |
| Scion of the Ur-Dragon | Approximate | Full | JD's decks' path (`t6_scion`) already follows Oracle (any Dragon permanent card incl. changelings, copy until end of turn, no ETB, person searches freely); AI choice of Dragon is policy. Fixed the pool Ur-Dragon path (`t45_urdragon::scion_go`), which put the Dragon in the graveyard but did not make Scion a copy. |
| Peter Parker's Camera | Approximate | Approximate | Unchanged: no clean way. Hand-written abilities run their effect inline after `ability_window`, paying costs inside the function pointer; re-running it for a copy would repay the cost. A fix needs each card's ability split into pay / resolve. |

## Shared-file changes
- `sim-core/src/dsl/mod.rs`: `ability_options` Activated arm moved into `activated_utility` (same logic); `activate` split into `activate`/`activate_ability(…, key)`; Marvin section rewritten (`borrowed_abilities`, `borrowed_options`, `activate_borrowed`; `borrowed_tap_options` removed).
- `sim-core/src/impls/newcards_b.rs` (`marvin_options` uses the above), `impls/t45_krenko.rs` (`life_ward` person path), `impls/t45_urdragon.rs` (`scion_go` becomes a copy), `impls/mod.rs` (new module registered).
- No change to engine/*, ai/*, state.rs, cardcode.rs, hooks.rs.
- Data: `data/card_overrides.toml`, `data/audit.toml`, `data/cards.json` (re-exported offline).

## AI changes
- Marvin: the AI now sees every compiled borrowed ability (including mana-cost ones), valued like the printed ones. The coordinator should sim Kefka/Sauron before and after.
- Pool Ur-Dragon Scion: becomes the copy (harmless at an opponent's end step).

## Tests
`rust/crates/sim-core/tests/model_gaps_others.rs` (7): Squelcher ward on itself/others/not own, AI refusal at 4 life; Thanos token-copy mana value; Lotho; Marvin mana-cost copy and token-copy copy; Crawler size and drain; pool Scion copy. Also run and passing: cards_kefka, cards_kefka_ai, cards_t6_legends1b, cards_t45_krenko, cards_thanos, cards_scion, cards_t45_urdragon, cards_t45_miirym, cards_urdragon_approx, cards_urdragon_jd, cards_sauron, cards_t6_copies, cards_t6_godo, paths_doubles, engine_t6_combo2, ai_t6_mixed_table, cards_kuja, play_thanos_human, play_urdragon_human, practice_ability_probe. `cargo build --release --all-targets`: no warnings. Full suite not run (coordinator).
