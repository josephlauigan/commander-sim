# Report t6-e-mana (plan row 16)
Mechanism: (a) partials::HAND_MANA table (Elvish G, Simian R) + engine/mana.rs adds hand mana even when no permanent is hooked (it used to need one); Simian prio_0 like Elvish. (b) engine/mana.rs creature mana sources (dork, token mana, rite) ask t6_legends1b::ability_haste lazily; t6_green1 can_tap too. (c) Thrasios uses t6_green1::creature_ability_cost (options + go). (d) t6_green1::untap_perm + speaker_options/go: land, or tapped creature/artifact; AI chains onto Candelabra/Magus (net mana after cost beats best land) or rock making 3+.
Full: Simian Spirit Guide, Formidable Speaker, Tyvar. Partial: Shang-Chi (restricted mana), Familiar/Training Grounds (other creature abilities still hard-code costs).
Other decks: Spirit Guides now usable with no hooked permanent in play (real fix, affects decks running them). Mana-source scan costs a hand scan per call when unhooked.
Needs found: none new. Tests: tests/engine_mana_effects.rs (5).
