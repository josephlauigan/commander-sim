# Handoff: ai-rules (Phase C2)

Branch ai-rules, worktree .claude/worktrees/ai-rules. Brief: rust/agents/ai-rules.md.

## Done (committed unless noted)
1. Council's Judgment AI voters (impls/rules.rs `judgment`, `judgment_score`).
2. Liliana of the Veil -6 piles with lands and tokens (rules.rs `Stuff`, `lotv_ult`).
3. Liliana, Dreadhorde General -9 AI keep set (rules.rs `ldg_keep`, `ldg_ult`).
4. Chain of Vapor copy chain (impls/partials.rs `chain`, `chain_links`, `chain_ai`, `chain_person`).
5. Maelstrom AI (impls/t45_urdragon.rs `maelstrom_options`).
6. Loran option (impls/partials.rs `loran`, `card_use`).
7. Revivalist target on activation (impls/galadriel.rs).
8. Gadgeteer: common.rs Food, sheoldred.rs/t45_edgar.rs Blood, dsl/mod.rs `pay_ability_cost`, cloud.rs equip, Scepter (t4.rs),
   Pod (t2.rs), Foundry (fixes.rs), Maskwood (galadriel.rs), Jester's Cap (yshtola.rs). (uncommitted until check passes)

## To do
- tests/ai_rules_c2.rs, then run the named existing tests, `cargo build --release --all-targets` (no warnings), rustfmt new file.
- Final report (summary <= 15 lines + full report).
