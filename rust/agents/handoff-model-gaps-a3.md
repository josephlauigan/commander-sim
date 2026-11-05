# Handoff: model-gaps-a3
Done: item 1 (remove_land(g,p,l,Some(Zone)) is the one place a land leaves; token lands go nowhere; tests in tests/model_gaps_a3.rs).
Next: item 2 Astral Dragon (land copy = token in lands AND perms, "dual"; Aura copy), 3 ward, 4 Forge counter, 5 Camera, 6 Marvin.
Gotchas: remove_land keeps `perms` in sync for token lands (dual-listed Astral lands); zones::leave must drop it from lands too.
