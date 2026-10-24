# Report t6-e-sba
Mechanism: engine/life.rs `sba_toughness` (called from `check_state` before the player-loss loop): every creature of
a live player with etgh <= 0 dies via `die(.., "sba")` (ignores indestructible and regeneration). Cheap prefilter:
on_bf, not phased, not army (as on entering), stored tgh+plus+eot_pt.1 < 6, then is_creature, then etgh.
Removed the workaround in impls/t6_redirects.rs (Eldrazi Confluence) and the unused `die` import.
(b) Already worked: zones::draw sets `decked` on an attempted draw from an empty library (Lab Man/Jace win first via
LABMEN); check_state eliminates at the next check. Tests added (tests/sba.rs, 5): zero toughness, indestructible +
regeneration, Dismember -5/-5 on indestructible Godo, decked loses only after attempted draw / empty library alone no.
Affected: no existing test changed or failed; Confluence test passes via the engine check. Other decks: any creature
at toughness <= 0 from -X/-X, static or eot effects now dies at the next check (previously only some paths).
Flaky once under load: practice http_play test_replayed_history_is_marked (passes on rerun).
No cards flipped (Eldrazi Confluence already Full).
