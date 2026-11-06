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
- **Not modelled / to watch in the first sim:** Esper Sentinel's forced draws and Opposition Agent (the guide says never
  cast it under Agent: the AI does not check); an enabler Borne's drawn card is not used by the pile; Noxious Revival is
  used after a counter (it is not a pile slot); the cast is not held for a better turn once it is legal.
