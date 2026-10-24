# Overnight 2026-10-24: decisions made for JD, and what was done

JD (2026-10-24, before bed): keep working; where you would ask, make the recommended decision and brief me in the
morning. Each decision below says what was chosen, why, and how to undo it. Newest last.

## Decisions

1. **Casual tier for the Phase 0 gate: Tier 3** (Aurelia, Korvold, Marwyn, Tergrid, Atraxa). Most of JD's decks play
   it (Sheoldred, Sauron, Seph, Kefka, Cloud, Veyran, Omnath). Branch `realism-casual-targets`, `data/realism/t3.toml`.
2. **Atraxa left out of every Tier 3 target.** 45% of Playgroup's Atraxa wins are poison (infect builds); the pool's
   Atraxa is superfriends, so with it the poison target would be 22.6%. Alternative: make the pool's Atraxa an infect
   list (JD allowed pool-deck changes), then add it back.
3. **Tier 3 median win round from Playgroup's average win turn** (8.1, an average; Playgroup has no distribution).
   Band 7-9. No targets for deck spread, games over 15 rounds, win-round spread (no fitting source).
4. **Knobs file format**: TOML, `--knobs FILE`, every value optional, unknown keys and decks refused
   (`rust/crates/sim-cli/src/knobs.rs`). Style overrides live in `Settings::styles`, read before the built-in tables.
5. **Phase 1 route** (JD chose "real lists first, then fit"; then "the default is to adjust the existing lists"): the
   Tier 6 pool decks are adjusted by card swaps toward what real tournament lists of the same commanders run
   (EDHTop16 decklists and their per-commander play rates). JD, later the same night: "You can actually just replace
   lists if it makes the most sense." So a list is replaced outright where adjusting it would mean most of the deck.
6. **Finding: the Tier 6 pool lists are already close to the real lists.** Against 8 recent EDHTop16 winning lists per
   commander (48 lists, Apr-Oct 2026), each pool deck has 49-86 of the real core (cards in 6+ of 8 lists; 51-87 such
   cards per commander), and only 6 pool cards appear in none of the 48 lists. So the lists are not why the sim's
   Tier 6 is slow (median win round 9-10 against 7); the gap is in how the sim plays or models the decks.
7. **Small list alignment applied** (15 swaps, `cardgen deck`, logged in decklists/pool/CHANGELOG.md): each missing
   real-core card in for the pool card that appears least in the real lists, land for land, keeping cards the Tier 6
   plan code names (Snap, Misdirection). Left out: Great Hall of the Citadel, Necrodominance, Sylvan Scrying (not in
   cards.json yet) and Everflowing Chalice (Karn's FINDINGS: the sim never casts it). Two new cards are Approximate:
   Prismatic Lens (filter ability) and Crop Rotation.
8. **cardgen bug fixed:** `cardgen deck` refused every partner pool deck ("commander 'A / B' is not in the import
   list"): update_deck read the `- **Commander:** A / B` line as one name. It now splits the pair as
   decks::load_pool_deck does (`split_pair`, with a test).
9. **What the Tier 6 speed is NOT** (each tested, adaptive AI; files in `audit/realism/t6-lists-2026-10-24/` and
   `audit/realism/knobs/`):
   - the AI knobs: no knob at either end of its range moves seat 1 off 25-28% or the median win round off 9-10;
   - the deck lists: aligned with real winning lists (decision 7), same seeds, before 27.0% / 10, after 26.0% / 10;
   - Dack Fayden's stax: the table without Dack (throwaway worktree) is 26.4% / 10;
   - mulligans: real players of these commanders average 0.69-0.96 mulligans a game (Playgroup), the sim 0.63-0.76.
   Real games with these commanders last 6-7 rounds on average (Playgroup); the sim's last 10.3.
   Leads left (calibrate-within report): fast mana held on turns 1-3 (Mox Diamond 201 card-turns in Kinnan, 212 in
   Rograkh/Silas, i.e. the AI won't discard a land to it), turn-1 mana 1.0-1.1 for every deck; Dack Fayden's removal
   (3,686 in 1,345 games, mostly repeated Fiend Hunter / All-Fates Stalker / ability: it is the sim's strongest deck at
   35%, the weakest in real play); Kraum/Tymna still waits with a ready line (153 of 588 ready games never went).
10. **Open bug found (not fixed): Ertai Resurrected in its owner's hand stops that player's Tataru Taru Treasure.**
    The list swaps changed Sisay's shuffle, so `scions_secretary_once_a_turn_and_not_on_their_own_turn` drew Ertai
    into Sisay's hand and the Treasure never came (the test passes with any one swap, fails with all four). The test
    now fixes every library (its file tests cards "on a hand-built position"), and the bug has its own ignored test,
    `tataru_with_ertai_in_hand` (cargo test -p sim-core --test cards_t6_legends2 -- --ignored). Traced as far as:
    nothing is logged (no flash, no counter); ertai_answer already refuses its owner's triggers; the Draw trigger is
    queued (`at_once` false) and goes through flush_triggers / resolve_batch, which is the next place to look.
11. **Mox Diamond's Tier 6 rule is now a knob, `[table] t6_moxd_lands` (default 3, unchanged).** The AI wanted three
    lands (hand plus battlefield) before casting it, so a turn-1 two-land hand never played it. At 2 or 1 the Tier 6
    table is the same (seat 1 26.0 -> 26.2%, median 10; 8,200 adaptive games each; `moxd*-8200.txt`); at 3 the output
    is byte-identical to no knobs. So fast mana held in hand is not the speed gap either. What is left in the
    calibrate-within report: each player tries a combo about 0.25 times a game and 30-47% of tries are stopped; lines
    are "one tutor away" at a median round of 6-7 but become ready in only 18-45% of games. The slowness is in how
    fast the decks turn tutors and draws into a finished line: AI and plan work (Phases 1 and 4), not knobs or lists.
12. **EDHREC not used.** Its JSON (per-card inclusion rates per commander) would be the casual equivalent of the
    EDHTop16 play rates for adjusting the Tier 3 lists, but it is not one of the plan's approved sources (Playgroup,
    EDHTop16, TopDeck.gg) and its terms for automated access weren't checked. One page was fetched to see the format;
    nothing from it is used. JD to decide.
13. **Dack Fayden's removal count is probably not a bug** (correcting the lead in item 9): Fiend Hunter and All-Fates
    Stalker return what they exiled when they leave (`impls/t6_dack.rs`, `oring_return`); the count is likely the
    flicker plan re-exiling the same creature each blink. Why Dack wins 35% in the sim is still open.
14. **First Tier 6 fits, not adopted** (`audit/realism/fit/`, adaptive, 2,000 games a candidate): shared scope 15
    generations, loss 752 -> 326; per-deck scope 12 generations, 752 -> 367. Nearly all of the gain is the win types
    (combo 68% -> 59-61%, damage 31% -> 38-40%), the weakest Tier 6 targets; the median win round got worse (10 ->
    11-12) and seat 1 moved 1.6-3.2 points. Several knobs ran to their bounds. Nothing changes the built-in values;
    the knobs files are there to test. Fresh-seed check: item 15.
15. **Lead: plays the option list offers but `act::perform` can't make.** In 8 traced Tier 6 games, 27 of 763 main-phase
    decisions rate a play about 100% and end in "stop" because perform failed (Chrome Mox 8, Damping Sphere 4, Mox
    Opal, Llanowar Elves, Elves of Deep Shadow, Ragavan, Mystic Remora 2 each...), always in pairs (both main phases).
    In isolation the Kinnan case (Llanowar off Talisman of Curiosity) casts fine, so it needs the game's own mana
    state: `commander-sim --calibrate within --pool t6 --ai adaptive --trace 3`, round 5, Kinnan. Each lost cheap
    mana play slows a deck a turn; worth fixing before more fitting.
16. **Item 15's lead is mostly Dack's taxes, not a bug.** With a log line added when `act::perform` refuses a chosen
    play (`ai/brain.rs`, logging only), 8 traced Tier 6 games have 132 refused plays (Erode 22, Mystic Remora 17,
    Redirect Lightning 16, Ragavan 12, Chrome Mox 12, Elves of Deep Shadow 11, Damping Sphere 11...), all in games
    with Dack Fayden seated: the two games without Dack have none, the one where Dack resolved two tax pieces has 63.
    So the refusals are taxes (Damping Sphere, Thorn of Amethyst, Defense Grid on other players' turns) the option
    list doesn't price in; the engine is right to refuse. The ranking could price taxes in, but the table without
    Dack is as slow, so this is not the speed gap.
17. **Gate-only fits: noise.** `--fit-stats seat1_win,seat2_win,seat3_win,seat4_win,win_round_median` (shared 12
    generations, per-deck 10) reached seat 1 28.4% / 28.1% on their own seeds; on fresh seeds (900000-908199, 8,200
    games) they give 25.6% / 26.2% against the built-in 26.0%, the median 10 / 9. That was the luckiest candidate
    fitting the seeds' noise. Conclusion for Phase 1: fitting the existing knobs cannot pass the gate; the speed has to
    come from the AI's and plans' conversion of tutors and draws into lines (REPORT.md, "Next leads").
18. **New Tier 6 stats: what a deck does while one tutor away.** The calibrate-within report now prints, per deck,
    the choices made while a line is one tutor away and none is ready, and the tutors held when it passed then
    (stats only: the rest of the report is byte-identical). Rograkh/Silas passed 365 times with a tutor (Diabolic
    Intent 241, Mystical Tutor 113, Demonic Tutor 24); Kraum/Tymna 212 (Mystical 86, Diabolic Intent 69, Enlightened
    47). Caveat: "one tutor away" counts any tutor, whether or not it finds the missing piece this turn, so holding
    Mystical (instant, to the top) or Enlightened is often right.
19. **AI fix, behaviour change: a commander with mana value 0 is sacrifice fodder.** `ai::brain::spare_creature`
    (Diabolic Intent's sacrifice) skipped every commander, so Rograkh, Son of Rohgahh, which real lists sacrifice
    freely (it is recast for the tax alone), never was, and Rograkh/Silas sat on Diabolic Intent. Now a 0-cost
    commander is spare at worth 0; costed commanders stay protected (test in ai_t6_rograkh_silas.rs). Same 8,200
    seeds: Diabolic Intent held while one tutor away 241 -> 41, passes 365 -> 167, Rograkh/Silas 21.7% -> 22.7%, the
    table's median win round 10 -> 9 (seat 1 unchanged, 25.9%). Undo: revert `free_commander`.
20. **The instant tutors are played right.** In 40 traced Tier 6 games, all 10 Mystical Tutor casts and 15 of 16
    Vampiric Tutor casts came on another player's turn (end of turn), as real players use them; Demonic Tutor and
    Diabolic Intent (sorceries) on the owner's. So the Mystical holds in item 18 are correct play, and the remaining
    "passed while one tutor away" cases are mostly not bugs. Remaining speed leads: Kraum/Tymna's held lines (it
    holds a ready line in a quarter of its ready games), Sisay and Rograkh/Thrasios (lines ready in only 10-11% of
    games, median round 10, wins at 13).
21. **Lead (not fixed): the AI prices printed costs, the engine charges taxed ones.** The most common combo-step
    failure in 48 traced games is Thassa's Oracle + Consultation stopping at step 2 (11 times): Oracle is cast, then
    Demonic Consultation can't be. In game 32 (`--calibrate within --pool t6 --ai adaptive --trace 32`) Dack's Damping
    Sphere, in play since round 1, makes Consultation cost one more after Oracle, so a line that passed the AI's
    affordability check dies half-cast. Same root as item 16's refused plays. The fix is to have the options,
    `combos::affordable` and the plans use the engine's taxed cost: cross-cutting (Phase 3's characteristics pass is
    the natural home), so left for JD. One stop (game 11) has no tax piece; its cause is not found.

## Where things stand (morning summary)

- Branches (pushed, no PRs): `realism-casual-targets` (Tier 3 targets: Phase 0's last step), `realism-phase1` (knobs,
  `--fit`, list alignment, the Rograkh fix, stats, reports; contains the first branch), `docs-refresh` (fresh docs and
  archives; contains `realism-phase1`). Merge order: realism-casual-targets, realism-phase1, docs-refresh, or just
  docs-refresh, which holds all three. Earlier: `yshtola-cedh` (pushed), PR #132 merged.
- Phase 0: done once `realism-casual-targets` is on main and a nightly run scores Tier 3.
- Phase 1 gate (Tier 6 seat edge) not passed: seat 1 ~26% against 31.5-33.5%. Knobs and lists can't move it (fitted
  and checked on fresh seeds); the first change that moved the speed is the Rograkh fix (median 10 -> 9). Next:
  item 21 (taxed costs), then Sisay's and Rograkh/Thrasios's lines (both win at a median round 13), then Kraum/Tymna's
  held lines.
- Decisions to confirm: 1-3 (Tier 3 method, Atraxa), 5 (list route), 7 (the 15 swaps), 12 (EDHREC unused), 19 (the
  Rograkh fix), and whether the win types keep full weight in `--fit`'s loss (14).

22. **JD, morning: EDHREC may be used for the casual decks** (answers item 12).
