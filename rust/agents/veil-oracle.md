# Task: Veil of Summer as its Oracle text (JD approved, 2026-11-06)

Worktree: `/home/joseph/projects/commander-sim/.claude/worktrees/veil-oracle` (branch veil-oracle, from main 336b44f5).
Task name: `veil-oracle`.

## The gap
Veil of Summer's Oracle text: "Draw a card if an opponent has cast a blue or black spell this turn. Spells you control
can't be countered this turn. You and permanents you control gain hexproof from blue and from black until end of turn."
The engine (`impls/rules.rs` `veil_response`, called from `cardcode.rs:285`) only answers one blue/black counter with
the Veil: it is cast in response, that one counter fails, and it always draws. Real play: once Veil resolves, *every*
counter that turn fails (blue, black, red blasts' counter mode, colourless, a second counter after the first), it draws
only if an opponent cast a blue or black spell this turn, and you and your permanents have hexproof from blue and from
black until end of turn (blue/black targeted removal and bounce can't target them). Mindbreak Trap exiles, it doesn't
counter: Veil does NOT stop it (keep that).

## Do
1. Model the full Oracle text: a per-player "spells can't be countered this turn" state (cleared at end of turn) that
   every counter path respects (find them all: counterspells, `cast_counter` in engine/stack.rs, the DSL's counter
   effect, the look-ahead's copies, "counter target ability" cards stay as they are since Veil covers spells only);
   the conditional draw; hexproof from blue and from black for the player and their permanents until end of turn
   (find how the engine models hexproof / protection from a colour for targeting, reuse it).
2. Veil can be cast in response to any counter (not only blue/black ones) and pre-emptively (e.g. before a key spell,
   if the AI already does that keep it). Keep the existing `CTHRESH` rule (JD's own decks don't answer with Veil in the
   generic code) unless something below needs it changed: say so.
3. The person path (`human/`) gets the same card behaviour (a person casts their own Veil; the state applies the same).
4. Check every deck that plays Veil of Summer (`jq` on `data/decks.json`), JD's `thrasios` included (its Doomsday pile
   treats Veil held with {G} as a lock: `impls/doomsday.rs` is NOT on this branch, it is in PR #272/#275; note in your
   report what that code should read once both are merged, don't port it here).
5. `cardgen audit` for Veil must stay/become Full.

## Tests
New `rust/crates/sim-core/tests/veil_oracle.rs`: two counters in a turn both fail after one Veil; a red blast's counter
mode fails; Mindbreak Trap still exiles; draw only after an opponent's blue/black spell; a blue removal spell can't
target the Veil player's creature this turn, a red one can; the effect ends at end of turn. Then run `ai`, `ai_flow`,
`engine_*` test files that touch counters, the card test files of decks that play Veil, and JD's decks' card test files
(`cards_sauron*`, `cards_sheoldred*`, `cards_cloud*`, `cards_kefka*`, `cards_yshtola`, `cards_alela`, `cards_galadriel`,
`cards_thanos`, `thrasios*` if present).

## Ground rules (these override the older rust/agents/RULES.md, which is from the Python-port era)
- **JD's rule: no shortcoming is accepted quietly.** Anything you can't make right (a card below Full, a case you skip, a
  caveat): commit what you have, list it at the top of your reply under `NEEDS JD`, and carry on.
- Work only in your worktree; `cd` into it for every command. Don't merge other branches. Don't push.
  Toolchain: `export PATH=$HOME/.cargo/bin:$PATH`; build from `rust/`.
- **Never run Python, not even a version check** (a hook refuses it). Use jq, shell tools, cardgen or Rust.
- **Never use `git stash`** in any form (the stash is shared with other sessions). Use a WIP commit.
- **Memory cap on every cargo command and every binary you run, no exceptions**:
  `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 <cmd>` (zsh: write the prefix out in full).
  Exit 137 = cap hit = a bug.
- No sims (no `--realism`, `--pool`, `--calibrate` or multi-game runs) and no full `cargo test` suite: the coordinator
  runs those. Single `--trace N` games are fine. Leave nothing running when you stop.
- Loose profile only. Other agents work in other worktrees at the same time: never touch files outside yours.
- Never let an unsigned counter go below zero. AI and person code stay separate (`if crate::human::is_human(g, p)`).
- `rustfmt --edition 2024 <the files you changed>` only; never `cargo fmt` on the workspace.
- Commit often inside the worktree (`git commit`, **no Co-Authored-By line**). Keep `rust/agents/handoff-<task>.md`
  current. If your context gets long (~80+ tool calls), stop at a clean point, everything committed, reply's first line
  `HANDOFF`.
- Write the full report to `rust/agents/report-<task>.md` and commit it (if the harness refuses the write, put the full
  report in your reply after a `---- FULL REPORT ----` line). Reply in at most 15 lines.
- `cargo build --release --all-targets` with no warnings before you finish; run the test files the brief names with
  `cargo test --release --no-fail-fast --test <name> ...`.
