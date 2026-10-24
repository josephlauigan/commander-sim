# Report: garland (deck registered, cards modeled, AI plan)

Branch `garland`. Deck `garland` registered (decks.rs FILES/MINE 13, args.rs, my_decks.toml, fixture, game.rs 78, practice
catalog tests list it); `cardgen export --offline` and `validate` OK.

## Cards and plan
Cards: `impls/garland_cards.rs` (Coerced to Kill, Skycoach Conductor, Specimen Freighter, Tragic Lesson, Amphin Pathmage,
Lazav [Approximate: copies only P/T and basic keywords], Xande, Potioner's Trove), plan in `impls/garland.rs` (re-crown
value, attack targets, cast priority, Unsummon/Whoosh!/Ropemaster re-crowns). Tests: `tests/cards_garland.rs`, 18 pass.
Plan details:
- Never hits the monarch: `attack_targets` in `brain::choose_defender` AND in the look-ahead `search::choose_attack_inner`
  (the search used to choose the monarch itself; fixing it cut "we became the monarch" from 1.03 to 0.51 per game and
  lengthened steals from 0.74 to 0.93 rounds).
- Re-crown: Rescue/Mage/Freighter/Conductor creatures (cast priority), Relm's Sketching, Echocasting Symposium, Whoosh!,
  Unsummon at an opponent's end of turn, and on our own turn after combat when the crown has stalled a round
  (`own_turn_recrown`); Peerless Ropemaster targets our tapped Garland (`ropemaster_target`, dsl/mod.rs bounce arm).
- Report: "-- Garland --" section in sim-cli report.rs (`gl_*` and `jr_garland_*` stats).

## Shared-file changes (all gated to the garland deck's seat; other decks' games unchanged)
legends.rs (garland_pick pub, ETB skips the current monarch only for the garland seat, entered stat), cardcode.rs
(become_monarch calls crown_moved), blue_cards.rs / common.rs / t2.rs (blink and etb scores for our Garland), bulk_blue.rs
(copy_value, token_copy legend rule, Whoosh! own-turn re-crown), cloud.rs (cmd_stays_in_hand for garland), zones.rs +
zur.rs + common.rs (coerce lock kind), brain.rs + search.rs (attack targets), decks.rs (prio), dsl/mod.rs (Ropemaster
target), stack.rs (Dismiss draws a card: a bug fix, affects every deck with Dismiss), card_overrides.toml (Shadewriter
gained 2 per opponent: fixed to 2 total; Potioner's Trove modeled).
An earlier version changed Korvold's (opponent pool) own Garland card; found by same-seed traces and gated.

## Same-seed check (seed 7, 48 games, lookahead, loose)
sauron t3, cloud t5, thrasios t3: identical to the pre-branch binary. kraum t3: differs (win 16.7% -> 12.5%, 18 diff
lines). Cause: Potioner's Trove (kraum plays it; new override and card code). Proof: with the Trove registration and
override removed in a scratch copy (plus Dismiss draw and the Shadewriter fix also off) kraum is identical; with only
the Trove restored it differs again. Dismiss and Shadewriter were not separated from each other.

## Checks
`cargo build --release --all-targets` no warnings; full `cargo test --release` passes (practice catalog tests updated
for the new 11th deck); `all_decks adaptive 3` and `lookahead 1` finish, no panic.

## 200 games `--deck garland --pool t3 --profile loose --ai lookahead --analyze --seed 1013`
No card "never cast by the sim".
| | before attack fix | final |
|---|---|---|
| win rate | 10.0% | 8.5% (n=200, noise about +-4) |
| steals per game | 3.08 | 2.84 |
| avg steal duration (crown-moved ends) | 0.74 rounds | 0.93 rounds |
| crown changes per game | 5.50 | 4.62 |
| we became the monarch per game | 1.03 | 0.51 |
| Garland entries per game | 2.35 | 2.36 |
Steals still last under a round: opponents attack the monarch (their AI, as the rules reward), and 1.5 steals per game
end because the creature leaves. A crown-choice tweak (prefer holders with strong boards) gave 0.81 rounds, 11.5% win:
no clear gain, reverted. Open: win rate is low; the remaining levers are cast timing (Garland often waits turns in hand)
and protecting the stolen creature.
