"""Practice mode: your decks' activated abilities by the rules (play/cards.py). The card code offers them to the AI
only when its plan says so, with its own targets; the human seat gets them whenever the card has them, with the
reason when one can't be used, and picks the targets."""
import unittest
from tests.table import table, hand, lands, perm
from commander_sim import engine as E
from commander_sim.play import legal, mana, human
from commander_sim.play.controller import ScriptController


def seat(g, p, answers):
    g.controllers = {p.key: ScriptController(answers)}
    return g.controllers[p.key]


def by_text(word):
    return lambda req: next(i for i, x in enumerate(req.choices) if word in x)


def use(g, p, m, *answers):
    """activate an ability of permanent m in p's main phase, answering the follow-up questions"""
    ctl = seat(g, p, [{'do': 'use', 'perm': p.perms.index(m)}] + list(answers) + [{'do': 'pass'}])
    human.human_main(g, p, False)
    return ctl


class Walkers(unittest.TestCase):
    def test_vraska_minus_two_on_your_pick(self):
        g = table('sauron', 'veyran'); s, v = g.players
        vr = perm(g, s, "Vraska, Betrayal's Sting")
        small = perm(g, v, 'Guttersnipe'); perm(g, v, 'Archmage Emeritus')
        use(g, s, vr, by_text('-2'), by_text('Guttersnipe'))
        self.assertNotIn(small, v.perms)
        self.assertEqual((vr.loyalty, v.treasures), (4, 1))

    def test_once_per_turn_and_sorcery_speed(self):
        g = table('sauron', 'veyran'); s, v = g.players
        vr = perm(g, s, "Vraska, Betrayal's Sting")
        ctl = use(g, s, vr, by_text('0:'), {'do': 'use', 'perm': s.perms.index(vr)}, by_text('0:'))
        self.assertIn('already used a loyalty ability', ctl.told[-1][1])
        g.active = v
        ctl = use(g, s, vr, by_text('0:'))
        self.assertIn("isn't your turn", ctl.told[-1][1])

    def test_ral_zarek_returns_your_pick(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        rz = perm(g, s, 'Ral Zarek, Guest Lecturer')
        s.gy += [E.DB['Orcish Bowmasters'], E.DB['Metallic Mimic']]
        use(g, s, rz, by_text('-2'), by_text('Metallic Mimic'))
        self.assertTrue(any(m.name == 'Metallic Mimic' for m in s.perms))
        self.assertIn(E.DB['Orcish Bowmasters'], s.gy)


class Permanents(unittest.TestCase):
    def test_mind_stone_whenever(self):
        g = table('seph', 'veyran'); s = g.players[0]
        ms = perm(g, s, 'Mind Stone'); mana.pool_of(s).add('C', 1); n = len(s.hand)
        use(g, s, ms, 0)
        self.assertNotIn(ms, s.perms); self.assertEqual(len(s.hand), n + 1)

    def test_mind_stone_needs_mana(self):
        g = table('seph', 'veyran'); s = g.players[0]
        ms = perm(g, s, 'Mind Stone')
        ctl = use(g, s, ms, 0)
        self.assertIn("Can't activate Mind Stone", ctl.told[-1][1]); self.assertIn(ms, s.perms)

    def test_prepared_lightning_bolt(self):
        g = table('veyran', 'seph'); v, s = g.players
        em = perm(g, v, 'Emeritus of Conflict // Lightning Bolt'); em.data = {'prepared': True}
        mana.pool_of(v).add('R', 1)
        use(g, v, em, 0, by_text('Sephiroth'))
        self.assertEqual(s.life, 37); self.assertFalse(em.data['prepared'])

    def test_aetherflux_reservoir(self):
        g = table('veyran', 'seph'); v, s = g.players
        ax = perm(g, v, 'Aetherflux Reservoir'); v.life = 60
        use(g, v, ax, 0, by_text('Sephiroth'))
        self.assertEqual((v.life, s.alive), (10, False))

    def test_aetherflux_needs_fifty_life(self):
        g = table('veyran', 'seph'); v = g.players[0]
        ax = perm(g, v, 'Aetherflux Reservoir')
        ctl = use(g, v, ax, 0)
        self.assertIn('You need 50 life', ctl.told[-1][1])

    def test_triskelion(self):
        g = table('seph', 'veyran'); s, v = g.players
        t = perm(g, s, 'Triskelion'); t.plus = 3
        gs = perm(g, v, 'Kessig Flamebreather')
        use(g, s, t, 0, by_text('Kessig'))
        self.assertEqual(t.plus, 2); self.assertIn(gs, v.perms)          # 1/3: it survives one damage


class Lands(unittest.TestCase):
    def test_strip_mine_any_land(self):
        g = table('seph', 'veyran'); s, v = g.players
        lands(s, 'Strip Mine'); lands(v, 'Island', 2)
        seat(g, s, [{'do': 'use', 'land': 0}, 0, by_text('Island'), {'do': 'pass'}])
        human.human_main(g, s, False)
        self.assertEqual((len(s.lands), len(v.lands)), (0, 1))

    def test_desolate_lighthouse_loots(self):
        g = table('veyran', 'seph'); v = g.players[0]
        lands(v, 'Desolate Lighthouse'); mana.pool_of(v).add('U', 1); mana.pool_of(v).add('R', 2)
        hand(v, 'Counterspell'); n = len(v.hand)
        seat(g, v, [{'do': 'use', 'land': 0}, 0, 0, {'do': 'pass'}])
        human.human_main(g, v, False)
        self.assertEqual((len(v.hand), len(v.gy)), (n, 1)); self.assertTrue(v.lands[0].tapped)


class Citadel(unittest.TestCase):
    def test_cast_the_top_card_for_life(self):
        g = table('seph', 'veyran'); s = g.players[0]
        cit = perm(g, s, "Bolas's Citadel")
        top = E.DB['Mind Stone']; s.library.append(top); life = s.life
        use(g, s, cit, by_text('Mind Stone'))
        self.assertTrue(any(m.name == 'Mind Stone' for m in s.perms))
        self.assertEqual(s.life, life - 2)

    def test_play_the_top_land(self):
        g = table('seph', 'veyran'); s = g.players[0]
        cit = perm(g, s, "Bolas's Citadel")
        s.library.append(E.DB['Command Tower']); n = len(s.lands)
        use(g, s, cit, by_text('play Command Tower'))
        self.assertEqual(len(s.lands), n + 1)

    def test_sacrifice_ten(self):
        g = table('seph', 'veyran'); s, v = g.players
        cit = perm(g, s, "Bolas's Citadel")
        for _ in range(9): E.make_tokens(g, s, 1, 1, sick=False)
        use(g, s, cit, by_text('sacrifice ten'), *[0] * 9)
        self.assertEqual(v.life, 30)
        self.assertNotIn(cit, s.perms)


class Mistrise(unittest.TestCase):
    def test_next_spell_cant_be_countered(self):
        g = table('veyran', 'seph'); v = g.players[0]
        lands(v, 'Mistrise Village'); mana.pool_of(v).add('U', 1)
        seat(g, v, [{'do': 'use', 'land': 0}, 0, {'do': 'pass'}])
        human.human_main(g, v, False)
        c = E.DB['Think Twice']; E.on_cast(g, v, c)
        self.assertTrue(E.mistrised(g, c))
        self.assertFalse(E.mistrised(g, E.DB['Counterspell']))      # only the next spell


class Responding(unittest.TestCase):
    def test_no_sorcery_speed_play_in_response(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        vr = perm(g, s, "Vraska, Betrayal's Sting")
        seat(g, s, [{'do': 'use', 'perm': s.perms.index(vr)}, by_text('0:'), {'do': 'pass'}])
        human.respond(g, s, 'Veyran casts Opt')
        self.assertIn('not in response', g.controllers[s.key].told[-1][1])


if __name__ == '__main__':
    unittest.main()
