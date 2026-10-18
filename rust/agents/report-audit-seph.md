# Report: audit-seph (seph, sephfs)

## seph (10 problems) - all fixed (ai/seph.rs, ai/decks.rs, ai/mod.rs, impls/mine.rs, impls/rules.rs, engine/turn.rs)
1. Hornet Queen: `seph::sbomb` rates her 7 (the shared `bomb=` tag is untouched, pool decks play her). Reanimation target, fill target, hardcast, bomb on board.
2. Token blink value: `mine::token_etb_value` (tok=N x 0.6, +0.2 flying, +0.2 deathtouch; Hornet Queen = 4.0) in `etb_value` (seph seat only) and `card_etb_value`.
3. Entomb / Buried Alive bin the missing loop piece first (`seph::fill_pieces`/`fill_targets`, used by `fill_go`, `options`, `seph_fill_resolve`).
4. One threshold: `BOMB_TARGET` 5 (yard, library, fill) and `BOMB_BOARD` 6 (on board, hand discard), both through `sbomb`. Mikaeus/Muldrotha/Hornet Queen in the yard now count.
5. Momentary Blink: tagged `prot=blink` (card_overrides.toml, cards.json regenerated with `cardgen export --offline`), `mine::momentary_blink_cast` (hand or flashback) in `protect`, hand option for value, no plain-spell cast.
6. Grand Abolisher: prio 70 once a loop is one piece away / a reanimation is ready / a bomb is out, else 35; `rules::ability_locked` limits an seph-owned Abolisher to artifact/creature/enchantment abilities (Oracle). Pool Abolishers unchanged.
7. Tireless Tracker: `seph::tracker_first` casts it before the land drop (called from `engine/turn.rs play_land`, seph seat only), unless the mana reanimates.
8. Muldrotha: `sbomb` 6 (hardcast plan, reanimation target).
9. Entomb ranking: no problem (noted by the audit); unchanged.
10. Tutor wish list: unowned cards removed; late list adds Toxic Deluge / Austere Command under pressure, Muldrotha from turn 7, Aura Shards.
Result (adaptive, t3, conservative, 20,000 games, same seeds): 42.2% -> 43.4%; bomb online T6 52% -> 54%. Hornet Queen cast 2.6% -> 4.5% of games, stuck rate 88.5% -> 80.5%; Muldrotha cast-when-drawn 28% -> 41%; Momentary Blink now cast in 17.6% of games (+29 when cast).

## sephfs (7 problems) (impls/sephfs.rs, impls/sheoldred.rs, sim-cli report.rs)
1. Tutors: three designs measured on t5, adaptive AI. (a) Fixed/state wish lists (loop piece, early mana, finisher at 7+ lands, wipe/edict/Ghostform by Sephiroth state, Titan/Demon at 6+ lands; ablated in 8 variants, 20,000-60,000 games, conservative profile, seed 700000): 27.7-29.0% vs 29.8% for the generic pick, noise about 0.3 (SE) per run; all lost 0.8-1.7. (b) Final: `sephfs::tutor_pick`, a board-state pick (finisher only when it kills, sweeper only when opponents are ahead, outlet/Ghostform with Sephiroth out and fodder, mana only when short, else the generic pick): 20,000 games per profile, seed 900000, vs ai-audit: conservative 30.0% vs 29.9%, loose 30.1% vs 29.6% (whole branch, noise about 0.4); no loss against the earlier generic-only build (29.8%). Kept.
2. Gray Merchant: `gray_merchant_prio` (devotion; 90 when lethal on one opponent, else 38 + 4 x devotion, max 84). Ablation: +0.4 pts. Exsanguinate / Butcher / Plunderer / Living Death / Blood on the Snow left alone (cast counts only unproven).
3. Altar + Skeleton + Pawn loop: `altar_loop` / `altar_loop_option` (wins with Sephiroth or Blood Artist / Zulaport / Bastion out, Skeleton may wait in the yard with {1}{B}); last piece cast at 88. Loop wins ~0.9% of games (was 0.1%).
4. Plan: milestone "sephfs" = turn of first flip; report plan text added. Online by T8 18%, at all 26%.
5. Citadel: capped at 30 unless Sephiroth or a free outlet is out. Unmeasured (ablation within noise).
6. Early Vampiric Tutor: no wish list, so the generic pick stays (see 1: the early-mana variant measured worse).
7. Massacre Girl / edicts: audit found nothing wrong; unchanged.
Result (t5, conservative, 60,000 games): 29.6% -> 29.8% (neutral), flips 28.8% -> 28.1% of games.

## Shared code touched (all gated on seph / sephfs seats)
engine/turn.rs `play_land` (Tracker call, key seph); impls/mine.rs `etb_value` (key seph), `card_etb_value`, `loops_blocked` pub(crate); impls/rules.rs `ability_locked` (key seph Abolisher); impls/sheoldred.rs flip milestone (key sephfs); sim-cli report.rs plan text/turn for sephfs; data/card_overrides.toml + cards.json (Momentary Blink only); ai/decks.rs `bomb_on_bf` uses `sbomb`; ai/mod.rs `seph_fill_resolve`.
Not touched: seph.rs `treasures -= 1` area, brain DSL fallback, Teferi's Protection, cardgen audit.

## Checks
cargo build --release --all-targets: no warnings. cargo test --release: all pass (new: tests/cards_audit_seph.rs 12, tests/cards_audit_sephfs.rs 6). `--calibrate within --pool t3 --games 40`: identical to the ai-audit binary.
