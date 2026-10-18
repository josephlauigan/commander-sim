# Report: audit-core (AI audit phase 2, core items)

Branch audit-core. Commits (in order): Teferi's Protection (Y'shtola), cardgen audit --md guard, instant-tutor
finishers, --cards stale lines, JD zero-priority hold.

1. brain.rs card_utility: for JD decks (`ai::jd_deck`: the 12 keys of data/my_decks.toml) a 0 from the deck's own
   priority is no longer replaced by `dsl::card_value` when the card is an instant or tagged rem/wipe/ctr/prot. Other
   0-priority cards (sorceries, permanents without those tags) keep the fallback: Titania's Rampant Growth, Thirsting
   Roots, Pick Your Poison, Bestial Incursion and Sheoldred's Carnage Altar would otherwise never be cast (generic_prio
   gives untagged DSL cards 0). Cards that change (priority 0 at turn 1 and 8; tests/jd_zero_prio.rs prints them):
   seph Momentary Blink; veyran Lightning Strike, Unsummon; galadriel Beast Within, Make a Stand; kefka Contract
   Killing, Crux of Fate, Dark Ritual, Deadly Embrace, Mana Leak; titania Blossoming Defense, Rending Vines;
   sheoldred Battle-Rage Blessing, Disembowel, Masterful Flourish, Sephiroth's Intervention, Silence the Echo, Stir Up
   Trouble, Terror, Vanquish the Weak, Wander Off; cloud Day of Judgment, Fumigate, Stand Up for Yourself; sephfs Crux
   of Fate, Dark Ritual, Sephiroth's Intervention, Wander Off. sauron, alela, yshtola, kraum: none. Pool decks unchanged.
2. Teferi's Protection (Y'shtola only; pool decks already use the full card in ai/pool.rs): zur_protect casts the real
   `zones::cast_teferis_protection` (exiles itself, life locked, protection from everything, all phased out) only when
   `pool::worth_whole_board` (commander or >= 40% of the board); zur_wipe_response takes the loss over all her permanents
   by the wipe's own rule (`decks::wipe_loss`), so enchantment/artifact modes count; Clever Concealment unchanged.
   Lethal damage/drain stays with the engine's `last_chance`. `zur_wipe_response` / `yshtola_wipe_response` got a
   `caster` argument.
3. cardgen: `audit --md FILE` refuses any path under decklists/ and any existing file that is not an audit table
   (`audit::check_md_target`); --deck/--mine/--cards already only print. Test audit_never_overwrites_a_decklist.
   I could not find another write path in cardgen audit.
4. yshtola::tutor_pick offers the X drains to "is" tutors as well as "any" (Greaves stays "any" only). Y'shtola-only.
5. cardgen sources.rs: an override with status Full clears the card's `unparsed` lines; data/cards.json re-exported
   (only the `unparsed` arrays changed). Venat, Static Net, Kraum, Black Widow, Smaug, Gilgamesh now print none.

Checks: build --all-targets no warnings; `cargo test --release --no-fail-fast` all pass (RUST_TEST_THREADS=2). Pool check
`--calibrate within --pool t3 --games 40 --ai adaptive --jobs 2`: output identical to the ai-audit binary (Teferi's
change is Y'shtola-only; pool decks already used the full card in ai/pool.rs, so no pool difference).
Note: first version of item 1 broke cards_jd_pkg (Kefka's Dark Ritual under Yawgmoth's Will); rituals are now exempt.
