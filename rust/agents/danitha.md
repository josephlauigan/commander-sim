# Task: danitha — model Danitha Capashen, Paragon's cost reduction (one card)

Read `rust/agents/T0-RULES.md` (Hard rules, How cards are modeled, Checks before you finish items 1-3). Branch cloud-sd.
`-j 16`. Be quick (~20 min); JD is waiting. No win-rate runs.

Danitha Capashen, Paragon ({2}{W} 2/2, first strike, vigilance, lifelink): "Aura and Equipment spells you cast cost {1}
less to cast." Today `data/card_overrides.toml` marks her Partial: the discount isn't modeled (seen as "not modeled"
in a 10-15 run). Model the discount with the existing cost-reduction machinery (grep how Foundry Inspector, Puresteel /
Sram-style or other "cost {1} less" statics hook into `engine/mana.rs` cost_of / the `cost` slot), generic part only,
can't reduce below the coloured part. Mark her Full with a note; test: with Danitha out, an Equipment costs 1 less, an
Aura costs 1 less, a creature spell doesn't; two Danithas would stack (rules: each applies). Check Sram, Senior Edificer
is Full too (`cardgen audit --cards`). `cardgen export`, `cardgen validate`, full `cargo test --release -j 16
--no-fail-fast`, `--all-targets` no warnings. If you touch a shared engine file, run a same-seed identity check of the
other 10 JD decks (parent binary in a detached worktree under .claude/worktrees/, removed after; `--deck <d> --pool all
--games 300 --ai adaptive --jobs 16 --seed 7 --brief`). Commit (no Co-Authored-By), never push. Report ≤8 lines,
line 1 DONE/HANDOFF, line 2 model ID.
