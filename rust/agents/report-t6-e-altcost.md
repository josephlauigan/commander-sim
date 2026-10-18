# Report t6-e-altcost: alternative costs for hand spells, and the tax on them

## Mechanism
- `hooks.rs`: `AltCost { ok, pay, lost }` and the `CardImpl.alt_cost` slot (a card's alternative cost; `lost` = cards' worth of value given up).
- `engine/altcost.rs` (new): `slot`, `tax` (cost increases/Trinisphere floor a free cast still pays), `tax_payable`, `pay_tax`, `available`, `lost`, `use_alt` (AI rule: pay the alternative cost when the mana cost can't be paid, or it costs nothing of value), `pay_alt` (tax then the card's pay).
- `engine/stack.rs`: `pick_counter` / `pay_counter` / `free_counter` use the slot; `ai_respond` raises the counter threshold by 1.5 per card lost (as the FoW pitch's +1.5); new counter kind `ctr=mvx` and Shoal's {X}{U}{U} in `counter_cost`; Mindbreak Trap exiles every counterable opposing spell (`t6_counters::mindbreak_exile` from `resolve_counter`).
- `ai/brain.rs`: `removal_options` / `cast_removal` / `attack_response` / `wipe_options` / `cast_wipe` use it (utility -1.5 per card lost).
- Slots registered: Daze, Flare of Denial, Disrupting Shoal, Misdirection, Snapback, Commandeer, Mindbreak Trap (`t6_counters::ALT_COST`), Pyrokinesis, Sunscour (`t6_redirects::register_alt_costs`).
- Defense Grid tax now paid by every no-mana cast: Force of Will pitch, Fierce Guardianship, Pact, Misstep, Force of Negation, Freecmd/Snuff removal, Deflecting Swat (mine.rs). The routing of existing Free cards through the slot was NOT done: they are tag modes with human prompts and early returns in pick_counter, and the slot would only have moved code; only the tax was unified.

## Cards
Full: Daze, Flare of Denial, Snapback, Mindbreak Trap, Disrupting Shoal (ctr=mvx; the AI always picks X = the spell's mana value), Pyrokinesis, Sunscour, Defense Grid.
Partial: Misdirection, Commandeer (pitch costs wired; stack effects/casting in response are a later feature).

## Other decks
No behaviour change without Defense Grid (tax is 0). With Defense Grid (Godo deck) free casts now cost {3} off the caster's turn: a rules fix. Cards already in decks that gained a slot (Daze etc.) are now cast for their alternative cost by the AI.

## Tests
`tests/altcost.rs` (15 tests: mechanism, each card, Defense Grid on Daze and FoW). Full `cargo test --release` passes; `cargo build --release --all-targets` has no warnings. Data re-exported with the worktree cardgen. No sim run.

## New needs
engine-needs-found.md: Snapback's db cost looks wrong ({1}{U}, printed {3}{U}); the human (practice) path offers only FoW's pitch.
