# Task: cloud-rampfix — make Cloud's look-ahead cast ramp and tutors early (cloud only)

Read `rust/agents/T0-RULES.md` (Hard rules, Checks before you finish items 1-3) and `rust/agents/report-cloud-sequencing.md`
(the diagnosis: the look-ahead's score gives a rock ~0 and charges the card leaving hand, so it overrides the ramp
priority; Cloud's `search_bonus` in impls/cloud.rs ~1571 doesn't cover rocks). Branch cloud-mana. `-j 16`.

1. Add the fix the diagnosis proposed, **gated to the cloud deck**: in `search_bonus`, a Cast on turns <= 6 of a card
   tagged Rock / Dork / land-ramp (Lr), or Burnished Hart, Weathered Wayfarer, Stoneforge Mystic, Smothering Tithe,
   gets a bonus (a named constant `RAMP_BONUS = 6.0`). Keep it small and documented.
2. A unit test that the look-ahead now casts a rock on turn 2-3 over a 2-mana creature/Equipment in a hand that has both.
3. Same-seed identity for every other deck: build the parent commit's binary (a detached worktree under
   .claude/worktrees/, removed afterwards) and compare `commander-sim --deck <d> --pool all --profile loose --games 300
   --ai adaptive --jobs 16 --seed 7 --brief` for yshtola, seph, sephfs, sauron, kefka, sheoldred, veyran, titania,
   galadriel, alela — identical. Also one look-ahead check (`--ai lookahead --games 100`) for sauron and kefka.
4. Full `cargo test --release -j 16 --no-fail-fast`; `--all-targets` build without warnings. No win-rate runs (the lead
   runs them). Commit (no Co-Authored-By), never push. Report ≤15 lines: line 1 DONE/HANDOFF, line 2 model ID.
