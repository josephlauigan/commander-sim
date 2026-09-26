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

    def swap_for(self, path):
        """an instant or sorcery in the list, and a Grixis creature it doesn't run (the list changes over time)"""
        from commander_sim import engine as E
        cards = load(path)
        out = next(n for n in sorted(set(cards)) if n in E.DB and (E.DB[n].instant or E.DB[n].sorcery))
        inn = next(n for n in ('Sheoldred, the Apocalypse', 'Grave Titan', 'Hellkite Tyrant', 'Witch-king, Bringer of Ruin',
                               'Gray Merchant of Asphodel') if n not in cards)
        return out, inn

    def test_a_swap(self):
        import re
        path = self.copy('mine/sauron-grixis-amass.md')
        before = _read(path)
        count = lambda text, label: int(re.search(r'\*\*' + label + r' \((\d+)\)\.\*\*', text).group(1))
        out_card, in_card = self.swap_for(path)
        out, code = self.run_it(path, block(path).replace(f'1 {out_card}\n', f'1 {in_card}\n'), '--log', 'A test.')
        self.assertEqual(code, 0, out)
        cards = load(path)
        self.assertIn(in_card, cards); self.assertNotIn(out_card, cards)
        self.assertEqual(len(cards), 100)
        text = _read(path)
        self.assertEqual(count(text, 'Creatures'), count(before, 'Creatures') + 1)
        self.assertEqual(count(text, 'Instants and sorceries'), count(before, 'Instants and sorceries') - 1)
        self.assertIn(in_card.replace(',', ''), text)               # the by-type lines drop commas in your decks
        self.assertIn(f'Out: {out_card}. In: {in_card}. A test.', text)

    def test_a_bad_list_is_refused_and_nothing_is_written(self):
        path = self.copy('mine/sauron-grixis-amass.md')
        before = _read(path)
        lines = block(path).split('\n')
        singles = [l for l in lines if l.startswith('1 ') and 'Sauron, the Dark Lord' not in l]
        new = '\n'.join(lines).replace(singles[0], singles[1]).replace(singles[2], '1 Swords to Plowshares')
        out, code = self.run_it(path, new)
        self.assertEqual(code, 1)
        self.assertIn('not singleton: ' + singles[1][2:], out)
        self.assertIn('outside colour identity', out)
        self.assertEqual(_read(path), before)

    def test_ninety_nine_cards_are_refused(self):
        path = self.copy('mine/sauron-grixis-amass.md')
        out_card, _ = self.swap_for(path)
        out, code = self.run_it(path, block(path).replace(f'1 {out_card}\n', ''))
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
        out_card, in_card = self.swap_for(path)
        out, code = self.run_it(path, block(path).replace(f'1 {out_card}\n', f'1 {in_card}\n'), '--dry-run')
        self.assertIn('nothing written', out)
        self.assertEqual(_read(path), before)


if __name__ == '__main__':
    unittest.main()
