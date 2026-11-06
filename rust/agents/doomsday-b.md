# Task: Doomsday follow-up, close the open items of PR 2 (JD: record snags and continue; 2026-11-06)

Worktree: `/home/joseph/projects/commander-sim/.claude/worktrees/doomsday-b` (branch doomsday-b, from doomsday 41dacca6).
Task name: `doomsday-b`. **No sims.** Read `rust/agents/report-doomsday.md` first (section "Open"), then
`rust/crates/sim-core/src/impls/doomsday.rs`, `tests/doomsday.rs`, and the pilot guide
`decklists/JD/tymna-thrasios-doomsday.md` (Line 2: Doomsday; "what can go wrong"; the keep table).

**JD's rule: no gap may be left quietly.** Anything you can't make right: commit, list it at the top of your reply under
`NEEDS JD`, and carry on.

## Items
1. **Pile extras in the search** (`solve`/`dfs`, `build`): add as moves/outs where the list has the cards:
   Thrasios, Triton Hero's {4} ability (scry 1, then reveal the top card: a land goes to the battlefield, else into hand)
   as a draw source when Thrasios is out and the mana is there; Mockingbird / Snapback as outs if the guide uses them
   (check the guide and the Oracle text); Veil of Summer in place of Silence against blue/black stoppers (Mindbreak Trap,
   Red Elemental Blast are not blue/black: check what Veil actually stops per Oracle and use it only where it does);
   Esper Sentinel's forced draws (a pile played while an opponent's Sentinel or similar forces draws must count them, or
   the AI must not start it); Borne Upon a Wind's own draw when it is the end-step enabler (the pile is then built with
   that draw counted).
2. **Noxious Revival as a pile slot / backup**: score piles by what remains if the Oracle is countered (Revival in hand or
   in the pile lets a riskier pile be accepted); keep it correct per the Oracle (Revival puts a card from a graveyard on
   top of its owner's library).
3. **A general coloured-source keep term**: a `plan_data.rs` quantity for "the hand has a source of each colour its
   spells need" (lands, rocks, fetches that can find them), and use it in thrasios.toml's keep rule per the guide's keep
   table ("no coloured source for its spells goes back"). Keep `hand-b-lands` working or replace it with the general one.
   Check other plan files still parse and their keep rules behave the same (no other file is required to use it).
4. **The table's risk check on the Doomsday cast**: run the same `go_risk` / stop-chance check the other Tier 6 lines use
   (see `combos.rs` / `plan_data::t6` go_now and `go_ok`), in addition to the existing declines, so the cast waits when the
   table can stop it and there is no cover (Silence in the pile, a free counter, Veil).
5. Update `rust/agents/report-doomsday.md` "Open" (as text in your reply; the coordinator edits the file) and
   `audit/thrasios/FINDINGS.md`'s Doomsday section if it names these gaps.

## Tests
Add to `rust/crates/sim-core/tests/doomsday.rs`: one or more per item (a pile that needs Thrasios's dig wins; a pile with
Veil wins through a Veil-able counter; an opponent's Esper Sentinel doesn't break a pile (or the AI declines); a riskier
pile accepted with Revival available; the keep rule with no blue source goes back; the cast waits under a table that can
stop it without cover). Run `--no-fail-fast`: `doomsday`, every test file seating `thrasios`, `ai_planner`, `thrasios_ai`,
`plans`, `plan_files_3`, `generic_paths`, `practice_*` probes, `play_cards`, `cards_t6_*`, `engine_t6_*`, `ai_t6_*`.

## Done when / reply
Items done (or under NEEDS JD); tests pass; build without warnings. Reply: at most 15 lines, then `---- FULL REPORT ----`
with per item what changed, files, tests, shared-file changes.

## Ground rules (these override the older rust/agents/RULES.md, which is from the Python-port era)
- Work only in your worktree; `cd` into it for every command. Don't merge other branches. Don't push.
  Toolchain: `export PATH=$HOME/.cargo/bin:$PATH`; build from `rust/`.
- **Never run Python, not even a version check.** Use jq, shell tools, or Rust.
- **Memory cap on every cargo command and every binary you run, no exceptions**: `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 <cmd>`
  (cargo from `rust/`). zsh: write the prefix out in full. Exit 137 = cap hit = a bug.
- No sims (no `--realism`, no multi-game runs) and no full suite; single `--trace N` games are fine. Leave nothing running.
- Other agents work in other worktrees: never touch files outside yours.
- Never let an unsigned counter go below zero. AI and person code stay separate (`if crate::human::is_human(g, p)`).
- `rustfmt --edition 2024 <your new files>` only; never `cargo fmt` on the workspace.
- Commit often (inside the worktree); **no Co-Authored-By line**. Keep `rust/agents/handoff-doomsday-b.md` current. If your
  context gets long (~80+ tool calls), stop at a clean point, everything committed, reply's first line `HANDOFF`.
