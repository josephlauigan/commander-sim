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

## Bulk sync (the card knowledge store)

```
cardgen bulk-sync [--from-file PATH] [--data-dir DIR] [--min-count N]
```

Downloads Scryfall's "Oracle Cards" bulk file (one record per Oracle card; a gzip file of JSON Lines, about 25 MB,
160 MB unpacked; the `jsonl_download_uri` and `updated_at` come from `api.scryfall.com/bulk-data`, type `oracle_cards`;
only an `https://` URI is downloaded) and writes, in `data/knowledge/` (override with `--data-dir` or
`CARDGEN_KNOWLEDGE_DIR`):

- `oracle.json` (generated, gitignored): `{"meta": {schema, bulk_updated_at, download_uri, synced, count}, "cards":
  [...]}`, one slim record per line, sorted by lower-case name (ties: oracle_id, name, record text), so the same bulk
  file gives the same bytes apart from `meta.synced`. A record is `scryfall::slim` (the cache's 15 fields, unchanged)
  plus `oracle_id`, `edhrec_rank` (absent when null), `usd` (`usd_foil` only when there is no `usd`; absent when
  neither), `released_at`, `set`, `rarity`, `reserved`, a `colors` list on each face, and on a meld result
  `all_parts_meld` (true) and `meld_parts` (the names of the cards that meld into it). A meld result is found by id or
  by name: the ids in `all_parts` are printing ids and the file keeps one arbitrary printing per Oracle card, so the
  id can differ from the card's own (Ragnarok, Divine Deliverance). `set` and `released_at` describe Scryfall's chosen
  printing, **not** the first printing; detect new cards by `oracle_id`.
- `oracle.prev.json` (generated, gitignored): the previous sync. It is a hard link (a copy if linking fails) made
  before the new file replaces `oracle.json`, so it is the generation just before the current one; it is gone after a
  rolled-back sync, and the next successful sync makes it again.
- `sync.lock` (tracked): `schema=1`, `bulk_updated_at`, `count` and `fnv1a64`, the FNV-1a-64 (hex) of the text of
  `oracle.json`'s `cards` array (from its `[` to its `]`). A lock without the current `schema=` line is invalid and
  causes a normal re-sync.

Dropped: layouts `token`, `double_faced_token`, `emblem`, `art_series`, `scheme`, `planar`, `vanguard` and
`front_card` (memorabilia cards: type "Card", all `not_legal`, and 13 of them repeat a real card's name); a card whose
type line, or any face's, starts with the word `Token`, `Card` or `Dungeon` (Role tokens that have a card layout, helper
cards such as Experience, Dungeons); and a repeated `oracle_id` (the first wins). Everything else is kept with its
commander legality, banned and not-legal cards and un-cards included. Digital-only printings are kept: the one printing
the file holds for an old staple is often a digital Masters set, so legality, not the `digital` flag, decides
playability. Nothing else branches on a layout name, and an unknown layout is a warning (the cards are kept). The
summary prints kept and dropped per layout, and the drops by layout, type and duplicate.

`up to date` (and no download) needs all of these to agree: the bulk file's `updated_at`, `oracle.json`'s meta, and
`sync.lock` (`bulk_updated_at` and `count`), and the lock's hash must equal the one recomputed from `oracle.json`. A
missing, corrupt or stale file means a normal re-sync, never a crash or a silent "up to date". Otherwise it streams the
file to a temp file (a dedicated ureq agent: 60 s read timeout, no total cap), decompresses and parses it one card at a
time (flate2; a JSON array or JSON Lines, plain or gzip, told by the bytes), refuses fewer than 25,000 cards
(`--min-count N` lowers this for fixtures), and writes `oracle.json` and `sync.lock` as staged temp files
(`oracle.<pid>-<n>.new.tmp`, `sync.<pid>-<n>.lock.tmp`, so concurrent runs do not share them), then renames them into
place. If the lock cannot be put in place the new oracle is rolled back (the error says whether the old one was
restored). An existing read-only `sync.lock` is refused before anything is touched. Any failure leaves `oracle.json`
byte-identical (unless the rollback itself fails, which the error says). `--from-file PATH` parses a local bulk file (either format) instead (no network, no up-to-date check;
`bulk_updated_at` is recorded as `from-file`, so it overwrites the lock: it warns on stderr when no `--data-dir` or
`CARDGEN_KNOWLEDGE_DIR` is given, and `--data-dir` points a scratch run elsewhere). `tests/bulk_sync.rs` runs on a 51-card fixture (`tests/fixtures/bulk_fixture.json`) and never
uses the network.

## Find (search the whole catalog)

```
cardgen find [--name N] [--name-contains S] [--ident WU] [--legal legal|banned|not_legal|any] [--cmc '<=2']
             [--type WORD]... [--text REGEX]... [--gc] [--deck KEY] [--meld]
             [--sort edhrec|name|cmc|usd] [--limit N] [--offset N] [--count] [--json] [--full] [--data-dir DIR]
```

Searches every Oracle card in `data/knowledge/oracle.json` (written by `bulk-sync`); read-only, offline. The catalog is
loaded into memory (a typed serde read, no persisted index, about 0.1 s) and every query is a linear scan. All filters are
optional and combine with AND.

| Flag | Meaning |
|---|---|
| `--name N` | the card's name, exact and case-insensitive; a face name works too (`Petty Theft` finds Brazen Borrower). An exact full name beats a face name: `Lightning Bolt` is the instant, not the `prepare` card `Emeritus of Conflict // Lightning Bolt` that has it as a face. Several cards with one name (15 names are shared): the Commander-legal one first |
| `--name-contains S` | substring of the full name or of any face name |
| `--ident WU` | colour identity limit: the card's identity must be a **subset** of the letters (a colourless card fits any limit; `WU` admits W, U, WU and colourless). `C` or an empty value means colourless only |
| `--legal` | Commander legality: `legal` (default), `banned`, `not_legal`, `restricted`, or `any` |
| `--cmc '<=2'` | mana value; operators `<`, `<=`, `=`, `>=`, `>`, a plain number means `=` (quote `<` and `>` in the shell) |
| `--type WORD` | a word of the type line of any face (`instant`, `Elf`, `legendary`; a word, not a substring); repeatable, all must match |
| `--text REGEX` | fancy-regex over the Oracle text of every face, case-insensitive unless the pattern says `(?-i)`; repeatable, all must match; a bad pattern exits non-zero |
| `--gc` | Game Changers only |
| `--deck KEY` | a deck of `data/decks.json` (yours, then the pool; an unknown key lists the valid ones). Uses its `ident` as the identity limit (unless `--ident` is given) and leaves out every card of the deck (its list and its commanders, matched by name and by face name) |
| `--meld` | include meld results (Brisela, Hanweir, ...), which are left out by default because the pair that makes them is not one card |
| `--sort` | `edhrec` (default: ascending, unranked last), `name`, `cmc`, `usd` (ascending, no price last); ties by name |
| `--limit N`, `--offset N` | page through the result (default limit 30) |
| `--count` | print only the number of matches (ignores `--limit`, `--offset` and `--json`) |
| `--json` | an array of `{name, mana_cost, cmc, type_line, identity, legal, gc, edhrec_rank, usd, oracle_text, faces?, set, released_at, rarity}`; `identity` is a string such as `"WU"` (`""` = colourless), `mana_cost` and `oracle_text` are `""` when the card has none (a multi-face card's text is in `faces`), `edhrec_rank` and `usd` are `null` when absent |
| `--full` | the complete record: adds `keywords`, `produced_mana`, `power`, `toughness`, `loyalty`, `oracle_id`, `reserved`, `meld_parts` (and per-face power, toughness, loyalty); without `--json` it is readable text, one block per card |
| `--data-dir DIR` | the knowledge directory |

The default output is a table: name, cost, type line, EDHREC rank, price, legality.

The knowledge directory is `--data-dir`, else `CARDGEN_KNOWLEDGE_DIR`, else `<repo>/data/knowledge`; when the default has
no `oracle.json` (a worktree has the directory, because `sync.lock` is tracked, but not the generated file) it falls
back to the main checkout's (`<git-common-dir>/../data/knowledge`). A missing catalog is an error that names
`cardgen bulk-sync`. A `--data-dir` or `CARDGEN_KNOWLEDGE_DIR` without an `oracle.json` is an error too, with no fallback.
An unknown flag (`--role`, `--digest` and `--map` do not exist yet), a flag without its value or a non-numeric number prints
the usage text and exits non-zero.

Examples:

```
cardgen find --ident WU --cmc '<=2' --type instant --text 'counter target' --limit 10
cardgen find --deck cloud --text 'search your library' --json --limit 20     # candidates for the Cloud deck
cardgen find --deck cloud --count                                            # how many cards a digest would cover
cardgen find --name "Sol Ring" --full                                        # layer 3: everything about one card
cardgen find --legal any --name "Primeval Titan"                             # banned, and why it is not in a default search
cardgen find --gc --count
```

`--deck KEY --count` is the size of a deck's candidate pool: the Commander-legal cards in its colours that it does not
already play. It is the number to size a digest against.

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
src/bulk.rs        bulk-sync (Scryfall Oracle Cards -> data/knowledge/oracle.json)
src/knowledge.rs   the in-memory card index and `find` (queries oracle.json)
src/json.rs, re.rs, py.rs   JSON, regex and string behaviour (ordered objects, byte-exact output)
```
