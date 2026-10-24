# Handoff: karn-core
Done (committed): key `karn` wired (practice catalog/setup/tests, sim-cli decks/args/poolmode/report, ai styles/prio/jd_deck,
engine key lists); artifact-only mana (`Floating::art`, `Source::FloatArt`, `ManaPool::art`, `mana::art_mana_ok`);
`impls/karn.rs` (Karn, Mightstone ETB, Tron + Urza's Workshop dyn mana, Factory/Foundry manlands + block response, karn_prio,
Bauble hold, Tron land search pick); overrides (Mightstone pstone, Gilded Lotus, Urza's Factory artifact token, statuses);
tests/cards_karn.rs (11 tests, pass).
Next: full `cargo test --release` (was running), data/printings/karn.txt (script in scratchpad; rate-limited Scryfall:
retry missing cards), 100-game run `commander-sim --deck karn --pool t3 --games 100 --jobs 7`, check Karn is cast and his mana
is spent (--cards), practice check, write report-karn-core.md.
Gotchas: engine floating mana empties only at turn start; spells must set `g.pay_for` (brain do_cast, cast_removal, cast_wipe do)
or Karn's mana leaks to nonartifact spells; DSL token types never include "artifact" (cardgen drops it) - Factory token via override.
