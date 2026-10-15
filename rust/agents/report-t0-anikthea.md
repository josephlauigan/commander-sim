# Report: t0-anikthea (second agent, finished the last 4 cards)

Audit (`cardgen audit --deck anikthea-enduring-enchantments`): 29 of 30 brief cards Full, 1 Partial: **Ghoulish Impetus**
(goad pending t0-zurgo's mechanic, by the lead's decision; the doc comment and override note say so).

## Cards, how modeled (all in impls/t0_anikthea.rs; first agent's 26 cards are described in their doc comments)
- **Calix, Destiny's Hand**: `common::walker` with +1 (top four, best enchantment to hand, rest to the bottom at random),
  -3 (`oring_exile` of the best opposing creature/enchantment, linked to a chosen enchantment you control; every deck
  enchantment without its own `leaves` hook gets one calling `oring_return`, which shares `DataKey::Oring` with the
  O-Ring family, so Cast Out/Helvault/Grasp links also work), -7 (all enchantment cards from the graveyard enter).
  Override `dsl = []`.
- **Demon of Fate's Design**: `options` entry, once per turn (`DataKey::Act` stamp), casts an enchantment from hand for life =
  mana value, only if life stays at 12+. The {2}{B} sacrifice +X/+0 stays the compiled activation.
- **Cunning Rhetoric**: `attack` hook (defender is the owner): exile the attacker's top card into the owner's hand, marked in
  `impulse_long` (mark `turns + 100000`, so it never lapses and can't be mistaken for plot marks); `options` play it as a
  land or cast it paying its mana value as generic mana (any colour) when its own colours can't pay.
- **Ghoulish Impetus**: `common::aura` +1/+1, deathtouch, `host_pick` = best targetable opposing creature, `host_dies` queues
  the card on `g.eot_returns`. NO goad: until the lead wires it, the Aura only buffs an opponent's creature (the AI does cast it).

## Shared-file changes (all small)
- `impls/galadriel.rs` `eot_returns`: also returns a card waiting in the graveyard (was exile only).
- First agent's: `engine/zones.rs` (3 lines in `draw`), `engine/combat.rs` (1 line in `attack_limits`).
- `data/card_overrides.toml` (4 entries), `data/cards.json` regenerated.

## Checks
- `cargo test --release -j 3 --no-fail-fast`: all pass (31 tests in cards_t0_anikthea, 6 new). Build with `--all-targets`: no warnings.
- `all_decks adaptive 3` (72 games) and `lookahead 1` (24 games): no panic, no memory-cap hit.
- Smoke run (`--deck cloud --pool t0 --any-tier --profile loose --games 48 --ai lookahead --jobs 3 --brief`) finished; Anikthea won 68.8%.
- Traces (games 0-4 and 8-19): the commander is cast on rounds 4-7 (mana permitting); Calix +1, Abundance replacing draws,
  Boon, Binding of the Titans, Eldest Reborn, Mending, Battle for Bretagard, Demon casting enchantments for life, Cunning
  Rhetoric exiling attackers' cards, Composer/Doomwake/Nessian/Ondu all seen in play.

## For the lead
- Wire goad into `IMPETUS` (impls/t0_anikthea.rs, `register_impetus`) after merging t0-zurgo, then set the override to Full.
- Cunning Rhetoric treats attacks at planeswalkers as attacks at the player (the engine attacks players only).
