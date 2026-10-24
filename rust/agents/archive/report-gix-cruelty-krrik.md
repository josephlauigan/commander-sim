# Report: gix-cruelty-krrik
Cards: The Cruelty of Gix (Seph FS), K'rrik, Son of Yawgmoth (Sheoldred). Both Full in data/card_overrides.toml; fetched into
data/scryfall_cache.json, `cardgen export` run (cards.json).

## Gix (impls/sephfs_cards.rs, gix_*; priority 56 in impls/sephfs.rs)
AI start chapter: III when any graveyard (opponents' too) holds a creature with steal_value >= 3; II when the deck's tutor has a
named wish (`tutor_pick_named`) and life >= 14; else I. I: best creature/planeswalker card in any opponent hand is discarded;
II: shared `tutors::tutor(.., "any")`, lose 3; III: best creature from any graveyard. Person: asked chapter, opponent, cards.
Lore counter at upkeep (like Braids, not after the draw step); no proliferate. The auto "tut" tag was cleared (override tags = "")
or it would have tutored on enter as well.
## K'rrik (impls/krrik.rs)
`krrik::life_pips` / `drop_black`: with K'rrik out, black pips of a black spell can be paid with 2 life. AI: only pips the mana
cannot pay and only while life stays >= 15 (LIFE_FLOOR): Sheoldred drains/gains so life is a resource, but T3 opponents can
take ~12 in a round, and the existing Phyrexian rule's 10 is too low with several 2-life pips. Person: `legal::phyrexian(g,p,c)` gives
every black pip as mana-or-life, no floor. Counter via `cast` hook. Not modeled: life for {B} in abilities and in removal casts.
Cast priority 70 (>= 16 life) in sheoldred.rs pkg2_prio.
## Shared-file changes
engine/mana.rs (`phyrexian_life` now takes the player; `cost_of` calls drop_black), ai/brain.rs (guard + calls), human/legal.rs and
human/human.rs (`phyrexian(g,p,c)`), impls/mod.rs, tests/cards_t6_green2.rs (signature).
## Checks
10 new tests (tests/cards_gix_krrik.rs); `cargo test --release -j 3` all pass (170 result lines, 0 failed); build: 0 warnings.
Cloud T1 200 games before/after: identical (both with a "Win rate" line, 20.0%).
Sims (50 games, smoke only): sephfs T5 Mind Stone=>Cruelty 36.0 -> 42.0 (noise 6.8), preflight cast 56.5% of draws;
sheoldred T3 Stir Up Trouble=>K'rrik 32.0 -> 32.0, cast 52.0%. FINDINGS updated for both decks.
