# Report: t0-zurgo (Zurgo Stormrender, Mardu Surge)

All 22 cards of the brief are Full in `cardgen audit --deck zurgo-mardu-surge`. Card code: `rust/crates/sim-core/src/impls/t0_zurgo.rs`
(its module doc lists each mechanic and the engine call site). Tests: `rust/crates/sim-core/tests/cards_t0_zurgo.rs` (30).
Verdicts and notes: `data/card_overrides.toml` (last block). `data/cards.json` regenerated with `cardgen export`.

## Mechanics I own
- **Myriad**: `attack_keywords` (called from `combat::attack_triggers_once`). Printed (Goldlust Triad), granted by Legion
  Loyalty, an equipped Blade of Selves, Ironwill Forger's lieutenant (`eot_kw "myriad"`). Token copies attack each other
  opponent (new `Game::atk_targets` records every attacker's defender; `route_new_attackers` puts new attackers into
  their own player's group) and are exiled at end of combat (`DataKey::MyriadTok`, `end_of_combat`; Zurgo sees them
  leave while attacking). A legendary creature is not copied (legend rule; the ability is a "may").
- **Mobilize** (Zurgo's attack hook; tokens use the existing `DataKey::EotSac`), **goad** (`DataKey::Goaded` = goader +
  goader's turn count; `goad_attackers` in `combat::combat` forces the attack at a player other than the goader),
  **fabricate** (Angel of Invention: Servos), **hideaway** (Windbrisk Heights), **lieutenant** (Ironwill Forger).
- **Token replacement/doubling**: Divine Visitation in `make_tokens` (also applies to myriad / Stormsinger copies);
  Kaya's -2 via `dsl::token_mult` (so Treasures double too), flag in `flag_turn["kaya_double"]`.
- **Token leaves the battlefield**: `zones::leave` calls `token_left` (Zurgo, Twilight Drover) for every way a token leaves.

## Cards (how each is modeled)
Zurgo Stormrender (mobilize 1 + leave trigger), Divine Visitation, Eliminate the Competition (AI sacrifices the
cheapest X creatures via `hand_options`, X chosen while each trade gains value; destroys the best X on resolve),
Legion Loyalty, Shadow Summoning, Will of the Mardu (both modes with a commander out, else the better), Ainok Strike
Leader (Goblin per opponent when it or a commander attacks; sacrifice ability is in the AI protector table as
`How::TokenIndes`: targeted removal on a token, or a wipe when tokens are 75%+ of the board), Angel of Invention,
Blade of Selves, Chittering Witch (Rats + sacrifice for -2/-2 aimed at a kill), Goldlust Triad, Grenzo (goad or
exile-and-cast via `yshtola::take_card`; the existing `impulse` list returns unplayed cards at the end step),
Ironwill Forger, Kaya (walker: +1 deathtouch via `dt` with a `kaya_dt` marker removed in `end_cleanup`, -2, -6),
Mindblade Render (once per combat), Myr Battlesphere, Neriv (new `Player::neriv` list: cards stay in exile and come to
the hand for any turn a commander attacked; lands go to hand for good), Redoubled Stormsinger (tokens with
`born > Game::turn_enter`), Twilight Drover (the old compiled ability needed `cost.other`, which the interpreter
refuses, so it is in Rust and `dsl = []`), Windbrisk Heights, Within Range (resolved after the other attack triggers),
Commander's Insignia (`tax / 2` in `values::static_bonus`).
Autotag fix: Ainok Strike Leader carried `tok=1 tokp=1` (a bogus enter-the-battlefield 1/1); its override replaces the tags.

## Shared-file changes (all small and additive)
`state.rs` (DataKeys Goaded, MyriadTok; `Game::atk_targets`, `turn_enter`; `Player::atk_count`, `cmd_atk`, `neriv`),
`engine/combat.rs` (5 one-line calls + `atk_targets` set where attackers are declared), `engine/zones.rs`
(`make_tokens`: Divine Visitation, `born`; `leave`: `token_left`), `engine/turn.rs` (`turn_enter`, `end_cleanup`),
`engine/values.rs` (Insignia), `dsl/mod.rs` (`token_mult`), `ai/pool.rs` (Ainok in PROTECTORS, `How::TokenIndes`).

## Checks
- `cargo test --release -j 3`: all pass. `cargo build --release -j 3 --all-targets`: no warnings.
- `all_decks adaptive 3` and `all_decks lookahead 1`: no panic.
- Smoke: `--deck cloud --pool t0 --any-tier --profile loose --games 48 --ai lookahead --jobs 3`: Zurgo won 66.7% of the
  games it was seated in (Cloud 12.5%, n=48, noisy). Worth a look when the lead balances Tier 0.
- Traces (10 games, adaptive): the commander is cast on turn 3 to 7 (the sampler picks other plays at times), mobilize,
  token drains, Grenzo, Kaya, Windbrisk, fabricate, Eliminate all appear.

## Known approximations
- Mindblade Render triggers once per combat, not once per damage step (first strike).
- Windbrisk Heights hides a card only when played as a land drop.
- Ironwill Forger picks its target among the attackers the AI already declared.
- Neriv's cards sit in exile until a commander attacks; lands go to hand for good.
