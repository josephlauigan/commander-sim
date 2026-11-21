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
             [--sort edhrec|name|cmc|usd] [--limit N|all] [--offset N] [--count] [--json] [--full] [--data-dir DIR]
```

Searches every Oracle card in `data/knowledge/oracle.json` (written by `bulk-sync`); read-only, offline. The catalog is
loaded into memory (a typed serde read, no persisted index, about 0.1 s) and every query is a linear scan. All filters are
optional and combine with AND.

| Flag | Meaning |
|---|---|
| `--name N` | the card's name, exact; case, accents and apostrophe style do not matter (see below); a face name works too (`Petty Theft` finds Brazen Borrower). An exact full name beats a face name: `Lightning Bolt` is the instant, not the `prepare` card `Emeritus of Conflict // Lightning Bolt` that has it as a face. Several cards with one name (15 names are shared): the Commander-legal one first |
| `--name-contains S` | substring of the full name or of any face name (same folding, but the value is not trimmed: `" of "` keeps its spaces; an empty value is a usage error) |
| `--ident WU` | colour identity limit: the card's identity must be a **subset** of the letters (a colourless card fits any limit; `WU` admits W, U, WU and colourless). `C` or an empty value means colourless only |
| `--legal` | Commander legality: `legal` (default), `banned`, `not_legal`, `restricted`, or `any` |
| `--cmc '<=2'` | mana value; operators `<`, `<=`, `=`, `>=`, `>`, a plain number means `=` (quote `<` and `>` in the shell) |
| `--type WORD` | a word of the type line of any face (`instant`, `Elf`, `legendary`; a word, not a substring); repeatable, all must match; a value with no word (`""`, `"—"`) is a usage error |
| `--text REGEX` | fancy-regex over the Oracle text of every face, case-insensitive unless the pattern says `(?-i)`; repeatable, all must match; a bad pattern exits non-zero |
| `--gc` | Game Changers only |
| `--deck KEY` | a deck of `data/decks.json` (yours, then the pool; an unknown key lists the valid ones). Its `ident` is the identity ceiling: `--ident` may only narrow it (an `--ident` outside the deck's identity is a usage error, `--ident U is outside deck cloud's identity W`). Leaves out every card of the deck (its list and its commanders, matched by name and by face name) |
| `--meld` | include meld results (Brisela, Hanweir, ...), which every search except `--name` leaves out by default (an exact `--name` shows them) |
| `--sort` | `edhrec` (default: ascending, unranked last), `name`, `cmc`, `usd` (ascending, no price last); ties by name |
| `--limit N\|all`, `--offset N` | page through the result (default limit 30; `--limit all` is no limit; `--limit 0` is a usage error, use `--count`) |
| `--count` | print only the number of matches (ignores `--limit`, `--offset` and `--json`) |
| `--json` | a bare array, one object per line, always the same complete row: `name, oracle_id, layout, mana_cost, cmc, type_line, identity, legal, gc, edhrec_rank, usd, oracle_text, keywords, set, released_at, rarity, reserved`, plus `faces, power, toughness, loyalty, produced_mana, meld_parts` when they apply. `identity` is a string such as `"WU"` (`""` = colourless); `mana_cost` is the card's own, else its face costs joined with ` // ` (`""` for a land); `oracle_text` is the text the regex sees: the card's faces joined by `\n//\n`; `edhrec_rank` and `usd` are `null` when absent. See "The JSON contract" below |
| `--full` | switches the text output to one readable block per card (every face, keywords, produced mana, meld parts, `oracle_id`); the JSON row is the same with or without it |
| `--data-dir DIR` | the knowledge directory |

Names are compared by a folded search key (`fold` in `knowledge.rs`): lower-case, accents and other combining marks
dropped, every apostrophe variant (`’ ‘ ʼ` `` ` `` `´`) a plain `'`, `Æ`/`Œ`/`ß` spelled `ae`/`oe`/`ss`, the ends trimmed. So
`Anduril, Narsil Reforged`, `Lim-Dul's Vault`, `Urza’s Saga` and `Æther Vial` find the catalog's `Andúril...`,
`Lim-Dûl's Vault`, `Urza's Saga` and `Aether Vial`. Hyphens, commas and inner spaces stay significant. The same key is used
for `--name-contains` and for `--deck` (a deck list and the catalog may spell a name either way); the output always shows
the catalog's spelling.

A flag that takes a value rejects a value that starts with `--` (`--name --json` is "needs a value", not a search for
`--json`); negative numbers are fine, and a regex that must start with `--` can be written `[-]-`. On load, a record with an empty name, no `oracle_id` or an `oracle_id` already used by another record
is an error that names the record and suggests `cardgen bulk-sync` (shared names with different `oracle_id`s are normal).

`--text` runs on the Oracle text with the card's faces joined by a line holding only `//`, so a multi-face card is one
string (`(?m)^//$` matches the joiner; two `--text` patterns may match different faces); the card's name and type line are
not searched. Table columns are padded by display width (wide characters count 2, combining marks 0).

The default output is a table: name, cost, type line, EDHREC rank, price, legality.

**The JSON contract.** Keys are only ever added, never renamed or removed, and a consumer ignores keys it does not know
(later phases add `roles`, `primary`, `status`, `decks`, `findings`). There is no compact form: the digest is the compact
view, `--json` is the detail.

**Notes.** A result never silently leaves something out. `find` also prints notes, to stderr for `--json` and `--count`
(so stdout stays one parseable value) and as trailing lines for a table:

- `(showing 2-3 of 9; --offset 3 or --limit all)` when a page is cut;
- `(1 card(s) named Primeval Titan exist outside these filters: banned; try --legal any)` when `--name` found cards but the
  other filters (legality, `--deck`, `--ident`, ...) removed them all;
- `(deck cloud: 2 name(s) not in the catalog: ...)` when a deck list names a card the catalog does not have (it could not
  be left out of the candidates);
- `(1 meld result(s) are hidden; add --meld)` when nothing matched and a meld result would have.

The knowledge directory is `--data-dir`, else `CARDGEN_KNOWLEDGE_DIR`, else `<repo>/data/knowledge`; when the default has
no `oracle.json` (a worktree has the directory, because `sync.lock` is tracked, but not the generated file) it falls
back to the main checkout's (`<git-common-dir>/../data/knowledge`). A missing catalog is an error that names
`cardgen bulk-sync`. A `--data-dir` or `CARDGEN_KNOWLEDGE_DIR` without an `oracle.json` is an error too, with no fallback.
An unknown flag (`--digest` and `--map` do not exist yet), a flag without its value or a non-numeric number prints
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

For code, `knowledge.rs` exports the same engine: `Index::open`, `by_oracle_id`, `lookup`, `search` (everything `find` does before it prints: the deck's
identity ceiling, the deck's own cards left out, the filters, the sort), `sort`, `deck_members`, `subtype_vocab`
(`Time Lord` is one subtype), `Query::default()` (no limit; `find` parses to 30) and `Card::{cost, lines, subtypes, self_names}` (`lines` numbers the Oracle lines without
the `//` joiner, so evidence indices and digest gists cite the same line).

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

## Roles (offline rule classifier)

```
cardgen roles --rules [--data-dir DIR]
cardgen roles --validate [--gold PATH] [--json] [--data-dir DIR]
cardgen roles --show "Ashnod's Altar" [--data-dir DIR]
cardgen roles --stats [--data-dir DIR]
cardgen roles --audit [--seed N] [--n 80] [--rule ID] [--certain] [--data-dir DIR]
cardgen find --role ramp --role sac-outlet --json
```

`roles --rules` reads the catalog without network access and atomically writes `roles.json` beside `oracle.json`.
Stickers and Attractions remain searchable in Oracle but are omitted from the role table and audit samples.
`--stats` reports their skipped count; routing denominators include only deck cards.
The JSON object is sorted by Oracle id and byte-deterministic. Each assignment has up to four ordered `roles`,
`primary` (the first role, or `""`), per-role `evidence` (`source`, rule id, `certain`, `face`, `line`), and `needs_llm`.
Evidence uses zero-based indices into `Card::lines()`; `face: null` means the card's top-level Oracle text.
`line` is the global line index without face separator lines. `--show` prints the original supporting lines.

The 26 definitions, sources and future LLM evidence guards live in `data/knowledge/roles.toml` (version 1).
Roles describe main or repeatable functions, with the conventions in `roles_gold.toml` taking precedence over
keyword matching. Reminder text is stripped, self names become `~`, land backs of MDFCs add no roles, prepare cards
use their spell face, and expensive planeswalker ultimates add no roles. Basic land types confer their intrinsic
mana abilities even when printed only as a parenthesized synopsis. Printed self evasion/protection gives no role;
printed lifelink is lifegain, listed last. Rules also recognize semantic keyword actions such as recruit, connive,
transmute, mobilize and afterlife without treating their reminder text as independent abilities. Repeated modal
abilities pass their trigger context to their own bullet lines. At the four-role ceiling, a copy's redundant token
role yields to an independent function.
Ward granted to other permanents is protection; a lifelink-only team grant is lifegain, not anthem. Creature,
Treasure and Clue tokens count; other artifact tokens are judged by their abilities. Repeatable Clue makers also
give draw. Team +1/+1 counter placement is counters and anthem; player counters are excluded, but proliferate
counts. Amass gives tokens, plus counters when repeatable. Returning noncreature permanents from your graveyard
to the battlefield is recursion; unrestricted permanent returns also give reanimate, which requires creatures.

Unrecognized lines, overflowing role sets and potentially incidental selection, recursion, graveyard hate,
finishers, stax, drain, copy, evasion and protection route to a future LLM stage. Removal with several clauses also
routes there. `--stats` and validation report the Commander-legal routing share. Certain assignments are limited to
audited direct templates: nonland single-color tap mana, direct spell counters, explicit nonland searches to hand,
draw two/three/four, and printed standalone lifelink. A certain role does not exempt the card's other effects from
routing. This command never calls an LLM; a pattern rejected as incidental still recognizes the line.

`--audit` freshly classifies a seeded sample without replacement of Commander-legal catalog cards outside the
project's `roles_gold.toml`, then prints complete Oracle lines, predicted roles, evidence and routing. It neither
reads saved assignments nor applies overrides, and writes no files. Defaults are seed 0 and 80 cards. Oracle ids
are ranked with a fixed FNV-1a hash mixed with the seed through SplitMix64, so catalog input order is irrelevant.
`--rule ID` samples that rule's assignments; `--certain` restricts to certain assignments (both filters apply to the
same evidence). If fewer than N qualify, every qualifying card is shown. The footer reports actual cards and
role assignments, including the rules-final subset, for a manual precision/recall audit.

Hand overrides in `data/knowledge/roles_overrides.toml` replace the entire role set, including certain rules;
the file documents the `[[card]]`, `name`, `roles`, `note` format. Overrides intentionally do not improve validation.
An override's evidence has source `override`, its note in `rule`, and no face or line, since it need not have a
regex justification. No initial overrides are installed.

`find` automatically attaches `roles.json` when present beside the catalog. Repeatable `--role R` filters combine
with AND. An unknown id is a usage error listing all 26 ids; a missing roles file names `cardgen roles --rules`.
JSON rows gain `roles` and `primary` only when the side table is attached. The Oracle catalog and `Card` remain
immutable; programs use `Index::attach_roles`, `Index::roles` and `Query.roles`. Missing ids, invalid evidence or
role sets, and deck-card count mismatches are errors rather than silently incomplete searches; regenerate after sync.

Validation reclassifies without overrides and reports per-role and micro recall against the engine's manual/pool
cards in owned decks, per-role precision and recall against independent gold labels (all and certain separately),
forbidden-role violations, exact missed/unexpected pairs, and routing by legality and layout. Every engine tag is
listed with its fixed translation or reason for leaving it unmapped. Tags describe simulator approximations and
are not exhaustive; only recall is measured against them. `--gold` permits an independently supplied gold set.
JSON validation uses integer `tp`, `fp`, `missed` counts; precision is `tp/(tp+fp)`, recall is `tp/(tp+missed)`;
a zero denominator is 1. Text output includes those ratios. Nonzero exit means an acceptance threshold failed:
hand recall 0.95 (0.85 for roles with at least ten pairs), gold micro precision 0.90 and recall 0.85 for the 19 rules
roles, certain precision 0.95 across every role, and zero certain forbidden-role violations. The rules-final subset
(`needs_llm == false`, reclassified without overrides) additionally requires precision 0.93 and recall 0.85 across
**all** roles, including those normally supplemented by the LLM. Its card count and integer micro counts are in
both reports. Other roles outside that subset are reported without precision/recall gates. Routing counts are
reported, not hidden by a threshold.

CI uses real Oracle text frozen in `tests/fixtures/roles_oracle.json`, with positive and decoy examples for every
rule. Full-catalog tests are opt-in: `cargo test --release -p cardgen --test roles_real -- --ignored` (requires the
local catalog). Run Cargo and test binaries under the repository memory cap.
