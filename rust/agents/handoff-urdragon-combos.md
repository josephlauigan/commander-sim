# Handoff: urdragon-combos

Brief: `rust/agents/urdragon-combos.md`. Branch `urdragon-combos` (worktree `.claude/worktrees/urdragon-combos`).

## State
- [x] `impls/t45_urdragon_combos.rs` (three catalogue entries), registered in `combos.rs` `all()` and `impls/mod.rs`
- [x] `t45_miirym.rs` `astral_etb`: skip while `g.combo_spell` (the combo's own trigger step stands in for it)
- [x] `data/plans/urdragon.toml`: three `[[line]]`, a prio for Astral Dragon, deck-document comment
- [x] `tests/combos_urdragon.rs`
- [x] `cargo test --release --no-fail-fast`, `cargo build --release --all-targets` (no warnings)
- [ ] `rust/agents/report-urdragon-combos.md` (could not be written: the report is in the final message)

## Design notes
- Stepped combos (as the Godo lines): the steps pay their own mana; `ready` checks the mana and the zones.
- Astral Dragon + Cursed Mirror and the Garth line need a combat still to come (hasty tokens): `precombat`.
- Miirym + Astral + Doubling Season: the board is summoning sick, so the finish floods the board with 3/3 flying Dragon
  tokens (capped by `TOKEN_CAP`) and does not win outright.
