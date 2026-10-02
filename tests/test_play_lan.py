"""Practice mode with two people (each on their own computer) and two AI opponents: the session seats both, asks each
their own decisions and keeps their answers in order (Undo, saving and loading replay both); the server's lobby and
join code, each browser's seat (a cookie), each seat's own stream with the other's hand and secrets left out, and an
Undo the other person must agree to."""
import http.client
import json
import queue
import threading
import time
import unittest

from commander_sim import engine as E
from commander_sim.play import server
from commander_sim.play.session import Session, EventLog
from tests.test_play import default


def bot(req, played):
    """land, tap, then pass; keep every hand; first choice otherwise (played: (seat, round) already given a land)"""
    if req.kind != 'priority': return default(req)
    view = req.data['view']
    me = next(p for p in view['players'] if p['you'])
    if view['active'] == me['key'] and view['step'] == 'main1':
        lands = [i for i, x in enumerate(me['hand_land']) if x]
        if lands and (me['key'], view['round']) not in played:
            played.add((me['key'], view['round']))
            return {'do': 'land', 'card': lands[0]}
        if me['mana_sources']: return {'do': 'tap', 'source': me['mana_sources'][0]['id']}
    return {'do': 'pass'}


def next_request(s):
    while True:
        ev = s.events.get(timeout=300)
        if ev['kind'] == 'request': return ev
        if ev['kind'] in ('over', 'error'): return ev


def frozen(ev):
    req = ev['request']
    return json.dumps({'seat': ev['seat'], 'kind': req.kind, 'prompt': req.prompt, 'choices': req.choices,
                       'view': req.data.get('view')}, sort_keys=True, default=str)


class TwoPlayerSession(unittest.TestCase):
    def play(self, decisions, seed=7):
        s = Session('sauron', 't2', seed=seed, ai='adaptive', partner='seph').start()
        played, seen = set(), []
        for _ in range(decisions):
            ev = next_request(s)
            self.assertEqual(ev['kind'], 'request', ev.get('text'))
            seen.append(frozen(ev))
            s.answer(bot(ev['request'], played), ev['seat'])
        nxt = next_request(s)
        return s, seen, nxt

    def test_both_seated_and_asked_their_own_decisions(self):
        s, seen, _ = self.play(60)
        try:
            self.assertEqual(sorted(s.humans), ['sauron', 'seph'])
            self.assertEqual(len([k for k in s.seats if k not in s.humans]), 2)
            self.assertEqual(set(s.answer_seats), {'sauron', 'seph'})
            kinds = [json.loads(x) for x in seen]
            self.assertEqual(sorted(x['seat'] for x in kinds if x['kind'] == 'mulligan'), ['sauron', 'seph'])
            for x in kinds:                          # each request's table is from its own seat
                if x['view']:
                    me = next(p for p in x['view']['players'] if p['you'])
                    self.assertEqual(me['key'], x['seat'])
                    self.assertTrue(all('hand' not in p for p in x['view']['players'] if not p['you']))
        finally:
            s.close(); s.join(30)

    def test_views_for_each_seat(self):
        s = Session('sauron', 't2', seed=7, ai='adaptive', partner='seph', views=True).start()
        try:
            played = set()
            while True:
                ev = s.events.get(timeout=300)
                if ev['kind'] == 'request': s.answer(bot(ev['request'], played), ev['seat'])
                if ev['kind'] == 'turn': break
            self.assertEqual(sorted(ev['views']), ['sauron', 'seph'])
            for k, v in ev['views'].items(): self.assertEqual(next(p for p in v['players'] if p['you'])['key'], k)
        finally:
            s.close(); s.join(30)

    def test_undo_by_either_replays_both(self):
        s, seen, _ = self.play(50)
        try:
            to = s.undo_point(1, 'seph')
            self.assertIsNone(s.undo(1, 'seph'))
            ev = next_request(s)
            self.assertEqual(ev['seat'], 'seph')
            self.assertEqual(frozen(ev), seen[to])
            self.assertEqual(len(s.answers), to)
        finally:
            s.close(); s.join(30)

    def test_save_and_load_replays_both(self):
        s, seen, nxt = self.play(40)
        try:
            data = s.saved()
            self.assertEqual(data['partner'], 'seph')
            self.assertEqual(len(data['answer_seats']), 40)
        finally:
            s.close(); s.join(30)
        t = Session.load(json.loads(json.dumps(data))).start()
        try:
            ev = next_request(t)
            self.assertEqual(frozen(ev), frozen(nxt))
        finally:
            t.close(); t.join(30)

    def test_separate_reviews_and_try_it(self):
        s = Session('sauron', 't2', seed=7, ai='adaptive', partner='seph', compare=True, max_rounds=4).start()
        played = set()
        try:
            while True:
                ev = s.events.get(timeout=600)
                if ev['kind'] == 'request': s.answer(bot(ev['request'], played), ev['seat'])
                if ev['kind'] in ('over', 'error'): break
            self.assertEqual(ev['kind'], 'over', ev.get('text'))
            a, b = s.review('sauron'), s.review('seph')
            self.assertEqual(a['summary']['answers'] + b['summary']['answers'], len(s.answers))
            mine = {i for i, k in enumerate(s.answer_seats) if k == 'seph'}
            self.assertTrue(b['decisions'] and all(d['n'] in mine for d in b['decisions']))
            self.assertTrue(all(d['n'] not in mine for d in a['decisions']))
            tryable = [d for d in b['decisions'] if d['can_try']]
            self.assertTrue(tryable)
            n = tryable[0]['n']
            self.assertIsNone(s.try_entry(n, 'sauron'))               # not one of Sauron's decisions
            self.assertIsNone(s.try_it(n, 'seph'))
            ev = next_request(s)
            self.assertEqual(ev['seat'], 'seph')
            self.assertGreaterEqual(len(s.answers), n)
        finally:
            s.close(); s.join(30)

    def test_hint_is_for_the_seat_deciding(self):
        s = Session('sauron', 't2', seed=7, ai='adaptive', partner='seph').start()
        try:
            ev = next_request(s)
            other = next(k for k in s.humans if k != ev['seat'])
            self.assertIn('no decision', s.hint(other))
            self.assertNotIsInstance(s.hint(ev['seat']), str)
        finally:
            s.close(); s.join(30)

    def test_same_deck_twice_is_refused(self):
        with self.assertRaises(ValueError): Session('sauron', 't2', partner='sauron')


class Secrets(unittest.TestCase):
    def test_a_tutored_card_is_named_only_to_its_owner(self):
        s = Session('sauron', 't2', seed=1, partner='seph')
        g = type('G', (), {})(); g.round = 3; g.log = EventLog(s)
        p = type('P', (), {})(); p.key = 'seph'
        E.log_secret(g, p, '    Sephiroth tutors a card', '    Sephiroth tutors Entomb')
        evs = [s.events.get_nowait() for _ in range(2)]
        self.assertEqual(g.log, ['R3      Sephiroth tutors Entomb'])
        self.assertEqual([server.for_seat(dict(e, id=i), 'sauron') for i, e in enumerate(evs)],
                         [{'id': 0, 'kind': 'log', 'text': 'R3      Sephiroth tutors a card'}, None])
        self.assertEqual([server.for_seat(dict(e, id=i), 'seph') for i, e in enumerate(evs)],
                         [None, {'id': 1, 'kind': 'log', 'text': 'R3      Sephiroth tutors Entomb'}])

    def test_an_ai_tutor_is_hidden_from_everyone(self):
        s = Session('sauron', 't2', seed=1)
        g = type('G', (), {})(); g.round = 3; g.log = EventLog(s)
        p = type('P', (), {})(); p.key = 'some-ai'
        E.log_secret(g, p, '    X tutors a card', '    X tutors Demonic Tutor')
        evs = [s.events.get_nowait()]
        self.assertTrue(s.events.empty())
        self.assertEqual(server.for_seat(dict(evs[0], id=1), 'sauron')['text'], 'R3      X tutors a card')

    def test_a_simulation_keeps_the_whole_line(self):
        g = type('G', (), {})(); g.round = 2; g.log = []
        p = type('P', (), {})(); p.key = 'x'
        E.log_secret(g, p, 'public', 'whole')
        self.assertEqual(g.log, ['R2  whole'])


class Browser:
    """one person's browser: its own cookie, and (when playing) its own event stream answered by the bot"""
    def __init__(self, port, remote=False):
        self.port, self.remote, self.cookie = port, remote, None
        self.events, self.requests = [], queue.Queue()
        self.stop = threading.Event()
        self.auto = True

    def call(self, method, path, body=None):
        c = http.client.HTTPConnection('127.0.0.1', self.port, timeout=60)
        h = {'Content-Type': 'application/json'} if body is not None else {}
        if self.cookie: h['Cookie'] = self.cookie
        if self.remote: h['X-Test-Remote'] = '1'
        c.request(method, path, json.dumps(body) if body is not None else None, h)
        r = c.getresponse(); data = r.read()
        sc = r.getheader('Set-Cookie')
        if sc: self.cookie = sc.split(';')[0]
        c.close()
        return r.status, json.loads(data) if data and data[:1] in b'{[' else data

    def listen(self, played):
        """read this seat's stream in the background; answer its requests with the bot while auto is on"""
        def run():
            c = http.client.HTTPConnection('127.0.0.1', self.port, timeout=300)
            h = {'Cookie': self.cookie}
            if self.remote: h['X-Test-Remote'] = '1'
            c.request('GET', '/api/events?since=0', headers=h)
            r = c.getresponse()
            while not self.stop.is_set():
                line = r.fp.readline().decode()
                if not line: break
                if not line.startswith('data: '): continue
                ev = json.loads(line[6:])
                self.events.append(ev)
                if ev['kind'] == 'request':
                    if self.auto:
                        req = type('R', (), {'kind': ev['request']['kind'], 'data': dict(ev['request']['data'], view=ev.get('view'))})
                        if req.data['view'] is None and req.kind == 'priority': continue
                        self.call('POST', '/api/answer', {'id': ev['id'], 'answer': bot(req, played)})
                    else:
                        self.requests.put(ev)
            c.close()
        threading.Thread(target=run, daemon=True).start()


class TwoPlayerServer(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.srv = server.make_server(port=0)
        cls.srv.hub.lan = True
        # requests marked X-Test-Remote come "from the other computer" (the test can't use a second machine)
        cls.srv.RequestHandlerClass.local = lambda self: self.headers.get('X-Test-Remote') is None
        cls.port = cls.srv.server_address[1]
        threading.Thread(target=cls.srv.serve_forever, daemon=True).start()

    @classmethod
    def tearDownClass(cls):
        cls.srv.hub.quit(); cls.srv.shutdown(); cls.srv.server_close()

    def open_table(self):
        host, friend = Browser(self.port), Browser(self.port, remote=True)
        host.call('GET', '/'); friend.call('GET', '/')
        st, r = host.call('POST', '/api/new', {'deck': 'sauron', 'tier': 't2', 'seed': 7, 'ai': 'adaptive',
                                               'images': False, 'two': True})
        self.assertEqual(st, 200, r)
        return host, friend, r['code']

    def test_the_lobby_and_joining(self):
        host, friend, code = self.open_table()
        _, st = host.call('GET', '/api/state')
        self.assertEqual(st['lobby']['code'], code)
        _, st = friend.call('GET', '/api/state')
        self.assertNotIn('code', st['lobby'])              # the friend needs it from the host
        self.assertEqual(st['lobby']['deck'], 'sauron')
        self.assertEqual(friend.call('POST', '/api/join', {'code': '000000' if code != '000000' else '111111',
                                                           'deck': 'seph'})[0], 409)
        self.assertEqual(friend.call('POST', '/api/join', {'code': code, 'deck': 'sauron'})[0], 409)
        self.assertEqual(host.call('POST', '/api/join', {'code': code, 'deck': 'seph'})[0], 409)
        self.assertEqual(friend.call('POST', '/api/join', {'code': code, 'deck': 'seph'})[0], 200)
        _, a = host.call('GET', '/api/state'); _, b = friend.call('GET', '/api/state')
        self.assertEqual((a['game']['you'], b['game']['you']), ('sauron', 'seph'))
        self.assertEqual(sorted(a['game']['humans']), ['sauron', 'seph'])
        self.assertEqual(friend.call('POST', '/api/new', {'deck': 'veyran', 'tier': 't2'})[0], 403)
        self.assertEqual(friend.call('POST', '/api/quit', {})[0], 403)
        stranger = Browser(self.port, remote=True); stranger.call('GET', '/')
        _, st = stranger.call('GET', '/api/state')
        self.assertIsNone(st['game']); self.assertTrue(st['busy'])
        host.call('POST', '/api/quit', {})

    def test_each_seat_sees_its_own_game(self):
        host, friend, code = self.open_table()
        friend.call('POST', '/api/join', {'code': code, 'deck': 'seph'})
        played = set()
        host.listen(played); friend.listen(played)
        deadline = time.time() + 300
        while time.time() < deadline and not any(e['kind'] == 'turn' and e.get('view', {}).get('round', 0) >= 3
                                                  for e in host.events):
            time.sleep(0.2)
        host.stop.set(); friend.stop.set()
        host.call('POST', '/api/quit', {})
        for b, me, other in ((host, 'sauron', 'seph'), (friend, 'seph', 'sauron')):
            reqs = [e for e in b.events if e['kind'] == 'request']
            self.assertTrue(reqs)
            self.assertTrue(any(e['kind'] == 'waiting' for e in b.events))    # the other person's decisions
            for e in b.events:
                v = e.get('view')
                if not v: continue
                you = [p for p in v['players'] if p['you']]
                self.assertEqual([p['key'] for p in you], [me])
                self.assertTrue(all('hand' not in p for p in v['players'] if not p['you']))
            self.assertFalse(any('seat' in e or 'views' in e or 'hide' in e for e in b.events))
            mull = [e for e in reqs if e['request']['kind'] == 'mulligan']
            self.assertEqual(len(mull), 1)

    def test_undo_needs_the_other_persons_ok(self):
        host, friend, code = self.open_table()
        friend.call('POST', '/api/join', {'code': code, 'deck': 'seph'})
        played = set()
        host.auto = friend.auto = False
        host.listen(played); friend.listen(played)
        answered = {'sauron': 0, 'seph': 0}
        while min(answered.values()) < 3:                  # both mulligans and a few decisions each
            for b, k in ((host, 'sauron'), (friend, 'seph')):
                try:
                    ev = b.requests.get(timeout=0.5)
                except queue.Empty:
                    continue
                req = type('R', (), {'kind': ev['request']['kind'], 'data': dict(ev['request']['data'], view=ev.get('view'))})
                self.assertEqual(b.call('POST', '/api/answer', {'id': ev['id'], 'answer': bot(req, played)})[0], 200)
                answered[k] += 1
        st, r = friend.call('POST', '/api/undo', {'n': 1})
        self.assertEqual(st, 200, r); self.assertTrue(r['asked'])
        self.assertEqual(friend.call('POST', '/api/undo', {'n': 1})[0], 409)       # one request at a time
        deadline = time.time() + 30
        while time.time() < deadline and not any(e['kind'] == 'proposal' for e in host.events): time.sleep(0.1)
        prop = next(e for e in host.events if e['kind'] == 'proposal')['proposal']
        self.assertIn('take back', prop['text'])
        self.assertFalse(any(e['kind'] == 'proposal' for e in friend.events))
        _, st = host.call('GET', '/api/state'); self.assertEqual(st['proposal']['id'], prop['id'])
        self.assertEqual(friend.call('POST', '/api/approve', {'id': prop['id'], 'yes': True})[0], 409)   # not theirs
        self.assertEqual(host.call('POST', '/api/approve', {'id': prop['id'], 'yes': False})[0], 200)
        deadline = time.time() + 30
        while time.time() < deadline and not any(e['kind'] == 'declined' for e in friend.events): time.sleep(0.1)
        self.assertTrue(any(e['kind'] == 'declined' for e in friend.events))
        n_before = len(self.srv.hub.session.answers)
        st, r = friend.call('POST', '/api/undo', {'n': 1})
        self.assertEqual(st, 200, r)
        deadline = time.time() + 30
        while time.time() < deadline and len([e for e in host.events if e['kind'] == 'proposal']) < 2: time.sleep(0.1)
        prop = [e for e in host.events if e['kind'] == 'proposal'][-1]['proposal']
        self.assertEqual(host.call('POST', '/api/approve', {'id': prop['id'], 'yes': True})[0], 200)
        deadline = time.time() + 120
        while time.time() < deadline and not (any(e['kind'] == 'reset' for e in friend.events)
                                              and any(e['kind'] == 'reset' for e in host.events)): time.sleep(0.1)
        self.assertTrue(any(e['kind'] == 'reset' for e in host.events) and any(e['kind'] == 'reset' for e in friend.events))
        self.assertLess(len(self.srv.hub.session.answers), n_before)
        host.stop.set(); friend.stop.set()
        host.call('POST', '/api/quit', {})

    def test_a_saved_two_player_game_waits_for_the_friend(self):
        import os
        host, friend, code = self.open_table()
        friend.call('POST', '/api/join', {'code': code, 'deck': 'seph'})
        played = set()
        host.listen(played); friend.listen(played)
        deadline = time.time() + 120
        while time.time() < deadline and len(self.srv.hub.session.answers) < 12: time.sleep(0.1)
        host.auto = friend.auto = False
        while time.time() < deadline and host.requests.empty() and friend.requests.empty(): time.sleep(0.1)
        st, r = host.call('POST', '/api/save', {})                 # a decision is waiting, unanswered
        self.assertEqual(st, 200, r)
        self.assertIn('+seph', r['name'])
        host.stop.set(); friend.stop.set()
        try:
            st, r2 = host.call('POST', '/api/load', {'name': r['name'], 'images': False})
            self.assertEqual(st, 200, r2); self.assertTrue(r2['code'])
            _, lb = friend.call('GET', '/api/state')
            self.assertEqual((lb['lobby']['deck'], lb['lobby']['partner'], lb['lobby']['loaded']), ('sauron', 'seph', True))
            self.assertEqual(friend.call('POST', '/api/join', {'code': r2['code'], 'deck': 'veyran'})[0], 409)
            self.assertEqual(friend.call('POST', '/api/join', {'code': r2['code'], 'deck': 'seph'})[0], 200)
            _, b = friend.call('GET', '/api/state')
            self.assertEqual(b['game']['you'], 'seph')
        finally:
            host.call('POST', '/api/quit', {})
            os.remove(os.path.join(server.SAVES, r['name']))

    def test_one_player_undo_is_at_once(self):
        host = Browser(self.port); host.call('GET', '/')
        host.call('POST', '/api/new', {'deck': 'sauron', 'tier': 't2', 'seed': 4, 'ai': 'adaptive', 'images': False})
        host.auto = False; host.listen(set())
        ev = host.requests.get(timeout=120)
        host.call('POST', '/api/answer', {'id': ev['id'], 'answer': 'keep'})
        host.requests.get(timeout=120)
        st, r = host.call('POST', '/api/undo', {'n': 1})
        self.assertEqual((st, r.get('asked')), (200, None))
        host.stop.set(); host.call('POST', '/api/quit', {})


if __name__ == '__main__':
    unittest.main()
