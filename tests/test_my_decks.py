"""Guard for your three deck files: they must parse to exactly the recorded lists (the sim never edits them).

Regenerate only after an intentional change to a deck file:  python3 tests/test_my_decks.py --record
"""
import json, os, sys, unittest
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.dirname(HERE))
from commander_sim.decks import DECKS

FIX = os.path.join(HERE, 'fixtures', 'my_decks_parsed.json')


def parsed_lists():
    return {k: sorted(v) for k, v in DECKS.items()}


class MyDecks(unittest.TestCase):
    def test_my_decks_parse_unchanged(self):
        with open(FIX) as fh: self.assertEqual(parsed_lists(), json.load(fh))


if __name__ == '__main__':
    if '--record' in sys.argv:
        json.dump(parsed_lists(), open(FIX, 'w'), indent=0)
        print('recorded')
    else:
        unittest.main()
