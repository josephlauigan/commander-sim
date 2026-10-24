# Task: cand-jenova — model the Jenova candidate deck and write its AI plan

Base branch: `rework-candidates`. Deck: `decklists/JD/candidates/jenova.md` (key `jenova`): Golgari +1/+1 counters and
sacrifice under Jenova, Ancient Calamity. The deck is wired into every deck list; your module is
`rust/crates/sim-core/src/impls/jenova.rs` (stub in place), tests in `rust/crates/sim-core/tests/cards_jenova.rs`.

**Not yours** (the cand-glissa agent models them; they may show as not Full until its work is merged): Bite Down,
Hard-Hitting Question, Cost of Brilliance, Knockout Maneuver, Culling Ritual, Witherbloom Charm, Vault Plunderer,
Armored Scrapgorger, Hardbristle Bandit, Noxious Newt.

## Cards to model (until `cardgen audit --mine` shows each as Full)
- **Jenova, Ancient Calamity** (1/5): *at the beginning of combat on your turn, put a number of +1/+1 counters equal
  to Jenova's power on up to one other target creature. That creature becomes a Mutant in addition to its other
  types. Whenever a Mutant you control dies during your turn, you draw cards equal to its power.*
- Unmodeled: Forced Adaptation, Lion Umbra (umbra armor). Partial: Adventurous Eater, Ambitious Augmenter, Dormant
  Grove, Edgar, Moonlit Sovereign, Efflorescence, Essenceknit Scholar, Hungry Graffalon, Old-Growth Educator,
  Teacher's Pest, Topiary Lecturer, Vinelasher Adept.

## AI plan for jenova
- Grow Jenova's power first (counter spells and creatures that target "a creature" should prefer Jenova), since her
  combat trigger gives counters equal to her power.
- Her trigger targets the best creature to make a Mutant: one with evasion or one you can sacrifice.
- Sacrifice Mutants on your own turn when the draw is worth it (power 3 or more), with the deck's sacrifice outlets
  (Carrion Feeder, Carnage Altar, Deadly Dispute, Witherbloom Charm, Lash of the Balrog, Stir Up Trouble, Silence the
  Echo, Cauldron of Essence, Thrashing Brontodon); removal spells with a sacrifice cost should use a Mutant.

Smoke run: `--deck jenova`.
