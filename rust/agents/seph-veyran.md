# Task: the user's Sephiroth and Veyran decks (card code + AI plans) (+ one bug fix)

Worktree: `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/agent-a0c0f5517cd52202e`
(branch `worktree-agent-a0c0f5517cd52202e`). A previous agent was stopped while adding the Breach-line test; its
work is in WIP commit 080f43e. Check `cargo test --release` first.

Files: `impls/mine.rs` (non-Sauron parts), the Sephiroth/Veyran functions and dispatch lines in `ai/decks.rs`,
`ai/brain.rs`, `ai/plans.rs`, `ai/mod.rs`, their placeholders in `cardcode.rs`; tests `tests/cards_seph.rs`,
`tests/cards_veyran.rs`.

1. The rest of `commander_sim/cards/impl/mine.py`: Sephiroth and Veyran sections and every non-Sauron part (Summon:
   Bahamut and CI.SAGA if there, `loop_need`, Galvanic Iteration, Fiery Emancipation, Docent of Perfection, the
   flicker package ...). Plaza of Heroes / Unclaimed Territory colours are already in jodah.rs. `mine.flicker_worth`
   replaces the `seph_flicker_worth` placeholder (t2 uses it).
2. Their AI plans from `ais.py` (and ai/brain.py, deck_plans.py, gc_prio.py ...): seph_prio, veyran_prio,
   seph_tutor_target / wish list, seph_bval, seph_dredge, end-step milestones, loops, reanimation, fill, tutors,
   hardcasts, sacrifice outlets under Conqueror's Flail, protection and answers (Ephemerate, Restoration Angel,
   Heroic Intervention, Galadriel's Dismissal kicked, Teferi's Protection ...), Veyran's combo and engine_payoff —
   everything marked `PORT(phase 6)` naming Sephiroth or Veyran. ai/decks.rs already dispatches yshtola, galadriel,
   alela, jodah: add seph and veyran arms alongside.
3. marchesa.rs carries a copy of mine.py's ETB_VALUE table: once mine.rs has it, point marchesa at mine's.
4. Bug fix (Rust only, a test): Sauron's finished Breach line keeps being offered once every opponent's library is
   empty (the 15.0 option does nothing until the action cap): return nothing when there's nothing left to do. Don't
   undo aab0df6 (Sauron no longer mills himself).
5. Tests: port test_my_cards.py's Sephiroth and Veyran classes and tests/test_*seph* / test_*veyran* (skip only
   practice-mode ones, say which), plus plan-choice tests.

Difftest: `python3 rust/tools/difftest.py --gen 1000 --seed 7 --decks seph,veyran,t1,t3` (phase 6 bucket for these
decks should be empty).
Commit message: "Rust port, phase 6: Sephiroth's and Veyran's card code and AI plans".
