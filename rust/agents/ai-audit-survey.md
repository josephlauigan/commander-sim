# Brief: deck AI audit, phase 1 (survey, read-only)

JD wants every card in each of his decks implemented correctly AND valued by that deck's AI for that deck's strategy
(not as a generic value card). Phase 1 finds the problems; phase 2 (later, separate agents) fixes them. You change
NO code and NO decklists in this phase.

Work in `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/ai-audit` (shared, read-only for code). The
binaries are built: `rust/target/release/commander-sim` and `rust/target/release/cardgen`. Do not run cargo.
No Python, ever. No Co-Authored-By lines anywhere.

## CPU (strict)

The machine is shared: run every sim with `--jobs 2` under `nice -n 19` and
`systemd-run --user --scope -q -p MemoryMax=8G -p MemorySwapMax=0`. One sim at a time. Use `--ai adaptive` (fast)
unless a check needs the look-ahead AI; keep `--games` small (300 per tier is enough for a card report).

## For each of your decks

1. Read the deck doc `decklists/JD/<file>.md` (the strategy, combos, key cards). File per key in `data/my_decks.toml`.
2. Card data: `rust/target/release/commander-sim --deck <key> --cards` (tags, source, unmodeled text) and
   `rust/target/release/cardgen audit --deck <key>` (model status per card).
3. The deck's AI: `rust/crates/sim-core/src/ai/decks.rs` `deck_prio` names the priority function for each key
   (e.g. `seph_prio` in `ai/seph.rs`, `sephfs_prio` in `impls/sephfs.rs`, `kraum_prio` in `impls/tymna_partners.rs`).
   Also look for the deck's plan config (`ai/plans.rs`, `plans::config`), tutor wish lists (grep the key or
   "wish"), reanimation/target valuation (`seph_bval` for reanimators), and hold/response logic. Grep, don't read
   whole files.
4. Card report: `--deck <key> --pool <one of its tiers> --analyze --ai adaptive --games 300 --jobs 2` (tiers in
   `data/my_decks.toml`): cards never cast, often stuck in hand, lowest win rate when cast.
5. For EVERY nonland card (and lands with abilities), classify:
   - **model**: Full / Approximate / Partial / Unmodeled, and anything in its Oracle text that matters for THIS deck
     and is missing or wrong.
   - **valuation**: `deck` (the deck's AI handles it by name or by a tag its plan uses), `generic-ok` (generic logic,
     and that is fine for this card here), or `generic-WRONG` (the deck's plan needs something the generic logic
     doesn't do: e.g. a reanimator that doesn't value a token-maker as a target, a tutor that can't find the combo
     piece, a combo piece cast early as a body, a card held forever or cast at the wrong time).
   - **evidence**: the card-report numbers or code line that shows it.

## Output (write early, write often)

Create `audit/ai-audit/<key>.md` as soon as you start a deck (a header first), then add rows as you go. Format:

```
# <Deck> (<key>) — AI audit, phase 1
Strategy in one paragraph (from the doc). Priority function: <fn, file>. Plan config: <...>.
## Problems (ranked by likely impact on win rate)
1. <card>: <what's wrong> — <evidence> — <suggested fix, one line>
## All cards
| Card | Model | Valuation | Evidence / note |
```

Only list a problem when you have evidence (code or numbers). Say "unknown" rather than guess.
Do the decks one after the other; finish the first file before starting the second.

## Report

At most 10 lines: per deck, the number of problems found and the top three. Details stay in the audit files.
