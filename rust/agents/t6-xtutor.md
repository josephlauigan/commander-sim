# Brief: X tutors fire toward a combo line (Tier 6 fix 1)

You work in `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/t6-xtutor` on branch `t6-xtutor`
(origin/main). Run every command from there. Read the common rules first:
`/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/common-rules.md`.

## Why (JD approved 2026-11-02)

Tier 6 wins only about 23% of its games against Tier 5; JD wants it "beating tier 5 handily". The diagnosis
(`audit/t6-structure/diagnosis-2026-11-01.md`, read H3, S6 and "4. What to change next", item 1) found that the
ready share (29% of player-games ever have a line ready) is the binding number, and that many players sit one tutor
away and pass. Rograkh/Thrasios held Finale of Devastation and Green Sun's Zenith 392 times in 1500 games; S6 shows
`PLANNER_PROGRESS` can't reach them because X tutors and pile tutors are not modelled as tutors toward a line (the
Rograkh/Thrasios cell was byte-identical).

## What to build

1. **Cast an X tutor toward a line** when the mana exists to cast it at the X the target needs and then still reach
   the line (this turn, or with the line held for next turn if the deck's plan allows that today). Cards: Finale of
   Devastation and Green Sun's Zenith (sorceries: X = the creature's mana value; Finale puts it onto the
   battlefield), Chord of Calling (an **instant** with convoke; X = mana value, onto the battlefield). Check each
   card's Oracle text in `data/scryfall_cache.json` and how its card code resolves today, and find any other X or
   pile tutor in the t5/t6 lists the same gap applies to (list them; include them only if the same code covers
   them cleanly).
2. Prefer a **generic** change (in `ai/planner.rs` or wherever the planner's tutor-toward-line logic lives, so every
   deck with these cards benefits) over a Tier-6-only rule. If only a t6 rule works, say why.
3. **Review Kinnan's "no counter and blue up: don't go" rule** (`kinnan-bonder-mana-combo.toml`; diagnosis: 17 ready
   games waited, 8 never went). Change it only if the code clearly holds back a line a real player would go for;
   either way, report what you found.
4. Put the new behaviour behind an env switch that defaults on (name it `AI_XTUTOR`, `AI_XTUTOR=0` = today's
   behaviour, read like the existing `AI_SEARCH_B` / `AI_RULES` switches), so the paired test can run off and on
   from one binary. With it off the binary must play byte-identically to origin/main.
5. Tests: a planner-level test per card that it is cast at the right X when it completes a line, and is not cast
   when the mana for the line afterwards isn't there.

You may run up to 20 traced games (`--jobs 4`) to see the tutor cast in a real Rograkh/Thrasios game; no more.

## The queue (write it; don't run it)

`/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/t6-xtutor/queue.sh`: bash,
resumable (skip a run whose output exists), each run under the memory cap with `--jobs 22`, one line per run and a
final `ALL DONE` line in `queue.log` in that folder. It takes the branch binary as its argument and builds nothing.
Outputs into `audit/t6-xtutor/` in the worktree. Runs (seeds 500000-501999, 2000 games, look-ahead, loose), each
with `AI_XTUTOR=0` and `AI_XTUTOR=1`:
- identity first: `--realism --pool t6` and `--realism --pool t5`, 300 games, `AI_XTUTOR=0` against a main binary
  passed as a second argument (`cmp` after `grep -v 's total)'`; write IDENTICAL / DIFFERS);
- `--calibrate ordering --pool t6` (Tier 6 into Tier 5), and `--realism --pool t6` (the scorecard);
- for each other pool tier whose decks hold one of the cards, that tier's `--realism` (paired, level-or-better
  check); for each of JD's decks that holds one (check `data/decks.json`), `--deck KEY --pool tN` on its own tiers
  only.
Put an estimated time per run in a comment (use the speeds in `audit/recal/` and `audit/t6-structure/` outputs).
