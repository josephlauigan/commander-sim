# Report t6-speed-rthras (Rograkh/Thrasios, end-of-turn dig) - no gain; scry change dropped
Branch t6-speed-rthras (from t6-speed). Runs: /tmp/claude-1000/t6speed-rthras/ (seed 500000).

## Finding: the end-of-turn dig already exists and uses the spare mana
Thrasios's {4} dig is offered in brain::end_of_turn_window (impls/tymna_cards.rs thrasios_options, utility 3) at the end of the turn
before the deck's own turn. That is equivalent to digging at each opponent's end step: the untapped mana is one pool that does not
refill between those steps. Gaea's Cradle mana counts (total_mana). Measured per Rograkh/Thrasios game (600 games): 8.8 windows,
Thrasios in play in 5.2 of them, 2.7 mana untapped on average, so about 2.9 digs offered; actual 2.2 at end of turn + 1.0 on its own
turn. The dig is mana-limited, not window-limited. No shared end-of-turn hook was added (nothing needed).
Only 0.41 of ~3.2 digs hit a land: scry bottoms a land when the deck has 4+ lands (topdeck land_need).

## Tried: keep a land on top (no scry while mana < 8, Tier 6)
600 games: Thrasios 14.0 -> 18.1 (land hits 0.41 -> 0.82). 2000 games, loose profile, seed 500000, base vs fix:
Thrasios 16.4 (14.5-18.5) -> 15.6 (13.8-17.7); Dack 28.3 -> 29.4; Kinnan 26.2 -> 26.9; Kraum/Tymna 33.6 -> 33.2; Silas 24.3 -> 22.3;
Sisay 21.5 -> 22.7. Median win round of Thrasios 12 -> 12. The 600-game gain was noise. DROPPED (code reverted).

## Kept: stats only
`mn|dig|eot|own|land`, `mn|eotw|*` (t6stats::eot_window, called in end_of_turn_window), one render line in t6report ("Thrasios digs per
seated game ..."). No decision or RNG change (win rates identical with and without).

## Checks
t3 lock diff (300 games, seed 500000): identical. Full `cargo test --release` (on the build with the scry change): 159 suites, 2753
passed, 0 failed. After reverting the scry hunk: ai_t6_rograkh_thrasios, cards_tymna, engine_t6_combo3 re-run (see final message).
Where the speed is: pieces (Cradle, Oboro, Gates) arrive late and commanders get removed (rthras3/4); more digging does not change that.
