# Task: cand2-yawgmoth — model the Yawgmoth candidate deck and write its AI plan

Base branch: `rework-candidates`. Deck: `decklists/JD/candidates/yawgmoth.md` (key `yawgmoth`): mono-black
aristocrats. Your module: `impls/yawgmoth.rs` (stub in place), tests `tests/cards_yawgmoth.rs`.

**Not yours** (the cand2-gix agent models them; they may show as not Full until its work is merged): Al Bhed
Salvagers, Callous Inspector, Corpulent Corpse, Dark Matter Manipulator, Gollum, Silent Slinker, Grim Wanderer,
Necrosquito, Restless Bloodseeker, Salvaged Manaworker, Sangrophage, Overkill.

## Cards to model (until `cardgen audit --mine` shows each as Full)
- **Yawgmoth, Thran Physician** (currently Partial): *protection from Humans; pay 1 life, sacrifice another creature:
  put a -1/-1 counter on up to one target creature and draw a card; {B}{B}, discard a card: proliferate.* Make all of
  it work, including -1/-1 counters killing creatures and proliferate on -1/-1 and +1/+1 counters.
- Braids's Frightful Return (read ahead Saga), Presumed Dead, Rewrite Regrets, Soul Enervation, Arnim Zola,
  Bio-Fanatic, Bloodsworn Squire (transforms), Catapult Fodder (transforms), Cheerful Osteomancer (prepared), Dread
  Drone, Festering Goblin, It Doesn't Add Up (suspect), Perigee Beckoner (warp), Phyrexian Delver, Send in the Pest.

## AI plan for yawgmoth
- Sacrifice to Yawgmoth when the card is worth the life: fodder first (tokens, creatures with death value, creatures
  about to die anyway), keep life above about 10; the -1/-1 counters go on the opposing creature they kill or shrink
  the most (toughness 1 creatures first).
- Proliferate with Yawgmoth's discard when it kills something or grows a big threat.
- Recursion spells bring back the best creature card in the graveyard.

Smoke run: `--deck yawgmoth`.
