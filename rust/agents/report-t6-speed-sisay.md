# Report t6-speed-sisay (2026-10-19)
## Changes (Tier 6 only)
- legends.rs `sisay`: Tier 6 also offers the fetch in the end-of-turn window (`post` None and is_t6). Per-deck gate; no shared hook was added.
- t6_combos_t6.rs: `uni_plan` / `ddr_plan` take `in_line`; steps of a running line ignore a tapped Bloom Tender (Unicorn stopped after the first fetch,
  trace game 40) and DDR skips the mana re-check. `fetch_next` logs "line no longer ready".
## Numbers (seed 500000, Tier 6 pool, loose profile)
| run | Sisay win % | median win round | ready (share of games) |
| baseline cal1000 (1000 games) | 20.7 (17.8-23.9) | 12 | 8 (14%) |
| 600 games, EOT only | 23.7 | 13 | 8 (13%) |
| 600 games, EOT + in_line | 24.0 | 13 | 8 (13%) |
| 2000 games, final | 21.5 (19.4-23.8) | 12 | 8 (10%) |
Others at 2000: Dack 28.5, Kinnan 26.1, Kraum/Tymna 32.8, Silas 24.7, Thrasios 16.5. Sisay stays in 18-32%.
## Verdict: no gain
Median win round, ready round and win % are unchanged within noise. The end-of-turn fetch fired in 1 of 60 traced games: after her own turn the
leftover mana is rarely WUBRG, and there is one mana pool per turn cycle, so fetching at the opponent's end step moves a fetch, it does not add one.
It costs nothing (gate is Tier 6, lock identical), so it stays. Wins: damage 65%, DDR 23%, Unicorn 11%; lines ready in only 10% of games.
## Unnamed stops (2000 games: 56 of 115 stops name no card)
Unicorn 38, DDR 18. The in_line fix removed the traced Unicorn stop (game 40) but the count did not fall, so other causes remain (not found).
DDR trace game 20 (--trace 20): at step 3 (fetch) ddr_plan returns None; debug print showed "unusable" with source Ragavan, haste=true, though
Ragavan/Marvin look usable; cause unfound. Partial traces of 107 games showed no other fetch-step stops.
## Checks
t3 lock: 0 diff lines. Full `cargo test --release`: no failures. Raw: audit/t6/speed-2026-10-19/sisay-v1-600.txt, sisay-v2-600.txt, sisay-v2-2000.txt.
