# Task: t0-giada — model the Giada, Font of Hope precon's last 30 cards

Read `rust/agents/T0-RULES.md` first. Deck: `decklists/pool/t0-b2-precons/giada-calling-all-angels.md` (pool key `giada-calling-all-angels`), an unmodified
WotC precon in the new Tier 0 (Bracket 2). Your module: `impls/t0_giada.rs` (stub in place), tests `tests/cards_t0_giada.rs`.
You own: cycling and landcycling, the monarch, Clues and investigate, lifegain triggers.

**Check the commander first:** Giada, Font of Hope must do everything its Oracle text says (it is in your list if the audit says
it isn't Full). The deck plays through the outside-deck AI (`ai/pool.rs`): make sure that AI casts the commander early
and uses its abilities; trace a game with `commander-sim --deck cloud --pool t0 --any-tier --trace 0` (and further game numbers until that deck is seated: each game draws three of the five) and follow Giada, Font of Hope.

## Cards to model (until `cardgen audit --deck giada-calling-all-angels` shows each Full)

```
Unmodeled   Search the Premises                      no effect recognised
Unmodeled   Secret Rendezvous                        You and target opponent each draw three cards.
Partial     Angel of Vitality                        body only, abilities not modeled: If you would gain life, you gain that much life plus 1 instead.; This cre
Partial     Angel of the Ruins                       Plainscycling {2}; keywords not modeled: cycling, landcycling
Partial     Angelic Field Marshal                    body only, abilities not modeled: Lieutenant — As long as you control your commander, ~ gets +2/+2 and c
Partial     Angelic Sleuth                           body only, abilities not modeled: Whenever another permanent you control leaves the battlefield, if it had 
Partial     Court of Grace                           When ~ enters, you become the monarch.; At the beginning of your upkeep, create a 1/1 white Spirit creature
Partial     Cut a Deal                               Each opponent draws a card, then you draw a card for each opponent who drew a ca
Partial     Defy Death                               Return target creature card from your graveyard to the battlefield. If it's an A
Partial     Emeria Shepherd                          Landfall — Whenever a land you control enters, you may return target nonland per
Partial     Exemplar of Light                        Whenever you put one or more +1/+1 counters on ~, draw a card. This ability trig
Partial     Fateful Absence                          Destroy target creature or planeswalker. Its controller investigates.   [partly:
Partial     Firemane Commando                        Whenever another player attacks with two or more creatures, they draw a card if 
Partial     Giada, Font of Hope                      Each other Angel you control enters with an additional +1/+1 counter on it for e
Partial     Herald of Eternal Dawn                   body only, abilities not modeled: You can't lose the game and your opponents can't win the game.
Partial     Herald of War                            Angel spells and Human spells you cast cost {1} less to cast for each +1/+1 coun
Partial     Heraldic Banner                          Creatures you control of the chosen color get +1/+0.
Partial     Merchant of Truth                        body only, abilities not modeled: Clues you control have exalted.
Partial     Metropolis Reformer                      body only, abilities not modeled: You have hexproof.; Whenever ~ is dealt damage, you gain that much life.
Partial     Norn's Choirmaster                       body only, abilities not modeled: Whenever a commander you control enters or attacks, proliferate.
Partial     Righteous Valkyrie                       Whenever another Angel or Cleric you control enters, you gain life equal to that
Partial     Sephara, Sky's Blade                     body only, abilities not modeled: You may pay {W} and tap four untapped creatures you control with flyin; O
Partial     Seraph of the Sword                      body only, abilities not modeled: Prevent all combat damage that would be dealt to ~.
Partial     Serra Avenger                            body only, abilities not modeled: You can't cast ~ during your first, second, or third turns of the game
Partial     Starnheim Aspirant                       body only, abilities not modeled: Angel spells you cast cost {2} less to cast.
Partial     Vanguard Seraph                          body only, abilities not modeled: Whenever you gain life for the first time each turn, surveil 1. 
Partial     Wojek Investigator                       body only, abilities not modeled: At the beginning of your upkeep, investigate once for each opponent who h
Partial     Youthful Valkyrie                        body only, abilities not modeled: Whenever another Angel you control enters, put a +1/+1 counter on ~.
Approximate Bishop of Wings                          regex tags: tok tokfly tokp
Approximate Linvala, the Preserver                   regex tags: tok tokfly tokp
```

## Done when
The T0-RULES checks pass and every card above is Full; a trace shows the deck casting its commander and using
its key cards. Report as those rules say; full report in `rust/agents/report-t0-giada.md`.
