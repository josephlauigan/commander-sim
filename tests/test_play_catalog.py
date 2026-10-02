"""Practice mode's setup screen data (play/catalog.py): your decks with commander, bracket, Game Changers and
simulation win rates, and the five tiers with their decks."""
import os
import tempfile
import unittest
from commander_sim.play import catalog


class Catalog(unittest.TestCase):
    def test_your_decks(self):
        c = catalog.catalog()
        d = {x['key']: x for x in c['decks']}
        self.assertEqual(set(d), {'seph', 'veyran', 'sauron', 'marchesa'})
        self.assertEqual(d['seph']['name'], 'Sephiroth, the Savior')          # the deck file's name for Atraxa
        self.assertEqual(d['seph']['commander'], 'Atraxa, Grand Unifier')     # the card the engine (and image) uses
        self.assertEqual(d['seph']['bracket'], 4); self.assertEqual(d['veyran']['bracket'], 3)
        self.assertGreater(len(d['seph']['game_changers']), 3)

    def test_tiers(self):
        tiers = catalog.catalog()['tiers']
        self.assertEqual([t['key'] for t in tiers], ['t1', 't2', 't3', 't4', 't5'])
        self.assertTrue(all(len(t['decks']) == 5 for t in tiers))
        self.assertEqual(tiers[0]['label'], 'T1 · High B2 / Low B3')

    def test_win_rates_from_the_results_table(self):
        md = ('# r\n\n### 0d. Your decks\n\n| Deck | T1 | T2 | T3 | T4 | T5 |\n|---|---|---|---|---|---|\n'
              '| Sauron | 24.2% (19-30) | **27.5%** (22-34) | 21.2% | 24.2% | 17.5% |\n| Other | 1% | 1% | 1% | 1% | 1% |\n\n## 1. Next\n')
        path = os.path.join(tempfile.mkdtemp(), 'r.md')
        with open(path, 'w') as f: f.write(md)
        self.assertEqual(catalog.win_rates(path), {'sauron': {'t1': 24.2, 't2': 27.5, 't3': 21.2, 't4': 24.2, 't5': 17.5}})
        self.assertEqual(catalog.win_rates(path + '.missing'), {})


if __name__ == '__main__':
    unittest.main()
