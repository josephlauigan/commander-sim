# Report: sim results write-up (2026-11-04)

Edited: `audit/{urdragon,sauron,cloud,kefka}/FINDINGS.md` (a 2026-11-04 section each, plus pointers on stale lines). Nothing
run, nothing committed. Raw outputs: `/home/joseph/projects/commander-sim/.claude/worktrees/sim-runs-2026-11-04/`
(abbreviated `R/`). Every number below is copied from the cited file and line; "derived" marks my arithmetic.

## The Ur-Dragon, t5 (`R/urdragon-baseline.txt`)
- Win rate 21.1%, CI 19.4-23.0, even share 25% (-3.9): line 18. Wins turn 10.1, losses 7.6: line 19. Plan online 1.8%: line 20.
  Wins by damage 93%, combo 7%: line 21. Opponent table (seated / won when seated / eliminated you): lines 23-27.
- Coverage (Full 94, Full-auto 4, Approximate 2, Partial 0): line 2.
- Derived: eliminated-you sum 12.1+20.6+17.4+13.2+15.1 = 78.4%; 21.1 + 78.4 = 99.5, so 0.5% neither.
- Prefix run (`R/urdragon-baseline-prefix.txt`): 1990 games, 21.1% (19.4-23.0), turns 10.1 / 7.5 (lines 17-19); 10 games skipped
  on a multiply overflow at `engine/repl.rs:383` (lines 32-38, 5 seeds listed).
- Approximate cards: `data/audit.toml` lines 258 (Scion of the Ur-Dragon) and 379 (Astral Dragon). Cursed Mirror has no
  Approximate entry; `rust/agents/report-urdragon-approx-cards.md` line 7 says Full.

## Sauron, t3 and t4 (`R/sauron-baseline.txt`)
- t3 29.1% (27.2-31.1): line 42. t4 25.9% (24.0-27.8): line 48. Average 27.5% over t3, t4: line 52. Coverage: lines 32-34.
- Derived: pooled band for the average, sqrt(0.995^2 + 0.97^2) / 2 x 1.96 = about +-1.4. t3 minus t4 = 3.2.
- The output has no win/loss turns, win shares or eliminators (multi-tier `--pool all` prints only the table).

## Cloud (flipped: new card over old)
Judgment of Alexander over Stand Up for Yourself (`R/cloud-judgment-vs-standup.txt`):
- t1 31.9 -> 30.1, delta -1.8, +-2.1: line 34. t2 22.3 -> 22.1, -0.2, +-2.1: line 125. t3 22.9 -> 21.9, -1.0, +-1.9: line 219.
- Card rows (drawn / cast / avg round / stuck / win% cast / lost per game): t1 lines 90, 93 (Stand Up 24.8 / 79.2 / 7.1 /
  20.8 / 44.0 / 0.00; Judgment 24.4 / 23.1 / 9.8 / 76.9 / 54.0 / 0.00); t2 lines 183, 186 (26.4 / 86.0 / 8.3 / 14.0 / 31.1;
  26.2 / 34.3 / 11.7 / 65.7 / 28.9); t3 lines 275, 278 (23.2 / 75.9 / 6.8 / 24.1 / 33.2; 22.8 / 17.5 / 9.4 / 82.5 / 55.0).
- Spot removal cast 0.97 -> 1.14 (line 66), 1.05 -> 1.21 (line 157), 0.77 -> 0.93 (line 251), each +-0.06 starred.
  Tutors resolved at t2 3.81 -> 3.70, +-0.10 starred: line 152.
- Game shape: lines 97-121 (t1), 190-214 (t2), 282-306 (t3).
- Derived averages: Judgment lists (31.9+22.3+22.9)/3 = 25.7; Stand Up (30.1+22.1+21.9)/3 = 24.7; delta (1.8+0.2+1.0)/3 = 1.0;
  band sqrt(2.1^2 + 2.1^2 + 1.9^2)/3 = about 1.2.
Forge Anew over Sphere of Safety (`R/cloud-forge-vs-sphere.txt`):
- t1 31.9 -> 30.6, -1.4, +-1.8: line 33. t2 22.3 -> 20.4, -1.9, +-1.8, verdict WORSE: lines 120, 122. t3 22.9 -> 22.6, -0.4, +-1.6: line 212.
- Card rows: t1 lines 85, 88 (Sphere 23.8 / 42.1 / 8.5 / 57.9 / 55.0 / 0.01; Forge 24.6 / 64.4 / 8.7 / 35.6 / 62.8 / 0.00);
  t2 lines 176, 179 (26.2 / 50.8 / 9.4 / 49.2 / 29.3 / 0.03; 26.8 / 68.5 / 9.1 / 31.5 / 43.1 / 0.01); t3 lines 264, 267
  (21.1 / 44.1 / 8.1 / 55.9 / 44.1 / 0.00; 21.3 / 54.7 / 8.0 / 45.3 / 55.4 / 0.00).
- "Other triggers" damage 4.51 -> 4.22 (line 76), 3.18 -> 2.86 starred (line 163), 3.98 -> 3.79 (line 255).
- Game shape: lines 92-116, 183-207, 271-295.
- Derived: Sphere lists (30.6+20.4+22.6)/3 = 24.5; Forge 25.7; delta (1.4+1.9+0.4)/3 = 1.2; band sqrt(1.8^2+1.8^2+1.6^2)/3 = 1.0.
- Model notes from `rust/crates/sim-core/src/impls/cloud.rs` lines 1192-1202 (Judgment) and 1339-1347 (Forge Anew), and
  `rust/agents/report-cloud-gaps.md` (AI triggers, priorities 68 / 45 / 0).

## Kefka, t3 (`R/kefka-urborg-vs-swamp.txt`)
- Win 18.3 -> 18.1, delta -0.1, +-1.1: line 29; verdict line 31. Axis table win rate -0.2, +-2.4: line 40.
- Mana turn 4 3.81 -> 3.82, turn 6 5.18 -> 5.18: lines 46-47; turn 8 6.62 -> 6.61: line 108; missed land drops line 45;
  card rows (Swamp drawn 87.6, Urborg 29.1): lines 83, 86; plan and commander timings lines 96-106.
- Black-source count (7 Swamp, Blood Crypt, Haunted Ridge, Sulfurous Springs, Underground River; Command Tower, Exotic
  Orchard, Gemstone Mine) is my count from `data/decks.json` (kefka `cards`), not from the run.

## Things that look odd
- Tool deltas sometimes differ from the rounded rates (Kefka 18.3 vs 18.1 prints -0.1 on one line, -0.2 on others).
- Win-rate noise differs between the headline (paired, e.g. +-2.1) and the axis tables (+-2.6-2.9). I used the headline.
- Cloud's current-list baseline (31.9 / 22.3 / 22.9) and Kefka's (18.3 at t3) are well above the 11/01 remeasure
  (Cloud 25.6 / 14.2 / 17.0; Kefka 13.6). Not investigated.
- The brief says "Cursed Mirror is Approximate"; `data/audit.toml` marks Astral Dragon and Scion of the Ur-Dragon instead.
- The brief says "six combo cards"; the three combos name seven distinct cards (Astral Dragon, Cursed Mirror, Miirym,
  Doubling Season, Garth One-Eye, Twinflame, Spark Double). The combo report also says "six".
