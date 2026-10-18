# Report t6-e-castctr (counter-on-cast hook)

Mechanism
- `Event::CastCounter` / `CardImpl::cast_counter` (hooks.rs): `fn(&Game, Src, caster, CardId, &Ctx) -> bool`. `engine/hooks.rs::cast_countered`
  is called in `engine/cast.rs::cast_card` after `on_cast` and the Jin-Gitaxias check, once the stack item exists;
  it counters when `counterable(item)` (Squelcher etc. still win). The spell was cast (triggers, storm count), then goes
  to the owner's graveyard through the normal countered-spell path.
- Mana spent record: `Game::mana_spent / mana_colored` filled in `mana::spend_unit` (a source with colours counts as
  coloured, even for a generic cost), restarted at each `mana::pay` (human pool payments too), kept across the tax and
  the alternative cost in `altcost::pay_alt` (`spent_keep`). `cast_card` takes it into `Ctx::mana_spent / mana_colored`
  (ready for Exhibition Tidecaller, Vibrance). A free cast pays {0} (restarts the record) so it reads 0.
  `Game::entering_spent` carries the spent mana to the entering permanent.
- Chalice (t6_stax.rs): `chalice_x(g, p)` picks X 1..3 by the cards still in hands/libraries (opponents' minus own) that
  an X would hit, X=0 only when every X>=1 hurts the owner more than the table, None (cost 99, not cast) when X=1 is
  unaffordable; cost {X}{X}; counters = mana paid / 2. Locks removed from Bauble, Chalice, Mirror.
- Also fixed: `EVENTS` was not sorted ("gy_upkeep" before "gy_options"), which `Event::from_name` (binary search) needs; it only
  worked by luck of the probe path until the 80th event.

Cards: Vexing Bauble, Chalice of the Void, Void Mirror -> Full (card_overrides.toml; export re-run, data unchanged otherwise).
Void Mirror follows the Oracle text ("if no coloured mana was spent"); I did not add a "colourless spell" exception.

Tests: tests/castctr.rs (13: record, Bauble free/own/alt, Chalice 0/1/2/cast X, Mirror); old lock tests removed from
cards_t6_stax.rs / cards_t6_godo.rs. Full `cargo test --release` passes, no warnings.

Other decks: none change (the new code only acts with these three cards on the battlefield; the record is bookkeeping).

Gaps / needs
- Free casts that never call `pay` (cascade, "without paying", Bolas's Citadel life) read a stale record if an earlier
  payment (an ability) was not followed by a cast; paths using `pay` or `pay_alt` restart it. Wrap such casts if it matters.
- The AI still casts its own free spells into its own Bauble/Chalice(0)/Mirror (no AI avoidance; the old locks avoided it).
- A generic cost paid from a mix of colourless and coloured sources: the engine's payment picks, Void Mirror reads that.
- The AI no longer casts into a cast-counter: castable() asks hooks::would_be_cast_countered (people may still).
