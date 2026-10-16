# Handoff: kraum-deck

## Done
- Deck re-registered (my_decks.toml tiers t1,t2 provisional; decks.rs FILES/MINE; args.rs; game.rs count 72; testkit retired_decks no longer has kraum; cardgen deck + export run).
- Card code in impls/tymna_cards.rs (register_gear): Gilgamesh, Dragoon's Lance, Samurai's Katana (job select via cloud::staff_etb), Hero's Heirloom (legendary trample/haste), Boiling Rock Prison, Irrigated Farmland cycling; overrides added.
- Conqueror's Flail now +1/+1 per colour (values.rs) instead of flat +3.
- AI: kraum equip placement (equip_holder_value, hooked in dsl::equip_options + mine.rs Nim), Flail equip option (kraum_special), Diabolic Tutor pick (tymna_cards::tutor_pick).

## Next
1. Tests in tests/cards_tymna.rs for all 13 cards + AI points.
2. cargo test, build --all-targets no warnings, audit, smoke run.
3. report-kraum-deck.md.
