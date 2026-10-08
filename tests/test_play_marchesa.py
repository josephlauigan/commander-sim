"""Practice mode, cards first written for the Marchesa deck (removed 2026-10-07) that other lists still run, and the
sacrifice outlets your decks share: Accursed Marauder's sacrifice, Mystic Remora's upkeep, dredge, Coalition Relic,
and free sacrifice outlets as abilities."""
import unittest
from tests.table import table, hand, perm
from commander_sim import engine as E
from commander_sim.play import mana, human
from commander_sim.play.controller import ScriptController


def seat(g, p, answers):
    g.controllers = {p.key: ScriptController(answers)}
    return g.controllers[p.key]


def by_text(word):
    return lambda req: next(i for i, x in enumerate(req.choices) if word in x)


def pool(p, **kw):
    for c, n in kw.items(): mana.pool_of(p).add(c, n)


def main(g, p, *answers):
    ctl = seat(g, p, list(answers) + [{'do': 'pass'}])
    human.human_main(g, p, False)
    return ctl


class Spells(unittest.TestCase):
    def test_accursed_marauder_you_choose_your_sacrifice(self):
        g = table('jodah', 'veyran'); s, v = g.players
        hand(s, 'Accursed Marauder'); keep = perm(g, s, 'Hellkite Tyrant'); perm(g, v, 'Guttersnipe'); pool(s, B=1, C=1)
        main(g, s, {'do': 'cast', 'card': 0}, by_text('Accursed'))
        self.assertIn(keep, s.perms); self.assertFalse(any(m.name == 'Guttersnipe' for m in v.perms))


class Triggers(unittest.TestCase):
    def test_mystic_remora_upkeep_declined(self):
        g = table('yshtola', 'veyran'); s = g.players[0]
        rm = perm(g, s, 'Mystic Remora'); pool(s, C=1)
        seat(g, s, [1])                                             # "pay {1}?" no
        E.CI.HOOKS['Mystic Remora']['upkeep'](g, rm, s)
        self.assertNotIn(rm, s.perms)

    def test_dredge_or_draw(self):
        from commander_sim.play import cards
        g = table('seph', 'veyran'); s = g.players[0]
        s.gy.append(E.DB['Stinkweed Imp'])
        seat(g, s, [0])                                             # yes
        self.assertTrue(cards.dredge(g, s))
        self.assertTrue(any(c.name == 'Stinkweed Imp' for c in s.hand)); self.assertEqual(len(s.gy), 5)


class Outlets(unittest.TestCase):
    def test_carrion_feeder(self):
        g = table('seph', 'veyran'); s = g.players[0]
        f = perm(g, s, 'Carrion Feeder'); r = perm(g, s, 'Burglar Rat')
        main(g, s, {'do': 'use', 'perm': 0}, 0, by_text('Burglar'))
        self.assertNotIn(r, s.perms); self.assertEqual(f.plus, 1)

    def test_ashnods_altar_makes_colourless(self):
        g = table('seph', 'veyran'); s = g.players[0]
        perm(g, s, "Ashnod's Altar"); perm(g, s, 'Grave Titan')
        main(g, s, {'do': 'use', 'perm': 0}, 0, by_text('Grave Titan'))
        self.assertEqual(mana.pool_of(s).m['C'], 2)
        self.assertTrue(any(c.name == 'Grave Titan' for c in s.gy))

    def test_coalition_relic_charge(self):
        g = table('jodah', 'veyran'); s = g.players[0]
        cr = perm(g, s, 'Coalition Relic')
        main(g, s, {'do': 'use', 'perm': 0}, 0)
        self.assertEqual((cr.data['charge'], cr.tapped), (1, True))


if __name__ == '__main__':
    unittest.main()
