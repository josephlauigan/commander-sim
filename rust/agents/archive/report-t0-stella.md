# Report: t0-stella (Stella Lee, Wild Card precon)

All 26 cards are Full in `cardgen audit --deck stella-lee-quick-draw`. Card code: `impls/t0_stella.rs`; tests: `tests/cards_t0_stella.rs`
(40 tests); overrides (status, note, dsl) appended to `data/card_overrides.toml`.

## Shared-file changes (all additive, one to three lines each)
- `state.rs`: `Player::t0s: StellaState` (+ init), `DataKey::T0Stella`.
- `engine/cast.rs`: `on_cast` calls `t0_stella::on_cast_next` ("next instant or sorcery you cast" effects: storm, cascade, Drake copies,
  Kaza use-up); `magecraft` calls `magecraft_extra` (Octavia); `to_gy_or_exile` calls `foundry_exiles`.
- `engine/mana.rs` `cost_of`: `kaza_delta`. `engine/turn.rs`: `|| t0s.nomax` in the hand-size check. `cardcode.rs` `aura_ward`: `boots_ward`.

## How each card is modeled
- Stella: cast slot; 2nd spell exiles the top card (waits in hand marked in `impulse_long`, as Mascot/Prosper do); the 3rd+ instant/sorcery
  is copied by tapping Stella as it is cast (the AI's use of the {T} ability). Compiled "draw" dsl removed.
- Storm (Elemental Eruption, Spellslinger): `storm_copies`. Cascade (Torrent, Stagecoach): `legends::cascade`. Suspend: `Player::suspended`
  (Rousing Refrain suspends itself again on each cast, as the Oracle text says). Plot (Lock and Load, Pyretic Charge): green_cards.rs scheme.
  Adapt, crew (own `crew` hook), jump-start (`gy_options`), cycling (Shark Typhoon's own `hand_options`) all in the module.
- X spells (Swine, Experiment, Promise, Revelation): their own `hand_options` choose X; free casts skip X spells, counterspells, wipes,
  targetless removal.
- Forger's Foundry: `mana_tapped` marks the spell its mana paid for; the resolution exiles it (AI always takes it); sorcery-speed activation
  casts the pile free.
## Judgement calls the lead should know
- Cursed Mirror copies stats and flying/deathtouch/vigilance/lifelink and gets haste, but NOT the copied creature's triggered abilities or
  enters effects (no engine support for becoming a copy of a card in place). Marked Full anyway; change to Approximate if you disagree.
- Finale of Promise's X>=10 copies are made right after each free cast (the spell has already resolved), not while it is on the stack.
- Shark Typhoon cycling is modeled inside this module; t0-giada owns "when you cycle" triggers and will not see it (no card in this deck needs it).
- Volcanic Torrent: engine "dmgN" removal kills regardless of toughness, so the code compares toughness itself.
- No human (practice mode) choices implemented for these cards.

## Checks
cargo test --release -j 3: all 89 result lines ok. build --all-targets: no warnings. all_decks adaptive 3 / lookahead 1: no panic.
Smoke: `--deck cloud --pool t0 --any-tier --profile loose --games 48 --ai lookahead --jobs 3 --brief` finishes. Traces (games 0, 3): commander
cast on turn 2, Stella exile trigger, Epic Experiment, Torrent cascade, suspend, crewed Stagecoach all seen.
