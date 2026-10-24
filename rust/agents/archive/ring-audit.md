# Task: ring-audit — is the AI playing The One Ring well in the sephfs deck?

Read `rust/agents/TYMNA-RULES.md` and `rust/agents/OVERNIGHT-RULES.md` first. Base branch: `tymna-thrasios` (the commit
that added this brief). Your worktree's branch: `ring-audit`. Use at most `cargo -j 8` and `--jobs 8` (another session
may be running sims; check /home/joseph/projects/mtg_sims/cpu-ledger.md). Never use `git stash`.

Context: the paired test The One Ring for Caged Zombie in `sephfs` (decklists/JD/sephiroth-fs-wipe-steal.md) gave +1.3
±1.5 over all tiers (T1 +1.9, T2 +3.1, T3 +2.4, T4 +0.5, T5 -1.2; audit/sephfs/2026-10-14-ring/). The Ring is
modeled (audit: Full; engine/life.rs ring protection, engine/turn.rs, impls/common.rs priority 6.0). JD wants to know
whether the AI uses it correctly and optimally before buying it (~$110).

## 1. Diagnose (no code changes yet)
Run the swapped list (`--swap "Caged Zombie=>The One Ring"`, seed 1014, look-ahead, loose) with traces (`--trace`)
and/or counters, against t1, t3 and t5 (200-300 games each is enough for the numbers; read 10-20 traced games in
which the Ring is cast). Measure and report:
- **Casting:** cast rate when drawn; turn cast; main phase 1 or 2; what else the mana could have done that turn
  (Sephiroth, a wipe, Grave Betrayal); whether it's cast on a turn when its protection matters (opponents about to
  attack us, a lethal attack coming) or just when mana is free.
- **Protection:** how often the protection prevents damage, and how much; games where it saved us.
- **Activation:** taps per game; does it tap every turn; does it tap at the right time (it's an untap-step permanent:
  tapping at the end of an opponent's turn is usually best, so the cards are available and the burden life loss comes
  at our upkeep); does it stop tapping when burden life loss gets dangerous (life total vs the next upkeep's loss).
- **Life loss:** total burden life loss per game; games lost where burden loss contributed (died within a few turns
  with high burden).
- **Rules checks:** protection from everything should also stop damage from our own sources (Pestilence) — the
  model only stops opponents' damage (engine/life.rs comment); confirm, and say how much it matters here.
- Tier 5: why is it slightly negative there (mana spent on the Ring instead of interaction?).

## 2. Fix what is clearly wrong (gated to key "sephfs" or rules-correct for everyone)
Only if the diagnosis finds a clear mistake: fix it (tests for each), then rerun the paired test for the affected
tiers with the fixed binary (`--swap "Caged Zombie=>The One Ring"`, 1,000 games, seed 1014) and report before/after.
A rules fix (protection vs our own damage) must leave other decks' games identical unless they run the Ring (only
Sheoldred-tested lists did; check `data/decks.json`).

## Report
`rust/agents/report-ring-audit.md` (numbers, findings, fixes, before/after); final reply at most 15 lines: line 1
DONE or HANDOFF, line 2 your model ID, then findings, fixes, numbers, anything I must act on.
