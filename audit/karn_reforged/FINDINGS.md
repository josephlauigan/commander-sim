# Karn, Legacy Reforged (key `karn_reforged`): findings

The deck's living list of strengths and weaknesses and what was tested.

**List:** `decklists/JD/karn-reforged-tempo.md`, registered 2026-11-26 (Karn, Legacy Reforged, colourless; 26 lands, 73 spells; six
Game Changers, Bracket 4). **Tiers:** t4 and t5 (`data/my_decks.toml`, JD: Commander Bracket 4). Not simulated yet.

## Plan (what the deck is meant to do)
- **Engine:** fast mana and cheap artifacts put Karn down on turn 2-3; his upkeep mana ({C} per artifact, for artifact spells
  and ability costs) pays for equip, Skullclamp, Mystic Forge, Rogue's Passage and the draw lands.
- **Wins:** Karn carrying a Sword or Batterskull, unblockable (Whispersilk Cloak, Rogue's Passage) or hexproof (Champion's Helm,
  Swiftfoot Boots); Wurmcoil Engine, Blightsteel Colossus and the Ugins behind him; Basalt Monolith + Rings of Brighthearth
  into Walking Ballista.
- **Cut from the first draft (list review, 2026-11-26):** Mana Crypt, Jeweled Lotus, Sundering Titan (banned in Commander);
  Isochron Scepter (no instants), Academy Manufactor (no Treasure, Food or Clue makers), Ratchet Bomb (destroys our own
  cheap artifacts), Panharmonicon, Colossus Hammer and Hall of the Bandit Lord (the haste rider is not modelled); Gilded Lotus,
  Hedron Archive and Burnished Hart (too slow for a tempo deck).

## Strengths and weaknesses
Not measured. The retired Karn deck (`audit/retired/karn/FINDINGS.md`, 2026-10, an older engine and list) lost to creature beatdown
(eliminated in 77% of games against Tier 0), saw Karn removed 0.87 times a game, and cast its finishers in only 10-16% of games. This
list adds Boots, Greaves, Mithril Coat and hexproof equipment against the removal, and Meteor Golem, Duplicant, Spine and the two Ugins
against the beatdown; whether that is enough is the first thing the baseline must show.

## Tested
Nothing yet.

## Limits and open items
- Wasteland and Umezawa's Jitte are modelled in `impls/kr_model.rs`. Jitte limits (none can matter at t4-t5 per the Opus review, JD to accept): one blocker per attacker (engine-wide), no trigger for Pariah-redirected damage, Gisela's halving not applied, and the counters are added at once instead of by a stack trigger (the opponent gets no response, lifelink comes after). The AI spends Jitte counters only after blocks and in its own main phase, never in response to removal or between first-strike and regular damage.
- Aetherflux Reservoir and Steel Overseer have no shared plan with the rest of the list (Opus review); keep them only if the
  baseline shows them cast and useful.
- Winter Orb is asymmetric here (Karn's mana and the rocks ignore it) but chokes our own Tron lands and Workshop; keep it only with evidence.
- Ownership: every card is recorded as `proxy`; JD has not marked which he owns.
