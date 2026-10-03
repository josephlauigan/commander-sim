"""Practice mode's review: once the game ends, a summary (result, decisions, matches with the AI, clearly worse ones)
and every compared decision; "Try it" goes back to a decision with the AI's choice in place of yours."""
import unittest
from commander_sim.play.session import Session
from tests.test_play_undo import bot, until_request, frozen


def play_out(s, limit=3000):
    """answer every decision with the bot until the game ends; returns the requests seen"""
    seen = []
    for _ in range(limit):
        req = until_request(s)
        if req is None: return seen
        seen.append(frozen(req)); s.answer(bot(req))
    raise AssertionError('the game did not end')


class Review(unittest.TestCase):
    def test_review_and_try_it(self):
        bot.played = {}
        s = Session('sauron', 't2', seed=11, ai='adaptive', compare=True, max_rounds=8).start()
        play_out(s)
        r = s.review()
        self.assertIsInstance(r, dict)
        summ = r['summary']
        self.assertGreater(summ['decisions'], 0); self.assertLessEqual(summ['matched'], summ['compared'])
        diff = next(d for d in r['decisions'] if d['can_try'] and d['differs'] and not d['ai'].startswith('stop'))
        self.assertIsNone(s.try_it(diff['n']))
        evs = []
        while True:
            ev = s.events.get(timeout=300); evs.append(ev)
            if ev['kind'] in ('request', 'over', 'error'): break
        self.assertEqual(evs[0]['kind'], 'reset'); self.assertEqual(evs[0]['tryit']['n'], diff['n'])
        self.assertEqual(len(s.answers) > diff['n'], evs[0]['tryit']['applied'])    # the AI's choice was answered for you
        s.close(); s.join(60)

    def test_the_same_seed_again(self):
        a = Session('veyran', 't3', seed=21, ai='adaptive')
        b = Session('veyran', 't3', seed=21, ai='adaptive', seats=a.seats)
        self.assertEqual(a.seats, b.seats)
        with self.assertRaises(ValueError): Session('veyran', 't3', seed=21, seats=a.seats[:3])

    def test_try_it_needs_a_compared_decision(self):
        s = Session('sauron', 't2', seed=11, ai='adaptive', compare=True)
        self.assertIn('no AI choice', s.try_it(5))


if __name__ == '__main__':
    unittest.main()
