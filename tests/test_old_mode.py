"""Regression guard for the original four-deck mode.

The fixtures were recorded before the opponent-pool work. The engine is deterministic for a given
seed, so any change meant to preserve old-mode behaviour must reproduce them exactly.
Regenerate (only after an intentional change to old-mode play):  python3 tests/test_old_mode.py --record
"""
import json, os, sys, unittest
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.dirname(HERE))
import engine, ais, compare
from decks import DECKS

FIX = os.path.join(HERE, 'fixtures')
SEEDS = range(500000, 500500)


def parsed_lists():
    return {k: sorted(v) for k, v in DECKS.items()}


def fingerprint(profile):
    compare.set_ai('adaptive', 1.0); engine.set_profile(profile)
    out = []
    for s in SEEDS:
        g = ais.play_game(s, DECKS)
        out.append([s, g.winner.key if g.winner else None, g.wintype, g.round,
                    [[p.key, p.life, p.alive, p.turns] for p in g.players]])
    return out


class OldMode(unittest.TestCase):
    def test_my_decks_parse_unchanged(self):
        self.assertEqual(parsed_lists(), json.load(open(os.path.join(FIX, 'my_decks_parsed.json'))))

    def test_games_unchanged(self):
        for prof in ('conservative', 'loose'):
            with self.subTest(profile=prof):
                want = json.load(open(os.path.join(FIX, f'old_mode_{prof}.json')))
                self.assertEqual(fingerprint(prof), want)


if __name__ == '__main__':
    if '--record' in sys.argv:
        json.dump(parsed_lists(), open(os.path.join(FIX, 'my_decks_parsed.json'), 'w'), indent=0)
        for prof in ('conservative', 'loose'):
            json.dump(fingerprint(prof), open(os.path.join(FIX, f'old_mode_{prof}.json'), 'w'))
        print('recorded')
    else:
        unittest.main()
