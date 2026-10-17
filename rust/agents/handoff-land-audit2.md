# Handoff: land-audit2

## Done (commit c42a01f and later)
- Audited all 243 nonbasic lands in data/cards.json against Oracle text (tags c/t/ck/f/pain, enters_rule, hooks).
- Fixed (test per land in rust/crates/sim-core/tests/cards_land_audit2.rs): Twilight Mire, Rugged Prairie, Mystic Gate
  (filter lands via t0_zurgo heath_cols, now also needs an UNTAPPED feeder and not just other filters), Sungrass Prairie /
  Ferrous Lake (0 mana without another untapped land), Spire of Industry (artifact + pain), Exotic Orchard (opponent
  land colours), Cavern of Souls (creature type only, territory_colors), Hall of the Bandit Lord (pay 3 life),
  Spectator Seating (c=RW, was any colour), Sulfur Falls (not a painland), Hidden Grotto (colourless only),
  Fabled Passage (untaps with 3 other lands, engine/turn.rs crack_fetch).
- Not faults (already modeled): Horizon lands (pain on_tap_land), Vivid lands, Gemstone Mine, Avengers Tower, Aether Hub,
  Plaza of Heroes, Unclaimed Territory, Secluded Courtyard, Tainted lands, Temple of the False God.

## In progress
- Same-seed before/after run: scratchpad cmp.sh (base worktree .claude/worktrees/land-audit2-base at origin/main, built).
  DO NOT edit data/ while it runs (binary reads data/ at run time).

## Next
- Dryad Arbor enters tapped (summoning sick land creature), then cardgen export.
- Write report-land-audit2.md with before/after table; remove the land-audit2-base worktree.
