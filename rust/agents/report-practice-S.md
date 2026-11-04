# Report: practice-mode combo shortcuts for the Ur-Dragon loops (agent S, 2026-11-04)

"start the loop: <combo>" is an ability on every battlefield piece of each of the three catalogue loops. It is listed
only while `combos::ready_to_go` holds (pieces out, card in hand, the whole line's mana in the person's pool, timing).
One prompt asks how far (cancel plays by hand). Then the catalogue's own `run_steps` runs the steps with the opponents'
windows (cast, Project Image, Miirym's copy, Twinflame, Regrowth copy, pieces-must-survive), with a finish made for the
person (no auto-win; the person attacks):

- **Astral Dragon + Cursed Mirror:** hasty Astral Dragon copies (artifact too; Mirror tokens that revert to Mirrors next
  turn), 2 per Project Image, x Doubling Season; N rounds (1, 2, 3, 5) or enough to kill the table. Token cap bounds it.
- **Miirym + Astral Dragon + Doubling Season:** the catalogue's Dragon flood with a number (token cap, ~100, ~50, ~10,
  ~5), summoning sick.
- **Garth + Twinflame + Spark Double + Doubling Season:** after the first Twinflame and Regrowth, N more rounds of
  Twinflame on every Garth: hasty token Garths (not legendary, exiled at end step).

Files: `human/urdragon.rs` (new section, header comment rewritten: Sauron deck's Godo/Celebrant/Helm, Ur-Dragon keeps
Twinflame), `human/abilities.rs` (one line in `loop_abilities`), `impls/t45_urdragon_combos.rs` (`dragon_flood_n`; the
AI's `dragon_flood` calls it with the cap: same behaviour), `tests/practice_urdragon_loops.rs` (16 tests).
Tests: offered when live, not offered with a piece/mana/timing missing, cancel changes nothing, rounds -> token counts,
Mirror revert, counterspell stops each loop. Whole suite `--no-fail-fast`: 3924 passed, 0 failed, 4 ignored (262 binaries,
including both probes and play_bot_games).

Not modelled (as in the catalogue): Lotus/Braingeyser rounds and the infinite mana/cards of the Garth loop; per-token
enter triggers of other cards (Terror of the Peaks ...) do run, and may prompt the person per token.
