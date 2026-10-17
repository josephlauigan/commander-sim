# Flips B2b: Preston, no Felidar (piles 21-25)

Notation: X* = original card, donated to an opponent (its ETB is still mine). X' = my Preston token copy (0/1 Illusion).
"Release" = a holder (banisher card or token) leaves, so the held card returns, not cast, under its owner's (my) control.
Preston* is donated, so it can only be used after I hold it and release it. Targets are chosen when a trigger goes on the
stack, so a card ETB cannot target a token that Preston has not made yet; a token ETB can target anything on the battlefield.
Fe* ETB (mine) can flicker a token I control: the token ceases, so it releases whatever it held (Felidar: "permanent you control").

## Pile 21: Preston + All-Fates Stalker + Radiant Solar  —  NO-WIN (no spell); with a Dack flicker spell it is a 2-of-4 lottery
Trace:
1. Batch: Pr*, St*, So* enter. Mine: So* x3 ventures (itself, Pr, St), Preston copies St' and So', St* ETB.
2. So' enters, ventures once: 4 ventures total = one Lost Mine = 1 drain per opponent. Nothing else.
3. Banishers are only St* and St'. Stalker cannot target St* (Assassin), St* ETB is placed before St' exists, and nothing else
   can remove a holder. So Preston (hold with St' ETB) can never be released, and Dack (hold) can never come back: no second Dack ETB.
Loop: none.
With a flicker spell: line = St' ETB holds Preston*. Cast Ephemerate/Cloudshift on Dack: second flip = 3 of {Ab,Fe,FH,Wf}.
  Then Fe* ETB flickers St' (releases Preston; or FH*/Wf* hits St' if Fe is not there). Needs Preston mine, Felidar card mine, So' (mine).
  x = the one left in the library.  x=Ab: Fe* flicks St'; Wf* holds Fe*; FH* exiles Wf*: Fe returns mine, Preston token Ft flickers it: WIN.
  x=FH: Wf* exiles St' (Preston mine); Fe* flickers Dack, flipping FH + FH' (Preston mine); FH* holds Fe*; FH' exiles FH*: Fe returns: WIN.
  x=Wf: Wf' cannot hit Wf* (Fox) and Fe* is needed for the flip: no line found.  x=Fe: no Felidar, banisher count too low: no line found.
Second flip: 2 of 4 (x=Ab, x=FH win; x=Wf, x=Fe no line found). Touch the Spirit Realm channel on Dack acts like Ephemerate but returns at the end step.
Doubts: Preston token copy type is Illusion only (assumed non-Fox/non-Assassin for targeting). Legend rule on Dack/D' keeps the card.

## Pile 22: Preston + All-Fates Stalker + Werefox Bodyguard  —  WIN-NOW
Trace:
1. Batch: Pr*, St*, Wf* enter. Mine, stack top to bottom: Wf* ETB (target Preston*), St* ETB (target Wf*), Preston copy St', copy Wf'.
2. Wf* ETB: Preston* exiled. St* ETB: Wf* exiled, so the "until" ends and Preston returns under my control (not cast).
3. Preston trigger: St' enters; its ETB targets Dack (non-Assassin): Dack held. Preston trigger: Wf' enters; Wf' ETB targets St'.
4. St' exiled (token ceases): Dack returns under my control, not cast. Preston mine: token D'. Dack ETB and D' ETB are both mine.
5. They flip 3 + 1 = all of {Ab,Fe,FH,So}. Each enters under me with Preston mine: tokens Ab', Fe', FH', So'; cards go to opponents.
6. Resolve FH' ETB: exile Fe*. Resolve Fe' ETB: flicker FH' (token ceases): FH' leave trigger returns the Felidar card to me, not cast.
7. Preston makes Fe''; Fe'' ETB flickers the Felidar card; it re-enters; Preston makes another token. So' ventures every entry.
Loop: Felidar card re-enters -> Preston token -> token ETB flickers the card; So' ventures each entry (drain every 4). Unbounded.
With a flicker spell: not needed.
Second flip: 4 of 4 (self-made: Dack returns with Preston mine, so all four remaining flip).
Doubts: legend rule on D' (keep the card; D' ETB is already on the stack). Wf* ETB (non-Fox) legally hits Preston (Rabbit Wizard).

## Pile 23: Preston + Fiend Hunter + Radiant Solar  —  WIN+1 (Ephemerate or Cloudshift on Dack; Touch channel works at the end step)
Trace:
1. Batch: Pr*, FH*, So* enter. Mine: So* x3 ventures, Preston copies FH' and So', FH* ETB (target Preston*).
2. FH* ETB: Preston* exiled. FH' enters; its ETB targets FH*: FH* exiled, its leave trigger (opponent-controlled, mandatory)
   returns Preston to me, not cast. So' enters (venture 4 = one drain). Board: Preston mine, FH' (idle), So' mine, Dack. No re-entry, no 2nd ETB.
3. Without a spell nothing more happens (hold+release of Preston and of Dack needs 4 banisher actions, I have 2).
4. Ephemerate/Cloudshift on Dack: it returns, not cast, Preston triggers D'. Dack ETB + D' ETB flip 3 + 1 = all of {Ab,Fe,St,Wf}, with tokens.
5. St' ETB holds Fe*. Wf' ETB targets St' (non-Fox): St' ceases, Felidar returns to me, not cast. Preston makes Fe'' etc.
6. Fe'' ETB flickers the Felidar card each time; So' ventures on every entry.
Loop: same Felidar loop as pile 22. Unbounded.
With a flicker spell: Ephemerate or Cloudshift on Dack (1 {W}); Touch the Spirit Realm channel ({1}{W}) returns Dack at the end step, then the same line.
Second flip: 4 of 4 (all four flip, once Dack is flickered with Preston mine).
Doubts: Touch channel's end-step timing (all triggers happen at that end step, still this turn).

## Pile 24: Preston + Fiend Hunter + Werefox Bodyguard  —  WIN-NOW
Trace:
1. Batch: Pr*, FH*, Wf* enter. Mine: Wf* ETB (target Preston*), FH* ETB (target Wf*), Preston copies FH', Wf'.
2. Wf* ETB: Preston* exiled. FH* ETB: Wf* exiled (FH holds it): Preston returns under my control.
3. FH' enters, ETB targets Dack: Dack held. Wf' enters, ETB targets FH' (non-Fox): FH' ceases, its leave trigger (mine) returns Dack, not cast.
4. Preston mine: token D'. Dack ETB + D' ETB flip all four of {Ab,Fe,St,So}; tokens Ab', Fe', St', So' (Preston mine).
5. St' ETB holds Fe*. Fe' ETB (placed after St' exists) flickers St': token ceases, Felidar card returns to me, not cast.
6. Preston makes Fe''; Fe'' ETB flickers the card; So' ventures on each entry.
Loop: Felidar card re-enters -> token -> flicker; So' ventures. Unbounded.
With a flicker spell: not needed.
Second flip: 4 of 4 (self-made).
Doubts: resolution order of the Preston triggers (St' must exist before Fe' ETB is placed): I choose it, so fine.

## Pile 25: Preston + Radiant Solar + Werefox Bodyguard  —  NO-WIN (no spell); with a Dack flicker spell a 2-of-4 lottery
Trace:
1. Batch: Pr*, So*, Wf* enter. Mine: So* x3 ventures, Preston copies So' (venture 4 = one drain) and Wf', Wf* ETB.
2. Only banishers: Wf*, Wf'. Werefox hits non-Fox only, so neither can ever exile Wf*, and Wf* ETB is placed before Wf' exists.
   Nothing can release a holder: Preston stays donated, Dack stays on the battlefield. No second Dack ETB.
Loop: none.
With a flicker spell: Wf' ETB holds Preston*. Cast Ephemerate/Cloudshift on Dack: second flip = 3 of {Ab,Fe,FH,St}; x = the one left.
  x=Ab: Fe* flickers Wf' (Preston mine); St* holds Fe*; FH* exiles St*: Felidar returns, loop with So': WIN.
  x=FH: St* exiles Wf' (Preston mine); Fe* flickers Dack, flipping FH + FH'; FH* holds Fe*; FH' exiles FH*: Felidar returns: WIN.
  x=St: only FH* + Fe* left, and St' cannot be released (Stalker cannot hit St*): no line found.  x=Fe: no Felidar, too few banishers: no line found.
Second flip: 2 of 4 (x=Ab, x=FH win).
Doubts: x=St and x=Fe: I did not find a win, not a proof there is none.

## Summary
| Pile | Outcome | Reason |
|---|---|---|
| 21 Pr+St+So | NO-WIN (spell: 2 of 4) | only St*/St' banishers; Preston and Dack can never be released |
| 22 Pr+St+Wf | WIN-NOW | Wf*/St* free Preston, St'/Wf' return Dack, all 4 flip with tokens, Felidar loop with So' |
| 23 Pr+FH+So | WIN+1 (Ephemerate/Cloudshift on Dack) | FH*/FH' free Preston; flicker Dack gives all 4 flipped with tokens, then St'/Wf' free Felidar |
| 24 Pr+FH+Wf | WIN-NOW | Wf*/FH* free Preston, FH'/Wf' return Dack, all 4 flip with tokens, Felidar loop |
| 25 Pr+So+Wf | NO-WIN (spell: 2 of 4) | Werefox cannot hit Wf*; no release possible |
Counts: WIN-NOW 2, WIN+1 1, NO-WIN 2.
