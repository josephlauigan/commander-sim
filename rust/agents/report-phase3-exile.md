# Report: Phase 3, PR 5: identity for exiled cards (branch `phase3-exile`)

Brief: `rust/agents/phase3-mana-exile.md` (PR 5). Design: `audit/phase3/exile-design.md`. Handoff:
`audit/phase3/exile-handoff.md`. Code: `rust/crates/sim-core/src/engine/exile.rs`, `tests/exile_objects.rs`. Commits:
d76ccbc4 (Stage A), fb948c40 (Stage B), 9267112b (Stage C). No sims (JD's rule).

## Stage A (results identical)
`ExileZone`: the cards plus one `ExileObj { id: ExileId, until: Option<PermId>, silver: bool }` per card, with a fresh
id on every push (rule 400.7). It reads like the old `Vec<CardId>`, so about 200 sites compile unchanged; about 20
`remove_card` calls became `exile.take(c)`.

## Stage B (results identical)
`common::link_exiled` / `linked`: the "until ~ leaves" exiles record which exiled object they hold: `oring_exile` /
`oring_return` (O-Ring family, Calix, Static Net, Palace Jailer, Fiend Hunter, All-Fates Stalker, Anikthea's
enchantments), Angel of Serenity, Detention Sphere. Karn, Scion of Urza's +1 sets `silver` on the object.

## Stage C: behaviour (6 tests)
- "Until ~ leaves" returns only the object it exiled: a card with the same card id exiled since by something else is
  not returned (the `oring_return` bug).
- `Player::silver` deleted: a card that leaves exile and comes back has no silver counter; Karn's -1 takes the copy
  with the counter.
- Impulse draw ("you may play it this turn") ends with the turn for every player, not only the active one.

## Checks
- `cargo test --release --no-fail-fast`: 3669 passed, 0 failed, 3 ignored (rerun by the coordinator). Build 0
  warnings. Practice 104 (UI unchanged). cardgen clean, no data change.
- `clone_cost` 2.95-3.05 µs against 2.91-2.94 (within noise, at most 3-4%). `mana_cost`, `repl_cost` unchanged.

## Decks touched (for the recalibration)
- **JD's karn** (Karn, Scion of Urza; t4-t5) and **yshtola** (Static Net; t5).
- anikthea, giada (t0); light-paws (t1); kaalia, sythis (t2); zur (t5); dack (t6).
- Impulse expiry: any seat given "play it this turn" on another player's turn (Grenzo, Mnemonic Betrayal, the
  ability language's `impulse`).

## Not fitted yet
- Other trackings still name cards by card id: `oath_return`, `valakut_cards`, `neriv`, `necro_wait`, `suspended`,
  `rebound`, `zur_due`, `eot_returns`, the `t12_*` lists, Hostage Taker.
- Impulse cards are held in hand, not in exile with a permission to play (needs a cast-from-exile path).
- No face-down objects yet (foretell, Necropotence in practice).
