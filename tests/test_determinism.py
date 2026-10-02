"""The same seed plays the same game wherever the objects land in memory (step 3a of practice mode: saved games and
Undo depend on it). Cards, players, permanents and lands hash without their addresses, and a game keeps every
permanent and land it made alive until it ends, so no new object can take a dead one's address (and with it the
entries tables keyed by id() still hold for the dead one)."""
import unittest, zlib
from tests.table import table, perm
from commander_sim import engine as E
from commander_sim.ai import search


class Determinism(unittest.TestCase):
    def test_hashes_do_not_depend_on_addresses(self):
        g = table('sauron', 'veyran'); s, v = g.players
        m = perm(g, s, 'Orcish Bowmasters')
        self.assertEqual(hash(E.DB['Sol Ring']), zlib.crc32(b'Sol Ring'))
        self.assertEqual(hash(s), zlib.crc32(b'sauron'))
        self.assertEqual(hash(m), m.hid)
        g2 = search.clone(g)
        m2 = next(x for x in g2.players[0].perms if x.name == 'Orcish Bowmasters')
        self.assertIsNot(m2, m); self.assertEqual(hash(m2), hash(m))          # a copy hashes like its original

    def test_a_dead_permanents_address_is_not_reused(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        m = perm(g, s, 'Grave Titan')
        old = id(m)
        g.eot_pt[old] = (-5, -5)                                            # an end-of-turn -5/-5 on it
        E.die(g, m, 'destroy'); del m
        new = [E.Perm(s, E.DB['Carrion Feeder']) for _ in range(200)]
        self.assertNotIn(old, {id(x) for x in new})                       # it is still alive: nothing takes its address

    def test_copies_keep_their_objects_alive(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        perm(g, s, 'Grave Titan')
        g2 = search.clone(g)
        self.assertTrue(any(x is g2.players[0].perms[0] for x in g2.alive_objs))


    def test_a_copy_of_a_copy_leaves_the_real_game_alone(self):
        """a playout copy that is copied again (the Breach line's dry run inside a look-ahead playout) must not copy
        the game it came from, which holds the practice-mode seat (threads and queues can't be copied)"""
        from commander_sim.play.controller import HumanController
        g = table('sauron', 'veyran'); s = g.players[0]
        perm(g, s, 'Grave Titan')
        g.controllers = {s.key: HumanController()}
        g2 = search.clone(g)
        search._start(g2, g)                                   # a playout of g: g2.search_parent is g
        g3 = search.clone(g2)
        self.assertIsNone(getattr(g3, 'search_parent', None))
        self.assertIs(g2.search_parent, g)
        self.assertFalse(any(x is g for x in g3.alive_objs))

if __name__ == '__main__':
    unittest.main()
