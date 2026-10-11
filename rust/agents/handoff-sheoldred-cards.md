# Handoff: sheoldred-cards

## Key finding (10/11)
The audit's "Full" verdicts for many deck cards are STALE: the candidate decks (glissa, gix, yawgmoth) were retired and
their card code deleted, but data/card_overrides.toml still says Full. A probe test showed no registry impl and no
effect for: Cost of Brilliance, Vault Plunderer, Mabel, Melancholic Poet, Seymour Flux, Canyon Crawler, Foolish Fate
(old glissa.rs), Al Bhed Salvagers, Necrosquito, Callous Inspector, Apex Witchstalker, Gravblade Heavy, Ulna Alley
Shopkeep, Restless Bloodseeker, End of the Hunt (old gix.rs), Perigee Beckoner, Phyrexian Delver, Soul Enervation
(old yawgmoth.rs). Old code is in git: `git show 6fc094b^:rust/crates/sim-core/src/impls/glissa.rs`, `9c289b5^:...gix.rs`,
`fdfb1c6^:...yawgmoth.rs` (copies in /tmp/claude-1000/old_*.rs may be gone). Plan: port those cards into impls/sheoldred.rs
(plus Player::bloods in state.rs, die_rest hook for Perigee in zones.rs), then the brief's 8 cards, Part 2, AI plan.

## Done
(nothing committed yet)

## Next in order
1. port the stale cards, with tests in tests/cards_sheoldred.rs
2. brief's 8 cards; Sheoldred herself tests; Part 2; AI plan
