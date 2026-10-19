# Report: practice-dsl-may

27 cards flagged (JD-relevant check via jd-cards.tsv not done; list below, effect kinds):
Auriok Champion | gain_life
Aven Fisher | draw
Burning-Tree Vandal | discard
Cartographer | regrow
Dune Mover | search
Emeria Angel | token
Environmental Scientist | search
Farhaven Elf | search
Gravedigger | regrow
Growth Spiral | put_land
Heliod's Pilgrim | search
Kitsune Dawnblade | tap
Kor Cartographer | search
Legion Conquistador | search
Mesa Enchantress | draw
Omen of the Hunt | search
Pilgrim's Eye | search
Pothole Mole | regrow
Reclamation Sage | destroy
Reya Dawnbringer | reanimate
Rogue's Gloves | draw
Silver Deputy | search
Solemn Simulacrum | search,draw
Sword of Light and Shadow | regrow
Tether Technician | discard
Verduran Enchantress | draw
World Shaper | mill

Unsure: Burning-Tree Vandal/Tether Technician (discard is the 'you may' of 'if you do' follow-ups; follow-up effect still runs on no), Solemn (dies draw flagged: correct per Oracle).
Sims (sheoldred t3, cloud t1, kefka t3, 200 games): before/after identical. Before binary already contained the run-time edit but old cards.json.
Shared change: none beyond dsl/mod.rs; rustfmt also reflowed a few lines of dsl/mod.rs.
