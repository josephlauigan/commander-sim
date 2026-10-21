# Task: karn-core — JD's new Karn, Legacy Reforged deck (key `karn`): commander, lands, wiring, AI plan, practice

Read `rust/agents/RULES.md` (skip its `git merge rust-port` step and its difftest/Python lines: there is no Python
any more; your worktree is based on branch `karn-deck`). Worktree:
`/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/karn-core`, branch `karn-core`.

Two other agents work at the same time on the same deck: `karn-threats` (threats, removal, wipes) and
`karn-engines` (card draw, engines, and checking the deck's already-modeled cards). Don't model their cards (lists in
`rust/agents/karn-threats.md` and `rust/agents/karn-engines.md`). Keep shared-file edits small and additive so the
three branches merge cleanly.

**Already done for you (on `karn-deck`):** the deck doc `decklists/JD/karn-colorless-artifacts.md` (read it: the
strategy and key lines are the AI's plan), the list `rust/agents/karn-list.txt` (100 lines), the `[karn]` entry in
`data/my_decks.toml` (colourless, `ident = ""`, placeholder tiers), and `cardgen deck` + `cardgen export`, so every
card is in `data/cards.json`. Oracle text: `data/scryfall_cache.json` or `curl -s -m 30
"https://api.scryfall.com/cards/named?exact=<name>"`. Model every card to its Oracle text and record
`status`/`note` in `data/card_overrides.toml` (`Full` only when every line is modeled; otherwise say what is missing).

## Do
1. **Wiring, as for Omnath** (commit `9a1f0ff`, `git show --stat 9a1f0ff`): every place a JD deck key is listed
   (grep `"omnath"` across `rust/`) gets `karn`: `sim-cli` (decks.rs, args.rs, poolmode.rs, report.rs), `ai/` (decks.rs,
   mod.rs, brain.rs), engine key lists, practice (`catalog.rs`, `session/setup.rs` and their tests, including the
   `my_decks_parsed.json` fixture if there is one). Check that a colourless deck (`ident = ""`) works everywhere:
   colour checks, mulligans, land colour choice, "Command Tower"-style code, practice's deck picker. Fix what breaks,
   with a test.
2. **Karn, Legacy Reforged** (the commander; the hard part): *Karn's power and toughness are each equal to the
   greatest mana value among artifacts you control. At the beginning of your upkeep, add {C} for each artifact you
   control. This mana can't be spent to cast nonartifact spells. Until end of turn, you don't lose this mana as steps
   and phases end.* The engine needs mana that only pays for artifact spells and for activated abilities (not for
   nonartifact spells) and lasts the whole turn. Find how the engine and practice handle restricted mana today
   (`engine/mana.rs`, `human/mana.rs` `add_restricted`, Secluded Courtyard, how `Mishra's Workshop` is played) and add
   the smallest general mechanism: e.g. a per-player "artifact-only mana this turn" amount that `can_pay` / payment
   use for artifact spells and ability costs. Both the AI engine and practice mode (the human pays from it) must
   use it. Tests: Karn with 4 artifacts adds 4; it pays an artifact spell and an ability cost; it can't pay a
   planeswalker or All Is Dust; it lasts through combat and is gone next turn; P/T follows the biggest artifact.
3. **Other restricted mana on the same mechanism:** Mishra's Workshop ({C}{C}{C} for artifact spells only; check
   what it does today and make it exact), The Mightstone and Weakstone ({T}: {C}{C} that can't cast nonartifact
   spells; ETB choose draw two or −5/−5).
4. **Lands:** Urza's Mine, Urza's Power Plant, Urza's Tower (Tron: 2/2/3 with all three), Urza's Workshop
   (metalcraft: {C} per Urza's land; Saga, Cave, Factory, Workshop and the Tron lands have subtype Urza's), Urza's
   Factory ({7},{T}: 2/2 Assembly-Worker token), Mishra's Factory and Mishra's Foundry (creature lands), Treasure
   Vault, Darksteel Citadel (artifact land, indestructible), Buried Ruin; check Urza's Cave, War Room (costs no life
   for a colourless commander), Bonders' Enclave, Gilded Lotus. Creature lands: follow whatever the engine already
   does for manlands (grep "becomes a" / existing manlands); if none, the AI animates to block a lethal attacker and
   to attack when unblocked damage is free, and the land counts as an artifact while animated.
5. **The AI plan** (`impls/`, a new `karn.rs` or the closest existing artifact-ramp plan; the Urza pool deck
   `urza-mono-blue-artifacts` and Godo's plan `t6_plan_godo.rs` are good references): cheap artifacts first, keep
   Baubles on the battlefield for Karn's mana, cast Karn as early as possible (with Greaves/Boots), spend Karn's
   mana every turn (artifact threats first, then ability sinks: Urza's Factory, Mishra's Factory, Staff of
   Domination), All Is Dust / Ugin, the Spirit Dragon when opponents have the coloured board, hold Oblivion Stone
   until behind. Expedition Map / Urza's Cave fetch the missing Tron piece. The threats/engines agents model the
   cards; you write the plan that uses them by name (it's fine if a card's code isn't there yet in your branch).
6. **Printings:** `data/printings/karn.txt` like the others; JD owns none of these, so use a common printing with
   `# placeholder until JD's copy is known`; mark the 8 proxies (see the doc) with `# proxy`.
7. **Practice mode:** the deck shows in the catalog, starts, and a human can pay with Karn's mana.

## Checks
`cargo test --release -j 7` (whole workspace, `--test-threads=7`) passes; `cargo build --release -j 7
--all-targets` has no warnings; `./target/release/commander-sim --deck karn --pool t3 --games 100 --jobs 7` runs
(no panic), the AI casts Karn and uses his mana (check the card report / `--cards`). Don't run longer sims.

Limits: at most 7 threads (`-j 7`), `nice -n 19`, memory cap `MemoryMax=8G` per RULES.md, no Python. Handoff
`rust/agents/handoff-karn-core.md` after your first step; commit after each piece (standalone, no Co-Authored-By);
don't push. Report in at most 15 lines; details in `rust/agents/report-karn-core.md`.
