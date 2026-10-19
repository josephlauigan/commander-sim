# Audit: where the AI decides for the person in practice mode

Read-only audit of the Rust card code and engine. **Do not edit any source file.** Write only your findings file
`audit/practice-human/findings-<group>.md` in the worktree
`/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/practice-human-audit` (branch `practice-human-audit`).
Don't commit: I do that.

## Why
JD plays his own decks in practice mode (`rust/crates/practice`) and keeps finding cards where the AI quietly chooses
for him, or where his card does nothing. Recent examples, all now fixed:
- **Sephiroth, Fabled SOLDIER**: `seph_sacrifice` began with `if human(g, p) { return }`, so his "you may sacrifice
  another creature" never happened for a person (`impls/sheoldred.rs`).
- **Ardenn, Intrepid Archaeologist**: `ardenn_combat` moved the Equipment to the AI's best holder without asking
  (`impls/bulk_white.rs`), and beginning-of-combat triggers fired after attackers (`engine/combat.rs`).
- **Solitude**: a missing half of the card (no life gain), seen in practice.

## Facts you need
- A seat is a person when `crate::human::is_human(g, p)` / `human_of(g, p)` / the local `human(g, p)` is true.
  Choices go through `crate::human::choices` (`choose`, `choose_one`, `pick_cards`, `yes_no`, `sacrifice_creature`,
  `discard`, `search`, `scry`, `target_opponent`, `pick_modes`, `etb_removal`, ...). Look at those helpers first.
- `engine::stack::trigger_window` **never asks the person** whether to use a "may" trigger: it only gives priority.
  So an optional trigger is decided by whatever the card code does after it.
- AI-only choice helpers look like `best_*`, `max_by`, `min_by`, `pval`, `card_worth`, `sac_worth`, `weakest_opp`,
  `best_opp_creature`, `*_fodder`, `pick_*`, plan/brain/utility code.
- Hooks: `etb`, `dies`, `attack`, `combat_start`, `upkeep`, `end_step`, `cast`, `options`, `hand_options`,
  `activated`, ... registered in each file's `register*` fn (`r.card(db, NAME)?.etb = Some(f)`).
- `options` / `hand_options` hooks are the **AI's** action generators. Many start `if human(g, p) || ... { return }`.
  That is fine **only if** the person can still do the same thing through their own UI (`human/human.rs`
  `abilities_of`, casting from hand, equip...). If an ability exists only as an AI option, the person can never use it:
  that is a finding.
- The card list `audit/practice-human/jd-cards.tsv` (card, then the decks it is in) holds every card in JD's decks.

## What counts as a finding
For a card **in JD's decks** (check the tsv), any place where, when the card's controller is a person:
1. **Skipped:** the effect silently does nothing (`if human ... return`) where the card would do something.
2. **Chosen for them:** a choice the card gives its controller (may or may not; which target, mode, creature, card,
   opponent, player, number X, order; "any number of") is made by AI logic with no `choices::` call.
3. **Unreachable:** an activated ability or alternative cost (evoke, flashback, ninjutsu, channel, cycling,
   equip variants, planeswalker loyalty...) that only the AI's `options` code can use.

And for **any** card (opponents' too), places where the **person is the affected player** but the AI chooses for
them: "each opponent sacrifices a creature" (the person should pick which), "target player discards" (they pick),
"opponent chooses" (Fact or Fiction piles, Tergrid, punisher cards), blocks or damage assignment forced, etc.

Not a finding: the AI deciding for AI seats; hidden-information randomness the rules make random; a choice the
rules give someone else. If in doubt, list it as "check" with the reason.

## How
Work through your files. For each `register*` fn, list the hooks per card; for each card in the tsv (or any card
that makes an opponent choose), open its hook functions and look for the patterns above. Use grep to find them fast
(`grep -n "human(g\|is_human\|human_of\|choices::"`), and read only the functions you need.

## Output (write the file after your first few findings, then append; keep it current)
`audit/practice-human/findings-<group>.md`:

```
# Findings: <group> (files: ...)
| Card | Decks | File:line | Kind (skipped / chosen / unreachable / opponent-choice / check) | What happens for the person | What the card says | Severity (high: card does nothing or wrong; med: choice taken away; low: minor) |
```
Then a short list of the cards you checked and found fine (just names), and anything you could not check.

Limits: read-only, no builds, no sims, no Python. Keep command output short (grep, `sed -n`). Finish in under ~60
tool calls; if you run out, write what you have, mark the files not done, and stop. Report to me in 10 lines or fewer:
counts by kind and severity, the top five findings, and the files not finished.
