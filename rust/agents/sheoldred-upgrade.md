# Task: sheoldred-upgrade — model the Sheoldred upgrade package (4 cards) so it can be swap-tested

Read `rust/agents/OVERNIGHT-RULES.md` first: its hard rules (never run Python, memory cap, edit only your files,
commit often, handoff, 15-line report) apply here. Where it says "Kefka/Zada", read "Sheoldred".

**Sharing the machine:** another session runs sims in parallel. Build and test with `-j 8`
(`cargo test --release -j 8`), and use `--jobs 6` for smoke runs.

Base branch: `sheoldred-deck` (176d564). Deck: `decklists/JD/sheoldred-mono-black.md` (key `sheoldred`), fully
modeled; its module `impls/sheoldred.rs` and tests `tests/cards_sheoldred.rs` already exist (read
`rust/agents/report-sheoldred-cards.md` for how they're built). Add your cards there, tests in the same file.

The four cards are **not in the deck list**: JD is considering buying them. They'll be tested with `commander-sim
--swap`, so they must work when swapped in. Don't change the deck list.

## Cards to model (whole Oracle text, a test each, verdict Full in `data/card_overrides.toml`)
Oracle text: `curl` Scryfall if `data/scryfall_cache.json` doesn't have them (`https://api.scryfall.com/cards/named?exact=...`).
- **Seizan, Perverter of Truth** ({3}{B}{B} legendary 6/5): at the beginning of each player's upkeep, that player loses
  2 life and draws two cards. Those draws must fire Sheoldred (both sides) and Underworld Dreams.
- **Underworld Dreams** ({B}{B}{B} enchantment): whenever an opponent draws a card, it deals 1 damage to that player.
  Damage, not life loss: check how the engine tells them apart and whether it matters (prevention, lifelink).
- **Dark Deal** ({2}{B} sorcery): each player discards all the cards in their hand, then draws that many cards minus one.
- **Painful Lesson** ({2}{B} sorcery): target player draws two cards and loses 2 life.

## AI plan (gated to `key == "sheoldred"`)
- **Seizan:** cast only while Sheoldred is out (without her it mostly helps opponents), unless your life is high and
  you're far behind on cards. Champion's Helm can go on Seizan when Sheoldred already has protection.
- **Underworld Dreams:** cast early, any time (a high priority after Sheoldred).
- **Dark Deal:** cast while Sheoldred is out when opponents hold enough cards to matter (about 4 or more in total)
  and your own hand is small; otherwise hold.
- **Painful Lesson:** with Sheoldred out, target the opponent the drain could kill first (lowest life); otherwise
  yourself (like Cost of Brilliance's rule already in `impls/sheoldred.rs`).

## Check the swap works
These swaps must run (the package, as one block):
`--swap "Ribskiff=>Seizan, Perverter of Truth" --swap "Locust Spray=>Painful Lesson" --swap "Gravblade Heavy=>Underworld
Dreams" --swap "Multiply by Zero=>Dark Deal"`. Find how a card outside every deck gets into the card data (`cardgen
export`, `data/carddb.txt`, the Scryfall cache) and make sure all four resolve with your code, not a bare Scryfall body.
Smoke: `--deck sheoldred --pool t3 --profile loose --games 24 --ai lookahead --jobs 6 --brief` with those swaps.

## Done when
The OVERNIGHT-RULES checks all pass (full `cargo test --release`, no warnings, `all_decks adaptive 3` and
`lookahead 1`, the audit still `sheoldred: 0 not Full`, the 48-game smoke run with `--deck sheoldred`), plus the
swap smoke above. Report as those rules say: line 1 DONE or HANDOFF, line 2 your model ID. Full report in
`rust/agents/report-sheoldred-upgrade.md`.
