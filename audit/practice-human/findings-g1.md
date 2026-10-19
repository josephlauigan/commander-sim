# Findings: g1 (mine.rs, common.rs, smaug.rs, mod.rs, newcards_b.rs, garland.rs, topdeck.rs, marchesa.rs, tymna_partners.rs, sweepers.rs, bites.rs, garland_cards.rs, threats.rs, sephfs.rs)

20 rows: 6 high, 7 med, 5 low, 1 check (plus a fine list). Paths under rust/crates/sim-core/src/impls/.

| Card | Decks | File:line | Kind | What happens for the person | What the card says | Sev |
|---|---|---|---|---|---|---|
| Fact or Fiction | kraum, sauron | common.rs:1949 `fof` | chosen / opponent-choice | Reveals 5, ranks by `card_worth`, best 2 to hand, rest to graveyard; no piles, no opponent split, no pile pick | An opponent separates into two piles; you choose one for hand, the other to graveyard | high |
| Comet Storm | sauron | threats.rs:324 `comet_resolve` | chosen | `comet_targets` (AI scoring) picks all targets; the person's targets ignored; no scored target → first opponent | Caster chooses any targets (one per kick) | high |
| Inferno Titan | sauron | sweepers.rs:213 `titan_damage` (etb 300, attack 307) | chosen | AI divides the 3 damage (lethal face, killable perms, walker or lowest-life opp); no choice of targets, split or decline | Divide 3 damage among up to three targets (ETB and each attack) | high |
| Smaug the Magnificent | kraum | smaug.rs:41 `smaug_attack` | chosen | X damage to a killable player, else best killable creature/walker, else lowest-life opp; no target prompt | X damage to any target (X = Treasures) | high |
| Triple Triad | veyran | mine.rs:4609 `triad_upkeep` | skipped / chosen | After the trigger window every playable exiled card is auto-played (lands, permanents, spells) | Each player exiles top card; you may play them, choosing which | high |
| Torment of Hailfire | opponents (not JD) | threats.rs:176 `torment_resolve` | opponent-choice | When the person is the opponent, `torment_pick` decides discard / sacrifice / 3 life for them, X times | The opponent chooses each time | high (person as opponent) |
| Grapeshot | kefka | mine.rs:933 | chosen | Every copy's 1 damage goes to the lowest-life opponent; never creatures/walkers/a chosen player; copies get no new targets | Each copy: 1 damage to any target | med |
| Brain Freeze | kefka | mine.rs:914 | chosen | `ctx.mill` set only by the AI plan (nothing in human/); the person's copies all mill the biggest library | Target player mills 3 | med |
| Ral and the Implicit Maze, ch. II | sauron | sweepers.rs:334 `ral_run` n=2 | chosen (may removed) | With a non-empty hand the person always discards (the pick is theirs, no decline) | You may discard; if you do, exile top two | med |
| Bojuka Bog | seph | common.rs:2333 `bog` | chosen | AI picks `worst_gy`; the person can't choose the player (or none) | Exile target player's graveyard | med |
| Esper Sentinel | yshtola | common.rs:654 `sentinel` | opponent-choice | When the person casts into it, code pays {X} itself if it can afford X+2 spare mana; no prompt | Draw unless that player pays {X} | med |
| Dauthi Voidwalker | kraum | common.rs:2375 `dauthi_play` | unreachable (partly) | Only one option: the best void-counter card, mana value 4+, main phase; the person can't pick another | Choose a card exiled with a void counter, play it | med |
| Kefka, Ruler of Ruin {8} | kefka (opp side) | mine.rs:800-826 `kefka_ruin_go` | opponent-choice | Opponent's sacrifice is `min_by(pval)`; a person on the receiving end doesn't choose | Each opponent sacrifices a permanent of their choice | med |
| Proliferate (Vraska 0, Bahamut saga) | seph, veyran | common.rs ~2800 | chosen | Everything with counters incremented; no subset | Choose any number of permanents/players with counters | low |
| Vision, Synthezoid Avenger | check | mine.rs:363 `vision` | check | Phase out vs +1/+1 counter picked automatically | Controller's mode choice | low |
| Thousand-Year Storm copies | veyran | mine.rs:2642 `storm` | check | Copies reuse the original ctx; no new targets | May choose new targets for each copy | low |
| Kindred Discovery | galadriel | mine.rs:244 `kindred_match` | check | Type read from DataKey::Ctype (default "orc"), set only in galadriel.rs:219 | Choose a creature type as it enters | check |
| Marvin, Murderous Mimic | kefka | newcards_b.rs:95 | check | Options hook reachable via abilities_of; inner choices of dsl::activate_borrowed unread | | low |
| Altar loop (sephfs) | sephfs | sephfs.rs:353-382 | check | AI option only; abilities.rs::loop_abilities requires deck key "seph", so no "start the loop" for sephfs (can run by hand) | | low |

**Fine:** Cecil, Lightning (mandatory), Coalition Relic, Notion Thief, Accursed Marauder, Hardbristle Bandit, Bugenhagen,
Tenured Concocter, Paladin of Predation, bite spells and Chelonian Tackle (the person picks both targets; AI fallback
only when none given), Dreadhorde Invasion, Dualcaster Mage (human::cards::copy_in_response), Unsummon, Potioner's
Trove, Iron Man, War Machine, Urabrask, Triskelion, Mind Stone, Dread Return flashback (human.rs:867), Hero of
Bladehold, Carrier Thrall, Pawn of Ulamog, Kaervek (choices::deal_damage), Scarlet Witch, Displacer Kitten, Nim
Deathmantle (pay_tax), Alania, Prismari Charm, Expressive Iteration, Flow State, Stock Up, the Kefka wheel.

**Not checked / skimmed:** mine.rs 2243-2640 and 3826-4400 (strip, necro_flash, muldrotha, ral, prepared_opt,
twinflame, Ephemerate/Blink value hooks, Bahamut saga, Docent, Fiery Emancipation: options hooks with no human early
return, probably reachable, bodies not read for AI-only picks); `ai::topdeck::scry` (Track Down, Betrayal ...) not
confirmed to ask the person; topdeck.rs human side; tymna_partners.rs, garland*.rs, mod.rs, sephfs.rs priority code;
Lazotep Reaver and other common.rs staples outside the JD list.
