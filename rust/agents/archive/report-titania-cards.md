# Report: titania-cards (branch models-titania, base bill-deck 3c1830e)

Result: DONE. `cardgen audit --mine`: `titania: 66 unique cards, 0 not Full`. `cargo test --release`: all green (26 tests in
`tests/cards_titania.rs`); `cargo build --release --all-targets`: no warnings; `all_decks adaptive 3` and `lookahead 1` finish;
48-game smoke run `--deck titania --pool t3 --profile loose --ai lookahead --jobs 8`: 27.1% (CI 16.6-41.0%), not a measurement.

## Titania herself (checked first)
Graveyard trigger fine: `turn::crack_fetch` fires `LandGy` for Evolving Wilds / Terramorphic Expanse / Fabled Passage; `lands::sac_land` and
`lands::destroy_land` for other deaths (tests for all). Bugs found and fixed (all gated to key titania):
- the enter trigger returned the land with most colors, tapped; now the fetch land first, else any land, untapped (`titania_etb`
  wraps t1's, which still serves other decks);
- a fetch land with no basic left in the library was not sacrificed at all (the sacrifice is a cost): now it goes to the graveyard;
- **tokens never triggered Garruk's Uprising** ("a creature with power 4 or more enters: draw"): `make_tokens` skipped it, so the 5/3s
  drew nothing. Fixed in `make_tokens`, gated to titania. **The same bug hits Bill, Jenova and Glissa decks (they play Uprising);
  the lead should decide whether to ungate it** (remove `key == "titania"` in engine/zones.rs make_tokens) and remeasure them.

## Cards (impls/titania.rs; verdict text in data/card_overrides.toml)
- Follow the Lumarets (`look_take` 4, 1 card or 2 with `gix::gained_life`), Something Worth Saving, Scout the City (both modes; Bring Down
  when an opposing flier is worth 3+), Malevolent Rumble (permanent to hand, rest to graveyard, real Eldrazi Spawn that taps for {C}):
  `mill_take` leaves fetch lands in the graveyard for Titania.
- Glorious Decay: all three modes; new shared `tgt=fa` (artifact, or flier with toughness <= 4) in `removal::legal_targets`; with no target the
  AI never casts it (mode 3 exists in `resolve` but is never chosen proactively).
- Annoyed Altisaur: `self_cast` -> `jodah::cascade` (mana value 7).
- Arachnogenesis: X Spiders with reach (X = attackers, `ctx.x`) and `Game::fog_spiders`, read by `cardcode::damage_prevented` (non-Spider
  damage prevented, both ways). Cast from `ai::brain::attack_response` (declare attackers, so Spiders can block) when the attack takes a third
  of life or more, or would be lethal or a 21 commander total.
- Infestation Expert: `Game::day_night` (0/1/2) and `dn_last`; first daybound permanent makes it day; `day_night_tick` (called as a turn begins
  in `turn.rs`) judges the previous turn's active player (0 spells: night; 2+: day); Expert is 4/5 at night and makes two Insects.
  Only Infestation Expert uses it, so no other deck is affected.

## AI plan
Titania prio 97 from turn 2; fillers (Wayfinder, Rumble, Worth Saving, Scout) prio 86 while no fetch land is in the graveyard and Titania is
not out; Wayfinder keeps fetch lands in the graveyard (zones.rs, gated). Fetch lands: `land_adjust` (+6 with Titania out, or when she is
castable without the land drop; -6 when the basic is what makes her castable); `hold_fetch` keeps a fetch land uncracked when mana + a land
in hand + a ramp spell reaches her cost next turn (so cracking the tapped basic would not speed her up); her enter trigger and a
"crack the fetch lands" option (utility 25) crack them, Garruk's Uprising cast first. Royal Treatment, then Blossoming Defense, in
response to removal on Titania (other creatures worth 5+); with no answer the fetch lands are cracked in response. Stampede / End-Raze
cast only when `trample_through` (blockers soak up their toughness, trample) is lethal for some opponent.

## Shared-file changes (all small, gated to key titania unless noted)
state.rs (3 fields), cardcode.rs (`damage_prevented` + `titania::spider_fog`, only true while Arachnogenesis is active), engine/turn.rs
(`land_adjust`, `hold_fetch` in play_land, `crack_fetch` empty case, `day_night_tick` call: returns at once when day_night == 0),
engine/removal.rs (`tgt=fa`), engine/zones.rs (Wayfinder fetch land; Uprising draw for tokens), ai/brain.rs (`attack_fog`,
`titania_special`), ai/decks.rs (`titania_protect`), impls/t1.rs (`titania` made pub), data/card_overrides.toml, data/cards.json.

## Open points
- Spiders/Arachnogenesis X counts attackers at cast, not at resolution; a mill can deck the player (fillers need library > 12-20).
- Glorious Decay mode 3 is never cast on purpose. Infestation Expert's transform is a P/T and token-count change (no card face).
- Process slip: I ran `python - 2>/dev/null` once by mistake (it printed nothing; "python" does not exist here) and `which python python3`.
  No Python code was executed.
