# Task: cloud-aifix3 — fix the four Cloud AI faults from the 10/15 audit

Read `rust/agents/T0-RULES.md` (Hard rules, Checks before you finish items 1-3, Keeping context small apply), then
`rust/agents/report-cloud-audit-1015.md` (section 2: F1, F2, F4, F8, with file:line). Your branch was made from the
audit branch (origin/main + the report). Handoff `rust/agents/handoff-cloud-aifix3.md`, full report
`rust/agents/report-cloud-aifix3.md`. You may use `-j 12` builds and `--jobs 12` sims.

## Fix, one commit each, each with a test
- **F1** — the look-ahead attack choice (`ai/search.rs` `choose_attack_inner`, and `keep_cloud_home` in
  `impls/cloud.rs`, see F3) keeps an equipped Cloud home on ~13% of turns. When Cloud wears at least one Sword or
  other combat-trigger Equipment and the attack isn't a likely loss of Cloud for nothing, Cloud should attack. Gate the
  change to the cloud deck (or to an equipped commander in the cloud plan) so **no other deck's play changes**.
- **F2** — Stormfront Riders must not return an equipped Cloud when another creature can be returned (or when the
  tutor value doesn't beat the lost equips); prefer a cheap ETB creature or an unequipped Cloud.
- **F4** — `cloud_pick` (impls/cloud.rs ~294): Sword of Feast and Famine should rank where its results say (the audit
  found games with it do ~15 points better) — weigh the swords by what's already on the battlefield and in hand.
- **F8** — Unfinished Business is never cast ("untagged"): model it from its Oracle text (and check Recommission), with a
  test, marked Full only when complete.

## Checks
- `cargo test --release -j 12 --no-fail-fast` all pass; `--all-targets` build has no warnings.
- **Same-seed identity for other decks:** build origin/main's binary in a detached worktree under .claude/worktrees/
  (remove it after) and compare `commander-sim --deck <d> --pool all --profile loose --games 300 --ai adaptive --jobs 12
  --seed 7 --brief` for yshtola, seph, sephfs, sauron, kefka, sheoldred, veyran, titania, galadriel, alela: all identical.
- Do NOT run the Cloud A/B measurement; the lead runs it. A quick 200-game Cloud smoke run is fine.

Report ≤15 lines: line 1 DONE or HANDOFF, line 2 model ID, commits per fix, tests, identity check.
