# Phase 3, PR 2: the characteristics pass (design)

Branch `phase3-chars`, stacked on `phase3-arena`. Brief: `rust/agents/phase3-chars.md`. Handoff:
`audit/phase3/chars-handoff.md`.

## The struct and the function

`engine/chars.rs`:

```rust
pub struct Chars {
    types: Types,            // card types (a copy's added types; a token is a creature)
    creature: bool,          // a creature now (animated, NotCreature)
    subtypes: Vec<Sym>,      // lower case
    all_creature_types: bool,// Maskwood Nexus, Mirror Entity
    colors: Colors,
    abilities_on: bool,      // not neutered (Tidebinder, Darksteel Mutation, Kenrith's Transformation, face-down)
    keywords: Vec<Sym>,      // which of `KEYWORDS` (the 18 the engine asks about) it has
    power: i32, toughness: i32,
}
pub fn chars(g: &Game, m: PermId) -> Chars
```

Each layer also has its own reader, and the old functions are thin wrappers of them:

| old (callers)                                   | reader in `chars`           | layer |
|-------------------------------------------------|-----------------------------|-------|
| `Game::is_creature` (~910)                      | `is_creature`               | 4     |
| `values::ptypes`                                | `card_types`                | 4     |
| `values::subtypes`, `values::has_type` (~150)   | `subtypes`, `has_subtype`   | 4     |
| `values::maskwood`                              | `maskwood`                  | 4     |
| (new) land types                                | `land_types`                | 4     |
| `values::colors_of`                             | `colors`                    | 5     |
| `p.neutered` checks                             | `abilities_on`              | 6     |
| `dsl::has_kw` (~90)                             | `has_keyword`               | 6     |
| `values::epow`, `values::etgh` (~400)           | `power`, `toughness`        | 7     |
| `values::static_bonus`, `dsl::pt`               | `static_bonus`, `pt_changes`| 7c    |

## Effects onto layers

1. Copy: `Perm::cd` is the copiable values (a copy's `cd` is the copied card), `Perm::exc` the copy's exceptions
   (`add_types`, `add_subtype`, `only_subtype`, `pt`, `colors`, `not_legendary`): already applied as the copy
   enters (`zones::enter` with `CopyOf`), read by layers 4, 5 and 7.
2. Control: `Perm::owner` is the controller; not a characteristic here.
3. Text: nothing in the pool.
4. Type: printed types; `DataKey::Anim` (animation: manlands' stand-in tokens, Yuriko's, Stella's ...);
   `DataKey::NotCreature` (Enduring Vitality's return); changeling; the copy's `only_subtype`; token subtypes;
   Maskwood Nexus and Mirror Entity's "all types"; land types: Blood Moon / Magus of the Moon (nonbasic lands are
   Mountains only), Yavimaya (every land a Forest), Urborg (every land a Swamp).
5. Colour: the copy's colours, devoid, the card's or the token's.
6. Ability: `neutered` (abilities lost); Blood Moon removes a nonbasic land's abilities (`mana::moon_land`); keywords
   from the card, the token, eot grants, Role, suspect, Elspeth's emblem, Auras, card code (`granted_kw`), the
   ability language (`self_keyword`, `keyword`, `equip_keyword` statics); Colossus Hammer removes flying.
7. Power and toughness: 7a Adeline (power = creatures you control); 7b Wrecking Ball Arm / Hulkbuster Armor (base
   7/7, 9/9), set-P/T effects that write `Perm::pow/tgh` in place (Darksteel Mutation 0/1, Kenrith's elk 3/3,
   face-down 2/2, the copy's `exc.pt`); 7c counters (`plus`), tagged anthems (`static_bonus`), Sword, Elesh Norn's
   -2/-2, eot pumps, Auras, compiled anthems and equipment, Steelshaper, self-scaling, Mouth of the Storm,
   `pumpadd`; `pump` ("base power X/X", the `set_pt` effect: no card in `cards.json` uses it today).

## The table (Stage B)

`chars::NAMED`: one table of the card names whose presence changes other permanents' characteristics, each with its
layer and effect. Stage B moves these name checks into it: Maskwood Nexus (4: all creature types), Yavimaya (4:
lands are Forests), Urborg (4: lands are Swamps), Colossus Hammer (6: loses flying), Wrecking Ball Arm and
Hulkbuster Armor (7b: base P/T), Commander's Insignia (7c: +1/+1 per command-zone cast), Auriok Steelshaper (7c:
Soldiers and Knights +1/+1 per equipped Steelshaper).

## Speed

Measured with `examples/chars_cost` (new, not a sim: 94 permanents and lands, every reader on every one, 2000
rounds) at the base: is_creature 1.5 ns, power 270-345 ns, toughness 231-236 ns, has_type 22 ns, subtypes 15 ns,
colors_of 1.3 ns, has_kw 407-417 ns, all seven 923-935 ns per permanent.

Building the whole `Chars` each time would cost ~18 keyword checks (~7 µs) where `is_creature` costs 1.5 ns, so
the readers stay per layer, on demand, with the old fast paths. A cache per state version was rejected: `Game` has
no version counter, and `perm_mut` changes permanents in place at ~2000 sites, so a stale cache would be easy to
get and hard to see. `chars()` is for tests, logs and practice displays.

## Layer bugs found (Stage C)

1. Abilities lost (layer 6) don't stop a source's characteristic statics: a neutered permanent's auto-tagged anthem
   (`Tag::Anth`, `static_bonus`) and its compiled `anthem` / `equip_bonus` / `bma_anthem` / `keyword` /
   `equip_keyword` / `self_keyword` / `self_scaling` statics still apply (Tidebinder on an anthem, Darksteel
   Mutation on Beastmaster Ascension's creature ...). The tagged ones (`has`) and card code (`hooked`) already stop.
2. Blood Moon vs land types (layer 4): a nonbasic land with a basic type (shocks, duals, Dryad Arbor) still counts as
   a Forest / Swamp under Blood Moon (`t3_marwyn::is_forest`, `common::is_swamp`); Urborg under Blood Moon still
   makes every land a Swamp (Yavimaya already stops).
3. 7a vs 7b: Adeline equipped with Wrecking Ball Arm gets creatures + 7 - printed power, not 7.

## What does not fit yet

- The keyword booleans on `Perm` (`fly`, `dt`, `vig`, `lifelink`, 73 reads of `.fly` alone) are set when a
  permanent is made and read directly by combat; they bypass layer 6.
- Set-P/T effects write `pow` / `tgh` in place (no timestamp, no undo); layer 7b is only the equipment.
- `pump` takes the max after every other change (a layer 7b effect applied last); unused by any card today.
- Colours and types of a face-down permanent (it should be colourless, no subtypes): only P/T and abilities are set.
- Lists of creatures (`player.perms`) miss lands that are creatures (Dryad Arbor in `player.lands`): attacks, board
  power, Adeline's count. A list question, not a characteristics one (Phase 3 follow-up: one battlefield list).
- Protection (`prot_colors`), untargetability and indestructible stay in `values` (they read `has_keyword`).
- Cards not on the battlefield (`galadriel::card_is`, `epow_card`) have their own readers.

## Done (Stages A-C)

- Stage A (d145332d): `engine/chars.rs`; the old readers are thin wrappers; `tests/chars.rs` compares `chars` with
  them on the tricky boards. Results identical (3632 passed).
- Stage B (45fad881): `chars::NAMED`, looked up once per card at load (`CardDef::named`, so a reader compares one
  enum, not names); the name checks in `cloud` (Hammer, base P/T, Steelshaper), `t3_marwyn` (Yavimaya),
  `common` (Urborg), `values` (Maskwood) and the Insignia check are gone. Results identical (3635 passed).
- Stage C: the three layer bugs above are fixed, plus a fourth found by the Stage A tests: Always Watching and
  Thraben Watcher's anthem was compiled with the filter subtype "nontoken" and pumped nobody (cardgen now writes
  `nontoken: true`). Land types read one function, `chars::land_types` (check lands' enter rules, Cabal Coffers /
  Crypt Ghast's Swamps, forestwalk and Forest counts, Krenko's Mountains). A neutered Equipment (tagged ones) gives
  no bonus or protection either (`values::equipped`).

Speed after Stage C (`chars_cost`, ns per permanent): is_creature 1.5, power 283-348, toughness 236-238, has_type 25,
subtypes 15, colors_of 1.4, has_kw 398-402, all seven 911-930 (base 923-935): no slower. `has_type` 22 -> 25 ns
(Maskwood through the table). `clone_cost` 2.95-3.02 µs (base 2.95-3.04).
