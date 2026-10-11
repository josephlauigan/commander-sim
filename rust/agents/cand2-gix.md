# Task: cand2-gix — model the Gix candidate deck and the black cards the Yawgmoth and B/R decks share

Base branch: `rework-candidates`. Deck: `decklists/JD/candidates/gix.md` (key `gix`): mono-black creatures that
connect, each hit a card with Gix, Yawgmoth Praetor (already modeled). Your module: `impls/gix.rs` (stub in place),
tests `tests/cards_gix.rs`. Several cards are also in other candidate decks: you model them, the other agents won't.

## Cards to model (until `cardgen audit --mine` shows each as Full)
Shared: Al Bhed Salvagers, Apex Witchstalker, Callous Inspector, Corpulent Corpse (suspend; fear), Dark Matter
Manipulator, Gollum, Silent Slinker, Grim Wanderer, Necrosquito (oil counters), Restless Bloodseeker (Blood tokens),
Salvaged Manaworker, Sangrophage, Overkill, Key to the Side-Door.
Gix deck only: Depressurize, End of the Hunt, Balustrade Spy, Deepcavern Imp (echo by discarding), Gravblade Heavy,
Mask of Memory, Medic's Kitesail, Ulna Alley Shopkeep (infusion).

## AI plan for gix
- Gix's trigger: pay 1 life and draw for each creature that connects while life is comfortable (above about 12).
- Attack with the evasive creatures (flying, menace, fear) whenever they connect safely; equipment that grants evasion
  goes on the best attacker.
- Gix's {4}{B}{B}{B} ability: use it late, discarding excess lands, against the opponent with the biggest library top.

Smoke run: `--deck gix`.
