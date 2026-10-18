# Report t6-e-combo3 (combo lines part 3)
Module: impls/t6_combos_extra.rs (one `pub mod` in mod.rs, one `.chain` in `combos::all()`). Tests: tests/engine_t6_combo3.rs (32). Full suite passes, no warnings.
Step tables: Oracle + Consultation/Pact (cast Oracle -> exile spell in response -> Oracle trigger window, finish = alternate win);
Kinnan + Basalt (pieces survive, then Kinnan's ability as the outlet). Both are `BASE` entries pointing at `ORACLE_STEPS` / `KINNAN_BASALT_STEPS`.
New combos (11): Kinnan+Basalt+Thrasios / +Mirage Mirror (when no {U}{G} can be paid); Breach+LED+Brain Freeze with Oracle finish and "at the table"
finish (real escapes, real storm mills via the card code, each cast counterable); Ad Nauseam into a win (real Ad Nauseam, then whichever line the
cards made ready, with the mana left); Hullbreaker Horror + Kinnan + two rocks; Godo: Welder+Engineer (Engineer -> Godo -> Welder swap, Hammer attaches Helm),
rebuild with Godo+Hammer out (Cursed Mirror/Steel, Jaxis, token copy spell), Basalt + a copied opposing Kinnan (Heat Shimmer / Cursed Mirror, loop = floating mana).
Behaviour changes: (1) base Kinnan+Basalt is ready only when {U}{G} is payable (else the Thrasios/Mirror outlets). (2) all new combos are "quiet"
(`combos::QUIET` first group): not registered as pieces, so no other deck changes (is_piece_name/pieces/piece_threat/combo_imp skip them).
(3) t6_godo::godo_wants: a Cursed Mirror/Sculpting Steel/copy spell on the battlefield no longer counts as a copier (Godo deck only). (4) t6_combos_godo
helpers/consts made pub(crate); Step derives Clone+Copy; welder_go pub(crate).
Gaps/needs (engine-needs-found.md): Ad Nauseam cast importance is low, so the table doesn't counter it (no counter test); Cursed Mirror doesn't truly copy
(stat-copy only), so Mirror lines use the same shortcut as combo1. Real loops are shortcuts except the Breach loop.
