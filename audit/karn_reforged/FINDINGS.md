# Karn, Legacy Reforged (key `karn_reforged`): findings

The deck's living list of strengths and weaknesses and what was tested.

**List:** `decklists/JD/karn-reforged-tempo.md`, registered 2026-11-26 (Karn, Legacy Reforged, colourless; since 2026-11-27 28 lands, 71 spells; six
Game Changers, Bracket 4). **Tiers:** t4 and t5 (`data/my_decks.toml`, JD: Commander Bracket 4). Baseline and a re-run after the stax cut are under Tested.

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
Not measured when written (the baseline is under Tested). The retired Karn deck (`audit/retired/karn/FINDINGS.md`, 2026-10, an older engine and list) lost to creature beatdown
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

Goldfish (three inert seats): median kill turn 10.0 (mean 9.8); won by turn 7 in only 7.4% of games (by turn 8, 23.9%); 41% of kills are commander damage; Karn
cast in 99.1% of games, median own turn 4.0; mana-short on 12.3% of turns 1-4; the plan ("ready") is reached in 13.9% of games.

### Why (first reading; the Opus diagnosis of 2026-11-27, `/tmp/handoff/kr-opus-diag.md`, is under it)
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
- Winter Orb and Defense Grid were cut on 2026-11-27 (see the last section; JD kept Unwinding Clock); the stax lines above describe the baseline list.
- Spellbook: the fetch (2026-11-27, `data/spellbook/karn_reforged.json`) lists 6 included combos. Five are modelled or already covered by a line (Basalt Monolith + Rings of Brighthearth; Karn, the Great Creator + Mycosynth Lattice; Sensei's Divining Top + Mystic Forge with Foundry Inspector or Ugin, the Ineffable; Voltaic Key + Rings + Everflowing Chalice). One is not: **Karn, the Great Creator + Aetherflux Reservoir** (variant 4740-5089, infinite damage; it needs a way to give the Reservoir lifelink and a life total of at least 51). It is in `KNOWN_GAPS` in `rust/crates/sim-core/tests/spellbook.rs`, as the marchesa entries are. A line needs the loop in the combo catalogue (`impls/combos.rs`), which is engine work; 51 life is rare at 40 starting life. Ticket.
- Ownership: every card is recorded as `proxy`; JD has not marked which he owns (JD will provide it).

### Opus diagnosis (2026-11-27; a rough mana model and six goldfish traces, not a paired test)
- **Karn on turn 2-3 is rare for this list:** a rough Monte Carlo of the mana puts Karn by T3 at about 24% and by T4 at about 41% (the sim does
  better, 56% by T4, T2 not reachable in most games). Causes: 26 lands, rocks that lose mana the turn they are cast (Coalition Relic, Worn Powerstone,
  Thran Dynamo, the 2-mana rocks), and 15 nonland cards at mana value 6 or more. Karn's cast priority is not a cause (he is offered at 85 and picked 81-100% of the time when castable).
- **AI sequencing cost three turns in 2 of 6 goldfish traces:** Mana Vault spent on Mazemind Tome instead of held for Karn, and Tezzeret's -3 fetching
  Walking Ballista (the plan's `key_cards` 70) with no mana left instead of Sol Ring.
- **The three never-cast cards** have a derived cast priority of 0 (worth 0.05 or less): Sword of Feast and Famine's `sword` tag is never read by
  `ai/values.rs from_tags`; Winter Orb and Unwinding Clock have no tags. The engine models all three effects; only the AI's value is missing.
- **Baseline numbers that mislead:** "Plan:" is empty and "online 2.2-2.4%" because `sim-cli` report.rs has no plan milestone for this deck (it counts only
  the combo line); Fellwar Stone makes {C} in goldfish with no opposing lands.
- Possible tickets: the t5 Urza opponent has no plan file, so its Winter Orb, Static Orb, Back to Basics and Unwinding Clock may never be cast (unconfirmed);
  Karn, Gilded Guardian may share the Sword tag gap (unconfirmed).

## 2026-11-27: stax removed (JD: "remove the stax aspect of the karn deck - that is not a playstyle I like, so like defense grid and winter orb")
- **Out (job lost):** Winter Orb (a stax lock that only stops lands untapping; cast in 0-0.9% of the games it was drawn in the baseline, and it chokes the deck's own Urza lands and Workshop) and Defense Grid (counter protection on the turn Karn lands; it taxes everyone's spells outside their own turn; it was cast in 16.6% of t4 games in the baseline, so not dead, but its +5.8 win rate when cast is among the deck's ten lowest at t4, correlation only; the loss is its job, counter protection on the turn Karn lands, which the sim does not measure).
- **In (job gained):** two Wastes (lands 26 -> 28; the Opus mana model put Karn by turn 4 at about 45% instead of 41%, an estimate). Nothing else changed.
- **Kept on purpose (JD):** Karn, the Great Creator and the Mycosynth Lattice lock; Unwinding Clock (JD: "Keep unwinding clock"; it is an untap engine, not a lock); Strip Mine and Wasteland. A first version of this change also replaced the Clock with Hedron Archive; JD reversed that, and the sims below are of the final list.
- **Counts after (before):** lands 28 (26); mana artifacts 15 (15); disruption 0 (3: Winter Orb, Defense Grid, Unwinding Clock); untap engine 1 (Unwinding Clock, now listed on its own); Game Changers 6 (6). Nothing now protects Karn on the stack (Defense Grid was the only answer to a counterspell on him); once he is on the battlefield Swiftfoot Boots (hexproof), Lightning Greaves (shroud), Champion's Helm (hexproof), Whispersilk Cloak (shroud), Mithril Coat and Hammer of Nazahn (indestructible) protect him, and Command Beacon puts him into your hand from the command zone.
- **Measured (same sims as the baseline: loose, look-ahead, 1,500 games per tier, seed 500000, `--analyze`; goldfish 1,000; two separate runs, not a paired test, about +/-1 point of noise per tier; outputs `/tmp/handoff/kr-sims/` before and `/tmp/handoff/kr-sims-clock/` after):**

| | before (t4 / t5) | after (t4 / t5) |
|---|---|---|
| Win rate | 18.3% / 18.8% | 20.7% / 19.1% |
| Eliminated | 81.7% / 81.1% | 79.2% / 80.8% |
| Karn resolves | 84.5% / 81.9% | 85.7% / 83.9% |
| Karn resolved by T4 | 56% / 52% | 58% / 54% |
| Goldfish median kill turn | 10 | 9 |
| Goldfish won by T7 / T8 (cumulative) | 7.4% / 23.9% | 7.7% / 26.9% |

  Reading: the net is within noise. t4 +2.4 is about 1.6 standard errors of the difference (about 1.46 points each tier), t5 +0.3 is nothing; averaged over t4-t5 it is +1.35 (18.55% -> 19.9%) with a 95% interval of about -0.7 to +3.4. So the data shows no measurable cost and cannot show a gain; a paired `--swap` package test would be needed for a confirmed number. Winter Orb was nearly dead in the baseline; Defense Grid was not, so its counter protection is a real loss that the sim does not show as a cost. Unwinding Clock is still a dead card: cast in 0.4% of the games it was drawn at t4 and never at t5 (the AI values it at 0). The two extra lands moved Karn's timing +2 points per tier (resolved by T4: 56% -> 58% at t4, 52% -> 54% at t5), within noise (about 1.8 points of standard error) and not distinguishable from the Opus mana model's +4 (41% -> 45%); the goldfish moves (mean kill turn 9.8 -> 9.6) are within noise too. The AI sequencing fixes (Mana Vault held for Karn, tutors fetching Sol Ring) and a plan row for the Clock are still unmeasured. Combo opponents (Kinnan, Urza, Zur) eliminate it in 41.5% of t5 games (41.3% before).
