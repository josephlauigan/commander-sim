"""Practice mode, Sauron's cards: the choices the AI used to make for them are yours. Damage triggers (Orcish
Bowmasters, Kaervek, Sword of Fire and Ice) ask for a target; the Ring asks for its Ring-bearer, Call of the Ring
and Sauron ask whether to use their "may" abilities."""
import unittest
from tests.table import table, hand, perm
from commander_sim import engine as E, ais
from commander_sim.play import legal
from commander_sim.play.controller import ScriptController


def mine():
    from commander_sim.cards.impl import mine as m        # imported after setup (an early import changes card data)
    return m


def seat(g, p, answers):
    g.controllers = {p.key: ScriptController(answers)}
    return g.controllers[p.key]


def pick(g, p, x):
    """an answer choosing x, whatever position it is listed in"""
    return lambda req: req.choices.index(legal.describe_target(g, p, x))


class Damage(unittest.TestCase):
    def test_bowmasters_enters(self):
        g = table('sauron', 'veyran'); s, v = g.players
        ctl = seat(g, s, pick(g, s, v))
        perm(g, s, 'Orcish Bowmasters')
        self.assertEqual(v.life, 39); self.assertIsNotNone(E.army_of(s))
        self.assertEqual(ctl.asked[0].kind, 'target')

    def test_bowmasters_on_extra_draws(self):
        g = table('sauron', 'veyran'); s, v = g.players
        perm(g, s, 'Orcish Bowmasters')
        g.controllers = {}
        ctl = seat(g, s, pick(g, s, v))
        E.draw(g, v, 2)                                    # two cards, neither is the draw step's first
        self.assertEqual(len(ctl.asked), 2)

    def test_a_creature_survives_damage_below_its_toughness(self):
        g = table('sauron', 'veyran'); s, v = g.players
        big = perm(g, v, 'Grave Titan')
        seat(g, s, pick(g, s, big))
        mine().kaervek(g, s, v, E.DB['Counterspell'])         # mana value 2 against a 6/6
        self.assertIn(big, v.perms)

    def test_kaervek_kills_what_it_reaches(self):
        g = table('sauron', 'veyran'); s, v = g.players
        gs = perm(g, v, 'Guttersnipe')
        seat(g, s, pick(g, s, gs))
        mine().kaervek(g, s, v, E.DB['Counterspell'])
        self.assertNotIn(gs, v.perms)


class Ring(unittest.TestCase):
    def test_tempt(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        perm(g, s, 'Call of the Ring'); perm(g, s, 'Sauron, the Dark Lord')
        bow = perm(g, s, 'Grave Titan')
        hand(s, 'Island')
        seat(g, s, lambda req: req.choices.index(legal.describe_target(g, s, bow)) if req.prompt.startswith('The Ring')
                   else {'Call': 0, 'Saur': 1}[req.prompt[:4]])     # pay 2 for a card: yes; Sauron's discard: no
        mine().ring_tempt(g, s)
        self.assertIs(s.ring_bearer, bow)
        self.assertEqual(s.life, 38); self.assertEqual(len(s.hand), 2)   # paid 2, drew 1, kept the hand



def float_mana(p, **kw):
    from commander_sim.play import mana
    for c, n in kw.items(): mana.pool_of(p).add(c, n)


class Activations(unittest.TestCase):
    def test_jaces_archivist(self):
        from commander_sim.play import abilities
        g = table('sauron', 'veyran'); s, v = g.players
        arch = perm(g, s, "Jace's Archivist"); arch.sick = False
        hand(s, 'Island', 'Swamp'); hand(v, 'Island', 'Mountain', 'Island')
        seat(g, s, []); float_mana(s, U=1)
        self.assertIsNone(abilities.archivist(g, s, arch))
        self.assertEqual((len(s.hand), len(v.hand)), (3, 3)); self.assertTrue(arch.tapped)

    def test_archivist_needs_to_untap_first(self):
        from commander_sim.play import abilities
        g = table('sauron', 'veyran'); s = g.players[0]
        arch = perm(g, s, "Jace's Archivist", sick=True)
        seat(g, s, []); float_mana(s, U=1)
        self.assertIn('summoning sickness', abilities.archivist(g, s, arch))

    def test_aggravated_assault(self):
        from commander_sim.play import abilities
        g = table('sauron', 'veyran'); s = g.players[0]
        aa = perm(g, s, 'Aggravated Assault'); t = perm(g, s, 'Grave Titan'); t.tapped = True
        seat(g, s, []); float_mana(s, C=3, R=2); g.step = 'main1'
        self.assertIsNone(abilities.assault(g, s, aa))
        self.assertFalse(t.tapped); self.assertEqual(s.extra_combats, 1)

    def test_rogues_passage(self):
        from commander_sim.play import human
        from tests.table import lands
        g = table('sauron', 'veyran'); s, v = g.players
        lands(s, "Rogue's Passage"); t = perm(g, s, 'Grave Titan'); t.sick = False
        perm(g, v, 'Guttersnipe').sick = False
        float_mana(s, C=4)
        seat(g, s, [0, pick(g, s, t)])                     # the ability, then the creature
        self.assertIsNone(human.use_land(g, s, s.lands[0]))
        self.assertTrue(s.lands[0].tapped)
        from commander_sim.play import combat
        seat(g, s, [[combat.attack_candidates(g, s).index(t)]])
        ais.combat(g, s)                                    # Guttersnipe can't block it
        self.assertEqual(v.life, 34)

    def test_scavenger_grounds(self):
        from commander_sim.play import human
        from tests.table import lands
        g = table('sauron', 'veyran'); s, v = g.players
        lands(s, 'Scavenger Grounds'); v.gy.append(E.DB['Grave Titan']); float_mana(s, C=2)
        seat(g, s, [0])
        self.assertIsNone(human.use_land(g, s, s.lands[0]))
        self.assertEqual(v.gy, []); self.assertIn(E.DB['Grave Titan'], v.exile)

    def test_cyclonic_rift_overloaded(self):
        from commander_sim.play import human
        g = table('sauron', 'veyran'); s, v = g.players
        hand(s, 'Cyclonic Rift'); a = perm(g, v, 'Guttersnipe'); b = perm(g, v, 'Sol Ring')
        float_mana(s, C=6, U=1)
        seat(g, s, [{'do': 'cast', 'card': 0}, 1, {'do': 'pass'}])
        human.human_main(g, s, False)
        self.assertEqual(v.perms, [])

if __name__ == '__main__':
    unittest.main()
