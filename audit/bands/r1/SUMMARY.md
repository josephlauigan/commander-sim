# Round 1 (01:22-03:14, 1,000 games per list per profile, look-ahead AI; win % = mean of conservative and loose)

| Deck (band) | Tier | Current | With round-1 swaps | Note |
|---|---|---|---|---|
| Seph FS (1) | T5 | 21.5 | 22.6 | tutors cast ~82%; Rune-Scarred Demon stuck 61% |
| Y'shtola (1) | T5 | 23.2 | 25.7 | Farewell stuck 60%, Teferi's Protection stuck 75% |
| Seph/Atraxa (2) | T3 | 34.2 | 37.6 | Worldly Tutor never cast (AI bug, fixed ebd8d6f after this run) |
| Seph/Atraxa (2) | T4 | 28.3 | 27.9 | |
| Sauron (2) | T3 | 28.9 | 31.7 | loose +6.1 is likely noise (conservative -0.4) |
| Sauron (2) | T4 | 31.6 | 31.2 | |
| Kefka (3) | T3 | 27.4 | 25.6 | (fixed binary from here on) |
| Tymna/Kraum (3) | T3 | 35.9 | 34.5 | Underworld Breach never cast (kraum AI has no use for it) |
| Titania (4) | T1 | 20.0 | 18.5 | |
| Titania (4) | T2 | 32.2 | 26.6 | |
| Titania (4) | T3 | 28.3 | 26.0 | |
| Veyran (4) | T1 | 23.4 | 24.9 | |
| Veyran (4) | T2 | 26.0 | 29.4 | |
| Veyran (4) | T3 | 23.5 | 24.4 | |
| Sheoldred (2) | T3 / T4 | 32.5 / 33.1 | (baseline) | |
| Cloud (4) | T1 / T2 / T3 | 29.9 / 26.2 / 20.2 | (baseline) | |

Problems found: Seph FS still short of 24 at T5; Tymna/Kraum far above band 3; band 3 and band 4 overlap at T3
(Kefka 25.6 vs Titania 26.0 / Veyran 24.4). Fix: lift Kefka a little (better filler), trim Tymna/Kraum hard.
