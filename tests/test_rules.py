"""Core rules, checked on hand-built positions: mana, state-based checks, counterspells, removal, wipes, tutors,
commander tax, mulligans and combat. Each expectation comes from the game rules or the card's Oracle text."""
import unittest
from tests.table import table, hand, lands, perm, token
from commander_sim import engine as E, ais

C = E.DB


class Mana(unittest.TestCase):
    def test_colours_must_match(self):
        g = table('veyran', 'seph'); v = g.players[0]
        lands(v, 'Island', 2); lands(v, 'Mountain')
        self.assertTrue(E.can_pay(g, v, 1, 'UR'))
        self.assertFalse(E.can_pay(g, v, 0, 'RR'))
        self.assertFalse(E.can_pay(g, v, 0, 'UUU'))
        self.assertFalse(E.can_pay(g, v, 3, 'U'))            # four mana from three lands

    def test_paying_taps_the_sources(self):
        g = table('veyran', 'seph'); v = g.players[0]
        lands(v, 'Island', 2); lands(v, 'Mountain')
        self.assertTrue(E.pay(g, v, 0, 'UU'))
        self.assertEqual([L.tapped for L in v.lands], [True, True, False])
        self.assertFalse(E.pay(g, v, 0, 'U'))               # nothing blue left

    def test_sol_ring_makes_two(self):
        g = table('veyran', 'seph'); v = g.players[0]
        perm(g, v, 'Sol Ring')
        self.assertEqual(E.total_mana(g, v), 2)

    def test_treasure_is_spent_last(self):
        g = table('seph', 'veyran'); s = g.players[0]
        lands(s, 'Swamp'); E.add_treasure(g, s, 1)
        E.pay(g, s, 1, '')
        self.assertEqual(s.treasures, 1)
        self.assertTrue(s.lands[0].tapped)

    def test_talisman_hurts_only_for_coloured_mana(self):
        g = table('seph', 'veyran'); s = g.players[0]
        perm(g, s, 'Talisman of Hierarchy'); E.pay(g, s, 1, '')
        self.assertEqual(s.life, 40)
        g = table('seph', 'veyran'); s = g.players[0]
        perm(g, s, 'Talisman of Hierarchy'); E.pay(g, s, 0, 'B')
        self.assertEqual(s.life, 39)

    def test_commander_tax(self):
        g = table('seph', 'veyran'); s = g.players[0]
        self.assertEqual(E.cost_of(s, s.cmd), (3, 'GWUB'))  # Atraxa, Grand Unifier
        s.tax = 4
        self.assertEqual(E.cost_of(s, s.cmd), (7, 'GWUB'))


class StateChecks(unittest.TestCase):
    def test_zero_life_eliminates(self):
        g = table('seph', 'veyran', 'sauron'); b = g.players[1]
        b.life = 0; E.check_state(g)
        self.assertFalse(b.alive); self.assertFalse(g.over)

    def test_21_commander_damage_eliminates(self):
        g = table('seph', 'veyran'); a, b = g.players
        b.cmd_dmg[a.key] = 21; E.check_state(g)
        self.assertFalse(b.alive); self.assertTrue(g.over); self.assertIs(g.winner, a)

    def test_20_commander_damage_does_not(self):
        g = table('seph', 'veyran'); a, b = g.players
        b.cmd_dmg[a.key] = 20; E.check_state(g)
        self.assertTrue(b.alive)

    def test_drawing_from_an_empty_library_loses(self):
        g = table('seph', 'veyran'); b = g.players[1]
        b.library = []; E.draw(g, b, 1); E.check_state(g)
        self.assertFalse(b.alive)

    def test_ten_poison_counters_lose(self):
        g = table('seph', 'veyran'); b = g.players[1]
        b.poison = 10; E.check_state(g)
        self.assertFalse(b.alive)


class Counterspells(unittest.TestCase):
    def test_what_each_counter_can_hit(self):
        titan, tutor, ring = C['Grave Titan'], C['Demonic Tutor'], C['Sol Ring']
        self.assertTrue(E.counter_ok(C['Counterspell'], titan))
        self.assertFalse(E.counter_ok(C['Spell Pierce'], titan))            # noncreature only
        self.assertTrue(E.counter_ok(C['Spell Pierce'], tutor))
        self.assertTrue(E.counter_ok(C['Disdainful Stroke'], titan))        # mana value 4 or more
        self.assertFalse(E.counter_ok(C['Disdainful Stroke'], ring))
        self.assertFalse(E.counter_ok(C["Dovin's Veto"], titan))            # noncreature only
        self.assertTrue(E.counter_ok(C['Swan Song'], C['Aura Shards']))     # enchantment
        self.assertFalse(E.counter_ok(C['Swan Song'], titan))

    def test_a_counter_needs_its_mana(self):
        g = table('veyran', 'seph'); v = g.players[0]
        hand(v, 'Counterspell'); lands(v, 'Island', 2)
        self.assertEqual(E.pick_counter(g, v, C['Grave Titan']).name, 'Counterspell')
        v.lands[0].tapped = True
        self.assertIsNone(E.pick_counter(g, v, C['Grave Titan']))

    def test_force_of_will_exiles_a_blue_card_and_one_life(self):
        g = table('veyran', 'seph'); v = g.players[0]
        fow, blue = hand(v, 'Force of Will', 'Think Twice')
        self.assertIs(E.pick_counter(g, v, C['Grave Titan']), fow)
        self.assertTrue(E.cast_counter(g, v, fow))
        self.assertEqual(v.life, 39)
        self.assertEqual([c.name for c in v.exile], ['Think Twice'])
        self.assertEqual(v.hand, [])


class Removal(unittest.TestCase):
    def test_swords_exiles_and_its_controller_gains_its_power(self):
        g = table('seph', 'sauron'); s, r = g.players
        t = perm(g, r, 'Grave Titan')                         # 6/6
        E.apply_removal(g, s, t, 'exile', C['Swords to Plowshares'])
        self.assertNotIn(t, r.perms)
        self.assertEqual([c.name for c in r.exile], ['Grave Titan'])
        self.assertEqual(r.life, 46)

    def test_path_gives_a_tapped_land(self):
        g = table('seph', 'sauron'); s, r = g.players
        t = perm(g, r, 'Hellkite Tyrant')
        E.apply_removal(g, s, t, 'exile', C['Path to Exile'])
        self.assertEqual([L.tapped for L in r.lands], [True])

    def test_bolt_kills_three_toughness_only(self):
        g = table('veyran', 'sauron'); v, r = g.players
        small = perm(g, r, "Jace's Archivist")                # 2/3
        big = perm(g, r, 'Hellkite Tyrant')                   # 6/5
        self.assertEqual(E.legal_targets(g, v, 'dmg3', 'c', False, C['Lightning Bolt']), [small])
        E.apply_removal(g, v, small, 'dmg3', C['Lightning Bolt'])
        self.assertNotIn(small, r.perms); self.assertIn(big, r.perms)
        self.assertEqual([c.name for c in r.gy], ["Jace's Archivist"])

    def test_a_removed_token_leaves_no_card(self):
        g = table('veyran', 'sauron'); v, r = g.players
        t = token(g, r, 2)
        E.apply_removal(g, v, t, 'destroy', C['Pongify'])
        self.assertNotIn(t, r.perms); self.assertEqual(r.gy, [])

    def test_toxic_deluge_kills_creatures_only(self):
        g = table('seph', 'sauron'); s, r = g.players
        perm(g, r, 'Grave Titan'); perm(g, s, 'Blood Artist'); perm(g, r, 'Sol Ring')
        E.apply_wipe(g, s, 'minus', {})
        self.assertEqual([m.name for q in g.players for m in q.perms], ['Sol Ring'])


class Tutors(unittest.TestCase):
    def test_a_tutor_moves_one_card_from_library_to_hand(self):
        g = table('seph', 'veyran'); s = g.players[0]
        lib = list(s.library)
        E.tutor(g, s, 'any')
        self.assertEqual(len(s.hand), 1)
        self.assertEqual(len(s.library), len(lib) - 1)
        self.assertIn(s.hand[0], lib)


class Mulligan(unittest.TestCase):
    def test_kept_hands(self):
        import random
        for seed in range(40):
            g = table('seph', 'veyran')
            p = g.players[0]
            ais.mulligan(g, p, random.Random(seed))
            self.assertEqual(len(p.hand) + len(p.library), 99)            # the commander is in the command zone
            self.assertIn(len(p.hand), (5, 6, 7))
            if len(p.hand) == 7 and 'mulls' not in p.stats:
                self.assertTrue(2 <= sum(1 for c in p.hand if c.land) <= 5)


class Combat(unittest.TestCase):
    def test_unblocked_commander_deals_commander_damage(self):
        g = table('sauron', 'veyran'); r, v = g.players
        w = perm(g, r, 'Witch-king, Bringer of Ruin'); w.is_cmd = True   # 5/3 flying
        ais.resolve_combat(g, r, [w], v, set())
        self.assertEqual(v.life, 35)
        self.assertEqual(v.cmd_dmg[r.key], 5)

    def test_flyers_need_flying_or_reach_to_block(self):
        g = table('sauron', 'veyran'); r, v = g.players
        blocker = perm(g, v, 'Murmuring Mystic')
        self.assertFalse(ais.can_block(g, blocker, perm(g, r, 'Witch-king, Bringer of Ruin')))
        self.assertTrue(ais.can_block(g, blocker, perm(g, r, 'Grave Titan')))

    def test_deathtouch_blocker_kills_a_big_attacker(self):
        g = table('sauron', 'seph'); r, s = g.players
        k = perm(g, r, 'Kaervek the Merciless')                          # 5/4
        imp = perm(g, s, 'Stinkweed Imp')                                 # 1/2 flying, deathtouch
        ais.resolve_combat(g, r, [k], s, set())
        self.assertNotIn(k, r.perms); self.assertNotIn(imp, s.perms)
        self.assertEqual(s.life, 40)

    def test_lifelink(self):
        g = table('seph', 'veyran'); s, v = g.players
        a = perm(g, s, 'Atraxa, Grand Unifier')                           # 7/7 lifelink
        s.life = 30
        ais.resolve_combat(g, s, [a], v, set())
        self.assertEqual((s.life, v.life), (37, 33))


if __name__ == '__main__':
    unittest.main()
