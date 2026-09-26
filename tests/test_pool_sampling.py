"""Pool sampling: opponents drawn without replacement, seeded, and paired across two versions of a deck."""
import collections, os, sys, unittest
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from commander_sim import pools, ais, poolmode
from commander_sim.decks import DECKS

KEYS = ['a', 'b', 'c', 'd', 'e']


class DrawSeats(unittest.TestCase):
    def test_without_replacement(self):
        for s in range(500):
            seats = pools.draw_seats(s, KEYS, 'me')
            self.assertEqual(len(seats), 4)
            self.assertEqual(len(set(seats)), 4)
            self.assertIn('me', seats)
            self.assertTrue(set(seats) - {'me'} <= set(KEYS))

    def test_four_from_pool(self):
        for s in range(200):
            seats = pools.draw_seats(s, KEYS, None, k=4)
            self.assertEqual(len(set(seats)), 4)
            self.assertTrue(set(seats) <= set(KEYS))

    def test_seeded(self):
        self.assertEqual([pools.draw_seats(s, KEYS, 'me') for s in range(50)],
                         [pools.draw_seats(s, KEYS, 'me') for s in range(50)])
        self.assertGreater(len({tuple(pools.draw_seats(s, KEYS, 'me')) for s in range(50)}), 30)

    def test_pool_order_does_not_matter(self):
        for s in range(50):
            self.assertEqual(pools.draw_seats(s, KEYS, 'me'), pools.draw_seats(s, list(reversed(KEYS)), 'me'))

    def test_roughly_uniform(self):
        n = 20000
        seated = collections.Counter(); pos = collections.Counter()
        for s in range(n):
            seats = pools.draw_seats(s, KEYS, 'me')
            seated.update(k for k in seats if k != 'me'); pos[seats.index('me')] += 1
        for k in KEYS: self.assertAlmostEqual(seated[k] / n, 0.6, delta=0.02)
        for i in range(4): self.assertAlmostEqual(pos[i] / n, 0.25, delta=0.02)


class Paired(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        pools.register()
        cls.tier = poolmode.pool_keys('t2')

    def opening(self, seed, my_cards):
        seats = pools.draw_seats(seed, self.tier, 'seph')
        g = ais.setup_pool_game(seed, [poolmode.seat_spec(k, my_cards if k == 'seph' else None) for k in seats])
        return {p.key: ([c.name for c in p.hand], [c.name for c in p.library]) for p in g.players}, seats

    def test_opponents_identical_across_deck_versions(self):
        base = list(DECKS['seph'])
        var = list(base); var.remove('Blood Artist'); var.append('Grim Tutor')
        for seed in range(500000, 500040):
            ob, sb = self.opening(seed, base)
            ov, sv = self.opening(seed, var)
            self.assertEqual(sb, sv)                              # same opponents, same seats
            for k in sb:
                if k != 'seph': self.assertEqual(ob[k], ov[k])    # same opening hand and library order

    def test_my_shuffle_is_common_random_numbers(self):
        """same permutation of my deck: the swapped-in card sits where the swapped-out card was"""
        base = list(DECKS['seph'])
        var = [('Grim Tutor' if c == 'Blood Artist' else c) for c in base]
        same = 0
        for seed in range(500000, 500040):
            ob, _ = self.opening(seed, base); ov, _ = self.opening(seed, var)
            swap = lambda xs: ['Grim Tutor' if c == 'Blood Artist' else c for c in xs]
            if (swap(ob['seph'][0]), swap(ob['seph'][1])) == ov['seph']: same += 1
        self.assertGreaterEqual(same, 30)                   # differs only when the mulligan decision changes

    def test_game_reproducible(self):
        for seed in (500000, 500001):
            a = poolmode.play(seed, 'seph', None, self.tier); b = poolmode.play(seed, 'seph', None, self.tier)
            self.assertEqual((a.winner.key if a.winner else None, a.round, [p.life for p in a.players]),
                             (b.winner.key if b.winner else None, b.round, [p.life for p in b.players]))


if __name__ == '__main__':
    unittest.main()
