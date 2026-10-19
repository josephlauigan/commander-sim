# new-cards-docs report
Five swaps applied via `cardgen deck` (CHANGELOG lines logged), deck docs prose/type counts updated, bulk-cards.txt
(+Feed the Swarm soa, Dread Return dmr, Citizen's Arrest dmu; Elemental Mascot 2x->3x, Hero's Blade 1x->2x since already
listed), binder-cards.txt untouched (none of the incoming cards listed), data re-exported, fixture updated.
Extra fix: removed `Andúril, Narsil Reforged|A|2|leg` from data/carddb.txt. Once a JD deck played it, the hand-tag
line made cardgen ignore the override's equip_cost dsl and test anduril_narsil_is_cast_and_equipped_to_cloud failed.
Printings (Scryfall default; JD to correct): Marvin (dsk) 253; Victimize (tdc) 198; Lightning, Army of One (fin) 233;
Stroke of Midnight (frc) 35; Andúril, Narsil Reforged (ltc) 491.
Checks: cargo test --release --no-fail-fast 2105 passed 0 failed; all four lists 100 cards.
Unclear: bulk counts for Mascot/Hero's Blade (printing lists left as is); Andúril ltc 491 may be a variant frame.
