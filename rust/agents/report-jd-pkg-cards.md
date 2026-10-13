# Report: jd-pkg-cards (claude-sonnet-5-5)

Status DONE. Branch jd-pkg-cards, not pushed. Commits without Co-Authored-By after the coordinator's rule change (the first commit has none either).

## 1. Vorinclex, Monstrous Raider (Full)
- Card data: scryfall_cache entry fetched; override (tags, Full, note); added `1x Vorinclex, Monstrous Raider` to binder-cards.txt so cards.json/--swap know it (JD: confirm you own it); cards.json/decks.json re-exported.
- Mechanic: `cardcode::vorinclex(g, actor, n)` (doubles for the controller's own, halves rounded down for opponents, per Vorinclex, signed so -1/-1 counters work) and `cardcode::put_plus(g, m, n)`.
  Routed: ~50 `g.perm_mut(x).plus += n` sites replaced by put_plus (sed, mechanical); `add_counters`; dsl `counters`, `proliferate`, `double_counters` (the added half); `common::proliferate` (own +1/+1, loyalty, opponents' -1/-1 and poison, actor = proliferator); `zones::amass`; enters-with-counters incl. persist (-1 -> -2) and undying; planeswalker entering loyalty; loyalty-ability plus costs (common::walker_go, dsl::use_loyalty); toxic poison (Dune Mover, Brutalizer, Paladin, Ribskiff) and mine.rs proliferate poison.
  Order with Hardened Scales: Scales first, then Vorinclex ((3+1)*2 = 8).
- NOT routed (listed in the override note): lands' charge/Incubator/Ozolith/lore counters, Crusade-style bulk pumps, `x.plus` writes through a held `let x = g.perm_mut` borrow (Spark Double, Mimic, Sephiroth, etc.).
- Audit: scratch root with Vorinclex in Titania's list: titania 0 not Full.

## 2. AI checks (found and fixed)
- Kefka / Yawgmoth's Will: was NEVER cast by Kefka (Yawg tag = special, prio 0). New `kefka::will_options`: offered only with a storm card in the yard plus mana, or 3 cheap spells that fit the mana left; held for an empty/thin yard. Under Will, `breach_hold_prio` frees the line cards so the yard's rituals/Brain Freeze/Grapeshot are replayed.
  Tutors: Mystical Tutor ("is") fetches the storm card for a waiting Breach (new arm in decks.rs); Vampiric's wish list gains Will after the rituals. Vampiric/Mystical are cast at the end of other turns by design (existing rule). Duress: not held (prio 0 only with no noncreature card in opponents' hands).
- Veyran: Breach escape still runs for Veyran (test). Terror of the Peaks did NOT trigger for tokens (make_tokens fires tokens_enter, not the creature etb): fixed in t2.rs (`terror_tokens`). Mirari already copies burn/draw/tutors and the copy fires magecraft (test).
- Sauron: Jace's Archivist wheel now counts Sheoldred (SheoA) like Bowmasters (`archivist_worth`, prio).
- Titania: Kura was NOT Full as claimed (compiled DSL fetched one nonbasic land and made a 1/1): hand-coded `kura_dies` (three lands, nonbasics first, while land drops remain; else X/X Spirit, X = lands). Cartographer's Survey prio 74 until 7 lands; Vorinclex prio >= 62. Tracker's sacrifice counter goes through put_plus (doubled with Vorinclex); clues are cracked with spare mana (existing 1.5 option). Topiary Lecturer's increment goes via cast-time counters (doubled).

## Checks
- `cargo build --release -j 3 --all-targets`: 0 warnings. `cargo test --release -j 3 --no-fail-fast`: all pass (practice http_lan failed once on a 120 s timeout in a first run under load, passed on the rerun).
- Smoke 48 games lookahead --jobs 3 for titania, kefka, veyran: finished. all_decks skipped per brief.
- Traces (3 games each, swaps in): Kefka cast Vampiric Tutor and Duress; Veyran cast Terror of the Peaks (dealt 5 to a planeswalker). Will, Mystical, Mirari, Breach and Sauron's Sheoldred were not drawn in those games: covered by tests/cards_jd_pkg.rs and cards_green.rs instead.

## Shared-file changes
cardcode.rs (vorinclex, put_plus, add_counters), dsl/mod.rs, engine/zones.rs, ai/decks.rs (breach_hold_prio, tutor arms, archivist), impls/common.rs, mine.rs, t2.rs, kefka.rs, green_cards.rs, titania.rs, titania-adjacent toxic sites; ~23 files touched by the put_plus sed.
Tests: crates/sim-core/tests/cards_green.rs (5 Vorinclex), cards_jd_pkg.rs (11).
