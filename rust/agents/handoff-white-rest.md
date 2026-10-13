# Handoff: white-rest

## Done (commit 2274ac7 and after)
All cards of the brief are modeled in `impls/bulk_white.rs`, tested in `tests/cards_bulk_white.rs` (29 tests), and
`cardgen audit --mine` lists every one as Full (overrides with notes appended at the end of data/card_overrides.toml).

## For the cloud-core / elrond-core agents (hooks they can use)
- `bulk_white::reenter_value(g, p, m) -> f64`: what it is worth to have permanent m leave and come back (Cloud: +6 while an
  Equipment is in the library). Ennis, Stormfront Riders, Forum Familiar, Unyielding Gatekeeper and Exit Specialist pick
  their targets by it; a deck plan may add to it.
- `bulk_white::ennis_target(g, p, ennis)`: Ennis's pick (Cloud first).
- `bulk_white::white_protect` and `white_wipe_response` are wired for key "cloud" in `ai/decks.rs` (protect_response,
  wipe_response); Patriot, Shardmage's Rescue, Cho-Manno's Blessing and Ultimate Magic: Holy answer removal there.
- Face-down creatures: `down_options` (a `hand_options` slot on the four cards) and `face_up_options` (called for every
  deck from `brain::hook_options`), so no deck plan has to offer them.
- Holy, Shardmage's Rescue, Cho-Manno's Blessing, the recursion spells and Repel the Darkness have `prio` 0: they are
  cast only by their own options or as responses. cloud_prio must not override that.

## Shared-file changes (all small)
state.rs (DataKey::ChosenColor, Entered), engine/zones.rs (bounce: Riders hook; to_zone_card: note exile),
engine/life.rs (prevents_damage: Holy shield), engine/removal.rs (legal_targets: Lay Down Arms mana value),
impls/common.rs (attached_prot: chosen colour), impls/green_cards.rs (extra_ward calls bulk_white::extra_ward),
ai/brain.rs (hook_options: face_up_options), ai/decks.rs (cloud arms in protect_response and wipe_response),
cardcode.rs (evasion_blocked: Exit Specialist).

## Gotchas
- `perm()` in tests runs enter triggers: a Stormfront Riders put in with `perm` bounces itself; use `g.skip_etb = true`.
- Disguise cards had ward 2 from Scryfall's reminder text even face up: overrides set `ward = 0`; face-down ward is
  `extra_ward`. Greenbelt Radical (not mine) still has that wart.
- A face-down creature's flags (fly, lifelink ...) are cleared on cast and restored from the tags on turning up.
