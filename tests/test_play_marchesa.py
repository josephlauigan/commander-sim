"""Practice mode, Marchesa's cards (and the sacrifice outlets your decks share): Act of Treason's target is chosen as
it's cast, enter and dies triggers ask for their targets, Crux of Fate's mode, Accursed Marauder's sacrifice, Mystic
Remora's upkeep, dredge, and free sacrifice outlets as abilities."""
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
    def test_act_of_treason_your_target(self):
        g = table('marchesa', 'veyran'); s, v = g.players
        hand(s, 'Act of Treason'); a = perm(g, v, 'Guttersnipe'); perm(g, v, 'Archmage Emeritus'); pool(s, R=1, C=2)
        main(g, s, {'do': 'cast', 'card': 0}, by_text('Guttersnipe'))
        self.assertIn(a, s.perms)

    def test_enslave_your_target(self):
        g = table('marchesa', 'veyran'); s, v = g.players
        hand(s, 'Enslave'); a = perm(g, v, 'Guttersnipe'); perm(g, v, 'Archmage Emeritus'); pool(s, B=2, C=4)
        main(g, s, {'do': 'cast', 'card': 0}, by_text('Guttersnipe'))
        self.assertIn(a, s.perms)

    def test_crux_of_fate_mode(self):
        g = table('marchesa', 'veyran'); s, v = g.players
        hand(s, 'Crux of Fate'); a = perm(g, v, 'Guttersnipe'); pool(s, B=2, C=3)
        main(g, s, {'do': 'cast', 'card': 0}, by_text('all Dragons'))
        self.assertIn(a, v.perms)                                  # you chose Dragons: Guttersnipe lives

    def test_forge_devil_your_target(self):
        g = table('marchesa', 'veyran'); s, v = g.players
        hand(s, 'Forge Devil'); a = perm(g, v, 'Harmonic Prodigy'); pool(s, R=1)
        main(g, s, {'do': 'cast', 'card': 0}, by_text('Harmonic'))
        self.assertNotIn(a, v.perms); self.assertEqual(s.life, 39)

    def test_accursed_marauder_you_choose_your_sacrifice(self):
        g = table('marchesa', 'veyran'); s, v = g.players
        hand(s, 'Accursed Marauder'); keep = perm(g, s, 'Hellkite Tyrant'); perm(g, v, 'Guttersnipe'); pool(s, B=1, C=1)
        main(g, s, {'do': 'cast', 'card': 0}, by_text('Accursed'))
        self.assertIn(keep, s.perms); self.assertFalse(any(m.name == 'Guttersnipe' for m in v.perms))

    def test_phyrexian_delver_your_pick(self):
        g = table('marchesa', 'veyran'); s = g.players[0]
        hand(s, 'Phyrexian Delver'); s.gy += [E.DB['Hellkite Tyrant'], E.DB['Burglar Rat']]; pool(s, B=2, C=3)
        main(g, s, {'do': 'cast', 'card': 0}, by_text('Burglar Rat'))
        self.assertTrue(any(m.name == 'Burglar Rat' for m in s.perms)); self.assertEqual(s.life, 38)


class Triggers(unittest.TestCase):
    def test_al_bhed_salvagers_target_opponent(self):
        g = table('marchesa', 'veyran', 'seph'); s, v, ph = g.players
        perm(g, s, 'Al Bhed Salvagers'); r = perm(g, s, 'Burglar Rat')
        seat(g, s, [by_text('Sephiroth')])
        E.die(g, r, 'destroy')
        self.assertEqual((ph.life, v.life, s.life), (39, 40, 41))

    def test_mystic_remora_upkeep_declined(self):
        g = table('marchesa', 'veyran'); s = g.players[0]
        rm = perm(g, s, 'Mystic Remora'); pool(s, C=1)
        seat(g, s, [1])                                             # "pay {1}?" no
        E.CI.HOOKS['Mystic Remora']['upkeep'](g, rm, s)
        self.assertNotIn(rm, s.perms)

    def test_dredge_or_draw(self):
        from commander_sim.play import cards
        g = table('marchesa', 'veyran'); s = g.players[0]
        s.gy.append(E.DB['Stinkweed Imp'])
        seat(g, s, [0])                                             # yes
        self.assertTrue(cards.dredge(g, s))
        self.assertTrue(any(c.name == 'Stinkweed Imp' for c in s.hand)); self.assertEqual(len(s.gy), 5)


class Outlets(unittest.TestCase):
    def test_carrion_feeder(self):
        g = table('marchesa', 'veyran'); s = g.players[0]
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
        g = table('marchesa', 'veyran'); s = g.players[0]
        cr = perm(g, s, 'Coalition Relic')
        main(g, s, {'do': 'use', 'perm': 0}, 0)
        self.assertEqual((cr.data['charge'], cr.tapped), (1, True))


if __name__ == '__main__':
    unittest.main()
