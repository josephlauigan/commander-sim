# Report: t6-ai kraum-tymna-blue-breach-midrange (branch t6-ai-kraum)

Files: `impls/t6_plan_kraum.rs` (all five slots), `tests/ai_t6_kraum.rs` (9 tests). No shared file touched.

## What the plan does
- keep: fast mana / engine / tutor / payoff counted ("action"); 1 land needs two fast sources or fast mana + engine/tutor;
  2 lands need action>=2 (or action + a counter); 3-4 lands keep with any action or a counter; 5 lands need action; a lone
  engine or lands + bounce is shipped.
- prio: fast mana 95, Esper Sentinel 90, Rhystic 88, Remora 84, Tymna 86, Kraum 72 only if a counter stays up (or the table is
  tapped out), else 20; Oracle, Consultation, Pact, Brain Freeze, Final Fortune, Silence, Chant and every counter 0 (held).
- wish: missing half of the Oracle pair; protection first if both halves are in hand with no cover; engine early; Ad Nauseam; Breach pieces.
- hold_up: develop on turns 1-2; keep counters up from turn 3-4 or when an opponent scores >=5 as archenemy.
- go_now: Oracle line waits if Kraum/Esper Sentinel is on my side and an opponent has a card and a land up, or if an opponent
  can stop it (stop_chance >= .3) and I hold no creature cover (Pact, FoW with 2 blue cards, Silence, Chant), unless round >= 5
  and someone else is about to win; Ad Nauseam needs life >= 26.

## Before / after (300 games per profile, `--calibrate within --pool t6 --games 300 --jobs 3`; conservative / loose)
| | before | after |
|---|---|---|
| win % | 29.7 / 32.8 | 24.6 / 24.6 |
| wins by | Oracle 89/91%, damage 7/9%, Breach 2% | Oracle 87/88%, damage 6/6%, Breach table 1-2%, unnamed combo (Breach+Oracle finish) 6/2% |
| win round avg / median | 8.8 / 7 ; 8.3 / 7 | 8.0 / 7 ; 9.1 / 9 |
| combo tries per game, stopped | 0.36, 27% ; 0.38, 23% | 0.28, 18% ; 0.28, 17% |
| tries by line | Oracle 67/72, Ad Nauseam 3, Breach 1/0 | Oracle 47/46, Ad Nauseam 4/5, Breach 4/3 |
| mulligans | 48.2% (16 triple) | 35.4% (8 triple) |
| counters resolved | 223 / 219 | 190 / 227 |
| stax cast | 11 / 11 | 10 / 10 |
Note: the before-run (fresh at branch start, 4 plans merged) differs from `audit/t6/stage5-baseline.txt`. The v1 run (stricter
keep) gave 22.6 / 20.0%; the loosened keep and a .3 stop threshold gave the table above. No simulator-error warnings.

## Differs from the spec / not done
- Win round median 7-9; the spec aims at turns 3-6. Oracle tries are mostly held until cover exists, so the deck wins later but
  stops less (23-27% -> 17-18%). Going on turn 2-3 under tapped-out tables (scenario 13) is left to the shared `go_ok`.
- Mulligans 35% (was 48%) is still above what the spec implies: the deck has 25 lands and the sim's 7-card hands often fail the
  "lands + a plan" test; I did not loosen further to avoid keeping lands-and-counters hands the spec says to ship.
- "Stopped by no card named": 4-6 per 195 games now. The Ad Nauseam ones (3-4 per run) are not answers: Ad Nauseam resolves but
  the cards drawn make no line ready (the framework records "?"); that is a model/line-choice matter, not a ward or lock. The Oracle ones
  (1-2) are a failed cast_hand (castable/can_pay false after Oracle's mana was spent on the wrong colours); I could not trace them
  without a Kraum trace mode (only `--deck godo` traces). Not fixed.
- Breach line: now tried 3-5 times per 195 games (was 1) because Oracle-without-cover is blocked; no Breach-vs-Oracle choice by
  the opponents' counters (scenario 18), Necropotence/Final Fortune lines (scenarios 8, 10) and Silence-vs-Chant ordering are
  not modeled in the plan. Scenarios 6, 2/17 (partly), 4/5, 13 (Ad Nauseam life), 1 are tested.
