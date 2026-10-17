# Report: land-audit2

Audited all 243 nonbasic lands in data/cards.json (tags, enters rules, hook slots) against Oracle text.

## Fixed (tests: rust/crates/sim-core/tests/cards_land_audit2.rs, 12 tests)
| Land | Decks | Fault -> fix |
|---|---|---|
| Twilight Mire, Rugged Prairie, Mystic Gate | meren; aurelia, winota, kraum; brago | filter lands made coloured mana with nothing to pay -> share Fetid Heath's `heath_cols` (t0_zurgo.rs). Feeder must now be an UNTAPPED land; filters feed each other only through a real source (closure), so two filters alone make nothing |
| Sungrass Prairie, Ferrous Lake | (precons, fixed to 1 mana on 10/14) | still made 1 mana alone -> `dyn_mana_land` 0 without another untapped land (lands.rs) |
| Spire of Industry | atraxa, yshtola | any colour free, no artifact needed -> colours only with an artifact, 1 life per coloured mana (`pain` tag) |
| Exotic Orchard | ~25 decks incl. most JD decks | any colour of the identity -> only colours an opponent's land can make (lands.rs `orchard_cols`; opposing Orchards ignored, filter lands counted by printed colours: avoids infinite recursion, found by a stack overflow in the Galadriel run and covered by a test) |
| Cavern of Souls | heliod, krenko, winota | any colour for every spell -> creature spells of the chosen type only (mine.rs, reuses Unclaimed Territory's `territory_colors`) |
| Hall of the Bandit Lord | krenko | free {C} -> costs 3 life per tap, none at 3 life or less |
| Spectator Seating | captain, kraum | tagged any colour -> R or W (carddb.txt) |
| Hidden Grotto | alela | free any colour; its {1},{T} mode nets zero mana -> colourless only (too strict by the fixing, noted) |
| Dryad Arbor | kinnan, marwyn | tapped for G the turn it entered -> enters tapped (summoning sick) |
| Sulfur Falls | captain, kefka, kraum, sauron, stella | TOO STRICT: tagged a painland (1 damage per coloured mana); it is a check land -> pain removed |
| Fabled Passage | galadriel, lord-windgrace, meren, seph, sythis, tatyova, titania, yshtola | TOO STRICT: untapped the fetched land only with 4 other lands; Oracle counts the new land, so 3 others (engine/turn.rs `crack_fetch`) |

Not faults (checked): Horizon lands (pain via `on_tap_land`), Vivid lands, Gemstone Mine, Avengers Tower, Aether Hub, Plaza of Heroes,
Unclaimed Territory, Secluded Courtyard, Tainted lands, Temple of the False God, painlands, Ancient Tomb, Mishra's Workshop, Cradle/Coffers.

## Known approximations left alone (not free mana)
- Terminal Moraine, Blighted Woodland, Krosan Verge, Myriad Landscape, Shattered Landscape: the pool "crack at once" rule gives one tapped basic
  with no activation cost and no later ramp. Roughly a wash (fixing generous, ramp lost); changing it needs an engine redesign.
- Nykthos: the colour is any identity colour (real: the chosen one); only matters outside mono-colour decks (it is in mono decks only).
- Urborg does not make other lands tap for {B}; Gilt-Leaf Palace, Den of the Bugbear, Hive of the Eye Tyrant always enter tapped (strict);
  Castle Garenbrig's six-{G} mode, Cavern's "can't be countered", Hall's haste are not modeled.
- Filter lands: Plains + filter cannot make {R}{R} (sim gives one coloured unit per land); slightly strict.

## Shared-file changes
- engine/turn.rs (crack_fetch): Fabled Passage threshold 4 -> 3 (one line).
- impls/mine.rs: one registration line (Cavern). impls/t0_zurgo.rs: `heath_cols` rewritten, `FILTER_LANDS` added. impls/lands.rs: new `register_audit2`.
- data/card_overrides.toml, data/carddb.txt, data/cards.json (regenerated with `cardgen export`).

## Tests
`cargo test --release --no-fail-fast`: all pass. One pre-existing flake: practice `http_play`
`test_undo_asks_the_last_decision_again_with_the_same_table` fails intermittently on origin/main too (a race, 1 in 3 runs under load).
Build has no warnings. `all_decks adaptive 3` and `lookahead 1` finish with no panic.

## Same-seed before/after (loose, adaptive, seed 7, 300 games per cell, each deck on its my_decks.toml tiers; base = origin/main cd55235)
| deck | before | after |
|---|---|---|
| seph (t4) | 39.3 | 41.0 |
| veyran (t1-t4) | 24.4 | 25.4 |
| sauron (t3-t5) | 29.2 | 29.9 |
| yshtola (t5) | 29.7 | 30.7 |
| galadriel (t1-t2) | 13.7 | 15.7 |
| alela (t1) | 20.3 | 18.7 |
| kefka (t3-t5) | 26.4 | 27.6 |
| titania (t2-t4) | 32.2 | 33.8 |
| sheoldred (t3-t5) | 40.1 | 39.2 |
| cloud (t1-t2) | 18.3 | 18.8 |
| kraum (t3) | 40.0 | 41.7 |
| sephfs (t4-t5) | 36.0 | 35.5 |

Every deck moved by 0.5 to 2 points, all inside the 95% CI (about +-5 per cell): a changed land anywhere (own or opponents') changes the random
stream, so "unchanged" decks do not stay identical. The fixes are small in effect; the clearest signs are Alela (lost Hidden Grotto's free
colour, -1.6), and Galadriel/Titania/Seph/Kefka/Kraum (Sulfur Falls / Fabled Passage were too strict, +1 to +2). No cell is a significant move.
