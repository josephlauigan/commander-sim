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
| 1. Catalog map | Whole catalog summarised: counts by colour identity, role, creature type, keyword; bans and Game Changers; known combos | about 3k tokens | `cardgen map` |
| 2. Deck digest | Every Commander-legal, in-identity card that clears a per-role cap, one line each, with roles, a rules-text gist, popularity and flags | about 40k tokens per deck | `cardgen digest --deck X` |
| 3. Card detail | Full Oracle text, roles and their evidence, engine model status, which decks use it, `FINDINGS.md` hits | per card | `cardgen find --name N --full` |
| 4. The sim | Paired swap-test evidence | per test | `commander-sim --swap` (existing) |

Layers 1 and 2 give breadth: the agent knows what exists. Layers 3 and 4 give correctness: exact wording, then
measured evidence. A digest line is a compressed view, so an agent working from digests can judge a card's role and fit
but will not catch subtle Oracle interactions until it reads the full text. Every card on a final shortlist therefore
gets layer 3 before it is proposed.

## How an agent uses it

The working steps are the section "Card recommendations: use the card knowledge base" in `CLAUDE.md` and `AGENTS.md`
(the same text in both). In short: read the map, read the deck's digest (header first), query `find` for what the digest
trimmed, read `find --name NAME --full` before proposing a card, run the swap checks and put a `--swap` test in the plan JD approves, and
say which digest and data date the answer used.

| Command | What it gives | Writes |
|---|---|---|
| `cardgen bulk-sync` | the catalog (`data/knowledge/oracle.json`) | `oracle.json`, `oracle.prev.json` and the untracked `sync.lock` |
| `cardgen roles --rules` / `--merge` | the role table (`roles.json`), rules and merged with the LLM answers | `roles.json` beside the catalog it read (in a worktree without its own catalog, the main checkout's) |
| `cardgen map` | the whole-catalog summary (`map.md`) | `map.md` (also with `--stdout`) |
| `cardgen digest --deck KEY` | one line per candidate card (`digests/KEY.txt`) | the digest file (also with `--stdout`) |
| `cardgen find ...` | any slice of the catalog, with engine status, deck usage and FINDINGS hits | nothing |

Keeping it fresh is the weekly job ([The weekly job](#the-weekly-job-phase-6): `bulk-sync`, `roles --rules`, `roles
--merge`, the LLM stage, `map`, `digest`, a report), which JD installs by hand; by hand it is the same commands. A recommendation
session never runs `bulk-sync`, `roles` or `export`; a refresh is its own step that JD approves.
`map` and `digest` write only their gitignored views.

## Where the data lives

Everything is under `data/knowledge/` (override with `CARDGEN_KNOWLEDGE_DIR`; a worktree without the directory falls
back to the main checkout's). Generated data is gitignored except what cannot be regenerated.

| File | Tracked | Content |
|---|---|---|
| `oracle.json` | no | Every catalog card as a slim record, from Scryfall's Oracle Cards bulk file (a gzip file of JSON Lines, read with `flate2`) |
| `oracle.prev.json` | no | The previous sync, for the weekly diff |
| `sync.lock` | no | Which bulk file `oracle.json` was built from: `bulk_updated_at`, `count` and `fnv1a64` (FNV-1a-64 of the `cards` array text). A cross-check: the bulk `updated_at` is also in `oracle.json`'s meta, and a missing lock is rebuilt from the file |
| `roles.toml` | yes | Role taxonomy: the shared rules block, cross-role conventions, and per role an id, a one-line definition, role-specific clauses and a guard regex (the prompt text is built from all but the guard), `keep_rule` |
| `roles_overrides.toml` | yes | Hand role sets, highest precedence |
| `roles_gold.toml` | yes | About 150 hand-labelled cards, the precision standard |
| `llm_roles.jsonl` | yes | Local-LLM classification cache: the only non-reproducible input |
| `llm_roles.pending.jsonl` | no | Rows from the scheduled job not yet reviewed; `roles --merge` ignores them unless given `--pending` |
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
roles, runs the LLM stage, regenerates the map and digests, and writes a report. It starts the dense Qwen3.6-27B server
only if none answers on port 8080 and the GPU is free, and stops it only if it started it. It never commits, pushes, edits a
tracked file, runs a sim, proposes a swap or merges pending LLM rows. JD installs the timer by hand. Details:
[The weekly job](#the-weekly-job-phase-6).

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
| 6 | The three failure scenarios (server already running, GPU busy, no network) leave the working tree and previous outputs untouched; a run with nothing new exits 0 and says there are no new cards; a ban injected into `oracle.prev.json` for a card in JD's decks leads the report's Action needed |

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
- **Worktrees.** A worktree falls back to the main checkout's store when its own `oracle.json` is missing, whether or
  not its `data/knowledge/` directory exists (it usually does, for the tracked role files; nothing relies on it).
- **`sync.lock` is untracked** (JD, 2026-11-22): every new bulk file rewrote it, which conflicts with a weekly job that
  must not edit tracked files. The bulk `updated_at` is in `oracle.json`'s meta and in the weekly report. A missing lock
  is rebuilt from a whole `oracle.json`, so the merge that untracks it (it deletes the file) costs nothing. Two concurrent
  `bulk-sync` runs can still pair one run's oracle with the other's lock; it heals on the next run, and the weekly job
  holds a lock of its own.

## The weekly job (phase 6)

`scripts/knowledge-weekly.sh`, run by the systemd **user** timer `scripts/systemd/knowledge-weekly.timer` (Sunday 04:00,
`Persistent=true` so a missed run happens at the next start, a small random delay) through
`scripts/systemd/knowledge-weekly.service` (`RuntimeMaxSec=4h`, `MemoryMax=24G`, `Nice=10`, idle disk priority). Nothing
installs it: JD does (below). JD's decisions of 2026-11-22: Sunday 04:00, the LLM stage on the dense Qwen3.6-27B (thinking
off), `sync.lock` untracked.

**Stages.** Each is recorded `ok`, `skip` or `fail` with its time and a reason, in the report. One failing never deletes
the previous outputs (`cardgen` writes a temp file and renames it) and never stops the report.

| # | Stage | What it does |
|---|---|---|
| 1 | `fetch-build` | fetches into a separate clone (`~/.cache/commander-sim-knowledge/repo`), resets it to `$KNOWLEDGE_REF` (default `origin/main`), builds `cardgen` there under the memory cap; the clone is `--root`, so decks, the engine cache and FINDINGS come from it. A failed fetch or build keeps the previous build in use |
| 2 | `bulk-sync` | the Oracle Cards bulk file, only if its `updated_at` changed (else `skip`, "up to date"); fewer than 25,000 cards is refused; the previous catalog stays as `oracle.prev.json` |
| 3 | `roles` | `roles --rules`, then `roles --merge --no-think --model Qwen3.6-27B-Q5_K_M.gguf` when the tracked `llm_roles.jsonl` has rows (add-only; pending rows stay out), so `roles.json` keeps the reviewed LLM roles; the table from before the run goes to the report |
| 4 | `llm` | cards the rules could not read (route `NoRole` or `UnknownLine`) that miss the cache: `roles --llm --routes NoRole,UnknownLine --only-routed --batch 10 --no-think --pending` with `CARDGEN_LLM_VARIANT=think`, new rows to `llm_roles.pending.jsonl` only. A dry run first counts the cards to do; none means `skip` and the GPU is not woken |
| 5 | `map-digest` | `map.md` and one digest per deck of `mine` |
| 6 | `report` | `reports/YYYY-MM-DD.md` (the newest 12 stay) by `cardgen weekly-report`; if `cardgen` cannot run, a short report is written by the script, and the exit code is non-zero only if no report could be written |

**Where things go.** Output goes to the main checkout's `data/knowledge/` (override `KNOWLEDGE_OUT`), through
`CARDGEN_KNOWLEDGE_DIR`, never to the clone. Everything written there is gitignored: `oracle.json`, `oracle.prev.json`,
`sync.lock`, `roles.json`, `map.md`, `digests/`, `reports/`, `llm_roles.pending.jsonl`. The tracked role files
(`roles.toml`, `roles_overrides.toml`, `llm_roles.jsonl`) are only read from there, so they must be checked out in the
main checkout: until the knowledge branch is merged and checked out there, set `KNOWLEDGE_OUT` to a checkout that has them
(for example the `knowledge-base` worktree's `data/knowledge`), and `KNOWLEDGE_REF` to the branch that has `cardgen
weekly-report` (for example `origin/knowledge-base`), because `origin/main` does not have this code yet. Logs of the last 12
runs are in `~/.cache/commander-sim-knowledge/runs/DATE/`.

**The LLM stage and the GPU.** The production model is the dense Qwen3.6-27B-Q5_K_M behind the user unit
`scripts/systemd/llama-27b.service` (tracked, not enabled; the coder `llama-server` service is not touched). The job:
if a server already answers on port 8080, it uses it only if its model id contains `Qwen3.6-27B` and never stops it (any
other model: skip, "other model on the port"); if the port is held but silent, skip; if something holds the B70's render
node open (`fuser` on `/dev/dri/by-path/pci-0000:04:00.0-render`, override `KNOWLEDGE_GPU_DEVICE`), skip, "GPU busy"; if the
unit is not installed, skip with the link command; otherwise it starts the unit (with a runtime limit, in case the job is
killed), waits up to 5 minutes for the model, sends a smoke prompt, runs the stage for at most 90 minutes (the rest carries
over to next week; every batch is flushed to the pending file as it finishes), and stops the unit only because it started
it (an `EXIT` and `TERM` trap, so a killed job stops its own server too). A start, readiness or smoke failure is a `fail` in
the report; the skips are not.

**The report** (`cardgen weekly-report`, in the cardgen README): Action needed (for JD's decks only: a card that became
banned or not legal; a Game Changer added or removed, with the deck's new count against the rule of each tier it lists,
`decks::gc_rules`: t0-t1 none, t2 one or two, t3 exactly three, t4-t5 unrestricted, which is the pool decks' rule and is
shown as information for JD's decks; a deck card the catalog does not know; Oracle errata against
`data/scryfall_cache.json`, meaning the engine's model may be stale; any failed stage) · Run · New cards (by set, then inside
each of JD's identities with their roles) · Bans/unbans and Game Changer changes · Role changes and LLM stats. On a failure
the script also calls `notify-send`, if it exists.

**Never automated:** commits, pushes, PRs, writes to any tracked file, `cardgen export`, sims, swap or cut proposals,
refreshing the cache with errata, llama.cpp or model updates, merging pending LLM rows (`roles --merge --pending`, in a
reviewed session, then `llm_roles.jsonl` is edited by hand), installing or enabling the timer.

**Install (JD, by hand; nothing here runs them):**

```
chmod +x ~/projects/commander-sim/scripts/knowledge-weekly.sh        # already executable in git
systemctl --user link ~/projects/commander-sim/scripts/systemd/llama-27b.service
systemctl --user link ~/projects/commander-sim/scripts/systemd/knowledge-weekly.service
systemctl --user link ~/projects/commander-sim/scripts/systemd/knowledge-weekly.timer
systemctl --user enable --now knowledge-weekly.timer
systemctl --user list-timers knowledge-weekly                          # next run
journalctl --user -u knowledge-weekly                                  # what a run printed
```

A manual run: `scripts/knowledge-weekly.sh --dry-run` (prints what each stage would do; writes nothing), or `systemctl --user
start knowledge-weekly.service` for a real one, or `scripts/knowledge-weekly.sh --stage 2` for one stage. To remove it:
`systemctl --user disable --now knowledge-weekly.timer`.

**Checks** (no network, GPU or systemd changes): `tests/weekly.rs` (the report), `tests/weekly_sync.rs` (the untracked lock,
`--updated-at`), `tests/weekly_script.rs` (a full offline run on a fixture bulk file, a failed stage, no binary, a failed
fetch, two runs at once, an interrupted run, and the LLM decisions against stubs of `curl`, `ss`, `fuser` and `systemctl`:
server already running, another model, GPU busy, quiet week, smoke failure, spent budget, a killed job), and
`systemd-analyze --user verify scripts/systemd/*`.

## Roles: the rule stage (phase 3)

`cardgen roles --rules` gives every deck card (35,014 of the catalog; Stickers and Attractions are skipped) its roles
from deterministic rules over its Oracle lines, with the evidence line for each and a `needs_llm` flag with route reasons
(`NoRole`, `UnknownLine`, `RejectedHit`, `RiskyRole`, `SameLine`, `Overflow`, `RegexError`). Every role the rules are
less than sure of, and every roleless card with a substantive line, routes to the LLM stage; a card the rules decide
alone keeps its rule roles. `roles --validate`, `--lint`, `--audit`, `--show` and `--stats` measure and inspect it, and
`find --role R` filters by role. The written standard is the header of `data/knowledge/roles_gold.toml`: the 26 role
definitions, 21 conventions and clauses C1-C18, every one ruled on by JD.

How it was measured, honestly:

| Set | Cards | Used for | Result (rules stage) |
|---|---|---|---|
| `roles_gold.toml` | 207 | tuning | 248 of 248 role assignments right: memorised, says nothing about generalisation |
| `roles_gold_regression.toml` | 113 | first a hidden acceptance set, scored four times and its failures fed back, so now regression only | precision 0.979, recall 0.960 |
| independent audits | 200 + 200 random cards, two unseen seed pairs | an estimate by a second labeller | cards the rules decide alone: precision 0.955 and 0.967, recall 0.955 and 0.983 when only clear errors count; precision 0.82 to 0.83 when debatable calls count against the rules |
| blind set 3 | 120, hidden | scored once, rules-only against rules plus LLM | not yet scored |

The first held-out half scored precision 0.851 and recall 0.841 against 0.989 and 1.000 on the tuning cards it had
been tuned on: that is the overfitting this process exists to catch.

Named limits of the rule stage (each is covered by a check or a later phase, none is accepted silently):
- The rules decide about 27% of legal cards alone. Routing is 73% of legal cards (23,773), because the rules are
  deliberately conservative: a routed card is re-checked and a confident wrong rule is not. The first LLM run is
  therefore about 4 to 5 hours of GPU time, paid again only when the prompt changes.
- A card with a real role plus a second function the rules never read can still be decided by the rules alone (2 of 52
  in one audit). The shadow audit in phase 3L re-checks a seeded sample of rules-decided cards for exactly this.
- Only 4 of 246 gold assignments are `certain`, so the LLM stage may veto most rule roles on routed cards.
- Roles are about what a card does, not whether it is good in a deck. The sim and JD decide that.

The local-LLM stage (phase 3L) is described in `rust/crates/cardgen/README.md` ("The LLM stage"): the prompt budget
(about 3.1k tokens for the system prompt, 3.4k with the worked-example turn), the validation and retry ladder, the cache keyed on the card text, the
prompt fingerprint and the model id (never the rules fingerprint), the merge policy and its disagreement reports. The
coordinator runs it (`llama up`, `roles --llm`, `roles --merge`, `roles --validate --stage merged`, `llama down`).

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
| 3 Roles, rule stage | `knowledge-roles` | done, merged into `knowledge-base` |
| 3L Roles, local LLM stage | `knowledge-llm` | built and measured against the real model (dense Qwen3.6-27B); the targeted production run over 12,520 cards started 2026-11-21 and its results are to be merged |
| 3b `map` and `digest` | `knowledge-digest` | done, merged into `knowledge-base` |
| 4 Status, decks and findings join | `knowledge-status` | done, merged into `knowledge-base` (adversarial tests, Opus review; no-record cards print `m:N`, Full-auto counts as below Full) |
| 5 Workflow in the rule files | `knowledge-workflow-doc` | section written and Opus-reviewed, merged into `knowledge-base`; JD signs off the wording before the final PR; linked from `cards.md` |
| 6 Weekly timer and report | `knowledge-weekly` | code, units, tests and this section written; the timer is JD's to install (`sync.lock` untracked, `--pending` merge default, 27B decisions of 2026-11-22) |

Branching: `knowledge-base` is the integration branch off origin/main. Each phase is built on its own branch
(`knowledge-<phase>`), and merged into `knowledge-base` only after its review chain passes: built by Sonnet,
adversarial tests by Sol, final review by Opus, checks re-run by the coordinator. `knowledge-base` goes to main by PR
at milestones, and JD decides when each merges.
