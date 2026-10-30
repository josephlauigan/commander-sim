# Brief: Phase 4, PR 1: deck plans as data

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/phase4-plans` on branch
`phase4-plans`, stacked on `phase3-exile` (Phase 3 is done; its PRs #166-#170 are not merged yet). Run every command
from there. Read `CLAUDE.md` first: memory cap, no Python, loose profile, opponent tiers. You are the only agent:
`-j 22` for cargo, `--jobs 22` for sims, one sim at a time, in the background; check rc=0 on every run you cite.

## Background

`documents/realism.md`, Phase 4 ("Generic AI"; gate: **each deck level with its hand code in a paired test**). JD's
plan document (claude.ai/artifact/Fa1usjF6hHZiSUBXwLzGNJ, "AI" section) says, point 1:

> Deck intent as data, not code. Each deck gets a short plan file (TOML): win lines: the combos (from Commander
> Spellbook) and the non-combo plans ("wide combat", "drain", "commander damage"), each with its key pieces; tutor
> priorities derived from the win lines, so wish lists can't go stale when the list changes; cards to hold
> (protection, counters) and what they protect; mulligan rules: lands, fixing for the deck's colours, and the plan's
> pieces. The generic policy reads it. A new deck needs a plan file and no Rust.

and, on deletion: "the per-deck priority functions, wish lists and plan files, deck by deck, each replacement checked
with a paired test and the scorecard. Keep a deck's hand code only where its paired test shows the generic policy is
worse and the gap can't be closed in its plan file."

Today: `ai/plans.rs` (`DeckConfig`: style, key_cards, wish fn, prio_fn, cmd_prio, cmd_turn, aura, top_pref, for 7
pool decks), `ai/decks.rs`, `ai/seph.rs`, `ai/veyran.rs`, `ai/pool.rs`, the 8 cEDH plan files `impls/t6_plan_*.rs`
(+ `t6_plans.rs`), and wish lists / priority functions spread through `cardcode.rs` and `impls/`. Combo data per deck
is in `data/spellbook/<deck>.json` (PR #162) and the catalogue in `impls/combos.rs`.

Sims are allowed again (JD: none until Phase 4). JD recalibrates every tier once, after Phase 4: don't retune.

## Work

1. **Design** (`audit/phase4/plans-design.md`): the plan file schema (`data/plans/<deck-key>.toml`): win lines
   (combo lines by catalogue name or Spellbook id, non-combo plans with their key pieces), how tutor priority is
   derived from win lines (missing piece of the closest line first, then engines, then ...), holds (card, what it
   protects, when to keep mana for it), mulligan rules, the static knobs (style, commander priority and earliest
   turn, aura host, top-of-library preference, key-card priorities). A loader in sim-core with validation (every
   card named must be in the deck: a test checks every plan file against `data/decks.json`, so a list change that
   strands a plan line fails `cargo test`). How the generic policy reads it, and an A/B switch (`AI_PLAN_DATA=0` /
   `1`, default to be decided per deck after its paired test).
2. **Stage A, results identical:** a plan file for every deck that has a `DeckConfig` entry, written so the
   generic policy with `AI_PLAN_DATA=1` reproduces today's choices exactly (the static knobs and key cards move to
   data; the hand `wish`/`prio_fn` still run). Check: `--realism --pool tN --games 300 --jobs 22 --quiet` for the
   tiers of those decks, byte-identical between the base binary and the branch.
3. **Stage B, derived wish lists and holds:** for those 7 decks, replace the hand `wish` functions by the derived
   tutor priority from their win lines (and the hand `prio_fn` where the plan file can express it). Behind
   `AI_PLAN_DATA=1`. **Paired test per deck:** `--realism --pool <its tier> --games 2000 --jobs 22` with
   `AI_PLAN_DATA=0` and `=1`, same seeds: report the deck's share both ways with its 95% CI, and the tier's other
   decks. "Level" = the difference inside the CI, or the plan better. Where level or better, delete that deck's hand
   code and default to the plan; where worse, keep the hand code, find why (trace a few games), and say what the plan
   file can't express yet.
4. **Then, if time allows, the same for the 8 cEDH decks** (`impls/t6_plan_*.rs`), one deck at a time, each with
   its paired test on t6. JD's decks (seph, veyran, sauron and the others in `ai/decks.rs`) come in a later PR:
   leave them.

## Checks

- `cargo test --release --no-fail-fast -j 22` passes; `cargo build --release --all-targets -j 22` 0 warnings;
  practice tests pass; `cardgen validate` passes; the plan-file test passes.
- Every paired run: rc=0, both arms on the same seeds, the numbers in the report.

## Rules

- No deck list changes. No recalibration, no style refit (JD does that once after Phase 4; leave style values as
  they are, moved to data).
- Don't seat `omnath` or `godo` in new tests. Format only new files (`rustfmt --edition 2024 <file>`), lines up to
  120.
- Commit at each stage and after each deck, no Co-Authored-By line, never push. Scratch files in
  `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/phase4-plans/`.
- Keep `audit/phase4/plans-handoff.md` current (which decks are done, their paired numbers), so the next agent can
  continue.
- Report in 15 lines or fewer, then the full report as text in your reply (the harness refuses report files from
  subagents): the schema, each deck's paired result, what was deleted, what the plan files can't express yet.
