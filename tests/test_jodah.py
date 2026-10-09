"""JD's Jodah deck (cards/impl/jodah.py): Jodah's anthem and legend cascade, Kaldra, Blackblade Reforged, the
planeswalkers, the legends with abilities, and the deck's spells (including the engine hooks they needed:
Desertion, Dissipate, Szadek's damage replacement, Memory Jar's end step, suspend)."""
import unittest
from tests.table import table, hand, perm, lands, card, token
from commander_sim import engine as E, ais


J = None


def setUpModule():
    global J
    table('jodah', 'veyran')
    from commander_sim.cards.impl import jodah as _j
    J = _j


def jodah(g, p):
    m = E.enter(g, p, p.cmd); m.is_cmd = True; p.cmd_in_zone = False
    return m


class Jodah(unittest.TestCase):
    def test_anthem_counts_legends(self):
        g = table('jodah', 'veyran'); p = g.players[0]
        j = jodah(g, p)
        perm(g, p, 'Lyra Dawnbringer')
        self.assertEqual(E.epow(g, j), 5 + 2)            # two legends: +2/+2 each
        perm(g, p, 'Solemn Simulacrum')                  # not legendary: no bonus, no count
        self.assertEqual(E.epow(g, j), 7)

    def test_cascade_from_hand_finds_a_cheaper_legend(self):
        g = table('jodah', 'veyran'); p = g.players[0]
        jodah(g, p)
        p.library = [card('Island'), card('Lyra Dawnbringer'), card('Mind Stone'), card('King Darien XLVIII')]
        lands(p, 'Plains', 3); lands(p, 'Swamp', 2); lands(p, 'Island', 2)
        c = hand(p, 'Dakkon Blackblade')
        E.pay(g, p, 2, 'WUUB'); E.cast_card(g, p, c)
        names = [m.name for m in p.perms]
        self.assertIn('King Darien XLVIII', names)        # first legend below 6 from the top
        self.assertNotIn('Lyra Dawnbringer', names)

    def test_a_cascade_that_finds_nothing_says_so(self):
        g = table('jodah', 'veyran'); p = g.players[0]
        jodah(g, p)
        p.library = [card('Island'), card('Lyra Dawnbringer'), card('Mind Stone')]   # no legend below mana value 2
        g.log = []
        E.cast_card(g, p, hand(p, 'Wrenn and Six'))
        self.assertEqual(sorted(c.name for c in p.library), ['Island', 'Lyra Dawnbringer', 'Mind Stone'])
        self.assertTrue(any('finds no legendary nonland card with mana value below 2' in l for l in g.log))

    def test_no_cascade_from_a_free_cast(self):
        g = table('jodah', 'veyran'); p = g.players[0]
        jodah(g, p)
        p.library = [card('Mirri, Weatherlight Duelist'), card('King Darien XLVIII')]
        E.cast_card(g, p, card('Lyra Dawnbringer'), 'lib', {})
        self.assertNotIn('King Darien XLVIII', [m.name for m in p.perms])


class Kaldra(unittest.TestCase):
    def test_helm_assembles_kaldra(self):
        g = table('jodah', 'veyran'); p = g.players[0]
        lands(p, 'Plains', 2)
        pieces = [perm(g, p, n) for n in ('Sword of Kaldra', 'Shield of Kaldra', 'Helm of Kaldra')]
        from commander_sim.ai import brain
        o = E.CI.HOOKS['Helm of Kaldra']['options'](g, pieces[2], p, brain.Situation(g, p), False)
        self.assertTrue(o and o[0][2]())
        k = next(m for m in p.perms if m.token and m.data and m.data.get('kaldra'))
        self.assertTrue(all(e.attached is k for e in pieces))
        self.assertEqual(E.epow(g, k), 9)
        self.assertTrue(E.indestructible(g, k))

    def test_sword_exiles_what_it_damages(self):
        g = table('jodah', 'veyran'); p, q = g.players
        sw = perm(g, p, 'Sword of Kaldra'); a = perm(g, p, 'Lyra Dawnbringer'); sw.attached = a
        b = perm(g, q, 'Grave Titan')
        self.assertTrue(J.kaldra_exile(g, a, b))
        self.assertNotIn(b, q.perms)
        self.assertTrue(any(c.name == 'Grave Titan' for c in q.exile))


class Legends(unittest.TestCase):
    def test_blackblade_counts_lands(self):
        g = table('jodah', 'veyran'); p = g.players[0]
        lands(p, 'Plains', 5)
        bb = perm(g, p, 'Blackblade Reforged'); m = perm(g, p, 'Lyra Dawnbringer'); bb.attached = m
        self.assertEqual(E.epow(g, m), 5 + 5)

    def test_korlash_counts_swamps(self):
        g = table('jodah', 'veyran'); p = g.players[0]
        lands(p, 'Swamp', 2); lands(p, 'Plains', 2)
        k = perm(g, p, 'Korlash, Heir to Blackblade')
        self.assertEqual(E.epow(g, k), 2)

    def test_szadek_mills_instead_of_damage(self):
        g = table('jodah', 'veyran'); p, q = g.players
        s = perm(g, p, 'Szadek, Lord of Secrets'); s.sick = False
        lib, life = len(q.library), q.life
        ais.resolve_combat(g, p, [s], q, set())
        self.assertEqual(q.life, life)
        self.assertEqual(len(q.library), lib - 5)
        self.assertEqual(s.plus, 5)

    def test_dromoka_stops_opponents_on_your_turn(self):
        g = table('jodah', 'veyran'); p, q = g.players
        perm(g, p, 'Dragonlord Dromoka')
        g.active = p
        self.assertFalse(E.castable(g, q, card('Lightning Bolt')))
        g.active = q
        self.assertTrue(E.castable(g, q, card('Lightning Bolt')))

    def test_mirri_tapped_caps_attackers(self):
        g = table('jodah', 'veyran'); p, q = g.players
        m = perm(g, p, 'Mirri, Weatherlight Duelist'); m.tapped = True
        self.assertEqual(E.CI.HOOKS['Mirri, Weatherlight Duelist']['attack_cap'](g, m, q, p), 1)

    def test_carth_adds_loyalty(self):
        g = table('jodah', 'veyran'); p = g.players[0]
        perm(g, p, 'Carth the Lion')
        w = perm(g, p, 'Mordenkainen')
        from commander_sim.ai import brain
        o = E.CI.HOOKS['Mordenkainen']['options'](g, w, p, brain.Situation(g, p), False)
        plus = next(x for x in o if '+3' in x[1])          # +2 becomes +3
        before = w.loyalty; plus[2]()
        self.assertEqual(w.loyalty, before + 3)


class Spells(unittest.TestCase):
    def test_memory_jar_wheels_and_returns(self):
        g = table('jodah', 'veyran'); p, q = g.players
        lands(p, 'Plains', 3)
        jar = perm(g, p, 'Memory Jar')
        old = hand(p, 'Lyra Dawnbringer')
        from commander_sim.ai import brain
        o = E.CI.HOOKS['Memory Jar']['options'](g, jar, p, brain.Situation(g, p), False)
        self.assertTrue(o and o[0][2]())
        self.assertEqual(len(p.hand), 7)
        ais.end_step(g, p)
        self.assertEqual([c.name for c in p.hand], ['Lyra Dawnbringer'])

    def test_profane_tutor_suspend_then_free(self):
        g = table('jodah', 'veyran'); p = g.players[0]
        lands(p, 'Swamp', 2)
        c = hand(p, 'Profane Tutor')
        from commander_sim.ai import brain
        o = E.CI.HOOKS['Profane Tutor']['hand_options'](g, c, p, brain.Situation(g, p), False)
        o[0][2]()
        self.assertIn(c, p.exile)
        n = len(p.hand)
        ais.upkeep(g, p)
        self.assertEqual(len(p.hand), n)
        ais.upkeep(g, p)
        self.assertEqual(len(p.hand), n + 1)                # cast free: a tutored card

    def test_dissipate_exiles_and_desertion_steals(self):
        dis, des = card('Dissipate'), card('Desertion')
        self.assertIn('ctrexile', dis.tags)
        self.assertEqual(des.tags.get('ctr'), 'any')

    def test_court_returns_a_permanent(self):
        g = table('jodah', 'veyran'); p = g.players[0]
        perm(g, p, 'Court of Ardenvale')
        self.assertIs(g.monarch, p)
        p.gy.append(card('Mind Stone'))
        E.CI.fire(g, 'upkeep', p)
        self.assertIn('Mind Stone', [m.name for m in p.perms])

    def test_fyndhorn_elder_makes_two(self):
        g = table('jodah', 'veyran'); p = g.players[0]
        perm(g, p, 'Fyndhorn Elder')
        self.assertEqual(E.total_mana(g, p), 2)


class JodahAudit(unittest.TestCase):
    """audit/jodah: Plaza of Heroes' colours, Jodah before the legends, the tutor wish list"""
    def test_plaza_any_colour_for_legendary_spells_only(self):
        g = table('jodah', 'veyran'); p = g.players[0]
        for n in ('Frontier Bivouac', 'Forest', 'Plaza of Heroes', 'Sandsteppe Citadel', 'Crumbling Necropolis'):
            lands(p, n)
        try:
            E.PAY_FOR = p.cmd
            self.assertTrue(E.can_pay(g, p, 0, 'WUBRG'))      # Plaza makes the fifth colour for Jodah
            E.PAY_FOR = card('Toxic Deluge')
            self.assertFalse(E.can_pay(g, p, 0, 'WUBRG'))
        finally:
            E.PAY_FOR = None

    def test_jodah_before_legends(self):
        g = table('jodah', 'veyran'); p = g.players[0]; g.active = p
        for n in ('Plains', 'Island', 'Swamp', 'Mountain', 'Forest'): lands(p, n)
        lyra = hand(p, 'Lyra Dawnbringer')
        self.assertLessEqual(J.jodah_prio(g, p, lyra), 15)    # cast Jodah first, then Lyra cascades
        self.assertGreater(J.jodah_prio(g, p, p.cmd), 80)
        jodah(g, p)
        self.assertGreater(J.jodah_prio(g, p, lyra), 40)

    def test_tutor_finds_fixing_early_and_a_big_legend_later(self):
        g = table('jodah', 'veyran'); p = g.players[0]
        lands(p, 'Forest', 2)
        p.library += [card('Coalition Relic'), card("Sisay's Ring"), card('Razia, Boros Archangel')]
        self.assertEqual(ais.tutor_pick(g, p, 'any'), 'Coalition Relic')
        lands(p, 'Plains', 3); jodah(g, p)
        self.assertEqual(ais.tutor_pick(g, p, 'any'), 'Razia, Boros Archangel')

    def test_champions_helm_goes_on_jodah(self):
        from commander_sim.ai import brain
        g = table('jodah', 'veyran'); p = g.players[0]; g.active = p
        lands(p, 'Plains', 3)
        helm = hand(p, "Champion's Helm")
        before = J.jodah_prio(g, p, helm)
        j = jodah(g, p)
        self.assertGreater(J.jodah_prio(g, p, helm), before)            # cast sooner once Jodah is out to protect
        perm(g, p, "Champion's Helm"); lyra = perm(g, p, 'Lyra Dawnbringer')
        opts = {name: go for _, name, go in brain.special_options(g, p, None, False)}
        self.assertNotIn(f"equip Champion's Helm to {lyra.name}", opts)  # Jodah, not the bigger legend
        opts[f"equip Champion's Helm to {j.name}"]()
        self.assertTrue(E.equipped(j, 'helm'))
        self.assertTrue(E.untargetable(g, j))                            # a legend wearing it has hexproof
        self.assertFalse(any('Champion' in name for _, name, _ in brain.special_options(g, p, None, False)))


if __name__ == '__main__':
    unittest.main()
