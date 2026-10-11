# Report: retire-bill (branch retire-bill, base bill-deck eee6900)

Result: DONE. Bill deck removed; module `impls/bill.rs` -> `impls/green_cards.rs`; tests `cards_bill.rs` -> `cards_green.rs`.
Titania 240-game check (`--deck titania --pool t2 --profile loose --games 240 --seed 500000 --ai lookahead --jobs 22`):
27.5% before and after, output identical (diff of the whole report, apart from the timing line).
Checks: `cargo test --release` all green; `build --all-targets` no warnings; `all_decks adaptive 3` and `lookahead 1` exit 0;
`cardgen audit --mine`: no bill, `titania: 66 unique cards, 0 not Full`; `grep -rn '"bill"' rust/crates` empty.

## Removed
Deck doc, `[bill]` in my_decks.toml, fixture entry (fixture file keeps its no-trailing-newline form), `cardgen export` rerun;
key from sim-cli decks.rs/args.rs, brain.rs (GENERIC_PLAYS, special_options), decks.rs (deck_prio, protect_response),
ai/mod.rs (MAIN_STYLES, forced-plan block), cast.rs, rules.rs CTHRESH, settings.rs, practice catalog.rs and the counts in
practice tests / game.rs (90 -> 85). From green_cards.rs: bill_prio, finisher_prio, bill_protect, harrow_in_response,
land_adjust (+ call in turn.rs), forced_plan and the lethal flag, bill_special, the `choose_target` gate in dsl/mod.rs.

## Kept although they look Bill-only (so Titania plays exactly as before)
- `bill_options` (the doubling, incl. the pre-combat "(lethal attack)" option and `damage_through`/`lethal_victim`): it is
  registered on the card, so Titania used it. Only the Bill-only forced attack after it is gone.
- `counter_target`/`counter_score`: Additive Evolution and Sample Collector call it for every deck (the `doomed` term was
  only ever set by the removed Harrow response, so it is dropped).
## Dead code removed that changes nothing for Titania, but note it
`radical_up` (Greenbelt Radical turned face up for {5}{G}{G}) was reachable only through `bill_special`; Titania never
turned the Radical up, so it stays face down with ward {2}. The audit still says Full; the face-up half is no longer
modeled in code. If JD wants Titania to turn it up, add it to `titania_special` (that changes Titania's results).
Also: `deck_prio` for a plotted card / the Radical in Titania is the generic one (the old 0-priority rules were Bill's).

## Tests
cards_green.rs: 42 tests (was 51). Deleted: landfall target choice (2), fetch sequencing (3), Harrow in response (2),
Stampede/End-Raze hold, forced attack after the lethal doubling, and the three "deck_prio is 0" tests. Royal Treatment is
now tested through `royal_apply` (+1/+1, ward 1, no counter); Evolution Sage asserts the total (4 or 5).
## Shared-file changes (deletions/renames only)
cardcode.rs, engine/{cast,removal,turn,zones}.rs, dsl/mod.rs, ai/{mod,brain,decks}.rs, impls/{mod,rules,jodah,titania}.rs
(paths), settings.rs.
## Left alone
History docs (pool-results.md, audit/bill/, agents/bill-*.md) still mention Bill.
