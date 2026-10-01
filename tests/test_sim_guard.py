"""Guard for the simulations: seeded games must play out exactly as recorded.

Practice mode (commander_sim/play) adds hooks to the engine that must do nothing in a simulation. This plays a fixed
set of seeded games (heuristic AI, which does not depend on memory layout) and compares winner, length, how the game
ended and every life total with the recording.

Regenerate only after an intentional change to how games play (a card fix, an AI change, a deck list change):
    python3 tests/test_sim_guard.py --record
"""
import json, os, sys, unittest
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.dirname(HERE))

FIX = os.path.join(HERE, 'fixtures', 'sim_guard.json')
DECKS = ('seph', 'veyran', 'sauron', 'marchesa')
TIERS = ('t2', 't4')
SEEDS = (500000, 500001, 500002)


def fingerprints():
    from commander_sim import poolmode
    poolmode._setup('conservative', 'adaptive', 1.0)          # the same settings tests/table.py uses
    out = {}
    for tier in TIERS:
        keys = poolmode.pool_keys(tier)
        for deck in DECKS:
            for seed in SEEDS:
                g = poolmode.play(seed, deck, None, keys)
                out[f'{deck} {tier} {seed}'] = [g.winner.key if g.winner else None, g.round, getattr(g, 'wintype', ''),
                                                [[p.key, p.life] for p in g.players]]
    return out


class SimGuard(unittest.TestCase):
    def test_seeded_games_unchanged(self):
        with open(FIX) as fh: want = json.load(fh)
        got = fingerprints()
        changed = [k for k in want if got.get(k) != want[k]]
        self.assertEqual(changed, [], 'these seeded games now play differently; if intended, re-record '
                                      '(python3 tests/test_sim_guard.py --record)')


if __name__ == '__main__':
    if '--record' in sys.argv:
        with open(FIX, 'w') as fh: json.dump(fingerprints(), fh, indent=0)
        print('recorded')
    else:
        unittest.main()
