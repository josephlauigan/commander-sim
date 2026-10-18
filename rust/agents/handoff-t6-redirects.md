# Handoff t6 redirects
Cards (14): Redirect Lightning; Ricochet Trap; Untimely Malfunction; Pyrokinesis; Scout's Warning; Kozilek's Command; Eldrazi Confluence; Get Lost; Erode; Soul Partition; Calamity's Wake; Emerald Charm; Sunscour; Gleaming Splendor
## Done
(none)
## Next
Oracle text fetch, build cardgen, first card.
## Gotchas
- Redirect Lightning: done (Partial). Shared fn redirect_removal covers Ricochet Trap + Untimely Malfunction's redirect too; their tests/notes come with their commits.
- Workflow: edit data/card_overrides.toml, `rust/target/release/cardgen export --offline` (cards.json may not change), audit with `cardgen audit --cards $SCRATCH/names.txt`.
- Untimely Malfunction: done (Partial).
