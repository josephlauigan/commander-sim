"""Practice mode's AI comparison log (play/shadow.py): your main-phase plays and attacks are copied before they apply,
compared with the AI's choice by look-ahead playouts while the engine waits for you, and finished before the review.
The game itself is not changed by any of it."""
import unittest
from commander_sim.play.session import Session
from tests.test_play_undo import bot, until_request, frozen


def run(compare, n=70, undo_at=None):
    bot.played = {}
    s = Session('sauron', 't2', seed=11, ai='lookahead', compare=compare).start()
    seen = []
    for k in range(n):
        req = until_request(s)
        if req is None: break
        seen.append(frozen(req))
        s.answer(bot(req))
    return s, seen


class Shadow(unittest.TestCase):
    def test_comparisons_change_nothing_and_finish(self):
        s, with_cmp = run(True)
        self.assertIsInstance(s.review(), str)                      # hidden until the game ends
        with s.hint_lock:                # the first game does no background work while the second one plays
            t, without = run(False)
            t.close(); t.join(30)
        self.assertEqual(with_cmp, without)                         # the game is the same with the comparison on
        entries = s.shadow.entries
        self.assertTrue(any(e.kind == 'main' for e in entries))
        with s.hint_lock: s.shadow.finish()                         # (the lock keeps the game's own thread out)
        done = [e for e in s.shadow.entries if e.scored]
        self.assertTrue(done)
        for e in done:
            self.assertIn(e.ai, e.scores); self.assertIn(e.yours_key, e.scores)
        s.close(); s.join(30)

    def test_undo_forgets_later_comparisons(self):
        s, _ = run(True, 40)
        k = len(s.answers)
        s.undo(10)
        self.assertTrue(all(e.n < k - 10 for e in s.shadow.entries))
        s.close(); s.join(30)


if __name__ == '__main__':
    unittest.main()
