# Task: gix-priest, model Priest of Gix and teach Seph/Atraxa's AI the Deathmantle loop

Read `rust/agents/RULES.md` (skip `git merge rust-port`; the worktree is on main). Worktree:
`/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/gix-priest`, branch `gix-priest`.
JD wants to test **Priest of Gix in Seph/Atraxa** (key `seph`, T3-T4). It isn't in `data/cards.json` yet.

## The card
**Priest of Gix** {2}{B}, Creature — Phyrexian Human Cleric Minion, 2/1. When this creature enters, add {B}{B}{B}.

## Why it's in Seph (the AI must see this)
Seph/Atraxa already has Nim Deathmantle, Ashnod's Altar, Blood Artist, Zulaport Cutthroat, Viscera Seer, Mikaeus the
Unhallowed, Muldrotha, and a flicker package (Ephemerate, Momentary Blink, Restoration Angel, Conjurer's Closet,
Teleportation Circle, Flickering Hound, Displacer Kitten). The deck's AI knows four infinite loops already
(`ai/seph.rs`, `impls/sephfs.rs::altar_loop` style, and `human/abilities.rs::loop_abilities` for the person's
shortcut); find how a loop is recognised and executed (abstracted) and add this one the same way:
**Priest of Gix + Nim Deathmantle + Ashnod's Altar** = sacrifice Priest to the Altar ({C}{C}), Deathmantle returns it
for {4}, it enters and adds {B}{B}{B}: +1 mana and a death per loop; with Blood Artist or Zulaport Cutthroat out that
is an infinite drain (a win), otherwise infinite mana and deaths.

## Do
1. Model Priest of Gix (the ETB mana) the repo's way (`cardgen` overrides + export; mana from an ETB exists for other
   cards, reuse it) with an override `status` and `note`.
2. Add the loop to Seph's loop recognition and execution, the person's shortcut list too (`loop_abilities`), and let
   Seph's tutors value Priest as a loop piece when Deathmantle and Altar are around (look at how the existing loops'
   pieces are tutored).
3. Blink value: the flicker package should see Priest as a fine blink target (3 mana back); check `flicker_worth` /
   the Seph blink valuation.
4. Tests: the ETB mana, the loop found and run (drain with Blood Artist), the loop not claimed when a piece is missing.

## Checks
`cargo test --release -j 3 --test-threads=3` passes (whole workspace), `cargo build --release -j 3` has no warnings,
`./target/release/commander-sim --deck seph --pool t3 --swap "<a cut>=>Priest of Gix" --games 50 --ai adaptive
--profile loose --jobs 3` runs and the preflight shows Priest cast (use any card in Seph's current list as the cut for
this check; I choose the real cut). **No other deck's games may change:** run `--deck cloud --pool t1 --games 200 --ai
adaptive --profile loose --jobs 3 --no-preflight` and `--deck seph --pool t3 ...` (without the swap) before and after
your change and diff; Seph without Priest must also be identical unless you changed Seph's existing loop code on
purpose (then say why). Make sure both outputs contain a "Win rate" line before diffing.

Limits: `-j 3`, `--test-threads=3`, `nice -n 19`, memory cap per RULES.md, no Python, no other sims. Run long commands
in the foreground with a timeout. Handoff `rust/agents/handoff-gix-priest.md` early; commit after each piece
(standalone, no Co-Authored-By); don't push. Report in 15 lines or fewer; details in `rust/agents/report-gix-priest.md`.
