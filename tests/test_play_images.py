"""Practice mode's card images (play/images.py): looked up by name, tokens found through the cards that make them,
downloaded once, two-faced cards with an image per face, and a failed download leaves the card to be drawn by name.
Nothing here touches the network: lookups and downloads are stand-ins."""
import os
import shutil
import tempfile
import unittest
import urllib.error
from commander_sim.play import images


CARDS = {
    'Sol Ring': {'id': 'sol', 'name': 'Sol Ring', 'image_uris': {'normal': 'https://img/sol.jpg'}},
    'Grave Titan': {'id': 'titan', 'name': 'Grave Titan', 'image_uris': {'normal': 'https://img/titan.jpg'},
                    'all_parts': [{'id': 'zombie', 'component': 'token', 'name': 'Zombie'},
                                  {'id': 'titan', 'component': 'combo_piece', 'name': 'Grave Titan'}]},
    'Sanar, Unfinished Genius // Wild Idea': {
        'id': 'sanar', 'name': 'Sanar, Unfinished Genius // Wild Idea',
        'card_faces': [{'name': 'Sanar, Unfinished Genius', 'image_uris': {'normal': 'https://img/sanar0.jpg'}},
                       {'name': 'Wild Idea', 'image_uris': {'normal': 'https://img/sanar1.jpg'}}]},
}
TOKENS = {'zombie': {'id': 'zombie', 'name': 'Zombie', 'image_uris': {'normal': 'https://img/zombie.jpg'}}}


class Images(unittest.TestCase):
    def setUp(self):
        self.dir = tempfile.mkdtemp()
        self.saved = images.DIR, images.INDEX
        images.DIR, images.INDEX = self.dir, os.path.join(self.dir, 'index.json')
        self.lookups, self.downloads = [], []

    def tearDown(self):
        images.DIR, images.INDEX = self.saved
        shutil.rmtree(self.dir)

    def lookup(self, ids):
        self.lookups.append(ids)
        out = []
        for i in ids:
            if 'id' in i: out.append(TOKENS[i['id']])
            else: out += [c for n, c in CARDS.items() if n.split(' // ')[0] == i['name']]
        return out

    def download(self, url, path, fail=()):
        self.downloads.append(url)
        if url in fail: raise urllib.error.URLError('offline')
        with open(path, 'wb') as f: f.write(b'jpg')

    def test_cards_tokens_and_faces(self):
        seen = []
        got = images.prepare(list(CARDS), lambda d, t: seen.append((d, t)), self.lookup, self.download)
        self.assertEqual(got['Sol Ring'], ['sol.jpg'])
        self.assertEqual(got['Zombie'], ['zombie.jpg'])                              # the token Grave Titan makes
        self.assertEqual(got['Sanar, Unfinished Genius // Wild Idea'], ['sanar.jpg', 'sanar-1.jpg'])
        self.assertEqual(seen[0], (0, 5)); self.assertEqual(seen[-1], (5, 5))

    def test_cached_the_second_time(self):
        images.prepare(list(CARDS), None, self.lookup, self.download)
        self.lookups, self.downloads = [], []
        got = images.prepare(list(CARDS), None, self.lookup, self.download)
        self.assertEqual((self.lookups, self.downloads), ([], []))
        self.assertEqual(len(got), 4)

    def test_a_failed_download_is_left_out(self):
        got = images.prepare(['Sol Ring', 'Grave Titan'], None, self.lookup,
                             lambda u, p: self.download(u, p, fail=('https://img/titan.jpg',)))
        self.assertNotIn('Grave Titan', got); self.assertIn('Sol Ring', got)

    def test_offline_lookup(self):
        got = images.prepare(['Sol Ring'], None, lambda ids: [], self.download)
        self.assertEqual(got, {})

    def test_game_names(self):
        self.assertEqual(images.game_names([('a', ['Sol Ring', 'Island', 'Island'], 'Grave Titan'), ('b', ['Sol Ring'], 'X')]),
                         ['Sol Ring', 'Island', 'Grave Titan', 'X'])


if __name__ == '__main__':
    unittest.main()
