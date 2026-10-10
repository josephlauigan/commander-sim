# Rust port: reference results

The Python results the Rust engine is checked against (documents/rust-rewrite-scope.md, section 7).

| Run | Code | Result | Time |
|---|---|---|---|
| Sauron vs Tier 1, loose profile, look-ahead AI, 960 games, seeds 500000–500959 | `rust-port` at 4314911 | **28.9%** (95% CI 26.1–31.8%) | 24m08s wall on 22 workers; 29,908 CPU-s (31 CPU-s per game) |

The full output is in `reference-sauron-t1-loose.txt`. The test suite ran alongside the first few minutes, so the
time is a little slower than an idle machine would give. The speed comparison at M5 reruns both engines on an idle
machine.

Command: `python3 -m commander_sim --deck sauron --pool t1 --profile loose --games 960 --seed 500000 --jobs 22`
