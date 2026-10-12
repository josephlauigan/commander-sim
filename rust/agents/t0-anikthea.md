# Task: t0-anikthea — model the Anikthea, Hand of Erebos precon's last 31 cards

Read `rust/agents/T0-RULES.md` first. Deck: `decklists/pool/t0-b2-precons/anikthea-enduring-enchantments.md` (pool key `anikthea-enduring-enchantments`), an unmodified
WotC precon in the new Tier 0 (Bracket 2). Your module: `impls/t0_anikthea.rs` (stub in place), tests `tests/cards_t0_anikthea.rs`.
You own: Sagas and read ahead, constellation, devotion, channel, landfall.

**Check the commander first:** Anikthea, Hand of Erebos must do everything its Oracle text says (it is in your list if the audit says
it isn't Full). The deck plays through the outside-deck AI (`ai/pool.rs`): make sure that AI casts the commander early
and uses its abilities; trace a game with `commander-sim --deck elrond --pool t0 --trace 0` (and further game numbers until that deck is seated: each game draws three of the five) and follow Anikthea, Hand of Erebos.

## Cards to model (until `cardgen audit --deck anikthea-enduring-enchantments` shows each Full)

```
Unmodeled   Abundance                                If you would draw a card, you may instead choose land or nonland and r
Unmodeled   Battle at the Helvault                   I, II — For each player, exile up to one target non-Saga, nonland perm; III — Create Avacyn, a legendary 8/8 whit
Unmodeled   Cacophony Unleashed                      When ~ enters, if you cast it, destroy all nonenchantment creatures.; Whenever ~ or another enchantment you control e
Unmodeled   Cunning Rhetoric                         Whenever an opponent attacks you and/or one or more planeswalkers you 
Unmodeled   Ghoulish Impetus                         Enchant creature; When enchanted creature dies, return this card to the battlefield at t
Unmodeled   The Binding of the Titans                I — Each player mills three cards.; III — Return target creature or land card from your graveyard to your 
Unmodeled   The Eldest Reborn                        I — Each opponent sacrifices a creature or planeswalker of their choic; II — Each opponent discards a card.; III 
Unmodeled   The Mending of Dominaria                 III — Return all land cards from your graveyard to the battlefield, th
Partial     Anikthea, Hand of Erebos                 Other enchantment creatures you control have menace.; Whenever ~ enters or attacks, exile up to one target non-Aura e
Partial     Battle for Bretagard                     III — Choose any number of artifact tokens and/or creature tokens you control wi
Partial     Binding the Old Gods                     I — Destroy target nonland permanent an opponent controls.; III — Creatures you control gain deathtouch until end
Partial     Boon of the Spirit Realm                 Constellation — Whenever ~ or another enchantment you control enters, put a bles
Partial     Calix, Destiny's Hand                    look at the top four cards of your library; reveal an enchantment card from among them and put that card into your ha
Partial     Composer of Spring                       body only, abilities not modeled: Constellation — Whenever an enchantment you control enters, you may pu
Partial     Demon of Fate's Design                   Once during each of your turns, you may cast an enchantment spell by paying life
Partial     Doomwake Giant                           body only, abilities not modeled: Constellation — Whenever ~ or another enchantment you control enters, 
Partial     Dreadhorde Invasion                      Whenever a Zombie token you control with power 6 or greater attacks, it gains li
Partial     Erebos, Bleak-Hearted                    body only, abilities not modeled: As long as your devotion to black is less than five, ~ isn't a creatur; Whenever an
Partial     Golgari Rot Farm                         land utility ignored: When ~ enters, return a land you control to its owner's hand.
Partial     Greater Tanuki                           keywords not modeled: channel
Partial     Heliod, God of the Sun                   As long as your devotion to white is less than five, ~ isn't a creature.
Partial     Khalni Heart Expedition                  Landfall — Whenever a land you control enters, you may put a quest counter on ~.
Partial     Love Song of Night and Day               Read ahead; I — You and target opponent each draw two cards.; III — Put a +1/+1 counter on each of up to two targ
Partial     Mirari's Wake                            Whenever you tap a land for mana, add one mana of any type that land produced.
Partial     Narci, Fable Singer                      body only, abilities not modeled: Whenever you sacrifice an enchantment, draw a card.; Whenever the final chapter abi
Partial     Nessian Wanderer                         body only, abilities not modeled: Constellation — Whenever an enchantment you control enters, look at th
Partial     Nyx Weaver                               {1}{B}{G}, Exile ~: Return target card from your graveyard to your hand.
Partial     Nyxborn Behemoth                         ~ costs {X} less to cast, where X is the total mana value of noncreature enchant
Partial     Ondu Spiritdancer                        body only, abilities not modeled: Whenever an enchantment you control enters, you may create a token tha
Partial     Orzhov Basilica                          land utility ignored: When ~ enters, return a land you control to its owner's hand.
Partial     Sandwurm Convergence                     Creatures with flying can't attack you or planeswalkers you control.
```

## Done when
The T0-RULES checks pass and every card above is Full; a trace shows the deck casting its commander and using
its key cards. Report as those rules say; full report in `rust/agents/report-t0-anikthea.md`.
