# Handoff t6-green1
Batch: Oboro Breezecaller; Badgermole Cub; Biomancer's Familiar; Earthcraft; Training Grounds; Candelabra of Tawnos; Magus of the Candelabra; Formidable Speaker; Springheart Nantuko; Vibrance
## Done
- Oboro Breezecaller: Full (commit "t6 green1: Oboro Breezecaller"). Helpers in t6_green1.rs: ability_discount / creature_ability_cost (Training Grounds, Familiar), land_yield.
## Next
Badgermole Cub next, then Familiar, Earthcraft, Training Grounds, Candelabra, Magus, Speaker, Nantuko, Vibrance. Oracle text saved in scratchpad g1.json.
## Gotchas
- Test cmd: from rust/: CARGO_BUILD_JOBS=2 nice -n 19 systemd-run --user --scope -q -p MemoryMax=12G -p MemorySwapMax=0 cargo test --release -p sim-core --test cards_t6_green1
- Override entries are appended to data/card_overrides.toml; verify with ./target/release/cardgen audit --deck rograkh-thrasios-creature-combo
- Lands are LandId (not perms): earthbend (land becomes creature) is an engine need.
