"""Practice mode (commander_sim/play): the session runs a game on its own thread, streams events, and shows the table
only as the human seat may see it."""
import io, unittest
from tests.table import table, hand
from commander_sim.play.session import Session
from commander_sim.play.controller import HumanController, Request
from commander_sim.play.view import build_view
from commander_sim.play import text


def default(req):
    """a do-nothing answer to any request: pass, keep the hand, first choice, no attack, continue"""
    return {'priority': {'do': 'pass'}, 'mulligan': 'keep', 'attack': []}.get(
        req.kind, 0 if req.kind in ('choose', 'target', 'block', 'search') else True)


def drain(s, answer=None, limit=20000):
    """run a started session to the end; answer each request with answer(req) (default: True)"""
    evs = []
    for _ in range(limit):
        ev = s.events.get(timeout=120)
        evs.append(ev)
        if ev['kind'] == 'request':
            req = ev['request']
            s.answer(answer(req) if answer else default(req))
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

    def test_the_ais_reasoning_is_not_shown(self):
        from commander_sim.play.session import is_private
        self.assertTrue(is_private('R4       [Tergrid decides: Jet Medallion 56%, Bloodline Keeper 11% -> Jet Medallion]'))
        self.assertFalse(is_private('R4    Tergrid casts Jet Medallion'))
        s = Session('sauron', 't3', seed=5, ai='adaptive').start()
        evs = drain(s)
        logs = [e['text'] for e in evs if e['kind'] == 'log']
        self.assertTrue(logs)
        self.assertFalse([t for t in logs if 'decides:' in t])
        self.assertTrue(any('decides:' in t for t in s.game.log))          # kept in the game's own log

    def test_actions_carry_the_table_for_playback(self):
        from commander_sim.play.session import is_action
        self.assertTrue(is_action('R4    Tergrid casts Jet Medallion'))
        self.assertFalse(is_action('R4      Jet Medallion (Tergrid) is removed: exile'))
        self.assertFalse(is_action('R4  --- Tergrid turn 4: life 40'))
        s = Session('sauron', 't2', seed=3, ai='adaptive', views=True).start()
        evs = drain(s)
        acts = [e for e in evs if e['kind'] == 'log' and is_action(e['text'])]
        self.assertTrue(acts and all('view' in e for e in acts))
        plain = drain(Session('sauron', 't2', seed=3, ai='adaptive').start())      # without views=True: none
        self.assertFalse(any('view' in e for e in plain if e['kind'] == 'log'))

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
        evs = drain(s, answer=lambda r: asked.append(r.kind) or default(r))
        self.assertEqual(evs[-1]['kind'], 'over')
        self.assertGreater(len(asked), 4)
        self.assertEqual(asked[0], 'mulligan')                  # your opening hand comes first
        self.assertTrue({'continue', 'priority'} <= set(asked))

    def test_closing_stops_the_game(self):
        s = Session('jodah', 't1', seed=2, ai='adaptive', step=True).start()
        ev = s.events.get(timeout=60)
        while ev['kind'] != 'request': ev = s.events.get(timeout=60)
        s.close(); s.join(10)
        rest = []
        while not s.events.empty(): rest.append(s.events.get())
        self.assertEqual(rest[-1]['kind'], 'over'); self.assertEqual(rest[-1]['how'], 'closed')

    def test_text_client(self):
        s = Session('sauron', 't1', seed=4, ai='adaptive')
        out = io.StringIO()
        ev = text.run(s, out=out, inp=lambda prompt='': 'keep' if 'mulligan' in prompt else 'pass')
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


class TokenNames(unittest.TestCase):
    """tokens the engine makes without a type are named after the token their owner's deck makes with that size"""
    def test_unnamed_tokens_take_the_decks_token_name(self):
        from commander_sim import engine as E
        from commander_sim.play import images
        from commander_sim.play.view import token_name
        kinds = [{'name': 'Goblin', 'power': 1, 'toughness': 1, 'colors': 'R', 'flying': False}] * 3 + \
                [{'name': 'Spirit', 'power': 1, 'toughness': 1, 'colors': '', 'flying': True},
                 {'name': 'Beast', 'power': 3, 'toughness': 3, 'colors': 'G', 'flying': False}]
        saved = images.token_kinds
        images.token_kinds = lambda names: kinds
        try:
            g = table('sauron', 'veyran'); s = g.players[0]
            goblin = E.make_tokens(g, s, 1, 1)[0]; goblin.colors = ''           # colourless: the most common 1/1
            spirit = E.make_tokens(g, s, 1, 1, fly=True)[0]; spirit.colors = ''
            beast = E.make_tokens(g, s, 1, 3)[0]
            odd = E.make_tokens(g, s, 1, 7)[0]
            self.assertEqual([token_name(m) for m in (goblin, spirit, beast, odd)],
                             ['Goblin token', 'Spirit token', 'Beast token', 'Token'])
        finally:
            images.token_kinds = saved


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
