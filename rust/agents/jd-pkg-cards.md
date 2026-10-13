# Task: jd-pkg-cards — model Vorinclex and check the AI for four card packages (2026-10-13)

Read `rust/agents/LASTDECK-RULES.md` first: its hard rules, modeling method and checks apply, with these changes:
- Your worktree is `.claude/worktrees/jd-pkg-cards`, branch `jd-pkg-cards` (made from `jd-packages`, which is
  `cloud-midgar` + this brief). Don't merge anything else in, don't push.
- **Shared machine, strict CPU cap:** every build and test runs as `nice -n 19 systemd-run --user --scope -q -p
  MemoryMax=8G -p MemorySwapMax=0 cargo ... -j 3`, smoke runs with `--jobs 3`. Never more than 3 threads.
- **No batch sims**: no `--analyze`, no `--swap` runs, no runs over 48 games. Unit tests, single-game `--trace`
  runs and the checks' smoke runs only. Skip check 3 (`all_decks`) — too heavy for the shared machine.
- No other agent works on these files right now, but keep shared-file edits small and listed anyway.

JD is testing four card packages (one paired sim each, run by the main session later):

| Deck (key) | In |
|---|---|
| Kefka (`kefka`) | Yawgmoth's Will, Vampiric Tutor, Mystical Tutor, Duress |
| Veyran (`veyran`) | Underworld Breach, Terror of the Peaks, Mirari |
| Sauron (`sauron`) | Sheoldred, the Apocalypse |
| Titania (`titania`) | Vorinclex, Monstrous Raider, Tireless Tracker, Kura, the Boundless Sky, Cartographer's Survey |

All of these are Full except Vorinclex. Your job: (1) model Vorinclex, (2) make sure each deck's AI actually uses its
new cards in the way the package intends, fixing what doesn't. Don't change any deck list; test with `--swap` in a
`--trace` run, or with testkit tables.

## 1. Vorinclex, Monstrous Raider (Full, Titania's module `impls/titania.rs` or the green module)
{4}{G}{G} 6/6 trample, haste. "If you would put one or more counters on a permanent or player, put twice that many of
each of those kinds of counters on that permanent or player instead. If an opponent would put one or more counters on
a permanent or player, they put half that many of each of those kinds of counters on that permanent or player
instead, rounded down." This is a replacement on counter placement by its controller vs opponents, all counter kinds
(+1/+1, -1/-1, loyalty, charge, poison on players...), including permanents entering with counters (planeswalker
loyalty, Genesis-style "enters with X counters"). Find the engine's counter-placing paths (grep `add_counter`,
`counters`, `loyalty`, proliferate) and route them through one hook if they aren't already. Two Vorinclex: each
applies. Tests: +1/+1 doubled (Vinelasher Adept 3 -> 6), opponent's halved (rounded down), planeswalker entering,
proliferate, a -1/-1 counter (persist) doubled.

## 2. AI checks (diagnose with traces/tests, fix gaps, one commit per fix, a test each)
- **Kefka, Yawgmoth's Will:** cast only when the graveyard holds enough to replay (a storm turn: Dark Ritual, Lotus
  Petal, Brain Freeze, Grapeshot, or Breach itself; or several cheap spells), not into an empty yard. Does Kefka's
  storm/Breach plan (`impls/kefka.rs`, `deck_has_storm`) count Will as a second engine? Do the tutor picks
  (`ai/decks.rs` tutor dispatch) know Will, and do **Vampiric Tutor** (top of library) and **Mystical Tutor**
  (instant/sorcery only, top) pick sensibly (Breach/Will/Brain Freeze on the kill turn, interaction otherwise)?
  Duress: target choice is generic; check it isn't held forever.
- **Veyran, Underworld Breach:** Veyran played Breach until 10/11 — check its escape logic still runs for Veyran.
  **Terror of the Peaks:** triggers on every token Veyran's makers create (Young Pyromancer, Talrand, Murmuring
  Mystic...), damage = power, targets chosen sensibly (face of the threatening opponent, or a key creature).
  **Mirari:** the AI pays {3} to copy when it's worth it (burn, draw, Grapeshot-likes, a copied tutor), and the copy
  counts as a copied spell for magecraft and Veyran's trigger doubling.
- **Sauron, Sheoldred, the Apocalypse:** does Jace's Archivist's activation value (opponents draw) account for
  Sheoldred being out? Cheap fix only if missing.
- **Titania, Tireless Tracker:** clues cracked when mana is spare; counters added on sacrifice (and doubled with
  Vorinclex). **Kura:** death mode chooses lands when Titania can use them (land count, lands in hand), the token
  otherwise. **Cartographer's Survey:** cast early. **Vorinclex:** cast priority as a top-end threat; Topiary
  Lecturer's increment counters doubled -> more mana.

## Done when
LASTDECK-RULES checks 1, 2, 4 (Vorinclex Full in a scratch audit: copy `data/ decklists/ tests/` and root `*.txt` to a
scratch dir, add `1 Vorinclex, Monstrous Raider` to the scratch Titania list, run `cardgen --root <scratch> audit
--mine`; never `audit --md` in the worktree) and 5 for `titania`, `kefka` and `veyran` (`--jobs 3`), plus one
`--trace` per package deck with the package swapped in that shows the new cards being used. Report in at most 15
lines (line 1 DONE or HANDOFF, line 2 your model ID); full report in `rust/agents/report-jd-pkg-cards.md`, committed;
handoff in `rust/agents/handoff-jd-pkg-cards.md`.
