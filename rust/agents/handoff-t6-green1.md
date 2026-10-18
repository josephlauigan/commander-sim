# Handoff t6-green1
Batch: Oboro Breezecaller; Badgermole Cub; Biomancer's Familiar; Earthcraft; Training Grounds; Candelabra of Tawnos; Magus of the Candelabra; Formidable Speaker; Springheart Nantuko; Vibrance
## Done
- Oboro Breezecaller: Full. Helpers in impls/t6_green1.rs: ability_discount / creature_ability_cost (Training Grounds, Familiar), land_yield.
- Badgermole Cub: Partial (earthbend in engine-needs-green1.md).
- Biomancer's Familiar: Partial (discount only for this batch's abilities; engine-needs-green1.md).
- Training Grounds: Partial (same engine need as Familiar).
## Next
Candelabra, Magus, Speaker, Nantuko (landfall Insect + bestow), Vibrance (evoke), then full cargo test + build warnings + report-t6-green1.md.
## Gotchas
- Test cmd (from rust/): CARGO_BUILD_JOBS=2 nice -n 19 systemd-run --user --scope -q -p MemoryMax=12G -p MemorySwapMax=0 cargo test --release -p sim-core --test cards_t6_green1
- Overrides appended to data/card_overrides.toml; check with ./target/release/cardgen audit --deck rograkh-thrasios-creature-combo
- Lands are LandId (not perms): earthbend is an engine need. Oracle text: scratchpad g1.json.
- A python3 call is blocked by a hook; don't.

## Takeover 2026-10-17
Second agent took over. Earthcraft: Full (committed). Remaining: Candelabra, Magus, Speaker, Nantuko, Vibrance.
