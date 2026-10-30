# Report: Phase 2, PR 5b: the 161 undeclared splits (branch `phase2-splits`)

Brief: `rust/agents/phase2-splits.md`. Handoff: `audit/phase2/splits-handoff.md`. Commits: 993093a6 (overrides and
known list), b14ecf9b (test, realism.md, handoff).

## Method
For each card: the Oracle text (`data/scryfall_cache.json`), the compiled abilities (`data/cards.json`), the card
code's registration and hook bodies, and the engine paths they touch (`zones::do_etb`/`etb_once`,
`cast::resolve_inner`, `dsl::fire`/`statics`/`ability_options`, `mana.rs`, `turn::crack_fetch`,
`zones::lands_entered`). For each suspected double, a throwaway probe test (deleted afterwards) showed what happens.

## Counts
| Group | Cards |
|---|---|
| Real split, `paths_ok` added | 152 (45 lands, 107 others) |
| Real double, line kept | 9 |
| Unsure | 0 |

`paths_ok` went onto 81 existing override entries. The other 71 got new entries holding only `paths_ok`, which is not
an override (`Entry::is_override`), so card data is unchanged. The known list is now 21 competing, 25 shadowed and
9 splits (55 cards).

## Lands (45)
On every one of these lands, the compiled ability never runs: the sim stores a land as a `Land`, not a permanent, and
`dsl::fire`, `dsl::statics` and `dsl::ability_options` read only `player.perms`. What actually runs:
- **Temples and gain lands:** the engine's `land_etb_fx` scries or gains life once (`zones::lands_entered`).
- **Fetches:** the `f` tag cracks them (`turn::crack_fetch`).
- **Reliquary Tower:** the `nomax` tag gives no maximum hand size.

The card code on each is City of Traitors' `land_etb` check (`t6_lands.rs` `traitors_check`), set on every land
that has none of its own. Two gaps found along the way, neither a double:
- the four surveil lands (Commercial District, Hidden Grotto, Undercity Sewers, Underground Mortuary) never surveil:
  they have no `land_etb_fx`;
- the lands' compiled activated abilities (Boseiju, Castle Ardenvale, Treasure Vault and others) are never offered.

## Real doubles (left in `paths_known.txt`; handed to PR 5a or a follow-up, since fixing them needs measured runs)
| Card | What runs twice, and when | Evidence |
|---|---|---|
| Battle Angels of Tyr | The scryfall `treas=1` tag makes a Treasure whenever it enters, which the card doesn't do. The card code makes the real, conditional Treasure on combat damage. | `zones::etb_once` (ETB_SCRYFALL_TAGS). Probe: +1 Treasure on entering. |
| Requiem Angel | The `tok` tag makes a 1/1 flier on entering. The card code makes the real Spirit when a creature dies. | `etb_once` Tag::Tok. Probe: +1 token. |
| Sifter of Skulls | The `tok` tag makes a 1/1 on entering. The card code makes the real Scion when a creature dies. | Probe: +1 token. |
| Springheart Nantuko | The `tok` tag makes a 1/1 on entering. The card code makes the real Insect on landfall. | Probe: +1 token. |
| Pawn of Ulamog | The `tok` tag (tokp=0) makes a 0/0 on entering, which dies at once, so death payoffs see it. The card code makes the real Spawn on dies. | `etb_once`. |
| Karn, Scion of Urza | The `tok` tag (tokp=0) makes a 0/0 Construct on entering. The card code's −2 makes the real Construct. | `etb_once`, `karn_engines` `KARN_ABILITIES`. |
| Sakura-Tribe Elder | The `lr` tag fetches a basic when the spell resolves. The card code fetches again when it is sacrificed. | `cast.rs` resolve_inner Tag::Lr. Probe: +1 land on cast. |
| Sowing Mycospawn | The cast trigger (`mycospawn_cast`) fetches a land, and the `lr` tag fetches another on resolve. | Probe: +2 lands on cast. |
| Doubling Season | A planeswalker's loyalty is doubled twice: by the engine's name check in `zones::enter` and by Doubling Season's own enter hook (`t3.rs` `doubling`). | Probe: Karn Liberated (6) entered with 24 loyalty instead of 12. Confirmed in the code by the coordinator. No JD deck plays it. |

## Real splits worth a note
- **Lock Auras** (Arrest, Bound in Silence, Coerced to Kill, Encrust, Kasmina's Transmutation, Luminous Bonds,
  Prison Sentence): the `rem` tag attaches the Aura as it enters (`zur::apply_lock`); the `aura` slot is only
  `LOCK_AURA`, with no enter hook of its own.
- **Daze, Flare of Denial:** the compiled `counter_spell` is a no-op in `dsl::run`; the engine counters through its
  response window using the `ctr` tag. The card code is only the alternative cost.
- **Canopy Tactician, Elvish Archdruid:** `dyn_mana_perm` sets the amount of the single mana unit the `dork` tag
  makes; it does not add a second unit.
- **Koma:** the compiled "can't be countered" and the `unc` tag are the same check, as with Hexing Squelcher.
- **Paladin Class:** the compiled anthem is on at every level (the level gate is missing): a gap, not a double.
- **Embercleave:** the card code attaches it twice when flashed in; the second just re-attaches, so nothing doubles.

## Test added
`rust/crates/sim-core/tests/paths_splits.rs`: Jungle Hollow, Scoured Barrens and Thornwood Falls gain 1 life once
and Radiant Fountain gains 2 once; the compiled abilities of Flooded Strand, Boseiju, Castle Ardenvale and Treasure
Vault are never offered; Cryogen Relic draws once as it enters; Canopy Tactician taps for exactly 3.

## Checks
- `cardgen export`: `data/cards.json` unchanged. `cardgen validate`: all decks passed.
- `cargo test --release --no-fail-fast`: 3593 passed, 0 failed, 2 ignored. The known-list test passes with the 152
  lines removed.
- `cargo build --release --all-targets`: 0 warnings.
- No sim-core code changed (only the new test file), so sim results can't change; no sims run.
