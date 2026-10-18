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
- Pyrokinesis: done (Partial; tags rem=dmg4 tgt=c added).
- Scout's Warning: done (Partial).
- Kozilek's Command: done (Full; tags tokx rem=exile tgt=c draw=1).
- Eldrazi Confluence: done (Full; tags rem=shrink3 tgt=c). Note: Titan blink_value=2 so the AI does not blink it; tests use Ravenous Chupacabra. Toughness<=0 is not an SBA in check_state: kill with die(g,m,"sba").
