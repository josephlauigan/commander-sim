# kefka-swat report

Cause: Deflecting Swat has no tag in data/carddb.txt, so `card_worth` (engine/tutors.rs) gives it 0 (Slip Out the Back
has prot=phase = 40). Kefka's wheel/looter discard (`kefka::discard_choice`) bins the lowest-worth card, so the Swat was
discarded to his own trigger before any removal could come. The report's "untagged" label is just `cast == 0 && !held`
(sim-cli/report.rs); `held` needs a prot=hi/phase/indes/blink/notw tag.

Checked and fine: `veyran_protect` (free with commander out, accepts every targeted kind except edict/wipe, target
must be a legal permanent of the caster's, Swat is tried before Slip Out). Tests added (cards_kefka_ai.rs): Murder
(destroy) and Chaos Warp (tuck) redirected onto the caster's creature, no-target case resolves normally (existing test
covers Swords/exile).

Fix (1 commit): `discard_choice` adds +40 for Deflecting Swat (same as other protection). Kept to Kefka only: a global
tag would change Veyran and the pool decks (brain.rs/pool.rs gate on Tag::Prot). Veyran does not discard to a wheel, so
it does not have this bug.

Measurement (kefka, t3, loose, lookahead, 320 games, seed default, jobs 8):
- Swat cast per game drawn: 1.6% -> 6.4%
- Kefka removed per game: 0.27 -> 0.26 (the removal events are mostly not answerable: sweepers/edicts, or Swat not in hand)
- Win rate 30.3% -> 25.0%: within noise for 320 games (SE ~2.5 points) but not a gain; trajectories differ after the
  first diverging discard. Re-run with more games before drawing any conclusion on strength.
Full cargo test --release -j 8 passes, no warnings. Shared files: none.
