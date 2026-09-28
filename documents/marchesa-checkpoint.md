# Checkpoint: adding the Marchesa deck (2026-09-28)

Work in progress: adding decklists/mine/marchesa-grixis-recursion.md as a fourth "your deck" (key `marchesa`) with
every card fully modeled. Nothing is committed yet; all changes are in the working tree.

## Done

- Deck file saved; list validated (100 cards, colour identity, 3 Game Changers, Bracket 3).
- Key `marchesa` registered: decks.py, poolmode.MINE, compare.KEYS/PLAN/plan_turn (milestone 'recur'),
  engine IDENT/NAME/TOKEN_COLOR/CTHRESH/PROFILES/spell_imp, ais.CMDS/MAIN/deck_prio, brain.STYLE/PRIO,
  t2.STYLE_KEYS, update_deck (MINE, fixture). Fixture re-recorded (tests/fixtures/my_decks_parsed.json).
- New module commander_sim/cards/impl/marchesa.py (loaded last in cardimpl.load): Marchesa's trigger and
  end-step return (with look-back in wipes via g.batch), dethrone grant (cardimpl.keyword_attack), sac_worth,
  AI plays (re-buy sacrifices, Triskelion loop, sacrifice stolen creatures, protect / wipe responses, reanimation
  targets, dredge), and hooks for: Act of Treason, Enslave, Soul Enervation, Al Bhed Salvagers, Balustrade Spy,
  Coalition Relic, Relic of Legends, Gemstone Mine, Vivid Creek/Marsh, Crackling Drake, Deep Analysis,
  Disintegrate, Disembowel, Lethal Throwdown, Festering Goblin, Forge Devil, Grave Researcher, Phyrexian Delver,
  Pteramander, Skullport Merchant, Dualcaster Mage, Notion Thief flash, Crux of Fate, Hellkite Tyrant alt win,
  Accursed Marauder (was 3/1, now the correct 2/1).
- carddb.py lines: Terror, Last Gasp, Orcish Cannonade, Scorching Dragonfire, Premature Burial, Zombify;
  Tezzeret's Gambit gets `phyU`.
- Engine: sac_worth, removal kinds `shrinkN` / tags `exiledie` `noregen` `newonly`, full proliferate for
  `prolif1`, Sephiroth's -2/-2 as real -2/-2, LAND_COLS + TAP_COLS for depleting lands, `hand_opp_cast` event,
  Phyrexian blue, flashback life (`fblife`), and a fix for two opposing Notion Thieves looping forever
  (rule 614.5).
- Audit: all 83 unique cards Full or Full-auto (8 Full-auto hand-checked against Oracle).
- Old test suite: 136/137 passed before the fixture re-record (the one failure was the expected fixture).

## Left to do

1. tests.test_my_cards.Marchesa: 4 of 26 fail.
   - test_removal_restrictions: Orcish Bowmasters also makes an Army token, a third target. Fix the test
     (ignore tokens).
   - test_lethal_throwdown...: Murmuring Mystic is worth only 1.5 to the AI, so no option. Use Guttersnipe
     as the target and put Marchesa on the battlefield.
   - test_disembowel...: no option offered even with Guttersnipe (pval 4, 3 Swamps, legal target found). Debug
     _disembowel_opts in marchesa.py (the utility cutoff, or can_pay with PAY_FOR).
   - test_stinkweed_imp_and_nim...: a REAL gap. Nim Deathmantle (mine.py _nim_return) still pays {4} for a
     creature Marchesa will return for free. Skip when the card is in g.marchesa_due.
2. Run the full suite: python3 -m unittest discover -s tests -t .
3. Docs: documents/card-audit.md (Marchesa section), README.md (--deck choices, "three decks"),
   tests/README.md (test counts), documents/architecture.md mentions of the three decks.
4. Measure: python3 -m commander_sim --deck marchesa --pool all --games 300 --ai adaptive (it ran cleanly
   before; 20% vs t2 on 40 games).
5. Tell the user: the deck file says it shares no cards with the other decks, but Stinkweed Imp, Triskelion
   and Nim Deathmantle are also in the Sephiroth list, and Tezzeret's Gambit is in Sauron.
