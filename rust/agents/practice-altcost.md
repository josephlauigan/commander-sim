# Task: practice-altcost, "play it another way": alternative costs and graveyard / land abilities for the person

Read `rust/agents/practice-fixes-common.md` first, with two changes: **build with `-j 3` and `--test-threads=3`**
(another agent runs beside you), and the audit is no longer in a worktree: read it with
`git show origin/practice-human-audit:audit/practice-human/REPORT.md` (and `findings-g1.md` ... `findings-g5.md`,
`jd-cards.tsv` the same way). Worktree: `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/practice-altcost`,
branch `practice-altcost`. JD's decks are in `data/decks.json` (`.mine[]`; `titania` is now `omnath`).

## The problem (REPORT.md cause 2)
`hand_options`, `gy_options` and `land_options` hooks are read only by `ai/brain.rs`. The person's UI offers only
`human/cards.rs::hand_ways` (alternative ways to cast from hand), `legal::gy_mode` (graveyard casts) and
`cards::land_abilities`. So every alternative cost or graveyard/land ability that lives only in an `*_options` hook
is invisible to the person. Examples in JD's decks (findings give file:line): Ragavan's dash, Woe Strider's escape,
Profane Tutor's suspend, Twinned Vision's flashback, Demonic Embrace from the graveyard, Reconstructed Thopter's
unearth, Muddle the Mixture's transmute, Jaxis's blitz, Susurian Voidborn / Perigee Beckoner's warp, Scarlet Spider's
mayhem, Exalted Angel / Forum Familiar / Greenbelt Radical morph-disguise, web-slinging / plot / foretell /
landcycling cards, Everflowing Chalice's multikicker (cast by a person it taps for 0), Reassembling Skeleton's
return, and the land abilities of Command Tower-style utility lands: Command Beacon, Karn's Bastion, Ghost Quarter,
Geier Reach Sanitarium, Mikokoro, Phyrexian Tower, Otherworld Atlas.

## Do
1. **Read first** how `hand_ways`, `gy_mode` and `land_abilities` reach the person (human/human.rs, human/cards.rs,
   the practice page's action list) and how an `*_options` hook builds its `Opt` (label, `Action::Plan` with an AI
   plan function). Write the plan in your handoff before coding.
2. **Prefer real person paths** for each card in JD's decks: add the alternative cost to `hand_ways` / `gy_mode` /
   `land_abilities` (the person picks targets, X, what to sacrifice or discard through `choices::`), reusing the
   card's own resolve code. Where that is a lot of work for one card, a fallback is acceptable: offer the AI's own
   `Opt` from the hook to the person as an action labelled with the hook's label (it runs the AI's plan, choices
   included), and list those cards in the report as "AI-chosen inside".
3. **Everflowing Chalice:** the person's cast asks how many times to kick (multikicker {2} each), and records
   `DataKey::Kicks` as the AI's path does.
4. Never change the AI path: the hooks stay; the AI still reads them as today.

## Tests
`tests/play_altcost.rs`: a person dashes Ragavan, escapes Woe Strider (or flashbacks Twinned Vision), uses a land
ability (Phyrexian Tower or Ghost Quarter), casts Everflowing Chalice with 2 kicks and it taps for 2. Plus the common
rules' before/after identical sims.

## Files owned
`human/cards.rs`, `human/legal.rs` (the person-path additions), the card functions you reuse (smallest change),
the new test, `rust/agents/{handoff,report}-practice-altcost.md`. Commit message for the last commit:
"Practice mode: alternative costs and graveyard / land abilities for the person (N cards)".
