# Report: opponent-effects fixes (2026-11-04)

Branch practice-opponent-fixes. Probe `practice_opponent_probe`: `TO_FIX` is empty (29 -> 0), `FALSE_ALARMS` 17 -> 14
(Vona's Hunger, Torment of Hailfire, Chain of Vapor fixed). Tests: `rust/crates/sim-core/tests/practice_opponent_fixes.rs`
(26 tests, every cause asks the person and applies the answer). Every fix is in an `is_human` branch; AI paths unchanged.

| Cause | Done | Files | Test |
|---|---|---|---|
| DSL "sacrifice" (Butcher, Dictate, Grave Pact, Demon's Disciple, Fleshbag, Mire in Misery, Vraska's Fall, Sheoldred's Edict, Rankle's Prank) | yes: `sacrifice_permanent` from the same candidates; prompt says "(1 of N)" | dsl/mod.rs | dsl_sacrifice_* |
| Liliana's Triumph (not DSL: a Rust `resolve`, the report was wrong) | yes | impls/rules.rs triumph | dsl_sacrifice_the_person_picks_the_creature |
| Flare of Malice (own impl, not DSL) | yes: only the greatest-MV creatures/walkers offered | impls/t3_korvold.rs flare_cands | flare_of_malice_* |
| Land sacrifice (Smallpox, Pox, Entish Restoration, DSL land arm) | yes (extra): person picks the land | t3_korvold.rs sac_worst_land | via Braids land test |
| DSL "discard" (Nezumi Informant, Virus Beetle, Raven Man, Vicious Rumors) | yes: non-random goes through `discard_worst`; random stays random | dsl/mod.rs | dsl_discard_* |
| Burglar Rat, Elderfang Disciple | yes | impls/t3.rs discard_worst_for | burglar_rat_and_elderfang_* |
| Chittering Rats | yes | t3.rs rats | chittering_rats_* |
| Archfiend of Depravity | yes: keeps the two the person chooses | t3.rs depravity; new `human::cards::pick_perms` | archfiend_* |
| Council's Judgment | yes: the person's own vote (any nonland permanent the caster doesn't control, own included, per Oracle) | impls/rules.rs judgment | councils_judgment_* |
| Painful Quandary | yes: yes/no, then the card | rules.rs quandary | painful_quandary_* |
| Rakdos, Patron of Chaos | yes: yes/no, two picks, else the controller draws two | t12_kaalia_sythis.rs | rakdos_* (2) |
| Single Combat | yes | t12_lightpaws.rs | single_combat_* |
| Sword of Feast and Famine, Archon of Cruelty discard | yes | engine/combat.rs | sword_*, archon_* |
| The Tabernacle | yes: pay {1} per creature, yes/no each | engine/turn.rs upkeep | the_tabernacle_* |
| Wandering Archaic | yes: `pay_tax`; declining leaves the AI's copy decision as before | impls/t45_chulane_kinnan.rs | wandering_archaic_* |
| Braids, Cabal Minion | yes: any artifact, creature or land (lands via the land list) | ai/decks.rs braids_sacrifice | braids_* (2) |
| Liliana of the Veil -6 | yes: the AI separates as before, the person picks the pile | rules.rs lotv_ult | liliana_of_the_veil_* |
| Liliana, Dreadhorde General -9 | yes: `ldg_person` keeps one permanent per type (artifact/creature/enchantment/planeswalker/land; one can serve several types) | rules.rs | liliana_dreadhorde_* |
| Attack taxes | yes: when the person can pay for only k of n creatures they pick which attack (up to k, or fewer); when they can pay for all it stays one yes/no | engine/combat.rs human_attack_tax | attack_tax_* (2); play_attack_tax.rs updated (one extra answer) |
| Chain of Vapor copy | yes: yes/no, which land, the copy's new target (or "don't copy") | impls/partials.rs chain_person | chain_of_vapor_* (2) |
| Prompts "1 of N" (Vona's Hunger, Liliana -4, Torment of Hailfire) | yes (extra): `edict_nth` in engine/zones.rs | zones.rs, t3.rs, sheoldred.rs, threats.rs | probe |

## Would change the AI
Nothing in an AI path. Notes (mismatches left as they are):
- Council's Judgment (AI): non-caster voters can't vote for their own permanents and it only counts the vote, while the Oracle lets each voter pick any nonland permanent the caster doesn't control. The person's vote follows the Oracle.
- Liliana of the Veil -6 (AI): piles contain only `perms`, never lands (Oracle: all permanents). The person gets the same piles.
- Liliana -9 (AI): keeps one permanent per first type; an artifact creature counts as a creature only. The person's version follows the Oracle (one permanent can cover two types).
- Flare of Malice: the AI already restricts to the greatest MV (no mismatch).
- Chain of Vapor: the copy is not chained further (a copy's new target's controller isn't offered a second copy), for the AI or the person; the AI's existing rule (>= 5 lands, pval >= 5) is unchanged. A person who is the caster and bounces their own permanent is offered the copy too (controller = person).
- Not touched but similar: Karn's +1 in impls/t3.rs `karn_plus` (AI picks the opponent's card to exile) is not in the probe's pool; not changed.

## Tests run (all pass)
practice_opponent_probe, practice_opponent_fixes, all `play_*` and `practice_*` files, cards_rules, cards_t3, cards_t3_korvold,
cards_t12_*, cards_sephfs, cards_sheoldred, cards_common, dsl, ai_t6_kinnan / rograkh_silas / kraum, and the other files that
name the touched cards (about 45 test binaries) plus the lib tests. Not run: the full suite and sims (the coordinator does that).
