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

(Filled in with the code: section 2 below is current as of the final commit.)
