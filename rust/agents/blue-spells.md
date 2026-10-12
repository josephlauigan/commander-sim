# Task: blue-spells — the Elrond deck's blue spells

Read `rust/agents/LASTDECK-RULES.md` first. Deck: `decklists/JD/elrond-ring-tempts.md` (key `elrond`), mono-blue.
Your module: `impls/bulk_blue.rs` (stub in place), tests `tests/cards_bulk_blue.rs`. You own: token copies, Jace
empower, spree, kicker, "put on its owner's choice of top or bottom of their library". The elrond-core agent writes
the deck's AI plan; you make the cards work and give each a sensible default use (the generic AI should cast them
well: target choice, when to hold an instant).

## Cards to model (until `cardgen audit --mine` shows each Full)
- **Token copies:** Relm's Sketching (a token copy of target artifact, creature or land: copy your best
  enter-the-battlefield creature, or the best permanent on the table); Echocasting Symposium (target player creates a
  token copy of target creature you control; paradigm: exile it after it resolves, and at the start of each of your
  first main phases you may cast a copy of it free). Find how token copies are already made (grep `copy` in
  `engine/zones.rs`, `engine/cast.rs`, `impls/rules2.rs`, `impls/t3.rs`) and reuse it.
- **Jace empower:** Keeper of the Quiet Hour is already Full, so the Jace token exists; find it (grep `empower`) and
  make it reusable by Jace, Reality Sculptor (its +1 empowers X = your Islands; its −3; its static "activate only
  if..." clause), and check Keeper still works. The elrond-core agent needs empower for Mindseeker Oculus: do this
  part first, commit, and note the function to call in your handoff file.
- **Spree / kicker / modes:** Phantom Interference (spree: a 2/2 flying Spirit and/or counter unless {2}); Whoosh!
  (kicker {1}{U}: draw); Amazing Acrobatics (counter and/or tap one or two creatures); Icy Reception (the -5/-0 mode).
- **Top or bottom:** Sudden Setback (target spell or nonland permanent), Vanish from Sight (surveil 1 too). The owner
  chooses: the AI picks the bottom unless it wants the card back (a removal-light rule is fine; say what you chose).
- **Others:** Kasmina's Transmutation (loses all abilities, base 1/1); Retrieve the Esper (3/3 Robot; flashback; two
  counters when cast from the graveyard); Rousing Read (draw two discard one; +1/+1 and flying); Unfathomable Truths
  (devoid; draw three and an Eldrazi Spawn); West Coast Expansion (draw X; with X 5 or more, cast a Hero from hand
  free); Sleight of Hand (look at two, keep one: replace the Approximate plain draw).

## Done when
The LASTDECK-RULES checks all pass, and every card above is Full. Report as those rules say; full report in
`rust/agents/report-blue-spells.md`.
