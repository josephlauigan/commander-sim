# Task: t0-captain — model the Captain America, Team Leader precon's last 27 cards

Read `rust/agents/T0-RULES.md` first. Deck: `decklists/pool/t0-b2-precons/captain-america-avengers-assemble.md` (pool key `captain-america-avengers-assemble`), an unmodified
WotC precon in the new Tier 0 (Bracket 2). Your module: `impls/t0_captain.rs` (stub in place), tests `tests/cards_t0_captain.rs`.
You own: Heroes and Hero cost reductions, Backgrounds (Folk Hero), casting spells as though they had flash, "becomes tapped" triggers.

**Check the commander first:** Captain America, Team Leader must do everything its Oracle text says (it is in your list if the audit says
it isn't Full). The deck plays through the outside-deck AI (`ai/pool.rs`): make sure that AI casts the commander early
and uses its abilities; trace a game with `commander-sim --deck elrond --pool t0 --trace 0` (and further game numbers until that deck is seated: each game draws three of the five) and follow Captain America, Team Leader.

## Cards to model (until `cardgen audit --deck captain-america-avengers-assemble` shows each Full)

```
Unmodeled   Avengers Quinjet                         Whenever this Vehicle enters or attacks, choose one —; • You may put a Hero creature card from your hand onto the
Unmodeled   Folk Hero                                Commander creatures you own have "Whenever you cast a spell that share
Unmodeled   Gift of Immortality                      Enchant creature; When enchanted creature dies, return that card to the battlefield unde
Unmodeled   Herald's Horn                            Creature spells you cast of the chosen type cost {1} less to cast.; At the beginning of your upkeep, look at the top 
Unmodeled   Love on the Battlefield                  Whenever you attack with exactly two creatures, those creatures gain f
Partial     Ant-Man, Elusive Avenger                 ~ can't be blocked by creatures with greater power.
Partial     Avenge                                   ~ costs {2} less to cast if a player attacked you during their last turn.
Partial     Avengers Tower                           land utility ignored: {4}, {T}: Look at the top three cards of your library. You may reveal 
Partial     Black Widow, Agile Avenger               body only, abilities not modeled: Whenever an opponent draws their second card each turn, put a +1/+1 co
Partial     Captain America, Living Legend           body only, abilities not modeled: Whenever a creature you control becomes tapped during your turn, if it
Partial     Captain America, Team Leader             body only, abilities not modeled: Whenever another Hero you control enters, it gains vigilance and haste
Partial     Captain Mar-Vell, Space-Born             body only, abilities not modeled: Cosmic Awareness — As long as an opponent has cast a spell this turn, 
Partial     Captain Marvel, Apex Avenger             body only, abilities not modeled: Whenever you put one or more counters on another creature, if it's not
Partial     Director Nick Fury                       body only, abilities not modeled: Hero spells you cast cost {1} less to cast.; Whenever you attack, look at the top f
Partial     Dismantling Wave                         When you cycle ~, destroy all artifacts and enchantments.; keywords not modeled: cycling
Partial     Hawkeye, Avenging Archer                 Whenever a creature an opponent controls dies, if ~ dealt damage to it this turn
Partial     Hercules, Olympian Hero                  body only, abilities not modeled: Whenever ~ attacks, put a +1/+1 counter on him. He gains indestructibl; Whenever ~ 
Partial     Heroic Return                            ~ costs {2} less to cast if a creature is attacking you.; Return target creature card from your graveyard to the batt
Partial     Jarvis, Earth's Mightiest Butler         body only, abilities not modeled: Whenever you cast a Hero spell, draw a card.
Partial     Photon, Mighty Marvel                    body only, abilities not modeled: Whenever ~ deals combat damage to a player, add that much mana of any 
Partial     Quicksilver, Speedster                   body only, abilities not modeled: As long as ~ is tapped, you may cast spells as though they had flash.
Partial     Raise the Palisade                       Choose a creature type. Return all creatures that aren't of the chosen type to t
Partial     Shang-Chi and the Ten Rings              body only, abilities not modeled: Whenever you draw a card, put a +1/+1 counter on Shang-Chi.; When the tenth +1/+1 c
Partial     She-Hulk, Wallbreaker                    body only, abilities not modeled: Other Heroes you control have trample.; Whenever a Hero you control becomes blocked
Partial     Winter Soldier, Reborn Avenger           body only, abilities not modeled: Whenever ~ attacks, return target creature card with mana value less t
Approximate Falcon and Redwing                       regex tags: tok tokfly tokp
Approximate Heroic Sacrifice                         regex tags: draw
```

## Done when
The T0-RULES checks pass and every card above is Full; a trace shows the deck casting its commander and using
its key cards. Report as those rules say; full report in `rust/agents/report-t0-captain.md`.
