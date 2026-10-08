"""Practice mode: casts from hand that the card code keeps for the AI (play/cards.py HAND), copying spells with
Return the Favor, and new targets for copies. The AI never casts these for the human seat."""
import unittest
from tests.table import table, hand, perm
from commander_sim import engine as E
from commander_sim.play import mana, human
from commander_sim.play.controller import ScriptController


def seat(g, p, answers):
    g.controllers = {p.key: ScriptController(answers)}
    return g.controllers[p.key]


def by_text(word):
    return lambda req: next(i for i, x in enumerate(req.choices) if word in x)


def pool(p, **kw):
    for c, n in kw.items(): mana.pool_of(p).add(c, n)


def main(g, p, *answers):
    ctl = seat(g, p, list(answers) + [{'do': 'pass'}])
    human.human_main(g, p, False)
    return ctl


class FromHand(unittest.TestCase):
    def test_twinflame_two_targets(self):
        g = table('veyran', 'seph'); v = g.players[0]
        hand(v, 'Twinflame'); perm(g, v, 'Guttersnipe'); perm(g, v, 'Kessig Flamebreather')
        pool(v, R=2, C=3)
        main(g, v, {'do': 'cast', 'card': 0}, by_text('Guttersnipe'), by_text('Kessig'))
        toks = [m for m in v.perms if m.token]
        self.assertEqual(sorted(m.name for m in toks), ['Guttersnipe', 'Kessig Flamebreather'])
        self.assertTrue(all(not m.sick for m in toks)); self.assertEqual(mana.pool_of(v).total(), 0)

    def test_twinflame_extra_target_needs_mana(self):
        g = table('veyran', 'seph'); v = g.players[0]
        c = hand(v, 'Twinflame'); perm(g, v, 'Guttersnipe'); perm(g, v, 'Kessig Flamebreather')
        pool(v, R=1, C=1)
        ctl = main(g, v, {'do': 'cast', 'card': 0}, by_text('Guttersnipe'), by_text('Kessig'))
        self.assertIn("Can't cast Twinflame on 2", ctl.told[-1][1]); self.assertIn(c, v.hand)

    def test_twinflame_copies_a_token(self):
        g = table('veyran', 'seph'); v = g.players[0]
        hand(v, 'Twinflame'); E.make_tokens(g, v, 1, 4, fly=True, sick=False); pool(v, R=1, C=1)
        main(g, v, {'do': 'cast', 'card': 0}, 0)
        self.assertEqual(sum(1 for m in v.perms if m.token and m.pow == 4), 2)

    def test_ephemerate_and_rebound(self):
        g = table('seph', 'veyran'); s = g.players[0]
        hand(s, 'Ephemerate'); m = perm(g, s, 'Grave Titan'); pool(s, W=1)
        main(g, s, {'do': 'cast', 'card': 0}, by_text('Grave Titan'))
        self.assertNotIn(m, s.perms)                                   # it left and came back as a new object
        self.assertEqual(sum(1 for x in s.perms if x.token), 4)          # Grave Titan's Zombies: two more
        c = E.DB['Ephemerate']; self.assertIn(c, s.exile)
        seat(g, s, ['cancel'])
        E.CI.HOOKS['Ephemerate']['rebound'](g, s, c)
        self.assertIn(c, s.exile)                                      # not cast: stays in exile

    def test_cycling_at_instant_speed(self):
        g = table('seph', 'veyran'); s, v = g.players
        g.active = v
        hand(s, 'Unearth'); pool(s, C=2); n = len(s.hand)
        seat(g, s, [{'do': 'cast', 'card': 0}, by_text('cycling'), {'do': 'pass'}])
        human.respond(g, s, 'End of turn')
        self.assertEqual(len(s.hand), n); self.assertIn(E.DB['Unearth'], s.gy)

    def test_unearth_your_pick(self):
        g = table('seph', 'veyran'); s = g.players[0]
        hand(s, 'Unearth'); s.gy += [E.DB['Carrion Feeder'], E.DB['Burglar Rat']]; pool(s, B=1)
        main(g, s, {'do': 'cast', 'card': 0}, by_text('cast it'), by_text('Burglar Rat'))
        self.assertTrue(any(m.name == 'Burglar Rat' for m in s.perms))

    def test_sokenzan_channel(self):
        g = table('veyran', 'seph'); v = g.players[0]
        hand(v, 'Sokenzan, Crucible of Defiance'); pool(v, R=1, C=3)
        main(g, v, {'do': 'cast', 'card': 0})
        self.assertEqual(sum(1 for m in v.perms if m.token and not m.sick), 2)

    def test_necromancy_at_instant_speed(self):
        g = table('seph', 'veyran'); s, v = g.players
        g.active = v
        hand(s, 'Necromancy'); s.gy.append(E.DB['Grave Titan']); pool(s, B=1, C=2)
        seat(g, s, [{'do': 'cast', 'card': 0}, by_text('Grave Titan'), {'do': 'pass'}])
        human.respond(g, s, 'End of turn')
        self.assertFalse(any(m.name == 'Grave Titan' for m in s.perms))      # sacrificed: cast as though it had flash
        self.assertEqual(sum(1 for m in s.perms if m.token), 2)          # its enter trigger happened


class Copies(unittest.TestCase):
    def test_return_the_favor_needs_a_spell(self):
        g = table('veyran', 'seph'); v = g.players[0]
        hand(v, 'Return the Favor'); pool(v, R=3)
        ctl = main(g, v, {'do': 'cast', 'card': 0})
        self.assertIn('in response', ctl.told[-1][1])

    def test_return_the_favor_copies_your_bolt(self):
        g = table('veyran', 'seph'); v, s = g.players
        hand(v, 'Lightning Bolt', 'Return the Favor'); pool(v, R=4)
        main(g, v, {'do': 'cast', 'card': 0}, by_text('Sephiroth'),           # Bolt at Sephiroth
             {'do': 'cast', 'card': 0}, by_text('copy'), by_text('keep'),      # then, holding priority: copy it
             {'do': 'pass'})
        self.assertEqual(s.life, 34)

    def test_the_ai_does_not_cast_return_the_favor_for_you(self):
        g = table('veyran', 'seph'); v, s = g.players
        hand(v, 'Lightning Bolt', 'Return the Favor'); pool(v, R=4)
        main(g, v, {'do': 'cast', 'card': 0}, by_text('Sephiroth'), {'do': 'pass'})
        self.assertEqual(s.life, 37); self.assertTrue(any(c.name == 'Return the Favor' for c in v.hand))


if __name__ == '__main__':
    unittest.main()
