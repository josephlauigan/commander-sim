"""Practice mode's browser server (play/server.py): the page and its files, the options for the setup screen, starting
a game, the event stream, answering decisions (stale answers refused), and hidden information in what's sent."""
import http.client
import json
import threading
import unittest
from commander_sim.play import server


class Server(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.srv = server.make_server(port=0)
        cls.port = cls.srv.server_address[1]
        threading.Thread(target=cls.srv.serve_forever, daemon=True).start()

    @classmethod
    def tearDownClass(cls):
        cls.srv.hub.quit(); cls.srv.shutdown(); cls.srv.server_close()

    def call(self, method, path, body=None):
        c = http.client.HTTPConnection('127.0.0.1', self.port, timeout=30)
        c.request(method, path, json.dumps(body) if body is not None else None,
                  {'Content-Type': 'application/json'} if body is not None else {})
        r = c.getresponse(); data = r.read(); c.close()
        return r.status, r.getheader('Content-Type'), data

    def wait_for(self, pred, since=0, limit=400):
        """read the event stream until an event matches pred; returns it"""
        c = http.client.HTTPConnection('127.0.0.1', self.port, timeout=60)
        c.request('GET', f'/api/events?since={since}')
        r = c.getresponse()
        try:
            for _ in range(limit * 4):
                line = r.fp.readline().decode()
                if line.startswith('data: '):
                    ev = json.loads(line[6:])
                    if pred(ev): return ev
            self.fail('no matching event')
        finally:
            c.close()

    def test_the_page_and_its_files(self):
        st, ct, body = self.call('GET', '/')
        self.assertEqual(st, 200); self.assertIn('text/html', ct); self.assertIn(b'app.js', body)
        st, ct, _ = self.call('GET', '/static/app.js')
        self.assertEqual(st, 200); self.assertIn('javascript', ct)
        self.assertEqual(self.call('GET', '/static/../server.py')[0], 404)
        self.assertEqual(self.call('GET', '/static/%2e%2e/server.py')[0], 404)

    def test_card_images_are_served(self):
        import os, tempfile
        d = tempfile.mkdtemp(); saved = server.IMAGES
        try:
            server.IMAGES = d
            with open(os.path.join(d, 'abc.jpg'), 'wb') as fh: fh.write(b'jpg')
            st, ct, body = self.call('GET', '/images/abc.jpg')
            self.assertEqual((st, ct, body), (200, 'image/jpeg', b'jpg'))
            self.assertEqual(self.call('GET', '/images/../index.json')[0], 404)
        finally:
            server.IMAGES = saved

    def test_the_loading_screen_comes_first(self):
        self.call('POST', '/api/new', {'deck': 'veyran', 'tier': 't1', 'seed': 3, 'ai': 'adaptive', 'images': False})
        first = self.wait_for(lambda e: e['kind'] in ('images', 'request', 'log'))
        self.assertEqual(first['kind'], 'images')
        self.call('POST', '/api/quit', {})

    def test_options(self):
        st, _, body = self.call('GET', '/api/options')
        d = json.loads(body)
        self.assertEqual(sorted(d['decks']), ['marchesa', 'sauron', 'seph', 'veyran'])
        self.assertEqual(len(d['tiers']['t3']), 5)

    def test_bad_new_game(self):
        st, _, body = self.call('POST', '/api/new', {'deck': 'nobody', 'tier': 't1'})
        self.assertEqual(st, 400)

    def test_a_game_from_the_browser(self):
        st, _, body = self.call('POST', '/api/new', {'deck': 'sauron', 'tier': 't2', 'seed': 7, 'ai': 'adaptive', 'images': False})
        self.assertEqual(st, 200)
        self.assertIn('sauron', json.loads(body)['seats'])
        mull = self.wait_for(lambda e: e['kind'] == 'request')
        self.assertEqual(mull['request']['kind'], 'mulligan')
        st, _, _ = self.call('POST', '/api/answer', {'id': mull['id'] + 1000, 'answer': 'keep'})
        self.assertEqual(st, 409)                                         # not the waiting decision
        st, _, _ = self.call('POST', '/api/answer', {'id': mull['id'], 'answer': 'keep'})
        self.assertEqual(st, 200)
        self.assertEqual(self.call('POST', '/api/answer', {'id': mull['id'], 'answer': 'keep'})[0], 409)   # answered
        nxt = self.wait_for(lambda e: e['kind'] == 'request', since=mull['id'])
        self.assertIn(nxt['request']['kind'], ('priority', 'choose', 'target', 'attack'))
        state = json.loads(self.call('GET', '/api/state')[2])
        self.assertEqual(state['pending']['id'], nxt['id'])
        view = state['view']
        me = next(p for p in view['players'] if p['you'])
        others = [p for p in view['players'] if not p['you']]
        self.assertEqual(me['key'], 'sauron'); self.assertIn('hand', me)
        self.assertTrue(all('hand' not in p for p in others))              # their hands are counts only
        self.assertEqual(self.call('POST', '/api/quit', {})[0], 200)
        self.assertIsNone(json.loads(self.call('GET', '/api/state')[2])['game'])


if __name__ == '__main__':
    unittest.main()
