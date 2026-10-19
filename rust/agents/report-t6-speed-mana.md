# Report t6-speed-mana (2026-10-19)

Branch t6-speed-mana (from t6-speed). Raw output: `audit/t6/speed-2026-10-19/mana-base600.txt` (stats only, no rule change),
`mana-v1-600.txt` (first rule set), `mana-v2-600.txt` (final), `mana-base-1000.txt` / `mana-v1-1000.txt` (seed 700000, paired).

## Bottom line
Mana is **draw-limited, not hold-limited**. Fast mana that can be cast is cast (T1: 0.55-0.95 mana cards cast per game);
what stays in hand is gated cards (Lion's Eye Diamond, rituals, Mox Diamond without a spare land, Chrome Mox without a spare
card). The shared "mana first" rule I added moves mana permanents by +0.0 to +0.15 on turn 3 and **no win median or win %
moves outside noise**. Median win round is still 7-13. The lever for 4-6 mana on turn 2-3 is not the AI's sequencing.

## Code (stats only, plus one small Tier 6 rule)
- `t6stats.rs`: `mana_snapshot` (start of main after the land drop, turns 1-5: mana, mana permanents, mana cards in hand and
  castable), `mana_cast` (mana cards cast on turns 1-3), `mana_held` (end of main 2, turns 1-3: castable mana cards still in
  hand, by name; rituals apart). `engine/turn.rs`: two calls, both gated on `t6`. `t6report.rs`: `render_mana` + 1 test.
- Rule (Tier 6 only; `must_option` is only called for Tier 6): `ai/t6_table.rs::mana_first`: on turns 1-3 a mana creature
  (cmc <= 2, Dork tag) or a three-mana rock that the deck plan has no opinion about is a "must" play (score 88, below
  commanders and plan priorities of 90+). `ai/pool.rs` (Tier 6, turns <= 3): Chrome Mox needs one spare card to imprint, not two.
- Tried and reverted: Mox Diamond with only two lands in hand+play (turn <= 3). It breaks `ai_t6_keeps`
  (`a_two_land_hand_does_not_cast_mox_diamond...`, the keeps agent's deliberate rule) and showed no gain.

## Mana by turn (mean over the deck's games, conservative profile, seed 500000, 600 games per profile)
Mana at the start of the main phase, after the land drop, before casting; "perm" = mana permanents (rocks, dorks, moxen) in play then.
| Deck | T1-T5 mana before | T1-T5 mana after | perm T3 before -> after | mana cards cast T1/T2/T3 before -> after |
| --- | --- | --- | --- | --- |
| Dack | 0.99 2.54 3.75 4.72 5.58 | 0.98 2.52 3.77 4.79 5.58 | 0.81 -> 0.82 | .54 .37 .29 -> .55 .37 .29 |
| Kinnan | 1.06 2.64 4.18 5.33 6.07 | 1.06 2.67 4.31 5.41 6.19 | 1.11 -> 1.20 | .85 .41 .44 -> .89 .45 .45 |
| Kraum/Tymna | 1.03 2.75 3.52 4.20 4.90 | 1.03 2.76 3.53 4.21 4.89 | 0.76 -> 0.76 | .69 .23 .15 -> .70 .21 .14 |
| Rograkh/Silas | 1.06 2.67 3.49 4.08 4.75 | 1.06 2.65 3.55 4.01 4.63 | 0.98 -> 0.99 | .80 .33 .24 -> .80 .35 .25 |
| Rograkh/Thrasios | 1.01 2.67 3.71 4.61 5.60 | 1.01 2.68 3.66 4.59 5.61 | 0.67 -> 0.66 | .56 .19 .20 -> .58 .18 .20 |
| Sisay | 1.02 2.72 3.66 4.66 5.60 | 1.02 2.76 3.74 4.67 5.57 | 0.83 -> 0.93 | .68 .25 .27 -> .73 .32 .31 |

## Ready / go / win and win % (conservative; medians in rounds), 600 games, seed 500000
| Deck | ready before -> after | go before -> after | win round before -> after | win % before -> after |
| --- | --- | --- | --- | --- |
| Dack | 7 (22%) -> 7 (20%) | 6.5 -> 7 | 7 -> 7 | 27.7 -> 24.7 |
| Kinnan | 7 (25%) -> 7 (27%) | 7 -> 7.5 | 9 -> 8 | 26.4 -> 26.7 |
| Kraum/Tymna | 7 (51%) -> 7 (54%) | 9 -> 8 | 10 -> 9 | 31.2 -> 32.5 |
| Rograkh/Silas | 7 (37%) -> 6.5 (40%) | 8 -> 7 | 8 -> 8 | 19.4 -> 21.7 |
| Rograkh/Thrasios | 10 (14%) -> 10 (16%) | 10 -> 10 | 12 -> 13 | 19.0 -> 16.6 |
| Sisay | 8 (12%) -> 8 (13%) | 8 -> 8 | 12 -> 13 | 26.7 -> 28.4 |
The win % moves are within the +-3.5 point noise of 400 seats (v1, which also had the Mox Diamond change, on a different
seed with 1000 games: Dack 27.5 -> 32.0, Kinnan 27.5 -> 26.2, Kraum 28.7 -> 30.3, Silas 22.2 -> 22.7, Thrasios 18.1 -> 16.7,
Sisay 25.8 -> 22.1: no consistent sign). Kraum's median go/win round drop (9 -> 8, 10 -> 9) appears in all three runs but is
not caused by this rule (its plan sets fast mana to 95 already); it is the same drift the other runs show for that deck.
Not run: the 2000-game confirmation (no effect to confirm).

## Why mana is late (what the numbers say)
1. **Supply.** A kept hand of 7 plus 2 draws holds 1.0-1.6 mana cards at turn 1 and 0.4-0.9 castable at turn 2; the decks
   run ~15-18 mana cards in 99 and ~22 lands. T2 mana is 2.5-2.8, T3 3.5-4.3, T4 4.1-5.4, T5 4.6-6.3. Real cEDH gets
   4-6 on turn 2-3 from 25+ mana sources and mulligans; here 38-55% of hands are mulliganed already (strict keeps, see
   `report-t6-ai-keeps.md`), so the next lever is deck composition and hand quality, not play order.
2. **What is held (castable, end of turns 1-3, card-turns over 600 games; `held at the end of turns 1-3` line per deck):**
   Lion's Eye Diamond ~90-96 per deck (intended: it is a Breach/storm piece), Mox Diamond 33-62 (Kinnan 62, Silas 57,
   Sisay 39, Thrasios 33; the keeps agent's gate needs 3 lands incl. the discard), Chrome Mox 11-24 (needs two spare
   cards), rituals held for the go turn (Rite of Flame, Culling the Weak, Cabal/Dark Ritual: by design, prio 0).
   Dorks and rocks that are castable were nearly never held (Elves/Hierarchs/Pilgrim 4-8 card-turns).
3. **The look-ahead did vote dorks down** (traces: Sisay T3 "Ignoble Hierarch 83% ... -> Touch the Spirit Realm [search]",
   T1 "Hierarch 95% -> Agatha's Soul Cauldron [search]"); `mana_first` fixes that, which is where Kinnan/Sisay's +0.1 perm
   comes from.
4. **Rograkh on turn 1 is countered or declined by the AI when an opponent has Vexing Bauble out** (game 1: Dack's Bauble;
   a {0} spell is countered): legal play, not a bug. Rograkh/Thrasios' "stop [search]" on T1-T2 there is that.
5. **Models checked, no rules error found:** Lotus Petal (sacrificed on tap, `rules.rs`), Mox Opal (metalcraft) and Mox Amber
   (legend) via `dyn_mana_perm`, Chrome Mox (imprint colours, skips line pieces), Mox Diamond (discard the lowest-key land),
   Ancient Tomb (2 mana, 2 damage, off at 2 life), City of Traitors (sacrificed on another land), Gemstone Caverns,
   Mana Vault / Grim Monolith / Basalt Monolith (rock tag + nountap / untap), Elvish and Simian Spirit Guide (hand mana,
   `HAND_MANA`), Gaea's Cradle (`dyn_mana_land`). Jeweled Lotus is a one-shot (not in `RITUALS` of the stats list as a perm).

## Checks
- t3 lock: LOCK IDENTICAL (final binary; `rust/agents/lock.sh`).
- Tests: `cargo test --release -p sim-cli t6report` 6 pass (1 new); sim-core `t6_stats ai_t6_must ai_t6_keeps ai_t6_table
  ai_t6_dack ai_t6_kinnan ai_t6_sisay ai_t6_rograkh_silas ai_t6_rograkh_thrasios ai_t6_kraum` all pass (the full
  `cargo test` was not run).
- `cardgen export --offline` left `git status` clean (no card data changed).

## Suggestions (not done)
- The remaining speed is in deck composition and hand quality: more cheap mana in the 99 (JD's lists are physical, so this is
  a deck question), or a keep rule that favours fast mana over land count. A fresh keep test with the new `mana T1-T5`
  line is the right instrument.
- If JD wants turn-2/3 power, rituals must count as mana in `ready` (they are held for the go turn by design).
