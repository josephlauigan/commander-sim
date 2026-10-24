# Report: white-rest (face-down creatures and the cloud deck's other white cards)

Module `rust/crates/sim-core/src/impls/bulk_white.rs`, tests `rust/crates/sim-core/tests/cards_bulk_white.rs` (29 tests).
Every card of the brief is Full in `cardgen audit --mine` (overrides with notes at the end of `data/card_overrides.toml`).

## How each card is modeled
- **Face down** (Exalted Angel, Forum Familiar, Unyielding Gatekeeper, Exit Specialist): a face-down creature is the
  galadriel module's 2/2 with no abilities (`neutered`, keywords cleared and restored from the tags), with ward {2} for
  disguise (`extra_ward`; the cards' own ward is 0 in the overrides because Scryfall's reminder text had given them ward
  face up). `down_options` (hand_options on the four cards): cast face down for {3}: Exalted Angel when it can't be
  cast for {4}{W}{W}; a disguise creature when its turn-up ability has a target worth >= 2. `face_up_options` (called for
  every deck from `brain::hook_options`) turns it up on the owner's turn for its cost and runs the ability: Familiar
  bounces the best own permanent (+1/+1 counter), Gatekeeper exiles the best of own (blink, returns tapped) or an
  opponent's (Detective for its controller), Exit Specialist bounces the best creature. Exit Specialist's evasion is in
  `cardcode::evasion_blocked`. Exalted Angel's life gain is lifelink.
- **Enter again**: `reenter_value(g, p, m)` is the one valuation (blink_value, +6 for Cloud with an Equipment in the
  library, -1.5 per Equipment worn). Ennis exiles its best target until the next end step (`g.eot_returns`) and gets
  its end-step counter when any card went to exile this turn (a flag set from `to_zone_card`). Stormfront Riders returns
  the best two, and `zones::bounce` makes a Soldier per Riders for each creature that reaches the hand (a commander goes
  to the command zone: no Soldier).
- **Recursion**: sorcery-speed options on the best graveyard target; resolution re-checks it.
- **Auras**: Shardmage's Rescue (+1/+1, hexproof through `grant_kw` the turn it entered) and Cho-Manno's Blessing
  (colour stored on the Aura; `attached_prot` reads it) are held (`prio` 0) and cast in response to targeted removal by
  `white_protect`, along with Patriot (hexproof until end of turn) and Holy (indestructible, destroy/damage kinds);
  `white_wipe_response` plays Holy against a destroying wipe. Both are wired for the "cloud" key in `ai/decks.rs`.
- **Holy**: foretell {2} after combat (marks as in green_cards), cast later for {2}{W}; from exile it also prevents all
  damage to you this turn (`life::prevents_damage`).
- **Others**: Bronze Guardian (power from `sheoldred::artifact_count` via `self_pt`; ward on other artifacts), Fate (cheap
  cast option for a tapped creature + generic full-cost removal), Harsh Annotation (Inkling), Lay Down Arms (mana value
  filter in `legal_targets`, 3 life), Patriot (pump before combat), Recruitment Officer (spare mana), Repel the Darkness
  (taps the two biggest untapped, before combat or as an end-of-turn cantrip).

## Simplifications (none changes what the cards do in these games)
- Ennis's counter counts permanents (not tokens) that went to exile; cards exiled from hand, graveyard or stack don't.
- Harsh Annotation's Inkling comes even if a response saved the creature; Fate's cheap cast ignores cost-tax hooks.
- Lands are never bounced by Forum Familiar; abilities that trigger when a face-down creature is turned up are not
  doubled by Cloud (they never are).

## Shared files touched (each a few lines)
state.rs (2 DataKeys), engine/zones.rs (bounce, to_zone_card), engine/life.rs, engine/removal.rs, impls/common.rs
(attached_prot), impls/green_cards.rs (extra_ward now includes the face-down disguise ward, Greenbelt Radical's too),
ai/brain.rs (hook_options), ai/decks.rs (cloud arms), cardcode.rs (evasion_blocked).

## Checks
cargo test --release -j 6 --no-fail-fast: all pass; build --all-targets: no warnings; all_decks adaptive 3 and lookahead
1: no panic; cloud smoke (48 games, lookahead, t3): finishes (win rate is low until cloud-core's plan lands).
