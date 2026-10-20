# Handoff gix-priest
Done: (1) Priest of Gix modeled: override in data/card_overrides.toml, name added to `extra` in cardgen export.rs, export run (cards.json, cache).
Baselines (before) in scratchpad cloud-before.txt / seph-before.txt (200 games, win rates 20.0% / 44.0%); script base.sh <tag>.
Next: (2) loop in impls/mine.rs LOOPS (+ run_loop non-kill: no Zombies for Priest), human loop_abilities uses LOOPS so automatic;
(3) blink valuation (mine.rs ~4312 blink_worth); (4) tests in tests/cards_seph.rs; then checks + report-gix-priest.md.
