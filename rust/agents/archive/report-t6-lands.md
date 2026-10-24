# Report: t6 lands batch (branch t6-lands)

Code: crates/sim-core/src/impls/t6_lands.rs; tests: crates/sim-core/tests/cards_t6_lands.rs (28 tests); data: data/card_overrides.toml; engine needs: rust/agents/engine-needs-lands.md. Shared files touched: impls/mod.rs (2 lines) only.

Full: City of Brass, City of Traitors, Abstergo Entertainment, Eiganjo, Deserted Temple, Minamo, Forbidden Orchard, Fomori Vault, Mount Doom, Talon Gates of Madara, Shatterskull Smashing // Shatterskull the Hammer Pass (both card names), Razorgrass Ambush // Razorgrass Field.
Partial: Gemstone Caverns (opening-hand clause), Emergence Zone (flash grant not read by the AI; option switched off), Petrified Hamlet (name chosen, ban and {T}: {C} not enforced), The Mycosynth Gardens (copy ability).
Notes: City of Traitors registers land_etb on every land without its own land_etb (registry gains an entry per land); lands with their own land_etb do not trigger it. MDFC lands are played via hand_options, not the engine land play (cleanup need listed).
