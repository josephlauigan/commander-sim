# Task: retire-zada — remove the Zada deck and the seven rework candidates

Read `rust/agents/OVERNIGHT-RULES.md` first: its hard rules (never run Python, memory cap, no wait loops, commit
often, handoff, 15-line report) apply. A hook refuses any command starting with python, python3, pip, pipx or uv.

Base branch: `titania-deck` (57ef99b). JD dropped the Zada deck and every candidate to replace it; the Titania deck
(key `titania`) takes the slot. Remove these eight deck keys completely: `zada`, `torbran`, `glissa`, `jenova`, `gix`,
`yawgmoth`, `lordofpain`, `smaug`. **Every remaining deck (seph, veyran, sauron, yshtola, galadriel, alela, jodah,
kefka, titania) and every pool deck must play exactly as before.** The retire-bill task did the same for one deck:
read `rust/agents/retire-bill.md` and `rust/agents/report-retire-bill.md` for how it went.

## Before you change anything: the invariance baseline
On the untouched base, for each remaining deck run `./target/release/commander-sim --deck KEY --pool t3 --profile
loose --games 48 --seed 500000 --ai lookahead --jobs 22 --brief` and save each output. After your change, rerun and
compare: the outputs must be identical apart from the timing line. If one differs, find out why and fix it.

## Remove
- Deck docs: `decklists/JD/zada-mono-red-go-wide.md` and the whole `decklists/JD/candidates/` folder; their entries in
  `data/my_decks.toml` and `tests/fixtures/my_decks_parsed.json`; then rerun `cardgen export`.
- Each key from every registration site (the same sites as retire-bill: sim-cli `decks.rs` FILES/MINE/test count and
  `args.rs`, `ai/brain.rs` GENERIC_PLAYS and option dispatch, `ai/decks.rs` dispatch arms, `ai/mod.rs` MAIN_STYLES,
  `engine/cast.rs`, `impls/rules.rs` CTHRESH, `settings.rs` cthresh, `impls/mod.rs`, practice `catalog.rs` (keep the
  parser working for the remaining keys) and the practice/game test counts and key lists). The practice test that
  reads win rates checks `zada`: drop it from that list.
- Deck-only AI for those keys: plans, priorities, and every branch gated on one of the eight keys (e.g. `key ==
  "gix"` in `impls/t3.rs`, `impls/partials.rs`, `dsl/mod.rs` `equip_options`; grep for each key as a string). Remove
  the gated branch, keeping the other side exactly.
- Card code used only by the removed decks: delete it, with its tests. A card stays if any remaining deck or any pool
  deck (`decklists/pool/`) plays it; then keep its code and rebuild its tests on a remaining deck key (or a testkit
  board that doesn't depend on a deck key). Check card by card with the deck lists; `cardgen audit --mine` and
  `cardgen audit --deck <pool key>` show what each remaining deck relies on. Machinery used by remaining cards
  (bites and fights in `glissa.rs`, increment and prepared in `jenova.rs`, plot, foretell, and so on) stays; move it to a
  sensibly named module if its file would otherwise be named after a removed deck, and fix the doc comments.
- Rules fixes made during the candidate work stay (Yawgmoth's protection from Humans for the pool deck, the life-gain
  merge, Garruk's Uprising tokens, etc.): they're not deck plans.

## Done when
Full `cargo test --release` passes, no warnings on `cargo build --release --all-targets`, `all_decks adaptive 3` and
`lookahead 1` exit 0, `cardgen audit --mine` lists only the nine remaining decks with the same "not Full" counts as
before (titania 0, yshtola 1, alela 2, jodah 4, the rest 0), a grep for each removed key as a quoted string in
`rust/crates` finds nothing, and the nine invariance outputs are identical. Report: line 1 DONE or HANDOFF, line 2
your model ID, then the commit, the checks, the invariance result, and anything the lead must decide.
