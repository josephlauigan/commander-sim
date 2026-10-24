# Report t6-ai-godo (Stage 5, Godo)

Files: `impls/t6_plan_godo.rs` (all five slots), `tests/ai_t6_godo.rs` (7 tests), and one small tooling change:
`sim-cli/src/poolmode.rs` now prints the per-deck Tier 6 report block (the t6report.rs text, Godo's seat only) after
`--deck <t6 key or godo> --pool t6`. No extra games (RunTally got a `t6: T6Tally`, filled only for t6 keys).
No AI hook outside the plan file. t3 lock: identical. Full `cargo test --release` passes; build has no warnings.

## Before / after (300 games per profile, `--deck godo --pool t6`, development runs, not the baseline)

| | before cons / loose | after cons / loose |
|---|---|---|
| win % | 45.3 / 51.0 | 49.0 / 50.0 |
| wins by | damage 48-44%, combo lines 52-56% | damage 48%, combo lines 52% (Hammer-out Godo+Helm 11-13%, token copy spell 8-10%, Dualcaster 5%) |
| win round avg / median | 6.8 / 6 and 6.5 / 6 | 6.8 / 6 and 6.8 / 6 |
| combo tries per game, stopped | 0.37, 42% / 0.37, 26% | 0.35, 35% / 0.32, 26% |
| mulligans (share of hands) | 35.0 / 35.0 | 38.7 / 38.7 |

Differences are inside noise (95% CI about +-5.5 points). The deck was already strong; the plan mostly makes the
choices follow the spec rather than moving the number.

## What the slots do
- keep: needs lands>=1 (1 land only with two accelerators, or one plus ritual+tutor), at least 2 sources, <6 lands,
  AND a named path (tutor, Helm/Hammer, Ogre, Ring, Dualcaster + copier, Squire, Camera...). Ramp-only hands are
  shipped (spec scenario 16). A looser first version (counting Grim/Ogre/Metalworker as accelerators and keeping
  mana-rich hands) gave 17% mulligans and the same win rate; I went with the spec's stricter rule. Stage 6 may relax it.
- prio: Vexing Bauble and Trinisphere wait for the zero-drops; Blood Moon waits while Tomb/City are our mana and Godo is
  not out; The One Ring / Party Thrasher early; Final Fortune / Last Chance held until Godo is out.
- wish: Hammer first (when Hammer/Helm are not out), then Dualcaster if a copier is in hand, Swat once Godo is out,
  Squelcher, Pyroblast, then the combo's missing pieces.
- hold_up: develop (-2) on turns 1-3 while mana/ritual cards sit in hand and Godo is not out.
- go_now: shared `go_risk`, x0.8 per protection spell in hand (max 2), threshold 0.85 rounds 1-3, 0.7 rounds 4-5, always later.

## Still differs from the spec
- Half of the wins are still plain damage (48%); the Helm lines fire 0.32-0.35 times per game, mostly late (median round 6;
  the spec wants turns 3-6). Several games are lost to opponents' faster combos before Godo has mana; no plan for racing.
- Not done: Squelcher-before-Godo sequencing (scenario 1), Pyroblast held for the Godo cast / not spent on draw spells (2, 22),
  Magistrate/Agent removal order (19), Blind Obedience removal first (6, 23), Chalice at 1 (7), Misstep-aware Squire vs Theft (8),
  Ogre same-turn Recruiter (14), indestructible-blocker attack order (20), Defense Grid + Final Fortune combo turn (21).
  These need hooks in the cast/target code (outside the plan slots) or opponent board reads; the slots cannot express them.
- Basalt + Kinnan copy (line 8) and Welder + Engineer sequencing were not touched.
