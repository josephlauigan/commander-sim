# Findings: g5 (t0_anikthea.rs, yshtola.rs, t0_zurgo.rs, rules.rs, lands.rs, green_cards.rs, t0_captain.rs + engine)

29 rows (card 15, engine 12, check 2): unreachable 8, chosen 14, skipped 1, opponent-choice 3, check 3;
high 4, med 17, low 8. Paths under rust/crates/sim-core/src/.

| Card / engine | Decks | File:line | Kind | What happens for the person | Sev |
|---|---|---|---|---|---|
| Everflowing Chalice | godo | impls/rules.rs:1202-1240 | unreachable / skipped | Multikicker only in hand_options; a person's cast records no DataKey::Kicks, so it taps for 0 | high |
| Fact or Fiction | kraum, sauron | impls/rules.rs:243 | chosen + opponent-choice | AI splits the piles and takes the better; the person never chooses (or splits as the opponent) | high |
| Planeswalker attacks | all | engine/combat.rs:559 | unreachable | `to_walker = vec![]` for a person: their creatures can't attack planeswalkers | high |
| Blocks / damage assignment | all | human/combat.rs:153, engine/combat.rs:600-700 | chosen | One blocker per attacker; damage and trample assignment fixed | med |
| Additional costs | Redirect Lightning, Seize the Spoils, Thrill of Possibility, Silence the Echo, Stir Up Trouble | engine/cast.rs:62-140 (from human.rs:860) | chosen | AI picks the discard / sacrifice, and sacrifice-or-pay | med |
| Legend rule | all | engine/life.rs:219 | chosen | AI picks which legend to keep | med |
| Commander to command zone | all | engine/zones.rs:559-585, 1779 | chosen | Always goes to the command zone (no choice to leave it in the graveyard/exile) | med |
| Mox Diamond | ? | engine/cast.rs:980, zones.rs:999 | chosen | Forced; AI picks the land discarded | med |
| Chrome Mox | ? | engine/tutors.rs:339 | chosen | Forced; AI picks the imprinted card | med |
| Spider-Man / Greenbelt Radical / Vinelasher Adept; Tumbleweed Rising, Lifestream's Blessing | titania | impls/green_cards.rs (hand_options) | unreachable | Web-slinging, disguise/turn up, plot, foretell, basic landcycling AI-only | med |
| Karn's Bastion | seph | land_options | unreachable | land_options never read for a person | med |
| Ghost Quarter | alela | land_options | unreachable | land_options never read for a person | med |
| Plaguecrafter | sephfs | impls/rules.rs:1073 | chosen + opponent-choice | Sacrifice min_by pval for every player, person included | med |
| Kura, the Boundless Sky | titania | impls/green_cards.rs:1054 | chosen | Mode and lands chosen by the AI | med |
| Boughside Wanderers | titania | green_cards.rs:213 via t1.rs:736 look_take | chosen | Card auto-taken; "may" gone | med |
| Mystic Sanctuary | veyran | impls/lands.rs:826 | chosen | Forced AI pick of the instant/sorcery | med |
| Springleaf Drum | galadriel | impls/rules.rs:49 drum_tap | chosen | AI picks the creature tapped | low-med |
| Carnivorous Canopy, Karn's Bastion proliferate | titania, seph | common.rs:2802 | chosen | Proliferates everything; no "any number" | low-med |
| Izzet Boilerworks | sauron | engine/turn.rs:255 | chosen | AI picks the land returned | low-med |
| Sauron's ward | sauron (opp side) | engine/removal.rs:200 | opponent-choice | AI picks the sacrifice for the person | low |
| Phyrexian Obliterator | ? | impls/rules.rs:121-150 | opponent-choice | AI picks the person's sacrifice | low |
| Eldest Reborn ch. I | opponent | impls/t0_anikthea.rs:381 | opponent-choice | Creature-only edict: the person can't sacrifice a planeswalker | low |
| "May" trigger windows (Curiosity, Gonti, Thief of Sanity, Y'shtola) | various | engine/stack.rs trigger_window | chosen | Only priority, never yes/no (always-good draws) | low |
| Big Score | ? | | check | Discard happens at resolution, not as a cost | check |
| Grab the Prize | kefka | | check | dsl null, only hand_options: can a person cast it at all? | check |
| Goad | opponents | | check | No goad enforcement found for a person's attackers | check |

**Fine:** Curiosity, Marauding Blight-Priest, Polluted Bonds, Exsanguinate, Debt to the Deathless (X asked in human.rs),
Ill-Gotten Inheritance, Urborg Syphon-Mage, Gonti, Thief of Sanity, Hostage Taker, Dark Petition, Champion's Helm,
Mystic Remora, Bloodchief's Thirst, Lotus Petal, Mox Opal, Spire of Industry, Exotic Orchard, Avenge, Ghalta, Topiary
Lecturer, Cactarantula, Studious First-Year, Crushing Canopy, Sokenzan, shockland pay-2-life.

**Not finished:** t0_anikthea.rs, t0_zurgo.rs, t0_captain.rs grepped only (Helvault, Binding of the Titans, Zurgo goad and
myriad redirect, Heroic Sacrifice); engine stack.rs (pick_counter 728), mana.rs, hooks.rs, altcost.rs, dungeon.rs,
cardcode.rs grepped only; yshtola.rs priority-table cards belong to other groups.
