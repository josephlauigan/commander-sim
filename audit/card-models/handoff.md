# Card models, part 1: handoff

Brief: `rust/agents/card-models-1.md`. Branch `card-models-1`. Tests for JD's cards: `rust/crates/sim-core/tests/cards_jd_models.rs`.

## JD's Partial and Unmodeled cards (cardgen audit --mine, 10-30, before this PR)

| Deck | Card | Status before | What is missing | Oracle text |
| --- | --- | --- | --- | --- |
| yshtola | Venat, Heart of Hydaelyn // Hydaelyn, the Mothercrystal | Partial | **all of its card code was dead**: legends.rs registered it as "Venat, Heart of Hydaelyn", a name not in cards.json (the list spells the full DFC name), and the override was keyed the same way. Y'shtola ran it as a 3/3 with the compiled draw trigger (no once-a-turn limit), no Sundering, no Hydaelyn | Whenever you cast a legendary spell, draw a card. This ability triggers only once each turn. Hero's Sundering — {7}, {T}: Exile target nonland permanent. Transform Venat. Activate only as a sorcery. // Indestructible. Blessing of Light — At the beginning of combat on your turn, put a +1/+1 counter on another target creature you control. Until your next turn, it gains indestructible. If that creature is legendary, draw a card. |
| seph | Glimpse the Unthinkable | Unmodeled | nothing compiled ("ten" is not a number word the parser knows); the AI never casts it | Target player mills ten cards. |
| sauron | Baleful Mastery | Partial | the alternative cost {1}{B} (an opponent draws a card) | You may pay {1}{B} rather than pay this spell's mana cost. If the {1}{B} cost was paid, an opponent draws a card. Exile target creature or planeswalker. |
| kefka | Price of Progress | Unmodeled | nothing compiled | Price of Progress deals damage to each player equal to twice the number of nonbasic lands that player controls. |
| cloud | Buster Sword | Partial | the free cast of a spell with MV <= the damage | Equipped creature gets +3/+2. Whenever equipped creature deals combat damage to a player, draw a card, then you may cast a spell from your hand with mana value less than or equal to that damage without paying its mana cost. Equip {2} |
| karn | Treasure Vault | Partial | the {X}{X}, {T}, sacrifice: X Treasures ability (no {X} activation cost in the ability language) | {T}: Add {C}. {X}{X}, {T}, Sacrifice this land: Create X Treasure tokens. |
| thanos | Gemstone Caverns | Partial | the opening-hand clause | If this card is in your opening hand and you're not the starting player, you may begin the game with Gemstone Caverns on the battlefield with a luck counter on it. If you do, exile a card from your hand. {T}: Add {C}. If Gemstone Caverns has a luck counter on it, instead add one mana of any color. |
| thanos | Orim's Chant | Partial | the kicker and "creatures can't attack"; the AI never casts it | Kicker {W}. Target player can't cast spells this turn. If this spell was kicked, creatures can't attack this turn. |
| thanos | Redirect Lightning | Partial | changing the target of an ability | As an additional cost to cast this spell, pay 5 life or pay {2}. Change the target of target spell or ability with a single target. |
| thanos | Shifting Woodland | Partial | modelled as a token copy (enter triggers fire wrongly), creature cards only | This land enters tapped unless you control a Forest. {T}: Add {G}. Delirium — {2}{G}{G}: This land becomes a copy of target permanent card in your graveyard until end of turn. Activate only if there are four or more card types among cards in your graveyard. |
| thanos | Tinder Wall | Partial | {R}, sacrifice: 2 damage to the creature it blocks | Defender. Sacrifice this creature: Add {R}{R}. {R}, Sacrifice this creature: It deals 2 damage to target creature it's blocking. |
| urdragon | Dragonlord Ojutai | Partial | body only: hexproof while untapped, the combat-damage dig | Flying. Dragonlord Ojutai has hexproof as long as it's untapped. Whenever Dragonlord Ojutai deals combat damage to a player, look at the top three cards of your library. Put one of them into your hand and the rest on the bottom of your library in any order. |
| urdragon | Nicol Bolas, the Ravager | Partial | the flip into Nicol Bolas, the Arisen | Flying. When Nicol Bolas enters, each opponent discards a card. {4}{U}{B}{R}: Exile Nicol Bolas, then return him to the battlefield transformed under his owner's control. Activate only as a sorcery. // +2: Draw two cards. −3: 10 damage to target creature or planeswalker. −4: Put target creature or planeswalker card from a graveyard onto the battlefield under your control. −12: Exile all but the bottom card of target player's library. |
| urdragon | Peter Parker's Camera | Partial | copying activated abilities; new targets | This artifact enters with three film counters on it. {2}, {T}, Remove a film counter from this artifact: Copy target activated or triggered ability you control. You may choose new targets for the copy. |

## The pool (cardgen audit, 10-30)

Partial 63 slots, Unmodeled 2, mostly Tier 6 (dack 13, rograkh-thrasios 12, sisay 9, kinnan-bonder 7, atraxa-infect 7+2
Unmodeled, kraum-tymna 6, rograkh-silas 4, prosper 2, kinnan, miirym, tergrid 1 each). Not ranked yet.

## Done

- Venat: card code registered under the full name (`impls/legends.rs` VENAT), override re-keyed; status Full (its note
  was already the full Oracle). Tests: 4 in cards_jd_models.rs.

## Next (in order)

Glimpse (seph, `fill=glimpse` + seph fill plan), Price of Progress, Baleful Mastery, Orim's Chant, Treasure Vault,
Tinder Wall, Ojutai, Gemstone Caverns, Buster Sword, Bolas, Camera, Shifting Woodland, Redirect Lightning.

## Before / after runs

Baseline: origin/main's binary (copied to the scratch dir) with origin/main's cards.json / decks.json via
`COMMANDER_SIM_DATA`, `--deck X --pool all --games 2000 --seed 500000 --jobs 12`. Outputs in the scratch dir
`runs/base-<deck>.txt`.

## Gotchas

- `r.card(db, NAME)` under `if db.id(NAME).is_some()` silently skips a card whose name is spelled differently in the
  data (Venat). A double-faced card in a deck list is in cards.json under the list's spelling.
- testkit `hand`/`perm` take the card out of the library: measure library sizes after taking.
