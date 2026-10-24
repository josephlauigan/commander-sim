# Report t6-speed kraumgo (2026-10-19), branch t6-speed-kraumgo
Change (impls/t6_plan_kraum.rs only): go_now (Oracle line) no longer demands creature cover while any opponent has cards
and mana. It also goes when (a) no opponent's stop_chance >= 0.2 and every opponent has <= 2 open mana (tapped low), or
(b) one piece of protection is in hand (counter/Silence/Chant) and no opponent's stop_chance >= 0.5. The Kraum/Esper
Sentinel "drawer" refusal and the Ad Nauseam life gate are unchanged. hold_up gives no pass bonus when a line is ready.
Runs: `--calibrate within --pool t6 --games 1000 --seed 500000 --jobs 7` (audit/t6/speed-2026-10-19/kraumgo-v1.txt, -v2.txt).

| Kraum/Tymna | ready | go | win round (median) | win % cons / loose |
| before (cal1000) | 7 (51%) | 9 (30%) | 10 / 9 | 31.3 / 31.7 |
| v1 (looser: tapped<0.25, cover<0.7) | 7 (51%) | 8 (35%) | 9 / 8 | 34.1 / 32.7 |
| v2 final (tapped<0.2, cover<0.5, hold_up fix) | 7 (51%) | 8 (34-35%) | 9 / 9 | 33.7 / 33.0 (avg 33.4) |
Ready-but-never-went: 141 -> 114-118 of ~330 ready games; later-than-ready mean wait 1.72 -> 1.25-1.59 rounds. The
remaining "stop (hold mana)" choices (~410-480) come from the look-ahead's generic option scoring, not go_now.
Result: go 1 round earlier and win median 1 round earlier (cons 10 -> 9), but win % is +2.1 points and sits ~1.4 above
the 32 ceiling (1000-game CI +-3.6, so not significant). Not trimmed further within the timebox. Next lever if JD wants
it back under 32: raise the Oracle cover requirement only when 2+ opponents have blue open, or a stricter keep.
Other decks' numbers in the run are within noise of cal1000 (Kraum only touched).
## Checks
t3 lock (300 games, seed 500000, filtered): LOCK IDENTICAL. Tests: sim-core ai_t6_keeps 7 pass, engine_t6_combat 4 pass.
