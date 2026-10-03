"""Practice mode's Hint: the AI's advice at the decision waiting for you, worked out on a copy of the game, so asking
changes nothing (the same game follows, decision for decision)."""
import unittest
from commander_sim.play.session import Session
from tests.test_play_undo import bot, until_request, frozen


class Hint(unittest.TestCase):
    def run_game(self, hints, n=60):
        bot.played = {}
        s = Session('sauron', 't2', seed=11, ai='lookahead').start()
        seen, got = [], []
        for _ in range(n):
            req = until_request(s)
            if req is None: break
            seen.append(frozen(req))
            if hints and req.kind in ('priority', 'attack'):
                h = s.hint()
                if isinstance(h, dict): got.append((req.kind, req.data['view']['step'], h))
            s.answer(bot(req))
        s.close()
        s.join(60)                  # as the server does: one game at a time touches the engine
        return seen, got

    def test_hints_change_nothing(self):
        with_hints, got = self.run_game(True)
        without, _ = self.run_game(False)
        self.assertEqual(with_hints, without)
        main = [h for k, step, h in got if k == 'priority' and step in ('main1', 'main2')]
        self.assertTrue(main and all(h['text'] for h in main))
        self.assertTrue(any(h['detail'] and h['detail'][0].startswith('Look-ahead') for h in main))

    def test_no_decision_waiting(self):
        s = Session('sauron', 't2', seed=11, ai='adaptive')
        self.assertEqual(s.hint(), 'There is no decision waiting for you.')


if __name__ == '__main__':
    unittest.main()
