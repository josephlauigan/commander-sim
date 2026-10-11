# Handoff: zada-cards (DONE: all cards Full, checks run; see report-zada-cards.md)

## Done (committed; 20 tests in rust/crates/sim-core/tests/cards_zada_cards.rs pass)
Full in `cardgen audit --mine` now: Goblin Arsonist, Bomber Corps, Glitterfang, Goblin Rimerunner, Skilled Battlecarver,
Rundvelt Hordemaster, Pia, Wrath of the Bloodmane, Suplex, Heated Argument, Violent Echoes, Orcish Cannonade,
Scorching Dragonfire, Reiterating Bolt (energy never paid: the deck makes none), Krenko's Enforcer (intimidate),
Blazing Firesinger / Goblin Glasswright / Strife Scholar (prepared + Strife's ward-pay-2-life).
Mechanics: **prepared** = rows in `mine.rs` PREPARED (+ `prepared_effect` 4/5/6, `prepared_value`) and tag `prepare` in
the card override; spell halves "Craft with Pride", "Awaken the Ages" are rows in data/carddb.txt.
**intimidate** = `rules::evasion_blocked` (kw "intimidate"). Ward-life = `rules::removal_taxes` (Strife Scholar).
Status = `status`/`note` in data/card_overrides.toml (section "zada-cards" at the end); run `cardgen export` after edits.
Code: rust/crates/sim-core/src/impls/zada_cards.rs (registered in impls/mod.rs after jodah).
Shared-file edits so far: impls/mine.rs (PREPARED table, prepared_effect/value), impls/rules.rs (intimidate, ward life),
impls/mod.rs, data/carddb.txt, data/card_overrides.toml.

## Left (audit --mine, in this order; designs already worked out)
1. **Burn, Burn, Tree and Fern** (Saga) and **Ral and the Implicit Maze** (Saga): copy the Fable pattern in `impls/t3.rs`
   (~line 484): `etb` sets `DataKey::Lore`=1 and runs chapter I through `trigger_window`; a `main1` slot fn (PlayerFn,
   `p == owner`) adds a lore counter and runs the chapter; sacrifice (`die(g, src, "sac")`) after the last chapter.
   Burn: I = 6 damage to an opposing creature (`apply_removal(.., "dmg6", None)`, best `pval` target), II = destroy an
   opposing artifact (`legal_targets(.., "destroy", "a", ..)`), III/IV = `g.player_mut(p).floating.r += 1` (add {R}; floating
   mana empties at end of phase: fine). Ral: I = 2 damage to each creature/PW opponents control (`apply_removal` "dmg2"
   on each), II = may discard (`discard_worst`) then exile top two, playable until end of next turn (see
   `playable_until_next_turn` idea: push to hand + `impulse_long.push((c, pl.turns + 1))`), III = Spellgorger Weird token
   (a 2/2 red Weird; check how zada-core models the Weird's counter trigger, tag/`DataKey::Counters` in zada.rs).
   Proliferate must not matter (no proliferate in deck). Tags needed: none beyond override (`dsl = []`).
2. **Command the Stage**: `resolve` hook: Cadet token 2/2 colourless types wizard+soldier, then +1 counter (`g.perm_mut(m).plus += 1`)
   on each OTHER Wizard token you control (`ttypes` contains "wizard"). The recursion clause (at each upkeep, if an opponent
   was dealt noncombat damage last turn, return it from the graveyard to hand) needs three small shared edits: a Player
   field (state.rs) set in `engine/life.rs::lose_life` when `damage && kind != "combat" && src != p` (store the TurnStamp),
   a new slot `gy_upkeep` in hooks.rs (fn(&mut Game, CardId, owner, active)) and a loop over every player's graveyard where
   `Event::Upkeep` fires in engine/turn.rs (~line 483). "Last turn" = the previous turn in seat order.
3. **Glimpse the Impossible**: `resolve`: exile top three, all playable this turn (hand + `pl.impulse.push`), at the next end step any
   still-exiled ones (the `impulse` list entries still in hand) go to the graveyard and make a 0/1 Eldrazi Spawn each
   (turn.rs ~688 already exiles unplayed `impulse` cards at the end step: count them there, or do it from an `end_step`
   delayed hook; note an Eldrazi Spawn token = `Tokens{ttypes: ["eldrazi","spawn"], color NONE}`, mana unit `Source::Scion`).
4. **Irascible Wolverine** (+ **plot**): ETB: exile top card, may play it this turn (`impulse`). Plot {2}{R}: `hand_options`
   returns `Action::Plan` (see `alela::plan`) only when `post == Some(true)` on your turn; it pays {2}{R} and marks the
   card with `impulse_long.push((c, u32::MAX - pl.turns))` (the engine keeps exile-playable cards in the hand, so plotted cards
   follow that convention); a second option on a later turn (`turns > u32::MAX - t`) casts it free:
   remove the impulse_long entry, `cast_card(g, p, c, "hand", Ctx::default())` (cast_card doesn't pay mana). Override
   `dsl = []` (the compiled ETB "draw a card" is wrong).
5. **Lightning, Security Sergeant**: `combat_damage` hook (a == src, d a player): exile top card, playable while you control Lightning:
   hand + `impulse_long.push((c, u32::MAX))`, remember cards in Lightning's perm data (new `DataKey::Linked`, `Val::List`
   of `Val::Card`, one small enum addition in state.rs); `leaves` hook for itself moves cards still in hand to exile and
   drops them from `impulse_long`. Override `dsl = []` (its compiled "draw a card" is wrong). Menace is a keyword already.
6. **Living History**: ETB 2/2 red and white Spirit (`Tokens{color RW, types ["spirit"]}`); "whenever you attack, if a card left your
   graveyard this turn, target attacking creature gets +2/+0": `attack` hook; needs a per-turn flag "a card left your graveyard"
   (Player field set where cards leave `gy`: casting from gy, flashback, Dreadhorde recasts (zada.rs), Wolverine...); the
   narrowest honest version: set the flag in the places zada-cards/zada-core code removes a card from `gy` + `engine/cast.rs`
   (`"gy" | "escape" | "mgy"` arm of cast_card). Override `dsl = []`.
7. **Spawn-Gang Commander**: `self_cast` slot: three 0/1 Eldrazi Spawn tokens when cast. Devoid (check `colors_of` handles
   kw "devoid"; else say so). {1}{C}, sacrifice an Eldrazi: 2 damage to any target: an `options` hook paying {C} from a
   colourless unit (`mana_units` entry with empty `cols`, e.g. a Spawn via `Source::Scion`, Bonders' Enclave, Giant's Boulder...)
   via `spend_unit` and {1} via `pay(g,p,1,"",false)`, sacrificing another Eldrazi (a Spawn) with `die(.., "sac")`. Override `dsl = []`.
8. **Relic of Legends**: keep `rock=1:A`; the second ability (tap an untapped legendary creature: add one mana of any
   colour) = an `extra_mana` hook (see `urza_mana` in t5.rs / `quirion` in partials.rs) returning a `Unit{src: Source::Perm(creature), cols: ident, amt: 1}`
   for each untapped legendary creature of yours (summoning sickness doesn't matter: it's not a {T} cost of that creature... it IS
   a tap cost, but "Tap an untapped creature you control" ignores sickness).
9. **Bonders' Enclave**: land with `land_options` hook (g, land, p, post): {3},{T}: draw a card if you control a creature with power >= 4
   (`epow`); see `lands.rs` for Karn's Bastion's pattern (`can_pay_without`, `pay_without`). It is hand-tagged in carddb.txt
   (`c=C`): the audit then reads `documents/card-audit.md` or an override status; add an override with status/note.
10. **Generator Servant** (Approximate -> Full): keep tags `dork=C noatk`; add `dyn_mana_perm` (amount 2), `on_tap_perm`
    (sacrifice it: `die(g, m, "sac")`; if only 1 of the 2 was used, add the other to `floating.c`), and for haste: when
    `g.pay_for` is a creature card, remember it (new Player field `haste_next: Vec<CardId>`, consumed in `zones::enter`
    where `x.sick = how.sick && !haste`). Both are small shared edits (state.rs, zones.rs).
11. Then: tests for each (one per card at least) in cards_zada_cards.rs, `cargo test --release` (all), `cargo build --release
    --all-targets` (no warnings), `all_decks adaptive 3` and `all_decks lookahead 1`, `commander-sim --deck zada --pool t3 --profile
    loose --games 48 --ai lookahead --jobs 8 --brief`, then the report `rust/agents/report-zada-cards.md`.

## Gotchas learned
- Overrides with `tags` REPLACE the card's tags (copy the existing ones from data/cards.json); `dsl = []` is needed whenever the
  compiled abilities are wrong/partial and the tag or `resolve` path should win (a spell with DSL and no `resolve` hook resolves
  from the DSL, not from `rem`/`draw`/`selfdmg` tags).
- A hand-written `resolve` gets the target from `ctx.target`/`ctx.face` chosen by the usual tag-driven AI (`rem`, `tgt`, `face`).
- "Can't block" is read only while `g.dsl_on` is true: set it when you grant `cant_block`.
- Tokens are named "Token": test them by `ttypes`. Treasures/Food/Clues are counters on the Player, not permanents.
- Planeswalker targets: `legal_targets` never picks a planeswalker for damage spells (engine-wide); the notes say so.
- Don't run Python; run everything under `systemd-run --user --scope -q -p MemoryMax=8G -p MemorySwapMax=0`.
