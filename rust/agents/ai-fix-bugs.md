# Brief: AI foundations step 1, fix the decision bugs (one commit each, each measured)

Work in the worktree `.claude/worktrees/ai-foundations` (branch `ai-foundations`). It is your whole world. Read
`CLAUDE.md` first. Don't use `rust/agents/RULES.md`.

## Why

The AI foundations audit (`audit/ai-foundations/REPORT.md`, with `architecture-review.md` and `loss-accounting.md`)
found that the decision layer loses turns to plain bugs, as well as lacking a turn plan. JD approved fixing the bugs
first, then a line planner (step 2), then a Tier 6 recalibration (step 3). This brief is step 1 only.

Fix each bug the way a competent player would play, not with another priority patch. One commit per bug. Measure
each commit in the goldfish.

## The bugs (the REPORT's table; file:line as of commit 157e4445)

1. **G: the cleanup discard** (`engine/turn.rs` ~796). The AI discards the highest-CMC card, or the first land when
   it holds two or more, and ignores `ai::discard_keep`. It threw away Thassa's Oracle 124 times in 2,000 scion
   goldfish games, and loses a combo piece in 18-33% of games for the Oracle decks. `discard_keep` keeps too few
   pieces to be the fix on its own (`loss-accounting.md`). Make the discard keep:
   - the pieces of the player's own catalogued lines (`combos` / `lines_of`);
   - protection, up to what the deck can use;
   - the cards it can cast soonest.

   Discard spare lands first once enough are held, then the lowest-value card. Keep Seph's bomb rule and the
   Tergrid rule. Say in the commit which cards it now keeps, and why.
2. **C: rituals.**
   - Cabal Ritual, Rite of Flame and Desperate Ritual have `prio = zero_prio` (`impls/t5.rs:978`,
     `impls/t6_mana.rs:564,585`), so they are never cast outside catalogued lines.
   - Dark Ritual scores 1.5, below "stop (hold mana)".
   - Nothing counts a ritual in hand as mana.

   A ritual is worth casting when the mana it nets lets you cast something this turn that you otherwise couldn't: a
   line, a tutor, a commander, or a good spell. Give it that value, the enabled play's value less a little, and
   nothing otherwise. Make the combo affordability check (`combos::affordable` or its equivalent) count the net mana
   of rituals in hand, without counting the same ritual twice or a ritual that is itself a piece of the line. Keep
   any reason `zero_prio` existed (read the git log for it) and say how it's handled.
3. **D: Tier 6 must-do plays before a ready line** (`ai/t6_table.rs` `must_option`, ~261-307). It forces a cheap
   rock or dork on turns 1-4, or a high plan priority such as Thrasios at 93, before the `combo:` option, which it
   ignores except for Sisay. That can make a ready line unaffordable. Fix both:
   - a ready, affordable line comes first;
   - a forced play that would make a ready line unaffordable is skipped.
4. **E: win now against win next turn** (`ai/search.rs` `evaluate` ~209, and the cap in `choose_inner` ~385). Every
   win scores 100, so since the cap (d10bbb05) a higher-utility play whose line still wins next turn ties with
   winning now and takes the tie. Score a win by when it happens: 100 minus a few points per own turn of the deck
   after the decision. Record the turn the playout's game ended. Keep the cap at the best possible score, so the hold
   bonus still can't beat a win. Keep `search_hold::a_hold_bonus_never_beats_a_sure_win` passing, and add a test
   where winning this turn beats a play that wins next turn.
5. **B: tutor targets** (`impls/combos.rs` `missing_pieces` ~720, `engine/tutors.rs` ~401-417, `ai/pool.rs`
   ~233-234). Three faults:
   - A line that is complete in hand but lacks only mana is skipped. A tutor should then fetch the mana that makes it
     affordable (fast mana, a ritual, a land), if that wins this turn or next.
   - A card already put on top of the library (Imperial Seal, Vampiric Tutor, Mystical Tutor, Enlightened Tutor) is
     still counted as missing, so a second tutor fetches it again. Record known top cards per player. The record
     ends when the library is shuffled or the card is drawn.
   - Enlightened Tutor is tagged `tut=any`, so it's scored at 72 for finding Oracle, but it really finds an artifact
     or enchantment and puts it on top. Score each tutor by what it can actually find, using its real restriction.
6. **I: `determinize` shuffles away the deck's own known top card** (`ai/search.rs` ~101). Keep known top cards, from
   the record in 5, in place when the deck's own library is shuffled for a playout.

Not in scope: the holding values in goldfish games (H; measured small), the line planner (step 2), the Tier 6 table
(step 3), and list changes.

## Measure

**Before you start**, build once and save the binary as the "before". Run:
- `--goldfish t6,scion --games 2000 --seed 500000` (look-ahead), with the loss-accounting section;
- `--realism --pool tN --games 4000` for t0-t6 (t5 now runs: the Treasure memory fix is cherry-picked);
- `--all-decks --pool all --games 1000`.

**After each bug's commit:** the goldfish run only. Record the median kill, by T3, the buckets, discards and pieces
lost, in a table with one row per commit.

**After the last commit:** the realism t0-t6 and all-decks runs again, with before and after side by side. Flag every
realism row that changes PASS/FAIL or moves more than 1.5 points, and every JD deck whose win rate moves outside its
confidence interval. Most of this code is shared, so lower tiers and JD's decks will move. Explain each large move:
trace a few games where needed.

Write everything to `audit/ai-foundations/step1-fixes.md`, and put the raw output under
`audit/ai-foundations/raw/step1/`.

## Rules

- **Commands.** Run every build, test and sim from `rust/`, with `run_in_background`, under
  `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 ...`. For a run longer than about 10 minutes,
  launch a detached unit (`systemd-run --user --unit=step1-<name> -p MemoryMax=24G -p MemorySwapMax=0 <script>`) and
  poll its output files. Never use one blocking command longer than 10 minutes.
- **Threads.** Pass `--jobs 16` to every commander-sim run and `-j 16` to cargo. Run one sim or cargo command at a
  time.
- **No Python, ever**, not even inside a longer command.
- **Shared files.** Mark each change `// --- ai-fix: <bug letter>`. Comments explain why, briefly, in the
  surrounding style.
- **Formatting.** Format only the files you write (`rustfmt --edition 2024 <file>`), never `cargo fmt`. Lines go up
  to 120 characters.
- **Commits.** One per bug, with the bug letter in the subject. No Co-Authored-By line. Never push. Keep
  `rust/agents/handoff-ai-fix-bugs.md` up to date (done, next, gotchas) and commit it with your work.
- **Tests.** Every fix gets a test of the behaviour (a game state where the old code chose wrong). Tests that
  asserted the old wrong behaviour may be changed; say which ones and why.
- **Before you finish:** `cargo test --release --no-fail-fast` passes, `cargo build --release --all-targets` has no
  warnings, and no process of yours is still running (check with `pgrep -af 'commander-sim|cargo'`).
- **Decisions.** JD may be away. Make the conservative call and record it, with the alternative, in a "Decisions"
  list in `step1-fixes.md`.
- **Report** in your final reply, 15 lines or fewer: the per-commit goldfish table in short, the realism and
  all-decks moves, and anything JD must decide.
