# Report: practice-core

Rust port of the human seat's rules: `sim-core/src/human/{mod,human,legal,mana,combat,choices,script}.rs`, the `HUMAN`/`PORT(phase 9)` sites in
engine/*, ai/*, cardcode.rs, dsl/mod.rs, and the text client `practice/src/text.rs` (`commander-practice --text`).

## Engine / shared-file changes (all additive, inert when `g.humans` is empty)
- `flow::Stop::Cancelled` (run_rounds sets `g.stopped`; playouts panic on it).
- `control.rs`: `Answer::Decline`, `Human::live()`, `Human::autopass()`.
- `state.rs`: `Player.pool` (ManaPool), `Game.responding`, `Game.trig_orders` (the `secret_lines` field was dropped in favour of the
  session's `Game::log_secret`; tutors.rs x3 and dsl/mod.rs call `g.log_secret`).
- `engine/mana.rs` can_pay/pay use the pool for a human seat; `turn::continue_turn` empties pools between steps;
  `turn::setup_game_for(.., human)` shuffles but does not mulligan human seats (session calls `choices::mulligan` with the returned rng).
- `human_of`, `Target`, `ScriptHuman` (list / list of functions / closure; `asked`, `told`, `set_autopass`).
- Decision sites in engine/*, ai/*, cardcode.rs, dsl/mod.rs: `if let Some(h) = human_of(g, p)` in front of the AI code, in Python's order.

## Text client
`practice::text::run(&Arc<Session>, &mut BufRead, &mut Write)` plays the Python `text.py` UI over a `Session`'s events (same commands, prompts,
TRIES=20 defaults). `commander-practice --text --deck D --tier T --ai A [--seed N --seat N --opponents a,b,c --profile P --step]`; scripted
stdin plays a game to the end with no panic (sauron t1, jodah/alela/seph t2).

## Tests (71, same names as Python's)
sim-core/tests/play_{mana,choices,respond,turn,hand,combat}.rs + play_common/mod.rs; practice/tests/play_bot_games.rs (the 8 whole-game bot
tests from test_play_turn.py BotGames). All pass except the 8 `#[ignore]`d in play_hand.rs, which need practice-cards' real `human::cards`
(hand_override, copy_card, gy_override ...): twinflame_two_targets, twinflame_extra_target_needs_mana, twinflame_copies_a_token,
ephemerate_and_rebound, cycling_at_instant_speed, unearth_your_pick, sokenzan_channel, return_the_favor_copies_your_bolt.
Rust differences: `ManaPool::can_pay/pay` take `&Game` + owner (restricted mana); `counter_back_when_the_ai_counters_you` checks the stack's copy.

## Python quirks kept
`FLOATS` omits floating black (shows as a source "FB"); floating white is never gathered into the pool; tokens named via `view.guess_token`
(hook `legal::set_token_guesser`, unwired until images/session registers it); planeswalkers can't be attacked directly; menace blocks use one blocker.

## Checks
cargo test --release green; build --all-targets no warnings; all_decks adaptive 3 / lookahead 1 no panic; difftest AI 0, build 0 (card-code
and random differences are the pre-existing ones, none from this work).
