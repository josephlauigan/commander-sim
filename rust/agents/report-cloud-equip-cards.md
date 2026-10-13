# cloud-equip-cards report
All four cards modeled in `rust/crates/sim-core/src/impls/bulk_white.rs` (`register_equip`, called at the end of `register`); verdicts Full in `data/card_overrides.toml`; `cardgen export` rerun.
- Ardenn: `combat_start` hook; moves every loose / worse-worn Equipment to `cloud::best_holder` (Cloud first), via `cloud::pullable`. Auras are never moved by the AI (leaving them is a legal choice). Partner unused.
- Balan: first strike from data; double strike via `grant_kw` with >=2 Equipment attached; {1}{W} ability (`balan_options`, pre-combat, own turn) attaches all Equipment, offered only if Balan can attack and Cloud is out of play or can't attack.
- Armored Skyhunter: `attack` hook; top six, best Aura/Equipment by `card_worth` onto the battlefield (Equipment attached to best holder, own-creature Aura onto best holder), rest to bottom in random order.
- Kor Duelist: `grant_kw` double strike while equipped.
Shared-file change: `cloud.rs` `pullable` made `pub(crate)` (one word). No cast-priority entries added (cards are not in the deck list).
Tests: 7 new in `tests/cards_bulk_white.rs`. `cargo test --release -j 4 --no-fail-fast`: all pass (http_lan did not fail here). Build `--all-targets`: 0 warnings. Scratch audit: cloud 74 unique, 0 not Full. Trace `--deck cloud --pool t3 --trace 0` finishes (the 4 cards are not in the real list, so unexercised in the trace).
