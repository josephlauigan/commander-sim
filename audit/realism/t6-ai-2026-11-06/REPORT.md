# t6-ai (2026-11-06): Tier 6 planner and hold fixes

Branch `t6-ai`, from c1b-engines (PR #276) on c1-t6-lines (PR #271). Brief: `rust/agents/t6-ai.md`. No sims were run
(single `--trace` games only); the coordinator runs the paired tests.

## NEEDS JD
(see the end of this file; the reply lists the same items)

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
