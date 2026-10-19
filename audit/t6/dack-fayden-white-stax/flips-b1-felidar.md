# Dack Fayden first-flip traces, batch b1: piles 6-15 (Felidar Guardian, no Preston)

Notation: Fe Felidar, Ab Abdel, St Stalker, FH Fiend Hunter, Pr Preston, So Solar, Wf Werefox. "b1" = first flip (batch 1),
"b2" = Dack's second ETB. T_X = X's ETB trigger (mine). X' = Preston token copy of X (mine). "hold" = banisher exiles it.
All b1 creatures are donated, so I control only Dack. Fe's trigger can target only a permanent I control: Dack (or a land).

## Common facts used in all ten piles
- Fe T_F targets DACK (mine): Dack leaves, returns, ETB flips 3 of the 4 creatures not in the pile = SECOND FLIP "b2".
  Which one stays in the library is uniform: 4 equally likely cases (exclude Pr / exclude each of the other three).
- No pile wins on b1 alone (no Preston, so no token engine; a banisher can't be removed by me). So every pile is
  NEEDS-2ND-FLIP, and the second flip is guaranteed to happen (via Fe), only its content is random.
- b2 creatures enter together, so Pr (if in b2) triggers for the other two (P-triggers, mine) and each real ETB is mine.
  Real ETB targets are chosen when placed (b2 real ETBs cannot target b2 tokens; token ETBs can target anything then in play).
- ENGINE (the win): get Preston under my control by exile-return, THEN get Fe real back from exile under my control.
  Fe re-enters not cast -> Preston makes Fe' token -> Fe' ETB flickers Fe real -> Fe re-enters -> repeat. Unbounded.
  Each cycle Fe real's own ETB also gives a free flicker of any other permanent I control (e.g. Dack for a 3rd flip).
- Solar: if Pr and So are both in b2, Solar' token is mine and ventures for every Fe re-entry -> drain 1 per 4 cycles, win.
  Otherwise fetch Solar: Preston ability {1}{W}, sac 5 Illusions (I have exactly 2 {W}) exiles the opponent-held banisher H
  that holds a banisher R of mine; R returns to me, its ETB exiles Solar, a free Fe flicker of R returns Solar to me.
  If Solar is the card left in the library, the free Fe flicker of Dack flips it first (3rd flip, last creature).
- Recipes for the two returns (Pr first, then Fe):
  PLAN-A (two banishers R1,R2 both in b2 with Pr): order Pr-hold: R1' exiles Pr; R2' exiles R1' (token vanishes, Pr
    returns to me). Then T_R1 real exiles Fe; T_R2 real exiles R1 real -> Fe returns to me. Any ordered pair of
    FH/St/Wf is legal (St can't hit Assassins, Wf can't hit Foxes; distinct pairs avoid both).
  PLAN-B (Abdel in b2 or b1): R' (a banisher token) exiles Pr; Abdel's trigger (or Abdel' token) exiles R' -> Pr returns.
  PLAN-C (Fe return): a banisher H already in play exiles Fe (its trigger targets Fe at placement, i.e. b1 time for a b1
    banisher), and a different banisher's ETB (b2 real or b1 trigger held back on the stack) exiles H AFTER Pr is mine.
  b1 triggers can be left at the bottom of the stack while Fe's flicker and all b2 triggers resolve first.
- Doubts (all piles): (1) goad ignored; (2) cases with Pr left in the library (exclude-Pr) judged NO-WIN: Pr then
  arrives alone on a 3rd flip, and no banisher ETB can target it afterwards (my banisher ETBs are all placed earlier);
  I did not find a counting-proof, only no line. (3) Treated "Preston still triggers after being exiled" as true because
  the P-triggers are already on the stack.

## Pile 6: Fe + Ab + St  —  NEEDS-2ND-FLIP (3 of 4)
Trace:
1. b1 enters, all three donated. My triggers: T_F (target Dack), T_A (Abdel: I choose what to exile at resolution), T_St (up to one).
2. T_St: choose no target (or Fe/Ab, pointless). Keep T_A and T_St at the bottom; T_F on top.
3. T_F: Dack flickers, D2 flips 3 of {Pr, FH, So, Wf}. Donated; P-triggers (if Pr) and real ETBs go on top.
4. Pr in b2 with two of FH/So/Wf: (FH,Wf) PLAN-A. (FH,So): FH real exiles Pr; FH' exiles FH real (Pr returns to me);
   T_A exiles FH' (FH real returns to me, not cast -> FH'' token + T_FH2 exiles Fe; FH'' exiles FH real -> Fe returns to me).
   (So,Wf): Wf' exiles Pr, T_A exiles Wf' (Pr mine); St real (T_St placed at b1) exiled Fe, Wf real exiles St real -> Fe mine.
   For this T_St must target Fe at b1 time (do that instead of step 2; legal, Fe is non-Assassin).
5. Engine runs. Solar' token mine when So in b2; for (FH,Wf) fetch Solar with the Preston ability.
Loop: Fe re-enters / Fe' flickers Fe real, Illusion per cycle, Solar venture per cycle.
With a flicker spell: only helps the exclude-Pr case? No: no flicker line makes exclude-Pr win. Not needed otherwise.
Second flip: 3 of 4 win (exclude FH, Wf or So); excluding Pr fails.
Doubts: exclude-Pr; Abdel's Dack chain (T_A exiling Dack, St exiling Ab) gives a 3rd flip but only brings Pr alone.

## Pile 7: Fe + Ab + FH  —  NEEDS-2ND-FLIP (3 of 4)
Trace:
1. Triggers: T_F (Dack), T_A, T_FH. T_FH targets Fe at b1 time (FH real holds Fe; returns when FH leaves). Hold T_FH, T_A low.
2. T_F: D2 flips 3 of {Pr, St, So, Wf}.
3. (St,So): St' exiles Pr, T_A exiles St' (Pr mine); then T_St (real) exiles FH real -> FH LTB returns Fe to me. Engine.
   (So,Wf): Wf' exiles Pr, T_A exiles Wf' (Pr mine); T_Wf exiles FH real -> Fe back. Engine.
   (St,Wf): PLAN-A, or same as above using FH for Fe.
4. Solar' mine when So in b2; for (St,Wf) fetch via Preston ability on the banisher holding FH real.
Loop: Fe / Fe' flicker cycle with a Solar venture each time.
With a flicker spell: none improves exclude-Pr.
Second flip: 3 of 4 (exclude St, So or Wf).
Doubts: exclude-Pr.

## Pile 8: Fe + Ab + So  —  NEEDS-2ND-FLIP (3 of 4)
Trace:
1. b1: Solar triggers 3 times for itself, Fe and Ab (ventures: rooms 1-3, one drain for 1). T_F (Dack), T_A. No banisher.
2. T_F: D2 flips 3 of {Pr, St, FH, Wf}. Any case with Pr has two banishers: PLAN-A (pairs (St,FH),(St,Wf),(FH,Wf)).
3. Engine. Solar is real in b1 (opp's), no Solar': use Preston ability on R2 real (holds R1 real) -> R1 returns, exiles Solar,
   free Fe flicker returns Solar to me. Each Fe re-entry then ventures.
Loop: Fe cycle plus Solar venture.
With a flicker spell: none helps exclude-Pr.
Second flip: 3 of 4 (every case that keeps Pr).
Doubts: exclude-Pr (S = St,FH,Wf all banishers but Pr arrives alone later).

## Pile 9: Fe + Ab + Wf  —  NEEDS-2ND-FLIP (3 of 4)
Trace:
1. Triggers: T_F (Dack), T_A, T_Wf. T_Wf targets Fe at b1 (Wf real holds Fe).
2. T_F: D2 flips 3 of {Pr, St, FH, So}.
3. (St,FH): PLAN-A. (St,So): St' exiles Pr, T_A exiles St'; T_St exiles Wf real -> Fe returns. (FH,So): FH' exiles Pr,
   T_A exiles FH'; T_FH exiles Wf real -> Fe returns. Engine; Solar' in the So cases, otherwise Preston ability fetch.
Loop: Fe cycle plus Solar venture.
With a flicker spell: none helps exclude-Pr.
Second flip: 3 of 4.
Doubts: exclude-Pr.

## Pile 10: Fe + St + FH  —  NEEDS-2ND-FLIP (1 of 4)
Trace:
1. Triggers: T_F (Dack), T_St, T_FH. Fe return is free: T_St targets Fe (hold); T_FH (held back) later exiles St real -> Fe returns.
   Both b1 triggers must stay on the stack until Pr is mine.
2. T_F: D2 flips 3 of {Pr, Ab, So, Wf}.
3. Pr must be held by a banisher ETB that enters in b2 (b1 ETB targets were fixed before Pr existed). Only Wf can.
   (Ab,Wf): Wf' exiles Pr, Abdel (real T or Ab') exiles Wf' -> Pr mine; then T_St (hold Fe), T_FH exiles St -> Fe mine. Engine;
   Solar is the card left: free Fe flicker of Dack flips it, Preston ability fetch (FH real holds St real).
   (Ab,So): no banisher in b2 to hold Pr -> fail. (So,Wf): Wf real can hold Pr but nothing can remove Wf real; Wf' holding Pr has
   no remover -> fail.
Loop: Fe cycle plus Solar venture.
With a flicker spell: Cloudshift/Ephemerate ({W}) or Touch channel ({1}{W}) on my token Wf' removes it and returns Pr, so
   (So,Wf) also wins (Solar' token mine, no Preston ability needed): 2 of 4. (Ab,So) still fails.
Second flip: 1 of 4 (exclude So only).
Doubts: spell lines assume token exile returns Pr immediately (token ceases, Wf' "until" ends).

## Pile 11: Fe + St + So  —  NEEDS-2ND-FLIP (3 of 4)
Trace:
1. b1: Solar ventures 3 times (one drain). T_F (Dack), T_St targets Fe (holds Fe; held back).
2. T_F: D2 flips 3 of {Pr, Ab, FH, Wf}.
3. (Ab,FH): FH' exiles Pr, Abdel exiles FH' (Pr mine); T_FH real exiles St real -> Fe returns. (Ab,Wf): Wf' exiles Pr, Abdel
   exiles Wf'; T_Wf exiles St real -> Fe returns. (FH,Wf): PLAN-A. Engine.
4. Solar real is b1 (opp's): Preston ability exiles the opp banisher holding St real; St returns, exiles Solar; free Fe flicker returns it.
Loop: Fe cycle plus Solar venture.
With a flicker spell: none helps exclude-Pr.
Second flip: 3 of 4.
Doubts: exclude-Pr.

## Pile 12: Fe + St + Wf  —  NEEDS-2ND-FLIP (2 of 4)
Trace:
1. Triggers: T_F (Dack), T_St, T_Wf. Fe return is free via b1: T_St exiles Fe, T_Wf (held back) exiles St real later.
2. T_F: D2 flips 3 of {Pr, Ab, FH, So}.
3. Need a b2 banisher to hold Pr: FH only. (Ab,FH): FH' exiles Pr, Abdel exiles FH' -> Pr mine, then T_Wf exiles St -> Fe mine.
   (FH,So): FH real exiles Pr, FH' exiles FH real (Pr mine, FH'' not needed), then b1 pair returns Fe. Solar' mine.
   (Ab,So): no banisher in b2 -> fail.
4. In (Ab,FH) Solar is left: free Fe flicker of Dack flips it; Preston ability fetch (Wf real holds St real).
Loop: Fe cycle plus Solar venture.
With a flicker spell: no change (Ab,So has no banisher to hold Pr).
Second flip: 2 of 4 (exclude So or Wf... i.e. exclude Ab or exclude So).
Doubts: exclude-Pr.

## Pile 13: Fe + FH + So  —  NEEDS-2ND-FLIP (3 of 4)
Trace:
1. b1: Solar ventures 3 times. T_F (Dack), T_FH targets Fe (hold, held back).
2. T_F: D2 flips 3 of {Pr, Ab, St, Wf}.
3. (Ab,St): St' exiles Pr, Abdel exiles St'; T_St exiles FH real -> Fe returns. (Ab,Wf): same with Wf. (St,Wf): PLAN-A.
4. Fetch Solar: Preston ability exiles St/Wf real holding FH real; FH returns, exiles Solar, free Fe flicker returns it.
Loop: Fe cycle plus Solar venture.
With a flicker spell: none helps exclude-Pr.
Second flip: 3 of 4.
Doubts: exclude-Pr.

## Pile 14: Fe + FH + Wf  —  NEEDS-2ND-FLIP (1 of 4)
Trace:
1. Triggers: T_F (Dack), T_FH (hold Fe), T_Wf (held back; exiles FH real later). Fe return free.
2. T_F: D2 flips 3 of {Pr, Ab, St, So}.
3. Pr needs a b2 banisher: only St. (Ab,St): St' exiles Pr, Abdel exiles St' -> Pr mine, then T_Wf exiles FH -> Fe mine. Engine;
   Solar left in the library: 3rd flip via free Dack flicker, fetch with Preston ability.
   (Ab,So): no banisher -> fail. (St,So): St' holds Pr but nothing can remove St' (St real can't hit Assassins) -> fail.
Loop: Fe cycle plus Solar venture.
With a flicker spell: Cloudshift/Ephemerate/Touch channel on St' returns Pr, so (St,So) also wins: 2 of 4.
Second flip: 1 of 4 (exclude So).
Doubts: exclude-Pr.

## Pile 15: Fe + So + Wf  —  NEEDS-2ND-FLIP (3 of 4)
Trace:
1. b1: Solar ventures 3 times. T_F (Dack), T_Wf targets Fe (hold, held back).
2. T_F: D2 flips 3 of {Pr, Ab, St, FH}.
3. (Ab,St): St' exiles Pr, Abdel exiles St'; T_St exiles Wf real -> Fe returns. (Ab,FH): same with FH. (St,FH): PLAN-A.
4. Fetch Solar with Preston ability (St/FH real holds Wf real).
Loop: Fe cycle plus Solar venture.
With a flicker spell: none helps exclude-Pr.
Second flip: 3 of 4.
Doubts: exclude-Pr.

## Summary
| Pile | Outcome | Reason |
|---|---|---|
| 6 Fe+Ab+St | NEEDS-2ND-FLIP, 3/4 | Fe flips again; any b2 with Pr works (T_A or two banishers return Pr then Fe) |
| 7 Fe+Ab+FH | NEEDS-2ND-FLIP, 3/4 | FH holds Fe from b1; Abdel returns Pr in b2 |
| 8 Fe+Ab+So | NEEDS-2ND-FLIP, 3/4 | b2 with Pr always has 2 banishers (PLAN-A); Solar fetched |
| 9 Fe+Ab+Wf | NEEDS-2ND-FLIP, 3/4 | Wf holds Fe from b1; Abdel returns Pr |
| 10 Fe+St+FH | NEEDS-2ND-FLIP, 1/4 | only Ab+Wf in b2 can hold and release Pr (2/4 with a flicker spell) |
| 11 Fe+St+So | NEEDS-2ND-FLIP, 3/4 | St holds Fe from b1; Pr returned via Ab or two banishers |
| 12 Fe+St+Wf | NEEDS-2ND-FLIP, 2/4 | b1 pair frees Fe; needs FH in b2 to hold Pr (Ab+FH, FH+So) |
| 13 Fe+FH+So | NEEDS-2ND-FLIP, 3/4 | FH holds Fe from b1; Pr returned via Ab+St/Wf or St+Wf |
| 14 Fe+FH+Wf | NEEDS-2ND-FLIP, 1/4 | only Ab+St works (2/4 with a flicker spell) |
| 15 Fe+So+Wf | NEEDS-2ND-FLIP, 3/4 | Wf holds Fe from b1; Pr returned via Ab+St/FH or St+FH |
Counts: WIN-NOW 0, WIN+1 0, NEEDS-2ND-FLIP 10 (win chances 75,75,75,75,25,75,50,75,25,75 %), NO-WIN 0.
