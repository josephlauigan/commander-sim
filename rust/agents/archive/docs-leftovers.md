# Task: docs-leftovers — bring five deck docs' prose in line with the 2026-10-14 leftover swaps

Your worktree's branch `docs-leftovers` was made from `leftovers-apply` (commit f90767c), where the lists already have
the swaps (see the newest **Updated 2026-10-14** line at the top of each doc). Read
`audit/leftovers/2026-10-14/README.md` first: swaps and measurements. Follow the rules of
`rust/agents/docs-jd-packages.md` (no Python, edit only your docs, leave generated sections and Updated lines alone,
no invented numbers or card text, no "also in X deck" notes, commit without a Co-Authored-By line, never push).

| Doc | Out | In | Result |
|---|---|---|---|
| decklists/JD/sephiroth-phyrexian-reanimator.md (call it Seph/Atraxa) | Orzhov Signet | Worldly Tutor (Game Changer: now 8) | -0.1 |
| decklists/JD/yshtola-esper-drain.md | Triplicate Spirits, a Swamp, an Island | Dakkon, Shadow Slayer, Underground River, Adarkar Wastes | -0.7 |
| decklists/JD/kefka-grixis-discard.md | Decorum Dissertation, two Swamps | Bitter Triumph, Sulfurous Springs, Shivan Reef | -1.9 |
| decklists/Avery/alela-esper-faeries.md | Gravedigger, two Plains | Tymna the Weaver, Exotic Orchard, Shattered Sanctum | +0.1 |
| decklists/Avery/galadriel-bant-rebels.md | Flicker | Dragonlord Dromoka | +1.1 |

All new cards come from JD's binder (the retired Jodah deck's leftovers); JD's cuts go to bulk; Avery's cuts
(Gravedigger, Flicker) go back to Avery. The two Avery docs are written for Avery (the deck's owner): say the new cards
are JD's, nothing else about JD's decks. All five results are within noise; say so plainly (JD's choice to apply).

Update in each doc: the Simulator line (keep its form: this paired test, pool run pending, older figures marked as the
previous list's), strategy paragraphs that list the changed cards (mana base, removal, tutors, card draw), counts you
can recount from the Import list (lands, rocks, removal, tutors, Game Changers), Bracket/Rule 0 text, card-source
tables. Y'shtola: Dakkon is a noncreature spell with mana value 3, so it triggers Y'shtola (check her text in the
cache). Kefka: Bitter Triumph's discard cost feeds the graveyard for Breach and Yawgmoth's Will. Alela: Tymna draws
for each opponent the flying Faeries hit. Galadriel: Dromoka can't be countered and stops opponents casting spells on
your turn; it triggers Galadriel's Alliance like any creature.

Done: commit (`git add decklists/ && git commit -m "..."`), final report at most 15 lines: line 1 DONE, line 2 your
model ID, commit hash, what changed per doc, any count you could not check.
