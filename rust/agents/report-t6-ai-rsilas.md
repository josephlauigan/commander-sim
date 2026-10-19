# Report t6-ai-rsilas (Rograkh/Silas turbo)

Changed: `impls/t6_plan_rograkh_silas.rs` (keep + wish), `tests/ai_t6_rograkh_silas.rs` (7 tests). No other file changed (a hook in ai/search.rs was tried and reverted).

## Before / after (300 games per profile; conservative / loose)
| | baseline | after |
|---|---|---|
| win % | 19.9 / 18.4 | 25.0 / 20.9 |
| wins by | Oracle line 95%, damage/other 5% | Oracle line 98% / 93%, damage 2-7% |
| median win round | 7 | 7 / 8 |
| combo tries per game | 0.33 | 0.37 / 0.29 |
| stopped | 41% | 34% / 32% |
| mulligans | 44.4% (18 triple) | 29.1% (5 triple) |
| t3 lock | | identical to perf-lock-t3.txt |

## What was done and why
- keep: the shared rule (2-5 lands) mulliganed every 0-1 land hand; the deck has 23 lands in 99 plus ~18 fast-mana pieces. New rule counts lands + fast mana, needs a tutor/payoff/Oracle (or 2+ fast mana), ships 6+ lands and no-land hands (unless 3 fast mana + tutor). Alone it gave 25.0/21.4 against 19.9/18.4 (A/B, same seeds).
- wish: missing half of Oracle line first (Consultation/Pact for Oracle, Oracle for Consultation/Pact). Neutral in the A/B, kept as it follows the spec.
- Tried and DROPPED (measured worse, 15-17% with all of it; each sub-part alone 17-23%, noisy): Rograkh cast first/Silas held, Mox Amber only with a legend, rituals held for Ad Nauseam, Squelcher before Oracle line, develop-first `hold_up` (needed a hook in `ai/search.rs hold_bonus`), Wishclaw/Floodcaller/Bauble held. The prio/hold_up/go_now slots return None.

## Findings (trace, godo seat)
- Rograkh was cast on turn 5 in one trace (shared commander rule: cmd_turn 2, Silas first); lookahead search picks "stop (hold mana)" over Petal/Rograkh because `hold_bonus` ignores `hold_up`; Oracle is mostly stopped at step 1 by a counter on the creature spell (Fierce cannot hit it). Fixing those by prio did not pay in the sim, probably because Rograkh is exiled/killed early more often when cast on turn 1 and Silas never replaces him; not resolved.
- `ad_ready` needs 8 mana (3+BBUUB) so Ad Nauseam/Breach lines rarely go; rituals are not counted in combo affordability.

## Still differs from the spec
Turn-1 Rograkh, win on turns 2-4 (median win round 7-8), Breach line (<=1 try/300), Ad Nauseam end-step turn, protection before going (Silence-like cards, Squelcher, Grid), go-or-wait reads. 5-10 "stopped, no card named" per 300 games not diagnosed. Stage 6 numbers untouched. Unrelated flaky test: practice `test_replayed_history_is_marked` failed once in the full run, passes alone (twice).
