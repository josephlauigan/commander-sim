# Cards

How the simulator knows what a card does, how faithfully each card is modeled, and how to add or fix one. `cardgen`
does the data side; its full reference is [`rust/crates/cardgen/README.md`](../rust/crates/cardgen/README.md).

## The pipeline

```
data/carddb.txt ─┐
Scryfall cache ──┼─► cardgen export ─► data/cards.json  (2,651 cards)  ─► sim-core loads them
card_overrides ──┤                     data/decks.json  (every deck)
decklists/ ──────┘
                                       + Rust card code in rust/crates/sim-core/src/impls/
```

A card's definition comes from the first source that has it:

1. **`data/carddb.txt`**: hand-checked tags, one `Name|types|cost|tags` line per card (`Grim Tutor|S|1BB|tut=any
   lose=3`). Types are `L C I S A E P` letters. The tags are the engine's vocabulary (`rem=exile`, `draw=2`,
   `rock=1:C`, `bomb=9`); `rust/crates/sim-core/src/tag.rs` lists them. JD's decks' AI is built around these.
2. **Scryfall** (`data/scryfall_cache.json`): the Oracle text is compiled into abilities (`cardgen/src/dsl_parse.rs`)
   and the AI's tags come from those abilities; when nothing compiles, a regex tagger (`autotag.rs`) makes the tags.
   Scryfall also gives the types, cost, keywords, colour identity and Game Changer flag. A card missing from the cache
   is fetched once (`--offline` forbids it).
3. **`data/card_overrides.toml`** then replaces a card's tags, types, cost or abilities, adds keyword data, and holds
   the audit verdict (`status`, `note`). It does not replace the tags or abilities of JD's hand-tagged cards.

**Rust card code** sits on top of any source, for what tags and abilities can't express (locks, taxes, commander
engines, combos, unusual replacements). A function is registered by card name and event in an `impls/*.rs` file
(`r.card(db, "Smothering Tithe")?.draw = Some(tithe);`); `hooks.rs` defines the events and slots, and a name that is
not in the card data is an error at start-up. `impls/mod.rs` registers the files in a fixed order; a later
registration of the same card and event replaces an earlier one. Combos live in `impls/combos.rs` (pieces, readiness,
mana, result). `impls/t1.rs` is a compact example of the pattern.

After any change to a data file, a deck list or the cache, run `cardgen export` and commit the JSON. The test
`export_matches_committed_files` fails when `data/*.json` is stale.

## Model statuses

`cardgen audit` rates every card (`rust/crates/cardgen/src/audit.rs`):

| Status | Meaning |
| --- | --- |
| Full | Hand-verified against Oracle text: an override's `status`, `documents/card-audit.md`, or `[card_notes]` in `data/audit.toml` |
| Full-auto | Every Oracle line compiled by the ability language and no ability keyword dropped; not hand-verified |
| Approximate | Modeled with a simplification (regex tags, or a note) |
| Partial | The main effect is in; some text or keywords are ignored |
| Unmodeled | The simulator never casts it, or it does nothing when cast |

```
cardgen audit                  # per pool deck: cards by status, key engines that are fully modeled
cardgen audit --deck yuriko    # every card of one deck (a key prefix is enough)
cardgen audit --mine           # JD's and Avery's decks: every card that is not Full
cardgen audit --md FILE        # also write the tables as Markdown
commander-sim --deck veyran --cards   # one deck: every card's tags, abilities, source and unmodeled text
```

`documents/card-audit.md` is the hand check of JD's and Avery's deck cards against Oracle text; `cardgen audit` reads
it, so it stays in `documents/`. A `--swap` run prints the status of each swapped-in card and warns on Partial and
Unmodeled ([`documents/testing-decks.md`](testing-decks.md)).

## Adding or fixing a card

- **A new card in a deck.** Put it in the list (`cardgen deck <key> new-list.txt`, or edit the file's
  `## Import list`), then `cardgen export`. A card with no `carddb.txt` line is built from Scryfall. Check it with
  `commander-sim --deck <key> --cards` and `cardgen audit --mine`.
- **The automatic model is wrong.** Add a `carddb.txt` line, or write its abilities or tags into
  `data/card_overrides.toml` (`dsl = [...]`, the ability JSON as TOML), with a `status` and `note`.
- **It needs code.** Register a slot in the `impls/` file for its deck or tier (`impls/mine.rs` for JD's decks,
  `common.rs` for staples), with a test in `sim-core/tests/cards_*.rs` that checks the behaviour against the Oracle
  text.
- **It needs a decision from the deck's AI** (when to cast it, what to tutor). Add it to the deck's priority function
  (`ai/decks.rs`, `ai/seph.rs`, `ai/veyran.rs` or the deck's `impls/` file); the swap run's preflight shows whether the
  AI would ever cast it.

What the engine can't do yet, found while modeling the Tier 6 cards, is listed in
`rust/agents/engine-needs-*.md` (one file per batch).

## Deck files

Every deck is a Markdown file: JD's in `decklists/JD/`, Avery's in `decklists/Avery/`, the pool in
`decklists/pool/<tier folder>/`. The simulator reads the `## Import list` block (`1 Card Name` lines), the
`- **Commander:**` line and the `**Game Changers (N)**` claim; the rest is for people. `data/my_decks.toml` maps each of
JD's and Avery's deck keys to its file, display name, commander, colour identity and opponent tiers. `cardgen validate`
checks every pool deck (100 cards, singleton, commander, names, colour identity, bans, the tier's Game Changer rule).
Deck changes are logged in `decklists/CHANGELOG.md` (JD's and Avery's decks) and `decklists/pool/CHANGELOG.md` (the
pool).
