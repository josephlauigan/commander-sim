# Report t6-e-legend
Mechanism
- Legend rule: engine/life.rs `sba_legend` (called in check_state after sba_toughness). Per controller, legendary permanents (cardcode::is_legendary, reads CopyExc) grouped by name; the AI keeps the best (untapped +2, equipment +1.5 each, counters, real card over token, older). Loser: `die(.., "sba")`. Lands not covered.
- Blood Moon / Magus: engine/mana.rs `moon_land(g, l)` / `moon_card(g, cd)`. Gated: mana_units (amount 1, no Tomb/Workshop/dyn-mana), is_pain and pay (no Tomb/pain damage, no on_tap_land), land_options and land_upkeep (lands.rs, cardcode.rs), land_defend, land_etb (cardcode + Planar Genesis), land_etb_fx (zones.rs), saga_step (no lore counters), play_land_card (no Chasm/bounce/fetch), crack_fetch, land_enters_tapped and the pay-life rule, land types for check/basic-type rules (turn.rs) and Korlash. `moon_amt` land_mana slot of Blood Moon removed.
- Damping Sphere: new Event `LandColorless` (hooks.rs, sorted, EVENTS len 82), engine/hooks.rs `land_colorless`, asked in mana_units after all bonuses: amt>=2 -> one {C}. t6_stax `sphere_land` now uses it.
Cards: Blood Moon, Magus of the Moon, Damping Sphere -> Full.
Tests: tests/engine_t6_legend.rs (13). Full suite and build pass, no warnings.
Rules note: Urza's Saga under Blood Moon is NOT sacrificed (it loses the Saga subtype, so 704.5s does not apply): it stays as a plain Mountain.
Affected decks: legend rule: every deck with a copier/duplicate of a legendary (Godo, Kraum/Tymna blue, Sheoldred/Seph clones, Helm tokens stay). Blood Moon/Magus: decks running them (godo, Magus owners) and opponents with nonbasic lands (Ancient Tomb, Saga, manlands). Sphere: decks with bouncelands/two-mana lands.
Not done: legendary lands (two Cradles) are not under the legend rule.
