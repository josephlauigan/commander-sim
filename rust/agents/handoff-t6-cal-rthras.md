# Handoff t6-cal-rthras
Task: calibrate Rograkh/Thrasios (13-14% -> 18-32%). Branch t6-cal-rthras.
Findings so far: Oboro loop needs 6 creatures for net mana (Cradle N+Gates 1 in, 2+4 out): 5 is mana-neutral, so 6 is right for an infinite-mana finish; spec scen. 20 (5 = phase-out only) is a different use. finisher() needs Thrasios on bf AND Finale in hand/library or Mastermind+GSZ.
Step 1: building, baseline 600 games + traces.

## Status (paused by lead, CPU busy ~1.5-2h; no cargo/sims until told)
- Release binary built: rust/target/release/commander-sim (build cmd: `cargo build --release --bin commander-sim` in rust/). No code changes yet.
- Baseline 600-game run NOT done (first attempt had no binary). Base 2000 numbers: s6/base-s500000.txt (13.7% win, 0.10 combo tries/game, A 108 tries/78 wins, B 25 tries).
- Code read: plan_a (t6_combos_t6.rs ~L712) needs Cradle, Gates in reach, finisher() (Thrasios ON BF + Finale/ (Mastermind+GSZ) in hand or library), Oboro in hand/bf, creatures>=6, 6 spare mana. plan prio holds Oboro/Snap/GSZ/Finale at 0. go_now: Oboro line only with shield or round>=5.
- Oracle check: loop nets Cradle N + Gates 1 - (2+4) => N=6 positive, N=5 neutral: 6 is correct for infinite mana (modeling OK). Spec scen. 20 (5 = phase-out only) is a separate use, not modeled.
- Next: baseline 600 + traces (--calibrate within --pool t6 --trace N); suspects: finisher requires Thrasios on bf (Thrasios cast prio 93 but needs {2}? check), Cradle tutoring, creature count 6, Rograkh recast after death; lines B-G rarely tried (Mastermind+GSZ needs infinite mana only).
