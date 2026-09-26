"""The look-ahead AI's building blocks: game copies, hidden information, position scoring, and a whole decision."""
import random, unittest
from tests.table import table, hand, lands, perm, token
from commander_sim import engine as E, compare
from commander_sim.ai import brain, search


def fingerprint(g):
    return [(p.key, p.life, [c.name for c in p.hand], [c.name for c in p.library], [m.name for m in p.perms],
             [(L.cd.name, L.tapped) for L in p.lands], p.treasures) for p in g.players]


class Clone(unittest.TestCase):
    def test_a_copy_is_independent(self):
        g = table('seph', 'veyran'); s, v = g.players
        perm(g, s, 'Grave Titan'); hand(v, 'Counterspell', 'Lightning Bolt')
        before = fingerprint(g)
        g2 = search.clone(g); s2, v2 = g2.players
        s2.perms.clear(); v2.life = 1; v2.hand.clear(); s2.library.pop()
        self.assertEqual(fingerprint(g), before)

    def test_card_definitions_are_shared_not_copied(self):
        g = table('seph', 'veyran')
        g2 = search.clone(g)
        self.assertIs(g2.players[0].library[0], g.players[0].library[0])
        self.assertIsNot(g2.players[0], g.players[0])


class Determinize(unittest.TestCase):
    def test_opponents_hands_are_redealt_from_hand_and_library(self):
        g = table('seph', 'veyran'); s, v = g.players
        hand(s, 'Entomb'); hand(v, 'Counterspell', 'Lightning Bolt')
        g2 = search.clone(g); s2, v2 = g2.players
        cards = sorted(c.name for c in v2.hand + v2.library)
        search.determinize(g2, s2, random.Random(3))
        self.assertEqual(sorted(c.name for c in v2.hand + v2.library), cards)   # same cards
        self.assertEqual(len(v2.hand), 2)                                          # same hand size
        self.assertEqual([c.name for c in s2.hand], ['Entomb'])                    # my own hand is known


class Evaluate(unittest.TestCase):
    def test_a_win_beats_any_board(self):
        g = table('sauron', 'veyran'); r = g.players[0]
        for _ in range(60): token(g, r, 5)
        big = search.evaluate(g, r)
        self.assertLess(big, 95.0)
        g.over, g.winner = True, r
        self.assertEqual(search.evaluate(g, r), 100.0)
        g.winner = g.players[1]
        self.assertEqual(search.evaluate(g, r), -100.0)

    def test_more_is_better(self):
        g = table('sauron', 'veyran'); r = g.players[0]
        a = search.evaluate(g, r)
        perm(g, r, 'Hellkite Tyrant')
        self.assertGreater(search.evaluate(g, r), a)


class Decision(unittest.TestCase):
    def setUp(self):
        compare.set_ai('lookahead', 1.0)

    def tearDown(self):
        compare.set_ai('adaptive', 1.0)

    def test_choose_picks_an_option_and_leaves_the_game_alone(self):
        g = table('veyran', 'sauron', seed=7); v = g.players[0]
        lands(v, 'Island', 2); lands(v, 'Mountain', 2)
        hand(v, 'Guttersnipe', 'Think Twice', 'Lightning Bolt')
        perm(g, g.players[1], "Jace's Archivist")
        opts = brain.main_options(g, v, False)
        before = fingerprint(g)
        pick = search.choose(g, v, False, opts)
        self.assertIn(pick, opts)
        self.assertEqual(fingerprint(g), before)
        self.assertIs(E.CUR_G, g)

    def test_decisions_are_reproducible(self):
        picks = []
        for _ in range(2):
            g = table('veyran', 'sauron', seed=7); v = g.players[0]
            lands(v, 'Island', 2); lands(v, 'Mountain', 2)
            hand(v, 'Guttersnipe', 'Think Twice', 'Lightning Bolt')
            picks.append(search.choose(g, v, False, brain.main_options(g, v, False))[1])
        self.assertEqual(picks[0], picks[1])


if __name__ == '__main__':
    unittest.main()
