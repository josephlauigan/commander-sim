# Report: t6 redirects (14 cards, one commit each)

Code `rust/crates/sim-core/src/impls/t6_redirects.rs`, tests `rust/crates/sim-core/tests/cards_t6_redirects.rs` (30 tests), data in `data/card_overrides.toml`, engine needs in `rust/agents/engine-needs-redirects.md`. Nothing shared touched except `impls/mod.rs` (two lines). Full `cargo test --release` passes; `cargo build --release --all-targets` has no warnings.

| Card | Status | Missing |
|---|---|---|
| Redirect Lightning | Partial | engine never calls `redirect_removal`; no re-aim of abilities or spells on the stack |
| Ricochet Trap | Partial | same hook ({R} after an opposing blue spell is worked out from the turn's cast list) |
| Untimely Malfunction | Partial | same hook; AI casts it only at an artifact (can't-block mode is in the resolve) |
| Pyrokinesis | Partial | alternative cost hook (`pay_pyro_pitch` ready); tags `rem=dmg4 tgt=c` added |
| Scout's Warning | Partial | engine does not read `scout_flash_active` |
| Kozilek's Command | Full | tags `tokx rem=exile tgt=c draw=1` added (X = spare mana) |
| Eldrazi Confluence | Full | tags `rem=shrink3 tgt=c` added |
| Get Lost | Partial | Map tokens; AI aims only at creatures |
| Erode | Full | |
| Soul Partition | Partial | stand-in: card returns to owner's hand; no play-from-exile, no {2} tax |
| Calamity's Wake | Partial | engine does not read `noncreature_lock` |
| Emerald Charm | Partial | lose-flying mode |
| Sunscour | Partial | alternative cost hook (`pay_sunscour_pitch` ready); wipe is the existing tag |
| Gleaming Splendor | Full | tags cleared (the old `treas=1` tag gave a Treasure on entering) |

Notes: {C}{C} costs are paid as generic mana (card data has no colourless pips). Toughness 0 is not a state-based check in `check_state`: kill with `die(g, m, "sba")`. Shared engine needs across batches: per-card alternative cost for hand spells; removal-response hook calling `redirect_removal`; per-turn "can't cast" flag.
