# Task: kraum-phyrexian3 — add and model five Phyrexian cards for the Tymna/Kraum deck

Read `rust/agents/LASTDECK-RULES.md` first: its hard rules, modeling method and checks apply, with these changes:
- Your worktree is `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/kraum-phyrexian2`, branch
  `kraum-phyrexian2` (from origin/main). Don't merge anything in; don't push. Don't change any deck list.
- Builds and tests: `nice -n 19 systemd-run --user --scope -q -p MemoryMax=18G -p MemorySwapMax=0 cargo ... -j 12`.
  No sim runs except the smoke run below (`--jobs 12`). Skip the `all_decks` checks.

## Why
JD wants one more round of Phyrexian package tests for the `kraum` deck (Tymna the Weaver + Kraum, W/U/B/R, T3). The
deck loses mostly to creature decks in combat (Aurelia, Tergrid, Marwyn's elves). These five cards are not in the card
database (`commander-sim --swap` refuses them). Add them and make each **Full** in `cardgen audit --cards <file>`, with
at least one test per card, from its Oracle text (fetch from Scryfall's API if the local cache lacks it). See how the
last agent added cards: `rust/agents/report-kraum-phyrexian.md` (it added Dismember, Porcelain Legionnaire, Vault
Skirge, Apostle's Blessing; reuse its Phyrexian-mana payment and protection code).

## Cards
- **Archfiend of the Dross** ({2}{B}{B} 6/6 flying Phyrexian Demon; enters with four oil counters; at the beginning of
  your upkeep remove one, then if it has none you lose the game; whenever a creature an opponent controls dies, its
  controller loses 2 life). The losing clause must be real. AI: cast it, attack with it, and before the last counter
  would go, get rid of it if it can (blink it with Restoration Angel resets the counters as a new object; sacrifice or
  bounce if available); if nothing can save it, the game is lost, as on the card. Report how often that happens in the
  smoke run.
- **Skrelv, Defector Mite** ({W} legendary 1/1 artifact creature, toxic 1, can't block; {W/P}, {T}: another target
  creature you control gains toxic 1 and hexproof from the colour of your choice until end of turn, and can't be blocked
  by creatures of that colour this turn). AI: use it in response to targeted removal on Tymna, Kraum or the best
  creature (choose the removal's colour), or before attacks to make the best attacker unblockable by the main blocker
  colour. Toxic/poison counters may already exist in the engine (grep `toxic`, `poison`).
- **Gut Shot** ({R/P} instant: 1 damage to any target). AI: kill a 1-toughness creature (elves, tokens, mana dorks),
  else hold; pay 2 life instead of {R} when red is short.
- **White Sun's Twilight** ({X}{W}{W} sorcery: you gain X life; create X 1/1 colourless Phyrexian Mite artifact creature
  tokens with toxic 1 and "This creature can't block"; if X is 5 or more, destroy all other creatures). AI: cast it with
  X ≥ 5 as a wipe when behind on board, else for life and bodies late.
- **Phyrexian Unlife** ({2}{W} enchantment: you don't lose the game for having 0 or less life; as long as you have 0 or
  less life, all damage dealt to you is dealt as though its source had infect). Poison counters must matter here.

Put the card code in `impls/tymna_cards.rs`, tests in `tests/cards_tymna.rs`. Run `cardgen export` after adding data.

## Done when
`cargo test --release -j 12` passes, `cargo build --release -j 12 --all-targets` has no warnings, all five cards are Full
in `cardgen audit --cards`, and this smoke run finishes:
`nice -n 19 systemd-run --user --scope -q -p MemoryMax=18G -p MemorySwapMax=0 rust/target/release/commander-sim --deck kraum
--pool t3 --profile loose --games 40 --ai lookahead --jobs 12 --swap "Elemental Mascot=>Archfiend of the Dross" --swap
"Tocasia's Welcome=>Skrelv, Defector Mite" --swap "Expressive Iteration=>Gut Shot" --swap "Memory Trap=>White Sun's
Twilight" --swap "Arcane Denial=>Phyrexian Unlife"`.
Report in at most 15 lines (line 1 DONE or HANDOFF, line 2 your model ID); full report in
`rust/agents/report-kraum-phyrexian3.md`, committed.
