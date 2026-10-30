# Handoff: Phase 3, PR 5, identity for exiled cards (branch `phase3-exile`)

Brief: `rust/agents/phase3-mana-exile.md` (PR 5). Stacked on `phase3-mana` (PR 4, done: `audit/phase3/mana-handoff.md`).
Design: `audit/phase3/exile-design.md`. The PR 4 agent stopped after PR 4; a second agent does PR 5.

**JD: no simulation runs of any kind until Phase 4.** Checks: release test suite, 0 warnings, practice tests,
cardgen export/validate/audit, microbenchmarks only (`examples/clone_cost`; add an exile one if the zone changes
shape).

Base = `phase3-mana` head (cc71b6cc): build 0 warnings; 3663 passed / 0 failed / 3 ignored (223 binaries); practice
104; cardgen clean; clone_cost 2.91-2.94 µs.

## Status
- [x] Design (`exile-design.md`).
- [x] Stage A: `engine/exile.rs` (`ExileZone`: cards + one `ExileObj { id, until }` each, a fresh `ExileId` per push;
      reads through `Deref<Target = [CardId]>`); `Player::exile` is one; the `remove_card(&mut ..exile, c)` sites use
      `exile.take(c)`; `cast.rs` copy removal and `human.rs` reanimation destination rewritten. Checks: 0 warnings;
      3663 passed / 0 failed / 3 ignored (223 binaries), as base; practice 104; cardgen clean; clone_cost 2.92-3.01 µs
      (base 2.91-2.94: noise); mana_cost and repl_cost as base.
- [ ] Stage B (call sites, identical): `oring_exile` records `until` + the `ExileId`; Karn records `silver`.
- [ ] Stage C (behaviour, tests): `oring_return` by id; silver from the objects; impulse expires at the end of the turn.

## Census so far (starting points, not complete)
- `Player::exile: Vec<CardId>`: 207 uses of `.exile` in `sim-core/src` (most in `impls/mine.rs` 19, `common.rs` 11,
  `t4.rs` 8, `kefka.rs` 7; `engine/cast.rs` 6, `zones.rs` 5). A drop-in idea for Stage A: an `ExileZone` type with
  the `Vec<CardId>` read API (`Deref<Target = [CardId]>`) and push / retain / remove that keep a parallel list of
  exile objects (`ExileId`, a fresh id on every push: rule 400.7), so the sites compile unchanged.
- Per-card tracking to map onto the object's links (Stage B), in `state.rs`:
  - "until ~ leaves": `DataKey::Oring` on the exiling permanent, a `Val::List` of (card, owner) pairs
    (`impls/common.rs` `oring_exile` / `oring_return`, 37 callers; Static Net, Journey to Nowhere, Grasp of Fate,
    Anikthea's enchantments, Calix's `DataKey::CalixLink`, `t2.rs:291`). `oring_return` returns the card if any card
    with that `CardId` is in exile: the Stage C bug (a card that left exile and was exiled again by something else
    comes back).
  - impulse draw: `Player::impulse: Vec<CardId>` ("held in hand", not in exile), `impulse_long: Vec<(CardId, u32)>`
    (Prosper), `neriv`, Valakut Exploration's list, Oath of Teferi's list (end step), `necro_wait` (practice).
    Stage C: "until end of turn" should expire.
  - silver counters: `Player::silver` (Karn, Scion of Urza; `impls/karn_engines.rs`).
  - suspend: `Player::suspended: Vec<(CardId, u32)>`; adventure: `Player::adv_done`; rebound: `Player::rebound`.
  - delayed returns: `Game::zur_due` (The Eternal Wanderer), `eot_returns` (Eerie Interlude), `t12_due`
    (Avacyn / Liesa), `t12lp` (Flickerform), `beheld` (behold costs), `t12_valg` / `t12_void` / `t12_kept` /
    `t12_pass` (Valgavoth).
  - foretell, plot, cascade-to-exile, "cards exiled with ~": grep `DataKey` names and `exile.push` in `impls/`.
- Phase 2's follow-ups for the ability language: `rust/agents/report-phase2-doubles.md`.

## Notes
- Scratch: `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/phase3-exile/`.
  The PR 4 checks script is `.../scratchpad/phase3-mana/checks.sh`: copy it and change the worktree path.
