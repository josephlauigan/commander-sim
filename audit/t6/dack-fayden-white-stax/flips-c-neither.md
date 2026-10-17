# Dack Fayden first-flip traces, group C (piles 26-35, neither Preston nor Felidar)

Method notes (rules sheet facts only):
- Notation: Ab Abdel, St Stalker, FH Fiend Hunter, So Solar, Wf Werefox, Pr Preston, Fe Felidar. "X holds Y" = X exiled Y and Y returns (under its owner's control = me, not cast) when X leaves.
- Without Pr and Fe in the pile, nothing in the first batch can loop by itself unless the pile has a 3-cycle (St+FH+Wf), and nothing drains without a Solar I control. Solar is donated, so I need a way to own it (a Preston token copy, or capture and release).
- 3-cycle (St, FH, Wf): state "Z holds Y (Y exiled), X's fresh ETB targets Z". X exiles Z, Z leaves so Y returns, Y exiles X, X leaves so Z returns, Z exiles Y, Y leaves so X returns, X exiles Z ... Every banisher re-enters under my control each lap, so a Solar I control ventures every entry.
- Second flip is reached by "banisher/Abdel holds Dack, then that holder is exiled by another banisher" (Dack returns, new ETB).

Shared tools used below (all rules from the sheet):
- T1 (second flip, no spell): banisher Y exiles Dack, banisher Z exiles Y. Y leaves, Dack returns under my control, its new ETB flips 3 more. Z now holds Y (a banisher). Needs two banishers, or Abdel + one banisher (Ab exiles Dack, banisher exiles Ab; the banisher then holds Ab).
- Capture + release of the donated Solar: a token banisher (Preston copy, mine) exiles Solar; a second token (Abdel token, or any banisher token) exiles it, or Felidar's token flickers it; a token that leaves ceases, so Solar returns to me. A Preston copy of Solar itself (Pr and So flipped together) is simpler: the token venture trigger is mine for good.
- Felidar in a flip: its own (nontoken) ETB targets Dack, so Dack flickers and flips the last creature in the library (third flip). That is how a banisher left out of the second flip still arrives.
- Lap trick (needs Cloudshift or Ephemerate): on a lap where a banisher of mine re-enters, its fresh ETB exiles the donated Solar; flicker that banisher, Solar comes back to me and the banisher re-enters with a fresh ETB that carries on the cycle.
- No first flip in this group has Pr or Fe, so none can loop by itself; a second flip is always needed. All 10 piles are NEEDS-2ND-FLIP. "k of 4" below = how many of the four possible second flips (one remaining creature is left out, each equally likely) have a winning line.

## Pile 26: Ab + St + FH  —  NEEDS-2ND-FLIP
Trace:
1. Pile enters under me, then Ab, St, FH go to the opponents. My triggers: Ab ETB, St ETB, FH ETB.
2. Targets: St -> Dack, FH -> St, Ab -> nothing (it can only exile MY nonland permanents).
3. Resolve St: Dack exiled until St leaves. Resolve FH: St exiled, FH holds St. St leaves, Dack returns under my control, its ETB flips 3 more (T1).
4. Board: FH (donated) holds St (exiled), Ab donated, Dack mine. Library has Pr, Fe, So, Wf.
5. Second flip (3 of {Pr,Fe,So,Wf}), Wf is the third banisher. Cycle used: Wf exiles FH (FH leaves, its trigger returns St to me), St exiles Wf, Wf leaves so FH returns, FH exiles St, St leaves so Wf returns, Wf exiles FH ... every lap re-enters nontoken creatures under me.
   - omit Wf {Pr,Fe,So}: Pr copies So and Fe. Solar token S' is mine. Fe token flickers Dack, third flip brings Wf, Wf exiles FH, cycle runs, S' ventures each lap. WIN.
   - omit Fe {Pr,So,Wf}: S' mine, Wf's pending ETB (-> FH) starts the cycle. WIN.
   - omit So {Pr,Fe,Wf}: Fe's own ETB flickers Dack (third flip brings Solar, donated). Wf token exiles Solar, Fe token flickers Wf token (it ceases, Solar returns to me). Then Wf's ETB (-> FH) starts the cycle. WIN.
   - omit Pr {Fe,So,Wf}: cycle runs but Solar stays donated and nothing I have can capture it. No win found.
Loop: none from the pile itself (only two banishers).
With a flicker spell: Cloudshift or Ephemerate makes the omit-Pr flip win too (lap trick above), so all 4 flips win. Touch the Spirit Realm: no help (an end-step return can't coexist with an unbounded cycle).
Second flip: 3 of 4 (fails only when Preston is the card left out).
Doubts: FH's leave trigger is controlled by the opponent holding FH (mandatory, assumed to resolve). Lost Mine draws a card every 4 ventures, ~40 draws for 160 ventures (library size should be checked). "No win found" is not a proof of NO-WIN.

## Pile 27: Ab + St + So  —  NEEDS-2ND-FLIP
Trace:
1. Pile enters under me, then donated. Triggers (mine): Ab ETB, St ETB, 3 Solar ventures (Solar sees itself, Ab, St): rooms 1-3 of Lost Mine, Dark Pool drains 1 (finite).
2. Targets: Ab exiles Dack (I get a Soldier); St -> Ab.
3. Resolve Ab first (Dack exiled until Ab leaves), then St (Ab exiled, St holds Ab). Ab leaves, Dack returns, ETB flips 3 more.
4. Board: St (donated) holds Ab; Solar donated; Dack mine. Library: Pr, Fe, FH, Wf.
5. Second flip. Cycle with St, FH, Wf: FH exiles St (Ab returns to me), Wf exiles FH (FH leaves, its trigger returns St), St exiles Wf, Wf leaves so FH returns, FH exiles St, ... Stack the nontoken ETBs FH -> St and Wf -> FH under everything and resolve tokens first.
   - omit Fe {Pr,FH,Wf}: tokens FH', Wf'. Wf' exiles Solar, FH' exiles Wf' (Wf' ceases, Solar returns to me; or sacrifice Wf' for {1}{W}). Then FH, Wf nontoken ETBs start the cycle with Solar mine. WIN.
   - omit FH {Pr,Fe,Wf}: Wf (-> St) pending, Wf' exiles Solar, Fe token flickers Wf' (Solar back to me), Fe's own ETB flickers Dack (third flip brings FH), FH exiles Wf, cycle. WIN.
   - omit Wf {Pr,Fe,FH}: same with FH: FH (-> St) pending, FH' exiles Solar, Fe token flickers FH', Fe ETB flickers Dack, Wf arrives and exiles FH, cycle. WIN.
   - omit Pr {Fe,FH,Wf}: the cycle starts (and Ab comes back once) but Solar stays donated, no extra exile to capture it. No win found.
Loop: none from the pile (one banisher; Abdel only moves my own permanents).
With a flicker spell: Cloudshift or Ephemerate fixes omit-Pr by the lap trick, so all 4 flips win. Touch: no.
Second flip: 3 of 4 (fails only if Preston is left out).
Doubts: order of triggers matters: the Fe/Pr triggers have to be stacked below or above the banisher ETBs exactly as above (all are my triggers, I order them). Same draw-count and FH-trigger notes as pile 26.

## Pile 28: Ab + St + Wf  —  NEEDS-2ND-FLIP
Trace:
1. Pile enters under me, donated. Triggers: Ab ETB, St ETB, Wf ETB.
2. Targets: St -> Dack, Wf -> St, Ab -> nothing useful.
3. Resolve St (Dack exiled), then Wf (St exiled, Wf holds St); St leaves so Dack returns, flips 3 more (T1). Ab is a spare body.
4. Board: Wf (donated) holds St, Ab donated, Dack mine. Library: Pr, Fe, So, FH.
5. Second flip, FH is the third banisher. Cycle: FH exiles Wf (Wf leaves, St returns to me), St exiles FH (FH leaves, its trigger returns Wf), Wf exiles St, St leaves so FH returns, FH exiles Wf, ...
   - omit FH {Pr,Fe,So}: Solar token S' mine, Fe token flickers Dack, third flip brings FH, FH exiles Wf, cycle, S' ventures. WIN.
   - omit Fe {Pr,So,FH}: S' mine, FH's pending ETB (-> Wf) starts the cycle. WIN.
   - omit So {Pr,Fe,FH}: Fe ETB flickers Dack (third flip brings Solar), FH token exiles Solar, Fe token flickers FH' (token ceases, Solar returns), FH (-> Wf) starts the cycle. WIN.
   - omit Pr {Fe,So,FH}: cycle runs, Solar donated, nothing to capture it. No win found.
Loop: none from the pile.
With a flicker spell: Cloudshift or Ephemerate (lap trick) makes omit-Pr win: 4 of 4. Touch: no.
Second flip: 3 of 4 (fails only if Preston is left out).
Doubts: as pile 26. Wf is a Fox: Wf cannot target itself or a Fox, nothing in the cycle needs it to.

## Pile 29: Ab + FH + So  —  NEEDS-2ND-FLIP
Trace:
1. Pile enters under me, donated. Triggers (mine): Ab ETB, FH ETB, 3 Solar ventures (finite, one drain).
2. Targets: Ab exiles Dack; FH -> Ab.
3. Resolve Ab (Dack exiled until Ab leaves), then FH (Ab exiled, FH holds Ab). Ab leaves, Dack returns, flips 3 more.
4. Board: FH (donated) holds Ab; Solar donated; Dack mine. Library: Pr, Fe, St, Wf.
5. Second flip. Cycle: St exiles FH (FH leaves, trigger returns Ab), Wf exiles St (St leaves, FH returns to me), FH exiles Wf, Wf leaves so St returns, St exiles FH, ...
   - omit Fe {Pr,St,Wf}: tokens St', Wf'. Wf' exiles Solar, St' exiles Wf' (Solar returns to me). Nontoken St (-> FH) and Wf (-> St) then start the cycle. WIN.
   - omit St {Pr,Fe,Wf}: Wf (-> FH) pending, Wf' exiles Solar, Fe token flickers Wf', Fe ETB flickers Dack (third flip brings St), St exiles Wf, cycle. WIN.
   - omit Wf {Pr,Fe,St}: St (-> FH) pending, St' exiles Solar, Fe token flickers St', Fe ETB flickers Dack (third flip brings Wf), Wf exiles St, cycle. WIN.
   - omit Pr {Fe,St,Wf}: cycle runs, Solar donated, no spare exile. No win found.
Loop: none from the pile.
With a flicker spell: Cloudshift or Ephemerate (lap trick) fixes omit-Pr: 4 of 4. Touch: no.
Second flip: 3 of 4 (fails only if Preston is left out).
Doubts: FH holds Ab through its leave trigger; when St exiles FH the trigger belongs to the opponent who controls FH (mandatory). Draw count as pile 26.

## Pile 30: Ab + FH + Wf  —  NEEDS-2ND-FLIP
Trace:
1. Pile enters under me, donated. Triggers: Ab ETB, FH ETB, Wf ETB.
2. Targets: FH -> Dack, Wf -> FH, Ab -> nothing useful.
3. Resolve FH (Dack exiled), then Wf (FH exiled, Wf holds FH). FH's leave trigger (the opponent's, mandatory) returns Dack: second flip (T1).
4. Board: Wf (donated) holds FH, Ab donated, Dack mine. Library: Pr, Fe, So, St.
5. Second flip, St is the third banisher. Cycle: St exiles Wf (Wf leaves, FH returns to me), FH exiles St (St leaves, Wf returns), Wf exiles FH (FH leaves, trigger returns St), ... St never targets itself.
   - omit St {Pr,Fe,So}: S' mine, Fe token flickers Dack, third flip brings St, St exiles Wf, cycle. WIN.
   - omit Fe {Pr,So,St}: S' mine, St's pending ETB (-> Wf) starts the cycle. WIN.
   - omit So {Pr,Fe,St}: Fe ETB flickers Dack (third flip brings Solar), St token exiles Solar, Fe token flickers St' (Solar back), St (-> Wf) starts the cycle. WIN.
   - omit Pr {Fe,So,St}: Solar stays donated. No win found.
Loop: none from the pile.
With a flicker spell: Cloudshift or Ephemerate (lap trick) fixes omit-Pr: 4 of 4. Touch: no.
Second flip: 3 of 4 (fails only if Preston is left out).
Doubts: as pile 26.

## Pile 31: Ab + So + Wf  —  NEEDS-2ND-FLIP
Trace:
1. Pile enters under me, donated. Triggers (mine): Ab ETB, Wf ETB, 3 Solar ventures.
2. Targets: Ab exiles Dack; Wf -> Ab (Ab is not a Fox).
3. Resolve Ab, then Wf: Ab leaves, Dack returns, flips 3 more.
4. Board: Wf (donated) holds Ab; Solar donated; Dack mine. Library: Pr, Fe, St, FH.
5. Second flip. Cycle: St exiles Wf (Ab returns), FH exiles St (St leaves, Wf returns), Wf exiles FH (FH leaves, trigger returns St), ...
   - omit Fe {Pr,St,FH}: tokens St', FH'. FH' exiles Solar, St' exiles FH' (it ceases, its trigger returns Solar to me). Nontoken St (-> Wf) and FH (-> St) start the cycle. WIN.
   - omit St {Pr,Fe,FH}: FH (-> Wf) pending, FH' exiles Solar, Fe token flickers FH', Fe ETB flickers Dack (third flip brings St), St exiles FH, cycle. WIN.
   - omit FH {Pr,Fe,St}: St (-> Wf) pending, St' exiles Solar, Fe token flickers St', Fe ETB flickers Dack (third flip brings FH), FH exiles St, cycle. WIN.
   - omit Pr {Fe,St,FH}: Solar donated, no spare exile. No win found.
Loop: none from the pile.
With a flicker spell: Cloudshift or Ephemerate (lap trick) fixes omit-Pr: 4 of 4. Touch: no.
Second flip: 3 of 4 (fails only if Preston is left out).
Doubts: Wf's sacrifice ability is the opponent's while Wf is donated; once Wf is back under me ({1}{W}, sac) it would release Ab/another prisoner but I don't need it.

## Pile 32: St + FH + So  —  NEEDS-2ND-FLIP
Trace:
1. Pile enters under me, donated. Triggers (mine): St ETB, FH ETB, 3 Solar ventures.
2. Targets (T1): St -> Dack, FH -> St. Resolve St, then FH: St leaves, Dack returns, flips 3 more. FH holds St.
3. Board: FH (donated) holds St; Solar donated; Dack mine. Both banisher actions are spent, so Solar cannot be captured as well. Library: Pr, Fe, Ab, Wf.
4. Second flip, Wf is the third banisher. Cycle: Wf exiles FH (FH leaves, trigger returns St to me), St exiles Wf, Wf leaves so FH returns, FH exiles St, St leaves so Wf returns, Wf exiles FH, ... Solar must be mine, which needs Preston tokens:
   - omit Fe {Pr,Ab,Wf}: Wf' exiles Solar, Abdel token (Ab') exiles Wf' (mine; it ceases, Solar returns to me). Ab and Wf nontoken ETBs: Ab exiles nothing, Wf (-> FH) starts the cycle. WIN.
   - omit Ab {Pr,Fe,Wf}: Wf' exiles Solar, Fe token flickers Wf' (Solar back). Wf (-> FH) starts the cycle. WIN.
   - omit Wf {Pr,Fe,Ab}: Fe flickers Dack, Wf arrives in the third flip and the cycle starts, but Pr is donated so there is no token to capture Solar. No win found.
   - omit Pr {Fe,Ab,Wf}: cycle starts, Solar donated, no spare exile. No win found.
Loop: none from the pile.
With a flicker spell: any of the three wins deterministically. Cloudshift or Ephemerate: spend St -> Solar and FH -> St first (Solar returns to me, FH holds St), cast it on Dack for the second flip; if Wf is in the flip it starts the cycle with Solar mine, if not Fe flickers Dack and Wf arrives in the third flip (the flip is then Pr, Fe, Ab, which has Fe). Touch the Spirit Realm channel ({1}{W}) on Dack does the same at the end step. So 4 of 4 with a spell.
Second flip: 2 of 4 (needs Preston and Wf in the flip: omit Fe or omit Ab).
Doubts: Ab' (token of a legendary) and the donated Ab are under different controllers, so no legend-rule issue. Draw count as pile 26.

## Pile 33: St + FH + Wf  —  NEEDS-2ND-FLIP
Trace:
1. Pile enters under me, donated. Triggers: St, FH, Wf ETBs (no drain source anywhere).
2. Targets: St -> Dack, Wf -> St, FH -> Wf. Order: St, then Wf, then FH last (FH stays pending on the stack until after the second flip).
3. St exiles Dack. Wf exiles St: St leaves, Dack returns, its ETB flips 3 more (batch 2 resolves before FH's pending ETB).
4. When FH resolves it exiles Wf: Wf leaves, St returns to me, St exiles FH (FH's trigger returns Wf), Wf exiles St, St leaves so FH returns, FH exiles Wf, ... an unbounded 3-cycle of nontoken entries. With no Solar I control nothing drains.
5. Second flip (3 of {Pr,Fe,So,Ab}) must give me a Solar before the cycle starts (resolve batch 2 first, FH last):
   - omit Ab {Pr,Fe,So}: Pr copies So and Fe: Solar token S' mine. WIN.
   - omit Fe {Pr,So,Ab}: S' mine. WIN.
   - omit Pr {Fe,So,Ab}: Solar donated, no banisher left to capture it. No win found.
   - omit So {Pr,Fe,Ab}: Fe flickers Dack, Solar arrives in the third flip, donated, Pr donated too (no token). No win found.
Loop: yes, St/Wf/FH 3-cycle (above) but with no drain, so no win from the first flip alone.
With a flicker spell: Cloudshift or Ephemerate wins all four second flips: after the cycle starts, on a lap where a banisher of mine re-enters, its ETB exiles the donated Solar (in omit-So, Fe's ETB flicker brings Solar first), then flicker that banisher: Solar returns to me and the cycle continues. Touch: no line found.
Second flip: 2 of 4 (needs Preston and Solar together: omit Ab or omit Fe).
Doubts: with FH pending, the three triggers stay on the stack in this order (St, Wf above, FH at the bottom); Dack's flip and all its tokens/triggers resolve before FH. The stale-trigger rule (a banisher exiled before its own ETB resolves does nothing, FH would exile permanently) was avoided by this ordering.

## Pile 34: St + So + Wf  —  NEEDS-2ND-FLIP
Trace:
1. Pile enters under me, donated. Triggers (mine): St ETB, Wf ETB, 3 Solar ventures.
2. Targets (T1): St -> Dack, Wf -> St. St resolves (Dack exiled), then Wf (St exiled, Wf holds St); St leaves, Dack returns, flips 3 more.
3. Board: Wf (donated) holds St; Solar donated; Dack mine. Library: Pr, Fe, Ab, FH.
4. Second flip, FH is the third banisher. Cycle: FH exiles Wf (Wf leaves, St returns to me), St exiles FH (FH leaves, trigger returns Wf), Wf exiles St, St leaves so FH returns, FH exiles Wf, ...
   - omit Fe {Pr,Ab,FH}: FH' exiles Solar, Ab' (token) exiles FH' (it ceases, its trigger returns Solar to me). FH (-> Wf) pending starts the cycle. WIN.
   - omit Ab {Pr,Fe,FH}: FH' exiles Solar, Fe token flickers FH' (Solar back). FH (-> Wf) starts the cycle. WIN.
   - omit FH {Pr,Fe,Ab}: Fe flickers Dack, FH arrives in the third flip, cycle runs, but Pr is donated (no token) so Solar is not captured. No win found.
   - omit Pr {Fe,Ab,FH}: Solar donated, no spare exile. No win found.
Loop: none from the pile (two banishers).
With a flicker spell: Cloudshift or Ephemerate, or Touch channel on Dack: first St -> Solar and Wf -> St (Solar returns to me, Wf holds St), then flicker Dack (Touch: at the end step). The flip brings FH (or Fe flickers Dack again for it) and the cycle runs with Solar mine. 4 of 4.
Second flip: 2 of 4 (needs Preston and FH: omit Fe or omit Ab).
Doubts: as pile 32.

## Pile 35: FH + So + Wf  —  NEEDS-2ND-FLIP
Trace:
1. Pile enters under me, donated. Triggers (mine): FH ETB, Wf ETB, 3 Solar ventures.
2. Targets (T1): FH -> Dack, Wf -> FH. FH resolves (Dack exiled), Wf resolves (FH exiled, Wf holds FH). FH's leave trigger (opponent's, mandatory) returns Dack: second flip.
3. Board: Wf (donated) holds FH; Solar donated; Dack mine. Library: Pr, Fe, St, Ab.
4. Second flip, St is the third banisher. Cycle: St exiles Wf (Wf leaves, FH returns), FH exiles St (St leaves, Wf returns), Wf exiles FH (FH leaves, trigger returns St), ...
   - omit Fe {Pr,Ab,St}: St' exiles Solar, Ab' exiles St' (Solar returns to me). St (-> Wf) starts the cycle. WIN.
   - omit Ab {Pr,Fe,St}: St' exiles Solar, Fe token flickers St'. St (-> Wf) starts the cycle. WIN.
   - omit St {Pr,Fe,Ab}: Fe flickers Dack, St arrives in the third flip; no token to capture Solar. No win found.
   - omit Pr {Fe,Ab,St}: Solar donated, no spare exile. No win found.
Loop: none from the pile.
With a flicker spell: as pile 34 (FH -> Solar, Wf -> FH first; Cloudshift/Ephemerate on Dack, or Touch channel on Dack at the end step): 4 of 4.
Second flip: 2 of 4 (needs Preston and St: omit Fe or omit Ab).
Doubts: St can only exile non-Assassins; nothing in the cycle is an Assassin except St itself. Draw count as pile 26.

## Summary
| Pile | Outcome | Reason | Second flip | With a spell |
|---|---|---|---|---|
| 26 Ab+St+FH | NEEDS-2ND-FLIP | T1 gives the second flip; Wf completes a 3-cycle; Solar needs Preston | 3 of 4 | Cloudshift/Ephemerate: 4 of 4 |
| 27 Ab+St+So | NEEDS-2ND-FLIP | Ab+St give the flip; FH+Wf complete the cycle; Preston tokens free Solar | 3 of 4 | Cloudshift/Ephemerate: 4 of 4 |
| 28 Ab+St+Wf | NEEDS-2ND-FLIP | T1; FH completes the cycle; Preston for Solar | 3 of 4 | Cloudshift/Ephemerate: 4 of 4 |
| 29 Ab+FH+So | NEEDS-2ND-FLIP | Ab+FH give the flip; St+Wf complete the cycle | 3 of 4 | Cloudshift/Ephemerate: 4 of 4 |
| 30 Ab+FH+Wf | NEEDS-2ND-FLIP | T1; St completes the cycle; Preston for Solar | 3 of 4 | Cloudshift/Ephemerate: 4 of 4 |
| 31 Ab+So+Wf | NEEDS-2ND-FLIP | Ab+Wf give the flip; St+FH complete the cycle | 3 of 4 | Cloudshift/Ephemerate: 4 of 4 |
| 32 St+FH+So | NEEDS-2ND-FLIP | T1 uses both banishers; needs Preston + Wf | 2 of 4 | any of the 3 spells: 4 of 4 |
| 33 St+FH+Wf | NEEDS-2ND-FLIP | 3-cycle already, no Solar; needs Preston + Solar | 2 of 4 | Cloudshift/Ephemerate: 4 of 4 |
| 34 St+So+Wf | NEEDS-2ND-FLIP | T1 uses both banishers; needs Preston + FH | 2 of 4 | any of the 3 spells: 4 of 4 |
| 35 FH+So+Wf | NEEDS-2ND-FLIP | T1 uses both banishers; needs Preston + St | 2 of 4 | any of the 3 spells: 4 of 4 |

Outcome counts: WIN-NOW 0, WIN+1 0, NEEDS-2ND-FLIP 10, NO-WIN 0. (Spell columns are lines I found, not proofs; "no win found" flips are not proven impossible.)
