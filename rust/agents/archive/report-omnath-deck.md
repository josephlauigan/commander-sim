# Report: omnath-deck
Deck `omnath` (Omnath, Locus of Rage; RG; tiers t1-t3) added beside `titania`: data/my_decks.toml, decks.json, fixture,
decklists/JD/omnath-landfall.md, CHANGELOG, data/printings/omnath.txt (4 new buys use placeholder printings),
practice catalog + tests (count 13->14; win-rate test skips omnath: no results row yet), sim-cli keys, ai/engine key lists.
Plan: impls/titania.rs `is_deck()` covers titania|omnath (shared by key, not copied); Titania cost from hand for omnath.
Omnath, Goblin Bombardment, Oracle of Mul Daya: Full in overrides; Harrow modeled from Scryfall+dsl. Test: tests/cards_omnath.rs.
Check: 100 games vs t1, adaptive loose: 27.0% (CI 19-36), plan online 60%, no panic. Full test suite and build clean.
Cards that would move from bulk-cards.txt to the deck when Titania retires (not moved):
Bestial Incursion, Cinder Glade, 2 Cultivate, Decimate, Evolving Wilds, 2 Follow the Lumarets, Greenhouse Propagator,
Hellkite Tyrant, Malevolent Rumble, Noxious Newt, 3 Rending Vines, Rootbound Crag, Scorching Dragonfire,
4 Silhana Starfletcher, Something Worth Saving (only the copies the deck uses; some may already be in Titania).
No Goblin Bombardment-specific cast rule added (not needed by the check).
