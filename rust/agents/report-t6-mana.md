# Report: Tier 6 batch `mana` (15 cards)

Code: `crates/sim-core/src/impls/t6_mana.rs` (registered in `impls/mod.rs`); tests: `crates/sim-core/tests/cards_t6_mana.rs` (26 tests);
data: `data/card_overrides.toml` + regenerated `data/cards.json`. Engine needs: `rust/agents/engine-needs-mana.md`.

| Card | Status | How |
|---|---|---|
| Chromatic Star | Full | override drops the free `rock` tag; options: crack for {1} (mana of any colour, floating) in main 2 / end window; self_dies draws |
| Desperate Ritual | Full | resolve slot: RRR floating red; splice dropped (no other Arcane shares its deck); prio 0 (deck plan only, like Cabal Ritual) |
| Rite of Flame | Full | RR + R per other copy in any graveyard |
| Simian Spirit Guide | Partial | engine need: hand mana is hard-coded to Elvish Spirit Guide (`partials::hand_mana`) |
| Manifold Key | Full | untap a tapped mana artifact making 2+ (with a spell in hand); {3}: biggest attacker unblockable (DataKey::Unbl, as Amphin Pathmage) |
| Honor-Worn Shaku | Full | dyn_mana_perm 1 + untapped legendary non-creature (or summoning-sick creature) permanents; on_tap_perm taps them |
| Metalworker | Full | dork=C tag override; dyn_mana_perm = 2 x artifact cards in hand |
| Treasonous Ogre | Full | dethrone is the engine keyword; extra_mana unit of red, 3 life per mana above 10 life, untapped Ogre only, ranked after lands; compiled floating-mana ability dropped |
| _____ Goblin | Approximate | ETB RRRR (primer's guaranteed 4 unique vowels; sticker sheets not modeled) |
| Ignoble Hierarch | Full | dork tag + engine exalted were already right; status only |
| Jeweled Amulet | Full | override drops free rock tag; charge via options (notes colour the planner spends), extra_mana unit of the noted colour, counter removed on tap |
| The Mind Stone | Full | options: harness {5}{W},{T} (DataKey::Flipped); end_step uses t2::end_step_flicker |
| Grinding Station | Full | etb hook untaps on any artifact entering; mill ability was already compiled |
| Apple of Eden, Isu Relic | Approximate | take opponent nonland cards (yshtola::take_card), AI casts what it can at once, rest return, owner draws per spell; lands stay; see engine-needs |
| Monumental Henge | Full | enters-tapped rule was already data; added land_options dig (top five, historic to hand, rest bottom shuffled) |

Shared code touched: only `impls/mod.rs` (two lines). Data files: card_overrides.toml entries for all 15; cards.json regenerated with `cardgen export --offline`.
Rituals have prio 0 (the AI casts them only through a deck plan) and float red mana; the Ogre/Amulet/Shaku units rely on `ExtraMana`/`dyn_mana_perm`/`on_tap_perm` slots.
Checks: full `cargo test --release` passes; `cargo build --release --all-targets` has no warnings; `cardgen audit` shows all but Simian Spirit Guide Full/Approximate.
