# Brief: Phase 4, PR 6: rule checks in the policy

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/phase4-rules` on branch
`phase4-rules`, from origin/main. Run every command from there. Read `CLAUDE.md` and `rust/agents/RULES.md` first:
memory cap, no Python. **Another agent works in parallel** (worktree `phase4-hand-off`, putting each deck's hand AI
behind a switch): use `-j 12` for cargo, in the background.

## No sims (JD, 2026-10-31)

**Run no `commander-sim` of any kind**, not even a short byte-identity or timing check, and no Rust example that
plays games. Checks are cargo tests only. So the change ships **behind a switch that defaults off**, and the paired
tests are written down for the recalibration after Phase 4.

Waiting on background work: wait for the task's completion notice, or on an output file. Never wait with
`until ! pgrep -f ...` (it matches its own command line and never ends), and don't leave a wait loop running when its
job is cancelled.

## The goal (JD's plan, AI point 6)

"A class of mistakes is better stopped by a generic guard than by fixing one card at a time: never cast a spell whose
required follow-up (Pact's upkeep, a sacrifice cost) can't be paid; never discard or mill a card a tutor just found;
never start a combo whose missing piece no tutor in hand can find. Each is one test that covers every card with the
pattern."

## Work

1. **Survey** (`audit/phase4/rules-design.md`, commit before code): where each of the three decisions is made today
   (casting: the policy's cast options and `engine/cast.rs`; discard to hand size, discard costs and self-mill; combo
   starts: the combo catalogue and plan lines), and which cards have each pattern, read from the card data (not a hand
   list): the Pacts (upkeep payment), spells and abilities with an additional sacrifice or discard cost, delayed
   costs; tutors (what they find and where it goes); combo lines with a missing piece.
2. **Design** each guard as one generic check, in a new module (e.g. `ai/rules.rs`), called from as few places as
   possible. Keep edits to `ai/brain.rs` to small hooks (the other agent edits that file too):
   - **Follow-up:** a spell or ability is not offered when its required later payment (Pact's upkeep cost on the next
     upkeep, counting the mana p will have) or its additional cost (a sacrifice, a discard) can't be paid. Say how the
     next upkeep's mana is estimated.
   - **Tutored card:** a card a tutor put in hand (or on top of the library) this turn or last is not discarded (hand
     size, discard costs, wheels where p chooses) or milled by p's own choice while another choice exists. Mark the
     card when the tutor resolves.
   - **Combo start:** a combo line is not started when a piece is missing and no tutor p holds (and can cast this turn)
     can find it.
3. **Switch** `AI_RULES`: unset or `0` = today's behaviour (default, since nothing can be measured now), `1` = all
   three guards for every deck, or a list of guard names (`followup`, `tutored`, `combo`). With it unset the code paths
   must be exactly today's: guard calls return early before reading or changing any game state or random numbers.
4. **Unit tests** on hand-built positions, with the switch forced on in the test (a thread-local override, as
   `ai/ismcts.rs` does with `FORCE`): each guard stops the mistake (Pact of Negation with too little mana for its
   upkeep; a sacrifice-cost spell with nothing to sacrifice; discarding to hand size keeps the tutored card; a combo
   with an untutorable missing piece is not started, and is started when a tutor in hand can find it). Also one test
   per guard that with the switch off, the old choice is made.
5. **For the recalibration**, list in the design doc the paired tests to run (the half-tier method in
   `audit/phase4/values-handoff.md`, plus JD's decks on their own tiers) and how to read them.

## Checks

- `cargo test --release --no-fail-fast -j 12` passes; `cargo build --release --all-targets -j 12` 0 warnings;
  `cardgen validate` passes.
- Don't seat `omnath` or `godo` (the pool deck) in new tests.

## Rules

- Stay out of `ai/search.rs`, `ai/ismcts.rs`, `ai/learned.rs`, `ai/values.rs`, `data/eval/`. No deck list changes,
  no recalibration, no style values.
- Format only new files (`rustfmt --edition 2024 <file>`), lines up to 120. Commit often, no Co-Authored-By line,
  never push. Scratch files in
  `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/phase4-rules/`.
- Keep `audit/phase4/rules-handoff.md` current.
- Report in 15 lines or fewer, then the full report as text in your reply (the harness refuses report files from
  subagents): the survey (cards per pattern), the design, the switch, the tests, and the recalibration list.
