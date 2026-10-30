# Brief: Phase 2, PR 4: combos from Commander Spellbook

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/phase2-spellbook` on
branch `phase2-spellbook` (from origin/main 2bdfead0). Run every command from there. Read `CLAUDE.md` first: memory
cap, no Python (use `curl`, `jq`, shell or Rust), loose profile. Another agent works in parallel on
`phase2-coverage`; use `-j 12` for cargo and `--jobs 12` for any sim.

## Background

Phase 2 of `documents/realism.md` ("One card model") includes **"combos from Commander Spellbook"**. Today the sim's
combos are hand-written: `rust/crates/sim-core/src/impls/combos.rs` (a `Combo` catalogue of piece groups, `ready`,
`finish`) plus the `t6_combos_*.rs` and `cc_*.rs` files. Nobody has checked that catalogue against a real combo
database, so a deck can hold a line the sim never plays, and the catalogue can hold lines whose cards no deck
plays. See `audit/realism/casual-combos-2026-10-26.md` for how this was done by hand for five decks.

Commander Spellbook (commanderspellbook.com) has a public API at `https://backend.commanderspellbook.com/`. Check
its current endpoints yourself (for example the "find my combos" endpoint that takes a deck list, and `/variants/`);
don't guess the request format. Check its terms and licence, and credit it wherever its data is stored or reported.
Be polite: one request per deck, cached, no bulk crawling.

## Work

1. **Fetch.** For every deck in the sim (the pool lists under `decklists/pool/` and JD's decks under
   `decklists/JD/`, i.e. every deck in `data/decks.json`), ask Spellbook which combos the deck's cards contain.
   Save the results to `data/spellbook/` (one JSON per deck or one file, with the fetch date and the source), with a
   `cardgen` subcommand or a shell script under `scripts/` that refetches them. Store only what the sim needs: the
   combo id, its cards, its results (win the game, infinite mana, infinite damage ...), its prerequisites, and
   whether every card is in the deck ("included") or one card is missing ("almost included"; keep those too, but
   apart).
2. **Compare with the catalogue.** For each deck: each Spellbook combo the deck fully contains, and whether the sim
   has it (match by card set against `Combo.groups` and the other combo files; one piece per group). Then the
   reverse: each catalogue combo, and whether Spellbook knows it. Write `audit/phase2/spellbook-<date>.md`: per tier
   and deck, the lines Spellbook finds that the sim lacks, with their results and prerequisites, and the sim's lines
   Spellbook doesn't list (which may be fine: say why if you can tell). Flag the lines that matter most: two- and
   three-card lines that win the game, in decks with tutors.
3. **Guard (cheap).** A test that loads `data/spellbook/` and the catalogue and lists the uncovered "included" combos
   per deck, failing only if a combo whose pieces are all in the catalogue's card names is missing for a deck that
   plays them. If that's not workable, say why and leave the report as the check.
4. **Do not model new combos in this PR.** List them for JD; modelling them changes the tiers and is a later PR.

## Checks

- `cargo test --release -j 12` in `rust/` under the memory cap: all pass. `cargo build --release --all-targets`: 0
  warnings. `cardgen validate` passes; the sim's results don't change (no engine change: say so, and show one
  `--realism --pool t6 --games 500 --jobs 12` run byte-identical to origin/main if you touched any sim-core code).
- Spot-check five matches and five gaps by hand against the deck lists and the cards' Oracle text
  (`data/scryfall_cache.json`).

## Rules

- Commit on your branch as you go; do not push; no `Co-Authored-By` trailer. Never commit a secret.
- Keep a handoff file `audit/phase2/spellbook-handoff.md`.
- Report in 15 lines or fewer, with the full report as text at the end of your reply (the harness may refuse your
  write of `rust/agents/report-phase2-spellbook.md`; if it does, put the text in your reply).
- No recalibration and no deck changes.
