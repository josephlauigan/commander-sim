"""The Game Changer audit (2026-10-05, audit/GAME_CHANGERS.md): one regression test per bug fixed, so none comes back."""
import unittest

from tests.table import table, hand, lands, perm, card
from commander_sim import engine as E, ais
from commander_sim.ai import brain, pool_ai, gc_prio


class Mana(unittest.TestCase):
    def test_gaeas_cradle_makes_one_mana_per_creature(self):
        # variable mana (Cradle, Mox Opal, the elf lords ...) used to make a flat 1 unless the card had an event hook
        g = table('yshtola', 'veyran'); p = g.players[0]
        lands(p, "Gaea's Cradle")
        self.assertEqual(E.total_mana(g, p), 0)
        for _ in range(3): perm(g, p, 'Llanowar Elves')
        self.assertEqual(E.total_mana(g, p), 6)                 # three from Cradle, three from the elves

    def test_ancient_tomb_is_not_tapped_when_it_would_kill_you(self):
        g = table('veyran', 'seph'); p = g.players[0]
        lands(p, 'Ancient Tomb')
        self.assertEqual(E.total_mana(g, p), 2)
        p.life = 2
        self.assertEqual(E.total_mana(g, p), 0)

    def test_mox_diamond_waits_for_a_land_to_discard(self):
        # it carries rock=1:A too, and the rock rule used to fire first: cast with no land, it went to the graveyard
        g = table('kinnan-simic-mana-combo', 'veyran'); p = g.players[0]; p.turns = 6
        mox = hand(p, 'Mox Diamond')
        self.assertEqual(pool_ai.generic_prio(g, p, mox), 0)
        hand(p, 'Forest', 'Island')
        self.assertGreater(pool_ai.generic_prio(g, p, mox), 0)

    def test_mox_diamond_put_onto_the_battlefield_still_needs_a_land(self):
        g = table('kinnan-simic-mana-combo', 'veyran'); p = g.players[0]
        E.enter(g, p, card('Mox Diamond'))                      # Urza's Saga's chapter III, with no land in hand
        self.assertFalse(any(m.cd is not None and m.cd.name == 'Mox Diamond' for m in p.perms))
        self.assertTrue(any(c.name == 'Mox Diamond' for c in p.gy))


class Constructs(unittest.TestCase):
    def test_a_construct_token_survives_and_grows_with_artifacts(self):
        # make_tokens checked toughness before the caller attached the Construct's data: every 0/0 Construct died
        g = table('sauron', 'veyran'); p = g.players[0]
        perm(g, p, 'Sol Ring')
        g.selfpt = True
        t = E.make_tokens(g, p, 1, 0, 0, color='', types=('construct',), data={'construct': True, 'artifact': True})
        self.assertEqual(len(t), 1)
        self.assertEqual((E.epow(g, t[0]), E.etgh(g, t[0])), (2, 2))     # Sol Ring and itself

    def test_urzas_saga_makes_two_constructs_and_fetches_an_artifact(self):
        from commander_sim.cards.impl import common as IC
        g = table('sauron', 'veyran'); p = g.players[0]
        lands(p, 'Island', 3); lands(p, "Urza's Saga"); perm(g, p, 'Sol Ring'); perm(g, p, 'Arcane Signet')
        p.sagas = [[p.lands[-1], 1]]
        IC.saga_step(g, p)                                      # chapter II
        for L in p.lands: L.tapped = False
        IC.saga_step(g, p)                                      # chapter III: one more in response, then the search
        self.assertEqual(sum(1 for m in p.perms if m.data and m.data.get('construct')), 2)
        self.assertFalse(any(L.cd.name == "Urza's Saga" for L in p.lands))


class Wipes(unittest.TestCase):
    def test_cyclonic_rift_is_overloaded_at_the_end_of_the_turn_before_yours(self):
        g = table('veyran', 'sauron', 'yshtola', 'seph'); p = g.players[0]
        lands(p, 'Island', 7); hand(p, 'Cyclonic Rift')
        perm(g, g.players[1], 'Grave Titan'); perm(g, g.players[2], 'Consecrated Sphinx')
        perm(g, g.players[3], 'Smothering Tithe')
        g.active = g.players[3]
        brain.end_of_turn_window(g, p)
        self.assertEqual([len(q.perms) for q in g.players[1:]], [0, 0, 0])

    def test_one_target_is_still_bounced_alone(self):
        g = table('veyran', 'sauron'); p = g.players[0]
        lands(p, 'Island', 7); hand(p, 'Cyclonic Rift'); perm(g, g.players[1], 'Grave Titan')
        g.active = g.players[1]
        brain.end_of_turn_window(g, p)
        self.assertEqual(sum(L.tapped for L in p.lands), 2)     # {1}{U}, not the overload's seven

    def test_a_wipe_that_misses_your_board_costs_you_nothing(self):
        # Teferi's Protection used to fire on a Vandalblast against a board of creatures
        g = table('seph', 'veyran'); q, c = g.players
        for n in ('Grave Titan', 'Consecrated Sphinx'): perm(g, q, n)
        self.assertEqual(ais.wipe_loss(g, q, 'vandal', c), 0)
        self.assertGreater(ais.wipe_loss(g, q, 'destroy', c), 10)
        lands(q, 'Plains', 3); hand(q, "Teferi's Protection")
        self.assertIsNone(ais.wipe_response(g, q, 'vandal', c))

    def test_farewell_keeps_your_reanimation_targets(self):
        g = table('seph', 'veyran'); p, o = g.players
        hand(p, 'Animate Dead'); p.gy += [card("Atraxa, Praetors' Voice"), card('Consecrated Sphinx')]
        o.gy += [card('Lingering Souls')]; perm(g, o, 'Grave Titan')
        self.assertNotIn('gy', ais.wipe_modes(g, p, 'farewell'))


class Combos(unittest.TestCase):
    def _oracle(self, library):
        from commander_sim.cards.impl import combos as C
        g = table('veyran', 'seph'); p = g.players[0]
        lands(p, 'Island', 2); lands(p, 'Swamp', 2)
        hand(p, "Thassa's Oracle", 'Tainted Pact')
        p.library = [card(n) for n in library]
        return g, p, next(c for c in C.COMBOS if 'Oracle' in c.name)

    def test_oracle_and_tainted_pact_win_off_two_islands_and_two_swamps(self):
        # Pact used to go first and pay its {1} with an Island, leaving no {U}{U} for the Oracle
        from commander_sim.cards.impl import combos as C
        g, p, cmb = self._oracle(['Sol Ring', 'Counterspell', 'Brainstorm', 'Ponder'])
        self.assertTrue(cmb.ready(g, p)[0])
        C.attempt(g, p, cmb)
        self.assertIs(g.winner, p)

    def test_tainted_pact_needs_a_library_without_duplicate_names(self):
        g, p, cmb = self._oracle(['Island', 'Island', 'Sol Ring'])
        self.assertFalse(cmb.ready(g, p)[0])

    def test_tainted_pact_is_not_a_cantrip_and_the_oracle_waits(self):
        self.assertNotIn('draw', card('Tainted Pact').tags)
        g = table('urza-mono-blue-artifacts', 'veyran'); p = g.players[0]
        self.assertEqual(pool_ai.generic_prio(g, p, card("Thassa's Oracle")), 0)      # a full library: no win


class Engines(unittest.TestCase):
    def test_breach_escapes_respect_drannith_magistrate(self):
        g = table('sauron', 'heliod-mono-white-stax'); p, o = g.players
        lands(p, 'Mountain', 4); perm(g, p, 'Underworld Breach'); perm(g, o, 'Drannith Magistrate')
        p.gy += [card('Lightning Bolt')] + [card('Island')] * 4
        bolt = p.gy[0]
        self.assertFalse(ais.breach_escape(g, p, bolt))

    def test_ad_nauseam_is_not_cast_at_the_end_of_a_turn_when_its_ai_says_no(self):
        # the end-of-turn window used to turn a "no" (None) into 3.0: cast for 0 cards at 22 life
        g = table('yawgmoth-mono-black-aristocrats', 'veyran'); p = g.players[0]; p.life = 18
        lands(p, 'Swamp', 6); hand(p, 'Ad Nauseam')
        g.active = g.players[1]
        brain.end_of_turn_window(g, p)
        self.assertIn('Ad Nauseam', [c.name for c in p.hand])

    def test_rhystic_study_is_paid_when_the_big_card_in_hand_is_out_of_reach(self):
        from commander_sim.cards.impl import rules
        g = table('chulane-bant-value-combo', 'veyran'); q = g.players[0]
        lands(q, 'Forest', 5); hand(q, 'Craterhoof Behemoth')
        g.active = g.players[1]
        self.assertTrue(rules.spare_after(g, q, 1))             # Craterhoof costs 8: it doesn't stop the payment


class Tutors(unittest.TestCase):
    def test_enlightened_tutor_finds_only_an_artifact_or_enchantment(self):
        from commander_sim.cards import dsl
        self.assertFalse(dsl.card_matches(card('Kiki-Jiki, Mirror Breaker'), {'type': 'ae'}))
        self.assertTrue(dsl.card_matches(card('Sol Ring'), {'type': 'ae'}))
        self.assertTrue(dsl.card_matches(card('Smothering Tithe'), {'type': 'ae'}))

    def test_mystical_tutor_puts_the_card_on_top(self):
        g = table('veyran', 'seph'); p = g.players[0]
        n = len(p.hand)
        E.tutor_to_top(g, p, 'is', life=0)
        self.assertEqual(len(p.hand), n)
        top = p.library[-1]
        self.assertTrue(top.instant or top.sorcery)
        self.assertEqual(p.life, 40)

    def test_gamble_waits_for_a_hand_to_hide_its_card_in(self):
        from commander_sim.cards.impl import fixes
        g = table('krenko-mono-red-goblins', 'veyran'); p = g.players[0]
        c = hand(p, 'Gamble')
        self.assertEqual(fixes.gamble_prio(g, p, c), 0)
        hand(p, 'Mountain', 'Mountain', 'Lightning Bolt')
        self.assertGreater(fixes.gamble_prio(g, p, c), 0)

    def test_natural_order_does_not_feed_a_real_creature_for_a_sidegrade(self):
        from commander_sim.cards.impl import t3
        g = table('marwyn-mono-green-elves', 'veyran'); p = g.players[0]
        perm(g, p, 'Elvish Archdruid')
        self.assertEqual(t3.natural_order_prio(g, p, card('Natural Order')), 0)

    def test_late_tutors_look_past_mana_rocks(self):
        # the priority fallback used to tutor Sol Ring on turn 10
        g = table('kaalia-mardu-creature-cheat', 'veyran'); p = g.players[0]; p.turns = 10
        self.assertNotEqual(ais.tutor_pick(g, p, 'any'), 'Sol Ring')

    def test_crop_rotation_fetches_gaeas_cradle_with_creatures_out(self):
        from commander_sim.cards.impl import fixes
        g = table('kinnan-simic-mana-combo', 'veyran'); p = g.players[0]
        lands(p, 'Forest', 3); p.library.append(card("Gaea's Cradle"))
        c = card('Crop Rotation')
        self.assertEqual(fixes.crop_prio(g, p, c), 0)
        for _ in range(4): perm(g, p, 'Llanowar Elves')
        self.assertGreater(fixes.crop_prio(g, p, c), 0)
        fixes._crop_rotation(g, p, c, {})
        self.assertIn("Gaea's Cradle", [L.cd.name for L in p.lands])


class Counters(unittest.TestCase):
    def test_fierce_guardianship_is_free_with_your_commander_out(self):
        g = table('veyran', 'seph'); p = g.players[0]
        fg = card('Fierce Guardianship')
        self.assertFalse(E.free_counter(p, fg))
        perm(g, p, p.cmd.name).is_cmd = True; p.cmd_in_zone = False
        self.assertTrue(E.free_counter(p, fg))


class Priorities(unittest.TestCase):
    def test_citadel_wants_life_above_what_the_table_can_hit_for(self):
        g = table('yshtola', 'veyran'); p = g.players[0]
        c = card("Bolas's Citadel")
        self.assertGreater(gc_prio.citadel_prio(g, p, c), 0)
        for _ in range(4): perm(g, g.players[1], 'Grave Titan')
        self.assertEqual(gc_prio.citadel_prio(g, p, c), 0)


if __name__ == '__main__':
    unittest.main()
