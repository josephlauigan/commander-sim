"""The deck updater (python3 -m commander_sim.update_deck), run on copies of real deck files."""
import contextlib, io, os, shutil, tempfile, unittest
from commander_sim import ROOT, update_deck
from commander_sim.decks import load
from commander_sim.update_deck import _read

MINE = os.path.join(ROOT, 'decklists', 'mine')
FIX = os.path.join(ROOT, 'tests', 'fixtures', 'my_decks_parsed.json')


def block(path):
    txt = _read(path)
    return txt.split('## Import list')[1].split('```')[1].strip()


class Parse(unittest.TestCase):
    def test_formats_from_deck_sites(self):
        text = """Commander
1 Sauron, the Dark Lord (LTR) 224 *F*

Deck
1x Sol Ring (C21) 263
Arcane Signet
7 Mountain
// a comment
1 Lightning Bolt [M10]

Sideboard
1 Counterspell"""
        self.assertEqual(update_deck.parse_list(text), [(1, 'Sauron, the Dark Lord'), (1, 'Sol Ring'), (1, 'Arcane Signet'),
                                                        (7, 'Mountain'), (1, 'Lightning Bolt')])

    def test_names_match_without_punctuation_or_accents(self):
        rec = {'name': 'Mauhúr, Uruk-hai Captain'}
        self.assertEqual(update_deck.canonical('Mauhur Uruk-hai Captain', rec), 'Mauhúr, Uruk-hai Captain')
        self.assertIsNone(update_deck.canonical('Sol Rng', {'name': 'Sol Ring'}))

    def test_a_two_faced_card_keeps_the_known_name(self):
        rec = {'name': 'Brazen Borrower // Petty Theft'}
        self.assertEqual(update_deck.canonical('Brazen Borrower', rec, known={'Brazen Borrower'}), 'Brazen Borrower')
        self.assertEqual(update_deck.canonical('Brazen Borrower', rec), 'Brazen Borrower // Petty Theft')


class Update(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.mkdtemp()
        self.fixture = _read(FIX)

    def tearDown(self):
        shutil.rmtree(self.tmp)
        self.assertEqual(_read(FIX), self.fixture)          # a copy never touches the real deck guard

    def copy(self, rel):
        dst = os.path.join(self.tmp, rel)
        os.makedirs(os.path.dirname(dst), exist_ok=True)
        shutil.copy(os.path.join(ROOT, 'decklists', rel), dst)
        return dst

    def run_it(self, deck, text, *flags):
        lst = os.path.join(self.tmp, 'list.txt')
        with open(lst, 'w') as fh: fh.write(text)
        out = io.StringIO()
        with contextlib.redirect_stdout(out):
            try:
                update_deck.main([deck, lst, *flags])
            except SystemExit as e:
                return out.getvalue(), e.code
        return out.getvalue(), 0

    def test_the_same_list_changes_nothing(self):
        path = self.copy('mine/sauron-grixis-amass.md')
        before = _read(path)
        out, code = self.run_it(path, block(path))
        self.assertEqual(code, 0); self.assertIn('Nothing to change', out)
        self.assertEqual(_read(path), before)

    def test_a_swap(self):
        path = self.copy('mine/sauron-grixis-amass.md')
        new = block(path).replace('1 Big Score', '1 Sheoldred, the Apocalypse')
        out, code = self.run_it(path, new, '--log', 'Testing Sheoldred.')
        self.assertEqual(code, 0)
        cards = load(path)
        self.assertIn('Sheoldred, the Apocalypse', cards); self.assertNotIn('Big Score', cards)
        self.assertEqual(len(cards), 100)
        text = _read(path)
        self.assertIn('**Creatures (16).**', text)                  # Sheoldred joins the creatures
        self.assertIn('**Instants and sorceries (25).**', text)
        self.assertIn('Sheoldred the Apocalypse', text)             # the by-type lines drop commas in your decks
        self.assertIn('Out: Big Score. In: Sheoldred, the Apocalypse. Testing Sheoldred.', text)
        self.assertIn('still mentions cards that left', out)

    def test_a_bad_list_is_refused_and_nothing_is_written(self):
        path = self.copy('mine/sauron-grixis-amass.md')
        before = _read(path)
        new = block(path).replace('1 Big Score', '1 Lightning Greaves').replace('1 Unearth', '1 Swords to Plowshares')
        out, code = self.run_it(path, new)
        self.assertEqual(code, 1)
        self.assertIn('not singleton: Lightning Greaves', out)
        self.assertIn('outside colour identity', out)
        self.assertEqual(_read(path), before)

    def test_ninety_nine_cards_are_refused(self):
        path = self.copy('mine/sauron-grixis-amass.md')
        out, code = self.run_it(path, block(path).replace('1 Big Score\n', ''))
        self.assertEqual(code, 1); self.assertIn('99 cards', out)

    def test_a_pool_deck_keeps_its_tiers_game_changer_rule(self):
        path = self.copy('pool/t3-high-b3/atraxa-superfriends.md')
        out, code = self.run_it(path, block(path).replace('1 Rhystic Study', '1 Mind Stone'))
        self.assertEqual(code, 1)
        self.assertIn('tier t3 allows 3-3', out)

    def test_a_pool_deck_header_follows_the_lands(self):
        path = self.copy('pool/t3-high-b3/atraxa-superfriends.md')
        new = block(path).replace('4 Plains', '3 Plains') + '\n1 Hinterland Harbor'
        out, code = self.run_it(path, new)
        self.assertEqual(code, 0)
        self.assertIn('- **Lands:** 37 (22 nonbasic + 15 basic)', _read(path))

    def test_dry_run_writes_nothing(self):
        path = self.copy('mine/sauron-grixis-amass.md')
        before = _read(path)
        out, code = self.run_it(path, block(path).replace('1 Big Score', '1 Sheoldred, the Apocalypse'), '--dry-run')
        self.assertIn('nothing written', out)
        self.assertEqual(_read(path), before)


if __name__ == '__main__':
    unittest.main()
