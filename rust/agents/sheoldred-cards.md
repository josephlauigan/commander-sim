# Task: sheoldred-cards — model the Sheoldred deck's last 8 cards and write its AI plan

Read `rust/agents/OVERNIGHT-RULES.md` first: its hard rules (never run Python, memory cap, edit only your files,
commit often, handoff, 15-line report) apply here. Where it says "Kefka/Zada", read "Sheoldred".

**Sharing the machine:** another session runs sims in parallel. Build and test with `-j 8`
(`cargo test --release -j 8`), and use `--jobs 6` for smoke runs.

Base branch: `sheoldred-deck`. Deck: `decklists/JD/sheoldred-mono-black.md` (key `sheoldred`): mono-black, built from
bulk. Sheoldred, the Apocalypse ({2}{B}{B} 4/5 deathtouch): whenever you draw a card, you gain 2 life; whenever an
opponent draws a card, they lose 2 life. Your module: `impls/sheoldred.rs` (stub in place: `sheoldred_prio`,
`sheoldred_special`, `sheoldred_protect` are already wired into the dispatch), tests `tests/cards_sheoldred.rs`.
Smoke run: `--deck sheoldred`.

## Part 1: cards to model (until `cardgen audit --mine` shows `sheoldred: ... 0 not Full`)
Unmodeled: Locust Spray (-1/-1; cycling {B}: the AI cycles it when no target is worth it), Ribskiff (Vehicle, toxic 2,
draw on enter, crew 3: find how Vehicles and crew are modeled first), Tome of Legends ("never cast": page counters
when your commander enters or attacks; {1},{T}, remove a counter: draw).
Partial: Diary of Dreams (the {5} draw, {1} less per page counter), Leech Collector // Bloodletting (the prepare
mechanic: first find whether "prepared" is modeled anywhere; Bloodletting: each opponent loses 2), Lethal Throwdown
(the modified-creature clause), Moseo, Vein's New Dean (the infusion end-step return), Restless Bloodseeker // Bloodsoaked
Reveler (Blood tokens on lifegain end steps; the transform; the back face's ability).

**Check Sheoldred herself first** (the audit says Full): `engine/zones.rs` drains opponents on their draws; make sure
(with tests) that she drains on opponents' draw steps, on extra draws (Vault Plunderer, Cost of Brilliance targeting
an opponent, a wheel), and that she gains you 2 for every card you draw, draw step and extra draws alike. If any of
that is wrong, fix it and say so.

## Part 2: cards from JD's Jodah deck that may move to this deck
These are already modeled (mostly in `impls/jodah.rs`); make them play right for `sheoldred` too, gated to that key:
- **Memory Jar.** Jodah's AI cracks it only with a hand of 2 or fewer. With Sheoldred out, crack it whenever you can
  (each opponent draws 7: 14 life each); without her, keep Jodah's rule.
- **Toxic Deluge.** Pick X so Sheoldred (toughness 5) survives when the sweep is still worth it, and check that X life
  is paid.
- **Bitter Triumph** and **Korlash, Heir to Blackblade** (P/T = your Swamps; {1}{B}: regenerate): check that they work
  in a sheoldred game (a test each).
Add one test per card that the effect works when the sheoldred deck plays it.

## AI plan for sheoldred (in `impls/sheoldred.rs`, through `sheoldred_prio` and the deck-options dispatch)
- Cast Sheoldred as early as the mana allows; rocks first.
- With Sheoldred out: equip Champion's Helm (hexproof: she is legendary) or Darksteel Plate on her before anything
  else that turn.
- Masterful Flourish, Battle-Rage Blessing and Dark Endurance in response to destroy-removal aimed at her (Helm's
  hexproof already stops targeted removal; indestructible doesn't stop exile or -X/-X: check the removal kind).
- Vault Plunderer and Cost of Brilliance target an opponent while Sheoldred is out (the one with the lowest life
  that the drain could kill first), and you otherwise.
- Removal goes to the biggest threat; hold one protection instant while she's out when you can.

## Done when
The OVERNIGHT-RULES checks all pass (full `cargo test --release`, no warnings, `all_decks adaptive 3` and
`lookahead 1`, audit `sheoldred: 0 not Full`, a 48-game smoke run with `--deck sheoldred`). Report as those rules say:
line 1 DONE or HANDOFF, line 2 your model ID. Full report in `rust/agents/report-sheoldred-cards.md`.
