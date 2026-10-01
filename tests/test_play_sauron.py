"""Practice mode, Sauron's cards: the choices the AI used to make for them are yours. Damage triggers (Orcish
Bowmasters, Kaervek, Sword of Fire and Ice) ask for a target; the Ring asks for its Ring-bearer, Call of the Ring
and Sauron ask whether to use their "may" abilities."""
import unittest
from tests.table import table, hand, perm
from commander_sim import engine as E, ais
from commander_sim.play import legal
from commander_sim.play.controller import ScriptController


def mine():
    from commander_sim.cards.impl import mine as m        # imported after setup (an early import changes card data)
    return m


def seat(g, p, answers):
    g.controllers = {p.key: ScriptController(answers)}
    return g.controllers[p.key]


def pick(g, p, x):
    """an answer choosing x, whatever position it is listed in"""
    return lambda req: req.choices.index(legal.describe_target(g, p, x))


class Damage(unittest.TestCase):
    def test_bowmasters_enters(self):
        g = table('sauron', 'veyran'); s, v = g.players
        ctl = seat(g, s, pick(g, s, v))
        perm(g, s, 'Orcish Bowmasters')
        self.assertEqual(v.life, 39); self.assertIsNotNone(E.army_of(s))
        self.assertEqual(ctl.asked[0].kind, 'target')

    def test_bowmasters_on_extra_draws(self):
        g = table('sauron', 'veyran'); s, v = g.players
        perm(g, s, 'Orcish Bowmasters')
        g.controllers = {}
        ctl = seat(g, s, pick(g, s, v))
        E.draw(g, v, 2)                                    # two cards, neither is the draw step's first
        self.assertEqual(len(ctl.asked), 2)

    def test_a_creature_survives_damage_below_its_toughness(self):
        g = table('sauron', 'veyran'); s, v = g.players
        big = perm(g, v, 'Grave Titan')
        seat(g, s, pick(g, s, big))
        mine().kaervek(g, s, v, E.DB['Counterspell'])         # mana value 2 against a 6/6
        self.assertIn(big, v.perms)

    def test_kaervek_kills_what_it_reaches(self):
        g = table('sauron', 'veyran'); s, v = g.players
        gs = perm(g, v, 'Guttersnipe')
        seat(g, s, pick(g, s, gs))
        mine().kaervek(g, s, v, E.DB['Counterspell'])
        self.assertNotIn(gs, v.perms)


class Ring(unittest.TestCase):
    def test_tempt(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        perm(g, s, 'Call of the Ring'); perm(g, s, 'Sauron, the Dark Lord')
        bow = perm(g, s, 'Grave Titan')
        hand(s, 'Island')
        seat(g, s, lambda req: req.choices.index(legal.describe_target(g, s, bow)) if req.prompt.startswith('The Ring')
                   else {'Call': 0, 'Saur': 1}[req.prompt[:4]])     # pay 2 for a card: yes; Sauron's discard: no
        mine().ring_tempt(g, s)
        self.assertIs(s.ring_bearer, bow)
        self.assertEqual(s.life, 38); self.assertEqual(len(s.hand), 2)   # paid 2, drew 1, kept the hand


if __name__ == '__main__':
    unittest.main()
