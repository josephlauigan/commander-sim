# Report: karn-engines

Code: `rust/crates/sim-core/src/impls/karn_engines.rs` (registered last in `impls/mod.rs`). Tests:
`tests/cards_karn_engines.rs` (18, Part 1) and `tests/cards_karn_verify.rs` (28, Part 2). Verdicts and notes are in
`data/card_overrides.toml`; `cardgen export` was run (cards.json changed for Mystic Forge, Urza's Tome, Urza's Cave,
Expedition Map, Endless Atlas: their compiled text was dropped or fixed).

## Part 1: new cards
| Card | Status | Modeled / not |
|---|---|---|
| Karn, Scion of Urza | Full | walker: +1 (opponent AI gives the card you want less, the other is exiled with a silver counter, `Player::silver`), -1 (best silver card), -2 (Construct, +1/+1 per artifact) |
| Mazemind Tome | Full | scry 1 / draw {2}, page counters, exile at 4 and gain 4 |
| Mirrorworks | Approximate | {2} token copy of a nontoken artifact; skips legends and X-counter cards; artifact lands are not copied |
| Mystic Forge | Approximate | the old compiled text drew a card (wrong). AI casts the artifact/colourless top card through the main-phase options (`card_utility`); {T}, 1 life exiles a land on top. Removal, wipes, X spells and counterspells on top are not cast |
| Urza's Blueprints | Full | echo {6} at upkeep (AI pays with 9+ mana or a hand of 2 or fewer, else sacrifices); the draw was compiled |
| Urza's Tome | Full | draw, then exile a historic card from the graveyard or discard |

## Part 2: the 20 engine cards (what was wrong)
- **Wastes were not basic lands for the engine** (`zones::BASIC_LANDS`): Wayfarer's Bauble, Solemn Simulacrum, Burnished Hart fetched nothing in the Karn deck. Fixed (shared edit).
- **Urza's Cave** was auto-tagged as a fetch land (`f` tag, `c:"A"`): it entered tapped and cracked itself for free. Retagged `c=C`; own land option ({3},{T}, sacrifice: any land, wanting a Tron piece / Workshop / Tomb).
- **Expedition Map**: compiled search found basics only; own option fetches any land (`karn_engines::fetch_land`, `land_want`).
- **Inventors' Fair**: only the mana; added the upkeep life and the {4} artifact tutor.
- **Endless Atlas**: ignored the three-same-name restriction; own option.
- **Rogue's Passage**: only an "Army" creature got it; new first-main-phase option on the biggest ready attacker (power 4+).
- **Hangarback Walker**: cast for nothing, died as a 0/0. Now {X}{X} (`self_cost`, X = a third of the mana), counters via `green_cards::enters_with` (before the 0/0 check), Thopters on death.
- **Batterskull**: no living weapon and no {3} return; both added.
- **Wurmcoil Engine**: Wurm tokens were not artifacts; fixed (counts for Karn, Mirrorworks, Constructs).
- **Pithing Needle**: now stops activated abilities: `rules::ability_locked` (compiled text, card code, walkers) and the land-option loop ask `t6_stax::needle_stops`. Practice-mode humans are not stopped. Approximate.
- **Liquimetal Torque**: second ability added (mark in `DataKey::Artifact` holding the turn; `t5::is_artifact` reads it). The AI never uses it. Approximate.
- **Staff of Domination**: with Basalt Monolith + Rings of Brighthearth the AI draws until a Walking Ballista is in hand/play or 3 cards are left (`staff_draw_down`, `SAFE_LIBRARY`). Karn-core can call `karn_engines::infinite_mana` / `staff_draw_down`.
- Verified correct, no change: Ancient Tomb, Burnished Hart (after the Wastes fix), Hedron Archive, Mana Vault, Meteor Golem, Sol Ring, Solemn Simulacrum, Thought Vessel, Wayfarer's Bauble.

## Shared-file edits (all small)
`state.rs` (+`Player::silver`), `engine/zones.rs` (BASIC_LANDS + Wastes; Wurmcoil Wurms are artifacts), `impls/green_cards.rs`
(`enters_with` adds `karn_engines::enters_with`), `impls/rules.rs` (`ability_locked` asks the Needle), `impls/lands.rs`
(`land_options` skips a Needle-named land), `impls/t5.rs` (`is_artifact` reads the Torque's turn mark), `impls/mod.rs`.

## For the other agents / merge notes
- **Walking Ballista dies as a 0/0 when cast** (`creature_entered` kills a 0/0 before its etb puts the counters; checked with a hand-built cast: it spends all the mana and is gone). karn-threats owns it; the same fix as Hangarback's (`enters_with`) works. I did not touch it.
- `data/cards.json` is generated: on a merge conflict, take either side and re-run `cardgen export`.
- Hangarback's `prio` (44) and `self_cost` are registered here; karn-core's AI plan may want to override the priority.
- FINDINGS (audit/karn/FINDINGS.md) not written: no sim was run (brief: smoke runs only).
