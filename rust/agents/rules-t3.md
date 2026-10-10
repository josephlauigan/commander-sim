# Task: the rest of rules.py, rules2.py and t3.py (Tier 3) (+ two bug fixes)

Worktree: `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/agent-aa6c102ce04c4b5ed`
(branch `worktree-agent-aa6c102ce04c4b5ed`). A previous agent was stopped while fixing three failing tests; its
work is in WIP commit 7cfbf73. Check `cargo test --release` first.

Files: `impls/rules.rs`, `impls/rules2.rs`, `impls/t3.rs`, tests `tests/cards_t3.rs` (and cards_rules.rs).

1. Port everything still unported in rules.py, rules2.py and t3.py. After you, every Tier 3 deck (data/decks.json
   `pool`, tier t3) has all its Python card code in Rust (yours, or listed as another module's). Placeholders: any
   cardcode.rs placeholder naming a rules.*, rules2.* or t3.* function (e.g. the Legion Loyalist clause of
   `evasion_blocked`; planeswalker ultimates in rules*.py appended to walker lists — t4/t5 register KARN, TEZZERET,
   JACE_WOM, Kaito, Chandra; rules registers later, so its full lists win).
2. t2.rs registers Charming Prince, Eldrazi Displacer, Whir of Invention, Beseech the Mirror, Wishclaw as dead code
   because rules2's versions replace them in Python: make sure rules2 registers its versions.
3. After merging rust-port: t4.rs has private copies of `t3.sac_worst_permanent` and `rules2.devotion` (marked
   MERGE): point them at yours. t3.rs's local `proliferate` copy: use common's if it exists with a compatible
   signature.
4. Bug fixes (Rust only, a test each): (a) Bloodchief's Thirst is never kicked (`p.thirst_kicked` never set): let
   the AI kick it when it can pay and the best target needs it; (b) Legion's Landing never transforms (`p.adanto`
   never set): transform into Adanto, the First Fort when you attack with three or more creatures.

Difftest: `python3 rust/tools/difftest.py --gen 1000 --seed 7 --decks t3,t2`.
Commit message: "Rust port, phase 6: the rest of rules, rules2 and t3; two bug fixes".
