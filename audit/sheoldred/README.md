# Sheoldred: the Jodah trade blocks in the Sheoldred deck (2026-10-11)

Paired swap tests on the bulk-only list (`decklists/JD/sheoldred-mono-black.md`, code dfeb5ee): look-ahead AI, loose
profile, 1,000 games per list per tier, the same seeds, seats and opponents' draws for both lists (`run.sh sheo-A
sheo-B sheo-C`; raw output in `sheo-A/`, `sheo-B/`, `sheo-C/`; per-tier lines in `log.txt`). Over all five tiers the
noise is about ±1.4 points for A and B, ±0.8 for C.

| Block (Jodah -> Sheoldred) | Sheoldred cuts | T1 | T2 | T3 | T4 | T5 | All tiers |
|---|---|---|---|---|---|---|---|
| Current list | | 16.8% | 21.7% | 20.5% | 23.5% | 22.0% | **20.9%** |
| A: Memory Jar | Carnage Altar | +2.5 | **+6.8** | **+5.2** | +0.6 | -0.6 | **+2.9** (23.8%) |
| B: Toxic Deluge, Bitter Triumph | Depressurize, Multiply by Zero | +1.0 | **+4.0** | +0.2 | -2.2 | +0.5 | +0.7 (21.6%) |
| C: Korlash, Heir to Blackblade | Gravblade Heavy | +1.1 | **+2.3** | +0.1 | -0.4 | -1.1 | +0.4 (21.3%) |

Bold: outside that tier's noise. Only block A is a clear gain: Memory Jar cracked with Sheoldred out makes each
opponent draw seven (14 life each). B and C are within noise over all tiers.

Not measured yet (JD, 10/11: later): what each block costs Jodah with its bulk refills (`run.sh jodah-A jodah-B
jodah-C`), and all three blocks at once. The current list's 20.9% comes from these paired runs, not the section 0d
seeds (500000-500959), so it is close to but not directly comparable with the pool-results table (Titania 22.8%).
