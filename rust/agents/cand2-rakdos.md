# Task: cand2-rakdos — model The Lord of Pain and Smaug the Impenetrable decks and write both AI plans

Base branch: `rework-candidates`. Decks: `decklists/JD/candidates/lordofpain.md` (key `lordofpain`) and
`decklists/JD/candidates/smaug.md` (key `smaug`). Your modules: `impls/lordofpain.rs` and `impls/smaug.rs` (stubs in
place), tests `tests/cards_rakdos.rs`.

**Not yours** (the cand2-gix and cand2-yawgmoth agents model them): Apex Witchstalker, Necrosquito, Salvaged
Manaworker, Key to the Side-Door, Dread Drone.

## Cards to model (until `cardgen audit --mine` shows each as Full)
- **The Lord of Pain** (5/5 menace): *your opponents can't gain life. Whenever a player casts their first spell each
  turn, choose another target player. The Lord of Pain deals damage equal to that spell's mana value to the chosen
  player.* It triggers on every player's first spell, yours included; the chosen player can't be the caster.
- **Smaug the Impenetrable** (8/7 flying, indestructible, haste): *whenever Smaug is dealt noncombat damage, create
  that many Treasure tokens.*
- Charforger (oil counters), Kalain, Reclusive Painter, Lagomos, Hand of Hatred, Ultimate Magic: Meteor, Hero's
  Heirloom.

## AI plans
- **lordofpain:** the Lord's target is the opponent closest to dying (or the biggest threat); cast a cheap spell first
  on your own turn only when a bigger one isn't coming, since only the first spell counts.
- **smaug:** ramp toward seven, cast Smaug as soon as possible, attack with him every turn, and ping Smaug with your own
  pingers (Goblin Medics, Viashino Fangtail, Goblin Arsonist, Orcish Cannonade) for Treasure when that's the best use.

Smoke runs: `--deck lordofpain` and `--deck smaug`.
