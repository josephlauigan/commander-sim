"""Practice mode, Sephiroth's (and Marchesa's) graveyard cards: reanimation targets are chosen as the spell is cast,
from any graveyard where the card allows; Entomb, Buried Alive and Grisly Salvage let you pick; Deadly Dispute asks
what to sacrifice."""
import unittest
from tests.table import table, hand, perm
from commander_sim import engine as E
from commander_sim.play import legal, mana
from commander_sim.play.controller import ScriptController


def seat(g, p, answers):
    g.controllers = {p.key: ScriptController(answers)}
    return g.controllers[p.key]


def float_mana(p, **kw):
    for c, n in kw.items(): mana.pool_of(p).add(c, n)


def by_text(word):
    return lambda req: next(i for i, x in enumerate(req.choices) if word in x)


class Reanimation(unittest.TestCase):
    def test_animate_dead_from_an_opponents_graveyard(self):
        from commander_sim.play import human
        g = table('seph', 'veyran'); s, v = g.players
        hand(s, 'Animate Dead'); v.gy.append(E.DB['Guttersnipe']); s.gy.append(E.DB['Grave Titan'])
        float_mana(s, C=1, B=1)
        ctl = seat(g, s, [{'do': 'cast', 'card': 0}, by_text('Guttersnipe'), {'do': 'pass'}])
        human.human_main(g, s, False)
        self.assertTrue(any(m.name == 'Guttersnipe' for m in s.perms))
        self.assertIn("graveyard", ctl.asked[1].choices[0])

    def test_unburial_rites_only_from_your_graveyard(self):
        from commander_sim.play import choices
        g = table('seph', 'veyran'); s, v = g.players
        v.gy.append(E.DB['Guttersnipe'])
        self.assertEqual(choices.rean_candidates(g, s, 'rites'), [])
        self.assertIn('no creature card', legal.check_cast(g, s, hand(s, 'Unburial Rites')))

    def test_entomb(self):
        from commander_sim.play import choices
        g = table('seph', 'veyran'); s = g.players[0]
        seat(g, s, [by_text('Grave Titan')])
        choices.fill(g, s, 'entomb')
        self.assertEqual([c.name for c in s.gy], ['Grave Titan'])

    def test_grisly_salvage(self):
        from commander_sim.play import choices
        g = table('seph', 'veyran'); s = g.players[0]
        n = len(s.hand)
        seat(g, s, ['cancel'])
        choices.fill(g, s, 'grisly')
        self.assertEqual((len(s.hand), len(s.gy)), (n, 5))


class Dispute(unittest.TestCase):
    def test_deadly_dispute_sacrifices_your_pick(self):
        from commander_sim.play import human
        g = table('marchesa', 'veyran'); s = g.players[0]
        hand(s, 'Deadly Dispute'); a = perm(g, s, 'Sol Ring'); float_mana(s, C=1, B=1)
        seat(g, s, [{'do': 'cast', 'card': 0}, 0, {'do': 'pass'}])
        human.human_main(g, s, False)
        self.assertNotIn(a, s.perms); self.assertEqual(s.treasures, 1)


if __name__ == '__main__':
    unittest.main()
