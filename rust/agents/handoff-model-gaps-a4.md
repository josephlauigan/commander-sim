# Handoff: model-gaps-a4
Items 1-6 are done and committed (see report-model-gaps-a4.md). Marvin is Full; `audit --mine` shows only Hexing Squelcher below Full.
Follow-up 5: Marvin does named-creature texts as itself (rule 201.5): Tekuthal's counter goes on Marvin, a borrowed Kefka {8} transforms nothing (701.28c).
Follow-up 6: a person chooses what to proliferate (`human/proliferate.rs`, hooked at the top of `common::proliferate`'s rounds); the AI is unchanged.
Nothing left under NEEDS JD. Not run (by the brief): the full suite and any sim.
Gotchas: no Python; cargo only under the systemd-run cap from rust/; zsh does not split an unquoted `$args` string, use an array for
several `--test` flags; `cargo test` without --no-fail-fast stops at the first failing test binary.
