"""Practice mode's saved games: the seed, the table, your answers, the AI's look-ahead decisions and the comparison;
loading one replays the same game to where it was saved, decision for decision."""
import json
import unittest
from commander_sim.play.session import Session
from tests.test_play_undo import bot, until_request, frozen


class SaveLoad(unittest.TestCase):
    def test_a_saved_game_comes_back_the_same(self):
        bot.played = {}
        s = Session('marchesa', 't2', seed=500000, ai='lookahead', compare=True).start()
        for _ in range(50):
            req = until_request(s); s.answer(bot(req))
        req = until_request(s)
        data = json.loads(json.dumps(s.saved()))                        # through JSON, as on disk
        here = frozen(req)
        s.close()
        t = Session.load(data).start()
        again = until_request(t)
        self.assertEqual(frozen(again), here)
        self.assertEqual(len(t.answers), 50)
        self.assertEqual(len(t.shadow.entries), len([e for e in data['shadow']]))
        t.close()

    def test_wrong_version(self):
        with self.assertRaises(ValueError): Session.load({'version': 0})


if __name__ == '__main__':
    unittest.main()
