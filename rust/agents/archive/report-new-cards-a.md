# report new-cards-a
All three cards Full (`cardgen audit --cards` shows them; `audit --deck X` lists only deck cards, and these are swap-ins).
- **Primeval Titan** (impls/titania.rs): enter/attack search of up to two lands, tapped, each a landfall, shuffle. Titania
  deck takes fetch lands first, then nonbasics, then basics; others take most colors. New `DataKey::TitanTapped` (state.rs):
  a Titan-fetched fetch land is skipped by `fetch_lands` while still tapped (its {T} cost), so Elementals come next turn.
  Prio 72 with Titania out and a fetch land left in the library, else 64. Preflight: cast 88.3% of draws.
- **Oppression** (impls/kefka.rs): `cast` hook, caster discards via `discard_worst` (Kefka's discard_choice for Kefka).
  Prio 66 with a discard payoff and opponent cards, 58 when opponents hold at least Kefka's spare cards, 25 otherwise,
  0 only with a combo turn (storm/Breach/Will piece + 4 other spells + 6 mana). Preflight: cast 58.3% of draws. The
  static "priority 0" warning no longer shows.
- **Andúril, Narsil Reforged** (impls/cloud.rs): attack hook, +1 (blessing: +2) counters on each creature through
  `add_counters`; blessing = 10+ permanents incl. lands, remembered in player stats ("city_blessing"). Cloud doubles it.
  Equip {3} via dsl `equip_cost`; `equip_utility_bonus` +2. Prio 79 (66 with no team). Preflight: cast 91.5%.
Data: carddb.txt (3 lines), card_overrides.toml (3 entries; Oppression/Andúril have dsl), cards.json re-exported,
scryfall_cache.json gained the 3 cards. Merge note: new-cards-b edits the same data files; re-run `cardgen export`.
Tests: one or more per card in cards_titania/kefka/cloud.rs. `cargo test --release` and all-targets build clean.
