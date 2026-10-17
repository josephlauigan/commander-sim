# Report: kraum-phyrexian3 (agent model claude-sonnet-5-5)

## Status
DONE. Archfiend of the Dross, Skrelv, Defector Mite, Gut Shot, White Sun's Twilight and Phyrexian Unlife are in the card
database (fetched from Scryfall, `cardgen export` run) and Full in `cardgen audit --cards`. 20 new tests in
`tests/cards_tymna.rs`; `cargo test --release -j 12` passes (all of it), `cargo build --release -j 12 --all-targets` has no
warnings. No deck list changed. Card code: `impls/tymna_cards.rs` (`register_phyrexian2` and the section above it).

## How each card is modeled
- **Archfiend of the Dross**: `dsl = []` (the compiled "its controller loses 2 life" resolved to nobody: `target_controller`
  is not set by an `opp_creature` trigger). 6/6 flying from the data. `as_enters` sets `DataKey::Oil` = 4 (a blinked Archfiend
  is a new object, so four again). `upkeep` hook: only on its controller's upkeep, `trigger_window`, remove one; at zero the
  controller really loses (`lose_game`: `eliminate` + `check_state`; Herald of Eternal Dawn's `cant_lose` is honoured).
  `dies` hook: an opposing creature (tokens too) dying costs its controller 2 life (`lose_life` kind "drain", not damage).
  AI: it is cast like any creature and attacks normally. With one counter left (the upkeep that would end the game) it
  first casts Restoration Angel from the hand ({3}{W}, through `cast_card` with `g.resto_target` = the Archfiend) so the
  trigger finds a new object. Otherwise the game is lost, as on the card. Stats `archfiend_saved` / `archfiend_lost`.
  Not modeled: saving it by sacrifice or bounce (the deck has none), holding Resto mana up a turn early.
- **Skrelv**: 1/1 artifact creature, can't block (compiled static), toxic from the data keyword. Poison from combat damage:
  new `tymna_cards::toxic_hit` called after combat damage to a player (counts Skrelv himself, Mite tokens with a `Kws` list
  containing toxic, and `eot_kw` "toxic" from his ability; Melira and Vorinclex honoured; the older toxic cards keep their own
  hooks, so a Dune Mover that Skrelv helps gets 1 + 1). Ability `skrelv_activate`: {W/P} ({W}, else 2 life) and {T}, needs
  an untapped, non-sick Skrelv; another creature of yours gets `eot_kw` toxic, `hx_<C>` (hexproof from the colour) and
  `nb_<C>` (can't be blocked by creatures of that colour); goes through `ability_window`. Hexproof from a colour is read in
  the three removal-targeting filters (`removal.rs` legal_targets, `common.rs`, `dsl/mod.rs` x2) and in `protect` (a later
  spell of that colour that is cast anyway is stopped). AI: `skrelv_protect` (first thing after the hexproof check in
  `tymna_cards::protect`): removal spell with a colour aimed at a commander or a creature worth 4+, Skrelv ready -> chooses
  the spell's first colour, the spell fizzles. `skrelv_options` (main phase 1): best non-sick attacker with power 2+ made
  unblockable by the colour whose untapped blockers (that `can_block` it) are worth the most (>= 2); life only above 20.
  Not modeled: protection against abilities (no source colour), a colourless spell.
- **Gut Shot**: `dsl = []` so the `rem=dmg1 face` tags drive it like Lightning Bolt (the compiled spell ignored the chosen
  target and hit a face). `prio` 0 (no plain cast), `gut_shot_options` offers `Action::Removal` on the best toughness-1
  creature with pval >= 0.9 (an elf is exactly 1.0) at utility 1.0 + 0.5 (pval - 1); otherwise held. Life payment is the
  engine's Phyrexian rule (2 life when {R} is short and life > 10). The existing "to the face when lethal" option stays.
- **White Sun's Twilight**: `dsl = []`, `resolve` hook: gain X, X Mite tokens (1/1 colourless, types phyrexian/mite/artifact,
  `Kws` toxic + cant_block; sets `g.dsl_on` so the keyword is read), and at X >= 5 `apply_wipe("destroy")` (all creatures,
  the Mites too, as the card does; wipe responses apply). `prio` 0; `twilight_options` (mana - 2 = X, main phases of the
  caster): X >= 5 wipe when `wipe_eval(destroy)` says the opponents lose more than 1.2 x ours + 4 (utility 2 + (diff)/3), else
  X capped at 4 for life and bodies, only when X >= 3 (utility 0.6 + 0.2 X). Never a wipe by accident.
- **Phyrexian Unlife**: `unlife` (on the battlefield). `engine/life.rs` `check_state`: no elimination for life <= 0 with it;
  `lose_life`: damage while life <= 0 becomes poison counters (`unlife_infect`; Melira stops them), life loss that is not
  damage (drain, Tymna's payment) still lowers life, ten poison still loses. `prio` 40 when life <= 20 and none out, else 0.

## Shared-file changes (all small, additive)
- `engine/life.rs`: `check_state` skips the life <= 0 elimination under Unlife; `lose_life` calls `unlife_infect`.
- `engine/combat.rs`: `toxic_hit` after combat damage to a player; `can_block` calls `colour_unblockable`.
- `engine/removal.rs`, `impls/common.rs`, `dsl/mod.rs` (x2): `hexproof_from` next to the `protected_from` targeting filters.
- `data/card_overrides.toml` (five entries at the end), `data/cards.json`, `data/scryfall_cache.json` (five cards).

## Smoke run (kraum, t3, loose, lookahead, 40 games, the five swaps; 20 s)
Win 30.0% -> 20.0% (noise +-20, a smoke test, not a measurement). Per card: Archfiend drawn in 10% of games, cast in 2 of
those 4 (50%); **Archfiend lost no game to its own oil counters (0 of 40; I traced all 40 swapped games)**. In the two games
it was cast: one was put onto the battlefield by Tergrid from the graveyard and wiped; the other left the battlefield before
its fourth upkeep. Skrelv drawn 32.5%, cast 77%; Gut Shot drawn 25%, cast 90% (e.g. R1 on Birds of Paradise); White Sun's
Twilight drawn 20%, cast 62%, average round 11; Phyrexian Unlife drawn 22.5%, cast in only 11% of those (88.9% stuck: the
AI casts it at 20 life or less, and the deck rarely gets there before it dies). Restoration Angel never had to save the
Archfiend in the smoke run (the save is covered by a test).

## Notes for the lead
- Phyrexian Unlife is rarely cast (life <= 20 gate); if JD wants it as a plan rather than insurance, raise the gate in
  `unlife_prio`.
- A Tergrid-style reanimator can take a dead Archfiend and its counters start again under their control (rules-correct).
