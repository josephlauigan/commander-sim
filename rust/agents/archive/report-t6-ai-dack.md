# Report: t6-ai-dack (deck:dack-fayden-white-stax)

Files: `impls/t6_plan_dack.rs` (all five slots), `tests/ai_t6_dack.rs` (7 tests). No other source file changed.
Measured with `--calibrate within --pool t6 --games 300 --jobs 3` (before = agents/dack-before.txt, after = agents/dack-after3.txt; other decks differ run to run by seed/AI noise, 300 games = +-6 points).

| Dack report block | before | after |
| --- | --- | --- |
| win % (CI) | 23.6 (18.3-29.9) | 28.1 (22.3-34.6) |
| wins by | flip 70%, damage 25%, cmd dmg 5% | flip 71%, damage 29% |
| win round avg / median | 9.0 / 8.0 | 9.4 / 7.0 |
| combo tries (per game) / stopped | 35 (0.17) / 11% | 41 (0.20) / 10% |
| mulligans | 36.0% (one 49, two 15, three 9) | 27.6% (36 / 13 / 7) |
| stax cast | 381 | 356 (Silence, Needle, Chalice, Grid, Void Mirror top) |
(An earlier strict keep rule raised mulligans to 41%; the final rule keeps 66% of random 7-card hands offline.)
Tier 3 lock: identical to perf-lock-t3.txt. `cargo build --release --all-targets`: no warnings. `cargo test --release`: 2576 passed, 0 failed.

## What the plan does
- keep: sources (lands + fast mana) >= 3, or 2 with fast mana/Tomb/City/Petal or 3 cheap plays; ship 0 or 6+ lands, no spells, no white with < 5 sources.
- prio: fast mana first (90); Trinisphere / Bauble / Void Mirror held while free plays or fast mana are in hand; Silence (2+ answerers) or Orim's Chant (1) cast before Dack when the mana covers both; Dack (92) only when the table is locked, or table risk (stop_chance product, discounted by Angel's Grace/Reprieve/Gambit) is under a bar that rises by round (0.3 -> 1.0 at round 6), lower with 7 creatures still in the library, 1.0 if an opponent's archenemy score >= 6. Chalice only if the best X nets >= 3 cards against the table, Cursed Totem vs creature tables, Defense Grid vs likely counters, Deafening Silence not on a protection turn, Vanquish/Sunscour only vs 5+ opposing creatures.
- wish (Enlightened Tutor): rock when short of Dack mana, else the lock piece that fits the table.
- hold_up: develop on turns 1-3. go_now: the Dack flip combo always goes (the pile is already donated; waiting gains nothing).

## Still differs from the spec
- Fiend Hunter / All-Fates Stalker are still cast about 100 times each per 200 games: they carry DSL values, and `card_utility` replaces a priority of 0 with the DSL value for such cards, so the plan cannot hold them. Fix would need a small shared hook (not done).
- No end-of-turn Dack (Emergence Zone / Scout's Warning); no Mandate/Abeyance timing; removal target choice is the shared table logic; the Dack-flip target assignment (who gets which creature) is untouched.
- Seat 1 share is still 26% (target 30-38): not tuned directly.
- Baseline run hit a pre-existing engine panic: seed 500059 "attempt to subtract with overflow" (2 games skipped in dack-before.txt); not mine, not investigated.
