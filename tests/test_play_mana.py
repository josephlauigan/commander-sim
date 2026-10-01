"""Practice mode: the human seat's mana pool. Tapping a source adds its mana (with the source's side effects),
costs are paid from the pool, and the pool empties between steps."""
import unittest
from tests.table import table, perm, lands
from commander_sim import engine as E, ais
from commander_sim.play import mana
from commander_sim.play.controller import HumanController


def src(g, p, name):
    return next(x['id'] for x in mana.sources(g, p) if x['name'] == name)


class Pool(unittest.TestCase):
    def test_colours_and_generic(self):
        pool = mana.ManaPool(); pool.add('B', 2)
        self.assertFalse(pool.can_pay(2, 'BB'))
        self.assertEqual(pool.missing(2, 'BB'), '{2}')
        pool.add('C', 2)
        self.assertTrue(pool.pay(2, 'BB')); self.assertEqual(pool.total(), 0)

    def test_generic_spends_colourless_before_colours(self):
        pool = mana.ManaPool(); pool.add('C', 1); pool.add('U', 1); pool.add('B', 1)
        self.assertTrue(pool.pay(1, 'B'))
        self.assertEqual(pool.text(), '{U}')

    def test_any_colour_mana_pays_a_pip(self):
        pool = mana.ManaPool(); pool.add('A', 1)
        self.assertTrue(pool.pay(0, 'U'))


class Tapping(unittest.TestCase):
    def test_a_land_taps_into_the_pool(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        lands(s, 'Swamp', 2)
        self.assertIsNone(mana.tap(g, s, src(g, s, 'Swamp')))
        self.assertEqual(mana.pool_of(s).text(), '{B}')
        self.assertEqual(sum(1 for L in s.lands if L.tapped), 1)
        self.assertEqual(len(mana.sources(g, s)), 1)

    def test_wrong_colour_is_refused(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        lands(s, 'Swamp', 1)
        self.assertIn("can't make blue", mana.tap(g, s, src(g, s, 'Swamp'), 'U'))
        self.assertFalse(s.lands[0].tapped)

    def test_talisman_hurts_only_for_colour(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        perm(g, s, 'Talisman of Dominance')
        self.assertIsNone(mana.tap(g, s, src(g, s, 'Talisman of Dominance'), 'C'))
        self.assertEqual(s.life, 40)
        g = table('sauron', 'veyran'); s = g.players[0]
        perm(g, s, 'Talisman of Dominance')
        mana.tap(g, s, src(g, s, 'Talisman of Dominance'), 'U')
        self.assertEqual(s.life, 39)

    def test_sol_ring_and_treasure(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        perm(g, s, 'Sol Ring'); s.treasures = 1
        mana.tap(g, s, src(g, s, 'Sol Ring'))
        mana.tap(g, s, src(g, s, 'Treasure'), 'R')
        self.assertEqual(s.treasures, 0)
        self.assertEqual(mana.pool_of(s).text(), '{R}{C}{C}')

    def test_paying_from_the_pool_and_the_reason_when_short(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        lands(s, 'Swamp', 2)
        for _ in range(2): mana.tap(g, s, mana.sources(g, s)[0]['id'])
        why = mana.pay_from_pool(g, s, 2, 'BB')
        self.assertEqual(why, 'It costs {2}{B}{B}; your mana pool has {B}{B}. Missing {2}.')
        self.assertIsNone(mana.pay_from_pool(g, s, 0, 'BB'))

    def test_ritual_mana_joins_the_pool(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        s.floatA = 3                                   # Dark Ritual's mana, as the engine adds it
        self.assertIsNone(mana.pay_from_pool(g, s, 2, 'B'))
        self.assertEqual(s.floatA, 0)

    def test_the_pool_empties_between_steps(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        g.controllers = {'sauron': HumanController()}
        mana.pool_of(s).add('B', 2)
        ais.continue_turn(g, s, 'end')
        self.assertEqual(mana.pool_of(s).total(), 0)


if __name__ == '__main__':
    unittest.main()
