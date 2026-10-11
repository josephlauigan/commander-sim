# Rust port: reference results

The Python results the Rust engine is checked against (documents/rust-rewrite-scope.md, section 7).

| Run | Code | Result | Time |
|---|---|---|---|
| Sauron vs Tier 1, loose profile, look-ahead AI, 960 games, seeds 500000–500959 | `rust-port` at 4314911 | **28.9%** (95% CI 26.1–31.8%) | 24m08s wall on 22 workers; 29,908 CPU-s (31 CPU-s per game) |

| The same games on the Rust engine (M5 go / no-go) | `rust-port` at 9de25a4, `--engine rust` | **30.9%** (95% CI 28.1–33.9%) | 70s wall on 22 workers; 1,439 CPU-s (1.5 CPU-s per game) |

The full outputs are in `reference-sauron-t1-loose.txt` and `rust-m5-sauron-t1-loose.txt`. The two win rates differ by
2.0 points (z = 0.96, p ≈ 0.34): within chance. Per opponent, Teysa wins more often in Rust when seated (26.7% against
20.2%) and Tatyova less (15.8% against 18.5%); those are for phase 8 to look into. Python had 1% timeout wins, Rust none. The test suite ran alongside the first few minutes, so the
time is a little slower than an idle machine would give. The speed comparison at M5 reruns both engines on an idle
machine.

Command: `python3 -m commander_sim --deck sauron --pool t1 --profile loose --games 960 --seed 500000 --jobs 22`
