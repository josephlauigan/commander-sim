# Report: ai-rules (Phase C2, 2026-11-05)

Branch ai-rules (from origin/main 336b44f5). Brief: `rust/agents/ai-rules.md`. The agent returned this report as text;
the coordinator saved it. No sims by the agent; the coordinator's checks are at the end.

## Per item
1. **Council's Judgment (AI vote).** Each player votes for a nonland permanent the caster doesn't control; all tied for
   most are exiled. An AI voter now picks from all legal permanents, its own included, by `judgment_score` (another
   player's permanent: value x (1 + 0.3 x that player's threat); its own: minus value minus 1, so only when nothing else is
   legal). The caster can't vote for its own. Votes logged. Person's vote untouched. `impls/rules.rs`. Tests:
   `judgment_an_ai_opponent_may_vote_for_its_own_permanent`, `..._votes_for_another_players_permanent_when_it_can`,
   `judgment_only_the_permanent_with_the_most_votes_is_exiled`.
2. **Liliana of the Veil -6.** Piles now hold lands and the counted tokens (Treasure, Clue, Food, Blood) as well as nonland
   permanents. The caster splits heaviest-first, each item to the lighter pile; land ~1 (plus colours, mana, `pval`),
   Treasure 0.6, Clue 0.4, Food and Blood 0.2; the AI chooser sacrifices the pile worth less to it; the person gets the
   same piles ("Swamp x3"). `impls/rules.rs` (`Stuff` helpers, `lotv_ult`); `impls/t3_korvold.rs` `sac_given_land` (Liliana's
   lands fire sacrifice triggers and Tergrid). Tests: `liliana_of_the_veil_the_piles_hold_the_lands_too`,
   `..._the_ai_chooser_keeps_the_pile_worth_more`, `..._the_person_sees_the_lands_in_the_piles`; `cards_t3` and
   `practice_opponent_fixes` pile tests updated (piles no longer alternate or equal in size).
3. **Liliana, Dreadhorde General -9.** `ldg_keep` keeps one permanent per type present (artifact, creature, enchantment,
   planeswalker, land), one permanent may cover several, searching the best few per type for the most valuable set; an
   artifact land counts for both; token kinds are artifact candidates (one stays). Follow-up: the person's `ldg_person` now
   offers the counted tokens and artifact lands as artifact choices too. Tests: `liliana_dreadhorde_*` (4);
   `model_gaps_a3::mass_sacrifice_of_lands_leaves_no_card_for_a_token` updated (Treasure Vault and Darksteel Citadel are
   artifact lands: one may stay as the artifact, a second as the land).
4. **Chain of Vapor copy chain.** `chain_links`: `chain_person` and the new `chain_ai` return the copy's target's
   controller, who is offered the next copy; each link costs a land; a cap of 40 is a guard. AI copy rule: 5+ lands; the
   best opposing nonland permanent worth 5+, or (its own turn, it bounced its own permanent) another own permanent with an
   enter effect if it can afford to recast both; sacrifice through `sac_given_land`. The caster can now copy. Pilot guide
   (`decklists/JD/tymna-thrasios-doomsday.md`): Chain as protection (bounce your own Oracle and recast) and an answer;
   warns "don't cast mid-combo: the opponent copies it and bounces your Oracle": matches the new AI. `impls/partials.rs`.
   Tests: `chain_of_vapor_*` (4).
5. **Maelstrom of the Spirit Dragon.** `maelstrom_options` (land_options hook): 6+ lands, {4} from other sources, a Dragon or
   changeling card in the library with mana value 4..land count (`body_worth`); end-of-turn window utility 1.4; second main
   phase only with 7+ lands, 0.7; never before combat. Pays {4}, taps and sacrifices as a cost, `tutor_named` (Opposition
   Agent, logging). `impls/t45_urdragon.rs`. Tests: `maelstrom_*` (3).
6. **Loran of the Third Path.** Target: the opponent (alive, not hexproof) a card helps least (`card_use`: high for an
   empty hand with mana, lower for a full hand or few lands, -2 for a library of 6 or less; +0.02 x threat). Fires only with
   its own library 8+, hand under 7, and a card worth at least as much to it as to the target; utility 0.6 + 0.2 x gain;
   `loran_go` rechecks the target. Timing kept at the end-of-turn window (it untaps right after, so the draw is free; on its
   own turn it would stay tapped as no blocker). `impls/partials.rs`. Tests: `loran_*` (3).
7. **Ramosian Revivalist.** Target chosen on activation and stored; not activated if the card has left the graveyard;
   nothing happens at resolution if it's gone. `galadriel.rs`. Tests: `revivalist_*` (2).
8. **Forensic Gadgeteer.** Every hand-coded AI artifact activation now goes through `t45_urza_yawgmoth::artifact_cost`
   (floor of one mana, a coloured pip stays coloured): Food, Blood, Clue, compiled (DSL) artifact abilities
   (`pay_ability_cost`, not {X}), equip (`cloud.rs`, not Powerblade's coloured case), Isochron Scepter, Birthing Pod,
   Retrofitter Foundry, Maskwood Nexus, Jester's Cap; follow-up sweep: Credit Voucher, Crystal Chimes, Mirage Mirror, Mind
   Stone, The Mind Stone, Expedition Map, Endless Atlas, Idol of Oblivion, Batterskull, Unstable Obelisk, Oblivion Stone,
   Tome of Legends, Manifold Key, Apple of Eden, Jeweled Amulet, Wishclaw Talisman, Soul-Guide Lantern, Midnight Clock,
   Sensei's Divining Top, Patriar's Seal, Thousand-Year Elixir, Gingerbrute, Candelabra of Tawnos, Grim Monolith. Left alone:
   triggered or pure-coloured costs (Nim Deathmantle, Mana Vault's upkeep, Sunforger), costs with no mana (Witch's Oven,
   Memory Jar, Baubles, Triskelion), graveyard/hand abilities, Walking Ballista's {4} (no ordinary AI option), Shadowspear
   and Hope of Ghirapur ({1}: unchanged by the floor). Tests: `gadgeteer_*` (8).

## Decks and tiers affected
- Council's Judgment: brago (t2) casts it; the vote logic applies to every AI at that table.
- Liliana of the Veil: tergrid (t3); LDG: teysa (t1), yawgmoth (t5).
- Chain of Vapor: thrasios (JD), kinnan, kraum-tymna, rograkh-silas, rograkh-thrasios (t6), and every opponent of those.
- Maelstrom: miirym (t5). Loran: yshtola (JD), brago (t2). Revivalist: galadriel (Avery). Gadgeteer: urza (t5) only.
- Equip pricing in `cloud.rs` is shared with the person (as the Oracle says).

## Open, for JD
- Chain of Vapor's AI "re-cast its own" copy is a narrow rule (own turn, own permanent with an enter effect, both recasts
  affordable).

## Shared-file changes
`impls/rules.rs`, `impls/partials.rs`, `impls/t3_korvold.rs`, `impls/galadriel.rs`, `impls/t45_urdragon.rs`,
`impls/cloud.rs`, `dsl/mod.rs` (`pay_ability_cost`, AI only), `human/cards.rs` (comment), and for the sweep `alela.rs`,
`common.rs`, `karn_engines.rs`, `karn_cards.rs`, `sheoldred.rs`, `t0_stella.rs`, `t0_zurgo.rs`, `t6_mana.rs`, `t6_green1.rs`,
`rules2.rs`, `topdeck.rs`, `t45_krenko.rs`, `t45_winota.rs`, `ai/decks.rs`, `t2.rs`, `t4.rs`, `fixes.rs`, `yshtola.rs`,
`t45_edgar.rs`. Tests: `tests/ai_rules_c2.rs` (30).
