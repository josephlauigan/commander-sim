# Task: practice-urdragon — the Ur-Dragon deck's combos playable by a person in practice mode (JD, 11/03)

JD asked that every infinite combo in their decks be modelled correctly in practice mode. Deck `urdragon`
(`decklists/JD/ur-dragon-equipment.md`). Fix, for a human seat:

1. **Godo, Bandit Warlord + Helm of the Host** and **Combat Celebrant + Helm**: work by hand but take 30-50
   prompts. Add a shortcut once the loop is live (e.g. the person declares "repeat: attack with everything at <an
   opponent> each extra combat" or a loop button that plays out the combats and logs them), keeping blocks, damage
   and opponents' answers real. Also: Godo's ETB search should let the person pick any Equipment
   (`impls/t6_godo.rs` ~62 limits it to the AI's wanted card); equip discounts (Puresteel Paladin, Steelshaper's
   Gift etc., `impls/cloud.rs` ~184) must apply to the person's equip (`human/legal.rs` ~397); Hammer of
   Nazahn (`t6_godo.rs` ~164), Sigarda's Aid (`impls/partials.rs` ~1036) and Brass Squire (`impls/cloud.rs`
   ~513-548, main 1 only: check its timing vs Oracle) must ask the person which creature to attach to.
2. **Dualcaster Mage + Twinflame:** Twinflame works only as an AI hand option (`impls/mine.rs` ~3147-3207); give it
   a real resolve for the person's cast (target creatures, hasty token copies exiled at end step) so Dualcaster's
   copy (`engine/cast.rs` ~1389 `copy_spell`) copies it; check the person gets a window to flash Dualcaster in
   response to their own Twinflame; add a loop shortcut (infinite hasty Dualcasters → damage/attack win).
3. **Peter Parker's Camera** and **Jaxis, the Troublemaker** (only if in the current list): a person activation that
   copies their ability on the stack / makes the copy, as the Oracle says.
4. Anything else in the current list that is a combo the person can't do: fix or report.

Tests: a practice-mode test per line (see `crates/practice/tests/session_basic.rs`, `session_common`, and human-seat
tests in `crates/sim-core/tests/` — grep `human`).

## Working rules (override rust/agents/RULES.md where they differ)

- Your worktree is your whole world: `cd` into it for every command. It is on origin/main; don't merge `rust-port`
  (gone). Never run Python. No Co-Authored-By trailer. Commit often; don't push.
- Memory cap on every build/test: `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 <command>`
  (exit 137 = a memory bug). `export PATH=$HOME/.cargo/bin:$PATH`; build from `rust/` with `cargo build --release`.
  Run only the targeted tests you add or touch (`cargo test --release -p practice --test <name>` /
  `-p sim-core --test <name>`); never the full suite or a sim — the coordinator runs those.
- **The bots must not change**: gate new behaviour on `crate::human::is_human(g, p)` so sims stay byte-identical.
  Where a fix would only be right as an engine-wide change (it would change sims), don't make it: describe it in
  your report.
- Follow the Oracle text (check `data/scryfall_cache.json` with `jq`). Prefer playing the real rules by hand; where
  a loop would take dozens of clicks, add a table shortcut like Sephiroth's (`human/abilities.rs` `loop_abilities`,
  `impls/mine.rs` `run_loop` / `loops_now`): offered only when the loop is really assembled and payable, gives the
  opponents a window to answer a key piece, then plays out the result and logs it. Another agent
  (`practice-rings`) is adding Rings of Brighthearth copies and a Basalt loop shortcut, touching
  `human/abilities.rs`, `human/karn.rs`, `dsl/mod.rs`, `engine/stack.rs`: keep your edits there small and additive.
- Edit the smallest set of files; format only files you touched (`rustfmt --edition 2024 <files>`).
- Before finishing: `cargo build --release --all-targets` has no warnings and your new tests pass.
- Report: your final message, at most 40 lines: commit hash; per combo what a person can now do (step by step) and
  what is still missing; shared files touched; anything the coordinator must test or decide.
- The audit these fixes come from: `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/practice-combo-audit/rust/agents/report-practice-combo-audit.md`
  (read your deck's section; its file:line notes are from reading code, so verify each one).

## Conventions from earlier practice-mode work
Read the sections "Asking the person" and "Tests" of `rust/agents/practice-fixes-common.md` and follow them (the
`human/choices.rs` helpers; scripted-person tests with `tests/play_common/mod.rs`). Ignore its before/after sims,
thread limits and report-file parts: the rules above replace them (the coordinator runs the identity sims). Use
`-j 4` on cargo commands.
