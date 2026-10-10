"""The card export for the Rust engine (commander_sim/tools/export_cards.py): every card definition comes out with the
fields the engine has, the export is deterministic, and the committed data/cards.json and data/decks.json are up to
date with the code and the card cache.

After an intended change to cards or decks, re-export:  python3 -m commander_sim.tools.export_cards
"""
import json, os, subprocess, sys, tempfile, unittest
from commander_sim import DATA


def export(out):
    """the export in a fresh process, as the command line runs it: other tests load cards outside the decks into
    engine.DB, which would otherwise change what is exported"""
    root = os.path.dirname(DATA)
    subprocess.run([sys.executable, '-m', 'commander_sim.tools.export_cards', '--out', out], cwd=root, check=True,
                   capture_output=True)


class ExportCards(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory()
        export(cls.tmp.name)
        with open(os.path.join(cls.tmp.name, 'cards.json')) as fh: cls.cards = json.load(fh)['cards']
        with open(os.path.join(cls.tmp.name, 'decks.json')) as fh: cls.decks = json.load(fh)

    @classmethod
    def tearDownClass(cls):
        cls.tmp.cleanup()

    def test_every_card_matches_the_engine(self):
        from commander_sim import engine as E
        from commander_sim.tools import export_cards as X
        X.load_everything()
        self.assertEqual([c['name'] for c in self.cards], sorted(c['name'] for c in self.cards))
        for c in self.cards:
            cd = E.DB[c['name']]
            self.assertEqual((c['types'], c['generic'], c['pips'], c['tags'], c['pow'], c['tgh'], c['bomb']),
                             (cd.types, cd.generic, cd.pips, cd.tags, cd.pow, cd.tgh, cd.bomb), c['name'])
            self.assertEqual(c['dsl'], cd.dsl, c['name'])
            self.assertEqual((set(c['kws']), set(c['subtypes']), c['protfrom'], c['ward'], c['start_loyalty']),
                             (set(cd.kws), set(cd.subtypes), cd.protfrom, cd.ward, cd.start_loyalty), c['name'])
            self.assertEqual(c['derived'], {'cmc': cd.cmc, 'land': cd.land, 'creature': cd.creature,
                                            'instant': cd.instant, 'sorcery': cd.sorcery, 'perm': cd.perm})

    def test_every_deck_card_is_exported(self):
        names = {c['name'] for c in self.cards}
        for d in self.decks['mine'] + self.decks['pool']:
            missing = [n for n in d['cards'] + [d['commander']] if n not in names]
            self.assertEqual(missing, [], d['key'])
        self.assertEqual(len(self.decks['pool']), 25)

    def test_deterministic_and_committed_files_up_to_date(self):
        with tempfile.TemporaryDirectory() as t2:
            export(t2)
            for f in ('cards.json', 'decks.json'):
                with open(os.path.join(self.tmp.name, f)) as a, open(os.path.join(t2, f)) as b:
                    fresh = a.read()
                    self.assertEqual(fresh, b.read(), f'{f}: two exports differ')
                with open(os.path.join(DATA, f)) as fh:
                    self.assertEqual(fh.read(), fresh, f'data/{f} is out of date: python3 -m '
                                                       'commander_sim.tools.export_cards')


if __name__ == '__main__':
    unittest.main()
