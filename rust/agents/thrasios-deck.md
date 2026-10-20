# Task: thrasios-deck, add JD's Tymna/Thrasios list (key `thrasios`) and model Kuja for Kefka

Read `rust/agents/RULES.md` (skip its `git merge rust-port` step: the worktree is on main). Worktree:
`/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/thrasios-deck`, branch `thrasios-deck`.

JD is thinking of switching Tymna's partner from Kraum to **Thrasios, Triton Hero** (W/U/B/G), trading cards with
the Omnath deck. He also bought Kuja, Genome Sorcerer for Kefka. The list is final for this test:
`rust/agents/thrasios-list.txt` (100 lines, basics repeated). It is the current `kraum` list with the 23 red cards
swapped for green/colourless ones, plus a Basalt Monolith + Rings of Brighthearth + Walking Ballista combo.

## Do
1. **Card data.** Two cards aren't in `data/cards.json`: **Undergrowth Stadium** (B/G land, enters tapped unless you
   have two or more opponents: model it like Spectator Seating, which is already in the db) and **Talisman of
   Unity** (like Talisman of Curiosity: {T}: {C}, or {G}/{W} for 1 damage). Oracle text from
   `data/scryfall_cache.json` or Scryfall (`curl -s -m 30`). Check `Breeding Pool`'s model (a shock land: tag `t`
   looks like "enters tapped"; JD's AI should pay 2 life when it needs the mana that turn, if the engine supports it
   for other shocks; if not, leave it and say so).
2. **Kuja, Genome Sorcerer // Trance Kuja, Fate Defied** ({2}{B}{R}, legendary 3/4 Human Mutant Wizard; JD's
   **Kefka** deck): "At the beginning of your end step, create a tapped 0/1 black Wizard creature token with
   'Whenever you cast a noncreature spell, this token deals 1 damage to each opponent.' Then if you control four or
   more Wizards, transform Kuja." Back: Trance Kuja, Fate Defied, 4/6 Avatar Wizard: "Flare Star — If a Wizard you
   control would deal damage to a permanent or player, it deals double that damage instead." Model it Full with
   tests (token, ping on noncreature casts, transform at 4+ Wizards counting Kefka's other Wizards, doubling for
   Wizard sources only). Do NOT change Kefka's list: I test Kuja with `--swap`.
3. **The deck `thrasios`.** Add it alongside `kraum` (Kraum stays; the loser is retired after JD sees the
   comparison). Follow how `kraum` is wired: grep `"kraum"` across `rust/` and `data/` (my_decks.toml, decks.json via
   `cardgen export`, `sim-cli` deck/arg lists, `ai/` deck plans, practice catalog + tests, fixtures). The Tymna plan
   (attack spread, protection, priorities) is shared by key list, not copied. `[thrasios]` in `data/my_decks.toml`:
   name "Tymna/Thrasios", commander Tymna the Weaver, commander2 Thrasios, Triton Hero, ident WUBG,
   `tiers = ["t5"]`. Doc `decklists/JD/tymna-thrasios-partners.md` written like `tymna-kraum-partners.md` (current
   list + strategy only). Use `cardgen deck` (import list, by-type, legality/identity/Game Changer check).
   Printings: `data/printings/thrasios.txt` from `kraum.txt` for shared cards, bulk/binder sets for collection
   cards, `# placeholder until JD's copy is known` for the rest.
4. **Thrasios and the combo for a JD deck.** Thrasios and the Basalt/Rings/Ballista pieces have code in
   `impls/t6_*.rs`, `impls/combos.rs` (power_artifact) and `tests/combo_outlets.rs`, written for pool and Tier 6
   decks. Check each works when the `thrasios` deck holds the cards, and fix any gating by deck key or pool flag:
   (a) Thrasios's {4} ability is used as a mana sink (spare mana at the end of an opponent's turn, or on your turn
   once nothing better is castable); (b) Basalt + Rings with the mana for the first loop is infinite colourless
   mana, and the AI then wins through Walking Ballista (on the battlefield or castable) or, without Ballista,
   draws with Thrasios; (c) Worldly Tutor can fetch Ballista when Basalt + Rings are out. Tests for each.
5. **Other draft cards** (Faeburrow Elder, Witherbloom Charm, Paradox Surveyor, Coiling Oracle, Avacyn's Pilgrim,
   Dragonlord Dromoka, Deathcap Glade, the painlands) were modeled for the 10/13 Thrasios build
   (`impls/tymna_cards.rs`): confirm with `commander-sim --deck thrasios --cards` that nothing is Partial or
   Unmodeled; fix anything that is.

## Checks
`cargo test --release -j 6` passes; `cargo build --release -j 6` has no warnings;
`./target/release/commander-sim --deck thrasios --pool t5 --games 100 --ai adaptive --jobs 6` runs, and the card
report shows Thrasios activating and the combo assembling at least sometimes; same for
`--deck kefka --pool t3 --games 100 --swap "Rank Rat=>Kuja, Genome Sorcerer" --jobs 6` (Kuja cast, tokens made).
Don't run longer sims: I will.

Limits: at most 6 build threads (`-j 6`, `--test-threads=6`), `nice -n 19`, memory cap per RULES.md, no Python.
Handoff `rust/agents/handoff-thrasios-deck.md` after your first step; commit after each piece (standalone, no
Co-Authored-By); don't push. Report in at most 15 lines; details in `rust/agents/report-thrasios-deck.md`.
