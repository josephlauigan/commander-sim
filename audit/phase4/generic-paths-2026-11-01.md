# Generic play paths for the cards a deck-key gate held back (2026-11-01)

Brief: `rust/agents/generic-paths.md`. Branch `generic-paths` (on `hand-ai-delete` + `plan-files-3`). Deleting the hand
AI of eleven decks (`audit/phase4/hand-off-design.md`, "Deleted") left some cards with no way to be played by the
generic policy: their only play path was behind a deck-key gate. This note is the audit, what was done per card, what
is left, and which decks and tiers each change reaches. **Every change below moves default play for the decks
listed** (that is the point); none was simulated here (cargo tests only). Tests: `tests/generic_paths.rs`.

## How the audit was done

- Every `hand_key(..)` site in `rust/crates/sim-core/src` (80; `ai/search.rs` excluded, out of scope), classed as a
  routing site (the gate closed, the generic branch it falls to still plays the card) or a play path (the gate closed,
  nothing else offers the play).
- Every function and call site the deletion removed (`git show 60d9022c c57928a1`): what each did, card by card, and
  whether a generic path still reaches that card.
- The retired-seat gates (`"kraum"`, `"sephfs"`, `"seph"`...): the kraum seat shared its arms with thrasios until the
  deletion, so its gates were checked against the live decks' lists; the others name cards no live deck plays.
- The cards the generic policy never casts by itself (`brain::SPECIAL` tags: Rean, Fill, Yawg, Avarice, Mastery,
  Crackle, Xdrain) and the Equipment with no compiled equip ability, against every list in `data/decks.json` (`mine`
  and `pool`).

The brief's list was complete for what it named. The audit found these besides: Exsanguinate in Sauron's list; the
four protection spells of Galadriel's answer (Unbreakable Formation, Make a Stand, Rootborn Defenses, Eerie
Interlude), Sauron's (Slip Out the Back, Not of This World) and Alela's (Angelic Shield); Alela's Lightning Greaves
move; four Equipment no deck but a hand AI ever equipped (Conqueror's Flail, Sword of Feast and Famine, Sword of the
Animist, Nim Deathmantle); Ephemerate's and Momentary Blink's value blinks; Dark Deal and Scavenger Grounds outside
their own deck. Deflecting Swat's exclusion was moot (no Veyran or Kefka list holds it).

## The audit list and what was done

| Card | Decks holding it (tiers) | The gate it sat behind | Done |
| --- | --- | --- | --- |
| Ballista Squad | galadriel (t1-t2) | `brain::attack_response`: `hand_key == "galadriel"` → `galadriel::attack_answers` (deleted) | answer to an attack, any deck |
| Lawbringer, Lightbringer | galadriel | the same (their own-turn exile, `bringer_opts`, was always generic) | answer to an attack, any deck |
| Errant Doomsayers, Whipcorder | galadriel | `stack::step_priority`: `hand_key == "galadriel"` → `galadriel::precombat` (deleted) | tap before another player's combat, any deck |
| Mishra's Factory, Mishra's Foundry | karn (t4-t5) | `brain::attack_response`: `karn::block_response`, `is_deck(hand_key)` (deleted) | animate to block, any deck |
| Jace's Archivist | sauron (t3-t4) | `decks::sauron_options` (deleted) | card options, any deck |
| Exsanguinate | sauron (sheoldred, yshtola have theirs) | `yshtola::x_option`: the sauron arm (deleted) | `plays` in `data/plans/sauron.toml` |
| Underworld Breach | kefka (t3), prosper (pool t4); thanos, kraum-tymna, rograkh-silas (t6) | `kefka::kefka_prio` and the kraum seat's `kraum_prio` | priority rule (not Tier 6); the storm line is left |
| Yawgmoth's Will | kefka, ghave (t3-t4), rograkh-silas (pool t6) | `kefka::kefka_options` → `will_options` (deleted) | hand options, any deck |
| Susurian Voidborn (cast from exile) | kefka | `kefka::kefka_options` (deleted) | `generic_paths::exile_options`, any deck |
| Mizzix's Mastery | veyran (t1-t3), stella (pool t0) | `t0_stella::mastery_options`: `is_main(hand_key)` keeps it from JD's decks with hand AI | nothing: the gate reads Veyran as generic now (test) |
| Blossoming Defense | thrasios (t5) | `tymna_cards::blossom_protect`, thrasios arm of `combat_trick` (deleted) | protection response; the combat-trick pump is left |
| Unbreakable Formation | galadriel, isshin (pool t1) | `galadriel::galadriel_protect` / `galadriel_wipe_response` (deleted); not in the pool's protectors | protection and wipe response, any deck |
| Make a Stand, Rootborn Defenses | galadriel | the same | protection and wipe response, any deck |
| Eerie Interlude | galadriel, brago (pool t2) | the same | protection and wipe response, any deck |
| Slip Out the Back, Not of This World | sauron | `protect_main`'s sauron arm, `mine::veyran_protect` (deleted) | protection response, any deck |
| Angelic Shield | alela (t1) | `alela::alela_protect` (deleted) | protection response, any deck |
| Lightning Greaves (the move for an Aura) | alela (and 24 more lists) | `alela::greaves_options`: `hand_key == "alela"` (emptied) | card options, any deck, any wearer |
| Conqueror's Flail | thrasios, urdragon (t5) | the kraum seat's `kraum_special` (retired), `decks::sauron_equip` (deleted) | generic equip |
| Sword of Feast and Famine | urdragon, aurelia (pool t3); cloud keeps its own | `sauron_equip` (deleted), Cloud's hand AI | generic equip (not for a kept hand AI) |
| Sword of the Animist | isshin, aurelia | `sauron_equip` (deleted) | generic equip |
| Nim Deathmantle | ghave | the kraum seat's equip value only | generic equip |
| Ephemerate (value blink) | ghave, brago; dack (t6) | `mine::ephemerate_value`: `is_main(hand_key)`: no deck holding it ever opened it | hand options, any deck but Tier 6 |
| Momentary Blink (value blink) | brago | `mine::momentary_blink_value`: the same | hand options, any deck but Tier 6 |
| Deflecting Swat | sauron and 10 pool lists | `decks::redirect_response` left it to the veyran and kefka seats' own (deleted) answers | exclusion dropped (no default change: neither list holds it) |
| Whispersilk Cloak on the Army | sauron | `sauron_equip` (deleted) | nothing: the pool's equip still equips the Cloak; only the Army preference went (deck tuning, not a missing path) |
| Dark Deal | tergrid (pool t3); sheoldred has its own | `sheoldred::plays_it` (sheoldred's key or a plan's `plays`) | left (see below) |
| Scavenger Grounds | sauron, captain-america (pool t0) | `decks::sauron_grounds_response`, against the retired seph seat only | left (see below) |

Checked and not on the list (the gate closed, a generic branch still plays the card): the `is_main(hand_key)` routing
in `brain`, `decks` (protect, wipe, tutor), `combos::combo_options` (open for every deck without hand AI),
`dsl::run`'s sacrifice fodder, `cardcode::defend_hooks`, `common::aristocrat_options` / `sac_in_response`,
`combat::vig_utility`, `partials::tidebinder_response`, `t12_brago_meren` (Buried Alive), `t45_urza_yawgmoth`; Crackle
with Power (`brain::extra_options`), the reanimation and Fill cards (`pool::special_options`), Profane Tutor (its
suspend kept a generic utility), Kefka's flip. The `cloud`, `yshtola` and `sheoldred` gates open by default (hand AI
on) and their cards are in those lists only, except Exsanguinate (above) and Dark Deal.

## What was done, per card

- **Ballista Squad, Lawbringer, Lightbringer** (`impls/galadriel.rs` `attack_answers`, `answer`, `ballista_use`;
  `brain::attack_response`): restored from the deleted code without its deck gate. Attacked, the defender weighs each
  answer (the attacker's value and power; Ballista: less for a big X) against its removal spells and uses one when it
  is worth it (more readily when the attack is dangerous). Ballista kills an attacker whose toughness it can pay as X
  ({X}{W}, {T}; not indestructible, not damage-proof, not protected from white); Law/Lightbringer exile a red / black
  attacker ({T}, sacrifice). Oracle: "target attacking or blocking creature" / "target red (black) creature".
- **Errant Doomsayers, Whipcorder** (`galadriel::precombat`, from `stack::step_priority` at another player's
  beginning of combat, every AI seat): tap the active player's creature that would attack hardest (power 3+, able to
  attack; Doomsayers: toughness 2 or less), if the cost ({W} for Whipcorder) can be paid.
- **Mishra's Factory, Foundry** (`karn::block_response`, first in `brain::attack_response`): attacked for 40% of its
  life (or 4+ at 12 or less), the defender animates its untapped Factories / Foundries ({1} / {2}, paid with other
  sources) up to one blocker per attacker; the block code then uses them.
- **Jace's Archivist** (`impls/generic_paths.rs`, an `options` hook): {U}, {T} on the owner's turn in a main phase,
  once a turn, when it pays: Orcish Bowmasters, Sheoldred, the Apocalypse or Kaervek the Merciless out (each draw or
  cast the wheel hands the table hurts them), Sauron out with p's hand at 2 or less against a hand of 4+, or p's hand
  at 1 or less against a hand of 4+. Sauron's deck-only reasons (an Army to grow) are card conditions now. Its plan's
  line "Orcish Bowmasters + Jace's Archivist (the wheel pings)" had no way to fire before.
- **Exsanguinate** (Sauron): `plays = ["Exsanguinate"]` in `data/plans/sauron.toml`, so `yshtola::x_option` offers
  the X drain (for a kill, or a big X late) as it did for Sauron's hand AI. `tests/plan_files_3.rs` allows it.
- **Underworld Breach** (`pool::generic_prio_base`, before the derived values; not Tier 6): the kraum seat's rule
  (`tymna_partners::breach_prio`, now public): 72 on p's own turn when a graveyard card (4+ cards in the yard) can be
  escaped with the mana left after Breach, else 0 (Breach is sacrificed at the end step). The escapes themselves were
  always generic (`mine::breach_gc_options`). Tier 6 keeps its own Breach lines (`t6_combos_extra`).
- **Yawgmoth's Will** (`generic_paths`, `hand_options`): Kefka's `will_options` as written for "a deck that isn't
  built around it": on p's turn, with {2}{B}, when the yard holds a storm card and the mana to cast it after, or three
  spells whose costs fit in the mana left. The replay from the graveyard was already generic (`brain` main options).
- **Susurian Voidborn** (`generic_paths::exile_options`, called by `brain::hook_options`): the warped Voidborn
  (exiled by its warp: the `voidborn_warped` flag) is cast from exile for its mana cost on a later turn, in p's main
  phase. Unlike the deleted code it needs the flag: a Voidborn exiled by removal is not castable (Oracle: warp only).
- **The protection spells** (`ai/pool.rs` `PROTECTORS`, eight entries appended, so the old entries keep their order
  and index): Unbreakable Formation, Make a Stand, Rootborn Defenses ({2}{W}, indestructible: destroy and damage, also
  against wipes), Eerie Interlude ({2}{W}: a nontoken creature, or every one against a wipe, exiled until the next end
  step: new `How::Interlude`), Blossoming Defense ({G}: a creature against any targeted removal: `How::Hexproof`),
  Slip Out the Back ({U}: a creature phases out with a +1/+1 counter: `How::PhaseOne`), Not of This World ({7}, free
  against removal aimed at a creature of power 7+: any targeted removal), Angelic Shield (sacrifice: a nontoken,
  noncommander creature returns to hand, not against a bounce: `How::Bounce`). `protector_effect` applies what lasts
  past the response (Formation's counters and vigilance in p's main phase, Make a Stand's +1/+0, Rootborn's populate,
  Blossoming's +2/+2, the counter and phasing, the bounce, the exile). As every protector, it is used only on what is
  worth protecting (`worth_protecting`) and only by a deck without hand AI (`protect_main`).
- **Lightning Greaves** (`alela::greaves_options`): before combat on p's turn, when an own-creature Aura in hand
  (its host pick) or Daydream wants the creature wearing the Greaves (shroud), the Greaves moves (equip {0}) to p's
  best other creature. Was Alela only.
- **The Equipment** (`dsl::manual_equip`, read by `dsl::equip_options`): an Equipment card with no compiled equip
  ability gets the printed equip from its tag (practice mode's table): Swords {2} (+2), Conqueror's Flail {2} (+1),
  Sword of the Animist {2} (+1), Nim Deathmantle {4} (+2). Not for a seat whose hand AI equips them itself (the kept
  decks, the kraum test seat); the Cloak and the Boots stay with the pool's equip, Champion's Helm with a plan's
  `[equip] extra`.
- **Ephemerate, Momentary Blink** (`mine::ephemerate_value`, `momentary_blink_value`): the `is_main(hand_key)` gate
  dropped (an AI seat, not Tier 6, whose flicker spells start its combo lines): blink p's creature with the best
  enter effect (worth 3+), kept back while it is a bomb's only protection. Their protection use was generic already.
- **Deflecting Swat**: `redirect_response` no longer leaves it to the veyran and kefka seats (their own Swat code was
  deleted). No list of theirs holds it, so nothing moves.

## What is left

- **The Underworld Breach storm line** (Kefka): the deleted `mine::breach_line` (about 740 lines: the line state, dry
  run, rituals, Brain Freeze / Grapeshot loops with Lion's Eye Diamond, Jeska's Will, Laboratory Maniac). Breach is
  now cast for a plain escape and its escapes are generic, but no generic code chains Breach + a storm card into a
  kill. Needs a catalogue combo (`impls/combos.rs`) or a line the generic `combo_options` can run; more than a third of
  this session, not attempted. Kefka's plan already names the line (`data/plans/kefka.toml`), so its tutors look for
  the pieces.
- **Blossoming Defense as a combat trick** (Thrasios's deleted `combat_trick` arm: +2/+2 on an attacker that would die
  in its block). The generic policy has no combat-trick step; the kraum seat's (Dark Endurance) is seat-gated.
- **Dark Deal for Tergrid** (pool t3): its only play is `sheoldred::deal_options`, which wants Sheoldred, the
  Apocalypse out (each redrawn card costs an opponent 2). For Tergrid the payoff is different (discarded permanents
  come to her); it needs its own valuation, so the gate was left.
- **Scavenger Grounds** (sauron, captain-america): no AI code for its ability at all; its one use was Sauron's answer
  to the retired seph seat's reanimation. A generic graveyard-hate response (answer a reanimation spell or a big
  escape) would cover it.
- **The protection cards in a main phase**: Make a Stand, Blossoming Defense and Eerie Interlude carry no `prot` tag,
  so the generic priority may still cast them as plain spells in a main phase. Tagging them is a `card_overrides.toml`
  + `cardgen export` change, left out to keep the data unchanged.
- Not deck-gated, found on the way: Starbreach Whale and Starwinder (`blue_cards`) warp but are never cast from exile
  (no list holds them now).

## Which decks and tiers each change reaches

| Change | JD decks (their tiers) | Pool decks (tier) |
| --- | --- | --- |
| Ballista / Bringers / tappers | galadriel (t1-t2) | — |
| Factory blocks | karn (t4-t5) | — |
| Jace's Archivist | sauron (t3-t4) | — |
| Exsanguinate `plays` | sauron (t3-t4) | — |
| Breach priority | kefka (t3) | prosper (t4) |
| Yawgmoth's Will | kefka (t3), ghave (t3-t4) | rograkh-silas (t6) |
| Voidborn from exile | kefka (t3) | — |
| Protection spells | galadriel (t1-t2), sauron (t3-t4), alela (t1), thrasios (t5) | isshin (t1), brago (t2) |
| Greaves move | alela (t1) | any Greaves list with an own-creature Aura or Daydream |
| Equipment | thrasios, urdragon (t5), ghave (t3-t4) | isshin (t1), aurelia (t3) |
| Ephemerate / Momentary Blink value | ghave (t3-t4) | brago (t2) |

To re-run: galadriel and alela (t1-t2), sauron, kefka and ghave (t3-t4), karn (t4-t5), thrasios and urdragon (t5);
and, as tables they sit at, the JD decks whose tiers seat isshin (t1), brago (t2), aurelia (t3), prosper (t4) and
rograkh-silas (t6: thanos). yshtola, sheoldred and cloud play their hand AI by default and are unchanged (the
Equipment fallback and the protectors skip them; their own cards are untouched).
