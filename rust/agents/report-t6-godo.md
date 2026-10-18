# Report t6 godo (14 cards, module impls/t6_godo.rs, tests/cards_t6_godo.rs, 33 tests)

Helm of the Host: already modeled (impls/t3.rs `helm`, `combat_start` slot: a hasty token copy of the equipped
creature at the start of the controller's combat); not in my list, untouched. Godo's token copy triggers Godo's own
enters and attack hooks, as the card does.

| Card | Status | Notes |
|---|---|---|
| Godo, Bandit Warlord | Partial | ETB fetches an Equipment onto the battlefield (Helm; Hammer first with a copier out/in hand; then Hammer); first attack each turn untaps Godo + Samurai, one extra combat. Engine caps combats at 4. |
| Hammer of Nazahn | Full | Equipment you control enters: attaches it (Godo first). |
| Magnetic Theft | Full | Moves own loose Equipment (Helm first) onto best creature (Godo first). |
| Sting, the Glinting Dagger | Partial | Untap at own combat only; no first strike clause. |
| Goblin Welder | Full | Own artifact for own graveyard artifact when worth 3+ more. |
| Goblin Engineer | Full | ETB to graveyard (Helm/Hammer with Welder, else Cursed Mirror); {R},T, sac: return MV<=3. |
| Tezzeret, Cruel Captain | Partial | 4 loyalty, +1 per artifact, 0 and -3; no -7 emblem. |
| Party Thrasher | Partial | Main-phase loot-impulse; no convoke from exile. |
| Vexing Bauble | Partial | Draw ability; counter clause as a can_cast lock on zero-mana spells. |
| Hexing Squelcher | Partial | Spells you control uncounterable; no ward-pay-life. |
| Blood Moon | Partial | Nonbasics tap for R, 2-mana lands make 1; no ability loss. |
| Defense Grid | Partial | +3 outside caster's turn; alt costs untaxed. |
| Final Fortune, Last Chance | Partial | Extra turns do not exist in the engine; spells never cast by the AI. |

Engine needs: rust/agents/engine-needs-godo.md (extra combats > 4, beginning of every combat + conditional first strike,
emblem for Tezzeret, convoke from exile, counter-on-cast hook, non-mana ward + granted ward, Blood Moon ability loss,
alt-cost tax, extra turns).
Shared files touched: only impls/mod.rs (2 lines), data/card_overrides.toml, data/cards.json (export). No engine/ai files.

Full `cargo test --release`: 107 suites pass; `cargo build --release --all-targets`: 0 warnings.
One test outside my batch changed: tests/cards_t6_mana.rs `shaku_does_not_tap_a_creature_that_can_attack` used Godo as
the legendary creature; Godo now fetches the legendary Helm, which Shaku counts, so the test uses Jaxis instead.
