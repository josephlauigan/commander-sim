"""Practice mode's Undo: the session keeps your answers (and the look-ahead AI's decisions on a tape); Undo restarts
the game from its seed, replays all but your last answers, and asks that decision again, with the table exactly as it
was."""
import json
import queue
import unittest
from commander_sim.play.session import Session
from tests.test_play import default


def bot(req):
    """a simple player: land, tap, cast what it can, attack with everything"""
    if req.kind != 'priority': return default(req)
    me = next(p for p in req.data['view']['players'] if p['you'])
    if req.data['view']['active'] == me['key'] and req.data['view']['step'] == 'main1':
        lands = [i for i, x in enumerate(me['hand_land']) if x] if 'hand_land' in me else []
        if lands and not bot.played.get(req.data['view']['round']):
            bot.played[req.data['view']['round']] = True
            return {'do': 'land', 'card': lands[0]}
        if me['mana_sources']: return {'do': 'tap', 'source': me['mana_sources'][0]['id']}
    return {'do': 'pass'}


def until_request(s, count=None):
    """events up to the next request (None at the end of the game)"""
    while True:
        ev = s.events.get(timeout=300)
        if ev['kind'] == 'request': return ev['request']
        if ev['kind'] in ('over', 'error'): return None


def frozen(req):
    return json.dumps({'kind': req.kind, 'prompt': req.prompt, 'choices': req.choices, 'view': req.data.get('view')},
                      sort_keys=True, default=str)


class Undo(unittest.TestCase):
    def play(self, ai, decisions):
        bot.played = {}
        s = Session('sauron', 't2', seed=11, ai=ai).start()
        seen = []
        for _ in range(decisions):
            req = until_request(s)
            self.assertIsNotNone(req)
            seen.append(frozen(req))
            s.answer(bot(req))
        until_request(s)                                     # the next decision is waiting
        return s, seen

    def test_undo_asks_the_last_decision_again_with_the_same_table(self):
        s, seen = self.play('adaptive', 40)
        self.assertIsNone(s.undo(3))
        ev = None
        while True:
            ev = s.events.get(timeout=300)
            if ev['kind'] == 'request': break
        self.assertEqual(frozen(ev['request']), seen[40 - 3])
        self.assertEqual(len(s.answers), 40 - 3)
        s.close()

    def test_replayed_history_is_marked(self):
        s, _ = self.play('adaptive', 15)
        s.undo(1)
        evs = []
        while True:
            ev = s.events.get(timeout=300); evs.append(ev)
            if ev['kind'] == 'request': break
        self.assertEqual(evs[0]['kind'], 'reset')
        self.assertTrue(all(e.get('replay') for e in evs[1:-1] if e['kind'] == 'log'))
        self.assertNotIn('replay', evs[-1])
        s.close()

    def test_nothing_to_undo(self):
        s = Session('sauron', 't2', seed=11, ai='adaptive').start()
        until_request(s)
        self.assertEqual(s.undo(), 'Nothing to undo yet.')
        s.close()


if __name__ == '__main__':
    unittest.main()
