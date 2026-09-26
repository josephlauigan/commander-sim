"""Key cards of your three decks, checked against their Oracle text on hand-built positions."""
import unittest
from tests.table import table, hand, lands, perm
from commander_sim import engine as E, ais

C = E.DB


def lives(g):
    return [p.life for p in g.players]


class Sephiroth(unittest.TestCase):
    def test_sheoldred_the_apocalypse(self):
        # "Whenever you draw a card, you gain 2 life. Whenever an opponent draws a card, they lose 2 life."
        g = table('seph', 'veyran'); s, v = g.players
        perm(g, s, 'Sheoldred, the Apocalypse')
        E.draw(g, v, 1)
        self.assertEqual(lives(g), [40, 38])                  # the opponent's draw gains you nothing
        E.draw(g, s, 1)
        self.assertEqual(lives(g), [42, 38])

    def test_massacre_wurm(self):
        # "When this enters, creatures your opponents control get -2/-2 until end of turn.
        #  Whenever a creature an opponent controls dies, that player loses 2 life."
        g = table('seph', 'veyran'); s, v = g.players
        perm(g, v, 'Guttersnipe'); mystic = perm(g, v, 'Murmuring Mystic')     # 2/2 dies, 1/5 lives
        perm(g, s, 'Massacre Wurm')
        self.assertEqual([m for m in v.perms], [mystic])
        self.assertEqual(v.life, 38)                          # one death, 2 life

    def test_archon_of_cruelty_enters(self):
        # opponent sacrifices a creature, discards a card and loses 3; you draw a card and gain 3
        g = table('seph', 'veyran'); s, v = g.players
        perm(g, v, 'Guttersnipe'); hand(v, 'Lightning Bolt', 'Think Twice')
        perm(g, s, 'Archon of Cruelty')
        self.assertEqual((s.life, len(s.hand)), (43, 1))
        self.assertEqual((v.life, len(v.hand), len(v.perms)), (37, 1, 0))

    def test_gray_merchant_drains_devotion(self):
        g = table('seph', 'veyran', 'sauron'); s = g.players[0]
        perm(g, s, 'Sheoldred, Whispering One')              # {5}{B}{B}: devotion 2 + Gray Merchant's 2
        perm(g, s, 'Gray Merchant of Asphodel')
        self.assertEqual(lives(g), [48, 36, 36])

    def test_blood_artist(self):
        g = table('seph', 'veyran'); s, v = g.players
        perm(g, s, 'Blood Artist')
        E.die(g, perm(g, v, 'Guttersnipe'))
        self.assertEqual(lives(g), [41, 39])

    def test_elesh_norn_grand_cenobite(self):
        g = table('seph', 'veyran'); s, v = g.players
        mystic = perm(g, v, 'Murmuring Mystic'); artist = perm(g, s, 'Blood Artist')
        perm(g, s, 'Elesh Norn, Grand Cenobite')
        self.assertEqual(E.etgh(g, mystic), 3)                # 1/5 gets -2/-2
        self.assertEqual((E.epow(g, artist), E.etgh(g, artist)), (2, 3))   # 0/1 gets +2/+2

    def test_sephiroth_planets_heir(self):
        # enters: opponents' creatures get -2/-2; a counter whenever a creature an opponent controls dies
        g = table('seph', 'veyran'); s, v = g.players
        snipe = perm(g, v, 'Guttersnipe')
        heir = perm(g, s, "Sephiroth, Planet's Heir")
        self.assertNotIn(snipe, v.perms)
        E.die(g, perm(g, v, 'Kessig Flamebreather'))
        self.assertEqual(heir.plus, 2)

    def test_grave_titan(self):
        g = table('seph', 'veyran'); s, v = g.players
        t = perm(g, s, 'Grave Titan')
        self.assertEqual(sum(1 for m in s.perms if m.token), 2)          # enters with two Zombies
        ais.attack_triggers(g, s, [t], v)
        self.assertEqual(sum(1 for m in s.perms if m.token), 4)          # and two more when it attacks

    def test_atraxa_takes_one_card_of_each_type(self):
        g = table('seph', 'veyran'); s = g.players[0]
        top = s.library[-10:]
        perm(g, s, 'Atraxa, Grand Unifier')
        self.assertTrue(set(map(id, s.hand)) <= set(map(id, top)))
        kinds = [c.types for c in s.hand]
        self.assertEqual(len(kinds), len(set(kinds)))

    def test_consecrated_sphinx(self):
        g = table('seph', 'veyran'); s, v = g.players
        perm(g, s, 'Consecrated Sphinx')
        E.draw(g, v, 1)
        self.assertEqual(len(s.hand), 2)


class Veyran(unittest.TestCase):
    def test_magecraft_pings_each_opponent(self):
        g = table('veyran', 'seph', 'sauron'); v = g.players[0]
        perm(g, v, 'Guttersnipe')
        E.magecraft(g, v, C['Lightning Bolt'])
        self.assertEqual(lives(g), [40, 38, 38])

    def test_veyran_doubles_magecraft(self):
        g = table('veyran', 'seph', 'sauron'); v = g.players[0]
        perm(g, v, 'Veyran, Voice of Duality'); perm(g, v, 'Guttersnipe')
        E.magecraft(g, v, C['Lightning Bolt'])
        self.assertEqual(lives(g), [40, 36, 36])

    def test_archmage_emeritus_with_veyran_draws_two(self):
        g = table('veyran', 'seph'); v = g.players[0]
        perm(g, v, 'Archmage Emeritus'); perm(g, v, 'Veyran, Voice of Duality')
        E.magecraft(g, v, C['Think Twice'])
        self.assertEqual(len(v.hand), 2)

    def test_rite_of_the_dragoncaller(self):
        g = table('veyran', 'seph'); v = g.players[0]
        perm(g, v, 'Rite of the Dragoncaller')
        E.magecraft(g, v, C['Lightning Bolt'])
        self.assertEqual([(m.pow, m.tgh, m.fly) for m in v.perms if m.token], [(5, 5, True)])

    def test_aetherflux_reservoir(self):
        # gain 1 life for each spell you've cast before it this turn
        g = table('veyran', 'seph'); v = g.players[0]
        perm(g, v, 'Aetherflux Reservoir')
        for name in ('Lightning Bolt', 'Think Twice', 'Counterspell'):
            v.spells_this_turn += 1; E.on_cast(g, v, C[name])
        self.assertEqual(v.life, 46)

    def test_jin_gitaxias_copies_the_first_spell(self):
        g = table('veyran', 'seph'); v = g.players[0]
        perm(g, v, 'Jin-Gitaxias, Progress Tyrant')
        E.on_cast(g, v, C['Quick Study'])                     # the copy resolves: draw two
        self.assertEqual(len(v.hand), 2)
        E.on_cast(g, v, C['Quick Study'])                     # once each turn
        self.assertEqual(len(v.hand), 2)

    def test_emeritus_of_ideation_draws_nothing_on_entering(self):
        g = table('veyran', 'seph'); v = g.players[0]
        perm(g, v, 'Emeritus of Ideation // Ancestral Recall')
        self.assertEqual(v.hand, [])


class Sauron(unittest.TestCase):
    def test_sauron_amasses_when_an_opponent_casts(self):
        g = table('sauron', 'veyran'); r, v = g.players
        perm(g, r, 'Sauron, the Dark Lord')
        E.on_cast(g, v, C['Lightning Bolt']); E.on_cast(g, v, C['Think Twice'])
        self.assertEqual(E.epow(g, E.army_of(r)), 2)

    def test_kaervek(self):
        g = table('sauron', 'veyran'); r, v = g.players
        perm(g, r, 'Kaervek the Merciless')
        E.on_cast(g, v, C['Counterspell'])                    # mana value 2
        self.assertEqual(v.life, 38)

    def test_witch_king_takes_the_least_power(self):
        # "defending player sacrifices a creature with the least power among creatures they control"
        g = table('sauron', 'veyran'); r, v = g.players
        w = perm(g, r, 'Witch-king, Bringer of Ruin')
        snipe = perm(g, v, 'Guttersnipe'); kessig = perm(g, v, 'Kessig Flamebreather')     # power 2 and 1
        ais.attack_triggers(g, r, [w], v)
        self.assertEqual(v.perms, [snipe])

    def test_orcish_bowmasters(self):
        g = table('sauron', 'veyran'); r, v = g.players
        perm(g, r, 'Orcish Bowmasters')
        self.assertEqual(v.life, 39)                          # enters: 1 damage, then amass 1
        self.assertIsNotNone(E.army_of(r))
        E.draw(g, v, 1, step=True)                            # the first card of a draw step doesn't count
        self.assertEqual(v.life, 39)
        E.draw(g, v, 1)
        self.assertEqual(v.life, 38)

    def test_rhystic_study(self):
        g = table('sauron', 'veyran'); r, v = g.players
        perm(g, r, 'Rhystic Study')
        E.on_cast(g, v, C['Lightning Bolt'])                  # the caster has no mana to pay {1}
        self.assertEqual(len(r.hand), 1)

    def test_phyrexian_arena(self):
        g = table('sauron', 'veyran'); r = g.players[0]
        perm(g, r, 'Phyrexian Arena')
        ais.upkeep(g, r)
        self.assertEqual((len(r.hand), r.life), (1, 39))


if __name__ == '__main__':
    unittest.main()
