# Phase 4, PR 7: each deck's hand-written AI behind a switch, off by default (design)

Default since the recalibration's stage 1 (`audit/recal/pairs-2026-10-31.md` section 4): on for four decks, see "The
switch".

Brief: `rust/agents/phase4-hand-off.md`. JD's decision (2026-10-31): with no sims, every deck's hand-written AI goes
behind a switch that defaults to the generic policy now; it is deleted later, deck by deck, after the recalibration's
on/off paired test for that deck. Nothing is deleted in this PR.

## How a site is classed

- **Deck AI**: a choice the generic policy also makes for itself: what to cast and when (priority, holds), what to
  tutor, mulligan, discard, attack with and at whom, block, which target, protection and wipe answers, which mode, an
  equip target, the look-ahead's per-deck bonuses. With the hand AI off the site is skipped and the generic answer
  stands. These sites ask the switch.
- **Card code**: how a card works, including a choice inside a card's own resolution that the generic policy has no
  option for (the card is played or activated only through that code). It stays as it is, even where it is gated on a
  seat: switching it off would remove the card's mechanism, not hand a choice to the generic policy.
- **Table reads**: other players reading a deck by its key (how much they want to counter its commander, how
  dangerous its board is). They are the opponents' policy, not the deck's; they stay. Reports and stats (milestones,
  snapshots) stay too.

## The switch

`AI_HAND` (env): unset or empty = **the default**, the hand AI on for `ai::HAND_KEEP` only (yshtola, sheoldred,
cloud, dack-fayden-white-stax) and the generic policy for every other switchable deck (set from the stage 1 paired
tests, `audit/recal/pairs-2026-10-31.md` section 4: yshtola +4.1, sheoldred +1.9, cloud +6.6; Dack is held on pending
a check of its -13.9); `0` = every switchable deck plays the generic policy (this PR's default until then); `1` = every
deck's hand AI as before this PR; a comma-separated list of deck keys = only those keep their hand AI (the paired test
of one deck with the rest of its tier unchanged). A thread-local override (`ai::force_hand`, a guard that restores the
previous value on drop) forces it on or off in a test, as `ai/ismcts.rs` does with `FORCE`.

One function answers it: `ai::hand_ai_key(key)` (and `ai::hand_ai(g, p)` for a seat). `ai::hand_key(g, p)` is the
seat's deck key when its hand AI is on and `""` when it is off; every deck-AI site reads its key through it
(`match hand_key(g, p) { "sauron" => ... }`, `hand_key(g, p) == "cloud"`, `is_main(hand_key(g, p))`), so with the
switch off the site falls through to the generic branch it already had for the pool decks. Card-code sites keep
`g.player(p).key`.

The switchable decks (`ai::HAND_DECKS`) are the live decks with hand AI: JD's ten with deck code (veyran, sauron,
yshtola, galadriel, alela, kefka, sheoldred, cloud, karn, thrasios) and the pool's four (brago, aurelia, sisay, dack).
Every other key answers "on" (nothing to switch): the pool decks without hand AI, and the retired test seats whose hand
AI stays as it is because no paired test of it is possible (sephfs, najeela, omnath / titania, garland, kraum, seph,
scion, godo, gaa, heliod; `omnath` and `godo` are never seated).

With the hand AI off for a deck:

- **Generic-policy deck**: `values::generic_deck` is true for it (one-line change, listed below), so derived values,
  the learned evaluation and ISMCTS apply to it as to the pool.
- **Plan file**: it plays from `data/plans/<key>.toml` through `plans::config` (priority rules, derived wish list,
  holds, mulligan, knobs). A deck with no plan file gets one in this PR, written from what its hand AI encodes. With
  the hand AI on, the new files are not read (`plans::config` and `plan_data::active` return the hand configuration),
  so `AI_HAND=1` is the old AI. `AI_PLAN_DATA` keeps its meaning for the decks it already switched; for the new files
  `AI_HAND` alone decides.
- **Style**: unchanged (JD's styles stay in `MAIN_STYLES`; no style values in this PR).

### Tier 6

The Tier 6 table code (`ai/t6_table.rs`, `pl.t6`, the five plan slots in `impls/t6_plans.rs`, `t6_combos_*`) is the
table's: fast-mana keeps, archenemy hold-up and the go-now stop checks apply to every cEDH seat alike, so it is not one
deck's hand AI and stays on. Tier 6 decks also stay out of `generic_deck` (derived values and ISMCTS were paired-tested
on t1-t5 only; their t6 behaviour is a separate question for the recalibration). The pool decks there already play
from their plan files (PR 2 deleted their hand plans after level paired tests). What is switched:

- **dack-fayden-white-stax**: its hand plan (`impls/t6_plan_dack.rs`, the only live hand plan left) goes behind the
  switch; off, it plays from a new `data/plans/dack-fayden-white-stax.toml` through the same slots. PR 2 found Dack's
  slots not expressible exactly (its commander-cost bar, Chalice's net, the table's creature share); the file encodes
  what it can (key cards, the Dack line, holds, mulligan pieces), and the recalibration decides whether the gap matters.
- **sisay-legends-toolbox**: the table's Sisay rule (a ready line before another Sisay fetch, `t6_table.rs`) is a
  Sisay-only priority: switched. Sisay's `fetch` (legends::sisay_pick) is the card's ability choice: card code.
- **thanos** (JD's): no hand AI of its own; it plays from `data/plans/thanos.toml` through the table's slots already.
- **godo** (unseated) and **scion** (retired test seat): their hand code stays on.

## Survey: deck AI vs card code, per deck

Sites are `file:function`. "(key)" means the site reads the key through `hand_key` after this PR.

### JD's decks

**veyran** (`ai/veyran.rs`, `ai/decks.rs`, `ai/brain.rs`)
- Deck AI: `decks::deck_prio` -> `veyran::veyran_prio`; `decks::tutor_pick_named` (Greaves first, `recruit_wish`,
  `is_wish`); `decks::protect_response` -> `veyran::protect`; `brain::special_options` -> `veyran::options`;
  `brain::card_utility` (the magecraft bonus for instants and sorceries, `veyran_sequence`); `stack.rs` counter value
  (+1.5, every counter is a doubled trigger); `combat::spare_tokens` (chump with small tokens); `t0_stella`
  `mastery_options` (is_main: Veyran makes the choice itself).
- Card code: `zones.rs` magecraft from a copied spell; `ai::combo_interrupted` (the opponents' answer to the loop and
  Mistrise Village); token colour.
- Table reads: `brain.rs` situation (an opponent's Aetherflux at 40 life), `cast.rs` commander importance.
- Report: `turn.rs` engine milestone.

**sauron** (`ai/decks.rs`, `impls/mine.rs`, `impls/sheoldred.rs`, `impls/threats.rs`, `impls/galadriel.rs`)
- Deck AI: `deck_prio` -> `sauron_prio`; `tutor_pick_named` -> `sauron_tutor`; `protect_main` (sauron arm);
  `wipe_response` (sauron arm), `sauron_grounds_response`; `wipe_eval` (the Army without Ozolith); `archivist_worth`;
  `special_options` -> `sauron_options` (equip, Helm, Archivist, Monolith untaps); `brain::hold_value` (counter
  shell); `stack.rs` pitch choice (`sauron_pitch_keep`);
  `sheoldred.rs` `cost_options` / `cost_plan` (host = the Army); `mine.rs` `breach_line` (runs the line without the
  pieces check), `galadriel::choose_type` / `mine::territory_type` (Orc), `legends.rs` territory type; `yshtola.rs`
  `x_option` (the drain X spell as a main-phase play); `threats.rs` `copy_worth`.
- Card code: `combat.rs` Sword + Aggravated Assault extra combats (the line exists only there);
  `threats::torment_options` (Torment of Hailfire's only cast path; shared with Tergrid); `mine.rs`
  `army_power_snapshot` (report).
- Table reads: `brain.rs` situation (Sword + Assault), `engine/values::threat` (+10), `reanimate.rs` (Mother of
  Runes against Sauron), `cast.rs` commander importance.

**yshtola** (`impls/yshtola.rs`, `impls/t5.rs`)
- Deck AI: `deck_prio` / `deck_prio_f` -> `yshtola_prio`; `tutor_pick_named` -> `yshtola::tutor_pick`;
  `protect_response` -> `yshtola_protect`; `wipe_response` (yshtola arm); `brain.rs` removal adjust (spot removal only
  on targets worth it, both in her turn and on attackers), no sampling far below the best target, wipe swing with
  Teferi's Protection first; `combat.rs` chump rule (Y'shtola chumps only to live); `yshtola::x_option`; Champion's
  Helm equip (`yshtola::helm`); `t5::zur` (Zur's pick for this deck).
- Card code: none deck-keyed beyond these.

**galadriel** (`impls/galadriel.rs`)
- Deck AI: `deck_prio` -> `galadriel_prio`; `protect_response` -> `galadriel_protect`; `wipe_response` (galadriel
  arm); `brain::attack_response` (Ballista Squad, Lawbringer answers); `stack.rs` pre-combat taps
  (`galadriel_precombat`); `GENERIC_PLAYS` membership is moot when off (the pool path runs).
- Card code: none deck-keyed.

**alela** (`impls/alela.rs`)
- Deck AI: `deck_prio` -> `alela_prio`; `protect_response` -> `alela_protect`; `alela::greaves_options` (move the
  Greaves off Alela for an Aura).

**kefka** (`impls/kefka.rs`, `impls/mine.rs`, `impls/legends.rs`)
- Deck AI: `deck_prio` -> `kefka_prio`; `tutor_pick_named` (storm wish, Mystical for the storm card);
  `ai::dsl_search_pick` (Grim Servant within devotion); `protect_response` (kefka uses `mine::veyran_protect`);
  `special_options` -> `kefka_options`; `cast.rs` discard (`kefka::discard_n`); `mine::kefka_wheel` own plan (count
  the pile); `kefka::loop_guard` (hold the face damage), `flip_utility`, `tutor_win_piece`; `legends.rs` purge /
  profane utilities; `stack.rs` soft-counter factor (the counterer is Kefka).
- Card code: `mine::breach_escape` Garrison Excavator trigger (`kefka::gy_left`).

**sheoldred** (`impls/sheoldred.rs`, `impls/legends.rs`)
- Deck AI: `deck_prio` -> `sheoldred_prio`; `protect_response` -> `sheoldred_protect`; `special_options` ->
  `sheoldred_special`; `deluge_cap` (Toxic Deluge's X while she is out); `lesson_options`, `deal_options`,
  `punish_window_utility`, `atlas_options`, `peer_options` (draw-punisher plays); `legends.rs` jar; `yshtola::x_option`;
  `plans::adnaus_cast_now` (Ad Nauseam in main 2).
- Card code: none deck-keyed.

**cloud** (`impls/cloud.rs`, `impls/bulk_white.rs`)
- Deck AI: `deck_prio` -> `cloud_prio`; `tutor_pick_named` -> `cloud::tutor_pick`; `protect_main` (cloud arm,
  `white_protect`); `wipe_response` (two cloud arms); `ai::wipe_modes` -> `cloud::wipe_modes`; `ai::attack_filter` ->
  `cloud::attack_filter`; `combat.rs` all-out trim; `turn::keeps_hand` -> `cloud::keep_hand`; `brain::card_utility`
  (`strands_equipment`); `special_options` -> `cloud_special`; equip weights (`holder_bonus`,
  `equip_utility_bonus`, `wants_move`); `must_attack`; `riders_spare`; `open_the_armory` (the pick);
  `search.rs` `search_bonus` (two sites) and the first-combat rule.
- Card code: `cmd_stays_in_hand` (rule 903.9b, with garland); `jocasta_attack` (Jocasta's trigger);
  `plan_milestone` (report).

**karn** (`impls/karn.rs`, `impls/karn_*`)
- Deck AI: `deck_prio` -> `karn_prio`; `karn::land_adjust` (Tron land first), `search_pick` (the missing Tron piece,
  also through `dsl` and `ai::dsl_search_pick`), `block_response` (Factory/Foundry blocks), `hold_ability` (keep the
  Baubles); `search.rs` `search_bonus` (two sites) and the first-combat rule; `combos.rs` (karn is the one main deck
  offered catalogue combos: moot when off, every deck is).
- Card code: none deck-keyed.

**thrasios** (`impls/tymna_partners.rs`, `impls/tymna_cards.rs`, `impls/common.rs`, `impls/combos.rs`)
- Deck AI: `deck_prio` -> `thrasios_prio` (and `partner_prio`); `tutor_pick_named` -> `tymna_cards::tutor_pick`
  (`land_pick`, `combo_tutor_pick`); `special_options` -> `kraum_special`; `spread`, `commander_first`,
  `thrasios_sink`, `protection_held`, `combat_trick`; `equip_holder_value`, `equip_utility_value`;
  `endurance_protect`, `endurance_wipe`, `blossom_protect`; `common.rs` Ballista counter targets.
- Card code: `combos.rs` Ballista found by Thrasios from the library (the line's reachability, Rust only, seat-gated
  to keep the pool's Tier 6 decks unchanged).
- Note: thrasios is in `jd_deck` but not `is_main`, so it already uses the pool path for most choices; its hand AI is
  the sites above.

**ghave**, **urdragon**, **thanos**: no deck-keyed hand AI; each plays from its plan file already (card code in
`impls/ghave.rs`, `t45_urdragon.rs`, `t6_thanos.rs` registers card abilities and card priorities, which any deck
playing the card uses). `values::generic_deck` already reads them as generic decks except thanos (Tier 6). Nothing to
switch.

### Shared JD-deck routing (`is_main`, `jd_deck`)

Every `is_main(key)` / `jd_deck(key)` routing site is deck AI ("your deck's own code decides; the pool path is
skipped"): `ai::discard_keep`, `ai::attack_filter`, `ai::dsl_search_pick`, `plans::wish_list`, `decks::tutor_pick`,
`decks::protect_response` / `protect_main`, `decks::wipe_response`, `brain::hold_value`, `card_utility` (and its
`jd_deck` hold of instants), `cast_utility`, `removal_options`, `hook_options` (`GENERIC_PLAYS`), `main_options`,
`choose_defender` ninja bonus, `search::choose_attack_inner`, `common::aristocrat_options`, `common::sac_in_response`,
`partials` Tidebinder, `t0_stella::mastery_options`, `t12_brago_meren` Buried Alive, `t45_urza_yawgmoth` Entomb,
`dsl` sacrifice choice, `cardcode` Defend hooks and ninjutsu, `combos::combo_options` / `line_enabled_by`,
`combat::vig_utility`. They read `is_main(hand_key(g, p))`.

`ai::base_style` keeps `MAIN_STYLES` (styles are not changed in this PR); `ai::is_main(key)` itself is unchanged.

### Pool decks

- **brago-azorius-blink-control** (`impls/cc_brago.rs`): deck AI: `hold` (Cloudshift waits for the Kitten loop).
  Card code: the loops (`cc_brago`, `t12_brago_meren` Combo entries).
- **aurelia-boros-extra-combats** (`impls/t3_lines.rs`): deck AI: `helm_options` (Helm on Aurelia before combat; the
  generic equip has its own target). Card code: Sword of Feast and Famine's and Aggravated Assault's priorities and
  Assault's activation (`soff_prio`, `assault_prio`, `sword_options`, `assault_again`): the only way the deck plays
  those cards (no generic option for the activation; the cards had no priority).
- **sisay-legends-toolbox** (`ai/t6_table.rs`): deck AI: the Sisay line/fetch priority. Card code: `sisay_pick`.
- **dack-fayden-white-stax** (`impls/t6_plan_dack.rs`): deck AI: the five slots (prio, wish, keep, hold_up, go_now).
- **teysa-orzhov-aristocrats** (`impls/cc_teysa.rs`): card code: its two loops are catalogued for this deck only
  (`blocked`), a catalogue scope, not a choice.
- **tergrid-mono-black-disruption** (`impls/threats.rs`): card code: Torment of Hailfire's cast path (shared with
  Sauron).
- **captain-america-avengers-assemble** (`impls/t0_captain.rs`): card code: Herald's Horn names Hero as it enters.
- **gaa-azorius-stax-control**, **heliod-mono-white-stax** (`ai/plans.rs` CONFIG): retired test seats, hand config
  stays.
- **sythis-selesnya-enchantress** (`impls/t12_kaalia_sythis.rs`): card code only (Strength of the Harvest, Calix,
  Resurgent Belief, Weaver of Harmony); its `prio` hooks are per card (Resurgent Belief never hard-cast), not keyed on
  the deck. Nothing to switch.
- Every other pool deck: no deck-keyed AI (card code and plan files only).

## Shared files touched (one-line changes forced by the switch)

- `ai/values.rs::generic_deck`: a JD deck with its hand AI off is a generic-policy deck.
- `ai/search.rs`: the karn / cloud `search_bonus` matches (two), the karn / cloud first-combat rules (two) and the
  `is_main` attack rule read `hand_key`.
- `ai/brain.rs`: switch checks only (`hand_key` in place of the key at each deck-AI site).
- `ai/ismcts.rs`, `ai/learned.rs`, `data/eval/`: untouched (they read `generic_deck`).

## Plan files written

See `audit/phase4/hand-off-handoff.md` for the list and what each encodes.

## For the recalibration

Per deck, a paired test (same seeds) of `AI_HAND=<key>` (its hand AI on, the rest of the table unchanged) against
the default (off), on the deck's own tiers for JD's decks, by the half-tier method of `audit/phase4/values-handoff.md`
for pool decks:

| deck | test | tiers |
|---|---|---|
| veyran | `--deck veyran`, own tiers | t1-t3 |
| sauron | `--deck sauron` | t3-t4 |
| yshtola | `--deck yshtola` | t5 |
| galadriel | `--deck galadriel` | t1-t2 |
| alela | `--deck alela` | t1 |
| kefka | `--deck kefka` | t3 |
| sheoldred | `--deck sheoldred` | t3-t4 |
| cloud | `--deck cloud` | t1-t3 |
| karn | `--deck karn` | t4-t5 |
| thrasios | `--deck thrasios` | t5 |
| brago-azorius-blink-control | `--realism --pool t2`, half-tier pair | t2 |
| aurelia-boros-extra-combats | `--realism --pool t3`, half-tier pair | t3 |
| sisay-legends-toolbox | `--realism --pool t6` | t6 |
| dack-fayden-white-stax | `--realism --pool t6` | t6 |

The rule: where off is level or better (the difference's CI includes 0 or is above it), delete the deck's hand AI
(the sites above, then the `HAND_DECKS` entry). Where off is worse, improve its plan file (lines, priority rules,
holds, mulligan) and test again; keep the hand code only if the gap can't be closed, and record why in this file.
