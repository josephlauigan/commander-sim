# Task: practice-oppchoice, the person makes their own choices when an opponent's card affects them

Read `rust/agents/practice-fixes-common.md` first. Worktree:
`/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/practice-oppchoice`, branch `practice-oppchoice`.

## The problem
When the person is the affected player, some cards let the AI choose for them, bypassing the prompts that the
generic `zones::edict` and `discard_worst` already have. Audit list (REPORT.md "When you are the opponent"):
Fact or Fiction (no piles at all: the AI splits and picks), Accursed Marauder and Plaguecrafter (each player
sacrifices), Kefka, Ruler of Ruin's {8} ability (each opponent sacrifices a permanent of their choice), Invoke Despair,
Torment of Hailfire, Intellect Devourer, End of the Hunt, Braids's Frightful Return ch. III, Sauron's ward
(`engine/removal.rs:200`), Phyrexian Obliterator, Esper Sentinel (pays the person's {X} for them), Norn's Annex
(auto-pays or drops the person's attackers), Eldest Reborn ch. I (creature-only edict; it should be "creature or
planeswalker").

## Do
1. For each card: read its Oracle text (`data/scryfall_cache.json` lowercase keys, `oracle_text`, or
   `curl -s -m 30 "https://api.scryfall.com/cards/named?exact=<name>"`), find the code (the findings files give
   file:line), and when the affected player is a person, ask them with the existing helpers
   (`choices::sacrifice_creature`, a generic sacrifice-a-permanent prompt if there is none yet, `discard`, `pay_tax`,
   `yes_no`). Keep the AI's choice for AI seats exactly as it is.
2. **Fact or Fiction** (common.rs `fof` and/or rules.rs:243; find which is live) needs both sides:
   - the person casts it: an opponent (AI) splits the five into two piles (keep the AI's split logic), the person
     picks a pile;
   - the person is the opponent who splits: they divide the five (`pick_cards` for one pile), the AI picks.
   Show the five cards by name in the prompts.
3. **Torment of Hailfire**: X times, the person chooses lose 3 life, sacrifice a nonland permanent, or discard.
4. **Esper Sentinel** and **Norn's Annex**: ask the person whether to pay ({X}, or {W} or 2 life per attacker)
   instead of auto-paying or dropping attackers.

## Tests
`tests/play_opponent_choices.rs`: a representative test per mechanism (an edict, Fact or Fiction both ways, Torment,
a pay-or-not tax). Check the prompt was asked and the answer applied.

## Files owned
The functions of the listed cards in `impls/*.rs`, `engine/removal.rs` (Sauron's ward only), a new sacrifice-a-
permanent helper in `human/choices.rs` if needed, the new test, `rust/agents/{handoff,report}-practice-oppchoice.md`.

Commit message for the last commit: "Practice mode: the person makes their own choices when an opponent's card affects them".
