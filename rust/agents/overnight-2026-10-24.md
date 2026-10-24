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
