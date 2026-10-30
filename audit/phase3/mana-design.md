# Phase 3, PR 4: a mana pool with restrictions (design)

Branch `phase3-mana`, stacked on `phase3-repl`. Brief: `rust/agents/phase3-mana-exile.md`. Handoff:
`audit/phase3/mana-handoff.md`.

## The pool

`engine/pool.rs`. `Player::mana: Pool` replaces `Player::floating: Floating` (eight `u32` fields: `w u b r g c any
art`). A pool is a short sorted `Vec` of entries `Mana { key: Key, amt }`, with

| field of `Key` | values | rule |
|----------------|--------|------|
| `kind`  | `W U B R G C`, `Any` (one of the player's colours, chosen as it is spent: the engine's rituals and the ability language's "one mana of any colour") | 106.1 |
| `restr` | `None`; what it may pay for (106.6): `NotNonartifact` (Karn, Legacy Reforged; Powerstones; The Mightstone and Weakstone), and from Stage B the source restrictions below | 106.6 |
| `keep`  | `Step` (empties as the step or phase ends) or `Turn` ("until end of turn, you don't lose this mana as steps and phases end") | 500.4 |

and, from Stage C, an optional rider on the mana (`Rider::Uncounterable`: Cavern of Souls, Delighted Halfling) kept in
the key the same way. Mana that differs only in amount is one entry. The entries are sorted by (restriction, kind,
keep), so the payment planner reads the floating mana in the old fixed order (red, any, blue, colourless, green,
black, then Karn's) and its first-best tie-breaking is unchanged. An empty pool is an empty `Vec` (no allocation when
a game is copied; almost every pool is empty almost always).

API: `add(kind, n)`, `add_key(key, n)`, `add_art(n)`, `get(kind)` (unrestricted, any keep), `amount(key)`, `art()`,
`take(key, n)`, `take_kind(kind, n)` (the mana that empties soonest first), `set`, `clear`, `total`, `iter`.

## Paying

The search is unchanged: `mana_units` lists every unit p can use now (untapped lands and mana permanents, Treasures,
Scions, hand cards, Quirion / Symbiote untaps) and then one unit per floating mana, `plan_pay` assigns them. The pool's
units are `Source::Pool(key)` (replacing `FloatR FloatAny FloatU FloatC FloatG FloatB FloatArt`), each of amount 1;
a pool entry whose restriction doesn't allow what is being paid for (`mana::restr_ok(g, restr)`, reading
`Game::pay_for`, the spell being cast, `None` for an ability or a tax) is left out, as `FloatArt` was. Paying a pool
unit takes it out of its entry (`Pool::take`). Restricted pool mana is spent first while it may be (the old "Karn's
mana first" key of the generic loop). A source that made more than the payment used floats the rest
(`float_leftover`), as `Kind::of_leftover(cols)`.

The AI is not made slower or worse: the units are the same units in the same order; the pool entries are read by one
loop over a usually empty `Vec` instead of seven field reads.

## Restrictions (Stage B)

Today a restriction is a `continue` in `mana_units` keyed on a name or tag, or a land's colour callback that answers
"any colour" only for the right spell. Stage B gives each a `Restr` value and one test, `restr_ok`:

| card(s) | today | `Restr` |
|---------|-------|---------|
| Karn, Legacy Reforged (`Floating::art`), Powerstone tokens, The Mightstone and Weakstone (`pstone`) | `FloatArt` + `art_mana_ok`; the rock skipped unless `art_mana_ok` | `NotNonartifact` |
| Mishra's Workshop (`workshop`) | the land skipped unless an artifact spell is paid for | `ArtifactSpell` |
| Giada, Font of Hope | the dork skipped unless an Angel spell | `Subtype("angel")` (a spell) |
| Hydro-Channeler | skipped unless an instant or sorcery spell | `InstantSorcery` |
| Rivaz of the Claw | `t45_urdragon::dragon_only_mana` | `DragonCreature` |
| Plaza of Heroes, Delighted Halfling | colours only for a legendary spell | `Legendary` |
| Unclaimed Territory, Cavern of Souls, Secluded Courtyard | colours only for a creature spell of the chosen type | `CreatureType(t)` |
| Haven of the Spirit Dragon | colours only for a Dragon creature spell | `DragonCreature` |
| Voldaren Estate | colours only for a Vampire spell | `Subtype("vampire")` |
| Avengers Tower | colours only for a Hero spell | `Subtype("hero")` |

The source-side tests are the same tests (results identical); what Stage B adds is that a unit's restriction is known
where its leftover floats (Stage C). Not restrictions on mana, left as they are: Damping Sphere and Blood Moon (what a
land makes), Collector Ouphe / Karn's static / Cursed Totem (`NoArtifactMana`, `NoCreatureMana`: abilities can't be
activated), Ancient Tomb at 2 life (a choice not to), `keep_untapped` (a combo piece), Honor-Worn Shaku's mana on
demand. Kinnan and Nyxbloom Ancient change amounts (`adjust_mana`), Forger's Foundry marks the spell its mana paid
for (an on-tap hook), Treasure bonuses: unchanged.

## Emptying (Stage C)

Rule 500.4: the pool empties at the end of each step and phase. Today floating mana lasts the whole turn (emptied in
`step_start`), and practice mode empties only the person's own pool between steps. Stage C empties every `Keep::Step`
entry as the engine's steps change (`turn::continue_turn`: start, main 1, combat, main 2, end), and the whole pool as
the turn starts. `Keep::Turn`: Karn, Legacy Reforged; Birgi; Klauth; Savage Ventmaw; Photon, Mighty Marvel; Rousing
Refrain. Mana Drain's mana is added at the beginning of the next main phase (it was the start of the turn).

## Where mana is added (the census)

`engine`: `cast` (Birgi), `tutors` (Jeska's Will), `mana::float_leftover`, `cardcode::turn_start` (Mana Drain),
`turn::step_start` (empties). Card code, by file: `common` (Ashnod's Altar, Phyrexian Altar, Priest), `t1`
(Nissa landfall), `t2` (Carpet of Flowers), `t3` (Waste Not), `t4` (Battle Hymn, a Chandra's {R}{R}), `t5` (Cabal
Ritual), `kefka` (Inner Fire), `karn` (Karn's upkeep), `yshtola` (Petition), `sheoldred` (Tower),
`legends` (Spark), `marchesa` (Relic), `galadriel` (Alliance), `mine` (Lion's Eye Diamond lines, Lotus Petal,
Manamorphose, the loops, the prepared Seething Song), `t6_mana` (Chromatic Star, _____ Goblin, Desperate
Ritual, Rite of Flame), `t6_green1` (Badgermole Cub), `t6_combos_t6` / `t6_combos_extra` (the unbounded lines),
`t12_tatyova_teysa` (Black Market), `t0_stella` (Rousing Refrain), `t3_marwyn` (Legolas), `t45_krenko` (Brightstone
Ritual), `t45_prosper` (Novelist), `t45_urdragon` (Klauth), `tymna_cards` (Photon), `dsl` (`add_mana`: rituals,
Savage Ventmaw). Practice: `human::mana::gather_floating` moves the engine's pool into the person's (`ManaPool`),
`advisor` and `shadow` put the person's pool back as floating mana on a copy.

Where mana is paid: `mana::pay` / `spend_unit` (every AI payment), `human::mana::pay_from_pool` (a person),
`mine::pay_line` (the Breach line spends ritual mana first), `t6_combos_t6::cast_thrasios` (the unbounded line's tax).

## Bugs the census found (Stage C)

1. Mana doesn't empty between steps (500.4): a ritual's or a big land's leftover in main 1 pays in combat and main 2;
   Black Market's says so ("Left out").
2. A restricted source's leftover floats unrestricted: Mishra's Workshop ({C}{C}{C} for artifact spells) paying a
   one-mana artifact floats {C}{C} for anything; The Mightstone and Weakstone's second {C}; Giada's, Rivaz's.
3. Photon's white mana is never spent by the AI (no white floating unit) and never empties.
4. The "can't be countered" riders of Cavern of Souls and Delighted Halfling are not modeled.
5. Maelstrom of the Spirit Dragon taps for any colour for anything (data `c=A`; it is "{C}, or any colour for a
   Dragon or Omen spell").

## Speed

`examples/mana_cost` (new, not a sim): one player's fixed mana base on the `clone_cost` table (8 lands, Sol Ring,
Arcane Signet, The Mightstone and Weakstone, Llanowar Elves), `mana_units`, `can_pay` and `pay` (on a copy, the copy
taken off), with an empty pool and with floating mana. Measured at the base and after each stage, with `clone_cost`.

## What doesn't fit (yet)

- Mana of "any colour" stays a choice made as it is spent, not as it is added (the old engine's model; a person
  in practice mode does choose when the colour matters).
- Shang-Chi's "only to activate abilities of creature sources", Castle Garenbrig's six {G}, Master of Dark Rites:
  the payment doesn't say whose ability it pays for, and those abilities are not modeled.
- Secluded Courtyard's and Avengers Tower's "or an ability of a ... source": abilities are not told apart.
