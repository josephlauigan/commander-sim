# kefka-breach2 report
DONE. Model claude-sonnet-5-5. Branch kefka-breach2 (see git log).

## 1. Breach-line diagnostics (`--analyze`, any deck whose list has Underworld Breach and Brain Freeze/Grapeshot)
Per-game flags in `pl.stats` (`bl_held` in `breach_options`, `bl_dry`/`bl_pass`/`bl_short` in `dry_run`, `bl_won` at the end of
the executed line; `breach_line` existed); report in `sim-cli/src/report.rs`. Report-only: fixed-seed `--brief` of sauron and
kefka (t3, 96 games, adaptive, seed 500000) byte-identical before/after. Shortfall is a rough reading of the last failed dry run
(mana < 3, no blue source, < 3 fuel, else storm count / other).

## 2. Hold fix
`breach_hold_prio` (ai/decks.rs): storm cards held while Breach is out OR still in the library, unless casting now finishes an
opponent (Brain Freeze: library <= 3 per copy; Grapeshot: life <= copies). New `breach_reachable`, `storm_kills`.
Sauron: `sauron_prio` already returns 0 for every Storm/LED card, no leak, unchanged (test added).
EFFECT: none measurable. Kefka smoke (200 look-ahead) and 1500-game adaptive runs are identical before/after (win rate, every
Breach-line number, Brain Freeze cast rate): Brain Freeze was already never cast plainly (generic priority holds Storm-tag cards).

## 3. Cards
Grapeshot, Gamble, Cabal Ritual already Full in overrides; Jeska's Will Full via Veyran's audit. Tests (cards_kefka_breach2.rs):
Grapeshot burns the table in the line, Cabal Ritual and Jeska's Will make the line pass (Cabal with negative control),
Gamble has a cast priority in Kefka and the tutor pick is Breach (then Brain Freeze). No code needed. Caveat: Gamble's random
discard can bin the Breach it just found (engine rule, unchanged).

## Where the line breaks (the Brain Freeze/Dark Ritual/Lotus Petal Kefka variant, t3 loose)
Adaptive, 1500 games, win 23.7%: Breach and a storm card held on our turn in 18.5% of games; dry run tried 16.3%, passed 8.2%;
executed 8.2%; won 7.9% (the line wins whenever it runs: Brain Freeze win rate when cast 63.3%). Failed dry runs: mana 3.5%,
blue 0.2%, fuel 0.9%, storm count/other 7.6% of games. So the loss is assembly (only 1 Breach + 1 storm card in 99: both held in
under a fifth of games) and half of the held games failing the dry run on storm count / mana, not execution or the hold.
200-game look-ahead (same swaps): held 14.5%, tried 13.0%, passed 6.0%, executed 5.0%, won 5.0%; win 24.0%.
Sauron current list (t3 loose, 200 look-ahead): held 11.0%, tried 9.5%, passed 3.5%, executed 3.0%, won 3.0%; win 19.5%.

## Checks
cargo test --release (all, 0 failed), build --all-targets 0 warnings, all_decks adaptive 3 / lookahead 1 no panic.
Shared files touched: none beyond impls/mine.rs (stats inserts, `line_shortfall`) and ai/decks.rs (hold fix).
