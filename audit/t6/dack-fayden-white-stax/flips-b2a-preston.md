# Flips B2a: piles with Preston, no Felidar (16-20)

Notation: X* = original card, donated to an opponent. X' = my Preston token copy (0/1 Illusion, mine). D' = Dack token.
Dack himself stays mine (I cast him). Preston* is donated in all these piles, so FIRST job = get Preston back, THEN re-enter Dack.
Key trick: a token that is exiled ceases to exist, so a token banisher/Abdel that gets exiled ends its hold at once.
Another trick: Ab* ETB (mine) exiles Dack until Ab* leaves; a banisher card's ETB (mine) exiles Ab* -> Dack returns (not cast).
After Dack returns with Preston mine, D flips 3 and D' flips the last 1: ALL FOUR remaining creatures flip, each with a
token (mine). Banisher tokens free any donated Felidar, the Felidar card + Preston + a Solar token (mine) = loop.

## Pile 16: Preston + Abdel Adrian + All-Fates Stalker  —  WIN-NOW
Trace:
1. Triggers (mine): Preston(copy Abdel)=PA, Preston(copy Stalker)=PS, Ab* ETB, St* ETB (target Ab*).
   Stack bottom to top: St*, Ab*, PA, PS.
2. PS: St' token. Its ETB targets Preston* (non-Assassin). Resolves: Preston exiled.
3. PA: Ab' token. Its ETB exiles St' only. St' is a token, so it ceases; Preston returns under my control (not cast).
4. Ab* ETB resolves: exile Dack (until Ab* leaves). Soldier token.
5. St* ETB resolves: Ab* exiled. Ab* left, so Dack returns under my control, not cast.
6. Dack triggers: ETB (flips 3 of Fe, FH, So, Wf) and Preston -> D' (legend rule: keep the card; D' trigger still resolves,
   it flips the last one). All four flipped; Preston is mine, so I get Fe', FH', So', Wf'; the cards are donated.
7. FH' ETB targets Fe* (exile). Fe' (made after, via LKI copy of Fe*) ETB flickers FH': token ceases, FH leave trigger (mine)
   returns Fe* to me; Preston mine -> Fe'' token; Fe'' ETB flickers the Fe card. Wf' works the same if FH is not used.
8. So' (mine) ventures on every entry of the nontoken Felidar card.
Loop: Felidar card re-enters -> Preston token -> its ETB flickers the card; Solar token ventures each entry.
With a flicker spell: not needed.
Second flip: self-made, 4 of 4 (all of Fe, FH, So, Wf flip).
Doubts: Ab* choosing Dack as an exiled permanent (legal: nonland permanent I control). Preston trigger copying Fe* after it was
 exiled uses last-known information (otherwise resolve Fe' trigger before FH' ETB, targeting FH' later; same result).

## Pile 17: Preston + Abdel Adrian + Fiend Hunter  —  WIN-NOW
Trace: same shape as pile 16 with FH in place of Stalker.
1. Triggers (mine): PA, PH (Preston copy of FH), Ab* ETB, FH* ETB (target Ab*). Stack bottom to top: FH*, Ab*, PA, PH.
2. PH: FH' token; ETB exiles Preston*. PA: Ab' token; ETB exiles FH' (token ceases). FH' leave trigger (mine) returns Preston to me.
3. Ab* ETB exiles Dack. FH* ETB resolves: Ab* exiled -> Dack returns (not cast), Preston mine -> D'.
4. Flips: all of Fe, St, So, Wf, each with a token. St' (or Wf') ETB exiles Fe*; Fe' flickers St' (token ceases, hold ends),
   Fe* returns to me (not cast), Preston makes Fe''; Fe'' flickers the Fe card ... So' ventures each entry.
Loop: Felidar card <-> Fe'' token flicker; Solar token ventures. Unbounded.
With a flicker spell: not needed.
Second flip: self-made, 4 of 4.
Doubts: FH* ETB is a "may" - I take it. FH leave trigger of the FH' token is mine (I controlled it).

## Pile 18: Preston + Abdel Adrian + Radiant Solar  —  NO-WIN (WIN+1 with a flicker spell on Dack in most flips; not fully traced)
Trace:
1. Triggers (mine): PA (Abdel copy), PSo (Solar copy), Ab* ETB, So* venture x3 (for Pr, Ab, So itself).
2. PSo: So' token, ventures once (itself). PA: Ab' token. Total 4 ventures = one Dark Pool drain (1 life each) + 1 draw. Not a win.
3. No banisher and no Felidar in the pile: nothing returns Preston* and nothing leaves. Ab'/Ab* can exile Dack or So' but
   nothing ever makes Abdel leave, so Dack never re-enters. Preston's sac ability needs five Illusions (I have two).
4. Library holds Fe, St, FH, Wf. Cannot reach them.
Loop: none.
With a flicker spell: Cloudshift or Ephemerate on Dack (1 W) flips 3 of Fe/St/FH/Wf. So' stays mine, so Solar is covered.
 At least two banishers are always among the 3. Chain X*->Preston*, Y*->X* frees Preston (mine). If Fe is among the 3
 (3 of 4), Fe* ETB -> Dack (placed first, resolved after Preston is free): Dack re-enters with Preston mine, D flips the last
 banisher B (token B'), then B* ETB -> Y* returns X* to me, X card exiles Fe*, X' exiles X card (or B' does for X=Stalker)
 -> Fe* returns, Fe' flickers the Felidar card, So' ventures: WIN. Traced generically; X=Stalker sub-case (Stalker can't hit
 itself) and the no-Fe flips (1 of 4) are not verified. Touch the Spirit Realm is too slow (returns at end step).
Second flip: only with the spell; see above (likely, not 4 of 4 confirmed). Without a spell: 0 of 4.
Doubts: whether Temple's draw helps (no: 2 W cannot cast anything relevant).

## Pile 19: Preston + Abdel Adrian + Werefox Bodyguard  —  WIN-NOW
Trace: same shape as piles 16/17.
1. Triggers (mine): PA, PW (Preston copy of Werefox), Ab* ETB, Wf* ETB (target Ab*; Abdel is non-Fox). Stack bottom to top: Wf*, Ab*, PA, PW.
2. PW: Wf' token; ETB exiles Preston* (non-Fox). PA: Ab' token; ETB exiles Wf' (token ceases, hold ends): Preston returns, mine.
3. Ab* ETB exiles Dack. Wf* ETB exiles Ab*: Ab* left, Dack returns (not cast); Preston mine -> D'.
4. D flips 3, D' flips the last: all of Fe, St, FH, So, each with a token. St' (or FH') ETB exiles Fe*; Fe' flickers
   that token (ceases), Fe* returns to me; Preston makes Fe''; Fe'' flickers the Felidar card; So' ventures per entry.
Loop: Felidar card <-> Fe'' token flicker; Solar token ventures. Unbounded.
With a flicker spell: not needed.
Second flip: self-made, 4 of 4.
Doubts: Wf can't target Wf* (Fox) but never needs to. Wf' is a Fox? (Illusion copy; irrelevant here).

## Pile 20: Preston + All-Fates Stalker + Fiend Hunter  —  WIN-NOW
Trace:
1. Triggers (mine): PS (Stalker copy), PH (FH copy), St* ETB (target FH*), FH* ETB (target Dack). Stack bottom to top: St*, FH*, PS, PH.
2. PH: FH' token; ETB exiles Preston*. PS: St' token; ETB exiles FH' (Illusion, non-Assassin; token ceases).
   FH' leave trigger (mine) returns Preston to me (not cast).
3. FH* ETB resolves: Dack exiled. St* ETB resolves: FH* exiled -> FH* leave trigger (opponent controls it, mandatory) returns
   Dack under my control, not cast. Preston mine -> D'.
4. D flips 3, D' the last: all of Ab, Fe, So, Wf, each with a token. Wf' (or Ab' no) ETB exiles Fe*; Fe' flickers Wf' (token
   ceases), Fe* returns to me; Preston -> Fe''; Fe'' flickers the Felidar card; So' ventures. (Ab' / Ab* choose zero cards.)
Loop: Felidar card <-> Fe'' token flicker; Solar token ventures. Unbounded.
With a flicker spell: not needed.
Second flip: self-made, 4 of 4.
Doubts: step 3 relies on FH* leave trigger resolving after St* ETB; its target-less return is mandatory so Dack does return.

## Summary
| Pile | Outcome | Reason |
|---|---|---|
| 16 Pr+Ab+St | WIN-NOW | St' frees Preston, Ab' exiles St'; Ab* exiles Dack, St* exiles Ab* -> Dack back with Preston; 4-of-4 flip, Felidar loop |
| 17 Pr+Ab+FH | WIN-NOW | same with FH (leave trigger frees Preston) |
| 18 Pr+Ab+So | NO-WIN | no banisher/Felidar: Preston stuck, Dack never re-enters; 4 ventures = 1 drain. WIN+1 with Cloudshift/Ephemerate on Dack in most flips (not fully traced) |
| 19 Pr+Ab+Wf | WIN-NOW | same with Wf |
| 20 Pr+St+FH | WIN-NOW | FH' exiles Preston*, St' exiles FH'; FH* exiles Dack, St* exiles FH* -> Dack back with Preston |
Counts: WIN-NOW 4, WIN+1 0 (pile 18 only as a likely upgrade), NEEDS-2ND-FLIP 0, NO-WIN 1.
