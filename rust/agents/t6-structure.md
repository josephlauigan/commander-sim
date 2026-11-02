# Brief: why Tier 6 barely beats Tier 5, and why its seat 1 edge is too small

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/t6-structure` on branch
`t6-structure` (origin/main). Run every command from there. Read `CLAUDE.md`, `rust/agents/RULES.md` and
`documents/realism.md` first: memory cap, **no Python in any form** (the hook misses commands that start with
`cd ... &&`; never run Python that way either), loose profile.

## The two problems (JD, 2026-11-01: "we should have tier 6 beating tier 5 handily"; "I am worried there is a
## structural issue")

1. **Tier 6 into Tier 5:** in `--calibrate ordering` (each deck against three from the tier below), Tier 6 decks win
   only 22.8% of their games against Tier 5, below even share (25%) and below every other tier into the one under it
   (t1 31.4, t2 29.8, t3 28.1, t4 26.0, t5 26.4). Per deck: Dack 23.1, Kinnan 23.2, Rograkh/Thrasios 12.9, Sisay 19.2.
   Output: `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/overnight-stack/audit/recal/stage2-calibrate-base.txt`
   (and `stage2-calibrate-knobs/`).
2. **Tier 6 seat 1:** real cEDH (TopDeck.gg / EDHTop16, `data/realism/t6.toml`) gives seat 1 31.8% of decisive
   games (seats 27.3 / 22.1 / 18.8); the sim gives about 26 / 27 / 24 / 23, and no style-knob fit moves it
   (`documents/realism.md` Phase 1 notes and `audit/realism/fit/`; the 11/01 fit: `audit/recal/stage2-2026-11-01.md`
   in the overnight-stack worktree).

They may share a cause: in real cEDH the seat 1 edge comes from games decided early (a fast combo before later seats
act or hold interaction up), and a cEDH deck crushes a casual t5 table for the same reason.

## Phase A: diagnose without sims (do this now)

Read the code and the existing outputs; form and rank hypotheses with evidence. At least check:
- **Speed:** how early the Tier 6 decks win in the sim (win-round distribution by seat, turn of the first combo
  attempt, mana by turns 1-3: fast mana, rituals, free spells) against what the real data implies; whether mulligans
  keep fast hands; whether the AI holds combos back (as Dack's hand AI did: `audit/recal/dack-check.md`).
- **Interaction:** whether loose-profile opponents (and Tier 5's) stop Tier 6 combos far more than real tables do
  (counters per game, free counters, how often a combo attempt is stopped), and whether a stopped attempt costs the
  game; whether t5 decks' interaction or stax is modelled stronger than t6's protection.
- **Seat order and turn structure:** anything in the engine or AI that weakens going first (first player's draw,
  mulligan rules, `Gemstone Caverns`-style rules, seat-order effects in the AI's threat assessment, who is targeted).
- **Modelling gaps:** Partial / Approximate cards in the Tier 6 lists (and t5's) that matter for speed, combo or
  protection; combos from Commander Spellbook the decks hold but the engine can't run.
- **Rograkh/Thrasios at 12.9%** into t5: why one cEDH deck is that weak.

Write `audit/t6-structure/diagnosis-2026-11-01.md`: the hypotheses ranked, the evidence for each, and a **sim plan**
to confirm or refute the top ones (each run: command, games, expected cost in minutes at the speeds in the existing
outputs, what result would confirm). Then **stop and report**: the coordinator approves the sims (another agent is
using the machine now). Don't change code in phase A except throwaway diagnostic counters in your scratch folder.

## Rules

- **No sims and no `commander-sim` runs in phase A**; cargo at `-j 6` in the background if you need a build or a test.
  Wait on background work by its completion notice or an output file, never `until ! pgrep -f ...`.
- No deck list changes; no knob or default changes. Commit the diagnosis, no Co-Authored-By line, never push. Scratch
  in `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/t6-structure/`.
- Report in 15 lines or fewer, then the full report as text (the harness refuses report files from subagents).
