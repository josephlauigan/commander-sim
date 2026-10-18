# Sauron (sauron) — AI audit, phase 1
Strategy: one big Orc Army. Sauron amasses on every opponent spell; early Army from Dreadhorde Invasion / Lazotep Reaver;
Bowmasters + Jace's Archivist wheel, Mauhur, Inexorable Tide, The Ozolith grow and protect it; Cloak / Rogue's Passage
get it through, Aggravated Assault adds combats; Exsanguinate / Comet Storm finish; Grixis control shell (6 counters,
6 spot removal, one-sided sweepers). Tutors (Diabolic Tutor, Diabolic Intent, Ringsight) are meant to be board-state picks.
Priority function: `sauron_prio` (rust/crates/sim-core/src/ai/decks.rs:86) + `threats::sauron_extra_prio`. Options:
`sauron_options` (equip, Archivist wheel). Tutors: `tutor_pick_named` "sauron" arm (decks.rs:761). Protection:
`protect` "sauron" arm (decks.rs:1304). Counters: generic `engine/stack.rs ai_respond`/`pick_counter` (cthresh 7 for
sauron, minus 0.3 per Ctr card in deck, max 2.5). Plan config: "first combat damage to a player by an Army".

Evidence runs (adaptive AI, `--jobs 2`, nice 19): `--analyze` vs t3 (300/profile, win 27.0%), and a 600-game paired
`--swap` of 18 key cards for Island vs t3 (used only for the drawn/cast/stuck columns; the +9.0 win delta is just
more lands, not a card verdict). Army power at start of turns 6/8/10: mean 1.7/2.4/2.3, median 0. Army plan online at
all 73.7%. Sauron cast 86.7% of games, median turn 6. Sauron removed 0.61 per game. Eliminated in 72% of games
(Aurelia combat 22%, Tergrid 11%, Marwyn 10%, Atraxa 8%).
`cardgen audit --deck sauron` could not run in this worktree (another deck's doc was mid-edit, "no '## Import list'
block"); model status below comes from `--cards` tags and "not modeled" lines, so Full/Approx. grades are from tags only.

## Problems (ranked by likely impact on win rate)
1. **Tutor wish list is for an old Sauron list (Diabolic Tutor, Ringsight; Diabolic Intent too).** `tutor_pick_named`
   "sauron" (decks.rs:761-815) names Sword of Feast and Famine, Rhystic Study, Deepglow Skate, Underworld Breach, Brain
   Freeze, Grapeshot, Laboratory Maniac, Lion's Eye Diamond, Jeska's Will, rituals. None are in the deck. With no Sword
   held the result is, in order: Aggravated Assault -> Counterspell -> Whispersilk Cloak -> (only then) the generic
   pick. The doc's board-state list (Ozolith before committing the Army, Not of This World for the swing, Sol Ring when
   short of mana, Barad-dur when flooded, Archivist when Bowmasters is out, Exsanguinate/Comet Storm to close) is not
   implemented. Evidence: Aggravated Assault is drawn in 58.8% of games vs 24-29% for every other single card
   (Ozolith 26.7, Not of This World 26.7, Ringsight 26.7); tutors resolve 0.58/game. Also Assault has prio 30 without a
   Sword (decks.rs:~133), i.e. the tutor takes a weak, expensive mana sink. Fix: write a Sauron tutor arm by board
   state (no Ozolith + Army growing -> Ozolith; Army huge, no evasion -> Cloak/Passage; Sol Ring early; Archivist with
   Bowmasters; Exsanguinate when lethal-ish; Not of This World when holding the swing); drop the dead names.
2. **Army is mostly absent mid-game.** Median Army power is 0 at turns 6, 8 and 10 (mean 1.7-2.4); plan online 73.7%
   but only 12% by T4 and 38% by T6. Cause is unknown from this data (Army killed, or never made because Sauron is late
   and Invasion/Reaver are drawn late); Reaver and Invasion are cast 88%/85% of the games they are drawn, so cast
   priority is fine (sauron_extra_prio 67/70 early). Needs a trace/ownership check of Army deaths and whether the Ozolith
   restores counters. Suggested: `--trace` ten games at T6 with no Army.
3. **Force of Will: used by generic logic, nothing Sauron-specific.** `pick_counter` (stack.rs:575-627) takes FoW only
   when no cheaper hard counter is castable, and pitches the lowest `card_worth` blue card (`pay_counter`
   stack.rs:672); threshold is +1.5 for a pitch cast (stack.rs:510). Verdict: sensible and matches the doc's "keep it for
   a spell that would end the game". Cast in 27.7% (conservative) / 39.5% (loose) of games where drawn, stuck 60-72%,
   win when cast 34.1% / 25.8% (noise at n~150). No evidence it is wrong; two caveats: (a) the pitch picks by
   `card_worth` with no Sauron knowledge, so it can exile Talrand/Archivist/Sephiroth when a worse blue card exists
   only if worth is lower (unverified); (b) it counters by `top.value_to(q)` importance like any counter, never
   specifically "the spell that wrecks the Army". Fix only if traces show a bad pitch.
4. **Counters sit in hand.** Counterspell cast 31.8% (cons) / 42.4% (loose) of games drawn, Undermine 27%/30%, Not of
   This World 31%/29%, Arcane Denial 37%/54%, Spell Pierce 40%/49%, Disdainful Stroke 44%/52%, FoW 28%/40%. All
   "reactive", but the profiles differ by 10+ points, so the threshold roll (`wants_counter`) governs it. Total
   counterspells cast 0.57-0.77/game with 6 counters in the deck. Unknown whether the deck also fails to keep mana up
   on its own turn (not checked). Suggested: leave unless a trace shows mana tapped out with a counter in hand.
5. **Cost of Brilliance**: `sheoldred.rs` option (generic utility 3.2, drains an opponent only when Sheoldred is out).
   It does not value giving an opponent cards while Bowmasters is out (2 pings + 2 amass per draw) and always puts the
   counter on `best_creature` (the Army by pval, fine). Minor.
6. **Jace's Archivist**: wheel gate `archivist_worth` is only Bowmasters / Sheoldred out, or hand<=1 and an opponent
   holds 4+ (decks.rs:499). Cast 49%, stuck 51%. Matches the doc ("hold until a payoff"). Mauhur is not part of the gate
   (30 counters line) and Sauron/Iron Man/Kaervek triggers from refilled opponents are ignored. Minor.

## All cards
Model grades are from `--cards` (tags + "not modeled"); "?" = not verified by the audit tool (see above).
| Card | Model | Valuation | Evidence / note |
|---|---|---|---|
| Sauron, the Dark Lord | tag `sauron`, manual | deck (85) | cast 86.7%, median T6; removed 0.61/game. Ward-sacrifice modelled? unknown |
| Aggravated Assault | `assault` | deck (30 w/o Sword) | over-fetched by tutors (problem 1); drawn 58.8%, cast 72%, win cast 35.0 (+8) |
| Arcane Denial | `ctr=any denial` | generic-ok | cast 37%/54% of draws; the drawing-back part ("denial") modelled? unknown |
| Bitter Triumph | rem destroy, lose=3 | generic-ok | removal logic in brain; no per-card data |
| Bloodchief's Thirst | rem destroy | generic-ok | kicker (4 mana, any MV) unknown |
| Bloodsoaked Insight // Sanguine Morass | land/spell tag only | generic-ok? | counted as land tag `c=BR t`; the spell half (exile-and-play) is unmodeled in tags; unknown |
| Call of the Ring | `callring` | deck (55) | not in sample |
| Chandra's Ignition | wipe=ignite | generic-ok | wipe logic; no data |
| Chaos Warp | rem tuck | generic-ok | |
| Comet Storm | hand-written (threats.rs comet_options) | deck | X/kick chosen by plan; only fires with a kill or late (X>=4, all opp faces) |
| Cost of Brilliance | hand-written (sheoldred.rs) | generic-ok | problem 5 |
| Counterspell | `ctr=any` | generic-ok | cast 31.8%/42.4% of draws; see problem 4 |
| Cyclonic Rift | wipe=rift (overload) | generic-ok | wipe_options holds instant-speed wipe to EOT |
| Diabolic Intent | `tut=any needsac` | deck (60, tutor wish list stale) | sacrifice picks `spare_creature`, never the Army/commander: good. Cast 61%/57% of draws, stuck 39-43% (no spare body). Target list: problem 1 |
| Diabolic Tutor | `tut=any` | generic-WRONG target | problem 1; cast 78% of draws |
| Disdainful Stroke | `ctr=mv4` | generic-ok | cast 44%/52% |
| Dreadhorde Invasion | dsl upkeep amass; lifelink clause unmodeled | deck (70 early) | cast 85% of draws; lifelink at power 6+ missing (small) |
| Exsanguinate | `xdrain` | deck (yshtola.rs option runs for Sauron seat) | not in sample, unknown |
| Fact or Fiction | `draw=2` | generic-ok (40) | instant-speed EOT cast: unknown; pile split modelled? unknown |
| Force of Will | `ctr=any free` | generic-ok | problem 3 |
| Go for the Throat | rem tgt=cna | generic-ok | |
| Grave Titan | bomb=7 | generic-ok | creature 42 / bomb rating |
| Inexorable Tide | prolif | deck (55) | cast 64-66% of draws, win cast 31-36 |
| Infernal Grasp | rem lose=2 | generic-ok | |
| Inferno Titan | dsl, 3-damage trigger "not modeled" in tags | generic-ok | threats/sweepers.rs says it is hand-coded; tag line may be stale |
| Iron Man, Armored Avenger | `ironman` | generic-ok (42) | draw-trigger counters; evasion grant for Army unknown |
| Jace's Archivist | `archivist` | deck (50 with Bowmasters/Sheoldred, else 20) + option | problem 6 |
| Kaervek the Merciless | `kaervek` | deck (54) | |
| Lazotep Reaver | dsl etb amass | deck (67 early) | cast 88% of draws |
| Lightning Greaves | `prot=boots` | deck (50 with Sauron out) | removed 0.02-0.03/game |
| Massacre Wurm | `wurm wurmdrain` | generic-ok (bomb 6) | |
| Mauhur, Uruk-hai Captain | `mauhur` | deck (63) | cast 86% of draws |
| Nibelheim Aflame | `wipe=nib` + flashback hand-discard | generic-ok | flashback "discard hand, draw 4" modelled? unknown |
| Night's Whisper | `draw=2` | generic-ok (40) | |
| Not of This World | `prot=notw` | deck (protect(): free with 7-power creature) | cast 31%/29%; cannot be tutored (problem 1) |
| Noxious Gearhulk | removal on ETB, gaintgh | generic-ok | |
| Orcish Bowmasters | `bowmasters flash` | deck (62) | cast 86% of draws |
| Ral and the Implicit Maze | hand-written (sweepers.rs) | generic (perm: 10) | valued as a plain permanent, never prioritised; the wipe is the point |
| Ringsight | `tut=ubr ringtempt` | generic-WRONG target | problem 1 |
| Sauron, the Necromancer | `necromancer` | generic-ok (42) | Wraith copy modelled? unknown |
| Scarlet Witch, Chaotic Avenger | tags only | generic-ok (42) | free-cast-on-connect modelled? unknown |
| Sephiroth, Planet's Heir | `heir` | generic-ok | |
| Sheoldred, the Apocalypse | `sheoA` | deck (60) | |
| Spell Pierce | `ctr=nc soft=2` | generic-ok | |
| Talrand, Sky Summoner | `spelltok=2` | generic-ok (42) | 25 spells make it a strong engine; cast priority not raised (unverified) |
| Terminate | rem | generic-ok | |
| The Ozolith | `ozolith` | deck (45) | cast 89% of draws; never tutored (problem 1) |
| Undermine | `ctr=any undermine` | generic-ok | 27%/30% cast |
| Urabrask, Heretic Praetor | mine.rs hook | generic-ok (42) | |
| Vandalblast | rem/wipe=vandal | generic-ok | |
| Vision, Synthezoid Avenger | tags only | generic-ok (42) | phasing on off-turn casts: unknown |
| Vraska, Betrayal's Sting | `vraska` | deck (52) | removed 0.07/game |
| War Machine | `warmachine` | deck (45) | |
| Whispersilk Cloak | `cloak` | deck (57) + equip option | |
| Rocks (Sol Ring, Arcane Signet, Mind Stone, Fellwar, 2 Signets, 3 Talismans, Lantern) | Full | deck (80 until turn 5) | Relic of Legends: legendary-tap mana "not modeled" |
| Lands (Barad-dur, Rogue's Passage, Scavenger Grounds, duals) | tags | generic-ok | Barad-dur X amass: not in sample; Scavenger Grounds response exists (decks.rs:1424). Unknown how Rogue's Passage is activated |
