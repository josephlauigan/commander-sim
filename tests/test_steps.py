"""Priority in every step (engine.step_priority): the person is asked in the steps their auto-pass setting stops at
('all': upkeep, draw, beginning of combat, the end step ...; by default only when attacked, at the end of each other
turn, and in the declare blockers step of their combat when they could do something). A creature whose blocker is
removed stays blocked."""
import unittest
from tests.table import table, hand, lands, perm
from commander_sim import engine as E, ais
from commander_sim.play.controller import ScriptController


def seat(g, p, answers, autopass=None):
    ctl = ScriptController(answers)
    if autopass: ctl.autopass = autopass
    g.controllers = {p.key: ctl}
    return ctl


def passes(req):
    return {'do': 'pass'} if req.kind == 'priority' else 'cancel'


def prompts(ctl):
    return [r.prompt for r in ctl.asked if r.kind == 'priority']


class Steps(unittest.TestCase):
    def test_full_control_stops_at_every_step(self):
        g = table('sauron', 'veyran'); s, v = g.players
        ctl = seat(g, v, passes, autopass='all')
        g.round = 2
        ais.take_turn(g, s)
        ps = prompts(ctl)
        for step in ('upkeep', 'draw step', 'beginning of combat', 'End of'):
            self.assertTrue(any(step in x for x in ps), (step, ps))

    def test_by_default_only_the_end_of_the_turn(self):
        g = table('sauron', 'veyran'); s, v = g.players
        hand(v, 'Lightning Bolt'); lands(v, 'Mountain', 1)          # something you could do
        ctl = seat(g, v, passes)
        g.round = 2
        ais.take_turn(g, s)
        ps = prompts(ctl)
        self.assertFalse(any('upkeep' in x or 'draw step' in x for x in ps), ps)
        self.assertTrue(any("End of" in x for x in ps), ps)

    def test_the_declare_blockers_step_of_your_combat(self):
        g = table('veyran', 'sauron'); v, s = g.players
        perm(g, s, 'Grave Titan').sick = False
        a = perm(g, v, 'Guttersnipe'); a.sick = False
        hand(s, 'Lightning Bolt'); lands(s, 'Mountain', 1)
        ctl = seat(g, s, [{'do': 'pass'}, 'cancel', {'do': 'pass'}])
        ais.resolve_combat(g, v, [a], s, set())
        self.assertEqual([r.kind for r in ctl.asked], ['priority', 'block', 'priority'])
        self.assertIn('declare blockers step', ctl.asked[2].prompt)

    def test_no_blockers_step_stop_with_nothing_to_do(self):
        g = table('veyran', 'sauron'); v, s = g.players
        perm(g, s, 'Grave Titan').sick = False
        a = perm(g, v, 'Guttersnipe'); a.sick = False
        ctl = seat(g, s, [{'do': 'pass'}, 'cancel'])
        ais.resolve_combat(g, v, [a], s, set())
        self.assertEqual([r.kind for r in ctl.asked], ['priority', 'block'])

    def test_a_creature_whose_blocker_is_removed_stays_blocked(self):
        g = table('veyran', 'sauron', 'seph'); v, s, ph = g.players
        a = perm(g, v, 'Guttersnipe'); a.sick = False
        b = perm(g, s, 'Grave Titan'); b.sick = False                # a block the AI makes
        g.controllers = {'nobody': ScriptController([])}            # practice mode, nobody of these seats
        real = E.step_priority
        def kill_blocker(g, step, defender=None, attackers=()):
            if step == 'blockers' and b in s.perms: E.die(g, b, 'destroy')
        E.step_priority = kill_blocker
        try:
            ais.resolve_combat(g, v, [a], s, set())
        finally:
            E.step_priority = real
        self.assertNotIn(b, s.perms)
        self.assertEqual(s.life, 40)                                 # Guttersnipe has no trample


if __name__ == '__main__':
    unittest.main()
