# Report t6-e-combo2 (Dack, Sisay, Rograkh & Thrasios combos)
Module impls/t6_combos_t6.rs (`pub static COMBOS: [Combo; 6]`), one `pub mod` line in impls/mod.rs, one chain line in `combos::all()`.
Combos and steps (answer windows):
- Dack: Preston + nontoken Felidar + Radiant Solar (or a Solar token), all the player's. (1) with Cloudshift/Ephemerate in hand: pieces
  (Permanent) -> flicker spell (Spell) -> first Felidar flicker trigger (Trigger) -> win. (2) "loop running": the engine's Preston cap was hit
  this turn (`PRESTON_CAP`), no spell needed: pieces -> Felidar trigger -> win.
- Sisay DDR: Ioreth + Marvin + a legend that taps for mana without help (Selvala, a dork-tagged legend; or any legend with Enduring Vitality /
  Cryptolith Rite / Relic of Legends) + Mount Doom (out, or in hand with the land drop open). Missing pieces are cast (Spell x5, plan recomputed
  each time); summoning-sick pieces need Shang-Chi/Tyvar (out or cast). Steps: casts -> pieces (Permanent) -> Ioreth untap (Ability) -> Mount Doom
  in (Engine) -> Doom ping (Engine, ability_window_card) -> win. Shang-Chi alone is not a source (his mana only pays creature abilities).
- Sisay Flash Unicorn: Derevi + Emiel (cast if in hand) + Gaea's Cradle with 4+ creatures, or an untapped Bloom Tender with 4+ colours among
  permanents; {3} for the first blink; + Mount Doom. Steps: casts -> pieces (Permanent) -> Emiel blink (Ability, {3}) -> Derevi enter trigger
  (Trigger) -> Doom in/ping -> win.
- Rograkh Plan A: Oboro (cast if in hand) + Cradle + 6+ creatures (commanders count) + Talon Gates (bf or hand) + {6} + Thrasios on bf + a
  finisher in reach (Finale in hand/library, else Faerie Mastermind + Green Sun's Zenith). Steps: cast Oboro (Spell) -> pieces (Permanent: Oboro,
  and every creature when only 6) -> Oboro activation (Ability, {2}) -> Gates back (Engine; its enter trigger is an ability window; all opposing
  creatures phase out) -> Thrasios draws (Ability) -> Finale cast (Spell, counter window) or Mastermind cast + draw ability -> win.
- Rograkh Witness + Snap + Cradle (5+ creatures, Witness and Snap in hand, same finisher chain): cast Witness (Spell) -> Snap (Spell) -> Witness
  trigger returns Snap (Trigger) -> pieces -> Thrasios/finisher steps -> win.
Caps: `PRESTON_CAP` (30) and Ioreth's `UNTAP_CAP` (6) KEPT. Real play (a Dack flip, Ioreth + Marvin out on the AI's own) reaches those loops
without these combos, so the caps are not unreachable. The Dack "running" combo reads the cap, so a real loop that hits it is finished by the combo.
Tests: tests/engine_t6_combo2.rs (26): wins per line, not-ready cases (missing piece, sickness, creature counts), removal / Counterspell / Ertai at
the right step. Counter tests are multi-seed (the AI counters by roll). Full `cargo test --release`: all pass (129 result lines ok, 0 failed); `cargo build --release --all-targets`: no warnings.
Not built (gaps): Dack Line A end-to-end (the 35-pile Dack flip test is row 19; donated pieces are the opponents', so the shortcut starts only
once Preston, Felidar and Solar are the player's) and the two-banisher trade (a way to regain pieces, no standalone assembly); Sisay Selvala chain,
Sisay-fetch lines (Sisay as the tutor), Deadpool + Forbidden Orchard, Mana Vault starter, Mount Doom as a Sisay fetch; Rograkh Badgermole lines,
Springheart/Vibrance, Veil of Summer and the GSZ X=0 detail (GSZ is only required, not cast). Loops are not played out (abstract win).
No behaviour change for other decks (new combos only gate on their own pieces; combo_options is for outside decks only).
