# Report: scion-goldfish

Branch `scion-goldfish`, from `origin/scion-goldfish-brief`. Not pushed. Commits: 1c9fd373 (goldfish mode), 2af0208f
(the scion key), bd27652b (the dragon line), 47244740 (the v1 baseline), then the main session's d10bbb05 (the
look-ahead win cap) and the corrected baseline.

## Main-session review (2026-10-27)

- Re-ran `cargo test --release --no-fail-fast`: 3514 passed, 0 failed (before d10bbb05; its new test passes too, and
  fails without the fix). No process left running.
- Oracle text checked against `data/scryfall_cache.json` for Scion, Prismari and Ancient Copper Dragon: the copy until
  end of turn, the lost ability, stacked activations, storm and ward match. Ward treats an opponent at exactly 5 life
  as unable to pay (the rules allow paying to 0; no sane opponent would), left as is.
- **The agent's main finding was half right.** It said the goldfish is slow because the shared Tier 6 AI is slow. A
  trace showed a goldfish-specific bug first: the look-ahead passed up a ready Thassa's Oracle for "stop (hold mana)"
  because the hold bonus lifted a line that also won every playout above 100. Fixed in d10bbb05 (scores capped at a
  sure win). After the fix the goldfish is 1-4 turns faster (scion median 13 -> 10, Rograkh/Silas 14 -> 10), the table
  is unchanged (`--realism` t0-t4, t6), and what remains (medians 7-10) is the Tier 6 heuristic's pace: the hold bonus
  off changes nothing, and the adaptive AI is at 11. Details: `audit/scion/FINDINGS.md` (2026-10-27 section).

## Part 1: goldfish mode

Command: `commander-sim --goldfish KEYS [--games N] [--seed S] [--ai ...] [--trace GAME]`. The code is in
`sim-cli/src/goldfish.rs`, and the inert seats are in `sim-core/src/goldfish.rs`. KEYS can be JD's deck keys, Tier 6
pool keys, or `t6` for all six.

The seats:
- The deck under test is seat 1.
- Seats 2-4 are `goldfish-2`, `goldfish-3` and `goldfish-4`. Each has 40 life, 99 Wastes, and Goblin Roughrider as a
  commander it never casts.
- They keep their first seven cards.
- They take real turns: untap, upkeep, draw and end step.
- They skip the land drop, both main phases, combat and the end-of-turn window, and never block.
- With no mana and a hand of Wastes, they have nothing to cast in any priority window.
- Seeds and seed streams are the same as pool mode.

The report, per deck:
- **Kill turn.** The median is over all games, with a no-kill sorting last; the mean is over kills only. Also the
  cumulative share of games won by turn 1 through 8, and the share with no kill by round 20 (with how many of those
  the deck lost itself).
- **How it won.** Uses `t6report::win_route`.
- **Mulligans.** Kept 7 cards (no mulligan, or the free one), 6, or 5 or fewer.
- **Ready turn.** The `sp|ready` speed counter, or the plan turn for a deck that isn't Tier 6.
- **Protection on the kill turn.** For a combo kill, it is recorded at each combo attempt (`gf|prot`). For a damage
  kill, it is the hand and locks at the end of the game. Protection means a card tagged `ctr` or `prot`, Silence or
  Orim's Chant cast that turn, or Grand Abolisher on the battlefield. The report lists the cards that count.
- **Other lines:** the lines gone for and converted, the turn the commander was cast, and the `scion|act|<Dragon>`
  counters.

Tests:
- sim-cli goldfish::tests: inert seats never act (12 traced games); the same seed gives the same games and report;
  the deck-key list; the summary maths.
- An args test for the flag.

Docs: README main commands and `--help`.

## Part 2: the `scion` key

The deck:
- `decklists/JD/scion-ur-dragon-turbo.md` uses the pool-file layout: Commander line, Game Changers (20), a short v1
  strategy, Decklist by type, Import list.
- It was written by `cardgen deck scion`, after a dry run: 100 cards, 20 Game Changers, legality and colour identity
  fine.
- `cardgen export` fetched Prismari and Tinder Wall, and added only those two cards.
- `data/my_decks.toml`: `[scion]`, tier t6.

Key wiring:
- sim-cli: decks.rs FILES/MINE and its test, args.rs DECK_KEYS, the poolmode tier test, and report.rs plan text
  ("first combo attempt").
- practice: catalog.rs and setup.rs MY_DECKS.
- Deck-count tests: app_entry, http_catalog, http_server, server_fake (two error strings) and game.rs (84 to 91).
- T6_KEYS goes from 7 to 8 in impls/t6_plans.rs (with its dispatch) and t6stats.rs; the ai_t6_table test is updated.

Style: ai/mod.rs has a new `T6_JD_STYLES` table with scion at temp 1.0, aggression 0.6, caution 0.35.
- Aggression stays at the default because Scion's attacks are for Copper Dragon's Treasures.
- Caution is below the pool's 0.5 so this turbo deck spends its mana instead of passing with a counter up.

Plan file: `impls/t6_plan_scion.rs`.
- `keep` is the Rograkh/Silas turbo rule with this list's rituals, tutors, payoffs and engines. It is marked
  provisional until D11.
- The other four slots return None.
- Its header explains the Scion rule.

Docs: the CLAUDE.md tiers line, the README tier table and deck list, the documents/testing-decks.md tier table, and
the documents/how-it-works.md Tier 6 plan paragraph.

## Part 3: Scion's dragon line

**The copy** (`impls/t6_scion.rs`):
- It uses Mirage Mirror's `set_face`: the permanent takes the Dragon's card, name, body, keywords and card code.
- Scion's own card is kept in `phys`, and a `ScionCopy` flag marks it.
- `copy_revert` ends the copy at the start of the next turn, after the end step and its window.
- The copied Dragon's hooks fire for Scion. Copper Dragon and Korvold work unchanged, because their hooks are keyed on
  the permanent.
- Scion's own ability is gone while it is a copy.
- `activate` puts all its activations on the stack, then resolves them last-first. The newest copy wins.

**Prismari:**
- Storm is a cast hook: one copy per spell cast before it this turn, by anyone. It goes through `trigger_window`, so
  it can be Stifled.
- Brain Freeze gets a second storm trigger: n copies milling 3 each, aimed at the targets after the n+1 its own storm
  uses.
- Ward—Pay 5 life applies to opponents' removal spells. At 5 life or less the removal is countered.

**The Scion AI** (deck `scion` only):
- Before combat, if Scion is untapped and not summoning sick and {2} is spare, it becomes Ancient Copper Dragon.
- If Copper is gone and there is a Treasure to sacrifice, it becomes Korvold.
- Prismari is fetched only as the first step of the storm line.

**The lines:**
1. **Catalogued: Scion as Prismari + Underworld Breach + Manamorphose + Brain Freeze, at the table.**
   - Sequence: {2} Scion → Prismari (skipped if Prismari is out). Then {1}{R} Breach. Then {1}{R/G} Manamorphose, whose
     storm copies make 2 × (storm+1) mana of any colour and draw a card each. Then {1}{U} Brain Freeze at yourself,
     which makes 2 × storm + 1 copies of "mill 3", for fuel.
   - Then escape Brain Freeze until every opponent's library is empty. When mana runs low, escape Manamorphose (or a
     Dark Ritual, Cabal Ritual or Rite of Flame).
   - Mana up front: 6 with Breach in hand, 4 with Breach out, 2 less with Prismari out.
   - A test covers it. In the corrected goldfish it won 0.9% of kills.
2. Prismari + Breach + a black ritual: not a sure kill, because the rituals make no {U} for the next Brain Freeze.
3. Prismari + Brain Freeze + Thassa's Oracle without Breach: needs a storm count near 12 on turn 3. Not a sure kill.
4. Scion as Copper Dragon, Scion as Korvold, and Flash Photography or Mockingbird copying Scion as Prismari: these are
   engines.
5. The existing Oracle line and both Breach + LED + Brain Freeze finishes are found by the scion seat (tested).

**Tests** (`tests/cards_scion.rs`, 13): the copy and its end at cleanup; the lost ability; stacked activations; the
AI's Copper option; Prismari's storm on Dark Ritual (3 → 9 mana); Brain Freeze milling 9 → 15 cards; Scion as Prismari
gives storm; Copper's Treasures with Scion as the copy; Korvold's draw and counter with Scion as the copy; Prismari's
ward; the Prismari line wins; the Prismari line needs Scion; the scion seat finds the Oracle and Breach lines.

## Shared-file changes (each marked `// --- scion`, except the main session's search fix)

- engine/turn.rs: inert-seat gates in `continue_turn`, `step_start`, `setup_game_for` and `run_rounds`; `copy_revert`.
- engine/combat.rs: inert seats never block.
- engine/zones.rs: a copy whose physical card is a commander goes to the command zone.
- engine/removal.rs: Prismari's ward.
- state.rs: `DataKey::ScionCopy`.
- t6stats.rs: `goldfish::mark_go`, and scion in T6_KEYS.
- combos.rs: the new line is chained at the end of `all()`.
- t45_urdragon.rs: the gate on deck key `scion`.
- alela.rs: `set_face` is now `pub(crate)`.
- ai/mod.rs: `T6_JD_STYLES`.
- impls/t6_plans.rs, impls/mod.rs and lib.rs: module wiring and the dispatch.
- data/audit.toml: notes for Scion (Approximate) and Prismari (Full).
- **ai/search.rs (main session, d10bbb05):** `choose_inner` caps rollout scores at a sure win. Every deck uses it.

## Measurements

See `audit/scion/FINDINGS.md`: the 2026-10-27 section (after the win cap) supersedes the v1 baseline's goldfish
numbers. Corrected goldfish, look-ahead, 2,000 games, seed 500000:

| deck | median | mean | by T2 | by T3 | no kill | protection |
| --- | --- | --- | --- | --- | --- | --- |
| scion | 10.0 | 10.2 | 0.3% | 2.9% | 3.9% | 80.5% |
| dack | 10.5 | 9.9 | 0.8% | 6.9% | 10.4% | 47.3% |
| kinnan | 8.0 | 8.7 | 0.1% | 2.1% | 1.0% | 86.1% |
| kraum | 9.0 | 9.5 | 0.3% | 2.0% | 7.2% | 90.5% |
| rograkh-silas | 10.0 | 9.2 | 0.6% | 4.8% | 21.1% | 77.3% |
| rograkh-thrasios | 9.0 | 9.3 | 0.0% | 0.1% | 0.5% | 85.0% |
| sisay | 7.0 | 7.6 | 0.0% | 0.1% | 0.2% | 73.0% |

Other runs: scion with the adaptive AI, median 11.0 (unchanged by the fix); scion vs Tier 6 (loose, look-ahead) 26.5%
(24.6-28.5), comparative only, because the Tier 6 table wins at a median round of 9 against about 7 in real games.

Scion in the v1 goldfish: cast in 67.4% of games (median turn 5); became Copper Dragon in 52.0% of games, Korvold in
39.0%, Prismari in 0.5%.

Cards not Full (`cardgen audit --mine`; `--deck` only takes pool decks):
- Partial: Gemstone Caverns, Orim's Chant, Redirect Lightning, Shifting Woodland, Tinder Wall.
- Approximate: Crop Rotation, Hexing Squelcher, Lotho Corrupt Shirriff, Scion of the Ur-Dragon.

## The hate scenarios: what goldfish mode would need

1. **A counter on the first combo piece.** A scripted inert seat with a fixed hand (for example Force of Will and a
   card to pitch) that answers once in `counter_window`. The counter must also be in that seat's deck list, so
   `t6_table::stop_chance` sees it.
2. **Drannith Magistrate in play.** A setup hook so a goldfish seat starts with a permanent on the battlefield.

Both need a `--goldfish-hate NAME` option and a seat spec next to `goldfish::inert_seat`.

## Decisions made without JD

By the agent:
1. **Flag shape.** `--goldfish KEYS` rather than `--goldfish` plus `--deck`, so one run covers scion and the six pool
   decks; `--deck` only accepts JD's keys.
2. **Inert seats take full turns, including the draw step**, as the brief says. Side effect: Smothering Tithe makes a
   free Treasure at every inert draw, because nobody pays. Rhystic Study, Mystic Remora and Esper Sentinel never
   trigger, because the inert seats never cast.
3. **Inert commander: Goblin Roughrider.** A Wastes commander would make every Wastes in the library a "commander"
   card.
4. **Statistics.** The kill-turn median is over all games; the mean is over kills only.
5. **Protection** is read at the combo attempt for combo kills, and at game end for damage kills. Veil of Summer,
   Misdirection and Redirect Lightning carry neither tag, so they don't count.
6. **scion is not in `settings.rs` cthresh, `rules.rs` CTHRESH, `MAIN_STYLES` or `jd_deck`.** Godo was never in those
   lists either; cthresh would replace the pool counter formula (about 3.5 for this list) with a flat 6, CTHRESH
   switches off the Veil of Summer response, and MAIN_STYLES and jd_deck would switch off the combo lines.
7. **The copy ends at the start of the next turn**, as Mirage Mirror's does, rather than literally at cleanup.
8. **zones.rs: a copied commander goes to the command zone.** Rules-correct and needed for Scion.
9. **The Scion AI is gated on deck key `scion`.** The Ur-Dragon pool deck is unchanged.
10. **The catalogued line uses Manamorphose, not a ritual.** Only Manamorphose makes blue.
11. **Prismari's Brain Freeze copies target after the original's own storm copies**, so they don't mill the same
    players twice.
12. **Ward covers removal spells only**, matching how Hexing Squelcher's ward is modeled.
13. **A bug outside scope, recorded and not fixed:** Thousand-Year Storm's copies of Brain Freeze each recount storm,
    so each copy mills 3 × (storm+1) instead of 3. The scion deck doesn't play it.
14. **The Tier 6 AI's speed was not tuned** (out of scope).

By the main session: the win cap in `ai/search.rs` (a shared AI change, measured neutral at the table); Tier 5 left
out of the before/after (memory cap, the Miirym Treasure bug fixed on phase2-doubles).

## What JD must decide

- Whether to start a speed task for the Tier 6 AI (or give scion its own `prio` / `hold_up`). Until then D1 can't be
  checked: even after the fix, every Tier 6 deck kills at a median own turn of 7-10 in the goldfish.
- Whether the search fix ships with this branch or as its own PR.
- Whether to keep the Smothering Tithe Treasures in the goldfish.
- The mulligan rule (D11).
