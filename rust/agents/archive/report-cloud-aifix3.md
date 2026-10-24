# Report: cloud-aifix3 (claude-sonnet-5-5)
Commits: F1 9dd4df7, F2 f054bed, F4 a0c485d, F8 a67caee. Tests in `tests/cards_cloud_aifix3.rs` (11).
- F1: `cloud::must_attack` (Cloud can attack, wears a Sword/Scope/Shuriken, no untapped blocker that kills him): `search::choose_attack_inner` drops the "none" plan; `keep_cloud_home` lets such a Cloud go. Cloud key only.
- F2: `cloud::riders_spare`; `bulk_white::riders_etb` skips an equipped Cloud (cloud deck only).
- F4: `cloud_pick` order: FF first with hand >= 4, FF before HH once FI/HH is held, else unchanged. cards_cloud.rs expectation updated (next Sword is now FF).
- F8: UB was modeled and Full, but `cloud_prio` was 0 unless Cloud (never: command zone) or a bomb sat in the graveyard. Now 70-76 with creature + Equipment, 60 bomb, 45 cmc>=3 creature. Recommission already casts (test added).
- Shared-file changes: ai/search.rs (3 lines), impls/bulk_white.rs (1 line).
- Checks: full cargo test pass, --all-targets no warnings; same-seed identity vs origin/main (300 games, adaptive, loose, all, seed 7) for the 10 decks identical (veyran differs only in progress timing lines). Cloud smoke 200 lookahead: 14.0% t1 / 13.0% t2 (not a measurement).
