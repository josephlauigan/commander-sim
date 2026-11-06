# Tymna/Thrasios (key `thrasios`): findings

The deck's living list of strengths and weaknesses and what was tested.

**List:** `decklists/JD/tymna-thrasios-doomsday.md`, the Doomsday rebuild of 2026-11-05 (cEDH; the Thanos turbo core under
Tymna and Thrasios, Doomsday and three cantrips in). **Tiers:** t6 (`data/my_decks.toml`). **No sims yet:** Doomsday has card code, a pile builder and pile play (2026-11-05, `impls/doomsday.rs`, tested in `tests/doomsday.rs`), but
nothing has been simulated on this list. The findings of the previous list (t5, Basalt
Monolith + Rings + Ballista deck with attack taxes) are in `FINDINGS-old-list.md`, and the retired Thanos turbo list's
(t6) in `audit/thanos/FINDINGS-old-list.md`.

## Doomsday in the simulator (2026-11-05, before any sim)

What the AI does with Doomsday (`impls/doomsday.rs`; all of it untested by games, only by `tests/doomsday.rs`):

- **Pile.** An exact search over the guide's Piles A, B, C and the devotion-3 variants (Probe, Brainstorm, Preordain,
  Ponder, Borne Upon a Wind, Silence, Thassa's Oracle) with the mana and life it will really have; the pile with the lock
  (Silence) first and the fewest spells wins. Fillers: the highest `card_worth` cards left.
- **Timing.** The end step of the turn before its own with an instant-speed enabler (Teferi +1, Valley Floodcaller, or a
  Borne Upon a Wind cast first); in its main phase (Pile C) only when no enabler is around. Declines with Demonic
  Consultation or Tainted Pact in hand, below 16 life, into a board whose power reaches the life that is left, and into
  two or more opponents who can stop a spell with no lock in the pile and no counter held.
- **Pile extras (doomsday-b, 2026-11-06).** The search also uses Thrasios's {4} dig as a draw source (scry 1, then a land
  goes onto the battlefield and anything else is drawn: either way the library is one card shorter); a devotion
  permanent in hand (Mockingbird, Mystic Remora, Rhystic Study, Valley Floodcaller) as one more devotion; Veil of Summer
  held with {G} open as the lock when Silence cannot be (it answers a blue or black counter on the Oracle, not Mindbreak
  Trap or a red blast, so it is only used when no opponent's deck holds those); and a Noxious Revival in hand (or drawn
  by the line) as a backup that lets a pile without the lock be accepted into two stoppers. Piles are checked against a
  forced draw (our Esper Sentinel when an opponent casts a spell) and the draw step's extra cards (Howling Mine, Font of
  Mythos). Noxious Revival is aimed at the countered Oracle, not the best-valued card of the graveyard.
- **The cast and the table.** The Doomsday cast now makes the same `go_risk` check as the other Tier 6 lines
  (`t6_table::go_risk_covered` against `go_threshold`): it waits when the table can stop it and there is no cover (Silence
  or Chant in hand, a free counter of ours, the lock in the pile, Veil held).
- **Keep rule.** `hand-castable:action=0` with an action card in hand sends the hand back: the hand's lands, a fetch
  (the colours of the library's lands) and the rocks and dorks it can cast must make the colours its action cards need.
  It replaces the Doomsday-only `hand-b-lands` term.
- **Still not modelled / to watch in the first sim:** Opposition Agent is declined, not played around; an opponent's Esper
  Sentinel taxes our first noncreature spell ({1} or they draw: the Doomsday cast in the main phase); Thrasios cast from
  the command zone as a devotion and dig source mid-pile; Mockingbird / Snapback / Chain of Vapor to redo an Oracle trigger
  that was countered (Stifle); the cast is not held for a better turn once it is legal.
