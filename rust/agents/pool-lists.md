# Task: review and update the pool lists flagged in Phase C (JD approved, 2026-11-06)

Worktree: `/home/joseph/projects/commander-sim/.claude/worktrees/pool-lists` (branch pool-lists, from main 336b44f5).
Task name: `pool-lists`. Read `audit/realism/c1b-2026-11-06/REPORT.md` ("What is left" 3-4), `audit/realism/c3-2026-11-06/REPORT.md`
("List-level findings"), `decklists/pool/README.md` and `decklists/pool/TARGETS-t6.md` first. These are opponent pool
decks; JD's own decks (`decklists/JD/`, `data/my_decks.toml`) are not touched.

## The findings
- The six Tier 6 lists (`decklists/pool/t6-cedh/`) cast cheap cantrips ~0.02 a turn: none plays Brainstorm, Ponder,
  Preordain or Opt. Tier 6 decks miss a land drop with no land in hand 36% of games by turn 3 and 52% by turn 4; most
  draw 1.0-1.3 cards a turn through turn 4. Median win round 8 against a real 4-6.
- Tier 3 Atraxa (`decklists/pool/t3-high-b3/atraxa-infect.md`) is pure infect: 23.8% of its wins by poison against 9.3%
  for typical Playgroup builds; its four-colour mana leaves a full hand and nothing cast in rounds 2-4 in many traces.

## Do
1. **Tier 6: match each list to its source.** Each list names its source tournament decklist (EDHTop16 link). Fetch it
   (WebFetch; if you can't reach it, try the Moxfield link the page gives; if neither works, `NEEDS JD` for that list) and
   diff card by card, lands included. The lists are meant to be the real tournament lists: where ours differs from the
   source (a missing cantrip, a missing land, a substitution made at import), make ours match, unless a card was
   swapped on purpose (look for a note in the file, `CHANGELOG.md`, git log of the file): then keep it and say why. Don't
   add cards the source doesn't play. If a list matches its source and still plays no cantrips, say so: that is the real
   list.
2. **Tier 3 Atraxa: move it toward a typical mixed build** (infect as one plan, not all of it), in the style of the other
   Tier 3 lists: still Bracket 3 (at most 3 Game Changers), a healthier curve and mana (count sources per colour), and
   proliferate / counters value as the other plan. Cite where the build comes from (EDHREC average Atraxa, Praetors'
   Voice deck or a named public list); keep the file's structure and update its text (Role, At the table, Lands count).
3. **Every card you add** must be: `legalities.commander == "legal"` (`data/scryfall_cache.json`, or
   `https://api.scryfall.com/cards/named?exact=<name>`), inside the deck's colour identity, and **Full** in
   `cardgen audit` (find the command in `rust/crates/cardgen`; check the card is Full after your change). A card below
   Full is modelled (card code with a test) or not used: never added below Full without listing it under `NEEDS JD`.
4. Regenerate whatever the lists export to (`data/decks.json` etc.; find the cardgen export command and run it under the
   memory cap) and fix the plan files (`data/plans/<deck>.toml`) that name cards you cut. Update `decklists/pool/CHANGELOG.md`.
5. Report: per list, the cards in/out with the reason (source diff or build citation), and the land / cantrip counts
   before and after.

## Tests
Run the test files that check deck contents or plans (`plans`, `plan_files_3`, `generic_paths`, `cards_t6_*`,
`engine_t6_*`, `ai_t6_*`, any `*atraxa*` / `cards_t3*` files, and whatever `grep -l` finds for the deck keys you changed)
and `cardgen audit` for the changed decks (every card Full).

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
