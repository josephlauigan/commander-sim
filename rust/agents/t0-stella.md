# Task: t0-stella — model the Stella Lee, Wild Card precon's last 26 cards

Read `rust/agents/T0-RULES.md` first. Deck: `decklists/pool/t0-b2-precons/stella-lee-quick-draw.md` (pool key `stella-lee-quick-draw`), an unmodified
WotC precon in the new Tier 0 (Bracket 2). Your module: `impls/t0_stella.rs` (stub in place), tests `tests/cards_t0_stella.rs`.
You own: plot, suspend, storm, cascade, jump-start, copying spells, adapt, Vehicles and crew.

**Check the commander first:** Stella Lee, Wild Card must do everything its Oracle text says (it is in your list if the audit says
it isn't Full). The deck plays through the outside-deck AI (`ai/pool.rs`): make sure that AI casts the commander early
and uses its abilities; trace a game with `commander-sim --deck elrond --pool t0 --trace 0` (and further game numbers until that deck is seated: each game draws three of the five) and follow Stella Lee, Wild Card.

## Cards to model (until `cardgen audit --deck stella-lee-quick-draw` shows each Full)

```
Unmodeled   Arcane Bombardment                       Whenever you cast your first instant or sorcery spell each turn, exile
Unmodeled   Baral's Expertise                        Return up to three target artifacts and/or creatures to their owners' ; You may cast a spell with mana value 4 or les
Unmodeled   Curse of the Swine                       Exile X target creatures. For each creature exiled this way, its contr
Unmodeled   Epic Experiment                          Exile the top X cards of your library. You may cast instant and sorcer
Unmodeled   Finale of Promise                        You may cast up to one target instant card and/or up to one target sor
Unmodeled   Rousing Refrain                          Add {R} for each card in target opponent's hand. Until end of turn, yo; Suspend 3—{1}{R}
Unmodeled   Volcanic Torrent                         Cascade; ~ deals X damage to each creature and planeswalker your opponents cont
Partial     Bloodthirsty Adversary                   body only, abilities not modeled: When ~ enters, you may pay {2}{R} any number of times. When you pay th
Partial     Crackling Spellslinger                   body only, abilities not modeled: When ~ enters, if you cast it, the next instant or sorcery spell you c
Partial     Cursed Mirror                            As ~ enters, you may have it become a copy of any creature on the battlefield un
Partial     Elemental Eruption                       keywords not modeled: storm
Partial     Finale of Revelation                     Draw X cards. If X is 10 or more, instead shuffle your graveyard into your libra; Exile ~.
Partial     Forger's Foundry                         {3}{U}{U}, {T}: You may cast any number of spells from among cards exiled with ~
Partial     Kaza, Roil Chaser                        body only, abilities not modeled: {T}: The next instant or sorcery spell you cast this turn costs {X} le
Partial     Leyline Dowser                           {1}, {T}: Mill a card. You may put an instant or sorcery card milled this way in; Tap an untapped legendary creature 
Partial     Lock and Load                            Plot {3}{U}; keywords not modeled: plot
Partial     Midnight Clock                           {2}{U}: Put an hour counter on ~.; At the beginning of each upkeep, put an hour counter on ~.; When the twelfth hour 
Partial     Octavia, Living Thesis                   body only, abilities not modeled: This spell costs {8} less to cast if you have eight or more instant an
Partial     Pteramander                              body only, abilities not modeled: {7}{U}: Adapt 4. This ability costs {1} less to activate for each inst; keywords no
Partial     Pyretic Charge                           Discard your hand, then draw four cards. For each card discarded this way, creat; Plot {3}{R}; keywords not modeled: 
Partial     Radical Idea                             Jump-start
Partial     Shark Typhoon                            When you cycle ~, create an X/X blue Shark creature token with flying.; keywords not modeled: cycling
Partial     Smoldering Stagecoach                    Whenever this Vehicle attacks, the next instant spell and the next sorcery spell; keywords not modeled: crew
Partial     Stella Lee, Wild Card                    {T}: Copy target instant or sorcery spell you control. You may choose new target
Partial     Thunderclap Drake                        {2}{U}, Sacrifice ~: When you next cast an instant or sorcery spell this turn, c
Partial     Winged Boots                             Equipped creature has flying and ward {4}.
```

## Done when
The T0-RULES checks pass and every card above is Full; a trace shows the deck casting its commander and using
its key cards. Report as those rules say; full report in `rust/agents/report-t0-stella.md`.
