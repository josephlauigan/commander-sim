"""Lands that enter tapped unless a condition holds, by their Oracle text: check lands, fast lands, slow lands,
battle lands, bond lands, Mystic Sanctuary, snarls and Barad-dûr."""
import unittest
from tests.table import table, lands, hand, card
from commander_sim import ais


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
        g = table('sauron', 'veyran', 'seph', 'marchesa'); s = g.players[0]
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


if __name__ == '__main__':
    unittest.main()
