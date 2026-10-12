# Report: sheoldred-upgrade2 (10/11)
Code: impls/sheoldred.rs "second upgrade package" (registered in `register_upgrade`); tests: end of tests/cards_sheoldred.rs (19 new).
New cards (data: slim Scryfall records appended to data/scryfall_cache.json, overrides Full, `cardgen export --offline`; Temple Bell, Geier Reach, Ambition's Cost, Syphon Mind have `dsl = []` so only the hand-written code runs):
- Howling Mine: `howling_mines(g, p)` is called in the draw step (engine/turn.rs, one line before the Necropotence check): every untapped Mine on the table gives the active player an extra draw. AI prio 70 while Sheoldred / Dreams / Fate Unraveler is out, else 0 (held).
- Temple Bell: `options` hook; each player draws, active player first. Tapped only with a punisher out and library >= 4: at the end of an opponent's turn (post None), or in a main phase when a draw would kill an opponent.
- Fate Unraveler: `draw` hook shared with Underworld Dreams (1 damage, prevention applies); prio 88.
- Peer into the Abyss: draws half the library (up), then loses half the life (up, computed after the draws). Option: the opponent it kills (draws x toll <= life - 1) at utility 12; else self only with library >= 30, life >= 30, hand <= 2.
- Geier Reach Sanitarium: `land_options`; {2},{T}: all draws, then all discards (discard_worst). Same window rule as the Bell.
- Ambition's Cost (draw 3, lose 3; prio 0 at life <= 6), Syphon Mind (each other player discards one, you draw per discard).
Old cards proven with tests in a sheoldred game: Mind Stone (mana + crack), Commander's Sphere (mana + sacrifice-draw via the DSL), Charcoal Diamond (enters tapped, makes B), Castle Locthwain, Phyrexian Arena, Read the Bones, Exsanguinate, Gray Merchant (devotion 4 with Sheoldred + itself), Swiftfoot Boots. All worked; two fixes were needed:
- Exsanguinate was never cast by the sheoldred seat (`yshtola::x_option` was gated to key yshtola): gate is now yshtola or sheoldred (shared file, one line).
- Boots: `equip_options` now offers Swiftfoot Boots on Sheoldred (utility 0.3 below the Helm; not while a Helm waits in hand; not if she is already hexproof).
Every opponent draw from the new cards fires Sheoldred, Underworld Dreams and Fate Unraveler (test).
Simplifications: Peer's life half is worked out after the draws; Geier Reach does all draws then all discards.
Checks: cargo test --release 1214 pass 0 fail; no warnings (--all-targets); all_decks adaptive 3 and lookahead 1 ok; audit --mine sheoldred 0 not Full (the new cards are not in the list; their verdicts are the overrides); 48-game --deck sheoldred smoke ok.
Swap smoke (all 20 swaps, 24 games, lookahead, loose): 16.7% -> 33.3%, delta +16.7 pts, noise +-26: no clear effect (too few games). `Swamp=>` twice works: apply_swaps removes the first remaining Swamp each time (34 -> 32), no fix needed.
Shared files: data/scryfall_cache.json, data/card_overrides.toml, data/cards.json (export); engine/turn.rs (1 line), impls/yshtola.rs (1 line).
Gotcha: the seph seat dredges instead of drawing, so draw-step tests use veyran as the opponent.
