"""Practice mode, Sephiroth's (and Marchesa's) graveyard cards: reanimation targets are chosen as the spell is cast,
from any graveyard where the card allows; Entomb, Buried Alive and Grisly Salvage let you pick; Deadly Dispute asks
what to sacrifice."""
import unittest
from tests.table import table, hand, perm
from commander_sim import engine as E
from commander_sim.play import legal, mana
from commander_sim.play.controller import ScriptController


def seat(g, p, answers):
    g.controllers = {p.key: ScriptController(answers)}
    return g.controllers[p.key]


def float_mana(p, **kw):
    for c, n in kw.items(): mana.pool_of(p).add(c, n)


def by_text(word):
    return lambda req: next(i for i, x in enumerate(req.choices) if word in x)


class Reanimation(unittest.TestCase):
    def test_animate_dead_from_an_opponents_graveyard(self):
        from commander_sim.play import human
        g = table('seph', 'veyran'); s, v = g.players
        hand(s, 'Animate Dead'); v.gy.append(E.DB['Guttersnipe']); s.gy.append(E.DB['Grave Titan'])
        float_mana(s, C=1, B=1)
        ctl = seat(g, s, [{'do': 'cast', 'card': 0}, by_text('Guttersnipe'), {'do': 'pass'}])
        human.human_main(g, s, False)
        self.assertTrue(any(m.name == 'Guttersnipe' for m in s.perms))
        self.assertIn("graveyard", ctl.asked[1].choices[0])

    def test_unburial_rites_only_from_your_graveyard(self):
        from commander_sim.play import choices
        g = table('seph', 'veyran'); s, v = g.players
        v.gy.append(E.DB['Guttersnipe'])
        self.assertEqual(choices.rean_candidates(g, s, 'rites'), [])
        self.assertIn('no creature card', legal.check_cast(g, s, hand(s, 'Unburial Rites')))

    def test_entomb(self):
        from commander_sim.play import choices
        g = table('seph', 'veyran'); s = g.players[0]
        seat(g, s, [by_text('Grave Titan')])
        choices.fill(g, s, 'entomb')
        self.assertEqual([c.name for c in s.gy], ['Grave Titan'])

    def test_grisly_salvage(self):
        from commander_sim.play import choices
        g = table('seph', 'veyran'); s = g.players[0]
        n = len(s.hand)
        seat(g, s, ['cancel'])
        choices.fill(g, s, 'grisly')
        self.assertEqual((len(s.hand), len(s.gy)), (n, 5))


class Dispute(unittest.TestCase):
    def test_deadly_dispute_sacrifices_your_pick(self):
        from commander_sim.play import human
        g = table('marchesa', 'veyran'); s = g.players[0]
        hand(s, 'Deadly Dispute'); a = perm(g, s, 'Sol Ring'); float_mana(s, C=1, B=1)
        seat(g, s, [{'do': 'cast', 'card': 0}, 0, {'do': 'pass'}])
        human.human_main(g, s, False)
        self.assertNotIn(a, s.perms); self.assertEqual(s.treasures, 1)



class FromTheGraveyard(unittest.TestCase):
    def test_flashback_deep_analysis(self):
        from commander_sim.play import human
        g = table('marchesa', 'veyran'); s = g.players[0]
        s.gy.append(E.DB['Deep Analysis']); float_mana(s, C=1, U=1); n = len(s.hand)
        seat(g, s, [{'do': 'cast', 'zone': 'gy', 'card': 0}, {'do': 'pass'}])
        human.human_main(g, s, False)
        self.assertEqual((len(s.hand), s.life), (n + 2, 37))
        self.assertIn(E.DB['Deep Analysis'], s.exile)

    def test_dread_return_flashback_sacrifices_three(self):
        from commander_sim.play import human
        g = table('seph', 'veyran'); s = g.players[0]
        for n in ('Llanowar Elves', 'Birds of Paradise', 'Viscera Seer'): perm(g, s, n)
        s.gy += [E.DB['Dread Return'], E.DB['Grave Titan']]
        seat(g, s, [{'do': 'cast', 'zone': 'gy', 'card': 0}, by_text('Grave Titan'), 0, 0, 0, {'do': 'pass'}])
        human.human_main(g, s, False)
        self.assertTrue(any(m.name == 'Grave Titan' for m in s.perms))
        self.assertFalse(any(m.name in ('Llanowar Elves', 'Birds of Paradise', 'Viscera Seer') for m in s.perms))

    def test_yawgmoths_will(self):
        import collections
        from commander_sim.play import human
        g = table('seph', 'veyran'); s = g.players[0]
        s.gy += [E.DB['Swamp'], E.DB['Entomb']]
        s.yawg = True; s.yawg_gy = collections.Counter(id(x) for x in s.gy)
        seat(g, s, [{'do': 'land', 'zone': 'gy', 'card': 0}, {'do': 'pass'}])
        human.human_main(g, s, False)
        self.assertEqual(s.lands[-1].cd.name, 'Swamp')
        float_mana(s, B=1)
        self.assertIsNone(legal.check_cast_gy(g, s, E.DB['Entomb']))        # its normal cost, from the graveyard

    def test_not_from_the_graveyard_otherwise(self):
        g = table('seph', 'veyran'); s = g.players[0]
        s.gy.append(E.DB['Entomb'])
        self.assertIn("can't cast Entomb from your graveyard", legal.check_cast_gy(g, s, s.gy[0]))


class Loops(unittest.TestCase):
    def test_start_the_loop(self):
        from commander_sim.play import human
        g = table('seph', 'veyran'); s = g.players[0]
        perm(g, s, 'Mikaeus, the Unhallowed'); trisk = perm(g, s, 'Triskelion')
        labels = [a[0] for a in human.abilities_of(g, s, trisk)]
        self.assertTrue(any(l.startswith('start the loop: Mikaeus + Triskelion') for l in labels), labels)
        k = next(i for i, l in enumerate(labels) if l.startswith('start the loop'))
        seat(g, s, [{'do': 'use', 'perm': s.perms.index(trisk)}, k])
        human.human_main(g, s, False)
        self.assertTrue(g.over); self.assertIs(g.winner, s)


class Triggers(unittest.TestCase):
    def test_aura_shards_target_or_none(self):
        from commander_sim.play import human
        g = table('seph', 'veyran'); s, v = g.players
        perm(g, s, 'Aura Shards'); ring = perm(g, v, 'Sol Ring')
        seat(g, s, ['cancel']); perm(g, s, 'Llanowar Elves')
        self.assertIn(ring, v.perms)
        seat(g, s, [by_text('Sol Ring')]); perm(g, s, 'Birds of Paradise')
        self.assertNotIn(ring, v.perms)

    def test_archon_targets_your_pick(self):
        g = table('seph', 'veyran', 'sauron'); s, v, x = g.players
        seat(g, s, [by_text('Sauron')])
        perm(g, s, 'Archon of Cruelty')
        self.assertEqual((v.life, x.life), (40, 37))

    def test_consecrated_sphinx_may(self):
        g = table('seph', 'veyran'); s, v = g.players
        perm(g, s, 'Consecrated Sphinx'); n = len(s.hand)
        seat(g, s, [1]); E.draw(g, v, 1)
        self.assertEqual(len(s.hand), n)
        seat(g, s, [0]); E.draw(g, v, 1)
        self.assertEqual(len(s.hand), n + 2)

    def test_nim_deathmantle_pay_or_not(self):
        from tests.table import lands
        g = table('seph', 'veyran'); s = g.players[0]
        perm(g, s, 'Nim Deathmantle'); lands(s, 'Swamp', 4)
        t = perm(g, s, 'Grave Titan')
        seat(g, s, [0]); E.die(g, t, 'destroy')
        self.assertTrue(any(m.name == 'Grave Titan' for m in s.perms))

    def test_tortured_existence(self):
        from commander_sim.play import abilities
        g = table('seph', 'veyran'); s = g.players[0]
        te = perm(g, s, 'Tortured Existence'); hand(s, 'Llanowar Elves'); s.gy.append(E.DB['Grave Titan'])
        float_mana(s, B=1); seat(g, s, [0, 0])
        self.assertIsNone(abilities.tortured(g, s, te))
        self.assertIn(E.DB['Grave Titan'], s.hand); self.assertIn(E.DB['Llanowar Elves'], s.gy)

    def test_gifts_ungiven_you_pick_they_split(self):
        g = table('seph', 'veyran'); s = g.players[0]
        seat(g, s, [0, 0, 0, 0]); n = len(s.hand)
        E.pile_tutor(g, s, 4, 2)
        self.assertEqual((len(s.hand), len(s.gy)), (n + 2, 2))


class Atraxa(unittest.TestCase):
    def test_you_pick_a_card_of_each_type(self):
        g = table('seph', 'veyran'); s = g.players[0]
        top = [E.DB[n] for n in ('Swamp', 'Sol Ring', 'Grave Titan', 'Animate Dead', 'Entomb', 'Reanimate',
                                 'Forest', 'Mind Stone', 'Kitchen Finks', 'Toxic Deluge')]
        s.library += list(reversed(top))                      # the first of these on top
        n = len(s.library)
        ctl = seat(g, s, [by_text('Mind Stone'), by_text('Kitchen Finks'), by_text('Animate Dead'), by_text('Entomb'),
                          'cancel', by_text('Toxic Deluge')])      # artifact, creature, enchantment, instant, no land, sorcery
        E.atraxa_reveal(g, s)
        self.assertEqual(sorted(c.name for c in s.hand[-5:]),
                         ['Animate Dead', 'Entomb', 'Kitchen Finks', 'Mind Stone', 'Toxic Deluge'])
        self.assertEqual(len(s.library), n - 5)                    # the other five on the bottom
        self.assertIn('artifact', ctl.asked[0].prompt); self.assertIn('land', ctl.asked[4].prompt)

if __name__ == '__main__':
    unittest.main()
