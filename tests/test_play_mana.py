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



class WhereSourcesAre(unittest.TestCase):
    def test_sources_name_their_land_permanent_or_treasure(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        lands(s, 'Island', 2); rock = perm(g, s, 'Sol Ring'); s.treasures = 1
        srcs = mana.sources(g, s)
        self.assertEqual(sorted(x['land'] for x in srcs if 'land' in x), [0, 1])
        self.assertEqual([x['perm'] for x in srcs if 'perm' in x], [s.perms.index(rock)])
        self.assertEqual(sum(1 for x in srcs if x.get('treasure')), 1)


class Phyrexian(unittest.TestCase):
    """{B/P}: pay its colour or 2 life, your choice"""
    def cast_vraska(self, answers, **pool):
        from commander_sim.play import human
        from commander_sim.play.controller import ScriptController
        from tests.table import hand
        g = table('sauron', 'veyran'); s = g.players[0]
        hand(s, "Vraska, Betrayal's Sting")
        for c, n in pool.items(): mana.pool_of(s).add(c, n)
        g.controllers = {s.key: ScriptController([{'do': 'cast', 'card': 0}] + answers + [{'do': 'pass'}])}
        human.human_main(g, s, False)
        return g, s, g.controllers[s.key]

    def test_pay_with_life(self):
        g, s, ctl = self.cast_vraska([1], B=2, C=4)                 # {4}{B}{B/P}: B B and four more: both ways work
        self.assertEqual(ctl.asked[1].choices[:2], ['{B}', '2 life'])
        self.assertTrue(any(m.name == "Vraska, Betrayal's Sting" for m in s.perms))
        self.assertEqual((s.life, mana.pool_of(s).total()), (38, 1))   # the spare {B} is left over

    def test_pay_with_mana(self):
        g, s, ctl = self.cast_vraska([0], B=2, C=4)
        self.assertEqual((s.life, mana.pool_of(s).total()), (40, 0))

    def test_only_life_works(self):
        g, s, ctl = self.cast_vraska([], B=1, C=4)                  # one {B}: the Phyrexian symbol must be life
        self.assertTrue(any(m.name == "Vraska, Betrayal's Sting" for m in s.perms))
        self.assertEqual(s.life, 38)

    def test_rules_check_counts_life(self):
        from commander_sim.play import legal
        from tests.table import hand
        g = table('sauron', 'veyran'); s = g.players[0]
        c = hand(s, "Vraska, Betrayal's Sting")
        mana.pool_of(s).add('B', 1); mana.pool_of(s).add('C', 4)
        self.assertIsNone(legal.check_cast(g, s, c))
        s.life = 1
        self.assertIn('2 life each', legal.check_cast(g, s, c))

if __name__ == '__main__':
    unittest.main()
