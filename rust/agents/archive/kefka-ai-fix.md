# Task: kefka-ai-fix — make Kefka's AI cast his cards properly, and ready him for Veyran's cards

Read `rust/agents/OVERNIGHT-RULES.md` first: its hard rules apply here (no Python, memory cap, no wait loops, edit
only your files, commit often, handoff, report in 15 lines or fewer). Where it says "Kefka/Zada", read "Kefka".
**Another Claude session is running sims on this machine at the same time:** use `--jobs 8` at most.

**Base branch:** `kefka-veyran-trade`. **Your worktree:** `.claude/worktrees/kefka-ai` (branch `kefka-ai-fix`).

**Your files:**
- `impls/kefka.rs` (Kefka's plan: `kefka_prio`, `loop_safe`, `flip_utility` and the rest);
- `tests/cards_kefka_ai.rs` (new);
- the hand_options or prio of the cards named below, wherever they live (Kaervek's Purge is in `impls/jodah.rs`).

Shared files (`ai/*`, `engine/*`) get only small additive changes, gated to `key == "kefka"` where the change is
about deck play, and each one is listed in your report.

**The deck:** `decklists/JD/kefka-grixis-discard.md` (key `kefka`). It discards and loots to flip Kefka; the
flipped side draws cards when opponents lose life on your turn. **The kill** is Psychosis Crawler plus the flipped
Kefka: every card drawn makes each opponent lose 1, and each loss draws more.

## Why

The lead ran the card report on Kefka:

```
commander-sim --deck kefka --pool tN --profile loose --games 600 --seed 500000 --ai adaptive --jobs 8 --analyze
```

That's 5 tiers. It shows these cast rates (the share of games where the card was drawn and then cast):

| Card | Cast rate | Note |
|---|---|---|
| Kaervek's Purge | **0%** | Marked "[never cast by the sim: untagged]", yet the audit says Full and `impls/jodah.rs` has `purge` hand_options |
| Mystic Confluence | about 1% | Only cast in response |
| Arcane Omens | 9–14% | |
| Splatter Technique | 13–17% | |
| Triskelion | 14–19% | |
| Rapturous Moment | 17–21% | |
| Inner Fire | 18–25% | |
| **Psychosis Crawler** | **20–27%** | His win condition |

The next step is choosing which cards Kefka cuts. That choice is only fair once the AI uses every card it should.

## Do

1. **Kaervek's Purge.**
   - Find why Kefka never casts it. Is it the "untagged" analyzer label? Do hand_options never get offered for this
     deck? Or does `purge`'s gate (`post`, active player, `pval >= 3`, `can_pay` for X) never pass?
   - Fix it so the deck casts it on a good target.
2. **Psychosis Crawler.**
   - Trace a few games where the Crawler sat in hand (`--trace N`) and find what held it.
   - On his own he is worth casting: Kefka's discard and loot draws ping the table. Before the flip he gets prio 30;
     check what beats him and whether that's right.
   - Keep the `loop_safe` guard: an unsafe mandatory loop decks us.
3. **The other low-use cards** (Mystic Confluence, Arcane Omens, Splatter Technique, Rapturous Moment, Inner Fire,
   Triskelion).
   - For each one, say whether the low rate is **genuine** (cost, situational) or an **AI gap** (a missing tag, prio
     or option). Fix the gaps.
   - Mystic Confluence: an instant whose draw-3 mode should be cast at the end of an opponent's turn when the mana
     went unused. Check whether that mode is offered.
4. **Veyran's incoming cards.** The lead will test these in Kefka with `--swap "Out=>In"` (repeatable); they must
   work there:
   - **Deflecting Swat and Slip Out the Back:** the protection hook already exists (`ai/decks.rs` protect_response
     `"kefka"` → `veyran_protect`). Just confirm with a swap and a test.
   - **Underworld Breach:** generic `breach_gc_options` in `impls/mine.rs`. Confirm Kefka escapes his discarded
     spells. Check whether **Garrison Excavator** (Kefka's) triggers when Breach exiles cards from the graveyard, if
     its Oracle text says it should.
   - **Niv-Mizzet, the Firemind:** "whenever you draw a card, deal 1 damage to any target". With the flipped Kefka
     that's the same loop as the Crawler.
     - Check both cards' Oracle text (`jq '.["..."]' data/scryfall_cache.json`) for whether the triggers are
       mandatory.
     - Make `loop_safe`, `flip_utility` and the targeting treat Niv like the Crawler: damage at opponents, never
       decking us.
     - Cast priority: like the Crawler's.
   - **Jin-Gitaxias, Progress Tyrant:** check that his copy and counter work for Kefka, and give him a sensible cast
     priority.
   - **For the swaps,** take the "Out" cards from Kefka's list, for example `Rummaging Goblin`, `Hallway Heckler`
     and `Arcane Omens`. Note that the four lands (Steam Vents, Sulfur Falls, Spirebluff Canal, Stormcarved Coast)
     will replace Islands and Mountains.
5. **Invariance.** Any change outside Kefka's own code must leave the other decks' fixed-seed results identical.
   Check Veyran, Sauron and Jodah with `--seed 502000 --games 200` on t2 and t4, before and after.
6. **Measure.** Run the card report above again and give before and after cast rates for each card listed. Also
   give before and after win rates per tier, with adaptive and 600 games per tier.

## Done when

- The OVERNIGHT-RULES checks all pass: the full `cargo test --release`, no warnings, `all_decks adaptive 3` and
  `lookahead 1`, the audit, and a 48-game `--deck kefka` smoke run at `--jobs 8`.
- There's at least one test per fix in `tests/cards_kefka_ai.rs`.
- The report is written: line 1 DONE or HANDOFF, line 2 your model ID. The full details go in
  `rust/agents/report-kefka-ai-fix.md`, with these sections:
  - each card's cause, fix or "genuine" verdict;
  - the shared-file changes;
  - the invariance results;
  - the before and after tables.
