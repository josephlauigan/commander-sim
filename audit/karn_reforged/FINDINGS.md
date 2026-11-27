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

### Baseline: the registered list (loose profile, look-ahead AI, 1,500 games per tier, seed 500000, `--analyze`; goldfish 1,000 games)
Outputs: `/tmp/handoff/kr-sims/` (`pool-all.txt`, `goldfish.txt`). All 100 cards are Full.

| | t4 (Low B4) | t5 (High B4) |
|---|---|---|
| Win rate (4-player baseline 25%) | 18.3% | 18.8% |
| Average rounds survived | 8.4 | 8.7 |
| Eliminated | 81.7% (round 7.9) | 81.1% (round 8.1) |
| Karn resolves | 84.5% of games, median turn 4.0 (by T4 56%) | 81.9%, median turn 4.0 (by T4 52%) |
| Karn removed per game | 0.37 | 0.28 |
| Wins by | damage 94.5%, combo 5.5% | damage 95.4%, combo 3.2% |
| Opponents it eliminates per game | 0.48 (combat 61%, commander damage 25%) | 0.53 (combat 58%, commander damage 31%) |
| Main killers | Krenko combat 12.4%, Edgar Markov combat 10.5%, Chulane combo 10.1%, Yuriko 8.3% | Kinnan combo 16.3%, Urza combo 15.1%, Zur combo 9.9%, Miirym 9.1% |

Goldfish (three inert seats): median kill turn 10.0 (mean 9.8); won by turn 7 in only 9.1% of games (by turn 8, 33.0%); 41% of kills are commander damage; Karn
cast in 99.1% of games, median own turn 4.0; mana-short on 12.3% of turns 1-4; the plan ("ready") is reached in 13.9% of games.

### Why (first reading, not yet confirmed by a trace)
- **It is not a tempo deck in the sim.** Karn lands on turn 4 at the median, not turn 2-3, and the goldfish kill is turn 10.
- **Three cards are never cast by the sim and are dead in these numbers:** Sword of Feast and Famine (tag `sword`, but never offered as a play:
  payable in 980 goldfish turns), Winter Orb and Unwinding Clock (no tags: they come from Scryfall with no ability the AI plays). The
  engine status of all three is Full, so the audit status does not show this; the card report does.
- **The finishers win when they are cast but are cast late:** Ugin the Spirit Dragon, Spine of Ish Sah, Portal to Phyrexia and Ugin the Ineffable
  are cast in only 34-47% of the games they are drawn; All Is Dust, Blightsteel, Portal and the Ugins show the highest win rate when cast
  (44-62%, correlation only).
- **The cheap artifacts do not convert:** Hope of Ghirapur, Sensei's Divining Top, Ornithopter, Mishra's Bauble and Hangarback Walker are cast
  in 16-19% of games with a win rate within 2-8 points of the deck's own.
- **Opponent combos beat it at t5** (Kinnan, Urza and Zur eliminate it in 41% of games): the list has no interaction that stops a combo.


## Limits and open items
- Wasteland and Umezawa's Jitte are modelled in `impls/kr_model.rs`. Jitte limits (none can matter at t4-t5 per the Opus review, JD to accept): one blocker per attacker (engine-wide), no trigger for Pariah-redirected damage, Gisela's halving not applied, and the counters are added at once instead of by a stack trigger (the opponent gets no response, lifelink comes after). The AI spends Jitte counters only after blocks and in its own main phase, never in response to removal or between first-strike and regular damage.
- Aetherflux Reservoir and Steel Overseer have no shared plan with the rest of the list (Opus review); keep them only if the
  baseline shows them cast and useful.
- Winter Orb is asymmetric here (Karn's mana and the rocks ignore it) but chokes our own Tron lands and Workshop; keep it only with evidence.
- Ownership: every card is recorded as `proxy`; JD has not marked which he owns.
