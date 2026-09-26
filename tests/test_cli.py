"""The commands, run the way you run them (python3 -m ...), with a handful of games each, plus the statistics they
report. Every run uses the fast heuristic AI; test_pool_games covers the look-ahead."""
import os, subprocess, sys, unittest
from commander_sim import poolmode

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
FAST = ['--ai', 'adaptive', '--profile', 'loose', '--quiet']


def run(*args, ok=True):
    r = subprocess.run([sys.executable, *args], cwd=ROOT, capture_output=True, text=True, timeout=600)
    if ok and r.returncode != 0:
        raise AssertionError(f'{" ".join(args)} exited {r.returncode}:\n{r.stdout[-2000:]}\n{r.stderr[-2000:]}')
    return r


def win_line(out):
    return next(l for l in out.splitlines() if l.startswith('Win rate'))


class Statistics(unittest.TestCase):
    def test_wilson_interval(self):
        lo, hi = poolmode.wilson(50, 100)
        self.assertAlmostEqual(lo, 0.4038, places=3); self.assertAlmostEqual(hi, 0.5962, places=3)
        lo, hi = poolmode.wilson(0, 10)
        self.assertAlmostEqual(lo, 0.0, places=9); self.assertAlmostEqual(hi, 0.2775, places=3)

    def test_paired_delta(self):
        base = {'by_seed': {1: 0, 2: 0, 3: 1, 4: 1}}
        var = {'by_seed': {1: 1, 2: 0, 3: 1, 4: 1}}
        mean, se, n = poolmode.paired_delta(base, var)
        self.assertEqual((mean, n), (0.25, 4))
        self.assertAlmostEqual(se, 0.25)


class Commands(unittest.TestCase):
    def test_one_deck_against_a_tier(self):
        out = run('-m', 'commander_sim', '--deck', 'seph', '--pool', 't1', '--games', '8', *FAST).stdout
        self.assertIn('Sephiroth vs Tier 1', out)
        self.assertIn('eliminated you', out)

    def test_results_do_not_depend_on_the_number_of_workers(self):
        args = ['-m', 'commander_sim', '--deck', 'seph', '--pool', 't2', '--games', '12', *FAST]
        one = win_line(run(*args, '--jobs', '1').stdout)
        three = win_line(run(*args, '--jobs', '3').stdout)
        self.assertEqual(one, three)

    def test_swap(self):
        out = run('-m', 'commander_sim', '--deck', 'seph', '--pool', 't1', '--games', '8', *FAST, '--brief',
                  '--swap', "Blood Artist=>Night's Whisper").stdout
        self.assertIn('VERDICT:', out)

    def test_analyze(self):
        out = run('-m', 'commander_sim', '--deck', 'sauron', '--pool', 't2', '--games', '4', *FAST, '--analyze').stdout
        self.assertIn('Opponent keys:', out)

    def test_trace(self):
        out = run('-m', 'commander_sim', '--deck', 'veyran', '--pool', 't3', '--trace', '0', '--ai', 'adaptive',
                  '--profile', 'loose').stdout
        self.assertIn('Winner:', out)

    def test_calibration(self):
        out = run('-m', 'commander_sim', '--calibrate', 'all', '--games', '4', *FAST).stdout
        self.assertIn('Within-tier balance', out)
        self.assertIn('Tier ordering', out)

    def test_cards(self):
        out = run('-m', 'commander_sim', '--deck', 'veyran', '--cards').stdout
        self.assertIn('unique cards', out)

    def test_no_mode_is_an_error(self):
        r = run('-m', 'commander_sim', ok=False)
        self.assertNotEqual(r.returncode, 0)
        self.assertIn('--pool', r.stderr + r.stdout)

    def test_validate_the_pools(self):
        self.assertIn('all decks passed', run('-m', 'commander_sim.pools', '--validate').stdout)

    def test_audit_of_your_decks(self):
        out = run('-m', 'commander_sim.pool_audit', '--mine').stdout
        self.assertIn('== seph:', out)

    def test_pool_audit_summary(self):
        run('-m', 'commander_sim.pool_audit')

    def test_swaptest_tool(self):
        out = run('-m', 'commander_sim.tools.swaptest', 'heliod-mono-white-stax', 't4', '8',
                  'Sol Ring>Lightning Greaves', '2').stdout
        self.assertIn('as written', out)


if __name__ == '__main__':
    unittest.main()
