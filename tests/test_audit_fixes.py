"""The gaps found by the October 2026 modeling audit: Muldrotha, the Gravetide; Restoration Angel's enters trigger;
Kefka's draw; Relic of Legends; and the choices practice mode used to make for you (card-text targets, Emeritus of
Ideation, Scarlet Witch, Force of Will's exiled card, X, Niv-Mizzet's and Torch Fiend's abilities)."""
import unittest
from tests.table import table, hand, perm, lands, card
from commander_sim import engine as E, ais
from commander_sim.play import human, mana
from commander_sim.play.controller import ScriptController


def seat(g, p, answers):
    g.controllers = {p.key: ScriptController(answers)}
    return g.controllers[p.key]


def by_text(word):
    return lambda req: next(i for i, x in enumerate(req.choices) if word in x)


class Muldrotha(unittest.TestCase):
    def setUp(self):
        self.g = table('seph', 'veyran', 'sauron')
        self.s = self.g.players[0]
        perm(self.g, self.s, 'Muldrotha, the Gravetide')

    def test_one_permanent_of_each_type_from_the_graveyard(self):
        g, s = self.g, self.s
        s.gy += [card('Sol Ring'), card('Mind Stone'), card('Command Tower')]
        self.assertEqual(E.CI.muld_types(g, s, card('Sol Ring')), ['A'])
        E.CI.muld_mark(g, s, 'A')
        self.assertEqual(E.CI.muld_types(g, s, card('Mind Stone')), [])            # artifact used this turn
        self.assertEqual(E.CI.muld_types(g, s, card('Command Tower')), ['L'])
        g.active = g.players[1]
        self.assertEqual(E.CI.muld_types(g, s, card('Command Tower')), [])          # only during your turns

    def test_the_ai_casts_from_the_graveyard(self):
        g, s = self.g, self.s
        lands(s, 'Swamp', 2)
        s.gy.append(card('Sol Ring'))
        from commander_sim.cards import cardimpl as CI
        opts = CI.HOOKS['Muldrotha, the Gravetide']['options'](g, next(m for m in s.perms if m.cd.name.startswith('Muldrotha')), s, None, False)
        self.assertTrue(any('Sol Ring' in l for u, l, f in opts))

    def test_the_ai_plays_a_land_from_the_graveyard(self):
        g, s = self.g, self.s
        s.gy.append(card('Command Tower'))
        ais.play_land(g, s)
        self.assertTrue(any(L.cd.name == 'Command Tower' for L in s.lands))
        self.assertIn('L', E.CI.muld_used(g, s))

    def test_you_cast_from_the_graveyard_in_practice_mode(self):
        g, s = self.g, self.s
        s.gy.append(card('Sol Ring'))
        mana.pool_of(s).add('C', 1)
        seat(g, s, [])
        self.assertIsNone(human.apply(g, s, {'do': 'cast', 'zone': 'gy', 'card': len(s.gy) - 1}))
        self.assertTrue(any(m.cd.name == 'Sol Ring' for m in s.perms))
        self.assertIn('A', E.CI.muld_used(g, s))


class RestorationAngel(unittest.TestCase):
    def test_cast_normally_it_blinks(self):
        g = table('zur', 'veyran', 'sauron'); z, v, s = g.players
        for n in ('Guttersnipe', 'Harmonic Prodigy'): perm(g, v, n)
        for n in ('Orcish Bowmasters', 'Witch-king of Angmar'): perm(g, s, n)
        for n in ('Esper Sentinel', 'Ministrant of Obligation'): perm(g, z, n)
        mar = perm(g, z, 'Accursed Marauder')                                         # each player sacrifices one
        theirs = sum(1 for q in (v, s) for m in q.perms if m.creature and not m.token)
        perm(g, z, 'Restoration Angel')                                               # blinks the Marauder
        self.assertLess(sum(1 for q in (v, s) for m in q.perms if m.creature and not m.token), theirs)

    def test_you_choose_or_decline(self):
        g = table('zur', 'veyran', 'sauron'); z = g.players[0]
        perm(g, z, 'Esper Sentinel')
        ctl = seat(g, z, ['cancel'])
        perm(g, z, 'Restoration Angel')
        self.assertTrue(any('Restoration Angel enters' in r.prompt for r in ctl.asked))


class Kefka(unittest.TestCase):
    def test_the_draw_is_not_optional(self):
        table('sauron', 'veyran')                       # the card database first (see setUpModule in test_zur)
        from commander_sim.cards.impl import mine
        self.assertNotIn('len(p.library) - 5', __import__('inspect').getsource(mine._kefka_ruin_draw))


class RelicOfLegends(unittest.TestCase):
    def test_the_ai_taps_every_spare_legend(self):
        g = table('marchesa', 'veyran', 'sauron'); m = g.players[0]
        r = perm(g, m, 'Relic of Legends')
        a = perm(g, m, 'Marchesa, the Black Rose', sick=True); b = perm(g, m, 'Hellkite Tyrant', sick=True)
        from commander_sim.cards import cardimpl as CI
        legs = sum(1 for x in (a, b) if __import__('commander_sim.cards.impl.mine', fromlist=['x']).is_legendary(g, x))
        self.assertEqual(CI.DYN_MANA['Relic of Legends'](g, m, r), 1 + legs)

    def test_your_second_ability(self):
        g = table('marchesa', 'veyran', 'sauron'); m = g.players[0]
        r = perm(g, m, 'Relic of Legends'); a = perm(g, m, 'Marchesa, the Black Rose')
        seat(g, m, [by_text('tap an untapped legendary'), 0, 0])
        self.assertIsNone(human.use(g, m, r))
        self.assertTrue(a.tapped)
        self.assertEqual(mana.pool_of(m).total(), 1)


class YourChoices(unittest.TestCase):
    def test_card_text_targets_ask_you(self):
        g = table('marchesa', 'veyran', 'sauron'); m, v, s = g.players
        rock = perm(g, v, 'Sol Ring'); perm(g, s, 'Sol Ring')
        tf = perm(g, m, 'Torch Fiend')
        mana.pool_of(m).add('R', 1)
        seat(g, m, [0, by_text('Veyran')])
        self.assertIsNone(human.use(g, m, tf))
        self.assertNotIn(rock, v.perms)

    def test_niv_mizzet_the_firemind_draws(self):
        g = table('veyran', 'seph', 'sauron'); v = g.players[0]
        niv = perm(g, v, 'Niv-Mizzet, the Firemind')
        n = len(v.hand)
        seat(g, v, [0, 0])                                                  # the ability; its ping's target
        self.assertIsNone(human.use(g, v, niv))
        self.assertEqual(len(v.hand), n + 1); self.assertTrue(niv.tapped)

    def test_emeritus_of_ideation_asks(self):
        g = table('veyran', 'seph', 'sauron'); v, s, r = g.players
        em = perm(g, v, 'Emeritus of Ideation // Ancestral Recall')
        v.gy += [card(n) for n in ('Island', 'Mountain', 'Think Twice', 'Deduce', 'Quick Study', 'Stock Up',
                                    'Flow State', 'Abrade', 'Burst Lightning')]
        seat(g, v, ['cancel'])
        ais.attack_triggers(g, v, [em], s)
        self.assertEqual(len(v.gy), 9)                                      # declined: nothing exiled

    def test_force_of_will_exiles_the_card_you_pick(self):
        from commander_sim.play import cards
        g = table('seph', 'veyran', 'sauron'); s = g.players[0]
        a, b = card('Counterspell'), card('Cyclonic Rift')
        seat(g, s, [by_text('Cyclonic Rift')])
        self.assertIs(cards.pick_blue(g, s, [a, b], 'Force of Will'), b)

    def test_you_choose_x(self):
        g = table('zur', 'veyran', 'sauron'); z = g.players[0]
        hand(z, 'Secure the Wastes')
        mana.pool_of(z).add('W', 1); mana.pool_of(z).add('C', 4)
        seat(g, z, [{'do': 'cast', 'card': 0}, by_text('X = 2'), {'do': 'pass'}])
        human.human_main(g, z, False)
        self.assertEqual(sum(1 for m in z.perms if m.token), 2)
        self.assertEqual(mana.pool_of(z).total(), 2)


if __name__ == '__main__':
    unittest.main()
