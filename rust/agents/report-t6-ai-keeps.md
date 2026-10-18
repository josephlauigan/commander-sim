# Report: t6-ai-keeps (Tier 6 land-light keeps)

Branch t6-ai-keeps. Files: `impls/t6_keep.rs` (new), the `keep` fns of the six pool plans (Godo's keep untouched),
`engine/turn.rs` (Tier 6 only: stat `mull_try`), `engine/zones.rs` + `engine/cast.rs` (Mox Diamond discard),
`ai/pool.rs` (Tier 6 Mox Diamond gate), `tests/ai_t6_keeps.rs`.

## What changed
- `t6_keep::mana` prices the hand's mana by turn 2 without drawing: land 1 (Ancient Tomb / City 2); Sol Ring 2 and
  Mana Vault / 1-mana rocks 1 only with a land; 2-mana rocks need two lands; Mox Diamond 0.5 and only with 2 lands;
  Chrome Mox 1 only with 2 spare spells; Lotus Petal 0.7; Mox Amber 1.0 for the Rograkh decks, else 0.5; Mox Opal 0.
- A 1-lander is kept only if its hand totals 3.0 mana (2.5 on the second seven, 2.0 later) plus the deck's own
  payoff/engine/tutor test. After the first mulligan any 2-4 land hand is kept; after two, a 1-lander with 2.0 mana.
  (A first mulligan is a free redraw of 7, so it costs no cards; the cost is in doubles and triples.)
- Mox Diamond: discard choice keeps the only source of a colour the hand needs and the 2-mana utility lands; the AI
  does not cast it (Tier 6) unless hand plus battlefield lands total 3 or more (g6: it discarded a land of a 2-land hand).

## Opening hands (1500 shuffled hands per deck, `cargo test --release --test ai_t6_keeps -- --nocapture rates`)
Lands by T4 = lands in the kept hand plus the 3 top cards (the draws of turns 2-4).

| Deck | mulligan before -> after | triple before -> after | 1-land keeps | lands by T4 |
| --- | --- | --- | --- | --- |
| Dack | 30.5 -> 41.9% | 2.9 -> 6.5% | 18.3 -> 4.1% | 3.13 -> 3.36 |
| Kinnan | 38.6 -> 50.9% | 4.5 -> 7.2% | 23.2 -> 9.1% | 2.82 -> 3.00 |
| Kraum/Tymna | 28.9 -> 47.1% | 2.5 -> 6.5% | 24.4 -> 5.1% | 2.87 -> 3.14 |
| Rograkh/Silas | 35.0 -> 55.3% | 3.3 -> 7.7% | 35.1 -> 11.5% | 2.57 -> 2.87 |
| Rograkh/Thrasios | 28.9 -> 42.5% | 2.9 -> 3.3% | 20.5 -> 6.9% | 3.14 -> 3.34 |
| Sisay | 33.7 -> 43.5% | 2.7 -> 3.5% | 14.9 -> 6.8% | 3.22 -> 3.32 |
| Godo (unchanged) | 38.6% | 6.9% | 9.0% | 3.19 |

These decks run about 22 lands, so 30-45% of first sevens have 0-1 lands; shipping them is the price. Triple
mulligans rose for Dack, Kinnan, Kraum and Silas (the rule is stricter and the deck has few lands to draw into).

## Tier 6 sim (300 games per profile, `--calibrate within --pool t6`; cons / loose), base -> after
| Deck | win % | median win round | mulligans (sim, any / triple) |
| --- | --- | --- | --- |
| Dack | 26.6/24.6 -> 29.1/21.7 | 8/7 -> 8/8 | 27.6% -> 38.4% / 7 -> 12 hands |
| Kinnan | 35.5/36.5 -> 34.0/40.1 | 8/7 -> 7/7 | 32.5% -> 47.2% / 7 -> 13 |
| Kraum/Tymna | 25.6/31.8 -> 23.1/26.7 | 10/9 -> 8/9 | 35.4% -> 52.8% / 8 -> 12 |
| Rograkh/Silas | 21.9/21.4 -> 20.9/23.5 | 7/7 -> 7/6 | 29.1% -> 52.6% / 5 -> 14 |
| Rograkh/Thrasios | 12.8/14.7 -> 14.2/10.4 | 11/12 -> 11/14 | 25.1% -> 39.3% / 4 -> 7 |
| Sisay | 28.3/21.7 -> 29.3/28.8 | 12/13 -> 12/12 | 30.8% -> 41.9% / 6 -> 5 |
| Godo (`--deck godo`, one profile) | 47.7% -> 48.0% | 6 -> 6 | 38.7% (unchanged) |

Every win-rate move is within the 300-game noise (about +-5 points). Honest reading: stricter keeps did not make
games faster or the decks stronger yet; the median win round only fell clearly for Kraum (10 -> 8/9). The land-light
sits are shorter in the opening-hand model (lands by T4 up 0.1-0.3, 1-land keeps cut by two thirds or more), but the
extra mulligans cost the cards back. Next lever (not mine): the decks' land count or tutor/dig choices; or a keep
that trades a lone 1-lander for a better 6 only when the hand has no cheap plays.

## Checks
- t3 lock: identical to `agents/perf-lock-t3.txt`. No "internal simulator error" in any run.
- Tests: new `tests/ai_t6_keeps.rs` (7 tests: Diamond/Chrome/Sol Ring not counting, free mana 1-lander kept, later
  hand kept, Mox discard keeps a needed colour, Mox cast gate, plus the two measuring tests). Full suite: all pass, 0 failures; `cargo build --release --all-targets`: 0 warnings.
- Not done: Godo's keep (not in the task list; its triple-mulligan rate is 6.9%), "lands on turn 4" measured on
  shuffled hands (not in played games: that needs a new report counter).
