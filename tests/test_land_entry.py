"""Lands that enter tapped unless a condition holds, by their Oracle text: check lands, fast lands, slow lands,
battle lands, bond lands, Mystic Sanctuary, snarls and Barad-dûr; and lands' enters triggers (scry, gain life)."""
import unittest
from tests.table import table, lands, hand, card
from commander_sim import ais, engine as E


def tapped(p, name):
    return ais.land_enters_tapped(p, card(name))


class LandEntry(unittest.TestCase):
    def test_check_land(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        lands(s, 'Mountain')
        self.assertTrue(tapped(s, 'Drowned Catacomb'))                 # needs an Island or a Swamp
        lands(s, 'Island')
        self.assertFalse(tapped(s, 'Drowned Catacomb'))
        lands(s, 'Swamp', 3)
        self.assertFalse(tapped(s, 'Drowned Catacomb'))                # however many lands
        self.assertTrue(tapped(s, 'Sunpetal Grove'))                   # Forest or Plains: none

    def test_check_land_counts_dual_types(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        lands(s, 'Steam Vents')                                         # Land — Island Mountain
        self.assertFalse(tapped(s, 'Drowned Catacomb'))

    def test_fast_land(self):
        g = table('veyran', 'sauron'); v = g.players[0]
        lands(v, 'Island', 2)
        self.assertFalse(tapped(v, 'Spirebluff Canal'))                 # two or fewer other lands
        lands(v, 'Island')
        self.assertTrue(tapped(v, 'Spirebluff Canal'))

    def test_slow_land(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        lands(s, 'Swamp')
        self.assertTrue(tapped(s, 'Haunted Ridge'))                     # two or more other lands
        lands(s, 'Mountain')
        self.assertFalse(tapped(s, 'Haunted Ridge'))

    def test_snarl_reveals_from_hand(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        self.assertTrue(tapped(s, 'Foreboding Ruins'))
        hand(s, 'Swamp')
        self.assertFalse(tapped(s, 'Foreboding Ruins'))

    def test_battle_bond_and_sanctuary(self):
        g = table('sauron', 'veyran', 'seph', 'jodah'); s = g.players[0]
        self.assertFalse(tapped(s, 'Luxury Suite'))                     # three opponents
        lands(s, 'Island')
        self.assertTrue(tapped(s, 'Sunken Hollow'))                     # two or more basic lands
        lands(s, 'Swamp')
        self.assertFalse(tapped(s, 'Sunken Hollow'))
        self.assertTrue(tapped(s, 'Mystic Sanctuary'))                  # three or more other Islands
        lands(s, 'Island', 2)
        self.assertFalse(tapped(s, 'Mystic Sanctuary'))

    def test_plain_tapped_and_untapped_lands(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        self.assertTrue(tapped(s, 'Crumbling Necropolis'))
        self.assertFalse(tapped(s, 'Command Tower'))


class ShockLands(unittest.TestCase):
    """'As this land enters, you may pay 2 life. If you don't, it enters tapped' (Steam Vents and the other shocks)"""
    def test_you_choose_to_pay(self):
        from commander_sim.play import human
        from commander_sim.play.controller import ScriptController
        for answer, paid in ((0, True), (1, False)):                 # 0 = yes (pay 2), 1 = no (tapped)
            g = table('sauron', 'veyran'); s = g.players[0]
            life = s.life
            s.hand.append(card('Steam Vents'))
            ctl = ScriptController(lambda req, a=answer: a); g.controllers = {s.key: ctl}
            self.assertIsNone(human.apply(g, s, {'do': 'land', 'card': len(s.hand) - 1}))
            self.assertTrue(any('pay 2 life' in r.prompt for r in ctl.asked))
            self.assertEqual(s.lands[-1].tapped, not paid)
            self.assertEqual(s.life, life - 2 if paid else life)

    def test_the_ai_pays_only_when_it_uses_the_mana(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        s.turns = 6; lands(s, 'Island', 1)
        s.hand.append(card('Counterspell'))                          # two mana: needs the shock untapped
        life = s.life
        ais.play_land_card(g, s, card('Steam Vents'))
        self.assertFalse(s.lands[-1].tapped)
        self.assertEqual(s.life, life - 2)
        g = table('sauron', 'veyran'); s = g.players[0]
        s.turns = 6; s.hand = []                                     # nothing to cast: save the life
        ais.play_land_card(g, s, card('Steam Vents'))
        self.assertTrue(s.lands[-1].tapped)
        self.assertEqual(s.life, 40)


if __name__ == '__main__':
    unittest.main()


class LandEntersTriggers(unittest.TestCase):
    """'When this land enters' effects read from the Oracle text: a Temple's scry, a gain land's life"""
    def test_you_scry_when_your_temple_enters(self):
        from tests.table import table, card
        from commander_sim.play import human
        from commander_sim.play.controller import ScriptController
        g = table('yshtola', 'veyran'); z = g.players[0]
        z.hand.append(card('Temple of Enlightenment'))
        ctl = ScriptController(lambda req: 0); g.controllers = {z.key: ctl}
        self.assertIsNone(human.apply(g, z, {'do': 'land', 'card': len(z.hand) - 1}))
        self.assertTrue(any(r.prompt.startswith('Scry') for r in ctl.asked))

    def test_a_gain_land_gains_life_and_each_land_only_once(self):
        from tests.table import table, card
        g = table('yshtola', 'veyran'); z = g.players[0]
        life = z.life
        ais.play_land_card(g, z, card('Scoured Barrens'))
        E.check_state(g); E.landfall(g, z)
        self.assertEqual(z.life, life + 1)
