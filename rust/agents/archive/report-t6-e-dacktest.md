# Report t6-e-dacktest (plan row 19: Dack first-flip shortcut)
Mechanism: `t6_dack::record_first_flip` (called in `flip_and_donate_inner` after the shuffle) stores this turn's FIRST flip when it is exactly
three of the seven creatures: pile bits + the creature the next flip would leave in the library (4th creature from the top of the shuffled
library), in grudge keys `dack_first_flip` / `dack_first_flip_turn`. `impls/t6_combos_t6.rs` section "Dack: the first flip": `pile_number`,
`pile_wins(n, left, spell)` (the 35-pile table), `dack_line` (needs a spell? needs {1}{W} for Preston's ability?), quiet combo
"Dack Fayden: the first flip wins the turn (35 piles)" (COMBOS now 7). Steps: pieces (Permanent: donated Preston/Felidar/banishers still out)
-> flicker spell on Dack if the pile needs it (Spell) -> Preston {1}{W} Solar fetch if needed (Engine, ability window) -> first flicker trigger
(Trigger: Felidar, else Dack) -> win. Dack's own enters trigger keeps its real trigger window (Ertai counters it: tested).
Table choices: followed the traces; "no line found" = loss; pile 18 with a spell wins when Felidar is not left (trace: "not fully traced");
only Cloudshift/Ephemerate count as spells (Touch the Spirit Realm not used); fetch ({1}{W}) only for piles 6-15 with Solar in the pile or left.
Tests (tests/engine_t6_dacktest.rs, 8): table vs all 35 summaries (exact; 95/140 = 23.75/35, groups 20/25/24/26), pile numbering, class (a) win,
Swords on Preston stops it (>=7/10), Ertai counters the ETB, 23 wins only with Cloudshift/Ephemerate, ready follows the library's left-over card,
Monte Carlo 2,000 fresh resolutions (seeded, random library order, no interaction): **1366 same-turn wins = 68.3%** (a 100%, b1 63.6%, b2 62.5%, c 63.1%); asserted 58-78%.
Full `cargo test --release`: all pass; `cargo build --release --all-targets`: no warnings. No behaviour change for other decks (new quiet combo, snapshot only written on Dack flips).
Gaps: real engine plays its own triggers before the shortcut (the abstract line then wins; loops not played out); Touch the Spirit Realm lines; the
spec combo tests of other decks not covered here (Sisay Selvala chain, Sisay-fetch, Deadpool+Orchard, Mana Vault starter, Rograkh Badgermole/Springheart) are unbuilt combos, not tests: not started.
