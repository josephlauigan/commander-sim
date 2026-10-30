# Phase 3, PR 3: one replacement step (design)

Branch `phase3-repl`, stacked on `phase3-chars`. Brief: `rust/agents/phase3-repl.md`. Handoff:
`audit/phase3/repl-handoff.md`.

## The step

`engine/repl.rs`. One function per kind of event that can be replaced; each collects the replacement (and
prohibition) effects that apply to that event from one place, orders them and returns the changed event. The caller
then does the event as changed.

| event kind (`Ev`) | function                                              | what it returns                         |
|-------------------|-------------------------------------------------------|-----------------------------------------|
| `Counters`        | `counters(g, actor, on, kind, n)`                     | how many counters are really put        |
|                   | `put_counters(g, actor, m, n)`, `give_poison(g, actor, q, n)` | the same, and puts them         |
| `Enter`           | `enter_counters(g, p, cd, plus, loyalty, undying)`, `enters_tapped(g, p, m)` | counters / loyalty it enters with, tapped |
| `Tokens`          | `tokens(g, p, n)`, `artifact_kinds(g, p, kind)`       | how many tokens, which kinds            |
| `Damage`          | `perm_damage_prevented(g, src, to, combat)`, `player_damage_prevented(g, p, src)` | prevented or not |
| `LifeGain`        | `life_gain(g, p, n)`                                  | the life really gained (0: can't)       |
| `LifeLoss`        | `damage_to_player(g, p, n, src, damage, combat)`      | the damage / loss after doublers        |
| `Graveyard`       | `to_graveyard(g, p)`                                  | `Gy` or `Exile` (Rest in Peace)         |
| `Draw`            | `draw_denied`, `draw_wins` (Lab Man), the draw replacements in order | what replaces the draw   |
| `Trigger`         | `trigger_count(g, p, kind, x)`                        | 0 (Hushbringer) or 1 + copies           |

`on` is `On::Perm(m)` or `On::Player(q)`; `kind` is `Ctr::Plus` (+1/+1, negative n is -1/-1), `Ctr::Loyalty`,
`Ctr::Poison`.

## The table

`repl::NAMED`: the cards whose replacement effect the engine knew by name, each with its event kind and effect
(`ReplFx`), like `chars::NAMED`. It is looked up once per card at load (`CardDef::repl`), together with the tags that
are replacement or prohibition effects (`melira`, `mauhur`, `nodmg`; `TAGGED`), so a step compares one small enum per
permanent and never a name.

| card                                   | kind      | effect                                                        |
|----------------------------------------|-----------|---------------------------------------------------------------|
| Hardened Scales                        | Counters  | +1 on +1/+1 counters put on a creature you control            |
| Vorinclex, Monstrous Raider            | Counters  | twice the counters you put; half (down) an opponent puts      |
| Doubling Season                        | Counters  | by name until Stage C (loyalty, proliferate); then its compiled doubler only |
| Solemnity                              | Counters  | no counters on players, artifacts, creatures, enchantments, lands |
| Melira, Sylvok Outcast (tag)           | Counters  | no poison for you, no -1/-1 counters on your creatures        |
| Mauhúr, Uruk-hai Captain (tag)         | Counters  | +1 on +1/+1 counters on your Army, Goblin or Orc              |
| Academy Manufactor                     | Tokens    | a Treasure, Food or Clue is one of each instead               |
| Cho-Manno (tag), Tajic (Iroas: own reader) | Damage | damage to it / your attackers / your other creatures (noncombat) prevented |
| Erebos, God of the Dead (tag, read by `life_gain`, not in the table yet) | LifeGain | opponents can't gain life |
| Hushbringer                            | Trigger   | creatures entering or dying trigger nothing                   |
| Laboratory Maniac, Jace, Wielder of Mysteries | Draw | drawing from an empty library wins instead              |
| Spelunking                             | Enter     | your lands enter untapped                                     |

The other effects are already data: the ability language's compiled `replacement` abilities (`tokens`, `counters`,
`lifegain` multipliers: Doubling Season, Anointed Procession, Parallel Lives, The Wind Crystal), its statics
(`lifegain_plus`, `no_lifegain`, `enters_tapped`), and the card-code hook slots that are replacements
(`no_graveyard`, `prevent_damage`, `no_lifegain`, `enters_tapped`, `steal_draw`, `skip_draw`, `trigger_copies`,
`proliferate_extra`). The step reads them where it reads the table, so each event kind has one reader.

## Order (rule 616.1) and who picks

The effects that change an amount are kept as a short list of modifications (`Mod::Add(k)`, `Mod::Mul(k)`,
`Mod::Half`, `Mod::Zero`), at most a handful, in a fixed array (no allocation).

- Self-replacement first (614.15): the event's own "instead" (a card's own "enters with N counters", undying's
  counter, a spell's "exile it instead") is built into the event the caller hands the step, before the others.
- Prohibitions ("can't": Solemnity, Melira, Erebos, Narset) are checked first; they stop the event.
- Then the affected object's controller (or the affected player) orders the rest (616.1). The AI tries each order of
  the list (2-4 effects: at most 24 orders, only when two or more apply) and keeps the best for that player: the
  most +1/+1 or loyalty counters on its own permanents, the fewest -1/-1 or poison counters on its own. A person
  (practice mode) is asked with `human::choices::choose_one` when two orders give different results, each choice
  labelled with its effects in order and the result.
- Stage A keeps today's order at every site (adds, then multipliers, then Vorinclex, player by player), so results
  stay identical; Stage C switches to the controller's choice.

## How it reaches the call sites

- Stage A: the step and the table exist; `cardcode::vorinclex`, `cardcode::scales`, `values::melira`,
  `t45_heliod_zur::solemnity`, `values::no_damage`, `cardcode::damage_prevented`, `dsl::token_mult`,
  `dsl::counter_mult`, `dsl::lifegain_mod`, `zones::hushed`, the Doubling Season checks become thin readers of it.
  Each old site keeps the subset of effects it applied today: the counters step takes a `Path` (which effects this
  path applies: `Path::PUT` is Solemnity + Vorinclex, `Path::ADD` adds Mauhúr and Scales, `Path::DSL` the ability
  language's counters, `Path::ENTER`, `Path::PROLIF` ...), so the old coverage is explicit.
- Stage B: each scattered site calls the step for its event kind (`repl::put_counters`, `repl::give_poison`,
  `repl::tokens`, `repl::perm_damage_prevented`, `repl::trigger_count` ...) and the old helpers go.
- Stage C: every counters path applies every effect (the `Path` subsets go), the order is the controller's, and
  the other coverage bugs below are fixed, each with a test.

## Speed

Measured with `examples/repl_cost` (new, not a sim: the `chars_cost` board, 63 permanents, every ask on every
permanent, 2000 rounds), at the base (ns per ask, empty board / with Doubling Season, Hardened Scales, Vorinclex):
+1/+1 counters 328-334 / 342-346, -1/-1 112-115 / 115-116, poison 112-115 / 115-116, tokens 186-196 / 228-245,
damage 1.4 / 1.4, life gain 358-368 / 373-374. `clone_cost` 2.95-3.13 µs.

Plan: one pass over the battlefield per ask reading `CardDef::repl` (a byte) instead of one pass per helper
(Solemnity, Melira, Scales, Vorinclex, Mauhúr each scan every permanent and compare names today); the ability
language's statics only while `dsl_on`; the order search only when two or more effects apply.

## Current sites, by event kind

Counters (+1/+1, -1/-1, loyalty, poison):
- `cardcode::put_plus` (84 callers; Solemnity, Vorinclex as the owner), `cardcode::add_counters` (59 callers;
  Mauhúr, Scales, then `put_plus`), `zones::minus_counter` (Melira), `zones::amass` (Mauhúr, `put_plus`).
- `dsl` effects `counters` (Solemnity, `counter_mult` of the actor, Scales, Vorinclex), `poison` (Melira,
  Vorinclex), `double_counters` (Scales, `put_plus`); loyalty abilities (`dsl` `Loyalty`, `common::pw_use`:
  Vorinclex).
- `zones::enter` (loyalty: Doubling Season by name, Vorinclex; enters with counters: Scales, Vorinclex, Solemnity;
  undying: Vorinclex, Solemnity), `zones` persist (Melira).
- `common::proliferate` (Doubling Season by name, Scales, Vorinclex, Solemnity, Melira), `mine::proliferate_all`
  (Melira, Vorinclex on poison only).
- Poison: `mine` Vraska -9 and proliferate, `t3_atraxa` (2), `tymna_cards` toxic and Phyrexian Unlife,
  `human/cards` Vraska, `karn_cards::infect_hit` (none at all), `dsl` poison.
- Combat wither / infect -1/-1 counters (`combat.rs`, `put_plus`).
- Card code with its own doubling: `green_cards` (Bristly Bill, Fractals: Scales; Topiary Lecturer:
  `counter_mult` then Scales), `blue_cards` (`counter_mult`, `add_counters`), `t12_tatyova_teysa` Mossborn Hydra
  (Scales), `legends` (`add_counters`), `t45_heliod_zur` / `combos` (Solemnity reads), direct `.plus +=` writes
  in `t12_isshin`, `common` (2), `legends`, `zones` (Mimic): no replacement at all.

Enter: `zones::enter` (enters tapped: `hooks::enters_tapped`, `dsl::opp_enters_tapped`; counters above),
`turn::land_enters_tapped` (Spelunking, Thalia / Archon, the land's own rule; 14 callers), `make_tokens`
(`hooks::enters_tapped`), `make_artifact_tokens_tapped`.

Tokens: `zones::make_tokens`, `zones::make_artifact_tokens_tapped` (Academy Manufactor by name), `common` (3337),
`sheoldred` (673), `t0_zurgo` (286): `dsl::token_mult` (Kaya, compiled doublers).

Damage: `values::no_damage` (29 callers: Cho-Manno, Iroas), `cardcode::damage_prevented` (source side: Old Fat
Spider, Arachnogenesis, Galadhrim Ambush; 4 combat sites), `cardcode::dovin_blocked`, `cardcode::tajic_protects`
(removal only), `combat` fog, `life::prevents_damage` (Glacial Chasm, Ultimate Magic: Holy, Solitary Confinement,
the Ring), `t0_giada` Seraph of the Sword, `t3_aurelia` Gisela, `g.unpreventable()` (9 sites).

Life gain / loss: `life::gain` (Erebos, `dsl::no_lifegain`, `hooks::any_no_lifegain`, Angel of Vitality,
`dsl::lifegain_mod`), `life::lose_life` (Fiery Emancipation, Stagger, Gisela, Bloodletter, prevention, Serra's
emblem, Pariah, Phyrexian Unlife).

Graveyard: `zones::die` (Rest in Peace `no_graveyard`, Hushbringer), undying / persist (3 `no_graveyard` reads),
`marchesa`, `sheoldred` (928), `mine::loops_blocked`, `removal` `exiledie` tag, regeneration (`noregen`).

Draw: `zones::draw` (Narset / Spirit of the Labyrinth, Notion Thief, Lab Man, Abundance, Urabrask, Uba Mask),
`cardcode::skip_draw` (Solitary Confinement).

Trigger: `hooks::total_trigger_copies` (12 callers: Panharmonicon, Teysa, Isshin, Ancient Greenwarden, Drivnod),
`zones::hushed` (3 callers), `hooks::fire_trigger`, `dsl` (1430).

## Bugs the census found (Stage C)

1. Doubling Season's counter doubling misses most paths: `put_plus` / `add_counters` (143 callers), enters with
   counters, undying, card code's counters. Only the ability language's `counters` effect, proliferate and loyalty
   (by name) double.
2. The ability language's `counter_mult` doubles by the actor, not by the permanent's controller (Doubling Season:
   "a permanent you control"): your effect on an opponent's creature doubles, theirs on yours doesn't.
3. Order: Topiary Lecturer puts n x 2 + 1 with Scales and a doubler (the controller would pick (n + 1) x 2);
   Vorinclex halving vs Scales; Gisela's halving after Fiery Emancipation (the player hit picks halve first).
4. Abilities lost: a neutered Hardened Scales, Vorinclex, Melira or Doubling Season still applies (Solemnity and
   Hushbringer stop).
5. Poison paths: `infect_hit` ignores Melira and Vorinclex; Phyrexian Unlife ignores Vorinclex; Vraska's -9 sets
   nine (not a placement: fine).
6. Solemnity misses the direct `.plus +=` writes and proliferate's kept land counters; Melira misses -1/-1
   counters put by wither / infect / `put_plus` with a negative n.

## What doesn't fit (yet)

- Damage to creatures is not an event: combat and removal ask "would it die" with prevention checks; doubling
  damage to creatures (Fiery, Gisela, Stagger, Flare Star) stays at those sites. Only the prevention is one step.
- Destroy replacements (regeneration, indestructible, Umbra armor, Welding Jar) stay in `zones::die`.
- Panharmonicon / Teysa are not replacements (603.2d) but share the trigger step; Hushbringer and Narset are "can't".
- Counters moved between permanents and Sagas' lore counters are not counters events. Kept counters (a land's
  `DataKey::Counters` and charge counters, the Incubator, The Ozolith) are, for proliferate only (`Ctr::Other`).
- Gisela's halving is applied after Fiery Emancipation / Stagger (the player hit would halve first): damage
  amounts stay at their sites (see above), so the order fix waits for a damage event.
- Mimic's extra counter on a new Orc Army, and Vraska's "nine poison counters" (a set, not a placement), stay direct.
- Two Academy Manufactors still make one of each kind once (the step reads "any").
- Erebos and Iroas read their own tag / card id, not `CardDef::repl` (no name compare either).
- Notion Thief's steal chain and the draw replacements keep their fixed order (no pool deck plays two).

## Done (Stages A-C)

- Stage A (e50a8466): `engine/repl.rs` (event kinds, `NAMED` + `TAGGED`, `CardDef::repl`, the counters step with a
  `Path` per old site so each kept its own coverage, `Mods`), the old helpers read through it; `tests/repl.rs`
  compares the readers with the old bodies. Results identical (3649 passed).
- Stage B (d15e2de9): every site calls the step for its event kind; the old helpers are deleted. Results identical
  (3649 passed).
- Stage C: bugs 1-6 above fixed except Gisela's order (doesn't fit): one counters path (`repl::put_counters`,
  `Path::ALL`) for every placement by an effect (put_plus, add_counters, the ability language, enters with counters,
  undying, proliferate, card code's direct writes in Breena, Hydaelyn and Spark Double); Doubling Season by its
  compiled doubler, the permanent's controller's (its name entry is gone); the affected controller orders the
  effects (`repl::ordered`: the AI the best for itself, a person asked, `replacement_order`); every table entry
  stops when its abilities are lost (`src_on`), and the compiled doublers too; infect and Phyrexian Unlife poison
  through the poison step; Solemnity covers kept counters and players; Tajic's prevention respects "can't be
  prevented" everywhere. One pass over the battlefield per counters ask (`gather`), the doublers' product kept on
  the card at load (`CardDef::counters_mult`). 3655 passed.

Speed after Stage C (`repl_cost`, ns per ask, empty / doublers; the counters rows ask the step): +1/+1 74-107 /
76-81 (base 328-334 / 342-346), -1/-1 47-74 / 68-70 (112-115 / 115-116), poison 46-56 (one run 108) / 51-52 (112-115 / 115-116), tokens
173-179 / 225-233 (186-196 / 228-245), damage 1.4, life gain 354-356 / 376-377 (358-368 / 373-374): faster or the
same. `clone_cost` 2.89-3.03 µs (base 2.95-3.13).

## Decks that play a card whose behaviour changed in Stage C (for the recalibration after Phase 4; no sims run)

- Doubling Season (now doubles every counter placement on its controller's permanents, by the controller) and
  Vorinclex (order, abilities lost, infect poison): atraxa-infect t3. Every deck at a t3 table with it is touched
  when atraxa puts counters on their permanents or theirs meet its Vorinclex (JD's sheoldred, sauron, seph, kefka,
  cloud, veyran play t3).
- Solemnity (kept counters, Incubator / Ozolith, artifact / creature planeswalkers, the direct writes now stopped)
  and Phyrexian Unlife (Vorinclex): zur t5 (JD's karn, thrasios, yshtola play t5).
- Mauhúr, Uruk-hai Captain: one more counter on Armies, Goblins and Orcs on every placement (it was add_counters
  and amass only): JD's sauron.
- Spark Double (miirym t5), Breena (isshin t1), Hydaelyn (no deck): their counters go through the step (the same
  unless a doubler, Scales or Solemnity is out).
- Abilities lost now stop Anointed Procession (edgar t4), Parallel Lives (miirym t5), Academy Manufactor (korvold t3,
  prosper t4), Laboratory Maniac (chulane t4); only under Darksteel Mutation / Kenrith's Transformation / Reprobation
  (light-paws t1, sythis t2, marwyn t3, zur t5, JD's thrasios).
- No pool or JD deck plays Hardened Scales, Melira, Topiary Lecturer or Vinelasher Kudzu.
- Tiers touched: t1, t2, t3, t4, t5; JD's decks: sauron directly (Mauhúr); the rest only through t3 / t5 opponents.
