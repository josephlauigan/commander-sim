# Phase 4 PR 6: rule checks in the policy

Brief: `rust/agents/phase4-rules.md`. JD's plan, AI point 6: stop a class of mistakes with one generic guard each,
not one card at a time. Three guards, behind `AI_RULES` (default off: nothing can be measured until the
recalibration after Phase 4).

## 1. Survey

Cards were read from `data/decks.json` (every deck, JD's and the pool, commanders included: 2063 names) joined with
the Oracle text in `data/scryfall_cache.json` (41 double-faced names have no entry under their full name; none of
them has one of the patterns below by its front face), and with the tags and compiled abilities in `data/cards.json`.

### 1a. Follow-up payments (casting)

Where the decision is made today:

- The generic main-phase menu, `ai/brain.rs::main_options`: a compiled additional cost is dry-run
  (`engine/cast.rs::additional_cost(.., dry)`), `Needsac` (Diabolic Intent) needs `spare_creature`. `do_cast_with`
  repeats both checks. The end-of-turn window (`end_of_turn_window`) builds its own list with neither check (instants
  and flash cards only; it falls through to `do_cast`, which refuses them).
- Removal: `removal_options` skips a `Pactpay` card (Slaughter Pact) when `cast::pact_affordable` says the debt
  can't be paid at the next upkeep; `attack_response` never casts `Needsac` or `Pactpay` removal.
- Counters: `engine/stack.rs::pick_counter` casts Pact of Negation when `stack::pact_affordable` holds: five lands
  (Tiers 0-5), or five lands per pact owed and two blue lands (Tier 6). **Gap:** outside Tier 6 a Pact whose upkeep
  fails the five-land test falls through to the `{0}` `can_pay` check and is cast anyway.
- Upkeep: `cardcode.rs` (around line 930) pays `{3}{U}{U}` per `Player::pacts`, then each `Player::pact_debts`;
  a payment that can't be made loses the game.
- Gaps in the two affordability tests: Slaughter Pact's test counts the other pact debts but not the Pacts of
  Negation owed (`pacts`); Pact of Negation's counts lands, not mana (colours, a land that makes `{C}`), and not the
  Slaughter Pact debts.

Cards with the pattern, by Oracle text:

| pattern | cards in the decks | how the engine reads it |
|---|---|---|
| upkeep payment ("At the beginning of your next upkeep, pay ... If you don't, you lose the game") | Pact of Negation (thanos, urdragon, kinnan, urza, kinnan-bonder, kraum-tymna, rograkh-silas, rograkh-thrasios, sisay) | tag `pact` (cost `{3}{U}{U}` in cardcode); Slaughter Pact (in the card data, in no deck now) tag `pactpay = "2B"` |
| additional cost: sacrifice (compiled) | Corrupted Conviction, Culling the Weak, Eldritch Evolution, Harrow, Infernal Plunge, Louisoix's Sacrifice, Natural Order, Rite of Oblivion, Seize the Spoils, Silence the Echo, Village Rites (+ Bone Shards, Champions of the Perfect, Defiler of Dreams, Deprive, Fear of Isolation, March of Otherworldly Light, New Blood, Redirect Lightning, Stir Up Trouble, Thrill of Possibility, Unexpected Windfall, Vicious Rivalry, Warlord's Elite with other costs) | `Ability::AdditionalCost`, dry-run in `main_options` and `do_cast_with`: already guarded |
| additional cost: sacrifice (tagged) | Diabolic Intent | tag `needsac`: already guarded |
| additional cost: sacrifice (card code) | Eliminate the Competition (`t0_zurgo`), Lethal Throwdown (`sheoldred`), Crop Rotation and Reshape (`fixes`) | each card's own option checks its fodder |
| additional cost: discard (tagged) | Big Score | tag `discard1`, discarded as the spell resolves (`discard_worst`): **not checked**: cast with an empty hand it discards nothing |
| additional cost: discard (card code) | Grab the Prize (`kefka`), Thrill of Possibility (compiled) | own option / dry run |
| additional cost: discard or life | Bitter Triumph | tag `lose = 3` (life always): nothing to check |
| sacrifice cost not modeled | Deadly Dispute (`fill = "dispute"`: draws two and makes a Treasure, sacrifices nothing) | a model gap for the card-model work, not a guard's job (a guard can't stop a cost the engine never asks for) |
| activated abilities with a sacrifice or discard cost | compiled abilities | `dsl::ability_options` offers only what `dsl::activate`'s cost check can pay: already guarded |

So the follow-up guard's new reach: Pact of Negation (and Slaughter Pact) priced in real mana against **every**
upkeep debt, and Big Score's discard. Everything else in the table is already checked; the guard repeats those
checks in one place for the paths that lack them (the end-of-turn window).

### 1b. Discards and self-mill of a tutored card

Where p chooses what to discard or mill:

- Hand size: `engine/turn.rs` cleanup -> `ai::cleanup_discard` (keeps line pieces, the wish list, needed lands,
  protection, then the soonest castable). A tutored card is kept only if it is one of those.
- Discard effects and costs at resolution: `engine/cast.rs::discard_worst` (Big Score, looting, "discard a card":
  38 callers), with Kefka's own `kefka::discard_n`. It keeps `ai::discard_keep` (combo pieces and wish list, outside
  decks only) and otherwise discards a spare land or the highest-cost / least-worth spell: a tutored card can go.
- Compiled discard costs: `cast::additional_cost`'s `Choice::Discard` (the least `card_worth`).
- Self-mill by choice: `ai/topdeck.rs::scry` (scry to the bottom, surveil to the graveyard) partitions the top cards
  by `desire`: a card a tutor just put on top (Vampiric Tutor, Imperial Seal, Mystical Tutor, Worldly Tutor,
  Enlightened Tutor) can be binned. Mills of the top N (Grisly Salvage, Stitcher's Supplier) are not p's choice of
  card; whole-hand wheels discard everything (no choice).

Tutors (where the card goes, from `ai/planner.rs::tutor_rule` and the tags): to hand: Demonic Tutor, Diabolic Tutor,
Dark Petition, Demonic Counsel, Diabolic Intent, Eladamri's Call, Fabricate, Idyllic Tutor, Open the Armory, Profane
Tutor, Ringsight, Solve the Equation, Steelshaper's Gift, Wargate (battlefield), Beseech the Mirror, the commanders and
creatures with search text (card code); to the top: Vampiric Tutor, Imperial Seal, Mystical Tutor, Enlightened
Tutor, Worldly Tutor; land searches (Harrow, Kodama's Reach, Three Visits ...) and X tutors are not "found cards".
Resolutions in the engine: `engine/tutors.rs::tutor` (hand), `tutor_to_top` (top), `pile_tutor` (Gifts/Intuition:
the opponent picks), the compiled search (`dsl/mod.rs`), and hand-written resolutions in `impls/` (17 places
increment the `tutored` stat).

### 1c. Combo starts

- The catalogue (`impls/combos.rs::all`, 16 base lines plus the Tier 6 modules) is offered only when ready
  (`combo_options`: every piece held, mana affordable) and `attempt` re-checks: the catalogue never starts a line
  with a piece missing.
- A line is started piece by piece by the ordinary cast menu: a piece's cast priority (key cards, plan `prio` rules
  with `next_of`, the tutor wish list) puts it on the stack while another group is still in the library. For a
  permanent piece that is a development play (it waits on the battlefield: Isochron Scepter, Sanguine Bond, Helm of
  the Host). For a **spent** piece (an instant or sorcery that is gone once it resolves) it is a lost card: Dramatic
  Reversal without the Scepter, Brain Freeze without Breach, Twinflame without Dualcaster Mage, Demonic Consultation
  or Tainted Pact without Thassa's Oracle.
- The Tier 6 line planner (`ai/planner.rs`) only plays the first step of a plan that wins this turn or is ready next
  turn: it already never starts a line it can't finish.

Spent pieces in the catalogue's lines (by the card data: non-permanent and in a group): Dramatic Reversal; Demonic
Consultation, Tainted Pact; Brain Freeze; Manamorphose; Ad Nauseam; the copy spells of the Dualcaster line. Of these,
Manamorphose (makes mana, draws), Ad Nauseam (draws) and the tutors (Consultation's own search is card code, not a
`tutor_rule`) dig for the missing piece themselves; see the design.

## 2. Design

One module, `ai/rules.rs`. Each guard is one function, called from as few places as possible; every guard first
asks `rules::on(g, p, guard)`, which with `AI_RULES` unset returns false before it reads the game.

### 2a. Follow-up: `followup_ok(g, p, c)` (+ the compiled-cost dry run in `filter_options`)

A card is refused when:

- **its upkeep payment can't be made**: `upkeep_due` reads it from the card data (`pactpay = "<cost>"`, or the
  `pact` flag: Pact of Negation's `{3}{U}{U}`), and `next_upkeep_affordable` adds every debt p already owes (each
  `Player::pacts` at `{3}{U}{U}`, each `pact_debts` entry). **The next upkeep's mana**: a copy of the game with every
  land and permanent of p's untapped and no longer summoning-sick, then `mana::can_pay` on the copy (so colours,
  `{C}`-only lands, mana rocks and dorks, Treasures all count as the engine counts them). Not counted: the land drop
  (the upkeep comes before it), mana from cards in hand, rituals; not subtracted: a source p might lose before then
  (an opponent's removal), mana spent at instant speed in between. This game is not touched.
- **a tagged discard cost has nothing to discard** (`discard1`: Big Score; another card in hand is needed);
- **a tagged sacrifice cost has nothing to sacrifice** (`needsac`: `brain::spare_creature`);
- **a compiled additional cost can't be paid** (`cast::additional_cost(.., dry = true)`; menu options only, as the
  counter pick has no `&mut Game`).

Called from: `ai/rules.rs::filter_options` (over Cast, Removal, Wipe, Face and Crackle options), hooked once at the
end of `brain::main_options` and once in `brain::end_of_turn_window`; and `engine/stack.rs::pick_counter` (one
`continue`), which is where Pact of Negation is chosen. The existing checks stay (the guard only adds refusals).

### 2b. Tutored: `mark_tutored`, `tutored`, `spare_tutored`

- **Mark** (`Player::tutored`: card and turn number, the turn number being the sum of every player's `turns`, so
  extra turns count): `engine/tutors.rs::tutor` (to hand), `tutor_to_top`, `finish_pile` (the cards kept), the
  compiled search (`dsl/mod.rs`, unless it puts the card onto the battlefield or into the graveyard), and the
  hand-written tutors that put a card in hand (`impls/t6_blue.rs` Demonic Counsel and transmute, `t1.rs`'s shared
  search, `t2.rs`, `rules2.rs` Wishclaw Talisman). A mark lasts this turn and the next (`now - mark <= 1`).
- **Use**: `spare_tutored(g, p, cands)` returns the candidates without the marked cards when some are marked and some
  are not (None otherwise: the choice is as today, so with only tutored cards to choose from one still goes). Hooked
  where p chooses: `ai::cleanup_discard` (hand size), `cast::discard_worst` (discard effects, Big Score, looting),
  `kefka::discard_n` (Kefka's own discard), `cast::additional_cost`'s discard choice; `topdeck::scry` keeps a marked
  card on top (scry to the bottom, surveil to the graveyard).
- Card ids are per name: a mark on a basic land covers every copy of it (singleton decks: no other card is affected).
- Not covered: whole-hand wheels (no choice), mills of the top N (no choice), hand-written discard choices inside
  single cards' code other than Kefka's.

### 2c. Combo: `combo_ok(g, p, c)`

- A **spent piece** (`spent_piece`): an instant or sorcery in a group of one of p's catalogued lines
  (`combos::deck_lines_with`: the lines whose every group is in p's deck, quiet markers left out) that draws no card
  (`brain::draws_cards`), makes no net mana (`mana::ritual_net`) and is not a tutor (`planner::tutor_rule`). Those
  three dig for the missing piece themselves, so casting them is not a wasted start. By the card data the spent
  pieces are Dramatic Reversal, Demonic Consultation, Tainted Pact, Brain Freeze and the copy spells of the Dualcaster
  line; Manamorphose and Ad Nauseam are exempt (mana / draw).
- Refused when no line of c's can be finished: each other group must be **held** (in hand, on the battlefield or
  among the lands, a commander in the command zone, a card p put on top of its library) or **findable**: a tutor in
  p's hand (not c) whose cost p can pay now, whose `planner::tutor_targets` (the restriction it resolves with) include
  a card of that group in the library. One tutor counts for one group.
- Permanent pieces are not guarded: they wait on the battlefield (a development play, not a lost card). Hooked in
  `filter_options` with the follow-up guard.

## 3. The switch

`AI_RULES`: unset, empty or `0` = no guard (today's play, the default); `1` = all three; a list of names
(`followup`, `tutored`, `combo`, comma-separated) = those. `AI_RULES_DECKS=a,b` (optional) limits the guards to the
players of those decks: one half of a paired test. People (practice mode) are never guarded. Each guard's first step
is `rules::on`, which reads the mask before anything else, so with the switch unset every hook returns before it
reads the game, changes it or draws a random number: `filter_options` returns at once, `followup_ok` and `combo_ok`
return true, `mark_tutored` does nothing (`Player::tutored` stays empty), `tutored` is false and `spare_tutored` None.
Tests force the guards on one thread with `rules::force(Some(mask))` (as `ai/ismcts.rs`'s `FORCE`). When a guard
drops a menu option it counts `rules|followup` or `rules|combo` in the player's stats.

## 4. Tests (`rust/crates/sim-core/tests/rules_guards.rs`, 14)

- switch: parsing (unset / `0` / `1` / names; an unknown name panics); default off on the test thread.
- follow-up: Pact of Negation with two Islands is not picked as a counter (today: picked); with five tapped Islands it
  is; with a Pact already owed five are not enough (today: five lands pass); a Slaughter Pact debt counts with it;
  Diabolic Intent and Culling the Weak with no creature are filtered (switch off: kept), and offered once a creature
  is there; Big Score alone in hand is not on the menu (today: it is), and is once another card is in hand.
- tutored: hand-size discard keeps the marked card (switch off: today's pick, which was that card); `discard_worst`
  the same; with only the tutored card to choose it goes; a mark lasts one more turn, then ends; the engine's tutor
  marks what it finds (and marks nothing with the switch off); surveil keeps a card tutored to the top (today: binned).
- combo: Tainted Pact with Thassa's Oracle in the library and no tutor is refused (switch off: allowed); with Demonic
  Tutor in hand but no mana, still refused; with the mana for it, allowed; with the Oracle in hand, allowed; a card
  outside the lines and a permanent piece are never refused.

## 5. For the recalibration after Phase 4 (no sims now)

Binary: this branch's `commander-sim`, built once. First an identity check: `AI_RULES` unset against the base binary
(origin/main before this PR), `--realism --pool tN --games 300` on t1-t5, byte for byte (the hooks must change
nothing with the switch off).

1. **Generic decks, the half-tier method** (`audit/phase4/values-handoff.md`: the same halves, fixed before any run;
   2000 games per arm, seeds 500000-501999, loose): per tier t1-t5, one shared old arm (`AI_RULES=0`) and two arms
   with `AI_RULES=1 AI_RULES_DECKS=<the half's keys>`; read each half's summed share of the decisive games with
   `half.sh OLD NEW "Name;Name"` and each deck with `audit/phase4/pair.sh OLD NEW`. Level = the CI includes 0.
2. **JD's decks on their own tiers** (`data/my_decks.toml`), one deck at a time: `--deck <key> --pool <tier>`,
   `AI_RULES=0` against `AI_RULES=1 AI_RULES_DECKS=<key>`, 2000 games, same seeds; report the average over the deck's
   own tiers. The decks the guards reach most: urdragon (t5: Pact of Negation, Demonic Tutor, Dualcaster + Twinflame),
   thanos (t6: Pact of Negation, Tainted Pact, Demonic Consultation, Thassa's Oracle), kefka (t3: discard choices),
   sheoldred (t3-t4: Demonic Tutor), ghave (t3-t4).
3. **Per guard**, if a pair moves: rerun that pair with one guard at a time (`AI_RULES=followup` / `tutored` /
   `combo`) to find which one did it.

How to read them: the guards only remove options, so a loss means a refused play was better than the guard thinks
(look at `rules|followup` / `rules|combo` counts in the stats, and in a traced game the turn a Pact was held or a
discard kept). Expect level for most decks (the patterns are rare); gains, if any, on the Pact decks (a lost game to a
missed upkeep) and the tutor-heavy decks at hand size. If level everywhere: make `1` the default for generic decks
(as values and search did), keep `0` as the measurement arm.
