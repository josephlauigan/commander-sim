# t6-ai (2026-11-06): Tier 6 planner and hold fixes

Branch `t6-ai`, from c1b-engines (PR #276) on c1-t6-lines (PR #271). Brief: `rust/agents/t6-ai.md`. No sims were run
(single `--trace` games only); the coordinator runs the paired tests.

## NEEDS JD
1. **Game 139 itself is not changed by fix 1.** Its R6-R7 board has 4 and 3 creatures; Witness + Snap + Cradle needs 5
   creatures besides Witness (`plan_b`: `feeders < 5`), and the trace line "lacks [Eternal Witness]" (`t6stats::hold_diag`)
   counts piece groups only, so it called a line "one tutor away" that was two cards away (Witness and a fifth creature).
   With the fifth creature on the board the planner now casts Finale for Witness (test). New trace of game 139 on this
   branch: the same passes at R6-R7; the line is tried at R8 when Witness is drawn. Say if the diagnostic should also count
   creatures, or Finale should be used to fetch a feeder creature.
2. **Lion's Eye Diamond** (item 3): the sim never casts it outside the Underworld Breach line; proposed plan-file rows and a
   statistics fix are in section 3. Apply?
3. **Whir's tag changes Urza** (pool deck, not Tier 6): Whir was cast with X = 0 and now takes X = all spare mana, as Finale
   does. Urza's numbers move; none of JD's decks play Whir. Keep, or restrict to Tier 6?
4. **`hold_value` still holds instant removal when an opposing commander is a combo piece** (`s.cmdr_combo &&
   instant_removal`, brain.rs ~217), Tier 6 included. It is not `near`, so I left it. Switch Tier 6 to `combo_near` there?
5. **Ad Nauseam**: the only decks that play it are JD's `sheoldred` and `thrasios` and the pool's Kraum/Tymna and Rograkh/Silas
   (Tier 6 keys, plus Sheoldred); no other non-Tier-6 deck plays it, so the gate (`is_t6 || sheoldred`) equals "every deck"
   today. The retired Thanos list (decklists only) plays it too.
6. **Not examined**: Urza's Saga and Tezzeret, Cruel Captain (Dack) as planner tutors (a Saga chapter is not a one-turn
   play; the planeswalker is left out); Mana Vault "not castable" with mana up (g32 R2, one decision in 60 games).
7. **rustfmt**: the mandated `rustfmt --edition 2024` on `ai/brain.rs`, `ai/planner.rs`, `impls/rules2.rs`,
   `impls/t6_combos_t6.rs` also re-wrapped some unrelated lines in them (`impls/t1.rs` was not run through it: its imports
   are unsorted and the diff would be large). Expect textual merge noise with branches that touch those files.

## 1. Finale of Devastation for Eternal Witness (fixed)

**Position** (seed 900000, `--calibrate within --pool t6 --ai adaptive --trace 139`, Rograkh/Thrasios, R6-R7): Finale +
Snap + Flusterstorm in hand, 7-14 mana, Cradle (Growing Rites) out, Witness the one missing piece. Trace lines
`passes (Eternal Witness + Snap + Gaea's Cradle lacks ["Eternal Witness"]; tutors: [Finale of Devastation(X) finds 1/1])`
at R6 (twice), R7 (twice).

**Cause** (not the tutor model: `xtutor_rule` already takes Finale for any creature and the hold diagnostic shows it finds
Witness, 1/1). `data/plans/rograkh-thrasios-creature-combo.toml` holds Finale and Green Sun's Zenith at prio 0 ("the combo
pieces and copies are held"), and `planner::plays` skips any card the plan slot holds (`castable_at`: "Dack waits for the
table's risk to drop"). So the X-tutor loop never offered Finale. (GSZ is not hit in the same way where Witness is the
piece: the line itself casts GSZ through `t6_combos_t6::x_tutor`.) A second gap behind it: with Finale cast as the tutor, the
line had no finisher left (`finisher()` needs Finale in hand, or Thrasios plus Finale/GSZ+Mastermind in the library).

**Fix.**
- `ai/planner.rs`: the X-tutor loop of `plays` ignores the plan-slot hold (`castable_with(.., held_ok = true)`). The search
  still casts it only where a line is reached after it. A tutor the deck holds is never spent only to come "closer"
  (`PLANNER_PROGRESS=1`, off by default).
- `impls/t6_combos_t6.rs`: line B (Witness + Snap + Cradle) counts Finale in the graveyard as a finisher
  (`finale_in_gy`), and a new step before Thrasios returns it (`witness_returns_finale`: the loop is unbounded, its last
  Witness returns Finale instead of Snap, as at a real table).

**Test** `t6_ai.rs`: `finale_of_devastation_is_cast_for_eternal_witness` (plan `[Finale for Witness (X=3), line]`; no plan
before), `the_main_phase_goes_for_witness_with_finale_and_gets_finale_back` (main phase makes the play and runs the line),
control `green_suns_zenith_finds_eternal_witness_too`.

Also checked, same way: Green Sun's Zenith (held at 0 in the same plan file; the line casts it itself for Witness/Oboro, and
the planner may now cast it too for a plan with more steps), Chord of Calling / Nature's Rhythm / Invasion of Ikoria (not held:
already planner tutors), Finale and GSZ in Kinnan's plan file (not held).

## 2. Whir of Invention and the other artifact tutors (fixed)

**Cause.** Whir ({X}{U}{U}{U}, improvise, an instant) had no tags, so (a) `tutor_rule`/`xtutor_rule` never saw it (the
planner could not cast it for Basalt Monolith), and (b) the real cast gave it X = 0: `brain::do_cast` pays "all spare mana"
only for `xtutor`/`tokx` cards, so Whir found an artifact with mana value at most the number of untapped non-rock artifacts
it tapped (usually 0). The c1 report's "Whir is not a planner tutor" was the visible half of this.

**Fix.**
- `data/card_overrides.toml` (+ re-exported `data/cards.json`, one card): Whir gets the `xtutor` tag (X = all spare mana
  when the utility AI casts it, as for Finale; the planner pays exactly the target's mana value).
- `ai/planner.rs`: `xtutor_rule` knows Whir (an artifact card); `pay_x` = the target's mana value less the untapped
  artifacts improvise taps (`rules2::improvise_arts`), used by `plays`, `perform` and the end-of-turn reserve; the cast pays
  that, the artifacts' taps add theirs when Whir resolves, so the X is the target's value.
- `impls/rules2.rs`: `whir` takes the planner's card (`plan_tutor`) before its own pick; `improvise_arts` also skips phased
  artifacts.
- Trophy Mage (Kinnan's list; MV 3 artifact to hand when it enters): `planner::tutor_rule` models it as a tutor and
  `t1::tutor_named` takes the planner's card, so the planner can cast it for the Monolith and then cast the Monolith.

**Other artifact tutors in the Tier 6 lists** (jq on `data/decks.json`, six pool decks plus JD's `thrasios`): Transmute Artifact
(Kinnan; c1 fixed it), Enlightened Tutor and Mystical Tutor (a DSL search; already planner tutors, to the top of the library),
Expedition Map and Sylvan Scrying (land searches, left out of the planner on purpose), Urza's Saga (Dack Fayden; a land
whose chapter III fetches an artifact with mana value 0 or 1 over three turns: not a one-turn tutor, left out), Tezzeret,
Cruel Captain (Dack Fayden; a planeswalker, left out). Fabricate and Reshape are in none of the Tier 6 lists.

**Tests.** `whir_of_invention_is_cast_for_basalt_monolith` (plan `[Whir for Monolith (X=3), line]`, and the cast puts the
Monolith onto the battlefield), `whir_counts_the_artifacts_it_taps_to_improvise` (three Islands cannot pay for Whir; with two
untapped non-rock artifacts and three more lands, X is paid at 1 mana and both artifacts end tapped),
`trophy_mage_is_cast_for_basalt_monolith` (plan `[Trophy Mage for Monolith, cast Monolith, line]`).

## 3. Lion's Eye Diamond and Moxen held on turns 1-3 (investigation only; no code changed)

**Counts** (c1b `t6-cal-4000.txt`, 4000 games, each deck seated in about 3200; card-turns where the card was in hand at the
end of own turns 1-3, payable, and would make mana):
- Lion's Eye Diamond: Dack 554 (T1 172 / T2 191 / T3 191), Kraum/Tymna 627 (175 / 198 / 254), Rograkh/Silas 557 (169 / 192 /
  196): in about every game that holds it. In the 60 traced games (`--calibrate within --pool t6 --ai adaptive --seed 900000
  --trace 0..59`) LED was **never cast**; it was passed over in 29 stop decisions in 10 games (g1, g10, g19, g25, g28, g32,
  g49, g56, g58, g59), every one "Lion's Eye Diamond: no utility (prio 0)".
- Moxen and other free mana: Chrome Mox 7 / 16 / 7 / 9 / 8 / 15 card-turns (Dack / Kinnan / Kraum / Silas / Thrasios / Sisay),
  Mox Diamond 5 / 7 / 9 / 14 / 9 (the first five), Mox Opal 7, Mana Vault 2-4: under 0.5% of seat-games each, all "lost"
  (offered or refused by a plan rule), and the 60 traced games show the legitimate reasons below. Moxen are not the problem:
  mana cards cast on T1 are 0.66-0.87 a game, and a hand with a payable Mox that is held is rare.

**Causes.**
1. LED has no cast priority anywhere. `data/cards.json` tags it `led` and nothing else, no plan file has a `[[prio]]` row for
   it (Dack lists it under `free_plays`, Kraum under `[[wish]]`), so `card_utility` returns "no utility (prio 0)" every time.
   It is cast only by the Underworld Breach line's own step (`t6_combos_extra::cast_led`). Once on the battlefield,
   `mine::led_options` offers the crack only with a lands-only hand and a use (recast a commander, flashback, Breach). So the
   sim never plays LED as a plain early permanent.
2. The `mn|held` statistic counts LED as "makes mana now" (`t6stats::makes_mana_now` returns true for it) though the engine has
   no mana ability for it outside `led_options`; the 550-630 card-turns are therefore the same single cause, not 550 decisions.
3. Mox Diamond (`pool::card_utility`): prio 0 unless the hand holds two lands, or one land after the land drop, and (Tier 6)
   at least `t6_moxd_lands` = 3 lands in hand plus on the battlefield. g17 Kinnan R2-R3 and g50 Rograkh/Silas R1-R3 held it with
   no land in hand: correct (it would discard nothing, a free card for nothing). Mox Amber without a legendary creature or
   planeswalker (g11 Kinnan R1) and Chrome Mox without a card to imprint are "not castable": correct.
4. One real loss: a held-mana stop beating a Mox (older trace, g164 R3 Rograkh/Silas: Mox Diamond offered at 2.0 after the
   reserve penalty, "stop (hold mana)" at 5.5 with Swan Song up). Same cause as item 4 below (the reserve penalty), now smaller
   for Tier 6 with a combo-piece commander on the table.
5. Not explained: Mana Vault "not castable" with two lands untapped (g32 Rograkh/Thrasios R2, after Wild Growth and Gene
   Pollinator). One decision in 60 games; not traced further.

**Proposed fix (needs JD).** Give LED a cast priority in the three plan files (Dack, Kraum/Tymna, Rograkh/Silas), for example
`[[prio]] cards = ["Lion's Eye Diamond"] when = ["hand:wheels>=1 | hand:breach>=1"] value = 60` with a set of Wheel of
Fortune / Windfall / Timetwister / Underworld Breach (LED costs {0}; on the battlefield it turns Wheel into a mana source,
which `led_options` would then have to offer in response to the wheel). Without the set condition the engine would crack it
into an empty plan and lose the hand, so the condition matters. Also let `t6stats::makes_mana_now` treat LED as "no" when no
use is open, so the held statistic stops counting it. Option (b): leave it, since real players hold LED for a line too.

## 4. "An opponent is close to winning" for every Tier 6 cast (fixed)

`Situation::cmdr_combo` (an opposing commander on the battlefield is a modelled combo piece) fed `near` twice: the hold
value in `brain::hold_of` (+2.5) and the reserve penalty in `main_options` (8.0 instead of 0.7 x the hold value). A Tier 6
table always has Kinnan, Thrasios or Tymna out, so it read as "close to winning" all game. C1b only switched the engine rule.
Now, when `impls::t6_plans::is_t6(g, p)`, both read `combo_near` (the archenemy's real progress) alone; every other deck
reads what it read before (`combo_near || cmdr_combo` for the hold, `cmdr_combo` for the reserve).

**JD's decks.** `thrasios` is a Tier 6 key (`T6_KEYS`), so it is affected: its held counters (Force of Will, Flusterstorm,
Mental Misstep ...) are worth 2.5 less in the hold value, and it no longer refuses to tap out with an 8.0 reserve penalty
for any cast that is not ramp while a Kinnan, Thrasios or Tymna commander sits on an opposing battlefield, until the
archenemy reading says someone is close. The other eight (sheoldred, sauron, cloud, kefka, yshtola, alela, galadriel,
thanos) are not Tier 6 and read exactly what they read.

**Test.** `a_tier_6_deck_ignores_a_combo_piece_commander_for_its_hold` (Kraum/Tymna and `thrasios`: the hold value is the same
with `cmdr_combo` set or clear; the old code gave +1.39 and more), control `a_deck_outside_tier_6_still_reads_the_commander`
(Sheoldred: still a positive gap). The reserve penalty uses the same `near` flag (the third use at `main_options`).
Left alone, deliberately: `hold_value` still holds instant removal when `cmdr_combo` is set (`brain.rs` line ~217); that is
"which cards to hold", not `near`. See NEEDS JD.

## 5. Ad Nauseam's hold for Sheoldred (fixed)

`brain::hold_value` kept mana up for Ad Nauseam at the end of the opponent's turn only with room for four cards under the hand
cap (`plans::adnaus_room >= 4`) for Tier 6. The cast itself (`adnaus_cast_now`) already needs that room for every deck, so
Sheoldred with a full hand held mana for a spell it would not cast. The test now reads `is_t6 || key == "sheoldred"`.
Decks that play Ad Nauseam (jq on `data/decks.json`): JD's `sheoldred` and `thrasios`, and the pool decks
`kraum-tymna-blue-breach-midrange` and `rograkh-silas-turbo-breach` (all three Tier 6 keys, so they had the rule). No other
deck outside Tier 6 plays it (the retired Thanos five-colour list, `decklists/pool/retired/`, is not simulated).
**Test.** `sheoldred_holds_no_mana_for_ad_nauseam_with_a_full_hand` (hold value 7.0 on the old code with a ten-card hand, 0
now; positive with room).

## Which of JD's decks each change can affect
| change | decks affected |
|---|---|
| 1 Finale/GSZ held-tutor fix, line B Finale return | none of JD's (planner is Tier 6 only; `thrasios` has no Finale, GSZ or Witness); pool: Rograkh/Thrasios |
| 2 Whir (`xtutor` tag, planner, `whir`), Trophy Mage | none of JD's decks play Whir or Trophy Mage; pool: Kinnan (T6), and Urza (not T6) whose Whir now takes X = all spare mana |
| 4 `near` | `thrasios` (T6). No other JD deck |
| 5 Ad Nauseam hold | `sheoldred` (new); `thrasios` already had it |

## Files changed
`ai/brain.rs` (hold_of, hold_value, main_options, pay_x in the reserve), `ai/planner.rs` (castable_with, xtutor_rule, pay_x,
tutor_rule Trophy Mage, closer-plan guard), `impls/t6_combos_t6.rs` (line B), `impls/rules2.rs` (improvise_arts, whir),
`impls/t1.rs` (tutor_named takes the planned card), `data/card_overrides.toml` and `data/cards.json` (Whir tag),
`tests/t6_ai.rs` (9 tests: 7 fail on the old code, 2 controls).

## Checks run
- `cargo build --release --all-targets`: no warnings.
- `cargo test --release --no-fail-fast` over the 74 test files the brief names (t6_ai, c1_t6_lines, c1b_engines, ai_t6_*,
  engine_t6_*, cards_t6_*, ai_planner, ai, ai_flow, ai_eval, plans, plan_files_3, generic_paths, thrasios_ai, cards_sauron*,
  cards_sheoldred*, cards_cloud*, cards_kefka*, cards_yshtola, cards_alela, cards_galadriel, cards_thanos, plus ai_xtutor,
  t6_stats, t6_stops): 1221 passed, 0 failed.
- `t6_ai.rs` against the old source (source and data checked out from bf3f1899): the Finale (2), Whir (2), Sheoldred and
  Tier 6 `near` tests fail, the two controls pass. The Trophy Mage test was added afterwards and not run on the old code (the
  old `tutor_rule` returned `None` for any permanent, so no plan could exist).
