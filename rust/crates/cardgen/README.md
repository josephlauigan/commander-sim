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
- `sync.lock` (generated, gitignored since the weekly job: a tracked file would be rewritten by every new bulk file):
  `schema=1`, `bulk_updated_at`, `count` and `fnv1a64`, the FNV-1a-64 (hex) of the text of `oracle.json`'s `cards`
  array (from its `[` to its `]`). A lock without the current `schema=` line is invalid and causes a normal re-sync.
  The bulk `updated_at` also lives in `oracle.json`'s `meta`, so the lock is a cross-check, not the only record.

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
corrupt or stale file, or a lock that is present but disagrees, means a normal re-sync, never a crash or a silent "up to
date". The lock itself may be absent (a new checkout; the merge that untracked it deletes it): then `oracle.json` is
trusted only if its `cards` array parses and has `meta.count` records, and the lock is written again from it, so the
previous sync in `oracle.prev.json` is not replaced by a copy of the current one. Otherwise it streams the
file to a temp file (a dedicated ureq agent: 60 s read timeout, no total cap), decompresses and parses it one card at a
time (flate2; a JSON array or JSON Lines, plain or gzip, told by the bytes), refuses fewer than 25,000 cards
(`--min-count N` lowers this for fixtures), and writes `oracle.json` and `sync.lock` as staged temp files
(`oracle.<pid>-<n>.new.tmp`, `sync.<pid>-<n>.lock.tmp`, so concurrent runs do not share them), then renames them into
place. If the lock cannot be put in place the new oracle is rolled back (the error says whether the old one was
restored). An existing read-only `sync.lock` is refused before anything is touched. Any failure leaves `oracle.json`
byte-identical (unless the rollback itself fails, which the error says). `--from-file PATH` parses a local bulk file (either format) instead (no network, no up-to-date check;
`bulk_updated_at` is recorded as `from-file`, so it overwrites the lock: it warns on stderr when no `--data-dir` or
`CARDGEN_KNOWLEDGE_DIR` is given, and `--data-dir` points a scratch run elsewhere). `--from-file PATH --updated-at
STAMP` lets the local file stand in for Scryfall's bulk entry at that date (`bulk::LocalRemote`): the up-to-date check,
the lock and `oracle.prev.json` behave as for a download, with no network (the weekly job's offline smoke test).
`tests/bulk_sync.rs` runs on a 51-card fixture (`tests/fixtures/bulk_fixture.json`) and never
uses the network; `tests/weekly_sync.rs` covers the untracked lock and `--updated-at`.

## Find (search the whole catalog)

```
cardgen find [--name N] [--name-contains S] [--ident WU] [--legal legal|banned|not_legal|any] [--cmc '<=2']
             [--type WORD]... [--text REGEX]... [--gc] [--deck KEY] [--meld]
             [--status LIST] [--in-deck KEY] [--not-in-deck KEY]
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
| `--status LIST` | the engine's model status (see "Engine facts" below): a comma-separated list of `full`, `full-auto`, `approximate`, `partial`, `unmodeled` and `no-record` (no engine record; `none` is an alias), any case; an unknown word is a usage error. `--status full,full-auto` = cards the sim models with no known gap (Full-auto is compiled, not hand-verified); every proposal still needs the three swap checks and a `--swap` run, and a card below Full (Full-auto, Approximate, Partial, Unmodeled) or with no record goes to JD first |
| `--in-deck KEY` / `--not-in-deck KEY` | the card is (is not) in a deck of `data/decks.json` `mine`, by the same name matching as `--deck`; an unknown key (a pool deck's too) is a usage error that lists the keys of `mine`. Each is given once |
| `--meld` | include meld results (Brisela, Hanweir, ...), which every search except `--name` leaves out by default (an exact `--name` shows them) |
| `--sort` | `edhrec` (default: ascending, unranked last), `name`, `cmc`, `usd` (ascending, no price last); ties by name |
| `--limit N\|all`, `--offset N` | page through the result (default limit 30; `--limit all` is no limit; `--limit 0` is a usage error, use `--count`) |
| `--count` | print only the number of matches (ignores `--limit`, `--offset` and `--json`) |
| `--json` | a bare array, one object per line, always the same complete row: `name, oracle_id, layout, mana_cost, cmc, type_line, identity, legal, gc, edhrec_rank, usd, oracle_text, keywords, set, released_at, rarity, reserved`, plus `faces, power, toughness, loyalty, produced_mana, meld_parts` when they apply, and `roles`, `primary` when a current role table is present, and the engine facts `decks`, `findings` (always present, possibly empty) and `status`, `status_note` (only when the card has an engine record). `identity` is a string such as `"WU"` (`""` = colourless); `mana_cost` is the card's own, else its face costs joined with ` // ` (`""` for a land); `oracle_text` is the text the regex sees: the card's faces joined by `\n//\n`; `edhrec_rank` and `usd` are `null` when absent. See "The JSON contract" below |
| `--full` | switches the text output to one readable block per card (every face, keywords, produced mana, meld parts, `oracle_id`, then the engine status with its note, the decks of `mine` that play it and each `FINDINGS.md` line that names it); the JSON row is the same with or without it |
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

The default output is a table: name, cost, type line, EDHREC rank, price, legality (no engine columns: use `--json`, `--full` or the filters).

**The JSON contract.** Keys are only ever added, never renamed or removed, and a consumer ignores keys it does not know
(phase 3 added `roles` and `primary`; phase 4 added `decks`, `findings`, `status` and `status_note`, below). There is no compact form: the digest is the compact
view, `--json` is the detail.

**Engine facts (phase 4).** `cardgen::facts` joins three facts, resolved to catalog card indices once and shared by `find`,
the digest and the map (so they cannot disagree):

- `status`, `status_note`: the engine's model status of the card (`Full` hand-verified, `Full-auto` compiled from Oracle
  text, `Approximate`, `Partial`, `Unmodeled`, as in `documents/card-audit.md`) and its note. They come from the code
  `cardgen audit` runs (`Audit::card_status`) for every card that has a record in `data/cards.json` (the records
  `cardgen export` builds from the same inputs); a card of any deck of `mine` is audited as JD's decks play it
  (`mine = true`, as `audit --mine` does), every other card as an outside deck plays it. The keys are **absent** for a card
  with no engine record: such a card has an unknown status, never `Full`: treat it as below Full (`--status no-record`;
  `--full` prints `engine status: NO RECORD`). The status is computed from the current data files, which is what the next
  `cardgen export` will give, while `data/cards.json` decides which cards have a record (see "Unavailable engine facts").
  `--full` words `Full-auto` as `Full-auto (compiled, not hand-verified)`. Every row also has the always-present key
  `engine`: `record`, `no-record` or `unavailable`, so a card with no record and a row printed while the export is
  unavailable cannot be confused. Engine
  names are resolved to catalog cards once, by folded name, face name and double-faced `A // B` name; a name that several
  catalog cards share maps to the Commander-legal one; when two engine records resolve to one catalog card (`A` and
  `A // B` of a double-faced card; nine today, five with different statuses) the lower status is kept (conservative), so a
  card is never shown above any of its records; engine names
  that match no catalog card are left out (the facts keep both lists). A project without `data/carddb.txt` has no engine records.
- `decks`: the keys of the decks of `mine` whose list or commanders name the card (folded name or face name, basic lands
  included), in the order of `decks.json`.
- `findings`: the lines of `audit/<key>/FINDINGS.md` of each deck of `mine` that mention the card, each `{file, line,
  heading, text}` (`file` relative to the repository, `line` from 1, `heading` the nearest heading line (starts with `#`)
  before the line, empty when none, `text` the line cut to 200 characters). A line mentions a card when the card's full
  name or any face name occurs word-bounded and case-sensitive, spelled as in the catalog or with accents folded and
  apostrophes plain; a name of 4 characters or fewer (`Opt`) only counts inside `**...**`, inside a backtick span or on a
  `=>` swap line, so `Option` never mentions `Opt`. A name shared by several cards, and a face name that is also another
  card's full name, mention the card `--name` would pick (the legal card; the exact name beats a face name). Each line
  is listed once per card. At each position the longest matching name wins and the names inside it are skipped, so
  `Sheoldred's Edict` or `Sheoldred, the Apocalypse` does not also mention the face `Sheoldred` of a double-faced card;
  a bare `Sheoldred` still mentions that card (commander shorthand is a known false hit of `k:`).

**Unavailable engine facts.** The exported `data/cards.json` is the engine's record set, and a status exists only for
names it contains. When `data/carddb.txt` exists but `data/cards.json` is missing, unreadable, not valid JSON, not the
shape `{"version": .., "cards": [..]}`, or its card names differ from the set the data files rebuild (a stale export,
even by one card), the engine facts are unavailable: a plain `find --json` or `--full` still prints decks and findings,
leaves out `status` and `status_note` and writes one stderr line, `(engine status unavailable: <reason>; run cardgen
export)`; `find --status ...` fails with that message and a non-zero exit; the digest prints the same line in its header
and no `m:` flags; the map's engine section says so and the deck columns show `-`. No read command regenerates the
export. A project with no `data/carddb.txt` has no engine data and says nothing. An unreadable data file (a broken
`audit.toml`) is an error, not "unavailable".

Decks: the membership bits, `decks`, the findings and `--in-deck`/`--not-in-deck` cover every deck of `mine` up to 64
(more is an error naming the limit). The digest's `d:` letters are `A`-`Z`, then `a`-`z`, so it names at most 52 decks;
more than 52 decks of `mine` is an error for the digest only. A mention inside a code span counts for a short name when
the span is a CommonMark one (a run of backticks closed by a run of the same length). One leading BOM is ignored; the
finding `text` is the physical line as written (trailing spaces kept) cut to 200 characters. A non-Unicode command-line
argument is a usage error (exit 1), never a panic.

The digest's `k:N` is the number of `findings` of a card. The engine is built only when the output or a filter needs it
(`--json`, `--full`, `--status`: about 0.2 s more); a plain table search does not touch it.

**Notes.** A result never silently leaves something out. `find` also prints notes, to stderr for `--json` and `--count`
(so stdout stays one parseable value) and as trailing lines for a table:

- `(showing 2-3 of 9; --offset 3 or --limit all)` when a page is cut;
- `(1 card(s) named Primeval Titan exist outside these filters: banned; try --legal any)` when `--name` found cards but the
  other filters (legality, `--deck`, `--ident`, ...) removed them all;
- `(deck cloud: 2 name(s) not in the catalog: ...)` when a deck list names a card the catalog does not have (it could not
  be left out of the candidates);
- `(1 meld result(s) are hidden; add --meld)` when nothing matched and a meld result would have.

The knowledge directory is `--data-dir`, else `CARDGEN_KNOWLEDGE_DIR`, else `<repo>/data/knowledge`; when the default has
no `oracle.json` (a worktree usually has the directory, because the role files in it are tracked, but never the
generated file; the code does not need the directory to exist, since `sync.lock` is untracked) it falls back to the
main checkout's (`<git-common-dir>/../data/knowledge`). A missing catalog is an error that names
`cardgen bulk-sync`. A `--data-dir` or `CARDGEN_KNOWLEDGE_DIR` without an `oracle.json` is an error too, with no fallback.
An unknown flag, a flag without its value or a non-numeric number prints
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

## Map and digest (the catalog for an agent)

```
cardgen map [--out PATH] [--stdout] [--data-dir DIR]
cardgen digest (--deck KEY | --ident LETTERS) [--role R | --view roleless|new] [--limit N] [--offset N]
               [--out PATH] [--budget-tokens N] [--stdout] [--data-dir DIR]
```

Layers 1 and 2 of `documents/card-knowledge.md`. Both are read-only except for the file they write, offline, and never
need the LLM: they read `oracle.json` and whichever `roles.json` is present (`cardgen roles --rules`, or the merged table
after `roles --merge`) and say which kind of table they used. A missing catalog or role table is an error that names
`cardgen bulk-sync` or `cardgen roles --rules`. The file is always written (a uniquely named temporary file in the same
directory, renamed over the target, removed on every error, so concurrent writers all succeed); the one-line summary
(path, bytes, estimated tokens) goes to stdout, or with `--stdout` the text goes to stdout and the summary to stderr.
The files are generated views and git-ignored.

**`map`** writes `data/knowledge/map.md` (default; `--out` or `--data-dir` change it), about 3k tokens and never over
32 KB: the bulk date and the counts (legal, banned, not legal, Game Changers), Commander-legal cards per colour
identity (all 32 combinations), cards per role by mono-colour, colourless and multicolour (and per primary role), role
coverage and review status (how many legal cards the rules decided alone, how many are routed to the LLM, how many of
those have an answer and how many are still unreviewed), engine model coverage (cards of the map's set by status: Full, Full-auto, Approximate, Partial, Unmodeled, and no engine record), the 15 biggest creature types (subtypes of creature-bearing
faces only, once per card), the 40 biggest subtypes of every kind and the 15 biggest keywords, the Game Changers, the
banned cards, a table of JD's decks (key, commander, identity, tiers, how many list cards the catalog does not call
Commander-legal, the cards with each role, how many of its cards are below Full in the engine (`f`/`A`/`P`/`U` counts) or have no record, and how many `FINDINGS.md` lines mention its cards), the combos in JD's decks (card names, from `data/spellbook/`) and the
commands to go further, with a paragraph on using the digest for a swap. Every count is over the cards `find` and
`digest` cover: Commander-legal, not a meld result, not a Sticker or Attraction. No timestamp, only the bulk file's
`updated_at`.

**`digest`** writes `data/knowledge/digests/KEY.txt` (`ident-WUB.txt` for `--ident`, `KEY.ROLE[.OFFSET].txt` or
`KEY.VIEW[.OFFSET].txt` for a page): one line per card, `Name|{cost}|Type—Subtypes|roles|gist|flags`, six `|`-separated
fields, at most 160 characters (a `|` inside Oracle text is written `/`; a name too long for its line is cut at a word
and ends in `…`).

| Field | Content |
|---|---|
| type | per face (joined with `/`, written once when the faces agree): `Lg` for a legendary face, then a letter per card type in the order A E L C P B I S (`AC` artifact creature, `LgC` legendary creature, `C/L` an MDFC with a land back, `O` other), then `—` and the subtypes (they give way first when the line is too long) |
| roles | the card's roles, primary first; `-` for none |
| gist | the Oracle line cited as evidence for the **primary** role (`roles::line_text`: reminder text gone, `~` the card), cut at a whole-token boundary to 110 characters (never inside `{...}`, never in an open quote; `…` marks a cut; `-` when no whole token fits). A card with no cited line (no roles, or a hand override) shows its first Oracle line that is not just keywords (`Flying, trample`), else the first line; `-` when it has no text. On a `--role R` page the gist cites R's own line. Never written text, so it cannot be a hallucination |
| flags | `*` the card is in the deck's list, `BAN` banned / `NL` not legal in Commander (a deck's own card can be one), `GC` Game Changer, `#N` EDHREC rank, `$N` price in USD, and `?` (always last) when the card's role tags are not reviewed yet (about the tags, not the card's quality: routed to the LLM and no answer, so they may be incomplete or wrong; no `?` means the rules or a hand override decided, or the LLM answered; with `llm_add` off for every role the LLM's answers are not used, so every routed card has `?`). `-` when there is nothing to say. `m:X` the engine model status of the card, (`F` Full, `f` Full-auto (compiled, not hand-verified), `A` Approximate, `P` Partial, `U` Unmodeled, and `N` = NO engine record: status unknown, treat as below Full; adding it needs a cards.json entry, `cardgen export` and `cardgen audit`, then JD; a card below `F`, or `N`, goes to JD before it is accepted). Every line has an `m:` flag when the engine facts are available and none when they are unavailable (the header says so), `d:XY` the other decks of JD's `mine` that play the card (one letter per deck, assigned from the keys in `decks.json` order and printed in the header; section A leaves out the digest's own deck, and the header legend ends with `(this digest: X)`), `k:N` the lines of the decks' `audit/*/FINDINGS.md` that name the card (only when N > 0). `digest::flags` makes them in the order `* BAN|NL GC #N $N m:X d:XY k:N ?`; `facts::Facts::context` fills the `Context` (indices resolved once: model status, a bit per JD deck with the letter table the header prints, the current deck, findings counts). The header gains one line for the `d:` letters and one for `m:` and `k:`; the gist shrinks to keep the line within 160 characters |

The candidates come from `Index::search`, so the deck's identity ceiling and exclusions are the ones `find --deck`
applies: Commander-legal cards inside the identity (`--ident` may narrow a deck's, never widen it), not in the deck, no
meld results, and no Stickers or Attractions. A header of `# ` lines gives the deck, the role-table kind, the
commander(s) with their full Oracle text, the deck's role counts and basic-land multiplicity, a reminder that absent
here is not the same as does not exist, the legend, the section sizes and any trimming; no timestamps. Sections, each
introduced by a `## ` line, EDHREC rank ascending with ties by name:

- **A** the deck's own cards (every name once, the commander first, `*`), uncapped;
- **C** (built first) synergy: every card whose type line or text names a subtype that the commander's Oracle text names
  (`Dragon`, `Equipment`; plurals count, `Time Lord` is one subtype; `named <card name>` references are masked, so a
  card named Spirit Guide is not Spirit synergy), at most 300;
- **B** per role: the 60 best-ranked cards whose **primary** role it is (150 for the deck's 5 most common roles: each
  card counts once for each of its roles; `fixing` is left out of that top 5 in a one-colour deck), not repeated from C;
- **D** cards new since the previous sync (the `oracle_id`s of `oracle.json` that `oracle.prev.json` lacks), the first
  100 (`--view new` has them all), not repeated from C; omitted with a note when there is no usable `oracle.prev.json`
  (a file with another `meta.schema` or a record without an `oracle_id` is unreadable);
- **E** the 60 best-ranked cards that have no role (not repeated from C or D; `--view roleless` has them all). Roleless
  cards are 31% of the catalog under the rules-only table and many are staples, so they are listed on purpose.

The size target is `--budget-tokens` (default 42,000, estimated as bytes/4, never below characters/4 and probably
15 to 30% under a real tokenizer on this symbol-dense text): when the caps give more, B's and E's caps are cut by one
common factor to the largest that fits, and the header says so (if even that is not enough the digest is left over
budget with a note, because A, C and D are not trimmed). A five-colour deck's digest stays under 200 KB.

`--role R` lists every non-deck card that has role R in any position as one list, best EDHREC rank first (name ties;
the roles field shows where R sits, and the gist cites R's own line); `--view roleless` and `--view new` list the
complete candidate sets of the same names. They are pages: a page ends at `--limit` cards or at 30,000 tokens (`--budget-tokens` overrides that, and a budget
that cannot hold the page header and one card is an error), and the block header says `next page: --offset N` while there
are more. `--limit` and `--offset` need `--role` or `--view`. Between them the role pages and `--view roleless` reach
every candidate of the digest.

For code: `digest::open` (catalog and role table), `digest::build(&Index, &Table, &Spec)`, `digest::Lines::{new,
with_context, line, line_for}`, `digest::build_with` (a `Context`), `digest::flags`, `digest::review`, `digest::deck_cards`, `digest::role_counts`,
`digest::commander_subtypes`, `digest::Prev::load`, and `map::build(&Index, &Table, Option<&Path>)` (JD's decks come from
the `decks.json` beside the spellbook directory) or `map::build_with` / `map::build_with_facts` (the engine coverage section and the model columns of JD's decks table need the facts); `facts::Facts::{load, decks_and_findings, context, status_of, decks_of, findings_of, deck_bit}` (phase 4: `Facts::load(project, &index, engine, decks)` builds the engine only when asked) and `knowledge::fold_case`; none of them reads argv.

## Weekly report (the weekly job)

```
cardgen weekly-report [--date YYYY-MM-DD] [--commit TEXT] [--stages FILE] [--roles-before FILE] [--llm-log FILE]
                      [--out PATH] [--keep N] [--stdout] [--data-dir DIR]
```

Written by `scripts/knowledge-weekly.sh` (see `documents/card-knowledge.md`, "The weekly job"), but usable by hand. It reads
`oracle.prev.json` and `oracle.json` (the knowledge directory is `--data-dir`, else `CARDGEN_KNOWLEDGE_DIR`, else the
catalog `find` would read), `data/decks.json` (`mine` only), `data/scryfall_cache.json` (the engine's cache) and
`roles.json`, and writes `reports/DATE.md` (an earlier report of the same date becomes `DATE.N.md`; the newest `--keep`,
default 12, dated reports stay and nothing else in `reports/` is touched). Offline; the one file it writes is the report: the text is written whole to a temporary file first, the report
of the same date already there is kept as `DATE.N.md` (a hard link, N counting up, ordered as a number; retention sorts by
date, then N, with `DATE.md` the newest), and only then is `DATE.md` renamed into place, so a failed write leaves the
previous report where it was. With `--out PATH` it writes only that file (the weekly script publishes it itself).
`--stages FILE` is the script's stage record, `name<TAB>ok|skip|fail<TAB>seconds<TAB>note` per line (`bulk-sync` and `llm`
are the names the report looks at); `--roles-before FILE` the role table from before this run's role stage; `--llm-log
FILE` the end of which is quoted.

Sections, in this order: **Action needed** (for JD's decks only: every deck card that is not Commander-legal now, every run ("became X (was Y)" when the comparison holds the change, else "is X": a ban that was already there, a skipped sync, a card that joined the deck), a Game Changer
added or removed with the deck's new Game Changer count against `decks::gc_rules` for each tier the deck lists in
`data/my_decks.toml`, a deck card the catalog does not know, Oracle errata against `data/scryfall_cache.json`, which means
the engine's model may be stale, and any failed stage) · **Run** (bulk `updated_at` old to new, card counts, the stage table)
· **New cards** (by `oracle_id` diff: per set, then the new Commander-legal cards inside each deck's identity with their
roles) · **Bans/unbans and Game Changer changes** (catalog-wide, and new cards that arrive banned or as Game Changers) ·
**Role changes and LLM stats** (cards whose roles changed, rows in `llm_roles.jsonl` and `llm_roles.pending.jsonl`, the last
merge's counts). When the sync was skipped or failed, there is no previous sync (the first run) or it cannot be read, the
comparison sections say so instead of listing anything. The catalog is read with the loader `find` uses and a deck's card names are resolved the way `find` and the digests do. The
Oracle text of the engine's cache is compared with the catalog's as `roles` reads it (reminder text stripped, the card's
own name as `~`, whitespace collapsed), the other fields with whitespace collapsed. Every name, key, set and note is
flattened to one line and Markdown-escaped. `--date` must be a real calendar day. The command fails only on a bad argument, an unreadable `--stages`
file or a report it cannot write: a missing, empty or malformed input is a line in the report (an unreadable catalog is the
first action item). The output is deterministic. `tests/weekly.rs` covers it on a tiny synthetic catalog
(`tests/fixtures/weekly/`: a new card, a ban, an unban, a Game Changer added and removed, an errata, a deck of `mine`
affected and one not, an empty `mine`, a missing and a malformed `oracle.prev.json`); `tests/weekly_script.rs` runs the
script offline.

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
src/weekly.rs      weekly-report (the weekly job's report from oracle.prev.json, oracle.json, decks, the cache)
src/knowledge.rs   the in-memory card index and `find` (queries oracle.json)
src/json.rs, re.rs, py.rs   JSON, regex and string behaviour (ordered objects, byte-exact output)
```

## Roles (offline rule classifier)

```
cardgen roles --rules [--data-dir DIR]
cardgen roles --llm [--limit N] [--ids FILE] [--only-routed] [--routes NoRole,UnknownLine] [--include-not-legal] [--batch 10]
                     [--dry-run] [--endpoint URL] [--model ID] [--pending] [--no-think]
cardgen roles --merge [--model ID] [--pending] [--no-think] [--data-dir DIR]
cardgen roles --validate [--gold PATH] [--json] [--stage rules|merged] [--data-dir DIR]
cardgen roles --show "Ashnod's Altar" [--data-dir DIR]
cardgen roles --stats [--data-dir DIR]
cardgen roles --lint [--data-dir DIR]
cardgen roles --audit [--seed N] [--n 80] [--rule ID] [--certain] [--data-dir DIR]
cardgen find --role ramp --role sac-outlet --json
```

`roles --rules` reads the catalog without network access and atomically writes `roles.json` beside `oracle.json`.
Stickers and Attractions remain searchable in Oracle but are omitted from the role table and audit samples.
`--stats` reports their skipped count; routing denominators include only deck cards.
The JSON envelope has `meta: {taxonomy, rules_fnv, oracle_fnv, count, prompt_fnv, model, table_fnv}` and `cards` sorted by Oracle id
(`prompt_fnv` and `model` are empty in a rules-only table and filled by `roles --merge`).
The hashes are FNV-1a-64: taxonomy text; compiled rule and line-extraction source plus override content; and
`sync.lock`'s Oracle checksum. Fixture catalogs without a lock fall back to canonical bulk hashing, independent
of input order and JSON whitespace.
The table is byte-deterministic. Each assignment has up to four ordered `roles`,
`primary` (the first role, or `""`), per-role `evidence` (`source`: `Rule`, `Override` or `Llm`; rule id; `certain`; `face`; `line`), and `needs_llm`, plus a `route` list of
explicit reasons and, in a merged table, `llm_failed`.
Evidence uses zero-based indices into `Card::lines()`; `face: null` means the card's top-level Oracle text.
`line` is the global line index without face separator lines. `--show` prints the original supporting lines.

The 26 definitions, sources and LLM evidence guards live in `data/knowledge/roles.toml` (version 1).
Roles describe main or repeatable functions, with the conventions in `roles_gold.toml` taking precedence over
keyword matching. Reminder text is stripped, self names become `~`, land backs of MDFCs add no roles, prepare cards
use their spell face plus their creature face's own abilities, and expensive planeswalker ultimates add no roles. Basic land types confer their intrinsic
mana abilities even when printed only as a parenthesized synopsis. Printed self evasion/protection gives no role;
printed lifelink is lifegain, listed last. Rules also recognize semantic keyword actions such as recruit, connive,
transmute, mobilize and afterlife without treating their reminder text as independent abilities. Repeated modal
abilities pass their trigger context to their own bullet lines. At the four-role ceiling, a copy's redundant token
role yields to an independent function.
Ward granted to other permanents is protection; a lifelink-only team grant is lifegain, not anthem. Creature,
Treasure and Clue tokens count; other artifact tokens are judged by their abilities. Repeatable Clue makers also
give draw. Repeatable or multi-Treasure makers also give ramp; a single Treasure rider gives tokens alone.
Mana filters with equal mana cost/output are fixing, not ramp. Haste alone and limited target pumps are not
anthem. Self-replacing death draws, aura/equipment cantrips and a permanent's one fixed-card enter or death draw (C19) are not draw; monarch is draw. Self-name searches are not tutor, and opponent
compensation tokens are excluded while their removal effect still counts.
A creature token copy gives copy and tokens, copy first. Team +1/+1 counter placement is counters and anthem; player counters are excluded, but proliferate
counts. Amass gives tokens, plus counters when repeatable. Returning noncreature permanents from your graveyard
to the battlefield is recursion; unrestricted permanent returns also give reanimate, which requires creatures.

Unrecognized lines, overflowing role sets and potentially incidental selection, recursion, graveyard hate,
finishers, stax, drain, copy, evasion and protection route to a future LLM stage. Removal with several clauses also
routes there. `--stats` and validation report the Commander-legal routing share. Certain assignments are limited to
audited direct templates: nonland single-color tap mana, direct spell counters, explicit nonland searches to hand,
draw two/three/four, and printed standalone lifelink. A certain role does not exempt the card's other effects from
routing. This command never calls an LLM. Rejected rule hits route unless a named `INCIDENTAL` convention in
`roles.rs` explains the rejection (cantrips/loot, costs, self keywords, zone alternatives, compensation, bookkeeping
or small riders). Two accepted roles on the same Oracle line also route, except subsets of ramp/fixing/sac-outlet
the Clue tokens/draw pair and the C6 Treasure tokens/ramp pair. `ROLE_POLICY` defines role sources and risky-role routing in one place.
Roleless cards with any line beyond pure keywords/keyword costs or one-mana taps route as NoRole. Two or more
produced WUBRG colors without fixing also route as NoRole. Boilerplate is matched per sentence.
`route` records UnknownLine, NoRole, RejectedHit, SameLine, RiskyRole, Overflow or RegexError. Statistics report overlapping
legal counts and an exclusive partition in that order. Regex input is bounded to 2048 bytes per line, with a raised
backtracking limit; a remaining regex error or oversized line routes instead of aborting classification.
Costs and initial trigger conditions are separated from effects; sentences are judged independently while linked
zone/pronoun abilities retain their necessary context. `--lint` lists every rule alternative matching fewer than
three catalog cards and fails unless its exact alternative is documented in the frozen `EXCEPTIONS` list. The
reason column explains the semantic exception; it is not permission to add new narrow phrase patches.

`--audit` freshly classifies a seeded sample without replacement of Commander-legal catalog cards outside the
project's `roles_gold.toml`, then prints complete Oracle lines, predicted roles, evidence and routing. It neither
reads saved assignments nor applies overrides, and writes no files. Defaults are seed 0 and 80 cards. Oracle ids
are ranked with a fixed FNV-1a hash mixed with the seed through SplitMix64, so catalog input order is irrelevant.
`--rule ID` samples that rule's assignments; `--certain` restricts to certain assignments (both filters apply to the
same evidence). If fewer than N qualify, every qualifying card is shown. The footer reports actual cards and
role assignments, including the rules-final subset, for a manual precision/recall audit.

Hand overrides in `data/knowledge/roles_overrides.toml` replace the entire role set, including certain rules;
the file documents the `[[card]]`, `name`, `roles`, `note` format. Overrides intentionally do not improve validation:
`roles --validate` scores the rules-only map (the library function scores whichever map it is given); `--stage merged`
leaves override cards out for the same reason.
An override's evidence has source `Override`, its note in `rule`, and no face or line, since it need not have a
regex justification. No initial overrides are installed.

`find` attaches `roles.json` for a role filter or to enrich JSON when the table is valid. A plain search remains
available with missing, damaged or stale role data. Plain text/count searches read the leading `meta` object and check `meta.table_fnv` (a hash of the card table
bytes) without parsing the table, and emit a repair note if metadata is missing, stale or the table is damaged. JSON enrichment and
role filters validate the complete table. Optional enrichment emits one stderr note on failure and omits role fields. Repeatable `--role R` filters combine
with AND. An unknown id is a usage error listing all 26 ids; a missing roles file names `cardgen roles --rules`.
JSON rows gain `roles` and `primary` only when the side table is attached. The Oracle catalog and `Card` remain
immutable; programs use `Index::attach_roles`, `Index::roles` and `Query.roles`. Missing ids, invalid evidence or
role sets, taxonomy/rule/Oracle hashes and deck-card count mismatches are errors for required role searches;
regenerate with `cardgen roles --rules` after sync or rule/taxonomy changes.

Validation scores the rules-only assignments and reports per-role and micro recall against the engine's manual/pool
cards in owned decks, per-role precision and recall against independent gold labels (all and certain separately),
forbidden-role violations, primary accuracy, overall all-role and src=rules micro scores, exact missed/unexpected
pairs, and routing by legality, layout and cause. Every engine tag is
listed with its fixed translation or reason for leaving it unmapped. Tags describe simulator approximations and
are not exhaustive; only recall is measured against them. `--gold` permits an independently supplied gold set.
JSON validation uses integer `tp`, `fp`, `missed` counts; precision is `tp/(tp+fp)`, recall is `tp/(tp+missed)`;
a zero denominator is 1. Text output includes those ratios. Nonzero exit means an acceptance threshold failed:
hand recall 0.95 (0.85 for roles with at least ten pairs), gold micro precision 0.90 and recall 0.85 for the 19 rules
roles, certain precision 0.95 across every role, and zero certain forbidden-role violations. The rules-final subset
(`needs_llm == false`, rules only) additionally requires precision 0.93 and recall 0.85 across
**all** roles, including those normally supplemented by the LLM. Its card count and integer micro counts are in
both reports. Other roles outside that subset are reported without precision/recall gates. Routing counts are
reported, not hidden by a threshold.

CI uses real Oracle text frozen in `tests/fixtures/roles_oracle.json`, with positive and decoy examples for every
rule. Full-catalog tests are opt-in: `cargo test --release -p cardgen --test roles_real -- --ignored` (requires the
local catalog). Run Cargo and test binaries under the repository memory cap. Run `cardgen roles --rules` before the
ignored tests: `real_saved_roles_json_matches_a_fresh_generation` compares the saved `roles.json` with a fresh rules
table, so it fails after any `roles --merge`.

The recall-only engine-tag mapping is value-aware:

| Tag/value | Roles | Reason |
| --- | --- | --- |
| `fill=dispute` | unmapped | Additional-cost sacrifice, excluded by convention 2. |
| other `fill` values | self-mill | A graveyard-filling engine effect. |
| `xtutor` | unmapped | X-spell engine flag, not evidence of a library search. |
| `prot=blink` | blink | Explicit exile/return engine mode. |
| other `prot` values | protection | Protective engine effect. |

The complete translation and unmapped reasons appear in every validation report.

### The LLM stage (local model)

`roles --llm` asks the local llama.cpp server (OpenAI-compatible API, default `http://127.0.0.1:8080/v1`, `--endpoint`
to change it) for the roles of the cards the rules routed, `roles --merge` combines rules, cached answers and
overrides, and `roles --validate --stage merged` scores the result. Everything an agent needs to run it:

```
systemctl --user start llama-27b    # the dense Qwen3.6-27B, 32K context ("llama up" starts the coder model: not this one)
llama ready                         # wait until it answers
cardgen roles --llm --only-routed [--limit N] [--batch 10]     # classify; resumable, writes llm_roles.jsonl
cardgen roles --merge                                           # writes roles.json and reports/
cardgen roles --validate --stage merged
systemctl --user stop llama-27b     # free the VRAM
```

The exact production commands, with the memory cap and the flags the cache rows depend on, are under "Final
configuration and the production run" below.

The server must already be up: `--llm` reads the model id from `GET /v1/models` first and exits non-zero with
`LLM unreachable` (touching no file) if it cannot. `--dry-run` counts candidates, cache hits and batches without a chat
request (it still reads the model id unless `--model ID` is given). Candidates are the Commander-legal deck cards the
overrides do not fix, optionally narrowed by `--only-routed` (the cards the rules routed), `--ids FILE` (Oracle ids or
exact names, one per line, `#` comments) and `--limit N` (at most N cards sent, taken after the cache is consulted).
Without `--only-routed` the rules-decided cards are sent too, which is the shadow audit: they never change, but every
difference is logged. `--pending` writes `llm_roles.pending.jsonl` (untracked) instead of the tracked cache, for the
scheduled job.

**The prompt.** The system prompt is `Taxonomy::prompt_text()` (the shared rules once, then each role's one-line
`definition` and role-specific `clauses`, then each cross-role convention once with the roles it concerns; no card is
named) and the fixed answer rules. The worked examples are a real user/assistant turn pair between the system prompt
and the real user message (eight cards: a rider on a removal spell, a cost-side counter, a "would draw" replacement, a
permanent steal, a one-turn Fog, a body with no role, and two cards with two roles each; none is in a gold set), so the
whole prefix is static and `cache_prompt` can reuse it. The user message is ten cards (`--batch`), each `[id] type line`
and its numbered lines from `roles::prompt_lines` (the same normalised text the rules and guards read: reminder text
stripped, `~` for the card name, MDFC land backs, planeswalker ultimates and over-long lines left out, original line
numbers kept); each later face is announced by an unnumbered header such as `(second face: Instant — Adventure)`. The
model is never shown the rule roles. Every request has `temperature` 0, `seed` 1, `cache_prompt` true and a
grammar-constrained `json_schema`: `cards` has `minItems` = `maxItems` = the batch size, `id` is an enum of the batch's
ids (`a0`, `a1`, ...) and `roles` is ONE array of at most four `{"role": <enum of the 26 ids>, "line": <integer >= 0>}`
pairs, most prominent first (the first is the primary). One array, not a map of optional keys, because llama.cpp's
schema converter only allows optional properties in declared order, which would make some multi-role answers
unwritable. If a server answers HTTP 400 to the schema, drop `,"minimum":0` in `response_schema_text` first. The system
prompt is about 3.1k tokens by the chars / 4 estimate (3.4k with the example turn), so a batch of ten stays far below the
32K context.

**Validation and retries.** Each card of an answer is judged alone. A card is structurally invalid if its id is missing
or repeated, a role is outside the vocabulary or repeated, there are more than four roles, or a line is missing, out
of range or was not shown; that card is asked once on its own (a card already asked alone is not asked again: the
request would be identical). A role whose `roles.toml` guard fails on the cited line is not a fault: the raw answer is
cached and `merge` drops that role as a `GuardDrop`. An unusable reply (not JSON, no `cards` array) or an HTTP error
status (the reply body is in the error) sends every card of the batch alone once. Only a transport error (no answer: a
refused or cut connection, a timeout of 180 s) resends the identical batch once; two unanswered requests in a row stop
the run after saving what was answered, and so do three batches in a row without one valid answer. A card with no valid
answer is reported in `reports/llm-failed.txt` and is `llm_failed` in the merged table; its rule roles stand. A run
whose failed cards exceed 20% of the cards it asked about exits non-zero. Every batch is flushed to the cache at once
(re-reading the file first and adding only this run's rows, atomically). A lock file beside the target
(`llm_roles.lock`, `llm_roles.pending.lock`) is created exclusively for the run: a second run waits for it, then
(after `CARDGEN_LLM_LOCK_WAIT_SECS`, default 900) names the file in its error, which is also how a stale lock from a
crashed run is found. The cache target is probed (read, and lock created) before anything is classified.
`reports/llm-run.json` is the last run that made requests; a warm run writes only `reports/llm-run-last.json`.

**The cache.** `llm_roles.jsonl` (tracked, one JSON row per line, sorted by key) is the only input that cannot be
regenerated. A row is `{key, name, roles, primary, evidence, model, taxonomy, prompt}`; the key is the FNV-1a-64 of the
card's normalised text (type line, face headers and numbered lines), the prompt fingerprint and the model id (a server
reports the path it was started with, so only the file name counts). The prompt fingerprint (`prompt_fnv`) covers
everything the model is told: the definitions, clauses, conventions, answer rules and the example turn.
It is never keyed on `rules_fnv`: editing the classifier, a guard, `keep_rule` or the overrides leaves every row valid,
while a changed definition, a changed prompt or another model id makes the old rows unreachable (they stay in the file;
writes are append-merge by key, atomic through a temp file and rename, and deterministic). A rerun with everything
cached makes no chat request (it prints the count; the one `GET /v1/models` is how a changed model is noticed).
`llm_roles.pending.jsonl` (untracked: the weekly job's unreviewed rows) is read after the tracked file when asked
(`roles --llm --pending` counts its rows as cached, `roles --merge --pending` merges them), so a pending row wins on
the same key; a plain `roles --merge` and `roles --validate --stage merged` read the tracked cache only.

**The merge policy** (`llm::merge`, a pure function):

- an override is used exactly;
- rule-`certain` roles are always kept;
- a card the rules decided (`needs_llm == false`) keeps its rule roles; the LLM never changes it;
- a routed card with a valid answer ends with the certain roles, plus every LLM role whose guard passes on the line it
  cited, plus each non-certain rule role the LLM also proposed or whose role has `keep_rule = true` in `roles.toml`
  (frozen: true for all 26 roles, so the LLM never removes a rule role: add-only; JD's earlier 16/10 split is
  superseded, per-role `keep_rule = false` is still supported). A role with `llm_add = false` (default true; frozen
  false for ALL 26 roles until the production run's additions are audited, so the merge uses nothing from the LLM: every
  assignment is the rules' own, order and primary included, and every routed legal card keeps `llm_failed`, so the digest
  shows `?`) is off for the LLM: its proposals are dropped like a failed guard and counted per role in the `--merge`
  output. Neither flag is part of the cache key. Order: the LLM's primary if it survived, then the LLM's order, then the certain roles, then the kept
  rule roles by prominence; at most four (a certain role is never the one cut). A card with no rule roles simply takes
  its guarded LLM roles. A role found by both stages keeps the rule's evidence; an LLM-only role has source `Llm`;
- a routed Commander-legal card with no valid answer (failed, or not run yet) keeps its rule roles with `llm_failed`
  set; `needs_llm` stays true. A routed card that is not legal is never sent and never flagged; the merge report counts
  failed, not run and not legal apart.

Every veto (a non-certain rule role the LLM did not confirm), guard drop, certain role the LLM omitted, overflow and
(for rules-decided cards) shadow difference is written to `reports/roles-disagreements.txt` with the rule and LLM role
sets, grouped by route reason; `reports/merge.json` has the counts. `roles --merge` is deterministic (byte-identical
reruns) and writes `roles.json` with `meta.prompt_fnv` and `meta.model`; `find --role`, `roles --show` and
`Index::attach_roles` work on it (LLM evidence is checked again there: a real, readable line and a passing guard). With
several models cached for the current prompt, `--merge` needs `--model ID`, and a model with no rows for the current
prompt is an error. `--pending` also merges the rows in `llm_roles.pending.jsonl`; without it they stay out, so an
unreviewed weekly run never reaches `roles.json`. The table's hash covers `meta.prompt_fnv` and
`meta.model` too, and `attach_roles` refuses a table built under another prompt.

**Validation.** `roles --validate --stage rules` (the default) is unchanged. `--stage merged` scores the saved merged
table, leaving out override cards, and adds per-role and micro precision, recall and F1, the routed-subset F1 for the
rules alone against the merged table (merged below rules fails the stage; the tuning and regression gold are sets the
rules were tuned on, so this can only tie or fail there, and with no role on it prints "LLM additions are off for every
role: the merged stage equals the rules stage" and ties), the rules-final figures, primary accuracy and
the LLM numbers from the last run and merge (classified, failed, retries, disagreements by route reason), and the LLM
alone: its cached answers on the routed gold cards with the guards applied, scored apart from the merge (the rules
were tuned to the tuning gold, so "merged at least rules" cannot show the LLM's own value there).

Tests (`tests/roles_llm.rs`) run against a fake local server (a `TcpListener` with canned `/v1/models` and
`/v1/chat/completions` replies); no test touches a real model or the network.

### Final configuration and the production run

**Selection flags.** `--routes NoRole,UnknownLine` keeps only cards whose rules-stage route contains any of the listed
reasons (`UnknownLine`, `NoRole`, `RejectedHit`, `SameLine`, `RiskyRole`, `Overflow`, `RegexError`; an unknown name is a
usage error); it combines with `--only-routed`, `--ids` and `--limit`. Commander-legal cards only are sent by default
(`--legal-only`, accepted for clarity; `--include-not-legal` lifts it). `--dry-run` prints the number of cards selected,
the number already cached and the batch count. `--no-think` adds `"chat_template_kwargs":{"enable_thinking":false}` to
every request (the dense Qwen3.6 is a reasoning model and otherwise spends its token budget thinking and truncates,
about five minutes per batch); it is a marker in the prompt fingerprint, so thinking and non-thinking rows never mix, it
is recorded in `reports/llm-run*.json`, and `roles --merge` needs the same `--no-think` to find those rows. The prompt
variant is `think` (a required one-sentence `why` before the roles) unless `CARDGEN_LLM_VARIANT=plain`; set the same
variable for `--llm` and `--merge`.

**Progress and resuming.** One line per batch goes to stderr (`batch N/M: K cards, T s, cache rows R`). The cache is
flushed atomically after every batch (temp file and rename, re-reading the file first), so a kill, even `kill -9`,
never leaves a half-written cache; there is no signal handler because none is needed. The killed run's lock file names
its dead process, so the next run takes it over, skips the cached cards (no request for them) and continues.

**Measured (pilot, the numbers behind the frozen configuration).** LLM alone on the routed gold cards, with the dense
Qwen3.6-27B, thinking off, the `think` prompt: precision 0.92 and recall 0.78 to 0.80 over the 18 roles it did best on;
eight roles (ramp, fixing, draw, selection, sac-outlet, finisher, evasion, blink) scored too weakly (all 26 are now off,
see below). On the blind
set the merged table against rules only made 26 errors against 27 (+4 true, +3 false, -4 missed roles). Letting the LLM
veto rule roles hurt, so the merge is add-only. The first pilot, with the 30B-A3B MoE model, scored P 0.73 / R 0.60 alone on the tuning routed cards.

**Production run** (the coordinator starts the server by hand; `--endpoint` names the `/v1` base). The server is the
dense Qwen3.6-27B-Q5_K_M from the tracked user unit `scripts/systemd/llama-27b.service` (link it once with
`systemctl --user link`, then `systemctl --user start llama-27b`). `llama up` starts the coder model, which is the wrong
one for this run:

```
systemctl --user start llama-27b; llama ready                  # Qwen3.6-27B, 32K context
M="systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0"
CARDGEN_LLM_VARIANT=think $M cardgen roles --llm --routes NoRole,UnknownLine --only-routed --batch 10 --no-think \
    --endpoint http://127.0.0.1:8080/v1 2> /tmp/roles-llm.err   # about 12,520 legal cards, about 12 hours; progress
                                                                 # lines go to stderr; rerun the same line after an interruption
systemctl --user stop llama-27b
$M cardgen roles --rules                                        # the rule table, unchanged
CARDGEN_LLM_VARIANT=think $M cardgen roles --merge --no-think
$M cardgen roles --validate --stage merged
git add data/knowledge/llm_roles.jsonl && git commit -m "LLM role answers ..."   # the cache is the only non-reproducible input
```

Check with `--dry-run` first (the selected count and batch count). Only the cards the rules cannot read (no role, or a
line no rule understands) are sent; every other card keeps its rule roles untouched. **`llm_add` is off for all 26 roles
until the audit** (see "LLM results (2026-11-22)" in `documents/card-knowledge.md`): the merge reads the cache and drops
every proposal, so `roles.json` has the rules table's assignments exactly (every routed card stays `?`) and no unaudited role reaches `find --role`. A merge with
rows in the cache but none under the current prompt (a missing `--no-think`, another `CARDGEN_LLM_VARIANT`) is an error
that names `--model <id>`, never a silent rules-only table.

**Known limits.** With `llm_add` off for every role the model's answers are cached but change nothing in `roles.json`.
A rule role is never removed, so a wrong rule role on a sent card stays. A sent card the model fails on
keeps its rule roles with `llm_failed`; a run with more than 20% failures exits non-zero. Answers are keyed on the card
text, prompt, variant, no-think marker and model file name, so changing any of them re-asks everything. The blind-set
result (26 versus 27 errors) was measured on the pilot's answers, which the production run did not reproduce card for
card (a 92-card comparison found 90.2% identical on-role answers: answers depend on the batch neighbours); the
production additions are unaudited.
