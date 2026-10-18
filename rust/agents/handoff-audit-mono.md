# Handoff: audit-mono (sheoldred + titania)

Done (all committed on branch audit-mono): sheoldred 2-7 (not 1), 6; titania 2-5, 8. See report-audit-mono.md.
Not done: sheoldred 1 and titania 1 (brain.rs DSL fallback, the core agent's), sheoldred 8/9 and titania 6/7 (by design / deckbuilding).
Gotchas: card_overrides.toml edits need `cardgen export --offline` (data/cards.json); never `cardgen audit`.
