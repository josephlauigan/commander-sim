# Report: opponent-effects probe (2026-11-04)

Saved by the coordinator from the agent's final reply (the agent's file write was refused). Test:
`rust/crates/sim-core/tests/practice_opponent_probe.rs` (env: `PROBE_ONLY="Card"`, `PROBE_LOG=1`, `PROBE_VERBOSE=1`).

## Counts
31 opponent decks, 1,422 unique cards, 262 probed (4,643 runs, a fresh table per card and moment). 46 flagged: 29 TO_FIX,
17 FALSE_ALARMS. 0 unprobed; 21 probed cards have an activated ability the AI is never offered. Asking correctly: Rhystic
Study, Mystic Remora, Esper Sentinel, Smothering Tithe, Oppression, Propaganda, Ghostly Prison, Archangel of Tithes,
Sheoldred Whispering One, Liliana of the Veil +1/-2, The Eldest Reborn, Plaguecrafter, Torment of Hailfire.

## Table and moments
P0 (person; first choice / yes; passes): 4 creatures (power 0-3), 3 artifacts, 3 enchantments, 2 planeswalkers (loyalty 30),
10 lands, 5 cards in hand, 6 in graveyard, floating mana and untapped lands. P1 = the pool deck's AI seat, owns the card.
Instants/sorceries cast by P1 (X=3); permanents cast, then 20 moments (P0 casts noncreature / creature / instant; P0 draws;
P1 and P0 upkeep and end step; P1 main phases; P1 attacks P0 with and without walkers; a P1 creature dies; the card dies; P1
casts a spell; a land enters for P1 / P0; P0 attacks P1 through `attack_limits`; every activated / hand ability the AI is
offered, loyalty ultimates included). Flagged: P0 lost permanents / hand / graveyard cards (2+ candidates, not all) or paid a
tax, the text fits, and P0 was asked nothing; also the same question twice in a row.

## TO_FIX (29), by root cause
| Root cause | Cards (pool decks) | Where | Fix | Size |
|---|---|---|---|---|
| DSL "sacrifice" picks `min_by(pval)` for the person | Butcher of Malakir, Dictate of Erebos, Grave Pact, Demon's Disciple, Fleshbag Marauder, Liliana's Triumph, Mire in Misery, Vraska's Fall, Sheoldred's Edict, Rankle's Prank, Flare of Malice (teysa, meren, yawgmoth, tergrid, atraxa-infect) | `dsl/mod.rs:1016-1041` (pick :1038) | `human::choices::sacrifice_creature` / `sacrifice_permanent` for a person | S |
| DSL "discard" discards at random for every player but the controller | Nezumi Informant, Virus Beetle, The Raven Man, Vicious Rumors (tergrid) | `dsl/mod.rs:972-987` | `cast::discard_worst` for a person (routes to them) | S |
| `discard_worst_for` | Burglar Rat, Elderfang Disciple (tergrid) | `impls/t3.rs:1567` | `choices::discard` for a person | S |
| Rats: least-worth card on top | Chittering Rats | `impls/t3.rs:1589` | `choices::pick_cards` | S |
| keeps the best two creatures | Archfiend of Depravity (kaalia) | `impls/t3.rs:1656` | `pick_cards` for the two kept | S |
| sacrifices lowest-value artifact/creature/land | Braids, Cabal Minion (tergrid) | `ai/decks.rs:929` (from `engine/turn.rs:564`) | candidates + `sacrifice_permanent`; a land via `remove_land` | M |
| the person's vote cast by the AI | Council's Judgment (brago) | `impls/rules.rs:213` | `human::cards::pick_perm` | S |
| AI picks the person's pile | Liliana of the Veil -6 (tergrid) | `impls/rules.rs:843` | `choose_one` between piles | M |
| keeps the best of each type | Liliana, Dreadhorde General -9 (teysa, yawgmoth) | `impls/rules.rs:882` | `pick_perm` per type | M |
| discard or 5 life decided for the person | Painful Quandary (tergrid) | `impls/rules.rs:1242` | `yes_no` + `choices::discard` | S |
| decides and picks the two sacrifices | Rakdos, Patron of Chaos (kaalia) | `impls/t12_kaalia_sythis.rs:306` | `yes_no` + two `pick_perm` | S |
| keeps the person's best creature / walker | Single Combat (light-paws) | `impls/t12_lightpaws.rs:643` | `pick_perm` over `fighters` | S |
| random discard on combat damage | Sword of Feast and Famine (aurelia) | `engine/combat.rs:918-924` | `discard_worst` for a person | S |
| pays {1} per creature or lets it die | The Tabernacle at Pendrell Vale (dack) | `engine/turn.rs:525-543` | `choices::pay_tax` per creature | S |
| pays {2} for the person | Wandering Archaic (kinnan) | `impls/t45_chulane_kinnan.rs:493` (~503) | `choices::pay_tax` | S |

Also: the same random discard in `archon_attack` (`engine/combat.rs:468-473`, Archon of Cruelty, no pool deck). Flare of
Malice's sacrifice is limited to the greatest mana value; the DSL arm ignores that (AI too).

## FALSE_ALARMS (17)
P1's own target or choice: Deathrite Shaman, Grasp of Fate, Static Prison, Riptide Gearhulk, Infectious Bite, Teferi Time
Raveler -3, Vraska Betrayal's Sting -2, Duress, Silent-Blade Oni, Dack Fayden / Helping Hand (the donated creature's enter
trigger is P1's). Mass or forced, no choice: Delayed Blast Fireball, Phyresis Outbreak, Living Death, Reality Acid. Several
identical prompts, all asked: Vona's Hunger, Torment of Hailfire (could say "1 of N"). Chain of Vapor: the bounce is P1's.

## Not driven / other findings
- Chain of Vapor's "may sacrifice a land to copy" is never offered to the person (no code).
- Tergrid's Lantern's {T} ability is never offered to the AI.
- Boseiju's channel isn't offered at this table; "that player may search for a land" isn't modelled (`impls/lands.rs:372-381`).
- Kazuul's "unless pays {3}" isn't modelled.
- Daze / Flusterstorm held by P1: the AI never answered the probe's spell; `engine/stack.rs:1394-1410` asks the person.
- Attack taxes: one yes/no for the strongest k creatures; the person can't pick which attack (`engine/combat.rs:198`).
- Limits: a partly asked question, or a refusal that leaves P0's state unchanged, isn't seen.
