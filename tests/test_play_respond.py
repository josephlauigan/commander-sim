"""Practice mode: priority on other players' turns. Every spell an opponent casts gives you priority: counter it,
cast an instant, or pass. When an opponent counters your spell you may counter back. You also get priority at the
end of each other turn and when attacked."""
import unittest
from tests.table import table, hand, lands, perm
from commander_sim import engine as E, ais
from commander_sim.play import mana, legal
from commander_sim.play.controller import ScriptController


def seat(g, p, answers):
    g.controllers = {p.key: ScriptController(answers)}
    return g.controllers[p.key]


def tap_all(g, p):
    while mana.sources(g, p): mana.tap(g, p, mana.sources(g, p)[0]['id'])


def opp_casts(g, q, name):
    """opponent q casts `name` from hand, paid automatically"""
    c = hand(q, name)
    E.pay(g, q, *E.cost_of(q, c))
    return E.cast_card(g, q, c, 'hand', {})


class Responding(unittest.TestCase):
    def test_counter_an_opponents_spell(self):
        g = table('veyran', 'sauron'); v, s = g.players
        hand(s, 'Counterspell'); lands(s, 'Island', 2); tap_all(g, s)
        lands(v, 'Mountain', 1); lands(v, 'Island', 1); lands(v, 'Island', 1); lands(v, 'Mountain', 1)
        ctl = seat(g, s, [{'do': 'cast', 'card': 0}])
        self.assertFalse(opp_casts(g, v, 'Guttersnipe'))
        self.assertFalse(any(m.name == 'Guttersnipe' for m in v.perms))
        self.assertIn(E.DB['Counterspell'], s.gy)
        self.assertEqual(ctl.asked[0].data['stack'], [{'name': 'Guttersnipe'}])

    def test_pass_and_it_resolves(self):
        g = table('veyran', 'sauron'); v, s = g.players
        lands(v, 'Mountain', 3)
        seat(g, s, [{'do': 'pass'}])
        self.assertTrue(opp_casts(g, v, 'Guttersnipe'))

    def test_priority_on_every_spell(self):
        g = table('veyran', 'sauron'); v, s = g.players
        lands(v, 'Island', 1)
        ctl = seat(g, s, [{'do': 'pass'}])
        opp_casts(g, v, 'Sol Ring')                      # a spell the AI would never bother to counter
        self.assertEqual(len(ctl.asked), 1)

    def test_tap_then_counter(self):
        g = table('veyran', 'sauron'); v, s = g.players
        hand(s, 'Counterspell'); lands(s, 'Island', 2); lands(v, 'Mountain', 3)
        seat(g, s, [{'do': 'tap', 'source': 0}, {'do': 'tap', 'source': 0}, {'do': 'cast', 'card': 0}])
        self.assertFalse(opp_casts(g, v, 'Guttersnipe'))

    def test_wrong_kind_of_counterspell(self):
        g = table('veyran', 'sauron'); v, s = g.players
        fg = hand(s, 'Fierce Guardianship'); lands(s, 'Island', 3); tap_all(g, s); lands(v, 'Mountain', 3)
        self.assertEqual(legal.check_counter(g, s, fg, E.DB['Guttersnipe']),
                         "Fierce Guardianship can't counter Guttersnipe.")      # noncreature spells only
        ctl = seat(g, s, [{'do': 'cast', 'card': 0}, {'do': 'pass'}])
        self.assertTrue(opp_casts(g, v, 'Guttersnipe'))
        self.assertEqual(ctl.told[0][0], 'invalid')

    def test_no_spell_to_counter(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        c = hand(s, 'Counterspell'); lands(s, 'Island', 2); tap_all(g, s)
        self.assertIn('counters a spell', legal.check_cast(g, s, c))

    def test_counter_back_when_the_ai_counters_you(self):
        g = table('sauron', 'veyran'); s, v = g.players
        sheo = hand(s, 'Sheoldred, the Apocalypse')
        hand(s, 'Counterspell'); lands(s, 'Island', 2); tap_all(g, s)
        ctr = hand(v, 'Counterspell'); lands(v, 'Island', 2)
        seat(g, s, [{'do': 'cast', 'card': 1}])            # hand: Sheoldred, Counterspell
        self.assertFalse(E._counter_resolves(g, v, s, sheo, ctr, 7))
        self.assertEqual(s.stats['counterwar_won'], 1)

    def test_priority_when_attacked(self):
        g = table('veyran', 'sauron'); v, s = g.players
        perm(g, s, 'Grave Titan').sick = False
        a = perm(g, v, 'Guttersnipe'); a.sick = False
        ctl = seat(g, s, [{'do': 'pass'}, 'cancel'])
        ais.resolve_combat(g, v, [a], s, set())
        self.assertEqual([r.kind for r in ctl.asked], ['priority', 'block'])
        self.assertIn('attacks you', ctl.asked[0].prompt)


if __name__ == '__main__':
    unittest.main()
