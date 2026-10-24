# Report t6-e-flash
Mechanism: `engine/cast.rs::grants_flash(g,p,c)` = Quicksilver/Mar-Vell `as_flash` || `t6_blue::static_flash` (Trickster all, Floodcaller noncreature, Borne/Emergence turn flag) || Scout's Warning (creature cards). `on_cast` spends Scout's grant on a creature spell.
Readers: `human/legal::instant_speed(g,p,c)` (new p arg; human timing), `ai/brain::end_of_turn_window` (sorceries and permanents with a grant, not only Gandalf/Quicksilver perms), `cardcode` mana reserve, main-phase hold of creatures under a permanent grant. `EMERGENCE_LIVE = true`.
Full: Borne Upon a Wind, Emergence Zone, Scout's Warning, High Fae Trickster, Valley Floodcaller.
Other decks: human timing now also honours Quicksilver/Mar-Vell; AI casts sorceries at EOT only with a grant (Gandalf unchanged); Cap'n/Quicksilver AI path for non-permanent creatures unchanged in effect.
Tests: tests/engine_flash.rs (6). Not done: AI responding mid-stack with a granted sorcery (only the EOT window and human play use grants).
