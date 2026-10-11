# Handoff: kefka-cards

Done: deck wired as a main deck (ai/mod.rs styles, sim-cli lists, settings, rules CTHRESH, brain GENERIC_PLAYS + special_options,
ai/decks.rs deck_prio); impls/kefka.rs holds all 24 cards' code (resolve / hand_options / gy_options slots), madness+mayhem
tables, discard policy, flip utility, Crawler loop gate; overrides in data/card_overrides.toml (all marked Full).
Next: write tests/cards_kefka.rs (one per card + Crawler loop), run checks from OVERNIGHT-RULES, write report-kefka-cards.md.
Gotchas: no Python (I ran `python3 --version` once by mistake, nothing else). Shared files touched: state.rs (Player.discard_log, DataKey::Warp),
zones.rs (2 calls to kefka::discarded), cast.rs (discard_worst), turn.rs (kefka::paradigm), brain.rs, mine.rs (wheel, flip).
