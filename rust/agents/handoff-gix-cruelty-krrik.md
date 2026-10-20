# Handoff gix-cruelty-krrik
Done: cards fetched into scryfall_cache + exported (overrides added); impls/krrik.rs (K'rrik: life_pips, drop_black, cast counter);
mana.rs cost_of/phyrexian_life(g,p,c,..) + brain.rs + human legal/human (phyrexian(g,p,c)); Cruelty in impls/sephfs_cards.rs (gix_*),
prio in sephfs.rs (56), K'rrik prio in sheoldred.rs pkg2_prio.
Next: build (-j 3), tests tests/cards_krrik.rs + tests/cards_gix_saga.rs, cloud t1 before/after diff (need "Win rate" line in both;
baseline from a build of origin/main = run BEFORE changes: use git stash or the main checkout binary), swap sims, FINDINGS, report.
