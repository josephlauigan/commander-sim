"""Practice mode: combat declarations for the human seat. Attackers are any creatures that can legally attack;
you pick whom to attack; when attacked, you pick blockers. The engine resolves the damage."""
import unittest
from tests.table import table, perm
from commander_sim import engine as E, ais
from commander_sim.play import combat
from commander_sim.play.controller import ScriptController


def seat(g, p, answers):
    g.controllers = {p.key: ScriptController(answers)}
    return g.controllers[p.key]


class Attacking(unittest.TestCase):
    def test_who_can_attack(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        arch = perm(g, s, "Jace's Archivist"); arch.sick = False          # the AI never attacks with it; you may
        new = perm(g, s, 'Orcish Bowmasters', sick=True)
        tapped = perm(g, s, 'Grave Titan'); tapped.sick = False; tapped.tapped = True
        c = combat.attack_candidates(g, s)
        self.assertIn(arch, c); self.assertNotIn(new, c); self.assertNotIn(tapped, c)

    def test_attack_the_only_opponent(self):
        g = table('sauron', 'veyran'); s, v = g.players
        t = perm(g, s, 'Grave Titan'); t.sick = False
        seat(g, s, [[combat.attack_candidates(g, s).index(t)]])
        ais.combat(g, s)
        self.assertEqual(v.life, 40 - 6); self.assertTrue(t.tapped)

    def test_choose_the_defending_player(self):
        g = table('sauron', 'veyran', 'seph'); s, v, x = g.players
        t = perm(g, s, 'Grave Titan'); t.sick = False
        ctl = seat(g, s, [[0], 1])
        ais.combat(g, s)
        self.assertEqual((v.life, x.life), (40, 34))
        self.assertEqual([r.kind for r in ctl.asked], ['attack', 'target'])

    def test_no_attack(self):
        g = table('sauron', 'veyran'); s, v = g.players
        t = perm(g, s, 'Grave Titan'); t.sick = False
        seat(g, s, [[]])
        ais.combat(g, s)
        self.assertEqual(v.life, 40); self.assertFalse(t.tapped)


class Blocking(unittest.TestCase):
    def test_block(self):
        g = table('sauron', 'veyran'); s, v = g.players
        wall = perm(g, s, 'Grave Titan'); wall.sick = False
        a = perm(g, v, 'Guttersnipe'); a.sick = False
        ctl = seat(g, s, [{'do': 'pass'}, 0])
        g.active = v
        ais.resolve_combat(g, v, [a], s, set())
        self.assertEqual(s.life, 40); self.assertNotIn(a, v.perms)
        self.assertEqual([r.kind for r in ctl.asked], ['priority', 'block'])

    def test_no_block(self):
        g = table('sauron', 'veyran'); s, v = g.players
        perm(g, s, 'Grave Titan').sick = False
        a = perm(g, v, 'Guttersnipe'); a.sick = False
        seat(g, s, [{'do': 'pass'}, 'cancel'])
        g.active = v
        ais.resolve_combat(g, v, [a], s, set())
        self.assertEqual(s.life, 38)

    def test_unblockable_attackers_are_not_asked_about(self):
        g = table('sauron', 'veyran'); s, v = g.players
        perm(g, s, 'Grave Titan').sick = False
        a = perm(g, v, 'Guttersnipe'); a.sick = False
        ctl = seat(g, s, [{'do': 'pass'}])
        g.active = v
        ais.resolve_combat(g, v, [a], s, {a})
        self.assertEqual([r.kind for r in ctl.asked], ['priority']); self.assertEqual(s.life, 38)


if __name__ == '__main__':
    unittest.main()
