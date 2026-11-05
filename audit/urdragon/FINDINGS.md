# The Ur-Dragon (deck key `urdragon`, Tier 5)

The list is in `decklists/JD/ur-dragon-equipment.md` (Bracket 4, 16 Game Changers). The deck joined the sim on
2026-10-30. It plays from `data/plans/urdragon.toml` through the generic policy, with no hand-written deck AI.

## 2026-11-04: Urza, Lord High Artificer for Jaxis, the Troublemaker (JD's choice; not simulated)
Urza came out of Sauron's binder. With him out, the six artifacts that make no mana (Lightning Greaves, The One Ring,
Cursed Mirror, Maskwood Nexus, Scion of Draco, his Construct) tap for {U}; the Construct grows with the deck's 15
artifacts, and Maskwood Nexus makes both Dragons. Jaxis was a sorcery-speed copy engine outside the three combos
(Spark Double, Twinflame and Cursed Mirror still copy); it left the plan file's Dragon-copies line. Wrenn and Six,
Decimate, Pact of Negation and Lightning Greaves were the other cuts considered. Measure over T5.

## 2026-11-03: Dragon-copy rebuild (JD's choice; not simulated)

The Godo, Helm and Equipment package moved to Sauron. Twenty slots now form a Dragon entry, token and copy package
led by Astral Dragon, Miirym, Lathliss, Ganax, Dragon Tempest, Scourge of Valkas, Terror of the Peaks and Utvara
Hellkite. Garth One-Eye and Scion of Draco then replaced the proposed Dragonborn Champion and Wrathful Red Dragon:
Garth adds a flexible paper toolbox, while Scion is an efficiently discounted Dragon that distributes keywords. The
old results below describe the retired Godo list and are not a baseline for the rebuilt deck. The next measurement
must use the matched T5 tier, loose profile and the default look-ahead AI. (Done 2026-11-04: first baseline 21.1%, last section.)

The simulator rates Scion of Draco Full. Garth One-Eye is Partial: it models the 5/5 body but not the tap ability or
the six named spells, so do not use a simulation to judge Garth until that behavior is implemented. (Superseded 2026-11-04: the 11/04 coverage line shows Partial 0 for the deck, and Garth is modelled in the Twinflame combo; the baseline in the last section includes him.) No simulation was
run for this list-only refinement.

The final owned-card pass replaced five proposed purchases: Ancient Copper Dragon for Dracogenesis, Doubling Season
for Panharmonicon, Korvold for Kindred Discovery, Dragonlord Dromoka for Atarka, and Scion of the Ur-Dragon for
Sarkhan. Ojutai stays. This creates a secondary Treasure engine: Copper and Ganax make tokens, Doubling Season doubles
them, and Korvold turns spending them into cards and counters. Dromoka protects proactive turns, while Scion supplies
the Dragon toolbox. Ten rebuild cards remain to buy. Copper, Doubling Season, Dromoka and Korvold are Full; Scion of
the Ur-Dragon is Approximate. No simulation was run for this list-only refinement.

## Retired Godo-list result (loose, 2000 games, seeds 500000-501999)
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

## 2026-11-01: recalibration stage 2 remeasure

The Ur-Dragon, t5: **19.2%** (CI 17.5-21.0, 1999 games: one skipped by an internal overflow panic) (2000 games, seeds 500000-501999, loose, look-ahead, defaults, rc=0; `audit/recal/stage2-2026-11-01.md` section 4, build efe45425). Stage 1, same seeds: 19.3%; the 10/30 number was 19.1%. Fitted t5 knobs: 19.0% (level).

## 2026-11-02: Sting, the Glinting Dagger out, Mox Jasper in (JD's choice, not simulated)

- **Why:** Jasper is a free artifact. It makes one mana of any colour while JD controls a Dragon: the eight Dragons in
  the 99, or any creature once Maskwood Nexus is out. It is also an artifact for Puresteel Paladin's metalcraft.
- **What it costs:** Sting untapped Gilgamesh every combat, so he attacked and dug six cards each combat without Godo.
  That is a value effect, not part of a win line.
- **Expectation:** Jasper is dead on turns 1-3 in most games. The early Dragons cost four to six, and The Ur-Dragon is
  cast in about a third of games, at a median own turn of 8. A cheap Dragon would turn it on early. Universal Automaton
  ({1} artifact changeling) is the candidate, still unmodelled.
- **Model:** Mox Jasper is Full (impls/t45_urdragon.rs; tests in cards_t45_urdragon.rs).
- **Number:** none yet. JD asked for no sims (11/02). The swap to run is `--deck urdragon --pool t5 --swap
  "Mox Jasper=>Sting, the Glinting Dagger"`, against the new list.
- **Stale above:** Cyclonic Rift and Conqueror's Flail, in the 10/30 weaknesses, have since left the list.

## 2026-11-04: first baseline of the rebuilt Dragon-copies list (t5)

List: `decklists/JD/ur-dragon-dragon-copies.md` (the 11/03 rebuild with the owned-card swaps). Run: The Ur-Dragon vs t5
(High B4: Zur, Kinnan, Miirym, Yawgmoth, Urza), 2,000 games, seed 500000, loose, look-ahead AI, defaults, on main at
PR #238 (build b8fc69f9). Raw: `.claude/worktrees/sim-runs-2026-11-04/urdragon-baseline.txt`.

- **Win rate 21.1% (95% CI 19.4-23.0%).** The even share is 25%, so it is 3.9 points under. This is the first number
  for the rebuilt list. The Godo-list results above and the 11/01 remeasure describe a different list and are not
  compared with it.
- **Win and loss turns:** average own turn of wins 10.1, of losses 7.6. It wins late and loses earlier.
- **How it wins: damage 93%, combo 7%.** The plan (first combo attempt) came online in 1.8% of games, so the three
  catalogue combos are a rare finish (7% of wins is about 1.5% of games). The Dragon damage plan carries the deck.
- **Who eliminates it** (share of all games in which that opponent dealt the finishing blow): Kinnan 20.6%, Miirym
  17.4%, Urza 15.1%, Yawgmoth 13.2%, Zur 12.1%. These add to 78.4%; the deck wins 21.1% and the remaining 0.5% is
  neither (my arithmetic from the table).
- **Win rate by opponent seated** (each is seated in about 60% of games, so about 1,200 games per cell, roughly +-2.5
  points at 95%): Kinnan 34.0%, Miirym 28.8%, Urza 25.8%, Yawgmoth 21.5%, Zur 21.2%. Kinnan's 34.0% is clearly above
  Yawgmoth's and Zur's; the other gaps are small.
- **Old vs new not run, on purpose.** The rewritten plan file (`data/plans/urdragon.toml`) no longer casts the old
  Equipment (preflight: Embercleave and Sword of Feast and Famine cast 0%), so an old-list run would measure a crippled
  list rather than the retired one.
- **Crash check:** a pre-fix run of the same seeds (`urdragon-baseline-prefix.txt`, before PR #238) skipped 10 of 2,000
  games on a counter-multiplier overflow (`engine/repl.rs:383`, e.g. seeds 500196 and 500203) and gave the same 21.1%
  (CI 19.4-23.0, 1,990 games; wins 10.1, losses 7.5 turns). The baseline above has no skipped games. The 11/01
  remeasure's "one skipped by an internal overflow panic" was the same bug.

**Caveats on what the number includes**
- **Approximate cards:** the coverage line reads Full 94, Full-auto 4, Approximate 2, Partial 0, Unmodeled 0. By
  `data/audit.toml` the two Approximate cards are **Astral Dragon** (it does not copy lands; a nonland permanent is the
  better target in this deck anyway) and **Scion of the Ur-Dragon** (it does not copy a Dragon with an enters ability or
  a cost-reduction static the AI doesn't read). Cursed Mirror has no Approximate entry there and
  `rust/agents/report-urdragon-approx-cards.md` calls it Full (the AI can copy only non-token, non-commander creatures
  that aren't its own legends). Garth One-Eye is no longer Partial (see the 11/03 note above).
- **The three catalogue combos** (Astral Dragon + Cursed Mirror; Miirym + Astral Dragon + Doubling Season; Garth One-Eye
  + Twinflame + Spark Double + Doubling Season) are stepped combos (`impls/t45_urdragon_combos.rs`) with these
  simplifications: the Lotus and Braingeyser rounds are not played out (the finish resolves the loop); Twinflame
  targets one Garth; the Regrowth copy fires no cast triggers; lines 1 and 3 need the Mirror or Spark Double already on
  the battlefield (a piece in hand doesn't count); line 2 stops at the engine's token cap. Full list:
  `rust/agents/report-urdragon-combos.md`.
- **Opponents see the combo cards now.** The combo cards are catalogue piece names for every deck (`is_piece_name`), so
  opponents' removal and counter choices treat them as combo pieces. That is part of this baseline, and it can shift
  the Miirym pool deck and Sauron slightly.
- **One seed, 2,000 games.** The CI above is the only noise estimate.

**Still open:** the Mox Jasper for Sting swap (11/02) has not been run on this list; Universal Automaton is still
unmodelled. "Strengths", "Weaknesses" and "Model status" under the Retired Godo-list heading describe the old list. A
card report (`--analyze`) of the new list is the next step to learn what sits in hand.

## 2026-11-04 (later): Spark Double fixed, t5 re-check

Spark Double never worked for the AI: the 0/0 died to the toughness check before its enter hook copied (a ported
Python bug that `tests/cards_common2.rs` even asserted). It now enters as the copy, as a replacement
(`impls::common::spark_copy`), with its additional +1/+1 (or loyalty) counter, and it isn't legendary (before, a copy
of a legend met the legend rule at once). Found through the practice-mode Garth loop shortcut.

Re-run of the baseline above, same settings (t5, 2,000 games, seed 500000, loose, look-ahead), on the practice-mode
follow-ups branch (all its other changes are person-only):

- **Win rate 22.1% (95% CI 20.3-24.0%)**, against 21.1% (19.4-23.0) before: +1.0 point, inside the overlap of the
  two intervals, so a small lift at most. Wins by damage 94%, combo 6% (7% before). Win turn 10.0, loss turn 7.5.
- Why it is small: Spark Double is one card in 100, drawn in about a quarter of games, and when it lands it is a
  second copy of the best creature (a Dragon) rather than a dead card. The Garth line (Spark Double's Garth) can now
  start in an AI game too, but the catalogue's Garth combo is still rare.
- Raw: `.claude/worktrees/sim-runs-2026-11-04/urdragon-spark-fix.txt`.
