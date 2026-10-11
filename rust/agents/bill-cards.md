# Task: bill-cards — model the Bristly Bill deck's last 17 cards and write its AI plan

Read `rust/agents/OVERNIGHT-RULES.md` first: its hard rules (never run Python, memory cap, edit only your files,
commit often, handoff, 15-line report) apply here. Where it says "Kefka/Zada", read "Bill".

Base branch: `bill-deck`. Deck: `decklists/JD/bristly-bill-landfall.md` (key `bill`): mono-green landfall counters.
Each land puts a +1/+1 counter on a creature with Bristly Bill, Spine Sower; his {3}{G}{G} doubles every creature's
counters. Your module: `impls/bill.rs` (stub in place), tests `tests/cards_bill.rs`. Smoke run: `--deck bill`.

## Cards to model (until `cardgen audit --mine` shows each as Full; `bill: ... 0 not Full`)
Unmodeled: **Hardened Scales** (a replacement on every +1/+1 counter placement on your creatures: find the one place
counters are added, or the few, and apply it there; it must also apply to Bill's landfall, Bill's doubling, Additive
Evolution, enters-with-counters and so on. Two Scales stack.)
Partial:
- **Bristly Bill, Spine Sower**: the {3}{G}{G} doubling (put as many counters as each creature has; Scales adds one).
- Boughside Wanderers, Cactarantula, Carnivorous Canopy, Freestrider Commando (plot), Ghalta the Unstoppable,
  Greenbelt Radical (disguise: a face-down 2/2 with ward {2}, turned up for {5}{G}{G}), Lifestream's Blessing
  (foretell), Sample Collector (collect evidence 3), Silhana Ledgewalker, Spider-Man, Brooklyn Visionary
  (web-slinging), Studious First-Year (prepared: cast a copy of Rampant Growth), Tumbleweed Rising (plot), Venomous
  Brutalizer (toxic 3; pay {1}{G} to proliferate), Wildgrowth Archaic (converge), Royal Treatment (it's Approximate:
  the Royal Role is a +1/+1 counter, not a Role with ward {1}).
The audit's line for each says what is missing. Existing machinery to reuse (grep before writing anything new):
plot (`impls/zada_cards.rs`, `impls/lordofpain.rs`), foretell (`impls/lordofpain.rs`), prepared and increment
(`impls/jenova.rs`, `state.rs`, `hooks.rs`), toxic (`state.rs`, `engine/removal.rs`), proliferate (`engine/turn.rs`,
`impls/yawgmoth.rs`), converge (`impls/kefka.rs`), landfall (`engine/turn.rs`, `impls/t1.rs`, `impls/t3.rs`), bites
and fights (`impls/glissa.rs`). Disguise, collect evidence and web-slinging are new: build each once, generally.

Check first how Retreat to Kazandu, Druid Class, Evolution Sage, Harrow and the fetch lands are modeled (the audit
says Full): if one of them is wrong for this deck, fix it and say so in your report.

## AI plan for bill (in `impls/bill.rs`, wired through `bill_prio` and the deck-options dispatch)
- **Sequencing:** cast Bill on turn 2 when you can; once he is out, save ramp spells, Harrow and fetch lands so their
  lands enter while he is on the battlefield (crack fetches at once when he is out; with no Bill, crack them for mana
  as normal).
- **Landfall target:** the creature that gains most: Silhana Ledgewalker (hexproof, hard to block) first, then the
  creature that already has the most counters (doubling), then increment/mana creatures (Topiary Lecturer taps for
  its power), Bugenhagen to reach power 7. Not a creature that is about to die.
- **Doubling ({3}{G}{G}):** at instant speed: end of an opponent's turn with the mana free, or before combat damage
  when it makes an attack lethal or much bigger. Only with 4 or more counters on board.
- **Removal:** bite/fight spells with the biggest creature; Harrow at instant speed (two triggers).
- **Finish:** Overwhelming Stampede / End-Raze Forerunners when the alpha strike is lethal or near it.
- Gate any change to shared AI code (`ai/*`, `engine/*`) to `key == "bill"` so other decks play exactly as before.

## Done when
The OVERNIGHT-RULES checks all pass (full `cargo test --release`, no warnings, `all_decks adaptive 3` and
`lookahead 1`, audit `bill: 0 not Full`, a 48-game smoke run with `--deck bill`). Report as those rules say, with the
first line DONE or HANDOFF and the second line your model ID.
