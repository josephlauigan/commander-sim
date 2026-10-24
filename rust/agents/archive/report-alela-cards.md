# Report: alela-cards

Branch `alela-cards`, model claude-sonnet-5-5. `cardgen audit --mine`: alela 75 unique cards, 0 not Full.

## The seven cards (impls/alela.rs, tests in tests/cards_alela.rs)
- **Credit Voucher**: `options` ability. {2}, tap, sacrifice (a cost, goes through `die`), then put the "junk" cards (hand cards
  with `card_worth` < 15: lands past 7, dead cards; never the commander) into the library, shuffle, draw that many. AI: two or
  more junk cards in the first main phase, one at other windows.
- **Scrapheap**: `dies` + `self_dies` hooks (same pattern as Karmic Justice). Gain 1 for each nonland artifact/enchantment card
  you own that dies (destroyed or sacrificed), Scrapheap itself too, tokens never, opponents' cards never. Limit: only
  permanents that go through `die`; a direct leave+graveyard move (an Aura falling off, a 0-loyalty walker) is not seen, and
  neither is an artifact exiled or bounced (correct, those are not "put into the graveyard").
- **Mirage Mirror**: `{2}`: the permanent's face becomes the target's card (`set_face`: cd, name, body, keywords, hooks,
  no enters triggers), its own card stored in `phys` (so it goes to the graveyard as the Mirror), `DataKey::MirageMirror` flag;
  `mirror_revert` (called from `step_start`, the same moment `eot_pt` resets) turns it back. AI: before attacks a copy of the
  best 3+ power non-defender creature on the battlefield (not the turn it came); on the other player's attack (blocks hook)
  a copy of a creature that survives and blocks the biggest unblocked attacker. Not modeled: copying a land (a land copy would
  need a `Land`; never worth {2}) and copying a token (no card); an Aura target is skipped.
- **Angelic Page**: `blocks` hook (runs for both players' Pages): once blocks are set, tap (not summoning sick) for +1/+1 on
  the creature of its side whose fight the +1/+1 turns (survives, or kills), else on the biggest unblocked attacker. Skipped
  for practice-mode owners.
- **Cabal Patriarch**: `options`; both abilities. Cost: a Faerie token / spare body first (never Alela, never itself),
  otherwise exile a creature card from the graveyard, otherwise a cheap non-token. Target: opposing creature with toughness
  <= 2 (one activation) or 3-4 (two activations, needs {4}{B}{B}); only if its `pval` beats the costs by 0.5.
  -2/-2 through `apply_removal("shrink2")`.
- **Serra the Benevolent**: walker (`SERRA`), `dsl = []` in the override so the compiled -3 is not offered twice.
  +2 fliers get +1/+1 until end of turn (best when fliers are ready to attack); -3 4/4 white Angel, flying + vigilance (best
  with an empty board, or <=3 creatures, or when the opponents' power >= her loyalty); -6 emblem "serra" (value 10).
  The emblem is `serra_floor`, called from `lose_life` for damage kinds only: with a creature, damage can't take you below 1
  (life loss, e.g. drain, is not damage and is not floored; commander damage is unaffected).
- **Wall of Junk**: `blocks` hook; the AI blocks with it freely (biggest unblocked attacker it survives, not deathtouch, not
  menace), marks `DataKey::WallBlocked`; `wall_return` (called by `resolve_combat` after damage) bounces it if it is still
  on the battlefield. Override tags add `noatk` (the combat code let any creature with power > 0 attack; a pumped Wall would
  have). Recast: the existing generic AI casts it (creature priority, plus the Faerie bonus).

## The four deck-AI checks
1. **Skullclamp on Faeries: already worked.** Alela decks use the generic `pool::special_options` Skullclamp play (equip onto a
   1-toughness token or a cheap creature). With Alela a Faerie is 2/1 and the toughness is what counts. Tests: Faerie, and
   Birds of Paradise.
2. **Angelic Shield**: the +0/+1 stops the Skullclamp play (the Faerie is 2/2): tested. The sacrifice ability was only reachable as the
   compiled "bounce target creature" option (attacks an opposing threat if worth it). **Added** `alela_protect` (arm in
   `ai/decks.rs::protect_main`): in response to removal aimed at one of your non-token, non-commander creatures with pval >= 3 the
   Shield is sacrificed and the creature returns to hand. Not spent on tokens (would be lost) or on Alela (command zone anyway).
3. **Crystal Chimes**: the compiled version was wrong (it returned only ONE card and was offered with 0 enchantments).
   **Added** `chimes_options`/`chimes_go` and `dsl = []`: activates only with 2+ enchantment cards in the graveyard and
   returns all of them. Tested.
4. **Lightning Greaves on Alela**: the Greaves sat on Alela forever and the deck's own Auras (`own_host` never checks
   shroud) and Daydream could "target" her illegally. **Added** `greaves_options` (registered on Lightning Greaves, active only
   for the `alela` key): with the Greaves on Alela and an own-target Aura (host = Alela) or a Daydream aimed at her in hand, the AI moves
   the Greaves ({0}) to the best other creature first. Not fixed: `own_host` ignoring shroud for every deck.

## Shared-file changes (all additive, one to three lines each)
- `engine/combat.rs`: `alela::wall_return(g, &assign)?` after `vanguard_blocks` in `resolve_combat_inner`.
- `engine/life.rs`: `alela::serra_floor(g, p, &mut n)` in `lose_life` after the prevention check.
- `engine/turn.rs`: `alela::mirror_revert(g)?` in `step_start` after the `eot_pt` reset.
- `state.rs`: `DataKey::MirageMirror`, `DataKey::WallBlocked`.
- `ai/decks.rs`: `"alela" => alela_protect(..)` arm in `protect_main`.
- `data/card_overrides.toml`: entries for the seven cards plus Crystal Chimes; `data/cards.json` regenerated by `cardgen export`.

## Checks
- `cargo test --release -j 5`: all pass (99 test binaries; `practice` crate's `http_play` failed once or twice under load
  in earlier runs, different test each time, and passes alone 3/3: flaky, unrelated).
- `cargo build --release -j 5 --all-targets`: no warnings. all_decks skipped as told.
- Smoke: `commander-sim --deck alela --pool t1 --profile loose --games 40 --ai lookahead --jobs 5 --brief`: finished
  (21 s; 5.0% win rate over 40 games, no meaning). `--pool t3` is refused (alela is matched to t1 in my_decks.toml).

## Open / for the lead
- Nothing was measured for win rate. Practice-mode seats get the new abilities through the shared options list, but the Page and
  Wall block hooks do nothing for a person's seat.
