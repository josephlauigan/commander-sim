"""Pool games run end to end: a few seeded games per tier, with and without one of the main decks."""
import os, sys, unittest
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import engine, compare, pools, poolmode


class PoolGames(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        pools.register(); compare.set_ai('adaptive', 1.0); engine.set_profile('conservative')

    def test_every_tier_plays(self):
        for tier in pools.TIERS:
            keys = poolmode.pool_keys(tier)
            for seed in range(500000, 500004):
                with self.subTest(tier=tier, seed=seed, me=None):
                    g = poolmode.play(seed, None, None, keys, 4)
                    self.assertTrue(g.over or g.wintype == 'timeout')
                    self.assertEqual(len({p.key for p in g.players}), 4)
                with self.subTest(tier=tier, seed=seed, me='veyran'):
                    g = poolmode.play(seed, 'veyran', None, keys, 3)
                    self.assertIn('veyran', [p.key for p in g.players])

    def test_old_mode_flag_restored(self):
        import ais
        from decks import DECKS
        poolmode.play(500000, 'seph', None, poolmode.pool_keys('t1'))
        self.assertTrue(engine.POOL_RULES)
        ais.play_game(500000, DECKS)
        self.assertFalse(engine.POOL_RULES)


if __name__ == '__main__':
    unittest.main()
