# Report t6-e-copies (plan row 13: copy effects)

## Mechanism
- `state.rs`: `CopyExc` (exceptions: added card types, added creature type, ONLY one creature type, power/toughness, colours, flying, haste, not legendary) and `CopyOf { cd, exc }`; `Perm::exc` stores them on the copy.
- `hooks.rs`: new card slot `CardImpl::enter_copy(g, p, c) -> Option<CopyOf>` ("enters as a copy of ..."; not an Event, EVENTS unchanged).
- `engine/zones.rs::enter`: before `new_perm` the slot (or Phyrexian Metamorph's `Tag::Clone` rule, unchanged choice) picks the copied card; the permanent is created as that card with the clone card in `phys`, exceptions applied before any enters trigger. No stand-in body: a clone with nothing to copy enters as itself (0/0 dies in `creature_entered`). `Enter::exc` + `enter_token_copy_with(g, p, cd, exc)` do the same for token copies (`enter_token_copy` = no exceptions).
- Queries: `values::subtypes` (only_subtype, ttypes are the added type), new `values::ptypes` (card types + added; used by `removal::legal_targets`, wipes' type test, `dsl::type_ok`, `ai::decks` mode values, clone `copyable`), `values::colors_of`, `mine::is_legendary` (not_legendary).
## Cards
- Wired: Clever Impersonator, Phantasmal Image, Flesh Duplicate, Copy Artifact, Mirrormade, Sculpting Steel, Mockingbird (target chosen at cast, read back from `g.cast_target`), Copy Enchantment (t5, now `enter_copy`), Molten Duplication (artifact token), Preston (0/1 white, ONLY Illusion, as it enters), Helm of the Host (token not legendary).
- Stand-ins removed: the three clones carry their printed 0/0 (`data/card_overrides.toml`, `cards.json` re-exported); `t6_copies::clone_etb`/`become_copy` deleted.
- Full now: Clever Impersonator, Mockingbird, Copy Artifact, Molten Duplication, Mirrormade, Sculpting Steel, Preston. Still Partial: Phantasmal Image (sacrifice when targeted needs the becomes-target event), Flesh Duplicate (vanishing 3: upkeep hook not tied to the card).
- Godo line: Sculpting Steel enters as Cursed Mirror, Mirror's enters code then makes it a hasty copy of Godo (test `sculpting_steel_copies_cursed_mirror_which_becomes_a_hasty_copy_of_godo`).
## Other decks
No behaviour change intended: Metamorph's choice is unchanged. Copy Enchantment now copies at entry instead of leave-and-re-enter (no stand-in enter event). Preston's Illusion copies now see 0/1, white, Illusion-only during their own enter. Helm tokens are not legendary (affects `is_legendary` queries only; the engine has no legend rule).
## Not converted / needs found
Spark Double (common.rs) and Mirage Mirror (alela.rs) still use leave-and-enter / set_face (counters and until-EOT behaviour); Cursed Mirror stays the Approximate until-EOT model in t0_stella. Imposter Mech, Camera, Stormdrake untouched.
