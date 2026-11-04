# Report L: AI logic fixes (2026-11-04)

Tests: `crates/sim-core/tests/ai_logic_fixes_l.rs` (11 tests, AI paths asserted directly; no sims run).
Whole workspace `cargo test --release --workspace --no-fail-fast`: 3995 passed, 0 failed (includes practice_ability_probe,
practice_choice_probe, practice_counter_probe, play_bot_games); no build warnings.

| # | Card | Change | Where |
|---|------|--------|-------|
| 1 | Resplendent Angel | AI pump now +2/+2 and lifelink (was +2/+0) | impls/rules2.rs |
| 2 | Iron Man | AI puts the draw counter on its highest-pval creature (Army wins ties); before: Army only, nothing without one | engine/zones.rs `draw` |
| 3 | Blood Artist | one drain per Artist; also only when a creature dies (it fired for any permanent) | engine/zones.rs `hand_tag_death` |
| 4 | Adeline | AI: one token per opponent, each attacking its own opponent (atk_targets); human path untouched | engine/combat.rs |
| 5 | Armored Skyhunter | trigger_window now first for the AI too (probe run no longer pops the library); the second AI may_window removed | impls/bulk_white.rs |
| 6 | Zur | new `zur_host`: own-target Aura on Zur (as before), "opp" Aura (Arrest, lock Auras) on the opponent's best creature via `lock_attach` (it was never attached before and stayed in the library/was lost), no legal host: not fetched; person: non-own Auras now ask where they go | impls/t5.rs |

Notes for the coordinator:
- Skyhunter: the AI now always opens a trigger window when it attacks (before, only when it had a pick), as the person's path did.
- Zur's "opp" branch only covers registry target "opp" without a `host_pick`; Auras with their own host picker keep it.
- Not touched: Zulaport Cutthroat / `Tag::Drain` has the same once-only pattern with two copies (`hand_tag_death`); not in my six cards.
