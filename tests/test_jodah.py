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


class Lands(unittest.TestCase):
    """the 10-09 land switch: Vivid Creek and Unclaimed Territory"""
    def test_unclaimed_territory_names_the_commonest_creature_type(self):
        g = table('jodah', 'veyran'); p = g.players[0]
        from commander_sim.cards.impl import mine
        self.assertEqual(mine.territory_type(p), 'human')          # Jodah, Grand Abolisher, Mirri ... are Humans
        self.assertEqual(mine.TERRITORY_TYPE['sauron'], 'orc')      # Sauron's deck keeps naming Orc
        lands(p, 'Unclaimed Territory'); lands(p, 'Island'); lands(p, 'Swamp')
        try:
            E.PAY_FOR = card('Grand Abolisher')                       # a Human: any colour
            self.assertTrue(E.can_pay(g, p, 0, 'WUB'))
            E.PAY_FOR = card('Lyra Dawnbringer')                      # an Angel: colourless only
            self.assertFalse(E.can_pay(g, p, 0, 'WUB'))
            self.assertTrue(E.can_pay(g, p, 1, 'UB'))
            E.PAY_FOR = card('Toxic Deluge')                          # not a creature spell
            self.assertFalse(E.can_pay(g, p, 0, 'WUB'))
        finally:
            E.PAY_FOR = None
        self.assertEqual(p.lands[0].data.get('ctype'), 'human')

    def test_vivid_creek_makes_any_colour_twice(self):
        g = table('jodah', 'veyran'); p = g.players[0]
        lands(p, 'Vivid Creek')
        for _ in range(2):
            self.assertTrue(E.can_pay(g, p, 0, 'R'))
            E.pay(g, p, 0, 'R'); p.lands[0].tapped = False
        self.assertFalse(E.can_pay(g, p, 0, 'R'))                      # the counters are gone: {U} only
        self.assertTrue(E.can_pay(g, p, 0, 'U'))
        E.pay(g, p, 0, 'U')
        self.assertEqual(p.lands[0].data['charge'], 0)


class Protection(unittest.TestCase):
    """the 10-09 protection study: what Jodah's AI does with protection when removal or a wipe comes for Jodah"""
    def setUp(self):
        self.g = table('jodah', 'veyran'); self.p, self.q = self.g.players
        self.j = jodah(self.g, self.p)
        self.g.active = self.q

    def test_heroic_intervention_answers_targeted_removal(self):
        g, p, q, j = self.g, self.p, self.q, self.j
        lands(p, 'Forest', 2); hand(p, 'Heroic Intervention')
        E.apply_removal(g, q, j, 'exile', card('Swords to Plowshares'))
        self.assertIn(j, p.perms)
        self.assertTrue(E.untargetable(g, j))                     # hexproof for the rest of the turn
        self.assertEqual(p.stats['jprot_Heroic Intervention'], 1)

    def test_flawless_maneuver_is_free_but_only_stops_destroy(self):
        g, p, q, j = self.g, self.p, self.q, self.j
        hand(p, 'Flawless Maneuver')
        E.apply_removal(g, q, j, 'exile', card('Swords to Plowshares'))
        self.assertNotIn(j, p.perms)                              # indestructible doesn't stop exile
        j = jodah(g, p)
        E.apply_removal(g, q, j, 'destroy', card('Swords to Plowshares'))
        self.assertIn(j, p.perms)                                 # no lands: cast free with Jodah out
        self.assertEqual(p.stats['jprot_Flawless Maneuver'], 1)

    def test_protection_is_held_not_cast_for_value(self):
        g, p = self.g, self.p
        from commander_sim.ai import brain
        lands(p, 'Plains', 3)
        for n in ('Flawless Maneuver', "Tamiyo's Safekeeping", "Teferi's Protection"):
            self.assertIsNone(brain.card_utility(g, p, brain.Situation(g, p), hand(p, n)))

    def test_safekeeping_keeps_jodah_through_a_wrath(self):
        g, p, q, j = self.g, self.p, self.q, self.j
        other = perm(g, p, 'Lyra Dawnbringer')
        lands(p, 'Forest'); hand(p, "Tamiyo's Safekeeping")
        self.assertIsNone(ais.wipe_response(g, p, 'destroy', q))   # Jodah alone is indestructible
        for m in list(p.perms): E.die(g, m, 'destroy')
        self.assertIn(j, p.perms)
        self.assertNotIn(other, p.perms)
        self.assertEqual(p.stats["jprot_wipe_Tamiyo's Safekeeping"], 1)

    def test_teferis_protection_against_an_exile_wipe(self):
        g, p, q = self.g, self.p, self.q
        lands(p, 'Plains', 3); hand(p, "Teferi's Protection")
        self.assertEqual(ais.wipe_response(g, p, 'exile', q), 'all')

    def test_plaza_of_heroes_saves_jodah(self):
        g, p, q, j = self.g, self.p, self.q, self.j
        lands(p, 'Plaza of Heroes'); lands(p, 'Swamp', 3)
        E.apply_removal(g, q, j, 'destroy', card('Swords to Plowshares'))
        self.assertIn(j, p.perms)
        self.assertNotIn('Plaza of Heroes', [L.cd.name for L in p.lands])
        self.assertIn('Plaza of Heroes', [c.name for c in p.exile])
        self.assertTrue(all(L.tapped for L in p.lands))

    def test_runes_and_swat(self):
        g, p, q, j = self.g, self.p, self.q, self.j
        perm(g, p, 'Giver of Runes')
        E.apply_removal(g, q, j, 'exile', card('Swords to Plowshares'))
        self.assertIn(j, p.perms)
        t = perm(g, q, 'Grave Titan')
        hand(p, 'Deflecting Swat')
        E.apply_removal(g, q, j, 'exile', card('Swords to Plowshares'))  # Giver is tapped: Swat, free, sends it back
        self.assertIn(j, p.perms)
        self.assertNotIn(t, q.perms)

    def test_equipment_goes_to_jodah(self):
        g, p, j = self.g, self.p, self.j
        g.active = p
        lands(p, 'Plains', 3)
        other = perm(g, p, 'Lyra Dawnbringer')
        boots = perm(g, p, 'Swiftfoot Boots'); boots.attached = other
        o = J.jodah_options(g, p, False)
        self.assertEqual([x[1] for x in o], ['equip Swiftfoot Boots to Jodah'])
        o[0][2]()
        self.assertIs(boots.attached, j)
        self.assertTrue(E.untargetable(g, j))
        coat = perm(g, p, 'Mithril Coat')                         # attaches to Jodah as it enters
        self.assertIs(coat.attached, j)
        self.assertTrue(E.indestructible(g, j))


if __name__ == '__main__':
    unittest.main()
