# Handoff: model-gaps-a4
All four items are done and committed (see report-model-gaps-a4.md). Marvin is Full; `audit --mine` shows only Hexing Squelcher below Full.
Open questions for JD (not gaps in the model, a reading of the rules): text that names its own creature (Hope of Ghirapur, Hawkeye,
Balan, Drivnod, Sisay, Scion of the Ur-Dragon, Thanos, Nicol Bolas, Venat) is done by Marvin as that creature (as the "this creature"
texts are); Kefka and Tekuthal (a3's choice) name the donor. Proliferate (Cankerbloom, High Perfect Morcant through Marvin) proliferates
everything useful, as for every other source in the engine.
Gotchas: no Python; cargo only under the systemd-run cap from rust/; zsh does not split an unquoted `$args` string, use an array for
several `--test` flags; `cargo test` without --no-fail-fast stops at the first failing test binary.
