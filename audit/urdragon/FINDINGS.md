# The Ur-Dragon (deck key `urdragon`, Tier 5)

The list is in `decklists/JD/ur-dragon-equipment.md` (Bracket 4, 17 Game Changers). The deck joined the sim on
2026-10-30. It plays from `data/plans/urdragon.toml` through the generic policy, with no hand-written deck AI.

## Result (loose, 2000 games, seeds 500000-501999)
- **t5: 19.1% (CI 17.4-20.9).** The even share is 25%, so it is 5.9 points under.
  - Wins: damage 58%, combo 42%.
  - The plan came online (first combo attempt) in 9.3% of games.
  - Average own turn of wins: 8.2; of losses: 7.2.
- **Goldfish:** median kill turn 9 (mean 8.9).
  - 45.2% of games end by turn 8.
  - Kills: damage 62.5%, combo 30.7%, commander damage 6.8%.
  - The plan came online in 30% of games.
  - The Ur-Dragon was cast in 33.5% of games, at a median own turn of 8.

## How it dies
Finishing blows: Kinnan 22.4% of games, Miirym 15.8%, Urza 14.4%, Yawgmoth 13.6%, Zur 12.9%.

## Strengths
- The Godo, Celebrant and Dualcaster lines all fire.
- The lines most often chosen when ready: Helm of the Host + Combat Celebrant, Godo + Brass Squire, Godo + Hammer.

## Weaknesses
- **Mulligans:** the rule (2-5 lands and 1 line piece) sends 16.6% of hands to 5 cards or fewer.
- **Mana:** short on 26.9% of turns 1-4.
- **Cards often passed while castable:** Twinflame, Nature's Lore, the Boots and Greaves, Imperial Recruiter.
- **Cards often held all game:** Pact of Negation, Cyclonic Rift, Conqueror's Flail, Swords to Plowshares.
- **Cards often discarded to hand size:** Kindred Dominance, Flail and the Swords are among the most discarded.

## Model status
- **Full:** Godo, Helm of the Host, Combat Celebrant (exert), Gilgamesh, Maskwood Nexus, Hammer of Nazahn, Gaea's
  Cradle, Brass Squire, Puresteel Paladin, Sigarda's Aid, Twinflame, Dualcaster Mage, Jaxis, the other Equipment.
- **Kindred Dominance: Full.** It names the creature type that costs the caster least; with Maskwood Nexus out, the
  caster's creatures are every type.
- **Kaldra Compleat: Approximate.** Living weapon, +5/+5, first strike, trample, indestructible and haste are
  modelled. Exiling a creature it damages is not.
- **Approximate:** The Ur-Dragon, Wrenn and Six.
- **Partial:**
  - Dragonlord Ojutai (body only).
  - Nicol Bolas, the Ravager (his enter-discard works; the flip doesn't).
  - Peter Parker's Camera.

## Next to try
- Loosen the mulligan (`pieces = 0`).
- Give the plan file cast priorities for Helm, the Celebrant and the tutors.

## 2026-10-31: card models (branch card-models-1; no sims until Phase 4 recalibration)

Dragonlord Ojutai: Partial -> Full (hexproof while untapped, the combat-damage dig). Nicol Bolas, the Ravager: Partial -> Approximate (the flip to the Arisen; an Arisen that leaves is a planeswalker card in the graveyard). Still Partial: Peter Parker's Camera. No new number yet.
