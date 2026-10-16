# Task: alela-cards — model the seven new cards of Avery's Alela deck

Read `rust/agents/LASTDECK-RULES.md` first: its hard rules, modeling method and checks apply, with these changes:
- Your worktree is `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/alela-cards`, branch
  `alela-cards`, made from `alela-doc-sync` (the new Alela list is already in `decklists/Avery/alela-esper-faeries.md`
  and `data/`). Don't merge anything else in; don't push.
- **The machine is shared and busy:** every build and test runs as
  `nice -n 19 systemd-run --user --scope -q -p MemoryMax=18G -p MemorySwapMax=0 cargo ... -j 5`, and smoke runs use
  `--jobs 5`. No batch sims beyond the smoke run below; skip the `all_decks` checks (too heavy right now).
- Put card code in `rust/crates/sim-core/src/impls/alela.rs` (already registered) and tests in
  `rust/crates/sim-core/tests/cards_alela.rs`. Shared files only for the smallest additive change, listed in your
  report.

## Why
JD wants the simulation of the Alela deck (key `alela`, Esper Faerie fliers: Alela makes a 1/1 flying Faerie whenever
you cast an artifact or enchantment spell, and gives other fliers +1/+0) to be accurate. `cardgen deck` reported these
seven cards of the new list as not fully modeled. Make each **Full** in `cardgen audit --mine`, with at least one test
per card, from its Oracle text (`jq '.["name lowercased"]' data/scryfall_cache.json`).

## Cards
Unmodeled:
- **Credit Voucher** ({2} artifact): {2}, {T}, sacrifice: shuffle any number of cards from your hand into your library,
  then draw that many. AI: activate when the hand holds cards the AI rates low (extra lands late, dead cards); put back
  only those.
- **Mirage Mirror** ({3} artifact): {2}: becomes a copy of target artifact, creature, enchantment or land until end of
  turn. Grep for existing copy machinery first (token copies, clones; blue-spells owned "token copies"). AI: before
  attacks, copy the best creature on the battlefield when that makes a good attacker; on an opponent's turn, a good
  blocker. Becoming a copy doesn't make it enter (no enters triggers) and it is not cast.
- **Scrapheap** ({3} artifact): whenever an artifact or enchantment is put into your graveyard from the battlefield, you
  gain 1 life. (Tokens never reach the graveyard: Faeries don't count.)

Partly modeled (finish them):
- **Angelic Page** ({1}{W} 1/1 flier): {T}: target attacking or blocking creature gets +1/+1 until end of turn. AI: use
  it in combat where the +1/+1 wins or saves a creature, else on any attacker.
- **Cabal Patriarch** ({3}{B}{B}{B} 5/5 legend): {2}{B}, sacrifice a creature: target creature gets −2/−2 until end of
  turn; {2}{B}, exile a creature card from your graveyard: same. AI: kill opposing creatures with toughness ≤ 2
  (several activations for bigger ones when worth it); sacrifice Faerie tokens first, never Alela.
- **Serra the Benevolent** ({2}{W}{W} planeswalker, loyalty 4): +2: your fliers get +1/+1 until end of turn; −3: 4/4
  white Angel token with flying and vigilance; −6: emblem "If you control a creature, damage that would reduce your
  life total to less than 1 reduces it to 1 instead." Model the emblem. AI: +2 on turns you attack with fliers, −3
  when the board needs a body or Serra would die, −6 when available.
- **Wall of Junk** ({2} 0/7 artifact creature, defender): when it blocks, return it to its owner's hand at end of combat
  (only if it is still on the battlefield). AI: block with it freely; recast it on your next main phase (it is an
  artifact spell, so Alela makes a Faerie each time).

## Also check the deck AI (fix in `alela.rs` if small; otherwise report)
These synergies decide how the deck plays; check each with a test or a single-game trace:
1. **Skullclamp on Faeries:** with Alela out a Faerie is a 2/1; equipping Skullclamp ({1}) kills it and draws two.
   Does the AI do this when it has spare mana? Also on other x/1 creatures that have done their job.
2. **Angelic Shield** (+0/+1 to your creatures; sacrifice: return target creature to its owner's hand) stops the
   Skullclamp trick. Does the AI use the sacrifice sensibly (save its own creature, or bounce a threat)?
3. **Crystal Chimes** ({3}, {T}, sacrifice: return all enchantment cards from your graveyard to your hand): does the AI
   activate it once two or more enchantment cards are there?
4. **Lightning Greaves** on Alela: does the AI move it off before targeting Alela with its own Aura or Daydream?

## Checking a card's audit status
`cardgen audit --mine` reads the deck lists; the new cards are already in the Alela list, so run
`rust/target/release/cardgen audit --mine | sed -n '/== alela/,/^== /p'` directly. Never use `cardgen audit --md` in
the worktree (it rewrites the deck doc). Don't change the deck list. After changing card data, run `cardgen export`.

## Done when
`cargo test --release -j 5` passes, `cargo build --release -j 5 --all-targets` has no warnings, all seven cards are
Full in the audit, and a smoke run finishes:
`nice -n 19 systemd-run --user --scope -q -p MemoryMax=18G -p MemorySwapMax=0 ./target/release/commander-sim --deck
alela --pool t3 --profile loose --games 40 --ai lookahead --jobs 5 --brief`.
Report in at most 15 lines (line 1 DONE or HANDOFF, line 2 your model ID); full report, including the answers to the
four AI checks, in `rust/agents/report-alela-cards.md`, committed.
