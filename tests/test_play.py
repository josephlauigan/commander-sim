"""Practice mode (commander_sim/play): the session runs a game on its own thread, streams events, and shows the table
only as the human seat may see it."""
import io, unittest
from tests.table import table, hand
from commander_sim.play.session import Session
from commander_sim.play.controller import HumanController, Request
from commander_sim.play.view import build_view
from commander_sim.play import text


def drain(s, answer=None, limit=20000):
    """run a started session to the end; answer each request with answer(req) (default: True)"""
    evs = []
    for _ in range(limit):
        ev = s.events.get(timeout=120)
        evs.append(ev)
        if ev['kind'] == 'request':
            req = ev['request']
            s.answer(answer(req) if answer else ({'do': 'pass'} if req.kind == 'priority' else True))
        if ev['kind'] in ('over', 'error'): break
    return evs


class Session_(unittest.TestCase):
    def test_a_game_plays_to_the_end(self):
        s = Session('sauron', 't2', seed=3, ai='adaptive').start()
        evs = drain(s)
        self.assertEqual(evs[-1]['kind'], 'over', evs[-1].get('text'))
        self.assertIsNotNone(evs[-1]['view'])
        self.assertTrue(any(e['kind'] == 'log' and 'Seat order' in e['text'] for e in evs))
        self.assertTrue(any(e['kind'] == 'turn' and e['player'] == 'sauron' for e in evs))

    def test_seat_and_opponents(self):
        s = Session('seph', 't4', seed=5, seat=1, opponents=['krenko-mono-red-goblins', 'yuriko-dimir-ninjas',
                                                            'heliod-mono-white-stax'], ai='adaptive')
        self.assertEqual(s.seats[0], 'seph')
        self.assertEqual(sorted(s.seats[1:]), ['heliod-mono-white-stax', 'krenko-mono-red-goblins', 'yuriko-dimir-ninjas'])
        with self.assertRaises(ValueError): Session('seph', 't4', opponents=['krenko-mono-red-goblins'])
        with self.assertRaises(ValueError): Session('najeela', 't4')

    def test_step_mode_waits_for_the_human(self):
        s = Session('veyran', 't1', seed=2, ai='adaptive', step=True).start()
        asked = []
        evs = drain(s, answer=lambda r: asked.append(r.kind) or ({'do': 'pass'} if r.kind == 'priority' else True))
        self.assertEqual(evs[-1]['kind'], 'over')
        self.assertGreater(len(asked), 4)
        self.assertEqual(set(asked), {'continue', 'priority'})

    def test_closing_stops_the_game(self):
        s = Session('marchesa', 't1', seed=2, ai='adaptive', step=True).start()
        ev = s.events.get(timeout=60)
        while ev['kind'] != 'request': ev = s.events.get(timeout=60)
        s.close(); s.join(10)
        rest = []
        while not s.events.empty(): rest.append(s.events.get())
        self.assertEqual(rest[-1]['kind'], 'over'); self.assertEqual(rest[-1]['how'], 'closed')

    def test_text_client(self):
        s = Session('sauron', 't1', seed=4, ai='adaptive')
        out = io.StringIO()
        ev = text.run(s, out=out, inp=lambda prompt='': 'pass')
        self.assertEqual(ev['kind'], 'over')
        self.assertIn('Game over:', out.getvalue())
        self.assertIn('(you)', out.getvalue())


class View(unittest.TestCase):
    def test_hidden_information(self):
        g = table('sauron', 'veyran'); s, v = g.players
        hand(s, 'Counterspell'); hand(v, 'Lightning Bolt')
        view = build_view(g, 'sauron')
        me, opp = view['players']
        self.assertEqual(me['hand'], ['Counterspell'])
        self.assertNotIn('hand', opp); self.assertEqual(opp['hand_count'], 1)
        self.assertIsInstance(opp['library'], int)

    def test_tokens_have_names(self):
        from commander_sim import engine as E
        g = table('sauron', 'veyran'); s = g.players[0]
        E.amass(g, s, 2)
        names = [m['name'] for m in build_view(g, 'sauron')['players'][0]['battlefield']]
        self.assertIn('Orc Army', names)


class Controller(unittest.TestCase):
    def test_look_ahead_copies_leave_the_human_out(self):
        from commander_sim.ai import search
        g = table('sauron', 'veyran')
        g.controllers = {'sauron': HumanController()}
        g2 = search.clone(g)
        self.assertFalse(hasattr(g2, 'controllers') and g2.controllers)
        self.assertIn('sauron', g.controllers)

    def test_ask_and_answer(self):
        import threading
        h = HumanController(); got = []
        t = threading.Thread(target=lambda: got.append(h.ask(Request('mulligan', 'Keep?', ['keep', 'mulligan']))))
        t.start()
        req = h.requests.get(timeout=5)
        self.assertEqual(req.choices, ['keep', 'mulligan'])
        h.answer('keep'); t.join(5)
        self.assertEqual(got, ['keep'])


if __name__ == '__main__':
    unittest.main()
