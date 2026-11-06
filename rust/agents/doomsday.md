# Task: model Doomsday and give the new Tymna/Thrasios deck its Doomsday lines (JD approved, 2026-11-05)

Worktree: `/home/joseph/projects/commander-sim/.claude/worktrees/doomsday` (branch doomsday, from deck-rewire f210aac7:
the rewire that points `thrasios` at `decklists/JD/tymna-thrasios-doomsday.md`, tier t6, and makes thrasios a Tier 6
key). Task name: `doomsday`. **No sims** (JD: one baseline after all the engine work).

**JD's rule: no gap may be left quietly.** Anything you can't make match the rules: stop that part, commit, and list it
at the top of your reply under `NEEDS JD` (exact gap, effect in games, what it would take). Then carry on with the rest.

## Read first
- `decklists/JD/tymna-thrasios-doomsday.md`: the pilot guide. "Line 2: Doomsday" (timing, life, Piles A/B/C, the Thrasios
  devotion-3 variants, "when Doomsday is the right line", "what can go wrong"), the Oracle lines, Necropotence/Ad Nauseam
  notes ("never cast Ad Nauseam after Doomsday"; Necro skips the draw step, which switches off Piles A and B), the keep table.
- `data/plans/thrasios.toml` (provisional Tier 6 plan from the rewire), `rust/crates/sim-core/src/ai/plan_data.rs` (the
  plan loader; `plan_data::t6` for Tier 6 slots prio/wish/keep/hold_up/go_now), `impls/t6_plans.rs`.
- How Thassa's Oracle, Demonic Consultation and Tainted Pact are modelled (grep the card names in `impls/` and `ai/`), and
  how other Tier 6 decks' combo lines are planned and executed (`impls/combos.rs`, `plans/*.toml`).
- Oracle text: `jq '.["doomsday"].oracle_text' data/scryfall_cache.json` and the same for Teferi, Time Raveler; Valley
  Floodcaller; Borne Upon a Wind; Gitaxian Probe; Brainstorm; Ponder; Preordain; Silence; Noxious Revival; Thassa's Oracle.

## Items
1. **Doomsday card code** ({B}{B}{B} sorcery): "Search your library and graveyard for five cards and exile the rest. Put
   the chosen cards on top of your library in any order. You lose half your life, rounded up." Rules: the five may come
   from library and graveyard together; if fewer than five cards exist, take all; "the rest" = every other card in library
   and graveyard is exiled; order is chosen; life loss = ceil(life/2) (life <= 0 cases per the rules). Mark it Full in
   the overrides/audit once done; `cardgen export --offline` + `cardgen audit --mine` must show nothing below Full.
2. **AI: when to cast it and which pile.** Implement the guide's Piles A, B, C and the Thrasios devotion-3 variants as the
   AI's pile builder: choose from the cards actually available (library + graveyard + hand) the pile that wins with the
   mana and life it will have; prefer a lock (Silence) first where the pile allows; with Noxious Revival available, a
   backup. Cast timing: end step of the player before you when it has an instant-speed enabler (Teferi +1, Valley
   Floodcaller, Borne Upon a Wind) and Thassa's Oracle can resolve on its turn; else main phase with Pile C when it holds
   Oracle and Probe/Borne. Don't cast it when an opponent can deal the remaining life before the win, or when Line 1
   (Consultation/Pact + Oracle) is available and cheaper (guide: Doomsday is for Oracle without Consultation/Pact, or a
   flash enabler without either half). Never Ad Nauseam after Doomsday. Then **carry out the pile**: the AI's following
   draws and cantrips must follow the pile's plan (Brainstorm puts back the right two; Preordain/Ponder keep Silence;
   Probe for life). Use the plan-file sections where the loader can express it (lines, go_now, hold_up, wish entries for
   Doomsday as a tutor-like line) and card/AI code where it can't. Every card a line names must be in the list.
3. **The instant-speed enablers**: check Teferi, Time Raveler's +1 (sorceries as though they had flash until your next
   turn), Valley Floodcaller (noncreature spells as though they had flash) and Borne Upon a Wind (spells this turn as though
   they had flash; draw a card) let Doomsday and Silence be cast at instant speed in the engine, for the AI and a person.
   Fix what doesn't.
4. **Practice mode (a person)**: casting Doomsday asks the person to pick the five cards (from library and graveyard) and
   their order (top first), exiles the rest, takes the life. Use `human::choices` (pick_cards / an order question). AI and
   person code stay separate (`if crate::human::is_human(g, p)`).
5. **thrasios.toml**: add the Doomsday line(s) and anything the Tier 6 slots need for them (wish/hold_up/go_now) and the
   keep rule's Doomsday terms from the guide's keep table. Keep the file's PROVISIONAL notes accurate.

## Tests
New file `rust/crates/sim-core/tests/doomsday.rs`: the card's rules (library+graveyard search, fewer than five, exile the
rest, order, life rounding: 40 -> 20, 25 -> 12); each pile (A, B, C, and one devotion-3 variant) played out by the AI to
a Thassa's Oracle win on a scripted table; Silence stopping a response; a countered cantrip stalling the pile; Noxious
Revival recovering a countered Oracle; end-step cast with each enabler; AI declines with lethal on the table and when
Line 1 is cheaper; a person's pick and order. Run: your file, every test file seating `thrasios`, `ai_planner`,
`thrasios_ai`, `plans`, `plan_files_3`, `generic_paths`, `practice_*` probes, `play_cards`, `cards_t6_*`, `engine_t6_*`,
`ai_t6_*`, with `--no-fail-fast`. Practice probes must stay green (Doomsday will now be probed).

## Done when
- Items 1-5 done (or under NEEDS JD); tests pass; `cargo build --release --all-targets` has no warnings; audit clean.
- Reply: summary of at most 15 lines, then the full report under a line `---- FULL REPORT ----` (per item: what changed,
  rules and AI, files, tests, every shared-file change). The coordinator saves it to `rust/agents/report-doomsday.md`.

## Ground rules (these override the older rust/agents/RULES.md, which is from the Python-port era)
- Work only in your worktree (path above); `cd` into it for every command. Don't merge other branches. Don't push.
  Toolchain: `export PATH=$HOME/.cargo/bin:$PATH`; build from `rust/`.
- **Never run Python, not even a version check** (a hook refuses it). Use jq, shell tools, or Rust.
- **Memory cap on every cargo command and every binary you run, no exceptions**: `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 <cmd>`
  (cargo from `rust/`). The shell is zsh: write the prefix out in full, don't store it in a variable. Exit 137 = cap hit = a bug.
- No sims and no full suite (the coordinator does both). Run tests with the cap; don't leave anything running.
- Another agent works in a different worktree at the same time: never touch files outside yours.
- Never let an unsigned counter go below zero (release builds have overflow checks).
- `rustfmt --edition 2024 <your new files>` only; never `cargo fmt` on the workspace.
- Commit often (`git add -A && git commit -q -m "..."`, inside the worktree); **no Co-Authored-By line**. Keep
  `rust/agents/handoff-doomsday.md` current. If your context gets long (~80+ tool calls), stop at a clean point with
  everything committed and end your reply with a first line `HANDOFF`.
- Finish with everything committed and the reply as above. Then stop.
