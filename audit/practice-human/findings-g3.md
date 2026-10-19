# Findings: g3 (cloud.rs, tymna_cards.rs, legends.rs, galadriel.rs, rules2.rs, t0_giada.rs, combos.rs)

28 findings: unreachable 6 (4 high, 2 high-uncertain), chosen 15 (1 high, 1 high-uncertain, 9 med, 4 low),
opponent-choice 1 (med), check 6. Paths under rust/crates/sim-core/src/impls/.

| Card | File:line | Kind | What happens for the person | Sev |
|---|---|---|---|---|
| Command Beacon | legends.rs:277 | unreachable | `land_options` only; the human path never reads it and `cards::land_abilities` has no entry: only the AI can sacrifice it for the commander | high |
| Profane Tutor | legends.rs:1618 | unreachable | Suspend ({1}{B}, suspend 2) is `hand_options` only | high |
| Woe Strider | rules2.rs:346 | unreachable | Escape is `gy_options` only (human escape exists only for Underworld Breach) | high |
| Ragavan | rules2.rs:657 | unreachable | Dash is `hand_options` only | high |
| Zack Fair | cloud.rs:871 | unreachable (check) | Sacrifice ability only reached via `cloud_protect` (AI protection) | high |
| Plaza of Heroes | cloud.rs:1391 | unreachable (check) | Exile ability only reached via `cloud_protect` | high |
| Winds of Abandon | rules2.rs ~755 | chosen (check) | Overload forced with 4 spare mana; without ctx.target falls back to `best_opp_creature` | high |
| Godo, Bandit Warlord | tymna_cards.rs:1034 | chosen | AI picks the Equipment to fetch (`godo_gear_score`) and attaches it | high |
| Akroma's Will | cloud.rs:1912 | chosen | Always mode 1, plus mode 2 only with a commander out | med |
| Armory Automaton | cloud.rs:487 | chosen | "Any number of Equipment" auto-set to every pullable one | med |
| Kor Outfitter | cloud.rs:530 | chosen | `best_pull` / `best_holder` pick the Equipment and the creature | med |
| Brass Squire | cloud.rs:549-585 | chosen | One AI-chosen pair offered, main phase 1 only | med |
| Cid, Freeflier Pilot | cloud.rs:683-710 | chosen | `card_worth` picks the Equipment/Vehicle returned | med |
| Helitrooper | cloud.rs:729 | chosen | Highest-power attacker targeted; always happens | med |
| Genesis Hydra | legends.rs:1276 | chosen | `first_max` picks the permanent from the top X | med |
| Cartographer's Survey | legends.rs:1494 | chosen | Up to two lands put in automatically | med |
| Invoke Despair | legends.rs:1530/1556 | chosen + opponent-choice | Target opponent by `max_by threat`; victim's permanent by `min_by pval` | med |
| Court of Ardenvale | legends.rs:1785 | chosen | `card_worth` max returned each upkeep | med |
| Gilgamesh, Master-at-Arms | tymna_cards.rs:949 | chosen | AI picks the Equipment to attach and the Samurai | med |
| Norn's Annex | rules2.rs:134 | opponent-choice | Person attacking into an Annex: W or life auto-paid per attacker; unpayable attackers silently dropped | med |
| Farseek | rules2.rs:1845 | chosen | AI picks the land | med |
| Archangel of Tithes | rules2.rs:1251 | check | Not traced whether a human attacker gets the {1} prompt | med |
| Jocasta, Automaton Avenger | cloud.rs:846 | chosen | "May return her" always happens | low |
| Puresteel Paladin | cloud.rs:606 | check | Always draws | low |
| Knight of the White Orchid | rules2.rs:312 | chosen | `cs.first()` Plains taken | low |
| Memory Jar | legends.rs:1710 | check | Offered in main phase 1 only | low |
| Resplendent Angel | rules2.rs:1610 | check | Pump offered in main phase 1 only | low |
| Sakura-Tribe Elder | rules2.rs:249 | check | Block-time sac guarded for humans; person has the ability | low |
| Dark Endurance | tymna_cards.rs:1945 | check | "{1} less" discount for a person's cast not verified | low |

**Fine:** galadriel.rs (Galadriel's alliance modes, Door of Destinies, Patchwork/Vanquisher's Banner, Secluded
Courtyard, Abduction, Bribery, Eerie Interlude, Flicker, Planar Genesis, Return to Dust, Mangara, Tocasia's Welcome,
Voice of Resurgence, Knight of the Holy Nimbus, Springleaf Drum, Unbreakable Formation, Maskwood Nexus, Panharmonicon,
Elvish Archdruid; Ramosians, Lin Sivvi, Lawbringer, Lightbringer, Whipcorder, Mirror Entity listed in human/cards.rs);
legends.rs (Tymna, Kaldra pieces, Plaza colours, Unclaimed Territory); cloud.rs (Cloud's ETB tutor, Open the Armory,
Fireshrieker, Andúril, Austere Command); tymna_cards.rs (Kraum, Black Widow, Falcon & Redwing, Elenda, Firebird,
Lightning); rules2.rs (Unquestioned Authority, Baneslayer Angel).

**Not finished:** combos.rs (AI combo runner; Thassa's Oracle not in tsv), t0_giada.rs (no tsv cards; grepped only),
tymna_cards.rs non-tsv cards; every "check" depends on the DSL's permanent_abilities / cards::more_abilities (skimmed);
Unclaimed Territory via choose_type (galadriel.rs:291) not confirmed.
