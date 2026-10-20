# practice-cedh report
- catalog.rs: tier decks get commander2 + commanders; name = "Rograkh and Silas Renn" / "Rograkh and Thrasios" / "Kraum and Tymna the Weaver" (pool_deck_name); commanders() includes partners, deduped.
- images.rs: Images::fetch_in_background / fetch_online_in_background (one at a time via shared AtomicBool, quiet offline). server/mod.rs fetch_commanders (start-up) uses it too.
- routes.rs Site::options: missing commander pictures start one background download (skipped when PRACTICE_NO_FETCH is set).
- app.js/styles.css: opponent cards and tier name chips show both commanders (deckImages); watchImages re-fetches options every 5 s up to 12 times until all commander pictures are there.
- Makefile ios-clean + help line; ios-shell/README.md (clean start, 13 decks).
- Table/review names come from decks.json "name" (Rograkh/Silas etc.), already distinct; not changed.
- Tests: unit (naming, commanders, background fetch once-at-a-time + offline), http_catalog t6 names. Full workspace cargo test --release: all pass; build no warnings.
- curl check (network on): first /api/options 7 images, 40 s later 49; t6 names distinct with commander2.
