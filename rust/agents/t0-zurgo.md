# Task: t0-zurgo — model the Zurgo Stormrender precon's last 22 cards

Read `rust/agents/T0-RULES.md` first. Deck: `decklists/pool/t0-b2-precons/zurgo-mardu-surge.md` (pool key `zurgo-mardu-surge`), an unmodified
WotC precon in the new Tier 0 (Bracket 2). Your module: `impls/t0_zurgo.rs` (stub in place), tests `tests/cards_t0_zurgo.rs`.
You own: myriad, goad, mobilize, fabricate, hideaway, lieutenant, token doubling.

**Check the commander first:** Zurgo Stormrender must do everything its Oracle text says (it is in your list if the audit says
it isn't Full). The deck plays through the outside-deck AI (`ai/pool.rs`): make sure that AI casts the commander early
and uses its abilities; trace a game with `commander-sim --deck elrond --pool t0 --trace 0` (and further game numbers until that deck is seated: each game draws three of the five) and follow Zurgo Stormrender.

## Cards to model (until `cardgen audit --deck zurgo-mardu-surge` shows each Full)

```
Unmodeled   Divine Visitation                        If one or more creature tokens would be created under your control, th
Unmodeled   Eliminate the Competition                Destroy X target creatures.
Unmodeled   Legion Loyalty                           Creatures you control have myriad.
Unmodeled   Shadow Summoning                         Create two tapped 1/1 white Spirit creature tokens with flying.
Unmodeled   Will of the Mardu                        • Create a number of 1/1 red Warrior creature tokens equal to the numb; • ~ deals damage to target creature equal
Partial     Ainok Strike Leader                      Sacrifice ~: Creature tokens you control gain indestructible until end of turn.
Partial     Angel of Invention                       Fabricate 2; keywords not modeled: fabricate
Partial     Blade of Selves                          Equipped creature has myriad.
Partial     Chittering Witch                         body only, abilities not modeled: When ~ enters, create a number of 1/1 black Rat creature tokens equal ; {1}{B}, Sac
Partial     Goldlust Triad                           Myriad; keywords not modeled: myriad
Partial     Grenzo, Havoc Raiser                     body only, abilities not modeled: Whenever a creature you control deals combat damage to a player, choos; • Goad ta
Partial     Ironwill Forger                          body only, abilities not modeled: Lieutenant — At the beginning of combat on your turn, if you control y
Partial     Kaya, Geist Hunter                       put a +1/+1 counter on up to one target creature token you control; until end of turn, if one or more tokens would be
Partial     Mindblade Render                         body only, abilities not modeled: Whenever your opponents are dealt combat damage, if any of that damage
Partial     Myr Battlesphere                         Whenever ~ attacks, you may tap X untapped Myr you control. If you do, ~ gets +X
Partial     Neriv, Crackling Vanguard                Whenever ~ attacks, exile a number of cards from the top of your library equal t
Partial     Redoubled Stormsinger                    body only, abilities not modeled: Whenever ~ attacks, for each creature token you control that entered t
Partial     Twilight Drover                          Whenever a creature token leaves the battlefield, put a +1/+1 counter on ~.
Partial     Windbrisk Heights                        land utility ignored: Hideaway 4; {W}, {T}: You may play the exiled card without paying its mana cost if
Partial     Within Range                             Whenever you attack, each opponent loses life equal to the number of creatures a
Partial     Zurgo Stormrender                        body only, abilities not modeled: Mobilize 1; Whenever a creature token you control leaves the battlefield, draw a c
Approximate Commander's Insignia                     regex tags: anth
```

## Done when
The T0-RULES checks pass and every card above is Full; a trace shows the deck casting its commander and using
its key cards. Report as those rules say; full report in `rust/agents/report-t0-zurgo.md`.
