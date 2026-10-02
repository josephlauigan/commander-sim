"""Triggered abilities on the stack (engine.queue_triggers / trigger_window): they wait until the spell resolving has
finished, go on the stack in APNAP order (the person orders their own), can be answered like any stack item, and the
engine's probe for card code that triggers never does anything twice."""
import unittest
from tests.table import table, hand, lands, perm, card
from commander_sim import engine as E
from commander_sim.play.controller import ScriptController


def seat(g, p, answers):
    g.controllers = {p.key: ScriptController(answers)}
    return g.controllers[p.key]


def passes(req):
    return {'do': 'pass'} if req.kind == 'priority' else 0


def drained(g, p):
    return [q for q in g.opps(p) if q.life < 40]


class Triggers(unittest.TestCase):
    def test_an_enters_trigger_waits_for_the_spell_to_finish(self):
        g = table('seph', 'veyran', 'sauron'); s = g.players[0]
        g.resolving = 1                                         # a spell is resolving
        perm(g, s, 'Archon of Cruelty')
        self.assertEqual(drained(g, s), [])
        self.assertEqual(len(g.trig_queue), 1)
        g.resolving = 0
        E.flush_triggers(g)                                     # it has finished: the trigger goes on the stack
        self.assertEqual(len(drained(g, s)), 1)

    def test_tidebinder_counters_a_trigger(self):
        g = table('seph', 'veyran', 'sauron'); s, v, r = g.players
        hand(v, "Tishana's Tidebinder"); lands(v, 'Island', 3)
        perm(g, s, 'Archon of Cruelty')
        self.assertEqual(drained(g, s), [])                     # the drain never happened
        self.assertTrue(any(m.cd is not None and m.cd.name == "Tishana's Tidebinder" for m in v.perms))

    def test_you_get_priority_on_a_trigger(self):
        g = table('seph', 'veyran', 'sauron'); s, v, r = g.players
        hand(r, 'Counterspell'); lands(r, 'Island', 2)          # something to respond with
        ctl = seat(g, r, passes)
        perm(g, s, 'Archon of Cruelty')
        req = next(q for q in ctl.asked if q.kind == 'priority')
        self.assertIn('has a trigger', req.prompt)
        self.assertEqual(req.data['stack'][0]['name'], 'Archon of Cruelty')
        self.assertEqual(len(drained(g, s)), 1)                 # passed: it resolves

    def test_the_probe_never_doubles_an_effect(self):
        g = table('zur', 'veyran', 'sauron'); z, v, r = g.players
        seat(g, r, passes)                                      # a person at the table: triggers are probed first
        m = perm(g, z, 'Ministrant of Obligation')
        E.die(g, m, 'destroy')
        self.assertEqual(sum(1 for x in z.perms if x.token), 2)

    def test_the_active_player_s_triggers_go_on_the_stack_first(self):
        g = table('seph', 'veyran', 'sauron'); s, v, r = g.players
        g.active = v
        ts = [E.Trigger(q, None, None, (), 'etb') for q in (s, r, v)]
        self.assertEqual([t.controller for t in E._apnap(g, ts)], [v, r, s])    # so they resolve last

    def test_you_order_your_own_triggers(self):
        g = table('seph', 'veyran', 'sauron'); s = g.players[0]
        a, b = card('Archon of Cruelty'), card('Grave Titan')
        ta, tb = E.Trigger(s, a, None, (), 'etb', name='drain'), E.Trigger(s, b, None, (), 'etb', name='Zombies')
        seat(g, s, [lambda req: next(i for i, x in enumerate(req.choices) if 'Grave Titan' in x)])
        out = E._order_triggers(g, [ta, tb])
        self.assertIs(out[-1], tb)                              # pushed last: on top, resolves first

    def test_a_copied_game_keeps_its_waiting_triggers(self):
        from commander_sim.ai import search
        g = table('seph', 'veyran', 'sauron'); s = g.players[0]
        g.resolving = 1
        perm(g, s, 'Archon of Cruelty')                         # its trigger waits for the spell resolving
        g2 = search.clone(g)
        E.settle_stack(g2)                                      # the look-ahead finishes the copy's stack
        self.assertEqual(len(drained(g2, g2.players[0])), 1)
        self.assertEqual(drained(g, s), [])                     # the real game is untouched
        self.assertEqual(len(g.trig_queue), 1)

    def test_a_countered_trigger_moves_no_card(self):
        g = table('seph', 'veyran'); s = g.players[0]
        m = perm(g, s, 'Grave Titan')
        it = E.StackItem(s, m.cd, {'source': m}, 'trigger', 3, {}, generic=False, kind='trigger', name='x')
        it.countered = True
        g.stack.append(it)
        E.settle_stack(g)
        self.assertIn(m, s.perms); self.assertNotIn(m.cd, s.gy)

    def test_a_converted_hook_has_its_window(self):
        from commander_sim.cards import cardimpl as CI
        table('zur', 'veyran')
        fn = CI.HOOKS['Ministrant of Obligation']['self_dies']
        self.assertTrue(E.converted(fn))


if __name__ == '__main__':
    unittest.main()
