# Task: docs-jd-packages — bring three deck docs' prose in line with JD's 2026-10-13 packages

You edit THREE deck docs: `decklists/JD/kefka-grixis-discard.md`, `decklists/JD/sauron-grixis-amass.md` and
`decklists/JD/titania-lands.md`. Your worktree's branch `docs-jd-packages` was made from `jd-packages` (commit
8c71f7f), where the lists already have the packages (see the newest **Updated** line at the top of each doc). Read
`audit/jd-packages/2026-10-13/README.md` first: the swaps and the measurements.

- Kefka (A): Twinflame, Kaervek's Purge, Disintegrate out; Yawgmoth's Will, Vampiric Tutor, Mystical Tutor in.
  +1.0 over all tiers (within noise; the gain grows with tier strength, T4 +2.2, T5 +2.8). Will is a second storm
  turn: it replays Dark Ritual, Lotus Petal, Brain Freeze and Grapeshot from the graveyard Kefka's discard fills.
  Six Game Changers now (Demonic Tutor, Jeska's Will, Mystical Tutor, Rhystic Study, Underworld Breach, Vampiric
  Tutor), Bracket 4.
- Sauron (C): Bedevil out; Sheoldred, the Apocalypse in. +2.7 over all tiers (T1 +5.0, T2 +4.8). With Jace's
  Archivist's wheel and Orcish Bowmasters every opponent draw costs life. Still two Game Changers, Bracket 3.
- Titania (D): Yavimaya Wurm, Elfhame Wurm, Cactarantula, Penumbra Spider out; Vorinclex, Monstrous Raider, Kura,
  the Boundless Sky, Tireless Tracker, Cartographer's Survey in. +1.8 over all tiers (up in four of five). Landfall
  value: Tracker's Clues, Survey's two lands, Kura's three lands on death (or an X/X Spirit late), Vorinclex doubles
  your counters (Tracker, Topiary Lecturer, Vinelasher Adept, Knockout Maneuver, Thirsting Roots' proliferate) and
  halves opponents'. No Game Changers.

## Hard rules
- **Never run Python** (a hook refuses it); no builds or sims are needed. Use `grep`, `sed`, `jq`.
- Your worktree is `.claude/worktrees/docs-jd-packages`; cd into it for every command. Edit only the three docs.
  Don't touch the generated sections (`## Decklist by type`, `## Import list`) or the existing **Updated ...** log
  lines (they are history; leave them word for word, including the new 2026-10-13 ones).
- Write for JD (the decks' owner, who plays them in person). Match each doc's existing style: plain prose, short
  paragraphs, card names in bold where the doc does that. No "also in X deck" notes (decks never share cards).
- Every card you mention as in the deck must be in the doc's Import list; every cut card must leave the prose (unless
  the sentence says it's gone). Rules text: from `jq '.["card name lowercased"]' data/scryfall_cache.json`; don't
  invent card text. Numbers: only the ones above, in the README or in the Updated lines; don't invent results.
- Sources tables ("From the binder", "Bulk"...): the new cards came from the binder (Vampiric Tutor, Mystical Tutor,
  Kura, Tireless Tracker, Cartographer's Survey: the retired Jodah deck's leftovers) or are JD's own copies
  (Yawgmoth's Will, Sheoldred, the Apocalypse, Vorinclex). Cut cards went to bulk.

## What to update
- The **Simulator** line at the top: keep its form; say the new package was measured in a paired test (numbers
  above, 500 games per list per tier) and that a pool run is pending; mark older figures as the previous list's.
- Strategy, Key lines, Consistency counts (interaction, wipes, tutors, top end...), Tutor targets, Bracket and Rule 0
  (Game Changers and combos), card-source tables: rewrite what the swaps made wrong, keep what is still right.
- Grep each doc for each cut card and each new card, and recount any number you change from the Import list.
  `rust/target/release/cardgen deck <key> <list> --dry-run` in the jd-packages worktree printed the stale lines;
  re-find them with grep.

## Done
Commit (`git add decklists/JD/ && git commit -m "..."`, no Co-Authored-By line), never push. Final report at most 15
lines: line 1 DONE, line 2 your model ID, commit hash, what sections changed per doc, any count you could not check.
