# Report: doomsday (PR 2, 2026-11-05)

Branch `doomsday`, from deck-rewire f210aac7, merged with origin/main (the rewire, #270) at 0901eff2. Brief:
`rust/agents/doomsday.md`. The agent returned this report as text; the coordinator saved it. No sims (JD: one baseline
after all the engine work).

## 1. The card (`impls/doomsday.rs`)
`apply` takes the chosen cards (at most 5, only those in library or graveyard), exiles every other card of both zones,
sets the library to the chosen order (top = first chosen) and loses `half_up(life)` = ceil(life/2), zero at life <= 0
(40 -> 20, 25 -> 12, 3 -> 1). Fewer than five cards in total: all are taken. Player stat `doomsday` (read by `active()`).
A person: `human_pile` asks `pick_cards` for the five, then the order, top first. The AI: `ai_pile` (`build`, else the
best cards by worth, line cards first). Overrides: Doomsday Full; Teferi, Brainstorm, Gitaxian Probe, Ponder, Preordain
notes updated. `cardgen audit --mine`: 0 not Full in every deck.

## 2. The AI
- **Pile search** (`solve`/`dfs`): exact iterative-deepening search over the known library after Doomsday. Moves: Silence,
  Gitaxian Probe ({U} or 2 life), Borne Upon a Wind, Brainstorm (every ordered put-back pair), Preordain (every scry
  outcome), Ponder (every order of the top three), Thassa's Oracle (wins when devotion without Oracle + 2 >= library size,
  with Silence up when a lock is required). Mana checked with the engine's `can_pay` on cumulative colours; the end-step
  case checks on a clone with everything untapped. A Probe paid in life needs life - 2 >= 3; no draw from a too-short
  library.
- **Builder** (`build`): ordered choices of the useful cards (Oracle, Silence, Probe, Brainstorm, Preordain, Ponder,
  Borne), filled to five with the highest `card_worth`, in EndStep (the draw step takes the first card; skipped under
  Necropotence) or Main; scores (has the lock, shorter line); cached per position. It finds lines equivalent to the
  guide's piles, not always in the printed order; tests check outcomes.
- **Timing and declines** (`plan_cast`/`doomsday_options`, utility 16): end-step cast needs `grants_flash` (Teferi's grant,
  Floodcaller, Borne's flag) or an enabler in hand cast first; main phase when no enabler or no end-step plan. Declines:
  Consultation or Pact in hand (Line 1 is cheaper); life < 16; Oracle not in hand or available; {B}{B}{B} not payable (Dark
  Ritual counts); an opponent's creature power >= life after - 2; two or more opponents with stop chance >= 0.15 and no lock
  in the pile and no counter in hand; Opposition Agent, Archon of Emeria, Deafening Silence, Rule of Law or Ethersworn
  Canonist on an opponent's side; plan gate "Doomsday cast" false.
- **Playing the pile** (`run_pile`): re-solves before every spell (lock wanted when someone can stop it), casts through
  the normal counter window, retries once, uses Noxious Revival on an Oracle countered into the graveyard. Main-phase case:
  the pile follows at once; otherwise the catalogue combo "Doomsday + Thassa's Oracle" plays it next main phase. A stall
  costs a turn, not the game. Cantrip choices in pile mode: `pile_putback` (Brainstorm), `pile_scry` (Preordain),
  `pile_ponder`, `pile_probe_life`; `hold_oracle` holds Oracle while Silence can be cast; Probe priority 0 and Thrasios's
  {4} refused while a pile is active.
- **Plan file** (`data/plans/thrasios.toml`, PROVISIONAL): `[[line]]` "Doomsday + Thassa's Oracle"; `[[prio]]` Doomsday 0;
  `[[go_now]]` true for the pile combo; `[[wish]]` Doomsday, Silence, Probe/Preordain/Ponder/Brainstorm; sets `exilers`,
  `enablers`, `locks`; `action` set gained Doomsday, Probe, Brainstorm, Ponder, Preordain, Borne; keep rule
  `have:Doomsday & hand-b-lands=0 & hand:action<=1` -> mulligan (new quantity `hand-b-lands` in `ai/plan_data.rs`).

## 3. Enablers
Teferi, Time Raveler's +1 was a no-op: now `Player::sorc_flash` (new field) until that player's next turn, read by
`cast::grants_flash` (AI end-of-turn window and the person's timing check in `human/legal.rs`); AI +1 value 6 with
Doomsday in hand and no grant yet, else 1.5; the person's +1 (`human/walkers.rs`) sets it. Valley Floodcaller and Borne
Upon a Wind already granted flash (tests confirm end-step Doomsday with each).

## 4. Person
Doomsday (`human_pile`); Ponder asks the order, then "shuffle?"; Brainstorm asks for the two cards (`put_back` returns
Res; the last pick ends on top); Preordain already asked; Gitaxian Probe was charged twice for a person (asked at cast,
paid again at resolve): now pays once, at cast. `practice_choice_probe` TO_FIX empty.

## Tests (`tests/doomsday.rs`, 26)
Card rules (search, exile, order, fewer than five, life rounding, the person's questions); Piles A, B, C and the Thrasios
variant played by the AI to the win (A, B, C cast Silence first); no pile when the cards don't suffice; Silence stops
Counterspell/Force of Will; a stalled pile recovers next turn; a counter war doesn't replay a countered cantrip; Noxious
Revival on a countered Oracle then the win next turn; end-step casts with Teferi, Floodcaller, Borne, none without an
enabler; Teferi's grant ends at the next turn; declines (Line 1 in hand, lethal board power, life < 16, Opposition Agent /
Archon / Deafening Silence); the keep rule; a person's Ponder, Brainstorm and Probe. The agent ran 1113 sim-core tests in
the related files plus practice crate files: all pass; build without warnings.

## Open (not to spec), from the agent
1. Pile extras the search doesn't use: Thrasios's {4} dig as a draw source, Mockingbird/Snapback as outs, Veil of Summer
   in place of the lock against Mindbreak Trap / Red Elemental Blast, Esper Sentinel's forced draws, Borne's own draw when
   Borne is the end-step enabler. Some winning lines missed; a mid-pile Sentinel draw could break a pile.
2. Noxious Revival is a recovery, not a pile slot: the builder doesn't use it to accept riskier piles.
3. The keep rule is colour-blind except `hand-b-lands`; the guide's "no coloured source for its spells goes back" is not a
   general rule (would be a general coloured-source quantity in `plan_data.rs`).
4. The Doomsday cast doesn't run the table's "can it be stopped" risk check (`go_risk`), only its own stopper count and the
   plan gate.
5. Shared behaviour changes: Gitaxian Probe pays once for a person; Brainstorm has a person path; Thrasios's {4} refused
   while a pile is active.

## Shared-file changes
`state.rs` (`Player::sorc_flash`), `engine/cast.rs` (`grants_flash`), `impls/t3.rs` (Teferi +1), `human/walkers.rs`,
`impls/topdeck.rs` (`put_back`, Ponder), `ai/topdeck.rs` (pile scry), `impls/partials.rs` (`probe`), `impls/tymna_cards.rs`
(Thrasios refused mid-pile), `impls/combos.rs` (pile combo, `oracle_prio`), `impls/mod.rs`, `ai/plan_data.rs`
(`hand-b-lands`), `data/card_overrides.toml`, `data/cards.json`, `data/plans/thrasios.toml`,
`decklists/JD/tymna-thrasios-doomsday.md` (two stale "no card code" lines), `audit/thrasios/FINDINGS.md` ("Doomsday in the
simulator"), `tests/practice_choice_probe.rs`.
