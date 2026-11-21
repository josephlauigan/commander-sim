# Card knowledge base

The approach for giving an agent as close to the entire Magic: The Gathering card catalog as context as is physically
possible when it makes a card or strategy decision for JD's decks. Status of each phase is in
[Status](#status); the card pipeline the simulator itself uses is in [`cards.md`](cards.md).

## Goal

**When JD asks for a recommendation (a swap, a cut, a buy, a package, "what should this deck be doing"), the agent
has as close to the whole catalog as context as it can: every Commander-legal card that could matter, with its
rules text, roles, popularity, legality, and what this repo already knows about it.**

The catalog (about 30,000 Oracle cards) cannot literally sit in context: as full text it is millions of tokens, and
even one compact line per card is on the order of a million. So the goal is met by layers. The agent always holds a
compact view of the whole catalog and pulls full detail for any card on demand. It never decides from memory
alone, and it never skips the sim.

| Layer | What the agent holds | Size | Command |
|---|---|---|---|
| 1. Catalog map | Whole catalog summarised: counts by colour identity, role, creature type, keyword; bans and Game Changers; known combos | about 8k tokens | `cardgen map` |
| 2. Deck digest | Every Commander-legal, in-identity card that clears a per-role cap, one line each, with roles, a rules-text gist, popularity and flags | about 50k tokens per deck | `cardgen digest --deck X` |
| 3. Card detail | Full Oracle text, roles and their evidence, engine model status, which decks use it, `FINDINGS.md` hits | per card | `cardgen find --name N --full` |
| 4. The sim | Paired swap-test evidence | per test | `commander-sim --swap` (existing) |

Layers 1 and 2 give breadth: the agent knows what exists. Layers 3 and 4 give correctness: exact wording, then
measured evidence. A digest line is a compressed view, so an agent working from digests can judge a card's role and fit
but will not catch subtle Oracle interactions until it reads the full text. Every card on a final shortlist therefore
gets layer 3 before it is proposed.

## Where the data lives

Everything is under `data/knowledge/` (override with `CARDGEN_KNOWLEDGE_DIR`; a worktree without the directory falls
back to the main checkout's). Generated data is gitignored except what cannot be regenerated.

| File | Tracked | Content |
|---|---|---|
| `oracle.json` | no | Every catalog card as a slim record, from Scryfall's Oracle Cards bulk file (a gzip file of JSON Lines, read with `flate2`) |
| `oracle.prev.json` | no | The previous sync, for the weekly diff |
| `sync.lock` | yes | Which bulk file the tracked caches correspond to: `bulk_updated_at`, `count` and `fnv1a64` (FNV-1a-64 of the `cards` array text) |
| `roles.toml` | yes | Role taxonomy: ids, one-line definitions (used verbatim in the LLM prompt), guard regexes |
| `roles_overrides.toml` | yes | Hand role sets, highest precedence |
| `roles_gold.toml` | yes | About 150 hand-labelled cards, the precision standard |
| `llm_roles.jsonl` | yes | Local-LLM classification cache: the only non-reproducible input |
| `llm_roles.pending.jsonl` | no | Rows from the scheduled job not yet reviewed |
| `roles.json` | no | Merged roles per card |
| `map.md`, `digests/`, `reports/` | no | Generated views and weekly reports |

`data/scryfall_cache.json` (the engine's small, tracked, name-keyed cache) is unchanged. The bulk record is a strict
superset of the cache record, so any catalog card can go through the existing autotag, DSL and audit code to get a
predicted model status without a second parser.

## Design decisions

**Slim record.** The cache's 15 fields stay byte-compatible; the bulk record adds `oracle_id`, `edhrec_rank`, `usd`,
`released_at`, `set`, `rarity`, `reserved` and the meld/face extras. `set` and `released_at` describe Scryfall's chosen
printing, not the first printing; new-card detection uses an `oracle_id` diff. Tokens, emblems, art cards, schemes,
planes, vanguards and the memorabilia `front_card` layout are dropped, and so is any card whose type line (or a
face's) starts with `Token`, `Card` or `Dungeon` (Role tokens that have a card layout, helper cards such as
Experience, Dungeons), and a repeated `oracle_id`. Digital printings are **not** dropped: Scryfall
picks one representative printing per card, and for many paper staples (Mind Twist, Lotus Petal, Badlands) it is a
Masters or Remastered set flagged `digital`, so dropping them would lose over a thousand Commander-legal cards. Banned
and not-legal cards are kept so "why can't I play X" answers correctly, but `find` and `digest` default to legal
only. Faces are searched by every face name and text;
nothing branches on layout names except the drop list.

**Index.** Typed serde load of `oracle.json` into an in-memory index (identity bits, cmc, type flags, a lower-case
search string, a role bitset). Every query is a linear scan, milliseconds for 30k cards. The only dependency added
is `flate2` (already in the lockfile through `ureq`), because Scryfall serves the bulk file as a plain gzip file, not
with a `Content-Encoding` header; `ureq` and `fancy-regex` were already in `cardgen`.

**Roles.** Twenty-six roles (ramp, fixing, draw, selection, tutor, removal, wipe, counter, protection, stax, discard,
gy-hate, recursion, reanimate, self-mill, tokens, anthem, sac-outlet, drain, lifegain, counters, blink, copy,
extra-phase, finisher, evasion), defined in `roles.toml`, separate from the engine's tag vocabulary in `sim-core`.
A card gets a role only for a main or repeatable effect, at most four, ordered by prominence. Rust rules over Oracle
text decide most roles. The local LLM proposes roles only for cards the rules cannot fully read, and each role it
proposes must cite an Oracle line and pass a textual guard for that role. Precedence: hand override, then rule-certain
roles, then the LLM's guarded roles together with the rules' non-certain roles. Every disagreement is reported.

**Digest.** One `|`-separated line per card (`Name|cost|type|roles|gist|flags`), at most 160 characters. The gist is
the Oracle line cited as evidence for the primary role, never LLM-written prose, so it cannot be a hallucination.
Sections: the deck's own cards; per-role top cards by EDHREC rank (caps keep a five-colour deck in budget); a synergy
section for subtypes the commander's text names; cards new since the last sync; unranked cards. `--role R` lifts that
role's cap and pages through the rest.

**Local LLM.** The loaded model is Qwen3-Coder-30B (32K context, no internet) behind llama.cpp's OpenAI API. Requests
use temperature 0, a fixed seed and a JSON schema that constrains roles to the taxonomy; ten cards per request. The
prompt is fixed and versioned. Output is validated (ids, vocabulary, cited line, guard regex), retried once as a batch
and once per card, and a card that still fails keeps its rule roles. Results are cached by a hash of the normalised
card text, the taxonomy version, the prompt version and the model id, so reruns are cheap and diffs reviewable.

**Weekly job.** A systemd user timer (a cron equivalent that survives missed runs and tracks units in git) runs a
script that builds from a separate clone of origin/main, syncs the bulk file only if `updated_at` changed, refreshes
roles, runs the LLM stage, regenerates the map and digests, and writes a report. It starts `llama-server` only if it
was not already running and the GPU is free, and stops it only if it started it. It never commits, pushes, edits a
tracked file, runs a sim, proposes a swap or merges pending LLM rows. JD installs the timer by hand.

## Acceptance

Each phase has command-level checks (below). They are what the adversarial review tries to break.

| Phase | Check |
|---|---|
| 1 | `bulk-sync` yields 25,000 to 40,000 cards, none a token, and every card in `data/scryfall_cache.json` present; a second run reports "up to date" and downloads nothing; `slim_bulk` agrees with `slim` on the 15 cache fields; a corrupt `--from-file` exits non-zero and leaves `oracle.json` byte-identical; peak memory under 2 GB |
| 2 | `find` returns the right cards for each filter and never an off-identity or already-in-deck card under `--deck`; back-face names resolve; a bad regex exits non-zero; a full scan under 1 s |
| 3 | `roles --rules` is byte-deterministic; hand-tag recall at least 0.95; gold precision at least 0.90 and recall at least 0.85; the LLM stage beats rules alone on the gold cards it handles or stays off for that role; bait cards get no guarded role; schema-valid after retries at least 99% |
| 3b | `map.md` under 32 KB; the five-colour digest under 200 KB; every digest line has six fields and at most 160 characters; two runs byte-identical |
| 4 | `find` shows model status, deck usage and `FINDINGS.md` hits; a card not in `cards.json` never shows `Full`; status agrees with `cardgen audit` on 20 sampled cards |
| 5 | `CLAUDE.md` and `AGENTS.md` carry the same workflow section; this document is linked from `cards.md` |
| 6 | The three failure scenarios (server already running, GPU busy, no network) leave the working tree and previous outputs untouched; a run with nothing new exits 0 |

## Catalog facts that shape the later phases

Found while building and reviewing phase 1, on the 2026-11-21 bulk file (39,356 records; 35,113 kept: 32,643 legal,
83 banned, 2,387 not legal).

- **Names are not unique.** 15 names are shared by two or more kept cards (Un-card variants, plus pairs such as Pick
  Your Poison, Red Herring and Fast // Furious). The `find` index maps a name to a list of cards, prefers a
  Commander-legal one, and shows all under `--legal any`.
- **An exact full name beats a face name.** About 25 face names (27 before name folding) equal another card's full name, mostly the `prepare` layout
  (Emeritus of Conflict // Lightning Bolt) and split cards. `find --name "Lightning Bolt"` returns Lightning Bolt first,
  and roles and digests never credit a `prepare` card as the spell it names.
- **Fields are absent, never null.** `mana_cost` is absent on 493 records and `oracle_text` on 896 (the text is on
  the faces); `power`, `toughness`, `loyalty`, `edhrec_rank`, `usd`, `produced_mana`, `card_faces` and the meld fields
  are absent when they do not apply. Face `colors` is on only 986 of 1,797 faces, so identity comes from the
  top-level `color_identity`.
- **Meld.** Seven meld results exist, all Commander-legal. They carry `all_parts_meld` and `meld_parts`, and `find`
  excludes them by default. Meld is matched by id or name because `all_parts` ids are printing ids.
- **Digital printings are real cards.** Scryfall's one representative printing per card is a digital Masters set for
  many paper staples, so the `digital` flag says nothing about playability; Commander legality does.
- **Names are matched folded.** `find` compares names, face names, `--name-contains` and deck lists after a fold:
  lower case, accents and combining marks dropped, every apostrophe a plain `'`, `Æ`/`Œ`/`ß` spelled out. So `Anduril`
  finds Andúril and a deck list may spell a name either way. Hyphens, commas and spaces stay, so folding creates no new
  shared names.
- **A deck's identity is a ceiling.** Under `--deck`, `--ident` may only narrow it. `Index::search` is the one place
  that applies the ceiling, the deck's exclusions and the filters, so the digest cannot re-implement it.
- **`find --json` is a contract.** One row shape, `oracle_text` is the joined text, keys are only added, never
  renamed or removed, and consumers ignore unknown keys. A cut page, a named card removed by the filters and
  unresolved deck names are reported as notes (stderr for JSON), never silently.
- **Worktrees.** `data/knowledge/` exists in every checkout because `sync.lock` is tracked, so a worktree falls back
  to the main checkout's store when `oracle.json` is missing, not when the directory is.
- **Open for phase 6.** Every new bulk file rewrites the tracked `sync.lock`, which conflicts with a weekly job that
  must not edit tracked files. The recommendation is to untrack `sync.lock` and keep the bulk `updated_at` in
  `oracle.json` and the weekly report; JD decides when phase 6 is planned. Two concurrent `bulk-sync` runs can still
  pair one run's oracle with the other's lock; it heals on the next run, and phase 6 adds a lock around the job.

## Limits

- Role tags will miss or mislabel some cards. The recall and precision numbers are reported, not assumed.
- A card the engine does not model can be recommended but not evaluated by the sim. Digest lines and `find` flag it; a
  card below Full goes to JD before it is accepted.
- EDHREC rank favours popular cards. The synergy, new-card and `--role` views exist to reach past it.
- Roles and gists describe a card; they do not say it is good in a particular deck. The sim and JD decide that.

## Status

| Phase | Branch | State |
|---|---|---|
| 0 Design consult (Opus) | none | done |
| Approach doc | `knowledge-base` | written |
| 1 `bulk-sync` | `knowledge-bulk-sync` | done, merged into `knowledge-base` |
| 2 `find` | `knowledge-find` | done, merged into `knowledge-base` |
| 3 Roles (rules + local LLM) | `knowledge-roles` | not started |
| 3b `map` and `digest` | `knowledge-digest` | not started |
| 4 Status, decks and findings join | `knowledge-status` | not started |
| 5 Workflow in the rule files | `knowledge-workflow-doc` | not started |
| 6 Weekly timer and report | `knowledge-weekly` | not started |

Branching: `knowledge-base` is the integration branch off origin/main. Each phase is built on its own branch
(`knowledge-<phase>`), and merged into `knowledge-base` only after its review chain passes: built by Sonnet,
adversarial tests by Sol, final review by Opus, checks re-run by the coordinator. `knowledge-base` goes to main by PR
at milestones, and JD decides when each merges.
