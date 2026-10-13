# Task: kefka-breach2 — why the Breach line rarely works in Kefka, and the hold fix

Read `rust/agents/SAURON-RULES.md` first, with these changes: **the machine is shared with a 22-thread sim run**: build
and test with `nice -n 19` and `-j 4`, smoke runs with `--jobs 4`. Your worktree's branch was made from
`sauron-weakness`.

Background: `kefka-breach` (report `rust/agents/report-kefka-breach.md`) made the Underworld Breach line work for any
deck. A screen with Brain Freeze, Dark Ritual and Lotus Petal in Kefka (for Duress, Dark Inquiry, Potioner's Trove)
measured -0.8 points: Breach cast in 23% of the games it was drawn, Brain Freeze in 12%, opponents decked 8.4% of
eliminations (0.4% without), and no game won by the line. We need to know where it breaks.

## 1. Diagnostics in `--analyze` (any deck whose list has Underworld Breach and a storm card)
A "Breach line" block: share of games where Breach and a storm card were both held (hand, battlefield or graveyard as
the line counts them) at some point on our own turn; games the dry run was tried / passed; games executed; games the
line won (every opponent dead or decked); and for failed dry runs, the main shortfall (mana, blue mana, graveyard
fuel, storm count) — reuse `line_state` / `dry_run` in `impls/mine.rs`, recording per-player stats (`pl.stat`) the
report reads. Report-only: no game decision may change (check a fixed-seed `--brief` run of sauron and kefka is
identical before and after this part).

## 2. Hold fix (AI)
`ai/decks.rs` `breach_hold_prio` holds a storm card only while Breach is out; otherwise Brain Freeze can be cast
plainly for a three-card mill and the piece is gone. For a deck that runs the line (`deck_has_storm`), hold storm
cards whenever Underworld Breach is still reachable (in hand, library, or findable by a tutor in hand) unless casting
it now kills an opponent. Same for Sauron's deck (it is `breach_first`-style logic in `impls/mine.rs`; check Sauron's
existing hold rule first and change it only if it has the same leak). Tests for both.

## 3. Cards for a Kefka helper package (Full, with tests in the line)
Grapeshot (the line already supports it), Gamble, Jeska's Will, Cabal Ritual: check each works in Kefka's line (as
ritual / storm card / tutor for Breach), and that Kefka's tutor picks use Gamble for Breach.

## Smoke and report
`--deck kefka --pool t3 --profile loose --games 200 --ai lookahead --jobs 4 --analyze --swap "Duress=>Brain Freeze"
--swap "Dark Inquiry=>Dark Ritual" --swap "Potioner's Trove=>Lotus Petal"`: paste the new Breach-line block into the
report (before and after the hold fix if you can, same seed). Also the same block for `--deck sauron` (current list).
