# audit-preflight: two-step preflight for `--swap` runs

Every `--swap` run (not --analyze / --all-decks / --calibrate / --trace) now runs a preflight before the main run.

1. AI-rule check (instant): `preflight::ai_rules` plays rounds 4 and 7 of two adaptive games of the swapped list vs the
   first tier, and takes the best `deck_prio` for each swapped-in card (a rule may wait for a graveyard or mana).
   "deck rule" means deck_prio differs from `pool::generic_prio`. Lines: `AI: <card>: priority N (deck rule|generic)`;
   `WARNING AI: <card>: priority 0 and no response path: the AI may never cast it` when prio is 0 and the card is not
   an instant or tagged ctr/rem/wipe/prot. Lands are reported as played, never cast.
2. Dry run (about a second to a minute): 200 adaptive-AI games per profile on the first tier (`Ctx::analyze_with`, new;
   `analyze` delegates to it). Per card: drawn% and cast% of drawn. Drawn in 20+ games and cast in under 5% ->
   `WARNING: <card> drawn N, cast M (x%): the deck's AI rarely casts it`, run stops with exit code 3.
   The dry-run verdict is the authority; the AI-rule warning alone does not stop a run.

Flags: `--no-preflight` skips both, `--preflight-only` runs them and exits (0 unless stopped). `--force` was not added.
Files: sim-cli `preflight.rs` (new), `args.rs`, `main.rs` (`run` returns an exit code), `poolmode.rs`, README options.

## Checks
- `cargo build --release --all-targets`: no warnings. `cargo test --release`: all pass (6 new tests: zero-priority
  warning, positive no-warning, response card, dry-run stop, --no-preflight skip, flag parsing).
- Bribery case: with the "Bribery" arm of `kefka_prio` temporarily set to `return 0` (reverted), `--deck kefka --pool t3
  --swap "Blightspeaker=>Bribery"` printed both warnings (drawn 156, cast 0) and exited 3.
- Real example (the brief's `Rune-Scarred Demon` is not in the kraum list; Breach is already there, so this puts a
  second copy in): `--deck kraum --pool t5 --swap "Archon of Absolution=>Underworld Breach" --preflight-only`:
  `WARNING AI: Underworld Breach: priority 0 and no response path` (on a plain board kraum_prio holds Breach for its
  combo line), but the dry run says drawn 38.8% (155), cast 27.1% (42): no stop. So the AI-rule check can false-alarm on
  held-for-a-line cards; the dry run settles it.
