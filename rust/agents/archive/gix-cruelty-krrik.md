# Task: gix-cruelty-krrik, model The Cruelty of Gix and K'rrik, Son of Yawgmoth (and let the AI use them)

Read `rust/agents/RULES.md` (skip `git merge rust-port`; the worktree is on main). Worktree:
`/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/gix-cruelty-krrik`, branch `gix-cruelty-krrik`.
JD wants to test **The Cruelty of Gix in Seph FS** (key `sephfs`, T5) and **K'rrik, Son of Yawgmoth in Sheoldred**
(key `sheoldred`, T3-T4). Neither card is in `data/cards.json` yet. Model both so the AI plays them well; I run the
sims afterwards.

## The cards (Oracle, from Scryfall)
- **The Cruelty of Gix** {3}{B}{B}, Enchantment — Saga. Read ahead (choose a chapter and start with that many lore
  counters; add one after your draw step; skipped chapters don't trigger; sacrifice after III).
  I — Target opponent reveals their hand. You choose a creature or planeswalker card from it. That player discards
  that card. II — Search your library for a card, put that card into your hand, then shuffle. You lose 3 life.
  III — Put target creature card from a graveyard onto the battlefield under your control.
- **K'rrik, Son of Yawgmoth** {4}{B/P}{B/P}{B/P}, Legendary Creature — Phyrexian Horror Minion, 2/2. Lifelink.
  For each {B} in a cost, you may pay 2 life rather than pay that mana. Whenever you cast a black spell, put a
  +1/+1 counter on K'rrik.

## Do
1. Get them into the card data the repo's way (`cardgen`: overrides in `data/card_overrides.toml`, `cardgen export`;
   check how existing sagas (read-ahead ones too, e.g. Braids's Frightful Return in `impls/sephfs_cards.rs`) and
   Phyrexian mana are modeled, and reuse that machinery). Add an override `status` and `note` for each.
2. **Cruelty of Gix (AI):** choose the starting chapter by value: III at once when any graveyard (an opponent's
   too, after a wipe) holds a creature worth reanimating; II (tutor) when the deck needs a piece and life allows;
   I otherwise. Reuse the deck's reanimation value (`sephfs.rs` / `sephfs_cards.rs` have the Seph FS plan's
   reanimation targets) and the shared tutor choice. Person path: ask the chapter and the choices
   (`human::choices`), as the newer practice code does.
3. **K'rrik (AI):** paying 2 life per {B} must reach the engine's cost payment for the AI (and a person): when K'rrik is
   out, black pips may be paid with life. The AI pays life only while it stays above a safe floor (Sheoldred's plan
   drains and gains; choose a floor like 15 and justify it in the report). His +1/+1 counter on each black spell.
   The {B/P} in his own cost: payable with life too.
4. Cast priority: add both to the relevant deck plans so the AI casts them sensibly (Cruelty in Seph FS, K'rrik in
   Sheoldred); a generic priority is acceptable if a deck rule is overkill, but check the preflight says they're cast.
5. Tests: each chapter of Cruelty (AI choice of chapter in two board states, the person choosing), K'rrik paying
   life for {B} (AI above and below the floor), the counter on a black spell.

## Checks
`cargo test --release -j 3 --test-threads=3` passes (whole workspace), `cargo build --release -j 3` has no warnings,
and `./target/release/commander-sim --deck sephfs --pool t5 --swap "Mind Stone=>The Cruelty of Gix" --games 50 --ai
adaptive --profile loose --jobs 3` and `--deck sheoldred --pool t3 --swap "Stir Up Trouble=>K'rrik, Son of Yawgmoth"`
run and the preflight shows each card cast. **These cards are new, so sims may change for decks that play them, but no
other deck's games may change:** run `--deck cloud --pool t1 --games 200 --ai adaptive --profile loose --jobs 3
--no-preflight` before and after and diff (must be identical; make sure both outputs contain a "Win rate" line).

Limits: `-j 3`, `--test-threads=3`, `nice -n 19`, memory cap per RULES.md, no Python, no other sims. Run long commands
in the foreground with a timeout. Handoff `rust/agents/handoff-gix-cruelty-krrik.md` early; commit after each piece
(standalone, no Co-Authored-By); don't push. Report in 15 lines or fewer; details in
`rust/agents/report-gix-cruelty-krrik.md`.
