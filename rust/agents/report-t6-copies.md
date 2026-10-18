# Report t6 copies (16 cards)
Code: `rust/crates/sim-core/src/impls/t6_copies.rs` (registered last-but-one in `impls/mod.rs`), tests `rust/crates/sim-core/tests/cards_t6_copies.rs` (18 tests), data in `data/card_overrides.toml` (notes and statuses; 0/0 stand-ins 1/1) and `data/cards.json` (re-exported).

| Card | Status | Missing |
|---|---|---|
| Twincast | Full | |
| Flare of Duplication | Full | |
| Heat Shimmer | Full | |
| Molten Duplication | Partial | token is not an artifact in addition |
| Electroduplicate | Full | |
| Clever Impersonator | Full | stand-in's own enter triggers fire once (see engine needs) |
| Mockingbird | Partial | Bird type |
| Phantasmal Image | Partial | Illusion type, sacrifice when targeted |
| Flesh Duplicate | Partial | vanishing 3 |
| Copy Artifact | Partial | enchantment type |
| Mirrormade | Full | |
| Sculpting Steel | Full | |
| Imposter Mech | Partial | vehicle/crew, copy loses types (AI does not cast it) |
| Flash Photography | Full | |
| Peter Parker's Camera | Partial | the ability-copy (3 film counters only) |
| Volatile Stormdrake | Partial | hexproof from abilities only |

Engine needs: `rust/agents/engine-needs-copies.md`.
Shared files touched: none besides `impls/mod.rs` (two lines). `cards.json` re-exported by `cardgen export --offline` (only the three clone cards' 1/1 change).
Reused: `marchesa::steal`, `enter_token_copy`, `copy_spell`, `EotSac`, the `fb` tag, Dualcaster's hand_cast pattern, Copy Enchantment's leave-and-enter pattern.
Process note: all cards were committed together, not one per commit (one module and one test file).
