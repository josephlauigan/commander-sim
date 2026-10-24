# Report: cand2-gix (agent model claude-sonnet-5-5)

## Status
DONE. `cardgen audit --mine`: gix has 66 unique cards, 0 not Full. 28 tests in `tests/cards_gix.rs`; all of
`cargo test --release` passes; `cargo build --release --all-targets` has no warnings; `all_decks adaptive 3` and
`all_decks lookahead 1` finish; smoke run `--deck gix --pool t3 --profile loose --games 48 --ai lookahead` finishes
(16.7% win rate, 48 games; a smoke test, not a measurement).

## Process slip
I ran `python3 --version` once (rules forbid any Python). It only printed a version; nothing else ran.

## How each card is modeled (code in impls/gix.rs; verdicts in data/card_overrides.toml)
- Al Bhed Salvagers: `dies` + `self_dies` + `sacrifice` (Token variant, so Clue/Treasure/Food/Blood sacrifices count); weakest opponent -1, you +1.
- Necrosquito: oil counters in `DataKey::Oil`, +1/+1 each through `self_pt`; enters with 2 (`as_enters`); +1 per own creature/artifact dying or artifact token sacrificed. Its compiled dsl is cleared (`dsl = []`) so nothing doubles.
- Callous Inspector (dsl cleared; 1 damage to you + Clue in code), Apex Witchstalker (gain 2 on enter/death; basic landcycling via `hand_options`), Sangrophage (pay 2 life above 14, else tapped), Deepcavern Imp (echo: discard worst card if junk or hand >= 4, else sacrifice), Balustrade Spy (mills self with Dark Matter out, else biggest-library opponent), Gravblade Heavy (self_pt +1/+0, deathtouch via an `sba` hook that sets the perm's `dt` flag because combat reads the flag), Ulna Alley Shopkeep (infusion via `self_pt`), Dark Matter Manipulator (compiled mill 3; `self_pt` +2/+0 per 7 graveyard cards).
- Corpulent Corpse: fear in `rules::evasion_blocked`; suspend 5 for {B} through the existing `suspended` table (hand option); haste when it enters during its owner's upkeep (`Step::Start`, only suspend can do that); hard-cast only with 8 lands.
- Gollum, Silent Slinker: Meager Meal as a hand option ({B}, counter on the best attacker, 2 life, `adv_done` like Brazen Borrower; creature stays in hand).
- Grim Wanderer: `prio` slot returns 0 unless a creature died this turn (`g.died_turn`). This only restricts the AI, not the engine's cast rules.
- Restless Bloodseeker: Blood tokens are `Player::bloods` (a count, like Clues; engine tokens are all creatures); end-step Blood if life was gained; transform for two Blood (`DataKey::Flipped`, +2/+0 via `self_pt`); Reveler drain {4}{B}; Blood rummage option.
- Salvaged Manaworker: colour filter in `engine::mana` (`fixed_plan`): when `plan_pay` fails, one single mana is turned into any colour of the deck's identity, once per turn (`mw_used` in eot_kw). Net mana is unchanged.
- Key to the Side-Door: `options` hook; {2},{T} gives the best blockable attacker `unblockable`. The second ability (discard a legendary card with the same name as one you control) can't happen in a singleton deck; not modeled.
- Overkill: tags `rem=shrink9999 tgt=c`. Depressurize: tags `rem=destroy tgt=c`, with a name filter in `legal_targets` (power <= 3); the bare -3/-0 trick is never cast. End of the Hunt: `resolve` slot, the opponent whose greatest-MV permanent is worth most exiles it (their least valuable among ties).
- Medic's Kitesail and Mask of Memory: compiled abilities written into the overrides (equip_bonus/equip_keyword/attack and combat_damage triggers).

## AI plan
Evasive creatures (flying/menace/fear) get +8 cast priority; equipment and the Key are 0 until a creature is out, then at least 46. Equip choice (dsl) prefers a non-flying creature for flying-granting equipment and an evasive one for connect-trigger equipment. Gix draws only above 12 life (was 10). Gix's {4}{B}{B}{B} needs 7+ lands, and every land in hand counts as junk once 7 lands are out.

## Shared-file changes (all small and additive)
- `state.rs`: `Player::bloods` (field + init).
- `engine/mana.rs`: `fixed_plan` + use in `can_pay` and `pay` (Manaworker filter; only runs when the normal plan fails).
- `engine/removal.rs`: Depressurize name filter in `legal_targets`.
- `dsl/mod.rs` (`equip_options` score): flying-granting and connect-trigger equipment targeting; existing equipment behaves the same except Sword-like (connect-trigger) equipment now also prefers evasive creatures (+2).
- `impls/rules.rs`: fear in `evasion_blocked`.
- `impls/t3.rs`: Gix life threshold 10 -> 12. `impls/partials.rs`: Gix ability needs 7 lands; lands in hand are junk then.
- `data/card_overrides.toml` (new section at the end), `data/cards.json`, `data/decks.json` (re-exported).

## Notes for the lead
- Other decks' prio functions must go through `generic_prio` (or the card's own `prio` slot) for Grim Wanderer, Corpulent Corpse and End of the Hunt priorities to apply; the Yawgmoth and B/R agents' deck prios are theirs.
- Restless Bloodseeker exists in cards.json under two names ("Restless Bloodseeker" and "... // Bloodsoaked Reveler"); code and verdict are on the front-face name the decklists use. Same for Gollum ("Gollum, Silent Slinker").
- Not modeled by design: a Clue sacrificed does not count as dying when it is a token created by something other than the engine's counts (all engine artifact tokens are counted).
