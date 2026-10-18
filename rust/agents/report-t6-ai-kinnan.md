# Report t6-ai-kinnan (Kinnan, Bonder Prodigy mana combo)

Files: `impls/t6_plan_kinnan.rs` (keep, prio, wish, hold_up, go_now), `tests/ai_t6_kinnan.rs` (7 tests: spec scenarios 3, 11, 12, 14, wish list, prio, gate). No other file touched.

## What the plan does
- keep (primer order: draw engine, early win piece, mana): ship 0 lands, 6+ lands, 1 land without two mana makers (Mox Diamond counts only with 2 lands), 2 lands with nothing to do; keep fast mana + Kinnan hands. The deck has 24 lands, so ~35% of 7-card hands are still 0-1 landers (unit sample: 43% -> 35%).
- prio: Kinnan 92 (before the rocks, each taps double), Rhystic Study / Remora 84 on turns 1-4, Sphinx 70, One Ring 66; counters/bounce held (0).
- wish: Basalt Monolith, Transmute Artifact, then an outlet (Thrasios, Mirage Mirror), Horror with Kinnan + 2 rocks, early engines, then cover (FoW, Fierce, Pact) once Monolith is held.
- hold_up: develop on turns 1-2, +1 hold with Kinnan out and a free counter in hand.
- go_now (Kinnan + Monolith / Horror lines): wait with stax on the table (Ouphe, Cursed Totem, Null Rod, Stony Silence), wait with no counter in hand while an opponent has 2+ blue open, wait before round 4 with no outlet (Thrasios, Mirror, Finale, Wan Shi Tong) and no free counter; from round 6 the shared rule decides.

## Before / after (300 games, `--calibrate within --pool t6 --games 300 --jobs 3`; before = branch start, stub plan; two profiles: conservative / loose)
| | before | after |
| --- | --- | --- |
| win % | 37.8 / ~32 | 41.3 / 35.2 |
| wins by Kinnan+Monolith (all variants) | 64% + 14% outlets | 76% + 12% outlets (Horror 8%, damage 4%) / 74% + 8% |
| win round avg / median | 8.1 / 7 | 8.3 / 7 (loose 8.1 / 8) |
| combo tries per game / stopped | 0.33 / 6% | 0.46 / 15% (loose 0.35 / 9%) |
| mulligans | 46.4% (one 47, two 25, three 19) | 32.7% (one 47, two 10, three 7) |
Checks: Tier 3 lock identical, `cargo test --release` passes, no build warnings. The before run hit 2 preexisting "attempt to subtract with overflow" panics (seed 500059; not mine, not traced).

## Still differs from the spec
- Win rate is HIGH (35-41%, band 18-32%): fewer mulligans made the strongest deck stronger, and go_now patience barely lowers it (tries went UP to 0.46). A Stage 6 number job; the plan does not hold back on purpose.
- No bait play (cast Thrasios/Horror before the key spell), no Kinnan end-step flip decision, no Boseiju-on-stax choice, no "start the loop on the turn of the player to your left": all live in shared code/combo framework.
- Mulligan does not read seat (primer Hand 5: One Ring hand is a ship going last); keep rule is seat-blind. Still 33% mulligans, mostly 0-1 land hands (24-land list).
- Win share is still mostly one line (Kinnan + Monolith 74-76%); the spec wants it the most common, with Horror/Mastermind lines rarer: roughly so (Horror 8%), Mastermind decking is not modeled as a win.
