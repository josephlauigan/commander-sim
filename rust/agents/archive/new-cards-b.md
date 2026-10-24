# Task: new-cards-b — model Cecil, Dark Knight; Lightning, Army of One; Marvin, Murderous Mimic (Kefka part)

JD bought new cards and wants to know they aren't big downgrades in his decks. Your job is only to model the cards
(and give each deck's AI a sane way to play them); the main session runs the swap tests afterwards.

Worktree: `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/new-cards-b` (branch `new-cards-b`, off
`new-cards-1018`). Work only there. Read `rust/agents/RULES.md`, but these rules override the old parts of it:
- **No Python, ever** (not even a version check); a hook refuses it. Ignore every Python / difftest / `rust-port`
  merge instruction in RULES.md. Don't merge any branch.
- Heavy commands: `systemd-run --user --scope -q -p MemoryMax=8G -p MemorySwapMax=0 nice -n 19 <command>`, cargo with
  `-j 4`. The machine is shared.
- No sims beyond the preflight check below (no `--games` runs, no `all_decks`).
- Commit messages: no Co-Authored-By line.

A second agent (new-cards-a) is modeling other cards in parallel and also edits `data/carddb.txt`,
`data/card_overrides.toml` and `data/cards.json`: add your lines in one block, keep changes to shared files small.
The main session merges both branches and re-exports.

## Cards (Oracle text)
1. **Cecil, Dark Knight // Cecil, Redeemed Paladin** — front {B}, Legendary Creature — Human Knight 2/3:
   *Deathtouch. Darkness — Whenever Cecil deals damage, you lose that much life. Then if your life total is less
   than or equal to half your starting life total, untap Cecil and transform it.* Back: Legendary Creature — Human
   Knight 4/4: *Lifelink. Protect — Whenever Cecil attacks, other attacking creatures gain indestructible until end
   of turn.* Deck: **Tymna/Kraum** (key `kraum`, tier t5; Tymna draws when opponents are dealt combat damage, so
   cheap attackers matter; see `audit/kraum/FINDINGS.md`, `ai/` for kraum/tymna). Use the engine's existing
   transform support (grep `transform` in `cardcode.rs`, `impls/rules.rs`). Status Full.
2. **Lightning, Army of One** {1}{R}{W}, Legendary Creature — Human Soldier 3/2: *First strike, trample, lifelink.
   Stagger — Whenever Lightning deals combat damage to a player, until your next turn, if a source would deal
   damage to that player or a permanent that player controls, it deals double that damage instead.* Deck:
   **Tymna/Kraum** (same). The engine has damage-doubling for other cards (grep `double` in `engine/life.rs`,
   `engine/combat.rs`, `engine/removal.rs`): extend it with the smallest additive change (a per-player "staggered
   until <your> next turn" mark that doubles damage from any source to that player and their permanents). Status
   Full if every damage path doubles, else Approximate with the gap in the note. Kraum's attack AI should prefer
   Lightning's target as the player to keep attacking that turn cycle where it makes sense.
3. **Marvin, Murderous Mimic** {2}, Legendary Artifact Creature — Toy 2/2: *Marvin has all activated abilities of
   creatures you control that don't have the same name as this creature.* Deck: **Kefka** (key `kefka`, tier t3).
   A full generic version is out of scope. Model it for Kefka's creatures only: while you control **Niv-Mizzet, the
   Firemind** ({T}: draw a card), **Viashino Fangtail** ({T}: 1 damage to any target), **Blightspeaker** ({T}: target
   player loses 1 life; {4}, {T}: search for a Rebel permanent card with mana value 3 or less, put it onto the
   battlefield) or **Rummaging Goblin** ({T}, discard a card: draw a card), Marvin can use that ability (one {T}
   ability a turn, he must be untapped and free of summoning sickness). Reuse those cards' existing code
   (`impls/kefka.rs` and wherever they live). AI: use the best of the available abilities each turn (Niv's draw
   when Niv is out, else the others). Status **Partial**, note naming the abilities it copies. Cast priority: a
   cheap creature, higher when one of those creatures is on the battlefield.

## For each card
- Line in `data/carddb.txt` (see `crates/cardgen/README.md`, "Add a card by hand"), an override with `status` and
  `note` in `data/card_overrides.toml`, then `cargo run --release -p cardgen -- export` (from `rust/`; it fetches
  the card from Scryfall if missing). Pattern for a recent small card: The Mind Stone in `impls/common.rs`
  (commit `2e93440`).
- Tests in `tests/cards_tymna.rs` (Cecil, Lightning) and `tests/cards_kefka.rs` (Marvin) that check the Oracle
  behaviour (doc comment quoting the Oracle text, like `mind_stone_harness_and_end_step_blink` in
  `tests/cards_seph.rs`): Cecil's life loss and flip at 20 life (starting 40) and his back face's indestructible
  grant; Lightning's doubling for combat and noncombat damage until its controller's next turn, then not; Marvin
  using Niv's draw only while Niv is out.
- The deck's AI must cast it: run the preflight from the repo root after building `sim-cli`:
  `rust/target/release/commander-sim --deck kraum --pool all --swap "Gilgamesh, Master-at-Arms=>Cecil, Dark Knight" --preflight-only --jobs 4`
  `... --deck kraum --swap "Elemental Mascot=>Lightning, Army of One" ...`, `... --deck kefka --swap "Feed the Swarm=>Marvin, Murderous Mimic" ...`
  (use the card name exactly as `data/cards.json` has it after export). Each must pass; if not, fix the priority.

## Checks before you finish
`cargo test --release` passes (whole workspace); `cargo build --release --all-targets` has no warnings; the three
preflights pass; `cardgen audit --deck kraum|kefka` shows your cards with the status you set.
Handoff: `rust/agents/handoff-new-cards-b.md`; full report `rust/agents/report-new-cards-b.md`; final reply ≤15 lines.
