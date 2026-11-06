# Task: Phase C2, AI paths that don't follow the rules (JD approved, 2026-11-05)

Worktree: `/home/joseph/projects/commander-sim/.claude/worktrees/ai-rules` (branch ai-rules, from origin/main 336b44f5:
the rewire is merged). Task name: `ai-rules`. **No sims.**

**JD's rule: no gap may be left quietly.** Anything you can't make match the rules: commit what you have, list it at the
top of your reply under `NEEDS JD` (exact gap, effect in games, what it would take), and carry on with the rest.

## Background
These AI-side mismatches were found earlier and left as they were (`rust/agents/report-opponent-fixes.md`, "Would change
the AI"; `rust/agents/report-model-gaps-b1.md` line ~50; the model-gaps A4/B1/B3 reports). The person's versions already
follow the Oracle text; make the AI follow it too. Check every card against
`jq '.["<lowercased name>"].oracle_text' data/scryfall_cache.json`.

## Items
1. **Council's Judgment** (AI vote): each voter (the caster and every opponent) votes for any nonland permanent the caster
   doesn't control; each permanent with the most votes or tied for most is exiled. Today non-caster voters can't vote for
   their own permanents and the vote is only counted. Make the AI voters vote as real players would: an opponent votes for
   the permanent it most wants gone among the legal ones (it may vote for its own only if that's what it wants, which is
   rare: never its best), the caster for its best target; ties exile all tied.
2. **Liliana of the Veil -6** (AI): "Separate all permanents target player controls into two piles. That player sacrifices
   all permanents in the pile of their choice." Piles must include lands (all permanents). The AI separates to minimise
   what the opponent keeps; the chooser picks the pile it would rather keep.
3. **Liliana, Dreadhorde General -9** (AI): "Each opponent chooses a permanent they control of each permanent type and
   sacrifices the rest." One permanent may cover several types (an artifact creature counts for both). The AI chooser keeps
   the set that keeps most value.
4. **Chain of Vapor's copy chain**: "Return target nonland permanent to its owner's hand. Then that permanent's controller
   may sacrifice a land of their choice. If the player does, they may copy this spell and may choose a new target for that
   copy." The copy's new target's controller must be offered a further copy (the chain continues), for the AI and a person.
   The AI's decision to copy: when it helps that player (bouncing an opponent's key permanent, or saving its own combo with
   a re-cast); keep the existing rule's spirit (lands to spare, a worthwhile target). This card is in JD's new Tymna/Thrasios
   list (`decklists/JD/tymna-thrasios-doomsday.md`): check how its pilot guide uses it (as protection and as an answer).
5. **Maelstrom of the Spirit Dragon** ({4}, {T}, sacrifice: search for a Dragon card, reveal it, put it into your hand,
   then shuffle): the AI never uses it (pool decks only now). Give the AI a use: when it has spare mana at an opponent's end
   step (or its own main phase with nothing better), a Dragon in the library worth fetching, and a land to spare.
6. **Loran of the Third Path's {T}: you and target opponent each draw a card**: the AI uses it only at end of turn. Check
   what its option hook does; let it use the draw when it helps (an opponent with the fewest cards / least able to use one,
   and an untapped Loran it doesn't need to block), at the best timing (an opponent's end step is usually right: say why in
   the report if you keep it there).
7. **Ramosian Revivalist** (AI): "{6}, {T}: Return target Rebel permanent card with mana value 5 or less from your graveyard
   to the battlefield." The AI picks its target at resolution; it must pick it on activation (rule 602.2b), and the ability
   does nothing if that card has left the graveyard.
8. **Forensic Gadgeteer** (pool AI): "Activated abilities of artifacts you control cost {1} less to activate. This effect
   can't reduce the mana in that cost to less than one mana." The AI's Clue path applies it; its Food (`common::eat_food`)
   and Blood (`sheoldred::blood_rummage`) paths don't. Make every AI token activation (and any other artifact activation
   priced by the AI) use the same reduction (`t45_urza_yawgmoth::artifact_cost`).

## Tests
New file `rust/crates/sim-core/tests/ai_rules_c2.rs`: one or more tests per item showing the AI now follows the Oracle
(e.g. Liliana's piles hold lands; an opponent's Council's Judgment vote for its own permanent is legal; a Chain of Vapor
copy is offered a second copy; Maelstrom used by the AI; Revivalist's target fixed on activation, fizzles if removed; Food
and Blood at Gadgeteer's discount). Also run the existing tests of each card (grep the names in `rust/crates/sim-core/tests`),
`practice_opponent_*`, `practice_*_probe`, `ai`, `ai_flow`, `generic_paths`, with `--no-fail-fast`.

## Done when
- Items 1-8 done (or under NEEDS JD); tests pass; `cargo build --release --all-targets` has no warnings.
- Reply: summary of at most 15 lines, then the full report under a line `---- FULL REPORT ----` (per item: rules, AI change,
  files, tests; every shared-file change; which decks/tiers each change can affect). The coordinator saves it to
  `rust/agents/report-ai-rules.md`.

## Ground rules (these override the older rust/agents/RULES.md, which is from the Python-port era)
- Work only in your worktree (path above); `cd` into it for every command. Don't merge other branches. Don't push.
  Toolchain: `export PATH=$HOME/.cargo/bin:$PATH`; build from `rust/`.
- **Never run Python, not even a version check** (a hook refuses it). Use jq, shell tools, or Rust.
- **Memory cap on every cargo command and every binary you run, no exceptions**: `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 <cmd>`
  (cargo from `rust/`). The shell is zsh: write the prefix out in full, don't store it in a variable. Exit 137 = cap hit = a bug.
- No sims and no full suite (the coordinator does both). Don't leave anything running.
- Other agents work in other worktrees at the same time: never touch files outside yours.
- Never let an unsigned counter go below zero (release builds have overflow checks).
- AI and person code stay separate (`if crate::human::is_human(g, p)`); don't change the person paths unless a rule says so.
- `rustfmt --edition 2024 <your new files>` only; never `cargo fmt` on the workspace.
- Commit often (`git add -A && git commit -q -m "..."`, inside the worktree); **no Co-Authored-By line**. Keep
  `rust/agents/handoff-ai-rules.md` current. If your context gets long (~80+ tool calls), stop at a clean point with
  everything committed and end your reply with a first line `HANDOFF`.
- Finish with everything committed and the reply as above. Then stop.
