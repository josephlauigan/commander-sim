# Zada, Hedron Grinder — Mono-Red Go-Wide

2026-10-11
**Added 2026-10-11.** Built from JD's bulk and binder around Zada, Hedron Grinder (a commander to buy, about $0.17). Not yet in the simulator's measured decks: Zada's own ability and many cards still need modeling (see "Simulator status").

## Strategy

Go wide with cheap creatures and tokens, then cast one cheap spell on Zada and let it hit the whole team.

Zada, Hedron Grinder ({3}{R}, 3/3 Goblin Ally) reads: *whenever you cast an instant or sorcery spell that targets only Zada, copy that spell for each other creature you control that the spell could target. Each copy targets a different one of those creatures.*

**The Zada spells (12).** Each is cheap and helps the creature it targets, so with Zada out it helps every creature:
- **Cards:** Ancestral Anger draws a card for each copy; Blazing Crescendo exiles a card you may play for each copy.
- **More bodies:** Twinflame makes a hasty token copy of every creature you control.
- **Damage:** Monstrous Rage (a +1/+1 trample Monster Role for everyone), Sure Strike, Furious Bellow, Smaug's Fury, Crowd's Favor (convoke: the team pays for it), Bulk Up (double each creature's power; flashback).
- **Utility:** Act of Treason on Zada untaps the team and gives it haste; Into the Fray is a cheap trigger for the spell-payoff creatures; Return the Favor copies Zada's trigger, so every creature gets two copies.

**Bodies (about 30 creatures, plus tokens).** Mogg War Marshal, Goblin Arsonist, Heartstring Puller, Spawn-Gang Commander, Pia, Determined Rebuilder, Strife Scholar, Goblin Glasswright, Command the Stage, Living History and Empty the Warrens (storm grows with the cheap spells) make two bodies or more for one card. Rundvelt Hordemaster pumps every Goblin, Zada included.

**Spell payoffs.** Speed, Young Avenger, Chandra's Emberling, Spellgorger Weird, Burning Prophet, Expressive Firedancer, Thunderdrum Soloist and Firebird grow or deal damage on every instant or sorcery. Dreadhorde Arcanist casts a one-mana Zada spell again from the graveyard on each attack (its power is 1; pump it to reach more, and that cast can target Zada too), Zealous Lorecaster returns one to hand.

**Card flow.** Ancestral Anger and Blazing Crescendo are the deck's best draw. Laelia, Lightning, Security Sergeant, Irascible Wolverine and Glimpse the Impossible exile cards to play; Rummaging Goblin, Seize the Spoils, Orcish Cannonade and Skyscanner fill the gaps.

## Key lines

**The usual turn:** three or four creatures down by turn 4, Zada on 4 or 5, then a one-mana Zada spell. Ancestral Anger with five other creatures draws six cards; Monstrous Rage gives six creatures a permanent +1/+1 and trample.

**Twinflame into a pump spell.** Twinflame on Zada doubles the board (the tokens have haste); a Zada spell after it reaches twice as many creatures. The copies are exiled at end of turn, so do it on the turn you attack.

**Haste for the tokens.** Akki Drillmaster, Taxi Driver and Act of Treason on Zada let fresh tokens attack the turn they arrive.

## Consistency

- **Lands:** 32, none of which enters tapped: 31 Mountain and Bonders' Enclave ({3}, {T}: draw a card with a 4-power creature out). Path of Ancestry was the one tapped land; a Mountain took its place. The curve is low (about 60 of the 67 spells cost 3 or less), so 32 lands and six rocks are enough.
- **Mana:** Arcane Signet, Relic of Legends, Foraging Wickermaw, Giant's Boulder, Potioner's Trove, Generator Servant.
- **Interaction:** Lightning Strike, Scorching Dragonfire, Suplex, Reiterating Bolt, Unsubtle Mockery, Wrath of the Bloodmane, Violent Echoes, Heated Argument, Burn, Burn, Tree and Fern; Ral and the Implicit Maze is a one-sided sweeper (2 to each creature and planeswalker opponents control).
- **Protection:** Darksteel Plate and Champion's Helm. No shroud: it would stop the Zada spells from targeting him.

**Weak spots.** Twelve Zada spells is light (dedicated lists run 18 to 25), so most games play as red go-wide aggro with Zada as the finisher. No board wipe for your own side and no hexproof spells in red. Cheap commons that would add Zada spells, if the bulk-only rule is ever lifted: Expedite, Crimson Wisps, Brute Force, Titan's Strength, Rush of Adrenaline, Infuriate, Fists of Flame, Fling.

## Bracket and Rule 0

**Bracket 2:** no Game Changers, no tutors, no infinite combos. Disclose the Zada copies (one spell can draw five or six cards or pump a whole board) and Empty the Warrens' storm.

## Simulator status

In `data/my_decks.toml` as deck key `zada`, but **not measured yet**: 40 of its 70 different cards aren't fully modeled (`cardgen audit --mine`), and Zada's own copy ability isn't modeled at all (body only). Unmodeled: Burn, Burn, Tree and Fern, Into the Fray, Ral and the Implicit Maze; Generator Servant is approximate; the rest are partial. Zada needs card code (copy a spell for each other creature it could target) and an AI plan: when to cast the Zada spells (with Zada and enough creatures out) instead of on other targets.

## Where the cards come from

Every card except the commander and the basics comes from JD's bulk and binder lists (bulk-cards.txt, binder-cards.txt). They are still on the lists: take them off when the deck is built.

## Decklist by type (100)

**Commander (1).** Zada, Hedron Grinder

**Creatures (31).** Akki Drillmaster, Bomber Corps, Burning Prophet, Chandra's Emberling, Dreadhorde Arcanist, Expressive Firedancer, Firebird Blazing Ranger, Foraging Wickermaw, Generator Servant, Glitterfang, Goblin Arsonist, Goblin Glasswright, Goblin Rimerunner, Heartstring Puller, Irascible Wolverine, Krenko's Enforcer, Laelia the Blade Reforged, Lightning Security Sergeant, Mogg War Marshal, Pia Determined Rebuilder, Rummaging Goblin, Rundvelt Hordemaster, Skilled Battlecarver, Skyscanner, Spawn-Gang Commander, Speed Young Avenger, Spellgorger Weird, Strife Scholar, Taxi Driver, Thunderdrum Soloist, Zealous Lorecaster

**Enchantments (3).** Burn Burn Tree and Fern, Living History, Ral and the Implicit Maze

**Artifacts (6).** Arcane Signet, Champion's Helm, Darksteel Plate, Giant's Boulder, Potioner's Trove, Relic of Legends

**Sorceries (10).** Act of Treason, Ancestral Anger, Artifist Acumen, Command the Stage, Empty the Warrens, Glimpse the Impossible, Reiterating Bolt, Seize the Spoils, Suplex, Twinflame

**Instants (17).** Blazing Crescendo, Bulk Up, Crowd's Favor, Furious Bellow, Heated Argument, Into the Fray, Lightning Strike, Monstrous Rage, Orcish Cannonade, Path of Anger's Flame, Return the Favor, Scorching Dragonfire, Smaug's Fury, Sure Strike, Unsubtle Mockery, Violent Echoes, Wrath of the Bloodmane

**Lands (32).** Bonders' Enclave, 31 Mountain

## Import list (100)

```
1 Act of Treason
1 Akki Drillmaster
1 Ancestral Anger
1 Arcane Signet
1 Artifist Acumen
1 Blazing Crescendo
1 Bomber Corps
1 Bonders' Enclave
1 Bulk Up
1 Burn, Burn, Tree and Fern
1 Burning Prophet
1 Champion's Helm
1 Chandra's Emberling
1 Command the Stage
1 Crowd's Favor
1 Darksteel Plate
1 Dreadhorde Arcanist
1 Empty the Warrens
1 Expressive Firedancer
1 Firebird, Blazing Ranger
1 Foraging Wickermaw
1 Furious Bellow
1 Generator Servant
1 Giant's Boulder
1 Glimpse the Impossible
1 Glitterfang
1 Goblin Arsonist
1 Goblin Glasswright
1 Goblin Rimerunner
1 Heartstring Puller
1 Heated Argument
1 Into the Fray
1 Irascible Wolverine
1 Krenko's Enforcer
1 Laelia, the Blade Reforged
1 Lightning, Security Sergeant
1 Lightning Strike
1 Living History
1 Mogg War Marshal
1 Monstrous Rage
1 Orcish Cannonade
1 Path of Anger's Flame
1 Pia, Determined Rebuilder
1 Potioner's Trove
1 Ral and the Implicit Maze
1 Reiterating Bolt
1 Relic of Legends
1 Return the Favor
1 Rummaging Goblin
1 Rundvelt Hordemaster
1 Scorching Dragonfire
1 Seize the Spoils
1 Skilled Battlecarver
1 Skyscanner
1 Smaug's Fury
1 Spawn-Gang Commander
1 Speed, Young Avenger
1 Spellgorger Weird
1 Strife Scholar
1 Suplex
1 Sure Strike
1 Taxi Driver
1 Thunderdrum Soloist
1 Twinflame
1 Unsubtle Mockery
1 Violent Echoes
1 Wrath of the Bloodmane
1 Zada, Hedron Grinder
1 Zealous Lorecaster
31 Mountain
```
