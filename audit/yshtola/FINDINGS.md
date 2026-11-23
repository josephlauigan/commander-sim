# Y'shtola, Night's Blessed — findings (living file)

What the simulator has taught us about the Y'shtola Esper drain deck: strengths, weaknesses, and what was tried and
why it did or didn't work. Updated after every run. Newest evidence first. Matched tier: T5 (`data/my_decks.toml`).

## 2026-11-22: Loran of the Third Path => Sapphire Medallion (JD's choice, not simulated)
Sapphire Medallion ({2}, an artifact: blue spells cost {1} less) takes Loran of the Third Path's slot. An Opus consult advised against it:
this is not a slot-for-slot swap (removal on a body for a cost reducer), and Loran was an answer to the artifact combos that take the most
games (Urza 13.2%, Kinnan 12.9% in the 11/07 run); with the blink pieces gone since the B swap, Loran's value was one destroy trigger and
a symmetrical draw. What it gains: {1} off the blue spells with generic mana in their cost (Rhystic Study, Mindbreak Trap, Zur, Teferi, Time
Raveler, Notion Thief and others); Counterspell, Absorb and Thassa's Oracle get nothing, and Doomsday is black. Triggers are unchanged:
Loran was a creature, and the Medallion has mana value 2. Counts: 12 creatures plus Y'shtola (13), artifacts 8 (7), two-drops 12 (11), three-drops 19
(20). The deck's artifact and enchantment answers are now Skyclave Apparition, Anguished Unmaking, Generous Gift, Prayer of Binding, and Austere Command and Farewell in their artifact and enchantment modes.
Model status: Full-auto before this change; now Full, with its own test (`sapphire_medallion_cuts_one_generic_from_blue_spells_only`). Not measured.

## 2026-11-18: Venat, Heart of Hydaelyn // Hydaelyn => Absorb (JD's choice, not simulated)
Absorb ({W}{U}{U} instant, counter target spell and you gain 3 life; mana value 3; not a Game Changer, so the list stays
at 16 and Bracket 4) takes Venat's slot. Why: Venat is a creature, so it never triggers Y'shtola, and its draw needs a
legendary spell, of which the list has only seven others (Y'shtola, Augustin, Zur, Sheoldred, Loran, Teferi, Palantír);
its draw is also forced, which shortens a Doomsday pile, and its {W}{W} sits badly beside the {U}{U} of Oracle and the
{B}{B}{B} of Doomsday. Absorb is a counter that aims at the deck's largest documented loss (fast combos took about 44% of
the t5 games in the 11/07 analyze run: Urza 13.2%, Kinnan 12.9%, Zur 9.4%, Yawgmoth 8.4%), and it is a spell of mana value
3, so it triggers her (2 to each opponent, and the trigger resolves first even if Absorb is countered). Its own 3 life is a
separate gain, so Sanguine Bond and Marauding Blight-Priest each fire on it. Cast on an opponent's turn it is the second
trigger toward her end-step draw. The list now has 29 triggering spells (28 before; the 11/15 Mindbreak Trap entry left the Trap, a mana value 4 instant, out of the guide's count, and this entry corrects it), 14 creatures (15), five counters
(Absorb, Counterspell, Disdainful Stroke, Force of Will, Mindbreak Trap; four before) and 18 instants (17). The average mana value is
unchanged (both cards cost 3).
What it costs: {W}{U}{U} on 13 blue lands, the same colour strain as Counterspell, Mindbreak Trap and Oracle, plus one {W};
Augustin's discount does not help it (no generic mana in the cost); it stops spells only,
not abilities (Kinnan's and Urza's activations, Walking Ballista); and it is dead against a table with nothing worth
countering, where Venat at least drew a card on a legendary spell. The Hydaelyn side ({7}: exile a nonland permanent, then
an indestructible 4/4) leaves with it.
What the numbers said before: the only measurement of Venat is 10/09, Secure the Wastes => Venat, +0.1 +-0.8 (adaptive,
paired, 1,000 games per tier), neutral, on an older list; the changelog's tuning notes already named it one of the weakest
slots (a creature, so no trigger, and the legendary spells are few); in the 11/07 analyze run it was in neither the best
nor the worst win-rate list. Nothing has measured Absorb in this deck. Absorb is modelled (cards.json DSL: counter any
spell, then gain 3; `engine/stack.rs`; one of the engine's counterspell-AI cards in `data/audit.toml`), and
`cardgen audit --mine` shows the list all Full. `data/plans/yshtola.toml` has no counterspell group, so no plan change;
`data/printings/yshtola.txt` loses Venat's line and has no Absorb printing yet (JD's to supply).
Check in the next run: a paired `--swap` (Venat out, Absorb in) against t5 with the explain breakdown: Absorb's drawn,
cast and stuck-in-hand numbers ({U}{U} is the risk), whether the share of games lost to combos (44%) falls, how often the
AI casts it into a combo spell rather than an early play, and the game-shape lines (survival, win round, mana at turns 4
and 8). A neutral result would only say the old slot was neutral too; a drop in stuck-in-hand cards or in combo losses is
what would justify it.

## 2026-11-15: Talisman of Progress and Talisman of Hierarchy => Mox Diamond and Chrome Mox (JD's choice, not simulated)
Mox Diamond and Chrome Mox ({0} artifacts, both Game Changers: the list goes from 14 to 16, still Bracket 4) take the
two Talismans' slots. Why: both are one turn faster than a two-mana rock and cost no life (the Talismans cost 1 life per
coloured tap); a Mox on turn 1 puts Smothering Tithe, Rhystic Study or Y'shtola a turn earlier, and the average mana value
of the nonland cards falls from about 2.9 to 2.8. What it costs: Mox Diamond needs a land card to discard (the list has
37 lands, so a spare one is usually there; with none in hand it goes to the graveyard), and Chrome Mox exiles a nonland,
nonartifact card from hand and makes only that card's colours, so it is a card down and its fixing depends on the
imprint (Mox Diamond makes any colour). Each Talisman made two of the deck's three colours without a card cost.
Neither Mox triggers Y'shtola (mana value 0), as neither Talisman did. Both are artifacts, so they die to the same
artifact hate and to Farewell's artifacts mode. Not measured: the 10/19 test of a rock for a rock (The Mind Stone for
Talisman of Progress) found no clear effect, because her losses are fast combos (Urza, Zur, Winota) that end T5 games
around round 9.4 and the deck often skips its rocks for better plays; the Moxen are faster than those, but that is a
guess. Mox Diamond (`rock=1:A moxd`) and Chrome Mox (`chromemox`) are modelled in the engine (`carddb.txt`), so `--swap`
against t5 can measure it. The plan file's early-rocks priority group (`data/plans/yshtola.toml`) now lists only Sol Ring,
Arcane Signet and Coalition Relic: a card named in a plan group gets that group's priority before the pool AI's Mox checks
(Mox Diamond needs a land to discard, Chrome Mox a spare coloured card; `ai/pool.rs`), so naming the Moxen there would have
let the AI cast them with nothing to pay. `data/printings/yshtola.txt`
loses the Talisman lines and has no printings yet for the Moxen (JD's to supply).

## 2026-11-21: Grand Arbiter Augustin IV => The Theorist, Jace Beleren; Urza's Cave => Expedition Map (JD's choice, not simulated)
Two swaps, both JD's, no sim run (JD asked for none). **The Theorist, Jace Beleren** ({2}{U}{U} planeswalker, loyalty 3;
commander-legal, mono-blue, not a Game Changer) takes Augustin's slot. Why: Augustin is a creature, so it never triggered
her; the Theorist is a noncreature spell of mana value 4 (it does), counts two blue devotion for Oracle (Augustin one), and
draws a card in each opponent's draw step (about three extra cards a round at a four-player table, each one a draw for
Sheoldred). An Opus consult picked Augustin from JD's pool of eligible cuts; the runner-up was Coalition Relic, which would
have removed a blue source. What it costs: Augustin's discount for white and blue spells and its {1} tax on the opponents'
spells (a small slowdown for the Kinnan and Urza combos); the Game Changer count goes 16 -> 15. Risks (Opus): the draw is
mandatory, so a stalled Doomsday pile loses its top card in every opponent's draw step while the Theorist is out; Yawgmoth's
Orcish Bowmasters pings on every extra draw; Narset (Urza, Zur) lets only one draw a turn; opponents attack planeswalkers in
the sim; the list now has four {U}{U} spells besides Force of Will against 15 blue sources. **Expedition Map** ({1}; {2},
{T}, sacrifice: a land card into your hand) takes Urza's Cave's slot (lands 37 -> 36). Opus called this close to a sidegrade:
the Cave already fetched a land, the Map was in the list 11/04-11/14 with no clear effect (Coffers package above), and its
win rate when cast was 34.9% in the 11/07 analysis (bottom ten); JD wanted it anyway, and chose the Cave over Caves of
Koilos (which would have cost a white and a black source). Not measured: no list with either card has run.

**Model (new engine code).** The Theorist was Unmodeled and is now Full in the card data: the draw trigger is
`impls/common.rs theorist_draws`, called by the draw step (`engine/turn.rs`) after the active player's own draw and the
state-based actions, through the trigger window, and not at all when the step is skipped (Necropotence); the abilities are
a walker (AI and practice mode). An adversarial review (Sol, 44 tests in `tests/theorist_adversarial.rs`) found 11 defects;
eight were in the card's code and were fixed (a skipped step still triggered, the draw came before the active player's,
no response window, no check for lost abilities, no protection-from-blue check on the -2, the AI chose -2 targets during
resolution, artifact lands and creature lands were ignored). The other three are engine-wide limits that every walker or
multi-target ability shares, **accepted by JD on 2026-11-21** as named limits (tests kept `#[ignore = "ENGINE LIMIT: ..."]`): unpaid ward counters only the
warded target, not the whole -2; the loyalty once-per-turn stamp does not reset on consecutive extra turns and practice
mode ignores Oath of Teferi; the walker options have no `neutered` check. To check in the first run: the Theorist's cast
and stuck-in-hand numbers, how often it survives to a second activation, and the Map's cracks (the breakdown counts casts
only, so cracks need a trace). The baseline for any run is a fresh one on this build.

## 2026-11-15: Urborg Syphon-Mage => Mindbreak Trap (JD's choice, not simulated)
Mindbreak Trap ({2}{U}{U} instant, model status Full: `t6_counters.rs` `mindbreak_free`; not a Game Changer) takes
Urborg Syphon-Mage's slot. Why: it is free once any opponent has cast three spells this turn, it exiles any number of
target spells (a whole combo chain, or a counter aimed at your spell), and exile answers spells that can't be countered
(Hullbreaker Horror in the Kinnan and Urza lists, Dragons under Cavern of Souls in Miirym's). It is a spell of mana value
4, so it triggers Y'shtola even when cast for free. It was aimed at the deck's largest documented loss: combos took about
44% of the t5 games in the 11/07 analyze run (Urza 13.2%, Kinnan 12.9%, Zur 9.4%, Yawgmoth 8.4%). What it costs: Urborg
Syphon-Mage ({2}{B}, discard a card: each other player loses 2 and you gain the total), one of the lowest win rates when
cast in that run (31.4%, +5.1; cast in 11% of games). Limits: it stops spells only, not abilities (Kinnan's and Urza's
activations, Walking Ballista); hard-cast it needs {U}{U} (about 58% by the 10th card with 18 blue sources), and four blue
sources at once on your own turn alongside Oracle. An Opus consult preferred cutting Disdainful Stroke for it (Stroke hits
only mana value 4 or more and the t5 combo pieces are cheaper); JD chose Syphon-Mage, so Stroke stays and the list now has
four counters. Not measured: no list with Trap has run, and the Option B package below is not simulated either. Opus
advised against adding Gifts Ungiven here (nothing in the list returns a card from the graveyard, and no pile that names
both Doomsday and Demonic Consultation is safe).

## 2026-11-14: Doomsday / Thassa's Oracle package, Option B (12 swaps; NOT SIMULATED)
Branch `yshtola-doomsday` (worktree `commander-sim-wt-yshtola-doomsday`), on the Cali branch tip `50196edd` after pulling
the other session's three newer commits (Ur-Dragon cards, phasing). **Status:** JD chose Option B on 2026-11-14 (first A,
then "actually can we do B"). The deck document, `decks.json`, the plan file, `decklists/CHANGELOG.md` and
`tests/fixtures/my_decks_parsed.json` describe the B list. **No simulation of the B list exists**: JD asked for no sims.
The only package run was 200 games of an earlier 18-card version (below). Nothing here is a measured effect of B.

**The swap (B).** OUT (12): Conjurer's Closet, Teleportation Circle, Displacer Kitten, Restoration Angel, Sphere of
Safety, Idyllic Tutor, Teferi's Protection, Bound in Silence, Expedition Map, Champion's Helm, Mystic Remora, Tezzeret's
Gambit. IN (12): Thassa's Oracle, Demonic Consultation, Doomsday, Silence, Grand Abolisher, Valley Floodcaller, Borne Upon
a Wind, Teferi, Time Raveler, Gitaxian Probe, Brainstorm, Preordain, Dark Ritual. It is the 18-card package minus Tainted
Pact, Orim's Chant, Cabal Ritual and Mockingbird (Opus review) and minus Lion's Eye Diamond and Ponder (Opus's optional
second step), with Ringsight, Loran, Arrest, Propaganda, Fatal Push, Read the Bones, Path to Exile, Go for the Throat
and Esper Sentinel (JD's protected card) kept. Checks made before proposing it: all 12 adds are commander-legal and fit
W/U/B (Scryfall cache); all 12 cuts were in the current list; Esper Sentinel is not cut. Counts after the swap: 15
creatures, 9 enchantments, 6 artifacts, 16 instants, 15 sorceries, 1 planeswalker, 37 lands (+ the commander = 100);
Game Changers 14 (Teferi's Protection out, Thassa's Oracle in); trigger spells (noncreature, mana value 3 or more) 33
-> 27; Zur's enchantment-search targets 6 -> 4 (Arrest, Necropotence, Propaganda, Rhystic Study). Tainted Pact is not in
the list on purpose: it stops at the second card with the same name and the list has 19 basics, so with Oracle it can't
exile the library (checked against its card text).

**Follow-up (JD, 2026-11-14): Voice of Victory in, Torment of Hailfire out** (13 swaps from the pre-package list). Voice of
Victory ({1}{W}, "your opponents can't cast spells during your turn", Mobilize 2) is a second, cheaper lock creature next to
Grand Abolisher. Unlike Abolisher it stops spells only, not abilities, and neither helps an end-step Doomsday on an
opponent's turn (only Silence and Teferi, Time Raveler do). JD chose the cut from three candidates I offered (Disdainful
Stroke, Fatal Push, Torment of Hailfire): Torment was cast in about 39% of the games it was drawn and stuck in hand the rest
(round 9.2, stuck in hand 61%: the 2026-11-04 Coffers package run below), though it won 62% of the games it was cast in (the
11/07 analysis below; both from pre-package lists, and a win rate when cast flatters a late card). Checks: Voice is
commander-legal and white, and
was not in the list; Torment was. Counts after the follow-up: 16 creatures, 9 enchantments, 6 artifacts, 16 instants, 14
sorceries, 1 planeswalker, 37 lands; trigger spells 26; Game Changers 14 (the counts in the paragraph above are the
12-swap list). The deck document's old Line 4 (Torment) was removed, so its Lines 5, 6 and 7 are now Lines 4, 5 and 6 (the
Oracle + Consultation line is Line 5 and Doomsday is Line 6). Not simulated, like the rest of this section; no run of any
list includes Voice of Victory.

**Follow-up (JD, 2026-11-14): Sphere of Safety in, Gitaxian Probe out** (JD's choice: a second pillowfort back against
creature decks; Miirym's combat is her top killer in the earlier runs). Sphere of Safety ({4}{W}, an enchantment, white,
commander-legal; creatures can't attack you unless their controller pays {X} for each, X being the number of enchantments
you control) was in the list before the package. In the earlier analysis it was cast in about 34% of the games it was
drawn in, and it stuck in hand in 67% of the 200-game test run, a small sample. What it costs: Gitaxian Probe was the free
cantrip inside the Doomsday piles, so the three piles were re-derived and walked through by hand without it, using only
Brainstorm, Preordain and Silence (Borne Upon a Wind starts Pile C). At devotion 2 the piles now need two cantrips (Piles A
and B cost five mana with four blue, Pile C ten mana with five blue), so a single counter on a cantrip stalls them; at
devotion 3 (Y'shtola out, or Teferi or Floodcaller as the enabler) Piles A and B need only one cantrip (Pile C still needs
Borne Upon a Wind and Preordain). Pile C needs Borne Upon a Wind (or Preordain) in hand to start; Probe was its other starter. Counts
after the swap: 16 creatures, 10 enchantments, 6 artifacts, 16 instants, 13 sorceries, 1 planeswalker, 37 lands; trigger
spells 27 (Sphere is mana value 5); Game Changers 14. Not simulated, like the rest of this section.

**Baseline (measured; run output `bv377p7og`):** **26.6%** (24.8-28.6) at t5, 2,000 games, seed 500000, loose, look-ahead,
on the Cali base `c1da6adb` before the swap: the same as the 11/07 baselines (27.4% in 10,000 games, 26.5%). Plan online
62.4%; wins by damage 72%, combo 26%. Eliminated by: Miirym 18.8%, Kinnan 16.4%, Urza 13.9%, Zur 12.6%, Yawgmoth 11.6%.
**A later run of the B list needs a fresh baseline on `50196edd`:** the three commits since `c1da6adb` changed engine code
(`turn.rs`, `control.rs`, `planner.rs`, `dsl/mod.rs`) and the Urza and Yawgmoth opponent code (`t45_urza_yawgmoth.rs`), so
this number is not a valid pair for it.

**The earlier 18-card version (measured, 200 games; run output `swap18-quick3.txt`; NOT the B list).** That version also
cut Propaganda, Arrest, Loran, Ringsight, Fatal Push and Read the Bones and added Tainted Pact, Orim's Chant, Cabal
Ritual, Mockingbird, Lion's Eye Diamond and Ponder, with an earlier plan file (a Silence stand-in at priority 95).
- The preflight stopped its first run: Doomsday was cast in 2% of the games it was drawn, Silence, Orim's Chant and Lion's
  Eye Diamond in 0%, Oracle 13.5%. After plan-file entries a 200-game dry run cast Doomsday 7.4%, Silence 6.5%, Orim's Chant
  6.7%, Lion's Eye Diamond 15%, Oracle 19%. The paired result was 25.0% -> 29.5%, noise +-8.1: no clear effect. Spot removal
  cast fell 1.36 -> 0.93 per game (outside its noise band).
- A full 2,000-game paired run of that version was started and stopped at JD's request; it produced no result.

**Engine facts that matter here (read from the code, not tested).**
- Plan files are **embedded at build time** (`build.rs`): an edit to `data/plans/yshtola.toml` does nothing until the
  simulator is rebuilt (the first quick run after an edit was identical to the run before it for that reason).
- The plan slots (`[sets]`, `[mulligan]`, `[[go_now]]`) and `t6_table::protect_before_go` (casts the lock before the combo)
  are read only for Tier 6 keys (`impls/t6_plans.rs`, `T6_KEYS`); `yshtola` is Tier 5. Doomsday's pile play and the combo
  catalogue are not Tier 6-gated; `go_risk_covered` does apply to a Tier 5 player's Doomsday.
- **So at Tier 5 the AI does not cast Silence as a lock before the Oracle line.** A stand-in priority rule was tried and
  removed: the line is offered at utility 14 or more and Silence at priority 66 is worth about 6.6, so the AI goes first
  without it, and the rule fires on turns the line cannot go, spending the only Silence. Doomsday piles still cast Silence
  inside the pile code. **Consequence for any future swap run:** the preflight will flag Silence (priority 0, never cast).
- At Tier 5 the Oracle line (Line 5 in the document after the follow-up renumber) also skips the one-spell-lock check (`combos.rs casts_castable`): against Zur's Rule of Law or Eidolon of
  Rhetoric the AI may cast Oracle and then be unable to cast Consultation. UNVERIFIED in play.
- Her end-step draw ("a player lost 4 or more life") is mandatory, as are Notion Thief's and Venat's draws; the pile builder
  counts only Esper Sentinel's forced draw. A Doomsday cast in her own main phase (Pile C) loses at least 4 life from 5 life
  up, so her draw takes a card off the pile at that end step. Unverified: whether the pile search uses Necropotence
  activations, and whether the AI declines Rhystic Study and Consecrated Sphinx draws while a pile is set.

**Plan file as it stands (`tests/plans.rs` passes).** The two combo lines and a held Doomsday (priority 0);
wishes: Consultation (88, Oracle held) and the Oracle (88, Consultation held) ahead of Doomsday (78, with a flash enabler
out or a Pile C starter in hand) and Sanguine Bond (85); a pile-starter wish (Borne Upon a Wind; it also named Gitaxian Probe until the swap below) at 58; an
explicit Silence priority of 0. The rules for the cut cards (Restoration Angel, Teferi's Protection, Mystic Remora, Idyllic
Tutor, Champion's Helm) were removed. Demonic Consultation has no priority rule: from the code it is not cast outside the
combo line (no tags and no DSL in `cards.json`), unverified on the derived-values arm.

**Opus reviews (2026-11-14; a reading of the files and code, not tests).** First review (18-card version): Tainted Pact
can't win here, Ringsight was the wrong cut, the cuts removed both pillowforts, the removal Auras, Loran and all her
protection, and the stand-in Silence rule outranked casting Y'shtola. Final review (A version): the list, counts and pile
tables were consistent; the 19 rows of its error table were applied in the B rewrite (hand 9, a countered Oracle can still
go into a Doomsday pile from the graveyard, which t5 lists run which counters, her end-step draw threshold, and others);
the plan-file points above. Delta review (B version): the list, counts, Piles A, B and C and the plan file were right; it
found smaller text errors (Disdainful Stroke's reach, hand 9's answer to Silence and its shock-land life, a stale
Expedition Map, Pile C's "spare card" not being needed, Muddle the Mixture missing from the counter census, "free counter"
for any counter, the baseline base) and a gap in the pile-starter wish, all fixed. Unverified: the doc advises fetching
Silence with both halves in hand, which the plan no longer does for the AI.

**Side effect on the opponents (not a decision anyone made).** `cardgen export` keeps a hand-tagged card's model only
while one of JD's decks plays it. Displacer Kitten left Y'shtola's list, the only one of JD's lists that played it, so its
`cards.json` entry now uses the pool model (the `vkitten` tag is gone; `source` is "pool"). Displacer Kitten is in the t5
Urza list and the t2 Brago list. The blink itself is name-based code and is unchanged; what changes (from the code) is the
Kitten's importance in the counter window (4.0 before) and its value on the battlefield (5.0 before, about 2.0 now). The
effect is probably small and unmeasured, and it shifts the opponent model behind the stored baselines of every deck that
meets Urza or Brago (yshtola, thanos, aragorn at t5; cloud at t2). Options: accept it with this note, or change the export
rule to keep hand-tagged entries that a pool deck plays (an export-wide change: JD's call).

**Not done:** `data/printings/yshtola.txt` (JD's printings: it still lists Kitten and Teferi's Protection and has no entry
for the new cards) and `data/spellbook/yshtola.json` (needs a network fetch; it doesn't list Oracle + Consultation yet; re-fetched 2026-11-23, it now does).

## Baseline 2026-11-07 (after package Y; main 7c622f5f; look-ahead, loose, 10,000 games, seed 500000)
**27.4%** (26.5-28.3) at t5, against 26.5% (24.6-28.5) in Baseline B (2,000 games): +0.9, inside the noise. Package Y
(Counterspell, Ringsight, Palantír, Mistrise Village) shows no measurable change. Plan online 61.0% of games; wins by damage
72%, combo 27%. Who eliminates her: Miirym 17.9%, Kinnan 17.4%, Urza 15.0%, Zur 11.3%, Yawgmoth 10.9% (the Kinnan and Urza
combos still about a third of her deaths). Analyze run: `audit/baseline-2026-11-07/yshtola-analyze.txt`. Raw:
`audit/baseline-2026-11-07/yshtola.txt`.

**Analyze (2,000 games, t5).** Eliminated in 73.7% of games (round 8.4): combos Urza 13.2%, Kinnan 12.9%, Zur 9.4%, Yawgmoth
8.4% (about 44%), then Miirym combat 9.9% and triggers 6.6%. Highest win rates when cast are the finishers and the combo:
Debt to the Deathless 68% (+42), Exquisite Blood 62%, Torment of Hailfire 62%, Exsanguinate 59%, Sanguine Bond 53% (cast in
36% of games). Lowest (cheap cards cast early, the telling ones): Teferi's Protection (-0.2), Esper Sentinel, Fatal Push and
Mystic Remora (+4.6), Arrest +7.3. Stuck in hand most: the blink engines Conjurer's Closet (cast 19% of the games drawn) and
Teleportation Circle (22%), then Sphere of Safety 34% and Displacer Kitten 35%. The package-Y cards (Counterspell, Ringsight,
Palantír, Mistrise Village) are in neither list: neither notably good nor bad.

## 2026-11-07: card-pile package Y, adopted untested
Memory Trap => Counterspell, Dark Petition => Ringsight, Ill-Gotten Inheritance => Palantír of Orthanc, Island =>
Mistrise Village (JD's spare-card pile, `audit/jd-packages/2026-11-06-card-pile/`). **Not measured:** JD stopped the runs
and adopted it. Why: she loses about 35% of t5 games to the Kinnan and Urza combos, and Counterspell answers a combo
piece at instant speed; Ringsight ({1}{U}{B}, a tutor for a white, blue or black card with her out) costs 3 where Dark
Petition cost 5, and both trigger her; Palantír (mana value 3, triggers her) drains or draws every end step where
Ill-Gotten Inheritance drained 1 a turn; Mistrise Village makes the next spell uncounterable.

**A correction to the reason given when it was chosen:** Memory Trap was described as a conditional counterspell. It is
a {2}{W} enchantment that exiles a nonland permanent an opponent controls until it leaves: removal that triggers her
(mana value 3) and that Zur can fetch. The swap trades it for a counterspell that doesn't trigger her (mana value 2).
JD kept it (2026-11-07) over cutting Arrest instead. **Engine fix with it:** Ringsight used to search for a blue, black
or red card (Sauron's colours, `tut=ubr`); it now finds a nonland card that shares a colour with a legendary creature
you control, the Ring-bearer included (`tut=legcol`, `ai::decks::tut_kind`, `tests/cards_ringsight.rs`). The preflight
cast Ringsight in 82% of the games it was drawn and Counterspell in 57% (t5, 200 games, before the fix). To check in the
next baseline: the combo losses (Kinnan, Urza) and Palantír's cast rate (44% in the preflight).

## Baseline B 2026-11-06 (after #278-#284; main 63d783b5; look-ahead, loose, 2,000 games per tier, seed 500000)
**26.5%** (24.6-28.5) at t5, against 26.3% in the morning: no change. The Tier 5 table changes of #284 (no wasted protection
holds; Garruk's Uprising and Welding Jar cast) and #281 (Urza's Whir) don't move her number. Raw:
`audit/baseline-2026-11-06-b/yshtola.txt`.

## Baseline 2026-11-06 (after the rewire, Doomsday and Phase C engine work)
Y'shtola, t5, 2,000 games, seed 500000, loose, look-ahead, integration build ef343cc1: **26.2%** (95% CI 24.3-28.2) over t5
(1,994 games counted: 6 hit an internal simulator error, see below). Same command on main 1e027077 (2026-11-05, old build): **25.8%**
(23.9-27.8). Change +0.4: noise (intervals overlap almost completely, under 2 points). Plan online 65.4% (old 63.5%); wins by
damage 66% / combo 34% (old 70 / 30).
Engine changes that can touch this deck: C2 Loran of the Third Path (fires less, picks better targets); C3 attack taxes no
longer idle attackers (it plays Propaganda and Sphere of Safety; Toxic Deluge is not affected) and opponents cast their commander on time
(+1.5), which changes the t5 opponents' play. C1/C1b and the rewire do not touch it. Nothing here is a measured effect of any
one change: the net is noise.
**Re-run after the crash fixes:** the 6 skipped games were the same look-ahead `cast_counter` panic as Thanos's (latent on main,
fixed on #274); a first re-run then skipped one game on an overflow of the "life gained this turn" counter (Exquisite Blood off a
doubled Dragon army, seed 501953), also fixed on #274. Final re-run (integration 23ecd983): **26.3%** (95% CI 24.4-28.3) over t5, all
2,000 games, no errors; plan online 65.5%; damage 66% / combo 33%. Against the old build 25.8%: noise. This is the number of record.
Raw: `audit/baseline-2026-11-06/yshtola-rerun.txt`; first run `audit/baseline-2026-11-06/yshtola.txt` and `yshtola-2026-11-05-oldbuild.txt`.

## Current state (2026-11-04)
Look-ahead, loose, current list (Cabal Coffers in for a Plains): **T5 24.9%** (95% CI 23.0-26.8), 2,000 games.
Her losses are still fast combo (Kinnan, Urza, Zur) and Miirym combat, about 49% of games.
**Applied 11/04 (JD):** the Coffers package with Urza's Cave (Demonic Tutor, Exquisite Blood, Torment of Hailfire,
Expedition Map, Urza's Cave for Deep Analysis, Prison Sentence, Chromatic Lantern, Static Net, an Island). Measured
26.1% vs T5 (+1.3, noise ±2.4) with the board-aware land choice before the Cave/Map utility change; the list as
applied (that utility change, Torment in the plan file's X-drain wish) is not measured yet.
**Revised 11/04 (JD's physical list; not simulated):** Demonic Tutor took Bribery's slot and Profane Tutor took Hostage
Taker's, so the package's Deep Analysis slot goes to **Loran of the Third Path** instead: a blink target (Displacer
Kitten, Restoration Angel, Conjurer's Closet, Teleportation Circle) whose enter trigger destroys an artifact or
enchantment each time, and whose tap draws a card (the opponent draws too). Seven tutors; no theft left. Measure over T5.

## Weaknesses (by evidence)
- **Lowest win rate when cast** (adaptive card report, `audit/yshtola/2026-10-17-adnaus/K0-analyze-adaptive.txt`;
  nearly every card is above the deck average): Mystic Remora (39%), Lightning Greaves, Luminous Bonds, Prison
  Sentence, Esper Sentinel, Curiosity, Chromatic Lantern, Thought Vessel (43–44%).

## What was tried, and why it did or didn't work
### 2026-11-04: Cabal Coffers package (T5, loose, look-ahead, 2,000 games per list, seed 500000; branch yshtola-coffers)
**Baseline, Cabal Coffers in for a Plains** (JD's physical list, 11/04): **24.9% vs T5** (95% CI 23.0-26.8). Not a
measured Coffers effect: the 11/01 number (22.4%) is an older build. Raw: `2026-11-04-coffers/baseline-t5.txt`.

**Package**: Deep Analysis => Demonic Tutor, Prison Sentence => Exquisite Blood, Chromatic Lantern => Torment of
Hailfire, Static Net => Expedition Map, Island => Tolaria West (Diabolic Tutor kept: six tutors).
Result: **24.9% -> 24.7%, -0.1 (noise ±2.4): NO CLEAR EFFECT.** Raw: `2026-11-04-coffers/package-t5.txt`.
Why:
- **It changes how she wins, not how often.** 29% of the package's wins are the Sanguine Bond + Exquisite Blood combo
  (current list: 99% damage). Exquisite Blood is cast in 68% of the games it is drawn (round 7.6) and wins 61% of those,
  the best card in the package; it takes wins the drain plan would have got anyway.
- **Her losses don't move**: Urza, Kinnan and Zur combo plus Miirym combat (about 49% of games both ways). Games end at
  about 9 rounds, and the big-mana half arrives too late: Torment of Hailfire is cast in only 39% of the games it is
  drawn, at round 9.2 (stuck in hand 61%).
- **The cuts cost tempo.** Games end 0.3 rounds sooner (±0.2) and the plan comes online 3.4 points less often (±3.0).
  She draws 1.5 cards fewer per game (Deep Analysis out), casts 0.8 spells fewer and 0.2 spot removal fewer (Prison
  Sentence, Static Net out), and has 0.16 less mana on turn 6 (Lantern out; Tolaria West enters tapped). Damage to
  opponents falls 4.2 a game.
- Engine fix for the test: Torment of Hailfire's cast option (`impls/threats.rs` torment_options) was Sauron/Tergrid-only,
  so her AI never cast it (preflight: 0 of 48). Y'shtola now gets it, with a test. Tutors don't name it in her plan file
  (a plan file may name only cards in the deck).
Next candidate (not run): Exquisite Blood alone for Prison Sentence, the one piece that pulled its weight.

**Same package, Urza's Cave for the Island instead of Tolaria West**: **24.9% -> 26.1%, +1.3 (noise ±2.4): NO CLEAR
EFFECT.** Raw: `2026-11-04-coffers/package-cave-t5.txt`. Same seeds, so the 1.4 points over the Tolaria West version is
also noise. Same shape: 27% of wins by the Blood combo (Blood cast 68%, wins 63% when cast), the same losses (Kinnan,
Urza, Zur, Miirym), plan online -4.5 (±3.0), 1.35 cards fewer drawn, damage -3.4 a game. **Caveat: the Cave never
fetched anything for her.** Its AI (`impls/karn_engines.rs`) cracks it only for a Tron piece, Mishra's Workshop or
Ancient Tomb, so here it was a colourless land; Urborg and Coffers aren't on its want list. A real Cave test needs
that AI rule first.

**Same Cave package, board-aware land choice** (`karn_engines.rs` board_land_want: Coffers/Urborg as a pair, Serra's
Sanctum, a colour the hand needs; tests `tests/land_search_board.rs`): **24.9% -> 26.1%, +1.3 (noise ±2.4): NO CLEAR
EFFECT**, 12 threads (shared CPU). Raw: `2026-11-04-coffers/package-cave-board-t5.txt`. 60 traced games
(`2026-11-04-coffers/traces/`) show why it matches the run before: the Cave was played in 10 games and its option
offered 9 times (e.g. Coffers out, no Urborg, 10 lands), but **it was never chosen**: its utility is a flat
1.2 + want/6 (about 3), below Champion's Helm, Sphere of Safety or holding mana, not scaled to the mana Urborg would
add. Expedition Map now cracks (twice in the 60: Cabal Coffers, Flooded Strand); before this fix it never did, so the
two earlier package runs had a dead Map too. Open: the Cave's and the Map's option utility should grow with the mana
the fetched land adds.

**Follow-up, not measured:** `search_utility` (`karn_engines.rs`) now scores a board-based search at 1.2 + want/2
(Urborg beside Coffers on 8 lands: about 6), Tron/Workshop wants unchanged at 1.2 + want/6; tests in
`tests/land_search_board.rs`, full suite passed. The package rerun was stopped at JD's request (11/04), so there is no
number yet for a Cave that actually cracks.

### 2026-10-19: Curiosity => Orcish Bowmasters, confirmed (T5, loose, 2,000 games per list, look-ahead)
Result: **31.6% -> 34.1%, +2.5 (noise ±2.3): BETTER**, the first swap to clear the noise for her. Raw:
`2026-10-19-bowmasters/full-curiosity-t5.txt`.
Why: Bowmasters is cast in 81% of the games it is drawn (Curiosity 68%, stuck in hand 32%), around round 5.9, and
wins 48% of the games it is cast in. Damage to opponents +2.3 a game and the plan online +2.4 points (60.3 -> 62.7%),
same game length (9.4 rounds). The losses to Urza, Zur and Winota combo fall slightly (all within noise): the ping
and the Orc Army pick off the fast decks' mana creatures and small pieces. Recommended: buy a second Bowmasters for her.

### 2026-10-19: Orcish Bowmasters for three cards (quick read: T5, loose only, 700 games per list, look-ahead)
| Cut (=> Orcish Bowmasters) | T5 (noise ±4.0-4.1) |
|---|---|
| Curiosity | +3.1 |
| Mystic Remora | +2.0 |
| Lightning Greaves | +3.3 |
**Leans good: about +2.8, every cut positive, none beyond noise alone.** The three share one baseline (30.9%), so they
are not independent; a full run of one cut is needed to confirm. Raw: `2026-10-19-bowmasters/<cut>-t5.txt`.
Why: Bowmasters is cast in 79-86% of the games it is drawn, early (round 5.4-5.8), and wins 45-56% of the games it
is cast in (the deck averages about 31%). Damage to opponents rises 1.8-3.6 a game in all three, and the plan comes
online slightly more often (+0.9 to +3.3). The ping kills T5's mana creatures and 1-toughness pieces, and the fast
tables' own extra draws feed it. JD would buy a second copy (the Sauron copy stays).

### 2026-10-19: Talisman of Progress => The Mind Stone (T5, 2,000 games per list per profile, look-ahead)
Result: **+2.5 conservative (±2.0) / -1.0 loose (±1.9): about +0.75 averaged, no clear effect** (the tool's
"leans better" rests on one profile, and the other goes the other way). Raw: `2026-10-19-mind-stone/t5.txt`.
Why: a rock for a rock. Mana on turns 4 and 6 and the plan (online in about 61% of games, turn 6.9) are unchanged.
The Mind Stone is cast in 67-73% of the games it is drawn (Talisman 67-68%), around round 5.2-5.5, and stuck in hand
in 27-33%, as much as the Talisman: her rocks are often skipped for better plays, so it is not a Mind Stone priority
problem. Her losses are the same fast combo either way (Urza 15%, Zur 13%, Winota 11-20%), which ends T5 games at
about 9.4 rounds, before the {5}{W} harness matters. The 10/18 quick test (Chromatic Lantern out, loose only) was
also noise (+0.4). Verdict: not an upgrade for her.

### AI fix: cards her rules don't name (2026-10-17, `9b7e19a`)
`yshtola_prio` returned 0 (never cast by priority) for any card it didn't name: a swapped-in Ad Nauseam was cast in
0.5% of the games it was drawn, and **Dakkon, Shadow Slayer in her list was never cast**. Unnamed cards now get the
pool AI's generic priority (`yshtola_named_prio` holds her rules). T5 baseline 31.1 → 31.6% (adaptive, noise).

### Ad Nauseam (2026-10-17 night; branch `adnaus`; JD: keep Esper Sentinel)
The shared Ad Nauseam AI and the look-ahead's hold-mana credit were fixed during these tests (audit/kefka/FINDINGS.md).
| Swap (Ad Nauseam in for), T5 | Result | Why |
|---|---|---|
| Mystic Remora / Curiosity (adaptive, new AI) | -0.9 / +1.0 | |
| Curiosity (look-ahead, before the hold credit) | -2.0 (±3.2) | The search tapped out: Ad Nauseam cast in 5.7% of the games it was drawn. |
| **Curiosity (look-ahead, hold credit)** | **-0.2 (±3.2)** | Cast 15%, wins 56% when cast vs Curiosity's 39% (cast 80%). |
| **Thought Vessel (look-ahead, hold credit)** | **-1.4 (±3.2)** | Cast 17%, wins 58% when cast; Thought Vessel cast 70% (no maximum hand size, useful with Necropotence and Citadel). |
| **Mystic Remora (look-ahead, hold credit)** | **-1.9 (±3.1)** | Cast 14%, wins 54% when cast; Mystic Remora cast 75%. |
**Reading it:** Ad Nauseam wins when it's cast but is rarely cast: T5 games are short (about 8.5 rounds) and the deck
pays life already (Necropotence, Bolas's Citadel), so the safe window seldom opens. Not a fit; Sheoldred is its home.

## 2026-10-18: band calibration (proposed swaps, NOT applied; see audit/bands/REPORT.md)
Band 1 (T5). Proposed: +Force of Will, +Farewell, +Teferi's Protection (all from Seph/Atraxa) for Dakkon, Thought Vessel, Bolas's Citadel (to Seph FS). 9 Game Changers. Final: **24.3% vs T5** (current list 23.2%). Why: Force of Will is cast 76% of the games drawn and answers T5 combo; Farewell and Teferi's Protection are mostly held (stuck 68-75%) but win 45-60% when cast. Citadel was cast in only 25% of the games Y'shtola drew it, so it was the cheapest card to give up.

## 2026-10-18: AI audit phase 2 (branch audit-ysh; problems 3-6 of audit/ai-audit/yshtola.md)
Measured at T5 (her tier), adaptive AI, 4,000 games per list, conservative/loose win rate: before 32.3 / 31.6; after
31.7-32.0 / 31.9-32.0. No change beyond noise (one run's SE is 0.7); the fixes make the plays right, they do not move the
win rate. Weakness this leaves: she still loses 35% of games to Kinnan/Urza combos, which spot removal cannot reach in time.
- **Removal and her trigger.** `removal_adjust` (impls/yshtola.rs): a noncreature spell of mana value 3+ with her out
  (Anguished Unmaking, Generous Gift, Your Fate Ends Here) earns `trigger_bonus / 4` and needs a target worth 1; every
  other removal needs a target worth 3 (before: Path on Ornithopter, Push on Esper Sentinel). `trigger_bonus` adds 3 when
  an opponent already lost 2-3 life this turn (one more trigger makes four and she draws). The same check applies to the
  attack-response removal. The unplayed 4-player test: choosing the single best target instead of sampling cost -1.3
  points (31.0 vs 32.3), so the sample stays, restricted to targets at least half as valuable as the best.
- **Farewell / Austere Command.** The wipe also earns the trigger bonus. With Teferi's Protection in hand and its
  {2}{W} left over, the wipe is valued without her own losses and cast with Teferi's in response (her permanents phase
  out, the opponents' are exiled): `teferis_first`, `cast_wipe`. The opponent-end-step version of the second trigger
  is not possible: the AI's end-of-turn window runs after the end-step triggers have been checked.
- **Mystic Remora 66 -> 40, Esper Sentinel 62 -> 46** (early casts; the two were in the bottom group by win rate when
  cast). Neutral on the win rate (31.6 / 32.0 with both); kept because Remora was being cast and sacrificed unpaid.

## 2026-10-18: AI audit re-measure (branch ai-audit; 2,000 games per profile, look-ahead)
**24.5% vs T5** (before 24.3): no change. Fixes changed play (trigger timing, removal targets, Teferi's Protection as the full card, Mystical Tutor finishers), not the win rate. Details: audit/ai-audit/yshtola.md, REPORT.md.

## 2026-10-18 evening: Chromatic Lantern => The Mind Stone (quick pass: loose only, 700 games per list, T5, look-ahead)
Result: **24.4% -> 24.9% (+0.4, ±3.3): noise.** Raw: `2026-10-18-mind-stone/mind-stone-quick.txt`.
Why: a better rock than the Lantern (cast 70% of the games drawn vs 51%; mana on turn 8 +0.13), but her losses are
fast combo (Kinnan 20%, Urza 16%, Zur 11%, Winota 11%) and the 6-mana harness (blink Hostage Taker, Massacre Wurm,
Gonti) comes too late. Open AI question: The Mind Stone sits in hand in 30% of the games it's drawn; her rules
don't name it (generic priority), so a re-test should check that first. JD decided to wait on buying it.

## 2026-10-18 evening: JD's new cards, coarse check (loose only, 500 games per list per tier, look-ahead; only big downgrades show, noise ±4-5 per tier)
**Prison Sentence => Stroke of Midnight: T5 -2.6 (noise ±4.3, no big downgrade).** Cast in 80% of draws (Prison
Sentence 67%), but it doesn't change her losses (fast combo: Kinnan, Urza, Zur, Winota). Cloud is the other home
(-1.4 there); JD has one copy. Raw: `2026-10-18-new-cards/`. JD plans to add the cards unless one is a big downgrade.

## 2026-10-31: card models (branch card-models-1; no sims until Phase 4 recalibration)

Venat, Heart of Hydaelyn // Hydaelyn: Partial -> Full. Its card code had never run (registered under a name not in the data); before this the deck played it as a 3/3 that drew on every legendary spell. No new number yet.

## 2026-11-01: recalibration stage 2 remeasure

Y'shtola, t5: **22.4%** (CI 20.6-24.2) with the hand AI on (default) (2000 games, seeds 500000-501999, loose, look-ahead, defaults, rc=0; `audit/recal/stage2-2026-11-01.md` section 4, build efe45425). `AI_HAND=0` (generic + plan file): 21.9%, level (+0.5 [-2.1, +3.1]). Stage 1, same seeds: 19.8% (hand AI on), 15.7% (off). The 10/18 AI audit number (24.5%) is from an older build and list. Fitted t5 knobs: 23.1% (level).

## 2026-11-04 (late): AI changes, not simulated (JD: no sims for this batch)

- **Zur the Enchanter**: an Aura it finds goes on a legal host (an opponent's Aura on their best creature; lock Auras such as Arrest attach properly), not always on Zur; with no legal host it isn't fetched.

The numbers above predate these changes; a re-run on the deck's own tiers would show their effect.
