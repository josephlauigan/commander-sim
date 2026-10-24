# cardgen: card data and deck lists

`cardgen` builds the two files the engine loads, `data/cards.json` and `data/decks.json`, from files you can edit by
hand, the Scryfall cache and the deck lists. It also edits deck lists (`cardgen deck`), checks the pool decks
(`cardgen validate`) and reports how faithfully every card is modeled (`cardgen audit`).

Run it from anywhere inside the repository (`--root DIR` names the repository otherwise):

```
export PATH=$HOME/.cargo/bin:$PATH
cd rust
cargo run --release -p cardgen -- export          # or ./target/release/cardgen export after a build
```

## Where a card's data comes from

Every card in the files is built in this order; the first source that has the card wins.

1. `data/carddb.txt`, the hand-tagged cards: one `name | types | cost | tags` line each. These are the cards your decks'
   AI is built around. The tags are the roles the AI reads (`pow=4 fly leg rock=1:C ...`, see
   `crates/sim-core/src/tag.rs`).
2. Every other card is built from its Scryfall record (`data/scryfall_cache.json`): the Oracle text is compiled into
   abilities (`dsl_parse.rs`), the tags the AI reads come from the compiled abilities (`dsl.rs`) and from the regex
   tagger (`autotag.rs`) when nothing compiles. Scryfall gives the types, cost, keywords, colour identity and the
   Game Changer flag.
3. `data/card_overrides.toml` then replaces a card's tags / types / cost / abilities, adds keyword data and tag edits,
   and holds the audit's verdict (`status`, `note`). The file's header lists every key. Your decks' hand-tagged cards
   (those in `carddb.txt` that one of your decks plays) ignore an override's tags and abilities.

`data/python_hooks.txt` is a frozen record of the events the old Python card code (since removed) handled for each card; it is copied
into cards.json (`python_hooks`, `spell_prio`) and `cardgen audit --hooks` compares it with sim-core's Rust card
registry. `data/my_decks.toml` names your decks (file under `decklists/`, display name, commander, colour identity).
`data/audit.toml` holds the audit's tables (keywords the engine lacks, the key engines of each pool deck).

## Add a card by hand

Add a line to `data/carddb.txt`:

```
Test Widget|A|2U|rock=2:C sac
```

Types are `L C I S A E P` letters (`AC` is an artifact creature), the cost is the generic number then coloured
pips (`2UU`; `-` for a land), then the tags. A later line for the same name replaces an earlier one. Then run
`cardgen export`. A card that isn't in the cache is fetched from Scryfall on the way (`--offline` forbids that).

To give a card that is built from its Oracle text different abilities, add a table to `data/card_overrides.toml`
(the abilities are the JSON of `dsl_parse.rs`, written as TOML):

```toml
["Grave Pact"]
types = "E"
tags = "pact"
dsl = [{ type = "triggered", event = "dies", source = "you_creature", effects = [{ do = "sacrifice", who = "each_opponent", filter = { type = "creature" } }] }]
status = "Full"
note = "each opponent sacrifices a creature when one of yours dies"
```

`dsl = []` gives the card no abilities. The card then gets the AI hint tags its abilities imply (`rem=`, `tut=`,
`ctr=` ...). Card *code* (hooks, options, AI priorities) is Rust in `crates/sim-core/src/impls/`.

## Edit a deck

Your decks are `decklists/<owner>/<deck>.md`, the outside decks `decklists/pool/<tier folder>/<deck>.md`; the cards are
the file's `## Import list` block. Either edit the file, or paste a whole new list:

```
cardgen deck sauron new-list.txt --dry-run         # what would change, nothing written
cardgen deck sauron new-list.txt --log "Why."      # rewrite the file; logs "Out: … In: … Why." in decklists/CHANGELOG.md
cardgen deck yuriko-dimir-ninjas -                 # paste the list on stdin
```

The key is one of your deck keys (`data/my_decks.toml`), a pool deck's file name, or a path. The list may come from
most deck sites (`1 Card`, `1x Card`, set codes and foil marks are ignored; everything after `Sideboard` is ignored).
It checks 100 cards, singleton, the commander, every name on Scryfall, colour identity, bans and the tier's Game
Changer rule before touching a file; it rewrites the import list and the by-type section, a pool deck's Game
Changers and Lands lines, records your decks' new lists in `tests/fixtures/my_decks_parsed.json`, and reports how well
the new cards are modeled and where the file's prose still mentions cards that left. Then rebuild the data:

```
cardgen export
```

`cardgen validate` runs the same checks over every pool deck (36 in seven tiers).

## Rebuild

`cardgen export` writes both files (`--out DIR` writes elsewhere). The output is deterministic: the same inputs give
the same bytes. After any change to a data file, a deck list or the cache, re-export and commit the JSON. The test
`export_matches_committed_files` fails when `data/*.json` is stale, and `deck_guard_fixture_is_current` when the deck
guard fixture is.

## Audit

```
cardgen audit                      # per pool deck: cards by status, key engines that are fully modeled
cardgen audit --deck yuriko        # every card of one deck (a key prefix is enough)
cardgen audit --mine               # your decks: every card that is not Full
cardgen audit --md FILE            # also write the tables as Markdown
cardgen audit --hooks [--all]      # what is left to port: events the old Python code handled that sim-core's registry
                                   # doesn't (cards the decks play, or every card)
```

Statuses: Full (hand-verified: an override's `status`, `documents/card-audit.md`, `[card_notes]` in audit.toml),
Full-auto (every Oracle line compiled), Approximate, Partial, Unmodeled.

## Scryfall

`data/scryfall_cache.json` uses the format lower-case name -> slim record. Lookups
follow Scryfall's API guidelines (User-Agent, at most 10 requests a second, batches of 75, everything cached) and only
happen when a listed card is missing from the cache. The tests never use the network.

## Equivalence

`cardgen export` once reproduced the old Python exporter's `data/cards.json` and `data/decks.json` byte for byte
(1,767 cards, 32 decks); the test `tests/equivalence.rs` now checks that the committed files are what `export` writes
(re-run `cardgen export` after editing data). The data files `carddb.txt`, `card_overrides.toml`, `audit.toml` and
`my_decks.toml` were extracted from the Python code once and are now the source.

## Layout

```
src/dsl_parse.rs   Oracle text -> abilities           src/autotag.rs   regex tagger (cards that don't compile)
src/dsl.rs         AI hint tags, build a card         src/cd.rs        a card definition (the card definition)
src/sources.rs     the build order above              src/cardinfo.rs  Phyrexian mana, land enters rules / types
src/export.rs      cards.json / decks.json            src/scryfall.rs  the cache and the client
src/decks.rs       deck files, pool, validation       src/update_deck.rs, src/audit.rs
src/json.rs, re.rs, py.rs   JSON, regex and string behaviour (ordered objects, byte-exact output)
```
