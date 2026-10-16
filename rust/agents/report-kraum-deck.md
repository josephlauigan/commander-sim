# Report: kraum-deck (Sonnet 5.5, 2026-10-16)

## Registration
- `data/my_decks.toml` `[kraum]` (tiers t1,t2 provisional); `sim-cli/src/decks.rs` FILES/MINE (after cloud), `args.rs` DECK_KEYS (12), `sim-core/tests/game.rs` count 72, `sim-cli/src/poolmode.rs` tier test (+ kraum, provisional), `decks.rs` mine.len 12.
- Practice mode: `practice/src/catalog.rs` already drops any deck with `commander2`, so the practice tests (11 decks) are unchanged.
- `testkit.rs`: kraum removed from `retired_decks()` (thrasios and garland stay). No test needed a setup change.
- `cardgen deck kraum <import list>` (needed network for Dragoon's Lance, Gilgamesh, Katana) rewrote only the doc's import list order (commanders now sorted in) and `tests/fixtures/my_decks_parsed.json`; `cardgen export` done. Strategy text untouched.

## Cards (audit: kraum 93 unique, 0 not Full)
All in `impls/tymna_cards.rs` (`register_gear`), verdicts in `data/card_overrides.toml`, tests in `tests/cards_tymna.rs`.
- **Gilgamesh**: enters/attacks, top six, every Equipment onto the battlefield, rest to the bottom shuffled, then the best Equipment without its own Hero (rank Flail > Blade > Nim > Heirloom > Kitesail > Boots) attached to a Samurai (himself, else a Katana wearer). The AI takes all Equipment; a person only answers one yes/no for the whole trigger. Job select pieces keep their Hero.
- **Dragoon's Lance / Samurai's Katana**: new dsl overrides (+1/+0 equip 4; +2/+2 trample haste equip 5), job select reuses `cloud::staff_etb` (made `pub`), Lance flying on its controller's turn via `grant_kw`; Samurai via `tymna_cards::is_samurai`. Knight type not tracked (nothing cares).
- **Hero's Heirloom**: was marked Full but "trample and haste while legendary" was NOT implemented (compiled data lists it unparsed). Now a `grant_kw` hook. This also changes the Cloud deck slightly (legends with Heirloom get trample/haste), which is correct.
- **Conqueror's Flail**: +1/+1 was a flat +3 in `values.rs`; now one per colour among the controller's permanents (`tymna_cards::colors_among`). Also affects the Sauron deck (which plays it); correct but a small shift. Casting lock verified: `values::silenced` is honoured in `stack::take_priority` (no opponent responses at all, including counterspells), `ai/mod.rs` and Teferi's Protection, and the human legality checks.
- **Boiling Rock Prison**: {4},{T}, sacrifice: draw. AI: end of turn only, six or more lands, no land in hand.
- **Irrigated Farmland**: was Full ("mana only") but cycling {2} was ignored; AI now cycles it when not needed as a land drop.
- Already correct, now tested for this deck: Hero's Blade, Trailblazer's Boots, Medic's Kitesail, Nim Deathmantle, Spectacle Summit. Diabolic Tutor: see AI.

## Deck AI checks
1. **Equipment placement**: `tymna_cards::equip_holder_value` (hooked into `dsl::equip_options` and Nim's equip in mine.rs; zero for every other seat). Evasion gear (Kitesail, Lance, Boots, Nim) is scored by the opponents it newly lets through, minus 3 if the creature already connects; size gear (Heirloom, Blade, Katana) prefers a creature that connects anyway. Generic scoring used to prefer fliers for everything. Equip utility is capped at +2 so a creature cast (6.6+) still comes before an equip; priorities in `partner_prio`: Gilgamesh 68, Lance/Katana 63, Blade 58, Flail 60 and other Equipment 56 once a creature is out (25 before; it used to be 10, i.e. cast last).
2. **Tymna's spread**: unchanged code; test with a Kitesail ground creature plus two fliers hits all three opponents. New evasion is read through `can_block`, so `spread` sees it.
3. **Flail**: has no compiled equip ability, so the kraum seat could never equip it; `kraum_special` adds an equip option (utility 4.5, main phase one only, before combat). Equipment stuck rate: not measurable at 40 games (the card report needs 100 per card); in 8 traced games Equipment was cast and equipped when mana allowed. **Diabolic Tutor**: `tymna_cards::tutor_pick`: a land when under five lands (colour it lacks first), Toxic Deluge when an opponent's board is 6+ power ahead, Flail then a finisher when ahead, else the cheapest evasive creature, else the generic pick.

## Numbers (smoke only)
40 games vs t1, loose, lookahead: win 2.5% and 5.0% on two runs (noise at n=40). Tymna lands turn 3.8, Kraum 5.9, opponents hit per Tymna turn 0.66, 2.2 cards drawn per game. The deck connects rarely; that is the analysis run's job.

## Shared-file changes (all small)
`dsl/mod.rs` (two added terms in `equip_options`), `engine/values.rs` (Flail colours), `ai/brain.rs` (one `"kraum"` arm), `ai/decks.rs` (one `"kraum"` tutor arm), `impls/mine.rs` (Nim wearer term), `impls/cloud.rs` (`staff_etb` pub), `impls/tymna_partners.rs` (priorities), `testkit.rs`, `data/card_overrides.toml`.

## Checks
`cargo test --release -j 6`: 1926 passed, 0 failed. `cargo build --release -j 6 --all-targets`: no warnings. `all_decks` runs skipped as briefed.
