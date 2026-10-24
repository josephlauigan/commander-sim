# Task: kefka-cards — model Kefka's deck and write its AI plan

Deck: `decklists/JD/kefka-grixis-discard.md` (key `kefka`). Read its Strategy and Key lines first: they say how the
deck should play. Kefka himself (`Kefka, Court Mage // Kefka, Ruler of Ruin`) is already modeled (`impls/mine.rs`).

Your files: `rust/crates/sim-core/src/impls/kefka.rs` (new; register it in `impls/mod.rs`), its tests in
`rust/crates/sim-core/tests/cards_kefka.rs`, card data rows for these cards in `data/carddb.txt` /
`data/card_overrides.toml`. Shared files as the rules say.

## Cards to model (24), until `cardgen audit --mine` shows each as Full
Unmodeled: Arcane Omens (converge: count colours spent), Dark Inquiry, Decorum Dissertation (Paradigm: exiled, then a
free copy at the start of each of your first main phases), Demonic Embrace (cast from the graveyard for 3 life and a
discard), Duress, Ichor Slick (cycling; **madness**: when discarded, cast it for {3}{B}), Inner Fire.

Partial: Alpharael, Dreaming Acolyte (discard two unless you discard an artifact; deathtouch on your turn), Corroding
Dragonstorm (the return-to-hand clause can stay a no-op if no Dragon is in the list: say so), Disintegrate,
Extinguish the Light, Gemstone Mine (three mining counters, sacrificed when the last is used), Grab the Prize,
Intellect Devourer (each opponent exiles a card from hand while it's out; you may play those cards), Postmortem
Professor (return from the graveyard by exiling an instant or sorcery from it), Sawblade Scamp (oil counters on
noncreature spells; tap and remove one: 1 damage to each opponent), Scarlet Spider, Kaine (**mayhem**: cast from the
graveyard for {B/R} if you discarded it this turn), Screeching Soulbreaker, Splatter Technique (both modes), Stadium
Tidalmage, Stargaze, Susurian Voidborn (drain on your creature or artifact dying; **warp**), Twinned Vision (flashback
with a discard; draws two when not cast from hand). Approximate: Garrison Excavator (a 2/2 Spirit whenever cards leave
your graveyard).

Madness, mayhem and casting from the graveyard are mechanics: build them so any card can use them, in the engine's
discard and cast paths, with the smallest change. Psychosis Crawler and Kefka are listed as Full: check that Kefka,
Ruler of Ruin's draw trigger works with the Crawler (each card drawn makes each opponent lose 1, which draws you one
per opponent) and stops cleanly when your library runs out or the table is dead. A test for that loop.

Not yours: Blazing Firesinger (prepared; the zada-cards agent builds "prepared" and models it).

## AI plan for kefka (in `impls/kefka.rs`, wired into `ai/decks.rs` / `ai/brain.rs` like the other decks)
- **Discard choice** for Kefka's trigger and the looters: discard madness, mayhem and graveyard-castable cards first,
  and pick a card of a type not yet in the discard pile when that adds a draw.
- **The flip:** activate {8} as soon as it's affordable after the main-phase plays that matter, unless holding up
  interaction is clearly better.
- **Psychosis Crawler with Kefka, Ruler of Ruin:** start the loop only when your library has more cards than the
  lowest opponent's life (otherwise hold the Crawler or the flip).
- **Decorum Dissertation:** target an opponent once Kefka is flipped (their loss draws you cards), yourself before.
- Cast priorities for the new cards, sensible in the deck's plan.

Smoke run for this deck: `--deck kefka`.
