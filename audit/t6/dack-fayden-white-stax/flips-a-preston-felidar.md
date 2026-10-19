# Dack first-flip traces: piles 1-5 (Preston + Felidar Guardian + X)

Notation: X* = the original card, now controlled by an opponent (donated). X' = my Preston token copy (0/1 Illusion, mine).
Ft = Felidar token. T = the Felidar token from Preston's trigger on Felidar. C = the Felidar* ETB (mine, target chosen
when it goes on the stack, so it can only name Dack or a land). "Flip-2" = the 3 creatures a second Dack ETB flips from
the 4 not in the pile; "flip-3" = the last one. All 4 remaining creatures get flipped because Felidar flickers Dack twice.

General facts used in all five piles (all follow the sheet):
- Stack order is mine: I put C at the BOTTOM so it resolves last, after Preston is back under my control.
- Preston trigger resolution does not need Preston to be on the battlefield, so I can resolve it late (T appears late).
- "Free a donated card" = a banisher I control (card or token) exiles the holder, or a Felidar token flickers a banisher
  token that holds it. Fiend Hunter's leave trigger is controlled by whoever controlled FH when it left (an opponent if
  FH*), but it is mandatory and returns the card under its OWNER's (my) control.
- Stalker only hits NON-Assassin creatures: it cannot target Stalker* (Drix Assassin). A Stalker token copy is an Illusion,
  so it CAN be targeted by Stalker. Werefox cannot target Werefox* (Fox); its token can't either (it hits "non-Fox", and
  Werefox* is a Fox).
- Win engine (the loop): Preston mine + Felidar CARD mine + a Solar (card or token) mine. Felidar card enters (not cast)
  -> Preston makes Ft -> Ft ETB flickers the Felidar card -> it re-enters -> Preston makes another Ft ... and every entry of
  the nontoken Felidar ventures through my Solar. Unbounded.

## Pile 1: Preston + Felidar Guardian + Abdel Adrian  —  NEEDS-2ND-FLIP (but self-made by Felidar: 4 of 4 flip-2 outcomes win, so effectively WIN-NOW with no luck and no extra card)
Trace (nothing in the pile is a banisher, Preston/Felidar/Abdel are all donated; I own only Dack, T, the Abdel token):
1. Triggers (mine): Preston x2 (copies of Felidar, Abdel), Felidar* ETB = C (target Dack), Abdel* ETB (choose zero cards).
   Stack bottom to top: Preston(Felidar copy), Preston(Abdel copy), Abdel*, C on top.
2. C resolves: Dack leaves, returns (not cast). Dack ETB flips 3 of {St, FH, So, Wf}; they enter under me, are donated;
   their ETBs are mine. Preston is NOT mine, so no tokens for them. Abdel and the Abdel token do nothing useful.
3. Banisher chain with the donated banishers' ETBs (targets chosen as they go on the stack; resolve in the order given):
   - Case A: flip-2 = {St, FH, Wf} (So is flip-3). FH*->Preston*; St*->FH*; Wf*->St*. FH* leaves: its trigger returns Preston
     under my control. Wf* exiling St* ends St's hold and FH returns to me: FH card + token FH' (Preston is mine now). FH
     card ETB -> Felidar*; FH' ETB -> FH card; FH card leaves, its leave trigger (mine) returns Felidar* to me.
     Felidar card + Preston mine => loop starts (Ft flickers Felidar). Resolve the Preston(Felidar) trigger now: T's ETB
     flickers Dack, flipping So under me with Preston mine: Solar token So'. Loop ventures every cycle: WIN.
   - Case B: flip-2 holds So + two banishers (X = the third banisher is flip-3). Free Preston with the two:
       {FH, Wf}: FH*->Preston*, Wf*->FH*. {St, Wf}: Wf*->Preston*, St*->Wf*. {St, FH}: FH*->Preston*, St*->FH*.
     Preston is mine. Now resolve Preston(Felidar copy): T's ETB flickers (the new) Dack: Dack ETB flips the last creature
     under me with Preston mine: token + card ETB for it. Finish the chain:
       {FH, Wf} + St flip-3: St* ETB->Wf* (FH returns to me, FH card ETB->Felidar*, FH' ETB->FH card, Felidar* returns);
          St' ETB->So* (hold). Felidar card ETB flickers St' (token ceases) -> So* returns to me. Ft->Felidar loop. WIN.
       {St, Wf} + FH flip-3: FH* ETB->Felidar*; FH' ETB->St* (Wf returns to me with Wf'); Wf card ETB->FH* (FH leaves,
          its leave trigger returns Felidar*); Wf' ETB->So* (hold); Felidar ETB flickers Wf' -> So* returns. WIN.
       {St, FH} + Wf flip-3: Wf* ETB->Felidar*; Wf' ETB->St* (FH returns to me with FH'); FH card ETB->Wf* (Felidar*
          returns); FH' ETB->So* (hold); Felidar ETB flickers FH' (its leave trigger returns So*). WIN.
   So every flip-2 outcome (4 of 4) wins. The Abdel token/Abdel* are not needed.
Loop: Felidar card re-enters -> Preston token Ft -> Ft ETB flickers the card; each entry ventures via Solar; unbounded.
With a flicker spell: not needed.
Second flip: 4 of 4 outcomes (all of {St,FH,So,Wf} appear because Felidar flickers Dack twice: C then T).
Doubts: (1) relies on the Felidar* ETB being a real trigger of mine after donation (sheet fact 2). (2) When D' (Dack token)
 is made the legend rule applies (keep the Dack card; the token's trigger still resolves); not needed in this pile except
 Case B where the token's ETB finds nothing to flip. (3) FH* leave trigger is an opponent-controlled mandatory trigger.

## Pile 2: Preston + Felidar Guardian + All-Fates Stalker  —  NEEDS-2ND-FLIP (self-made by Felidar flickering Dack: 4 of 4 flip outcomes win; effectively WIN-NOW)
Trace:
1. Triggers (mine): Preston(Felidar copy)=PA, Preston(Stalker copy)=PS, Stalker* ETB (target Felidar*), C = Felidar* ETB (target Dack).
   Stack bottom to top: C, Stalker*, PA, PS.
2. PS resolves: token S' (Illusion). Its ETB, target Preston* (non-Assassin Rabbit Wizard). Resolves: Preston exiled.
3. PA resolves: T appears. T ETB flickers S' (token ceases, the "until" ends): Preston returns under my control.
4. Stalker* ETB resolves: Felidar* exiled until Stalker* leaves.
5. C resolves: Dack exiled/returned (not cast). Preston is mine: Preston trigger -> Dack token D'. Dack ETB and D' ETB each
   flip creatures (3 + the last 1: all of Ab, FH, So, Wf). Each flipped creature enters under me with Preston mine, so I
   get a token of each (Ab', FH', So', Wf'); the cards are donated; all their ETBs are mine.
6. At least one of FH/Wf is always among the four (only the 4-set is flipped, and both flips happen). FH' (or Wf') ETB
   targets Stalker*: Stalker leaves, Felidar* returns under my control (not cast). Preston mine: token Ft2. Loop begins.
7. Solar token So' is mine (So entered while Preston was mine), so every re-entry of the Felidar card ventures. WIN.
Loop: Ft ETB flickers the Felidar card -> re-enters -> Preston makes the next Ft; So' ventures each entry. Unbounded.
With a flicker spell: not needed.
Second flip: 4 of 4 (always). Even if only the first 3 flip (D' token lost to the legend rule), FH or Wf is always among the
 3 and the running Felidar loop flickers Dack again to get the last creature (still a token because Preston is mine).
Doubts: Stalker's text can't target Stalker* (Assassin) so S' targets Preston instead; Stalker* is later removed by an FH/Wf
 token. Legend rule on Dack/D': keep the card; the D' trigger still happens.

## Pile 3: Preston + Felidar Guardian + Fiend Hunter  —  NEEDS-2ND-FLIP (self-made by Felidar flickering Dack: 4 of 4 win; effectively WIN-NOW)
Trace:
1. Triggers (mine): Preston(Felidar copy)=PA, Preston(FH copy)=PH, FH* ETB (target Felidar*), C (Felidar* -> Dack).
   Stack bottom to top: C, FH*, PA, PH.
2. PH resolves: token FH'. Its ETB: exile Preston*. Resolves.
3. PA resolves: T. T ETB flickers FH' (token leaves): FH' leave trigger (mine) returns Preston to me.
4. FH* ETB resolves: Felidar* exiled.
5. C resolves: Dack returns, Preston mine -> D'. Flips: all of {Ab, St, So, Wf}, each with a token (mine) and the card donated.
6. St' or Wf' (at least one is always among the flipped; both exist if all four flip) ETB targets FH*: FH* leaves, its
   leave trigger (opponent-controlled, mandatory) returns Felidar* under my control. Preston token Ft2: loop.
7. So' (token, mine) ventures every re-entry. WIN.
Loop: same Felidar card / Ft loop with Solar token ventures.
With a flicker spell: not needed.
Second flip: 4 of 4 (St or Wf always present; So, if missed in a 3-creature flip, comes from the loop's own Dack flicker).
Doubts: the FH* leave trigger is controlled by an opponent, but it is mandatory and returns to the owner (me).

## Pile 4: Preston + Felidar Guardian + Radiant Solar  —  NEEDS-2ND-FLIP (self-made by Felidar flickering Dack: 4 of 4 win; effectively WIN-NOW)
Trace:
1. Triggers (mine): Solar* ventures for Preston, Felidar, Solar (3 ventures); Preston triggers: Felidar copy (PA), Solar copy
   (PS2); C (Felidar* -> Dack). PS2 makes So' (token, mine): its own entry ventures once more (4 ventures = 1 drain, not a
   win). So' stays mine for the loop.
2. No banisher in the pile. Resolve C first (it is not delayed here): Dack flickers, flip-2 = 3 of {Ab, St, FH, Wf}; the
   donated cards' ETBs are mine (no tokens yet). Keep PA unresolved (so T is made late).
3. Free Preston with two banishers: FH*->Preston*, other banisher*->FH* (St or Wf; if no FH, Wf*->Preston*, St*->Wf*).
4. Resolve PA: T flickers Dack (new object); Preston is mine, flip-3 (the last creature) arrives with a token.
5. Finish as in pile 1 (cases): all-three-banishers {St,FH,Wf}: FH*->Pr*, St*->FH*, Wf*->St*, then FH card ETB->Felidar*,
   FH' ETB->FH card, Felidar* returns. {FH,Wf}+St: St*->Wf* (FH back), FH->Felidar*, FH'->FH. {St,Wf}+FH: Wf*->Pr*,
   St*->Wf*; FH* ETB->Felidar*, FH' ETB->St*, Wf back, Wf ETB->FH*, leave trigger returns Felidar*. {St,FH}+Wf:
   FH*->Pr*, St*->FH*; Wf* ETB->Felidar*, Wf' ETB->St*, FH back, FH ETB->Wf*, Felidar* returns.
   Abdel (in the flip) is simply left alone (choose no cards).
6. Preston + Felidar card + So' mine: loop, ventures each cycle. WIN.
Loop: Ft ETB flickers Felidar card; So' ventures per entry.
With a flicker spell: not needed.
Second flip: 4 of 4 (any 3 of the 4 work, the last comes from T).
Doubts: Solar's first-batch ventures need Solar* to see the simultaneous entries (sheet fact 3); not needed for the win, only
 gives a partial drain. The Dack card/D' trigger legend-rule note applies to step 4 (token of Dack: keep the card).

## Pile 5: Preston + Felidar Guardian + Werefox Bodyguard  —  NEEDS-2ND-FLIP (self-made by Felidar flickering Dack: 4 of 4 win; effectively WIN-NOW)
Trace:
1. Triggers (mine): Preston(Felidar copy)=PA, Preston(Werefox copy)=PW, Werefox* ETB (target Felidar*, non-Fox), C.
   Stack bottom to top: C, Werefox*, PA, PW.
2. PW resolves: Wf' (Illusion). Its ETB: exile Preston* (non-Fox). Resolves.
3. PA resolves: T. T ETB flickers Wf' (token ceases): Preston returns under my control.
4. Werefox* ETB resolves: Felidar* exiled until Werefox* leaves.
5. C resolves: Dack returns, Preston mine -> D'. Flips all of {Ab, St, FH, So}, each with a token for me.
6. St' or FH' (one is always present) ETB targets Werefox* (not an Assassin; any creature for FH): Werefox leaves, Felidar* returns
   under my control; Preston token Ft2: loop. So' (token) ventures. WIN.
Loop: same Felidar/Ft loop with Solar token ventures.
With a flicker spell: not needed.
Second flip: 4 of 4.
Doubts: Werefox* is a Fox so Werefox' can't target it (Wf' hits Preston instead). Legend rule on Dack/D': keep the card.

## Summary
| Pile | Cards | Outcome | Reason |
|---|---|---|---|
| 1 | Pr+Fe+Ab | NEEDS-2ND-FLIP (self-made, 4 of 4 flip-2 outcomes win) | Fe flickers Dack twice; flipped banishers free Preston then Felidar via chains; no luck needed |
| 2 | Pr+Fe+St | NEEDS-2ND-FLIP (self-made, 4 of 4) | Stalker token + Fe token free Preston; FH/Wf token (one always flips) frees Felidar; Solar token via Dack flick |
| 3 | Pr+Fe+FH | NEEDS-2ND-FLIP (self-made, 4 of 4) | FH token + Fe token free Preston; St/Wf token frees Felidar from FH*; loop |
| 4 | Pr+Fe+So | NEEDS-2ND-FLIP (self-made, 4 of 4) | Solar token already mine; flipped banishers free Preston then Felidar; loop |
| 5 | Pr+Fe+Wf | NEEDS-2ND-FLIP (self-made, 4 of 4) | Werefox token + Fe token free Preston; St/FH token frees Felidar from Werefox*; loop |
All five could equally be called WIN-NOW: none needs luck or an extra card, only Felidar's own flicker of Dack.
Weak points for the lead to double-check: donated-source triggers staying mine (fact 2), token legend rule on Dack, and
Felidar* getting exiled by a banisher token before C (C is an independent trigger, still resolves).
