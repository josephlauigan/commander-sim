# Report: cand2-yawgmoth

Status: DONE. Every card in the brief is Full in `cardgen audit --mine` (the 11 still not Full are the cand2-gix agent's list).
Code: `impls/yawgmoth.rs`; tests: `tests/cards_yawgmoth.rs` (31 tests). Verdicts: `data/card_overrides.toml` (end of file, plus Yawgmoth's entry).

## Cards and how each is modeled
- Yawgmoth, Thran Physician: new `options` (sacrifice fodder for counter + draw; proliferate) and `defend` (blockers that die anyway
  are sacrificed after blocks, the counter goes on an X/1 attacker or the creature the blocker faced). The old t5 code still runs
  for the opponent pool's Yawgmoth deck (dispatch on the player's deck key). Counter target: kills first, else the creature it shrinks
  most. Life floor 10 (6 for a big kill). Proliferate = `common::proliferate` (already handled +1/+1, loyalty, lore, opposing -1/-1).
  Protection from Humans: no damage from Humans (`rules2::prot_vs`), Humans can't block it (`rules2::prot_unblockable`).
  NOT modeled: a Human creature's ability can't target it (no deck's Humans matter for this).
- Braids's Frightful Return: read ahead (`as_enters` picks the chapter by value), chapters I-III, lore after the draw step and from
  proliferate (`saga` slot), sacrificed after III. The opponent's chapter III choice: a spare permanent (worth < 2.2) or 2 life + a card.
- Presumed Dead / Perigee Beckoner (+2/+0, returns when it dies): an `eot_kw` marker; `zones::die_rest` calls `returns_when_dies`.
  Beckoner warp {1}{B} (exiled at the end step, castable from exile on a later turn), like Susurian Voidborn.
- Suspect (It Doesn't Add Up, Presumed Dead): `DataKey::Suspect`; `dsl::has_kw` gives menace and "cant_block".
- Rewrite Regrets (MV <= 6, Empower Jace 2 via `alela::empower`), Phyrexian Delver (life = MV, never below 6), It Doesn't Add Up,
  Send in the Pest (opponent discards its worst card; Pest token), Festering Goblin, Soul Enervation (-4/-4, and the drain as a
  state check comparing the graveyard's creature cards with the last look), Arnim Zola, Bloodsworn Squire (transform = `DataKey::Flipped`
  + `self_pt`), Catapult Fodder/Captain (combat_start, sacrifice drain), Cheerful Osteomancer (prepared Raise Dead copy), Dread Drone
  (the existing compiled text; the Spawn mana is in `mana.rs`; test added).
- AI plan: `yawgmoth_prio` (Yawgmoth first; recursion waits for a graveyard card worth >= 2.5; Delver/Beckoner/Zola etc.),
  `yawgmoth_options` (Presumed Dead before combat on a good attacker, warp, Beckoner from exile).

## Shared-file changes (all additive, small)
- `state.rs`: `DataKey::Suspect`.
- `dsl/mod.rs` (`has_kw`): suspected permanents have menace and "cant_block" (3 lines).
- `engine/zones.rs` (`die_rest`): 3 lines before the card goes to the graveyard, calling `yawgmoth::returns_when_dies`.
- `impls/rules2.rs`: Yawgmoth's protection from Humans in `prot_vs` and `prot_unblockable` (6 lines).
- `impls/t5.rs`: `yawg` and `yawg_defend` made `pub` (no other change).
- `ai/brain.rs`: one match arm in `special_options` for `yawgmoth`.
- `cardcode.rs` (`defend_hooks`): the `defend` hooks also run for the `yawgmoth` main deck (`|| dk == "yawgmoth"`).
- `data/card_overrides.toml`, `data/cards.json`, `data/decks.json` (re-exported).

## Checks
cargo test --release: all pass. cargo build --release --all-targets: no warnings. all_decks adaptive 3 and lookahead 1: no panic
(under the 8G cap). Smoke: `--deck yawgmoth --pool t3 --profile loose --games 48 --ai lookahead --jobs 8 --brief` finished,
win rate 4.2% (CI 1.2-14.0%) -- tuning was not part of the task.

## Notes for the lead
- No response-time Yawgmoth: it does not sacrifice a creature in response to removal/targeted spells (only main phases and after blockers).
- Soul Enervation's drain fires at the next state check after a creature card leaves the graveyard, not at the exact instant.
