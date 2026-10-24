# Report t6-cal-sisay (2026-10-18)
Seed 500000, `--calibrate within --pool t6`. Sisay win rate / combo tries per game / share of tries stopped:
- Start (brief, 2000 games): 19-21% / 0.06 / 69-87%
- after2 (modeling fixes 1-3, 600): 20.0% / 0.06 / 52% (25 tries)
- after3 (+ combo before fetch in must_option, 600): 19.8% / 0.11 / 84% (43 tries)
- after4 (+ go_now off, 600): 20.0% / 0.11 / 79%
- Final (2000, go_now off): Sisay 21.3% (19.2-23.6), Dack 34.1, Kinnan 31.2, Kraum/Tymna 27.7, Rograkh/Silas 21.4, Rograkh/Thrasios 14.5 (LOW, other agent). Sisay wins: damage 68%, DDR 20%, Unicorn 11%. 0.12 tries/game, 84% stopped. Seats 28.0/26.3/23.9/21.8 (seat 1 and 4 still OUT, table-wide).
- t3 lock: identical (0 diff lines). sim-core tests: no failures.

## Modeling changes
1. Marvin's `options` were set twice (newcards_b = Kefka copy, t6_legends1b = Ioreth untap); the later overwrote the earlier. `marvin_both` returns both (lead's merge kept one copy).
2. `ddr_keys` counted every legend creature (Sisay too) as a piece an answer can stop; now Ioreth, Marvin, one mana legend.
3. Fetch-aware readiness: `ddr_plan`/`uni_plan` return Plan{casts, fetches}; a missing piece counts if in hand or fetchable by Sisay (on bf, MV < power); Doom/Selvala fetchable; WUBRG per fetch in the mana check; new "Sisay fetches" steps (`legends::sisay_fetch_card`).
4. must_option (t6_table.rs): Sisay's fetch (91) beat a ready combo option every turn (trace: R9 she fetched Emiel, Selvala, Tyvar, Cauldron with Ioreth+Marvin+Selvala+Doom out and never tried DDR); a "combo: " option now scores 92 for the sisay deck. Tries doubled.
## Tuning
go_now: a "go when no spell is needed" rule changed neither tries nor wins; left as no opinion (`t6_combos_t6::line_needs_cast` kept).
## What still differs
Win rate stays ~20-21%: more tries are stopped (the table holds Snap, Chain of Vapor, REB, FoW). 61 of 136 stops name no card ("a cost, a ward or a lock"), 41 of them Unicorn: likely a step failing for no game reason (not found yet; trace with `grep "stops at step"`). Sisay is the top removal target (258 hits in 600 games), is recast repeatedly (R7, R9, R11 in a trace), and fetches junk late (Lavinia, Ragavan). Mulligans 42.7%. Sisay is inside 18-32% already; best-worst gap (Dack vs Rograkh/Thrasios) is the other agent's.
