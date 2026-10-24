# Report: cloud-ai-fix2 (model claude-sonnet-5-5)

Branch cloud-ai-fix2 from cloud-midgar; one commit per fix; tests in `tests/cards_cloud_ai_fix2.rs` (15). No batch sims.

1. Engine-aware tutor (cloud.rs `cloud_pick`, `equip_engine`): Puresteel Paladin on the battlefield with metalcraft counting the
   new piece, Ardenn out, or Kor Outfitter in hand make equips free: the biggest castable payoff is fetched first (Sword, Helm,
   Shield of Kaldra, Wrecking Ball Arm, Colossus Hammer); Steelshapers lower the equip cost in `total()`. Cid is already in
   `cost_of`. Enlightened Tutor (kinds "ae"/"art") now uses the same pick via `ai/decks.rs::tutor_pick_named`. Not done: Armored
   Skyhunter (it does not attach to a pick), Helm-after-first-Sword unchanged (needs `exposed`).
2. Stranded Equipment (`strands_equipment`, checked in `brain::card_utility` for cloud): held when no creature is out, none can be
   cast with it this turn, and another non-land card is castable (so it is not the only use of the mana). Self-attaching pieces exempt.
3. Protection: Plaza of Heroes `{3},{T}`, exile modeled in `cloud_protect` (`plaza_protect`, last resort after Giver and the
   others); Rootborn Defenses / Unbreakable Formation cast against destroy wipes (`cloud_wipe_response`, wired in
   `decks::wipe_response`; formation code shared via `cast_team_indestructible`). Giver of Runes works when Cloud is the commander
   (test); it cannot activate while summoning sick, which explains the audit's "never activated".
4. `cloud::PROTECT_FIRST` (default true): `search_bonus` penalizes casting Cloud (-12) and boosts Giver/Boots/Helm (+14) when removal
   risk >= 0.3, the piece fits now, and piece + Cloud fit next turn but not now; after a death (tax > 0) Cloud waits unless
   protection can follow (Giver untapped, Grand Abolisher, Plaza with {3} left, Boots/Helm equip) or mana >= 3x his cost.
5. Ping-pong: `dsl::equip_options` moves a worn piece only if the best holder scores `MOVE_MARGIN` (2.0) above the current one. Only
   reachable for Cloud (`wants_move`), so Sauron/Sheoldred/Y'shtola are unchanged by construction (their traces still show their equips).
6. Mulligan: `cloud::keep_hand` (3-5 lands, or 2 with a rock/dork or creature of cmc <= 2) used by `turn::mulligan` for key cloud.
7. Logging: lines for wipe victims (`removal.rs`), edict sacrifices (`zones::edict`), Pernicious Deed deaths (`common.rs`). Deed has no unit test
   (could not get the AI to offer it in a small setup).

Shared files touched: `ai/decks.rs`, `ai/brain.rs`, `dsl/mod.rs`, `engine/turn.rs`, `engine/removal.rs`, `engine/zones.rs`, `impls/common.rs` (all small).
Checks: `cargo test --release -j 4`: all pass except the known flaky practice `http_lan` save test; `--all-targets` build 0 warnings;
`all_decks adaptive 3` and `lookahead 1` exit 0; 20 cloud traces (t1, t5 x10): no panic, no Mask ping-pong, wipe/edict lines present.
Plaza was played once but never needed in those traces.
