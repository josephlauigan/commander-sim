# Handoff: t0-anikthea (first agent stopped at ~60 tool calls)

## Done (committed; `cargo test --release -j 3 -p sim-core --test cards_t0_anikthea`: 25 tests pass)
26 of the 30 cards are Full in `cardgen audit --deck anikthea-enduring-enchantments` (overrides at the end of
data/card_overrides.toml, card code in impls/t0_anikthea.rs, tests in tests/cards_t0_anikthea.rs):
- Saga frame (`saga_enter`, `saga_main1`, `saga_lore`, `run_chapter`, `chapter`, `saga_final` for Narci) + the 7 Sagas.
- Constellation: Composer, Nessian, Doomwake, Ondu, Boon (creature_pt + g.selfpt), Khalni (landfall + option), Mirari's Wake
  (land_mana +1), Heliod God + Erebos (devotion toggle via DataKey::Anim, types overridden to "E"), Narci, Greater Tanuki
  (hand_options channel), Nyx Weaver (option), Nyxborn (self_cost), Sandwurm (combat::attack_limits calls
  `t0_anikthea::flyer_barred`), Anikthea (copy on enter/attack, menace grant), Abundance (zones::draw calls
  `abundance_draw`; registered with a no-op skip_draw only so it is "live"), Cacophony, bouncelands (tags override).
- Shared-file changes: engine/zones.rs (3 lines in `draw`: abundance_draw), engine/combat.rs (1 line in `attack_limits`:
  flyer_barred). Nothing else outside my module, tests and data.

## Still Partial/Unmodeled (4 cards), in this order
1. **Calix, Destiny's Hand** (planeswalker): use `common::walker(r, db, "Calix, Destiny's Hand", &CALIX)` with WalkerAb
   list (+1 look at top 4, may take an enchantment, rest bottom random; -3 exile target creature/enchantment you don't
   control until target enchantment you control leaves: `oring_exile(g, ench, p, pred, false, false)` with the linked
   ENCHANTMENT as `src`, which needs a `leaves` hook calling `oring_return` on every enchantment card of the deck, or the
   exile list stored on Calix; -7 return all enchantment cards from the graveyard, Auras only onto a host). Override
   `dsl = []` in card_overrides.toml (the compiled -3/-7 would double up), status Full.
2. **Demon of Fate's Design**: `options` slot: once per turn cast an enchantment card from hand paying life = mana value
   (lose_life then `cast::cast_card(g, p, c, "hand", Ctx::default())`, mana is paid before cast_card, not inside it).
   The {2}{B} sacrifice-an-enchantment +X/+0 is a compiled activation: check it really adds the sacrificed card's MV.
3. **Cunning Rhetoric**: Event::Attack hook (src is the defender's enchantment: `d == owner(src)`): exile the attacker's
   top card, put it in the owner's hand marked in `impulse_long` with a far-future turn (see tymna_cards/mascot_cast),
   plus an `options` entry to cast it with any colour of mana (pay `cmc` generic, then cast_card; lands: play as the land drop).
4. **Ghoulish Impetus**: `common::aura(r, db, name, &SPEC)` with AuraSpec { pow 1, tgh 1, kws ["deathtouch"], target "opp",
   host_dies: return the Aura at the beginning of the next end step }. Goad is t0-zurgo's mechanic: check whether it has landed
   on main (grep goad) and reuse; otherwise MustAttack (DataKey) each turn and say so.

## Then the TO-DO from T0-RULES "Checks before you finish"
full `cargo test --release -j 3`, `cargo build --release -j 3 --all-targets` warnings, all_decks adaptive 3 and lookahead 1,
audit, the smoke run, and a trace (`--trace 0`, find a game seating the anikthea deck) to confirm the commander is cast early and
Sagas / constellation fire. Then write rust/agents/report-t0-anikthea.md and the final 15-line report.
Gotchas: never `python*`; `trigger_window` inside etb hooks (don't mark them at_once); Saga lore is DataKey::Lore;
pool decks are not hooks-checked by the AI, so check in a trace that Abundance/Boon are cast (generic_prio: `prio` slot).
