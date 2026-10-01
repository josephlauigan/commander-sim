"""Practice mode: the smaller choices are the person's. Mulligans, discards (effects and hand size), sacrifices to
edicts, tutors (only what the tutor may find), basic-land searches, scry and surveil."""
import random, unittest
from tests.table import table, hand, perm, card
from commander_sim import engine as E, ais
from commander_sim.play import choices
from commander_sim.play.controller import ScriptController
from commander_sim.cards.impl import topdeck


def seat(g, p, answers):
    g.controllers = {p.key: ScriptController(answers)}
    return g.controllers[p.key]


class Choices(unittest.TestCase):
    def test_discard_is_yours(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        a, b, c = hand(s, 'Swamp', 'Counterspell', 'Sol Ring')
        seat(g, s, [2])                                     # the third card listed: Sol Ring
        E.discard_worst(g, s, 1)
        self.assertIn(c, s.gy); self.assertEqual(s.hand, [a, b])

    def test_discard_to_hand_size(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        hand(s, *['Swamp'] * 4, *['Island'] * 5)
        ctl = seat(g, s, [0, 0])
        ais.end_step(g, s)
        self.assertEqual(len(s.hand), 7); self.assertEqual(len(ctl.asked), 2)

    def test_edict(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        a = perm(g, s, 'Orcish Bowmasters'); b = perm(g, s, 'Grave Titan')
        seat(g, s, lambda req: next(i for i, x in enumerate(req.choices) if x.startswith('Grave Titan')))
        E.edict(g, s)
        self.assertIn(a, s.perms); self.assertNotIn(b, s.perms)

    def test_tutor_finds_only_its_kind(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        ctl = seat(g, s, [0])
        E.tutor(g, s, 'art')
        self.assertTrue(all(E.DB[x.split(' {')[0]].types.find('A') >= 0 for x in ctl.asked[0].choices[:-1]))
        self.assertEqual(len(s.hand), 1); self.assertIn('A', s.hand[0].types)

    def test_finding_nothing(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        n = len(s.library)
        seat(g, s, ['cancel'])
        E.tutor(g, s, 'any')
        self.assertEqual((len(s.hand), len(s.library)), (0, n))

    def test_basic_land_search(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        ctl = seat(g, s, [0])
        E.land_ramp(g, s, 1, True)
        self.assertEqual(ctl.asked[0].choices[:-1], ['Island', 'Mountain', 'Swamp'])
        self.assertEqual(s.lands[-1].cd.name, 'Island'); self.assertTrue(s.lands[-1].tapped)

    def test_scry(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        top, second = s.library[-1], s.library[-2]
        seat(g, s, [1, 0])                                  # first card to the bottom, second stays
        topdeck.scry(g, s, 2)
        self.assertIs(s.library[0], top); self.assertIs(s.library[-1], second)

    def test_mulligan(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        s.library.extend(s.hand); s.hand = []
        ctl = seat(g, s, ['mulligan', 'mulligan', 'keep', 0])
        choices.mulligan(g, s, random.Random(1))
        self.assertEqual(len(s.hand), 6)                    # two mulligans, the first free: one card to the bottom
        self.assertEqual([r.kind for r in ctl.asked], ['mulligan', 'mulligan', 'mulligan', 'choose'])
        self.assertEqual(len(s.library) + len(s.hand), 99)


    def test_choose_modes(self):
        from commander_sim.cards import dsl
        g = table('sauron', 'veyran'); s, v = g.players
        hand(v, 'Island', 'Mountain')
        kc = card("Kolaghan's Command")
        modal = next(e for a in kc.dsl for e in a.get('effects', []) if e.get('do') == 'modal')
        ctl = seat(g, s, [1, 0])                             # 'a player discards', then (of the rest) the first
        dsl.run(g, s, modal, None, {}, kc, 0)
        self.assertEqual(len(ctl.asked), 2)
        self.assertEqual(len(ctl.asked[1].choices), 3)        # a mode can't be chosen twice
        self.assertEqual(len(v.hand), 1)

    def test_yes_no(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        seat(g, s, [1])
        self.assertFalse(choices.yes_no(g, s, 'Draw?'))

if __name__ == '__main__':
    unittest.main()
