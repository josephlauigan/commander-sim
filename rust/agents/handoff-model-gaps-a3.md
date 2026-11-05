# Handoff: model-gaps-a3
Done: items 1-4 (see report-model-gaps-a3.md). Tests tests/model_gaps_a3.rs (13).
Next: item 5 (Camera: split cost/effect of sauron's hand-written abilities, Ctx marker, person ability), item 6 (Marvin), and the
ward long tail (NEEDS JD in the report). Then re-run `cardgen export --offline` and `cardgen audit --mine`.
Gotchas: token creature lands are in lands AND perms; remove_land(g,p,l,Some(Zone)) is the only way a land leaves.
