"""The iPad app's start-up (ios/): staging copies the core, deck lists and card data; the app copies its bundled data
into a writable folder and serves the practice table from the copies. Runs without Toga or an iPad."""
import json
import os
import subprocess
import sys
import tempfile
import unittest

from commander_sim import ROOT

IOS = os.path.join(ROOT, 'ios')
sys.path.insert(0, os.path.join(IOS, 'src'))
from commander_ipad import bootstrap            # noqa: E402  (bootstrap imports nothing from commander_sim)


def write(path, data):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, 'w', encoding='utf-8') as f: f.write(data if isinstance(data, str) else json.dumps(data))


def read(path):
    with open(path, encoding='utf-8') as f: return f.read()


class SyncData(unittest.TestCase):
    def setUp(self):
        t = tempfile.mkdtemp()
        self.bundled, self.data = os.path.join(t, 'bundled'), os.path.join(t, 'data')
        write(os.path.join(self.bundled, 'bundle.json'), {'commit': 'a'})
        write(os.path.join(self.bundled, 'scryfall_cache.json'), {'sol ring': 1})
        write(os.path.join(self.bundled, 'images', 'index.json'), {'sol ring': {'files': ['s.jpg']}})
        write(os.path.join(self.bundled, 'images', 's.jpg'), 'bundled image')

    def test_first_launch_copies_everything(self):
        self.assertTrue(bootstrap.sync_data(self.bundled, self.data))
        self.assertEqual(json.loads(read(os.path.join(self.data, 'scryfall_cache.json'))), {'sol ring': 1})
        self.assertEqual(read(os.path.join(self.data, 'images', 's.jpg')), 'bundled image')
        self.assertFalse(bootstrap.sync_data(self.bundled, self.data))          # same build: nothing to do

    def test_a_new_build_replaces_the_cache_and_keeps_what_the_ipad_added(self):
        bootstrap.sync_data(self.bundled, self.data)
        write(os.path.join(self.data, 'saves', 'g.json'), 'a saved game')
        write(os.path.join(self.data, 'images', 'd.jpg'), 'downloaded on the iPad')
        idx = os.path.join(self.data, 'images', 'index.json')
        write(idx, {**json.loads(read(idx)), 'dockside': {'files': ['d.jpg']}})
        write(os.path.join(self.bundled, 'bundle.json'), {'commit': 'b'})
        write(os.path.join(self.bundled, 'scryfall_cache.json'), {'sol ring': 2})
        self.assertTrue(bootstrap.sync_data(self.bundled, self.data))
        self.assertEqual(json.loads(read(os.path.join(self.data, 'scryfall_cache.json'))), {'sol ring': 2})
        self.assertEqual(set(json.loads(read(idx))), {'sol ring', 'dockside'})
        self.assertEqual(read(os.path.join(self.data, 'images', 'd.jpg')), 'downloaded on the iPad')
        self.assertEqual(read(os.path.join(self.data, 'saves', 'g.json')), 'a saved game')


APP = r'''
import json, os, sys, time, urllib.request
out, data = sys.argv[1], sys.argv[2]
sys.path.insert(0, out)
from commander_ipad import bootstrap
srv, url = bootstrap.start(os.path.join(out, 'commander_ipad'), data, fetch_images=False)
import commander_sim
from commander_sim.play import server
page = urllib.request.urlopen(url).read().decode()
opts = json.load(urllib.request.urlopen(url + 'api/options'))


def post(path, body):
    req = urllib.request.Request(url + path, json.dumps(body).encode(), {'Content-Type': 'application/json'})
    return json.load(urllib.request.urlopen(req))


post('api/new', {'deck': 'sauron', 'tier': 't1', 'seed': 1, 'seat': 1, 'ai': 'adaptive', 'images': False})
for _ in range(600):                                   # the game deals and waits for your first decision
    st = json.load(urllib.request.urlopen(url + 'api/state'))
    if st['pending']: break
    time.sleep(0.1)
saved = post('api/save', {})
print(json.dumps({'root': commander_sim.ROOT, 'data': commander_sim.DATA, 'saves': server.SAVES,
                  'core': commander_sim.__file__, 'page': '<html' in page.lower(),
                  'decks': sorted(d['key'] for d in opts['decks']), 'tiers': len(opts['tiers']),
                  'decision': bool(st['pending']), 'app': opts['app'], 'saved': sorted(os.listdir(server.SAVES)) if os.path.isdir(server.SAVES) else [],
                  'save_reply': saved}))
srv.shutdown()
'''


class StagedApp(unittest.TestCase):
    def test_the_staged_app_plays_and_saves_from_its_own_copies(self):
        t = tempfile.mkdtemp()
        out, data = os.path.join(t, 'src'), os.path.join(t, 'appdata')
        subprocess.run([sys.executable, os.path.join(IOS, 'stage.py'), '--no-images', '--out', out], check=True,
                       capture_output=True)
        os.makedirs(os.path.join(out, 'commander_ipad'), exist_ok=True)
        for f in os.listdir(os.path.join(IOS, 'src', 'commander_ipad')):
            if f.endswith('.py'):
                with open(os.path.join(IOS, 'src', 'commander_ipad', f), encoding='utf-8') as a, \
                        open(os.path.join(out, 'commander_ipad', f), 'w', encoding='utf-8') as b: b.write(a.read())
        self.assertFalse(os.path.exists(os.path.join(out, 'commander_ipad', 'resources', 'decklists', 'pool', 'retired')))
        r = subprocess.run([sys.executable, '-c', APP, out, data], capture_output=True, text=True, timeout=120,
                           cwd=t, env={k: v for k, v in os.environ.items() if not k.startswith('COMMANDER_SIM_')})
        self.assertEqual(r.returncode, 0, r.stderr[-2000:])
        got = json.loads(r.stdout.strip().splitlines()[-1])
        res = os.path.join(out, 'commander_ipad', 'resources')
        self.assertEqual(got['root'], res)
        self.assertEqual(got['data'], data)
        self.assertEqual(got['saves'], os.path.join(data, 'saves'))
        self.assertTrue(got['core'].startswith(out))                     # the staged copy, not the repository's
        self.assertTrue(got['page'])
        self.assertEqual(got['decks'], ['alela', 'galadriel', 'jodah', 'sauron', 'seph', 'veyran', 'yshtola'])
        self.assertEqual(got['tiers'], 5)
        self.assertTrue(os.path.exists(os.path.join(data, 'scryfall_cache.json')))
        self.assertTrue(got['app'])                                      # the page hides two-player mode
        self.assertTrue(got['decision'])                                 # a game dealt and asked for your first play
        self.assertEqual(sorted(got['saved']), sorted(['autosave.json', got['save_reply']['name']]))   # in the app's folder


if __name__ == '__main__':
    unittest.main()
