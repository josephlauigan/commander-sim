# Handoff: model-gaps-a2
Done: all eight cards handled (see report-model-gaps-a2.md), tests pass, data exported, everything committed.
Next: coordinator runs the full suite and sims (Marvin's AI options changed), then re-exports after merging with model-gaps-a1.
Gotchas: statuses come from data/audit.toml [card_notes] first, then card_overrides.toml; a scryfall-source card's bad tag is dropped with `tags = "..."`, not `pool_remove_tags`.
