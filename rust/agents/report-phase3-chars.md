# Report: Phase 3, PR 2: the characteristics pass (branch `phase3-chars`)

Brief: `rust/agents/phase3-chars.md`. Design: `audit/phase3/chars-design.md`. Handoff: `audit/phase3/chars-handoff.md`.
Code: `rust/crates/sim-core/src/engine/chars.rs`, `tests/chars.rs`, `examples/chars_cost.rs`. Commits: 9ce2e44a
(design), d145332d (Stage A), 45fad881 (Stage B), 6e6c5423 (Stage C), 543d9670 (handoff). No sims (JD's rule).

## Design
`Chars { types, creature, subtypes, all_creature_types, colors, abilities_on, keywords, power, toughness }`, built
by `chars(g, m)`, with a reader per layer: `is_creature`, `card_types`, `subtypes`, `has_subtype`, `maskwood`,
`land_types` (layer 4); `colors` (5); `abilities_on`, `has_keyword` (6); `power`, `toughness`, `static_bonus`,
`pt_changes` (7). Computed on demand per layer, with the old fast paths. A cache per state version was rejected:
`Game` has no version counter and `perm_mut` changes permanents in place everywhere, so a stale cache would be easy
to get and hard to see. Building a whole `Chars` costs about 7.9 µs, so the hot readers stay per layer.

## Stage A (d145332d): results identical
The bodies of `Game::is_creature`, `epow`/`etgh`, `static_bonus`, `dsl::pt`, `dsl::has_kw`, `subtypes`, `ptypes`,
`has_type`, `maskwood` and `colors_of` moved verbatim into `chars.rs`; the originals are thin wrappers. New tests
compare `chars()` with the old readers on: anthems + counters + equipment, Elesh Norn, Maskwood Nexus, an animated
land, Mutavault animated, Blood Moon, a copy with its exceptions, a neutered Serra Angel.

## Stage B (45fad881): results identical
`chars::NAMED`, one table of 8 cards with their layer: Maskwood Nexus, Yavimaya, Urborg (4); Colossus Hammer (6);
Wrecking Ball Arm, Hulkbuster Armor (7b); Commander's Insignia, Auriok Steelshaper (7c). Each card's entry is looked
up once at load (`CardDef::named`). Deleted: `cloud::base_pt`, `cloud::steelshaper_bonus`, the Yavimaya and Urborg
scans, the Maskwood and Insignia name checks.

## Stage C (6e6c5423): behaviour changes, each tested
1. A permanent that lost its abilities now changes nothing (layer 6 before 7): its tagged or compiled anthems,
   equipment bonuses, keyword grants and self statics no longer apply, and a neutered Equipment gives nothing.
2. Always Watching and Thraben Watcher pump again: cardgen compiled "nontoken creatures" as subtype "nontoken", so
   the anthem pumped nobody. It now writes `nontoken: true` (`dsl_parse.rs`); these 2 cards change in `cards.json`.
3. Land types come from one function, `chars::land_types` (check lands, Cabal Coffers / Crypt Ghast, Forest counts,
   forestwalk, Krenko's Mountains): under Blood Moon a nonbasic dual is a Mountain only; Urborg stops under Blood
   Moon; Yavimaya's Forest and Urborg's Swamp count for check lands.
4. Adeline equipped with Wrecking Ball Arm is 7/7.

## Checks
- `cargo test --release --no-fail-fast`: 3640 passed, 0 failed, 3 ignored (rerun by the coordinator). Build 0
  warnings. Practice 104/104. Export (the 2 cards) + validate pass; 0 doubles; `paths_known` empty.
- `chars_cost` (ns per permanent, all seven readers): 911-930 against 923-935 at the base; `has_type` 22 → 25 ns.
  `clone_cost` 2.95-3.02 µs against 2.95-3.04. Not slower.

## Decks touched by Stage C (for the recalibration)
- Neutering: Darksteel Mutation (light-paws t1, sythis t2, zur t5, **JD's thrasios**); Kenrith's Transformation
  (sythis t2, marwyn t3); Reprobation (light-paws t1).
- Always Watching, Thraben Watcher: giada t0.
- Blood Moon: krenko t4 (and the decks at its table that use Coffers, Crypt Ghast, check lands or forestwalk).
- Urborg: lord-windgrace t2, meren t2, tergrid t3, edgar t4, yawgmoth t5. Yavimaya: lord-windgrace t2, marwyn t3,
  chulane t4.

## Not fitted yet
- The keyword booleans on `Perm` (`fly`, `dt`, `vig`, `lifelink`; 73 reads of `.fly`) skip layer 6.
- P/T-setting effects (Darksteel Mutation 0/1, the elk, face-down 2/2) write `pow`/`tgh` in place: no timestamp,
  can't be undone. `pump` takes the max after every other change (no card uses it).
- A face-down permanent keeps its colours and subtypes. The stand-in Mutavault token isn't every creature type.
- Creature lists (`player.perms`) miss lands that are creatures (Dryad Arbor, a t6 deck).
- `dsl::statics` still reads neutered sources for other kinds (cost reducers).
- Cards off the battlefield (`galadriel::card_is`, `epow_card`) keep their own readers.
