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

    def test_bolt_bend_costs_r_with_power_four_and_redirects(self):
        g, p, q, j = self.g, self.p, self.q, self.j
        self.assertEqual(J.prot_cost(g, p, 'Bolt Bend'), (0, 'R'))   # Jodah is a 5/5
        E.leave(g, j); p.perms.clear(); perm(g, p, 'Mirri, Weatherlight Duelist')
        self.assertEqual(J.prot_cost(g, p, 'Bolt Bend'), (3, 'R'))   # a 3/2 only: full price
        j = jodah(g, p)
        t = perm(g, q, 'Grave Titan')
        lands(p, 'Mountain'); bend = hand(p, 'Bolt Bend')
        E.apply_removal(g, q, j, 'exile', card('Swords to Plowshares'))
        self.assertIn(j, p.perms)
        self.assertNotIn(t, q.perms)                                 # the Swords hit the caster's Titan
        self.assertIn(bend, p.gy)
        self.assertTrue(p.lands[0].tapped)                            # paid {R}
        self.assertEqual(p.stats['jprot_Bolt Bend'], 1)

    def test_bolt_bend_guards_a_key_legend_while_jodah_is_away(self):
        g, p, q, j = self.g, self.p, self.q, self.j
        E.leave(g, j); p.perms.clear(); p.cmd_in_zone = True
        wanderer = perm(g, p, 'Maelstrom Wanderer')                  # a 7/5 bomb
        t = perm(g, q, 'Grave Titan')
        lands(p, 'Mountain'); hand(p, 'Bolt Bend')
        E.apply_removal(g, q, wanderer, 'destroy', card('Swords to Plowshares'))
        self.assertIn(wanderer, p.perms)
        self.assertNotIn(t, q.perms)
        self.assertEqual(p.stats['jprot_Bolt Bend (legend)'], 1)


class Rework(unittest.TestCase):
    """the 10-09 rework candidates: Command Beacon (Jodah from hand, no tax), Maelstrom Nexus and Maelstrom Wanderer
    (plain cascade, on top of Jodah's), Sisay, Weatherlight Captain"""
    def setUp(self):
        self.g = table('jodah', 'veyran'); self.p, self.q = self.g.players

    def test_command_beacon_puts_jodah_in_hand_and_skips_the_tax(self):
        g, p = self.g, self.p
        from commander_sim.ai import brain
        from commander_sim.cards.impl import lands as IL
        for n in ('Plains', 'Island', 'Swamp', 'Mountain', 'Forest', 'Command Beacon'): lands(p, n)
        p.tax = 2                                                     # castable from the zone (7 mana is not there):
        o = IL.land_options(g, p, brain.Situation(g, p), False)       # the Beacon makes it castable now
        self.assertEqual([x[1] for x in o], ['Command Beacon: Jodah, the Unifier to hand'])
        o[0][2]()
        self.assertIn(p.cmd, p.hand)
        self.assertFalse(p.cmd_in_zone)
        self.assertEqual(E.cost_of(p, p.cmd), (0, 'WUBRG'))           # no commander tax from hand
        self.assertTrue(brain.do_cast(g, p, p.cmd))
        j = next(m for m in p.perms if m.cd is p.cmd)
        self.assertTrue(j.is_cmd)
        self.assertEqual(p.tax, 2)                                    # a cast from hand doesn't add tax
        self.assertEqual((p.stats['jr_beacon'], p.stats['jr_jodah_from_hand']), (1, 1))
        E.die(g, j, 'destroy')
        self.assertTrue(p.cmd_in_zone)                                # back to the command zone

    def test_command_beacon_waits_while_the_tax_is_small(self):
        g, p = self.g, self.p
        from commander_sim.ai import brain
        from commander_sim.cards.impl import lands as IL
        for n in ('Plains', 'Island', 'Swamp', 'Mountain', 'Forest', 'Forest', 'Forest', 'Command Beacon'): lands(p, n)
        p.tax = 2                                                     # 7 lands pay for Jodah with tax: keep the land
        self.assertEqual(IL.land_options(g, p, brain.Situation(g, p), False), [])
        p.tax = 4
        self.assertEqual(len(IL.land_options(g, p, brain.Situation(g, p), False)), 1)

    def test_a_discarded_commander_goes_to_the_command_zone(self):
        g, p = self.g, self.p
        p.cmd_in_zone = False; p.hand.append(p.cmd)
        E.discard_cards(g, p, [p.cmd])
        self.assertTrue(p.cmd_in_zone)
        self.assertNotIn(p.cmd, p.gy)

    def test_nexus_cascades_the_first_spell_only(self):
        g, p = self.g, self.p
        perm(g, p, 'Maelstrom Nexus')
        p.library = [card('Fact or Fiction'), card('Island'), card('Arcane Signet')]
        E.cast_card(g, p, hand(p, 'Coalition Relic'))                 # mana value 3: Arcane Signet (2), not the 4
        self.assertIn('Arcane Signet', [m.name for m in p.perms])
        E.cast_card(g, p, hand(p, 'Star Compass'))                    # the second spell: no cascade
        self.assertEqual(p.stats['jr_nexus'], 1)
        self.assertEqual(len(p.library), 2)

    def test_nexus_passes_on_a_counterspell(self):
        g, p = self.g, self.p
        perm(g, p, 'Maelstrom Nexus')
        p.library = [card('Arcane Denial')]
        E.cast_card(g, p, hand(p, 'Coalition Relic'))
        self.assertEqual([c.name for c in p.library], ['Arcane Denial'])

    def test_nexus_and_jodah_both_cascade_a_legend_from_hand(self):
        g, p = self.g, self.p
        jodah(g, p); perm(g, p, 'Maelstrom Nexus')
        p.library = [card('Moss Diamond'), card('King Darien XLVIII')]   # Jodah finds the legend, Nexus the rock
        E.cast_card(g, p, hand(p, 'Lyra Dawnbringer'))
        names = [m.name for m in p.perms]
        self.assertIn('King Darien XLVIII', names)
        self.assertIn('Moss Diamond', names)
        self.assertEqual((p.stats['jodah_cascades'], p.stats['jr_nexus']), (1, 1))

    def test_wanderer_cascades_twice(self):
        g, p = self.g, self.p
        p.library = [card('Moss Diamond'), card('Island'), card('Lyra Dawnbringer')]
        E.cast_card(g, p, hand(p, 'Maelstrom Wanderer'))
        names = [m.name for m in p.perms]
        self.assertIn('Lyra Dawnbringer', names)
        self.assertIn('Moss Diamond', names)
        self.assertEqual(p.stats['jr_wanderer'], 2)

    def test_sisay_grows_with_colours_and_fetches_below_its_power(self):
        g, p = self.g, self.p
        from commander_sim.ai import brain
        s = perm(g, p, 'Sisay, Weatherlight Captain')
        self.assertEqual(E.epow(g, s), 2)
        perm(g, p, 'Dakkon Blackblade')                               # white, blue, black: +3/+3
        self.assertEqual(E.epow(g, s), 5)
        p.library = [card('Lyra Dawnbringer'), card('Mirri, Weatherlight Duelist'), card('Caparocti Sunborn')]
        for n in ('Plains', 'Island', 'Swamp', 'Mountain', 'Forest'): lands(p, n)
        p.cmd_in_zone = False                                         # (Jodah gone for good: no Jodah first)
        o = E.CI.HOOKS['Sisay, Weatherlight Captain']['options'](g, s, p, brain.Situation(g, p), False)
        self.assertEqual([x[1] for x in o], ['Sisay: Caparocti Sunborn'])    # the dearest below 5 (Lyra is 5)
        o[0][2]()
        self.assertIn('Caparocti Sunborn', [m.name for m in p.perms])
        self.assertEqual(p.stats['jr_sisay'], 1)

    def test_sisay_waits_while_jodah_is_castable(self):
        g, p = self.g, self.p
        from commander_sim.ai import brain
        s = perm(g, p, 'Sisay, Weatherlight Captain')
        p.library = [card('Blackblade Reforged')]
        for n in ('Plains', 'Island', 'Swamp', 'Mountain', 'Forest'): lands(p, n)
        self.assertEqual(E.CI.HOOKS['Sisay, Weatherlight Captain']['options'](g, s, p, brain.Situation(g, p), False), [])

    def test_shalai_gives_hexproof_to_jodah_and_you_but_not_herself(self):
        g, p, q = self.g, self.p, self.q
        j = jodah(g, p)
        w = perm(g, p, 'Teferi, Hero of Dominaria')
        sh = perm(g, p, 'Shalai, Voice of Plenty')
        self.assertTrue(E.untargetable(g, j))
        self.assertTrue(E.untargetable(g, w))                         # planeswalkers too
        self.assertFalse(E.untargetable(g, sh))
        self.assertTrue(E.player_hexproof(g, p))
        self.assertFalse(E.player_hexproof(g, q))
        tg = E.legal_targets(g, q, 'exile', 'cp', False, spell=card('Swords to Plowshares'))
        self.assertNotIn(j, tg); self.assertNotIn(w, tg); self.assertIn(sh, tg)
        E.apply_removal(g, q, j, 'exile', card('Swords to Plowshares'))
        self.assertIn(j, p.perms)
        life = p.life
        E.cast_card(g, q, card('Lightning Bolt'), 'hand', {'face': p})  # burn aimed at a hexproof player does nothing
        self.assertEqual(p.life, life)
        E.apply_removal(g, q, sh, 'exile', card('Swords to Plowshares'))
        self.assertFalse(E.untargetable(g, j))                        # Shalai gone: Jodah is exposed again

    def test_shalai_counters_with_mana_left_late(self):
        g, p = self.g, self.p
        from commander_sim.ai import brain
        j = jodah(g, p)
        sh = perm(g, p, 'Shalai, Voice of Plenty')
        lands(p, 'Forest', 2); lands(p, 'Plains', 4)
        hook = E.CI.HOOKS['Shalai, Voice of Plenty']['options']
        self.assertEqual(hook(g, sh, p, brain.Situation(g, p), False), [])   # not in the first main phase
        o = hook(g, sh, p, brain.Situation(g, p), True)
        self.assertEqual([x[1] for x in o], ['Shalai: +1/+1 counters'])
        p0, s0 = E.epow(g, j), E.epow(g, sh)
        o[0][2]()
        self.assertEqual((E.epow(g, j), E.epow(g, sh)), (p0 + 1, s0 + 1))
        self.assertEqual(p.stats['jr_shalai_counters'], 1)

    def test_feed_the_cycle_costs_1bb_and_answers_an_attack(self):
        g, p, q = self.g, self.p, self.q
        from commander_sim.ai import brain
        c = card('Feed the Cycle')
        self.assertEqual((c.generic, c.pips, c.tags['rem'], c.tags['tgt']), (1, 'BB', 'destroy', 'cp'))
        g.active = q
        lyra = perm(g, q, 'Lyra Dawnbringer')
        hand(p, 'Feed the Cycle'); lands(p, 'Swamp', 2)
        self.assertFalse(brain.attack_response(g, p, q, [lyra]))      # {1}{B}: one mana short
        lands(p, 'Swamp')
        self.assertTrue(brain.attack_response(g, p, q, [lyra]))
        self.assertNotIn(lyra, q.perms)


class VenatTymna(unittest.TestCase):
    """swap candidates (10-09): Venat, Heart of Hydaelyn // Hydaelyn, the Mothercrystal and Tymna the Weaver"""
    V = 'Venat, Heart of Hydaelyn'

    def setUp(self):
        self.g = table('jodah', 'veyran', 'seph'); self.p, self.q, self.r = self.g.players

    def test_venat_draws_once_a_turn_for_legendary_spells(self):
        g, p = self.g, self.p
        perm(g, p, self.V)
        n = len(p.hand)
        E.cast_card(g, p, hand(p, 'Mind Stone'))                      # not legendary: nothing
        self.assertEqual(len(p.hand), n)
        E.cast_card(g, p, hand(p, 'Mirri, Weatherlight Duelist'))
        self.assertEqual(len(p.hand), n + 1)
        E.cast_card(g, p, hand(p, 'King Darien XLVIII'))              # the second legend this turn: no card
        self.assertEqual((len(p.hand), p.stats['jr_venat_draws']), (n + 1, 1))
        E.cast_card(g, self.q, card('Lyra Dawnbringer'))              # an opponent's legend: no card
        self.assertEqual(len(p.hand), n + 1)

    def test_venat_front_is_not_indestructible(self):
        g, p = self.g, self.p
        v = perm(g, p, self.V)
        self.assertEqual((E.epow(g, v), E.etgh(g, v)), (3, 3))
        self.assertFalse(E.indestructible(g, v))

    def test_heros_sundering_exiles_and_transforms(self):
        g, p, q = self.g, self.p, self.q
        from commander_sim.ai import brain
        v = perm(g, p, self.V, sick=True)
        hook = E.CI.HOOKS[self.V]['options']
        lyra = perm(g, q, 'Lyra Dawnbringer')
        lands(p, 'Plains', 7)
        self.assertEqual(hook(g, v, p, brain.Situation(g, p), False), [])    # summoning sick: no {T}
        v.sick = False
        self.assertEqual(hook(g, v, p, brain.Situation(g, p), None), [])     # sorcery speed only
        o = hook(g, v, p, brain.Situation(g, p), False)
        self.assertEqual([x[1] for x in o], ["Hero's Sundering -> Lyra Dawnbringer"])
        o[0][2]()
        self.assertNotIn(lyra, q.perms)
        self.assertTrue(v.tapped)
        self.assertEqual((E.epow(g, v), E.etgh(g, v)), (4, 4))
        self.assertTrue(E.indestructible(g, v))
        self.assertEqual(hook(g, v, p, brain.Situation(g, p), False), [])    # once transformed, no more
        n = len(p.hand)
        E.cast_card(g, p, hand(p, 'Mirri, Weatherlight Duelist'))     # Hydaelyn has no cast trigger
        self.assertEqual(len(p.hand), n)

    def test_sundering_waits_for_a_worthy_target(self):
        g, p, q = self.g, self.p, self.q
        from commander_sim.ai import brain
        v = perm(g, p, self.V)
        lands(p, 'Plains', 7)
        token(g, q, 1)
        self.assertEqual(E.CI.HOOKS[self.V]['options'](g, v, p, brain.Situation(g, p), False), [])

    def test_sundering_waits_while_jodah_is_castable(self):
        g, p, q = self.g, self.p, self.q
        from commander_sim.ai import brain
        v = perm(g, p, self.V)
        perm(g, q, 'Lyra Dawnbringer')
        for n in ('Plains', 'Island', 'Swamp', 'Mountain', 'Forest', 'Plains', 'Plains'): lands(p, n)
        hook = E.CI.HOOKS[self.V]['options']
        self.assertEqual(hook(g, v, p, brain.Situation(g, p), False), [])
        jodah(g, p)
        self.assertEqual(len(hook(g, v, p, brain.Situation(g, p), False)), 1)

    def _hydaelyn(self):
        v = perm(self.g, self.p, self.V)
        v.data = {'hydaelyn': True}; v.pow, v.tgh = 4, 4
        return v

    def test_blessing_protects_jodah_first_until_your_next_turn(self):
        g, p, q = self.g, self.p, self.q
        self._hydaelyn()
        lyra = perm(g, p, 'Lyra Dawnbringer')
        j = jodah(g, p)
        n, c0 = len(p.hand), j.plus
        E.CI.fire(g, 'crew', p)
        self.assertEqual((j.plus, lyra.plus), (c0 + 1, 0))
        self.assertEqual(len(p.hand), n + 1)                           # Jodah is legendary: a card
        self.assertTrue(E.indestructible(g, j))
        E.CI.fire(g, 'crew', p)                                        # once a turn
        self.assertEqual(j.plus, c0 + 1)
        g.active = q                                                    # the opponents' turns: still indestructible
        self.assertTrue(E.indestructible(g, j))
        E.apply_removal(g, q, j, 'destroy', card('Swords to Plowshares'))
        self.assertIn(j, p.perms)
        p.turns += 1                                                    # your next turn: it ends
        self.assertFalse(E.indestructible(g, j))
        self.assertEqual((p.stats['jr_hyd_turns'], p.stats['jr_hyd_draws']), (1, 1))

    def test_blessing_prefers_a_legend_and_draws_only_for_one(self):
        g, p = self.g, self.p
        self._hydaelyn()
        perm(g, p, 'Solemn Simulacrum')
        mirri = perm(g, p, 'Mirri, Weatherlight Duelist')             # a smaller legend, but a card
        n = len(p.hand)
        E.CI.fire(g, 'crew', p)
        self.assertEqual((mirri.plus, len(p.hand)), (1, n + 1))
        g2 = table('jodah', 'veyran'); p2 = g2.players[0]
        v = perm(g2, p2, self.V); v.data = {'hydaelyn': True}
        s = perm(g2, p2, 'Solemn Simulacrum')
        n = len(p2.hand)
        E.CI.fire(g2, 'crew', p2)
        self.assertEqual((s.plus, len(p2.hand)), (1, n))               # not legendary: no card

    def test_tymna_pays_a_life_per_opponent_hit(self):
        g, p, q, r = self.g, self.p, self.q, self.r
        t = perm(g, p, 'Tymna the Weaver')
        a = token(g, p, 3)
        n = len(p.hand)
        E.CI.fire(g, 'main2', p)                                        # nobody hit: nothing
        self.assertEqual(len(p.hand), n)
        ais.resolve_combat(g, p, [t], q, set())
        ais.resolve_combat(g, p, [a], r, set())
        life = p.life                                                   # (Tymna's lifelink: +2)
        E.CI.fire(g, 'main2', p)
        self.assertEqual((p.life, len(p.hand), p.stats['jr_tymna_draws']), (life - 2, n + 2, 2))
        E.CI.fire(g, 'main2', p)                                        # once a turn
        self.assertEqual(len(p.hand), n + 2)

    def test_tymna_keeps_low_life(self):
        g, p, q = self.g, self.p, self.q
        perm(g, p, 'Tymna the Weaver')
        a = token(g, p, 3)
        ais.resolve_combat(g, p, [a], q, set())
        p.life = 10                                                     # Necropotence's floor: 10 at least
        n = len(p.hand)
        E.CI.fire(g, 'main2', p)
        self.assertEqual((p.life, len(p.hand)), (10, n))


class Garland(unittest.TestCase):
    """swap candidate (10-09): Garland, Royal Kidnapper"""
    G = 'Garland, Royal Kidnapper'

    def setUp(self):
        self.g = table('jodah', 'veyran', 'seph'); self.p, self.q, self.r = self.g.players

    def test_etb_crowns_the_opponent_with_the_best_creature_and_steals_it(self):
        g, p, q, r = self.g, self.p, self.q, self.r
        lyra = perm(g, q, 'Lyra Dawnbringer')
        s = perm(g, r, 'Solemn Simulacrum')
        self.assertEqual((E.epow(g, lyra), E.etgh(g, lyra)), (5, 5))
        perm(g, p, self.G)
        self.assertIs(g.monarch, q)                                    # Veyran has the creature worth taking
        self.assertIn(lyra, p.perms)
        self.assertNotIn(lyra, q.perms)
        self.assertEqual((E.epow(g, lyra), E.etgh(g, lyra)), (7, 7))   # +2/+2: controlled, not owned
        self.assertIn(s, r.perms)
        self.assertEqual(p.stats['jr_garland_steals'], 1)

    def test_control_returns_when_that_opponent_loses_the_monarch(self):
        g, p, q, r = self.g, self.p, self.q, self.r
        lyra = perm(g, q, 'Lyra Dawnbringer')
        perm(g, p, self.G)
        a = token(g, r, 2)
        ais.resolve_combat(g, r, [a], q, {a})                           # Sephiroth takes the crown from Veyran
        self.assertIs(g.monarch, r)
        self.assertIn(lyra, q.perms)
        self.assertNotIn(lyra, p.perms)
        self.assertEqual((E.epow(g, lyra), E.etgh(g, lyra)), (5, 5))
        self.assertEqual(p.stats['jr_garland_back'], 1)

    def test_an_opponent_taking_the_monarch_from_you_triggers_a_steal(self):
        g, p, q, r = self.g, self.p, self.q, self.r
        perm(g, p, self.G)                                              # no creatures yet: nothing to take
        self.assertIs(g.monarch, q)
        E.CI.become_monarch(g, p)                                       # (you take it back: no trigger for you)
        self.assertEqual(p.stats['jr_garland_steals'], 0)
        s = perm(g, r, 'Solemn Simulacrum')
        a = token(g, r, 1)
        ais.resolve_combat(g, r, [a], p, {a})                           # Sephiroth hits you and takes the crown
        self.assertIs(g.monarch, r)
        self.assertIn(s, p.perms)                                       # the more valuable of Sephiroth's creatures
        self.assertEqual(p.stats['jr_garland_steals'], 1)

    def test_stolen_creatures_cant_be_sacrificed(self):
        g, p, q = self.g, self.p, self.q
        t = token(g, q, 1)
        gar = perm(g, p, self.G)
        self.assertIn(t, p.perms)
        self.assertEqual((E.epow(g, t), E.etgh(g, t)), (3, 3))
        E.die(g, t, 'sac')
        self.assertIn(t, p.perms)
        self.assertIsNone(E.sac_fodder(g, p, 'creature', exclude=gar))
        E.edict(g, p)                                                   # not the stolen 1/1: Garland has to go
        self.assertIn(t, p.perms)
        self.assertNotIn(gar, p.perms)
        self.assertEqual((E.epow(g, t), E.etgh(g, t)), (1, 1))         # Garland gone: no bonus, still yours
        E.edict(g, p)                                                   # ... and it can be sacrificed again
        self.assertNotIn(t, p.perms)

    def test_the_steal_outlasts_garland_and_ends_with_the_owner(self):
        g, p, q = self.g, self.p, self.q
        lyra = perm(g, q, 'Lyra Dawnbringer')
        gar = perm(g, p, self.G)
        E.die(g, gar)
        self.assertIn(lyra, p.perms)                                    # "for as long as they're the monarch"
        q.life = 0; E.check_state(g)                                    # Veyran leaves the game: so does its card
        self.assertNotIn(lyra, p.perms)
        self.assertFalse(g.garland)


if __name__ == '__main__':
    unittest.main()
