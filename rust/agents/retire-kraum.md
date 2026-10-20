# Task: retire-kraum, Tymna/Thrasios replaces Tymna/Kraum (key `kraum` goes away)

Read `rust/agents/RULES.md` (skip `git merge rust-port`). Worktree:
`/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/retire-kraum`, branch `retire-kraum` (from
`thrasios-deck`: the Thrasios deck, its AI and the Kefka/Omnath list changes are already there; see
`rust/agents/report-thrasios-deck.md`, `report-thrasios-ai.md`, `audit/thrasios/FINDINGS.md`).

JD (10/19) switches Tymna's partner from Kraum to Thrasios. The **deck** `kraum` goes; the `thrasios` deck stays at
tier t5. Do it the way Titania was retired today: read `rust/agents/retire-titania.md` and commits `98def04`,
`9428718`, `df493e8`, `475293d` for the shape. **Keep all card code** (Kraum's card, the Tymna partner plan and the
`"kraum" | "thrasios"` gates can stay if harmless; say what you kept). Don't touch the Tier 6 pool deck
`kraum-tymna-blue-breach-midrange` (a different key).

## Do
1. **Deck key:** remove `kraum` from `data/my_decks.toml`; `cardgen export` (decks.json) and the
   `my_decks_parsed.json` fixture through cardgen's own commands; remove `kraum` from every deck-key list in `rust/`
   (grep `"kraum"`: practice catalog, sim-cli, ai, settings, engine). Deck counts 14 -> 13.
2. **Tests:** tests that seat `kraum` move to `thrasios` where the cards are in Thrasios's list (Tymna, equipment,
   Dark Endurance, the defence cards); tests about Kraum-only cards (Kraum, Gilgamesh, Underworld Breach, the red
   cards) use a hand-built seat on another key or are dropped with a note in the report.
3. **Docs:** `decklists/JD/tymna-kraum-partners.md` is removed; `decklists/CHANGELOG.md` gets a "Retired 2026-10-19:
   became Tymna/Thrasios" line in the Kraum section(s) (keep past entries); `decklists/JD/tymna-thrasios-partners.md`
   gets a one-line "Built from the Tymna/Kraum deck (2026-10-19)" note. `audit/kraum/FINDINGS.md` gets a final
   "Retired: see audit/thrasios/FINDINGS.md" line. Update any README / `documents/` list of JD's decks. Root
   `CLAUDE.md`, "Opponent tiers": "yshtola, sephfs and kraum t5" becomes "yshtola, sephfs and thrasios t5".
   `decklists/JD/omnath-landfall.md` "Cards to confirm": drop Terror of the Peaks (Tymna's copy now goes to Omnath).
4. **Printings:** remove `data/printings/kraum.txt`; for the cards Thrasios kept from Kraum, `thrasios.txt` should
   already carry Kraum's printings (check; fix any that differ).
5. **Physical cards** (`bulk-cards.txt`, `binder-cards.txt`; `1x Name (set)` lines, sorted, sets from
   `data/printings/kraum.txt` / `kefka.txt`):
   - **To bulk** (cut from Kraum): Kraum, Ludevic's Opus; Underworld Breach; Gilgamesh, Master-at-Arms; Expressive
     Iteration; Talisman of Conviction; Lightning, Security Sergeant; Lightning, Army of One; Bedevil; Marchesa, the
     Black Rose; Black Widow, Agile Avenger; Firebird, Blazing Ranger; Battlefield Forge; Clifftop Retreat; Dragonskull
     Summit; Furycalm Snarl; Rugged Prairie; Spectator Seating; Sulfur Falls; 3 Mountain. And **Hallway Heckler**
     (cut from Kefka for Kuja).
   - **Not to bulk:** Glorybringer, Thundermaw Hellkite, Smaug the Magnificent (now in Omnath), Terror of the Peaks
     (Omnath's copy).
   - **Out of bulk/binder** (now in Thrasios's list), one copy each: Yavimaya Coast, Brushland, Llanowar Wastes,
     Deathcap Glade, Faeburrow Elder, Avacyn's Pilgrim, Dragonlord Dromoka, Witherbloom Charm, Coiling Oracle, Paradox
     Surveyor, and 3 Forest.
   - The new buys (Thrasios, Walking Ballista, Rings of Brighthearth, Breeding Pool, Undergrowth Stadium, Sunpetal
     Grove, Talismans of Unity and Curiosity, Kuja) are in neither file. Basalt Monolith: leave as is and note it
     (JD's one copy is also listed in Godo; JD decides). Report every line you changed.

## Checks
`cargo test --release -j 6` (whole workspace) passes; `cargo build --release -j 6` no warnings; `commander-sim --deck
kraum ...` is refused; `--deck thrasios --pool t5 --games 50 --ai adaptive --jobs 6` runs. Practice lists 13 decks
with thrasios. Limits: `-j 6`, `nice -n 19`, memory cap, no Python, no other sims. Handoff
`rust/agents/handoff-retire-kraum.md` early; commit after each piece (standalone, no Co-Authored-By); don't push. Report
in 15 lines or fewer; details in `rust/agents/report-retire-kraum.md`.
