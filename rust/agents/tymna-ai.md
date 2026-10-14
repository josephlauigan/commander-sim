# Task: tymna-ai — the AI plans of the thrasios and kraum decks

Read `rust/agents/TYMNA-RULES.md` first (it points to OVERNIGHT-RULES). Base branch: `tymna-thrasios` (the merge of
partner-engine and tymna-cards plus this brief). Your worktree's branch: `tymna-ai`. Your module:
`rust/crates/sim-core/src/impls/tymna_partners.rs` (`thrasios_prio` / `kraum_prio` are stubs now), tests
`rust/crates/sim-core/tests/tymna_ai.rs`. Deck documents: `decklists/JD/tymna-thrasios-partners.md` and
`decklists/JD/tymna-kraum-partners.md` (read Strategy and Key lines). Card code: `impls/tymna_cards.rs`, Tymna in
`impls/legends.rs`. Both decks are registered (`--deck thrasios`, `--deck kraum`); the seat has two commanders
(`Player::cmd`, `Player::cmd2`, helpers in `state.rs`). These decks will be measured against each other, so give both
the same quality of plan.

## 1. Spread the attack for Tymna (the big one)
Tymna draws one card per opponent dealt combat damage this turn, but the combat code sends every attacker at one
defender and only moves one when it clearly helps (`engine/combat.rs` `split_attack`, called around line 920; the
look-ahead's `all` plan puts everything at one player). For the `thrasios` and `kraum` seats, **while Tymna is on the
battlefield, or Reconnaissance Mission / The Aetherspark are out** (or the attack would get her trigger anyway): after the
normal split, make sure each living, attackable opponent who can be hit gets at least one attacker that will
connect (it can't be blocked by that player's untapped potential blockers: flying vs no flier/reach, can't be
blocked, menace vs one blocker, protection, power limits, ...; reuse `blockable` and friends), the cheapest such
attacker first, without breaking a lethal attack. A small 1-power evasive creature per opponent is worth a card each.
Gate every change in shared code to these two keys so other decks' games stay identical.

## 2. Priorities (`thrasios_prio` / `kraum_prio`)
- Mana creatures and rocks first on turns 1-2; Thrasios on turn 2 when possible (cheap engine; his activation AI is
  already in `tymna_cards.rs`); Tymna as soon as she can be cast (turn 3); Kraum at 5 mana (he has haste: cast him
  precombat so he attacks the same turn).
- Cheap evasive creatures before big ones until each opponent can be hit; then the finishers.
- Hold protection (Shardmage's Rescue, Ultimate Magic: Holy) for removal on Tymna / the commanders when the deck's
  protect-response dispatch allows; Bastion Protector once both commanders are out.
- Look at how other decks' modules (`impls/titania.rs`, `impls/yshtola.rs`) do it, and at `ai/decks.rs` dispatch.

## 3. Check it works (numbers in the report)
Same-seed check: other decks unchanged (`--deck sauron --pool t3`, `--deck cloud --pool t5`, seed 7, 48 games,
lookahead, identical result lines before and after). Then for each deck, `--analyze` (see `sim-cli` args) or traces of
96 games vs t1 and t3, before and after your change: Tymna cards drawn per game, opponents hit per Tymna turn, turn
Tymna/Thrasios/Kraum land, Thrasios activations per game, win rate. Write them in the report.

## Done when
OVERNIGHT-RULES checks 1-3 pass, the same-seed check is identical, 96-game smoke runs finish for both decks, the
numbers above are in `rust/agents/report-tymna-ai.md`.
