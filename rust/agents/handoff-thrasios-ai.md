# Handoff: thrasios-ai

Done (commit 1): Blossoming Defense protection (tymna_cards::blossom_protect, thrasios seat) + combat trick
(tymna_partners::combat_trick, hooked in engine/combat.rs after defend_hooks) for Blossoming Defense / Dark Endurance;
held mana for both (tymna_partners::protection_held, used by brain::hold_value and the Thrasios main-2 keep);
Ballista/Basalt/Rings priorities (combo_prio, thrasios seat only); combo tutor pick (combo_tutor_pick); Thrasios's {4}
left out of the search and made when the search says stop (search.rs thrasios_sink); plan milestone "tymna" (legends.rs)
+ report plan_turn/plan_text for thrasios and kraum; Ballista pings X/1 utility creatures (common.rs, thrasios only);
tests in tests/thrasios_ai.rs (11). CARD_REPORT_MIN env var lowers the card report threshold for small diagnostic runs.

Next (in order): check 200-game numbers vs base (scratchpad base200.txt: BD never cast, activations 0.69);
Dark Endurance why-not-cast; stuck-in-hand cards (item 6); green cards (item 7); trace 10 games; report.
