# Report: karn-practice-abilities

Code: `rust/crates/sim-core/src/human/karn.rs` (new). Wiring: `human.rs` (`abilities_of` adds `karn::perm_abilities`, skips the
AI hook options for cards karn.rs owns), `abilities.rs` (`land_abilities` adds `karn::land_abilities`), `alt.rs`
(`land_alts` empty for owned lands). Visibility only (`pub(crate)`): `impls/karn_cards.rs`, `karn_engines.rs`, `karn.rs`,
`rules2.rs`, `lands.rs`, `ai/decks.rs`. No JavaScript touched.

| Card | Offered now (paid from the pool; choices asked) |
|---|---|
| Bonders' Enclave | {3},{T} draw, only with a power 4+ creature (else says so) |
| Buried Ruin | {2},{T}, sac: pick the artifact card in your graveyard |
| Endless Atlas | {2},{T} draw, only with 3 same-named lands |
| Grim Monolith | {4} untap |
| Inventors' Fair | {4},{T}, sac: search an artifact (3+ artifacts); you pick the card |
| Kuldotha Forgemaster | {T}, sacrifice three artifacts you pick, then pick the artifact to fetch |
| Liquimetal Torque | {T}: pick the nonland permanent |
| Mazemind Tome | scry 1 and draw, page counters |
| Mishra's / Urza's Bauble | pick the player; shows the top card / a random hand card; draw at next upkeep |
| Mishra's Factory / Foundry | animate ({1}/{2}; a land played this turn is summoning sick); pump (land or animated) |
| Nevinyrral's Disk | {1},{T} wipe |
| Oblivion Stone | {4} fate counter (you pick the permanent); {5} wipe |
| Ratchet Bomb | tick; blow (label shows the counters) |
| Soul-Guide Lantern | exile opponents' graveyards; {1} draw |
| Steel Hellkite | {2} +1/+0; {X} (you pick X; needs combat damage this turn, once per turn) |
| Steel Overseer | {T} counters on artifact creatures |
| Treasure Vault | {X}{X},{T}, sac: you pick X |
| Unstable Obelisk | {7},{T}, sac: you pick the permanent |
| Urza's Factory | {7},{T}: 2/2 Assembly-Worker token |
| Walking Ballista | {4} counter; remove a counter: 1 damage to a target you pick |
| War Room | {3},{T}, life = colours of the identity: draw |
| Batterskull | {3} return to hand |

Engine code reused: wipes, Hellkite, Overseer, Atlas, Tome, Torque, Skull, Fair, War Room, Monolith untap. Written new (the engine never
modelled them): Bonders' Enclave, Buried Ruin, Treasure Vault, Urza's Factory, the Baubles' look, Lantern exile, Forgemaster
and Obelisk with the person's choices, Ballista, pumps.

Tests: `play_karn_audit` (now a real test: every Oracle line is offered, conditions set per card), `play_karn_abilities` (26:
each ability used with scripted answers, effect checked).

Not modelled / limits: Oblivion Stone can't mark a land (lands aren't permanents here); Forgemaster fetches non-land artifacts
only; damage from Ballista pings does not accumulate on a creature (engine-wide: only toughness <= damage kills);
Mystic Forge's {T}, pay 1 life ability still shows as "cast <top card>" (pre-existing, not in the 23).
