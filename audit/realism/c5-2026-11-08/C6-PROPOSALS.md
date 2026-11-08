# C6: keep-rule changes proposed from the C5 data (2026-11-08) — for JD's review

Nothing here is applied: no plan file has changed. Each proposal JD accepts gets a paired sim on the deck's own tiers
before it is adopted (the test plan is at the end). The evidence is in `REPORT.md` next to this file.

**In short:** the keep rules lose under 1.5 win points per hand in every deck, so the gains on offer are small. The
data point to one change: **mulligan a first seven with two lands and no fast mana**, in Y'shtola, Sheoldred and
Aragorn. Nothing for Thanos or Thrasios; Cloud is borderline.

"Fast mana" below is the keep rules' `fast` count (`plan_data.rs`: mana rocks and dorks the hand can cast early, plus
rituals). The C5 CSV counted it slightly differently (rocks and dorks of mana value 2 or less, Lotus Petal, the Moxen).
The first test step checks that the two agree on the C5 hands.

## P1. Y'shtola (t5): mulligan two lands with no fast mana — the clearest case

- **Today:** no keep table, so the engine default applies: keep 2-5 lands, whatever else the hand holds.
- **The data:** in 22 first sevens with two lands and no fast mana, mulliganing won **+7.5** points more than keeping
  (±4.6). Examples: Caves of Koilos, Hallowed Fountain, Anguished Unmaking, Displacer Kitten, Exsanguinate, Path,
  Propaganda (keep 7%, mulligan 40%). Two-landers with fast mana were fine (-4.3, 7 hands). Three or more lands: keeping
  is better (-4.7 to -10).
- **The change:** a `[mulligan]` table in `data/plans/yshtola.toml`, the same as today plus one row:
  ```toml
  [mulligan]
  fast_mana = true          # after the free mulligan, any 2-4 land hand is kept (the new row is for first sevens only)

  [[mulligan.rule]]
  when = ["hand-lands>=6"]
  keep = false

  [[mulligan.rule]]
  when = ["hand-lands<=1"]
  keep = false

  [[mulligan.rule]]
  when = ["hand-lands=2 & fast=0"]
  keep = false
  ```
  (No row fits: keep, the same as today's 2-5 range.)
- **What `fast_mana = true` also changes:** on the third seven (after two mulligans), a one-lander with 1 more mana
  and two spells is kept; today it is shipped to five. This is the same as Aragorn and Thanos already do. It touches very
  few hands.
- **Expected effect:** about 22% of first sevens change from keep to mulligan. At +7.5 each that is at most **+1.6** win
  rate, probably less: the best-scoring groups in a sample like this usually shrink when tested again.

## P2. Sheoldred (t3-t4): the same rule

- **Today:** no keep table, so keep 2-5 lands.
- **The data:** in 52 first sevens with two lands and no fast mana, mulligan minus keep **+3.8** (±2.8). The worst:
  Swamp x2, Call of the Ring, Exquisite Blood, Gix, Grave Pact, Triskelion (keep 22%, mulligan 45%). Two-landers with
  fast mana: +1.0 (18 hands, ±5.5), no signal. Three lands -2.3, four -5.1.
- **The change:** the same table as P1, in `data/plans/sheoldred.toml`.
- **Expected effect:** about 26% of first sevens change. At most about **+1.0**, likely less.

## P3. Aragorn (t4-t5): the same row added to its table

- **Today:** a table (`fast_mana = true`; mulligan at 6+ lands, 0 lands, 1 land with under 2 fast mana, or mana under 2).
  Every two-lander with mana 2 or more is kept.
- **The data:** in 44 first sevens with two lands and no fast mana, **+2.4** (±2.2), just past the noise. Two-landers
  with fast mana were also about +2.2 (28 hands, about ±3.5), so for Aragorn the weak spot may be all two-landers.
  Example: Sunpetal Grove, Verdant Catacombs, Mana Drain, Path, Smothering Tithe, Sylvan Library, Young Pyromancer (keep
  10%, mulligan 28%).
- **The change:** one row in `data/plans/aragorn.toml`, after the `hand-lands=1` row:
  ```toml
  [[mulligan.rule]]
  when = ["hand-lands=2 & fast=0"]
  keep = false
  ```
- **Option P3b, tested next to P3:** `when = ["hand-lands=2 & fast<2"]` (also ship two lands with one rock or dork).
  The data for this is weaker (28 hands).
- **Expected effect:** about 22% of first sevens change (40% for P3b). At most about **+0.5-0.9**. At the edge of what
  a sim can see.

## Not proposed

- **Cloud (t1-t3):** two lands without fast mana +2.1 (±2.2), inside the noise. Its table already keeps two-landers
  only with a `keep2` card. JD can ask for P1's row as a fourth test; I don't expect it to show anything.
- **Thanos (t5), Tymna/Thrasios (t6):** two-landers are no worse than a fresh seven (-1.9 and -1.4 on few hands). No
  pattern settles against either rule. Thanos keeps only 50% of first sevens and may be a little strict, but its
  mulliganed hands are 43 of 50 unclear, so there is nothing to act on.
- **One-landers:** every rule already ships them, and the oracle agrees.

## How each accepted proposal would be tested (needs JD's approval of this plan too)

1. **Build both versions:** a branch per deck with the plan-file change. Plan files are compiled into the binary, so the
   test is two builds (main and the branch), not a `--swap`.
2. **Check the `fast` count:** compare the keep rule's count with the CSV's on the C5 hands. They should differ on very
   few hands.
3. **Run the sims:** both binaries, the same seed (500000), look-ahead, loose, the deck's own tiers only. Use 10,000 games
   per tier per build: Y'shtola t5; Sheoldred t3 and t4; Aragorn t4 and t5 (P3 and P3b). The same seeds deal the same
   opening hands, so the two runs are close to paired until the first mulligan that differs. Noise is about ±1.3-1.9.
   Report each deck averaged over its own tiers, one number per tier.
4. **Run the practice tests:** the new row shows up in the practice-mode mulligan hint (the row that fired is named).
   Run `cargo test --release` and the `practice/tests/mulligan_hints.rs` cases.
5. **Write up:** adopt only on a clear gain; record the result and the why in each deck's `audit/<deck>/FINDINGS.md`;
   JD merges.

Cost: 12 runs of 10,000 games (5 deck-tier pairs × 2 builds, plus P3b on Aragorn's two tiers). A 10,000-game Thrasios
run at t6 took 45 minutes and the fair tiers are usually faster, so roughly 4-9 hours. It runs as one background queue.
