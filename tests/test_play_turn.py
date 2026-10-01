"""Practice mode: the human seat's main phase. Lands, casting from hand and the command zone, the rules check's
reasons, and whole games played through the human seat by a simple bot (so every card goes through that path)."""
import unittest
from tests.table import table, hand, lands, perm
from commander_sim import engine as E
from commander_sim.play import legal, mana, human
from commander_sim.play.controller import ScriptController, Cancelled
from commander_sim.play.session import Session


def seat(g, p, answers):
    g.controllers = {p.key: ScriptController(answers)}
    return g.controllers[p.key]


def tap_all(g, p):
    while mana.sources(g, p): mana.tap(g, p, mana.sources(g, p)[0]['id'])


class RulesCheck(unittest.TestCase):
    def test_one_land_per_turn(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        a, b = hand(s, 'Swamp', 'Island')
        self.assertIsNone(legal.check_land(g, s, a))
        self.assertIsNone(human.apply(g, s, {'do': 'land', 'card': 0}))
        self.assertEqual(legal.check_land(g, s, b), "You've already played a land this turn.")

    def test_lands_and_sorceries_only_in_your_main_phase(self):
        g = table('sauron', 'veyran'); s, v = g.players
        c = hand(s, 'Swamp'); sheo = hand(s, 'Sheoldred, the Apocalypse')
        g.step = 'combat'
        self.assertEqual(legal.check_land(g, s, c), 'You can only play a land in one of your main phases.')
        g.active = v; g.step = 'main1'
        self.assertEqual(legal.check_cast(g, s, sheo), "It isn't your turn.")

    def test_instants_any_time_you_have_priority(self):
        g = table('sauron', 'veyran'); s, v = g.players
        cs = hand(s, 'Infernal Grasp'); lands(s, 'Swamp', 2); perm(g, v, 'Guttersnipe')
        g.active = v; tap_all(g, s)
        self.assertIsNone(legal.check_cast(g, s, cs))

    def test_the_reason_when_mana_is_short(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        sheo = hand(s, 'Sheoldred, the Apocalypse'); lands(s, 'Swamp', 2); tap_all(g, s)
        self.assertEqual(legal.check_cast(g, s, sheo), "Can't cast Sheoldred, the Apocalypse. It costs {2}{B}{B}; "
                                                        "your mana pool has {B}{B}. Missing {2}.")

    def test_commander_tax(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        s.tax = 2; lands(s, 'Island', 2); lands(s, 'Swamp', 2); lands(s, 'Mountain', 2); tap_all(g, s)
        self.assertIn('Missing {2}', legal.check_cast(g, s, s.cmd, 'cmd'))     # {3}{U}{B}{R} + 2 tax from 6 mana
        lands(s, 'Swamp', 2); tap_all(g, s)
        self.assertIsNone(legal.check_cast(g, s, s.cmd, 'cmd'))


class MainPhase(unittest.TestCase):
    def test_tap_then_cast(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        hand(s, 'Orcish Bowmasters'); lands(s, 'Swamp', 2)
        ctl = seat(g, s, [{'do': 'tap', 'source': 0}, {'do': 'tap', 'source': 0}, {'do': 'cast', 'card': 0},
                          lambda req: next(i for i, x in enumerate(req.choices) if 'player' in x and 'you' not in x),
                          {'do': 'pass'}])                       # Bowmasters' 1 damage: at the opponent
        human.human_main(g, s, False)
        self.assertTrue(any(m.name == 'Orcish Bowmasters' for m in s.perms))
        self.assertEqual(mana.pool_of(s).total(), 0)
        self.assertEqual(ctl.told, [])

    def test_an_illegal_move_is_explained_and_you_keep_priority(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        hand(s, 'Sheoldred, the Apocalypse')
        ctl = seat(g, s, [{'do': 'cast', 'card': 0}, {'do': 'pass'}])
        human.human_main(g, s, False)
        self.assertEqual(len(ctl.asked), 2)
        self.assertEqual(ctl.told[0][0], 'invalid')
        self.assertIn('Missing {2}{B}{B}', ctl.told[0][1])
        self.assertIn(E.DB['Sheoldred, the Apocalypse'], s.hand)

    def test_casting_the_commander_raises_the_tax(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        for n in ('Island', 'Swamp', 'Mountain', 'Island', 'Swamp', 'Mountain'): lands(s, n, 1)
        seat(g, s, [{'do': 'tap', 'source': 0}] * 6 + [{'do': 'cast', 'zone': 'cmd'}, {'do': 'pass'}])
        human.human_main(g, s, False)
        self.assertTrue(any(m.cd is s.cmd for m in s.perms))
        self.assertEqual(s.tax, 2); self.assertFalse(s.cmd_in_zone)


def bot_for(sess):
    """plays the human seat through the same actions a person would send: land, tap everything, cast what's legal"""
    def answer(req):
        if req.kind in ('target', 'choose', 'block', 'search'): return 0
        if req.kind == 'mulligan': return 'keep'
        if req.kind == 'attack': return list(range(len(req.choices)))
        if req.kind != 'priority': return True
        g = sess.game; p = next(x for x in g.players if x.key == sess.deck)
        for i, c in enumerate(p.hand):
            if c.land and legal.check_land(g, p, c) is None: return {'do': 'land', 'card': i}
        srcs = mana.sources(g, p)
        if srcs: return {'do': 'tap', 'source': srcs[0]['id']}
        for i, c in enumerate(p.hand):
            if not c.land and legal.check_cast(g, p, c) is None: return {'do': 'cast', 'card': i}
        if p.cmd_in_zone and legal.check_cast(g, p, p.cmd, 'cmd') is None: return {'do': 'cast', 'zone': 'cmd'}
        tried = sess.__dict__.setdefault('bot_tried', set())   # equip each piece at most once a turn
        for i, m in enumerate(p.perms):
            if g.active is p and legal.equip_cost(m) is not None and m.attached is None and (g.round, id(m)) not in tried:
                tried.add((g.round, id(m)))
                return {'do': 'use', 'perm': i}
        return {'do': 'pass'}
    return answer


def bot_game(deck, tier, seed):
    s = Session(deck, tier, seed=seed, ai='adaptive')
    s.human = ScriptController(bot_for(s))
    s._run()
    evs = []
    while not s.events.empty(): evs.append(s.events.get())
    return s, evs


class BotGames(unittest.TestCase):
    def test_every_deck_plays_through_the_human_seat(self):
        for deck, tier, seed in (('sauron', 't2', 1), ('sauron', 't4', 2), ('seph', 't3', 1), ('veyran', 't1', 1),
                                 ('marchesa', 't2', 1)):
            s, evs = bot_game(deck, tier, seed)
            self.assertEqual(evs[-1]['kind'], 'over', f"{deck} {tier} {seed}: {evs[-1].get('text', '')[-1500:]}")
            p = next(x for x in s.game.players if x.key == deck)
            self.assertGreater(p.stats['spells_cast'], 3, f'{deck} {tier} {seed}')
            bad = [t for k, t in s.human.told if k == 'invalid' and not t.startswith(("Can't equip", 'You have no creature',
                                                                                  "Can't cast Twinflame on", "Can't cast Disembowel with"))]
            self.assertEqual(bad, [], f'{deck} {tier} {seed}')


if __name__ == '__main__':
    unittest.main()
