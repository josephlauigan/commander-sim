"""Key cards of your four decks, checked against their Oracle text on hand-built positions."""
import unittest
from tests.table import table, hand, lands, perm, token, card
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

    # --- the four loops: Mikaeus + Triskelion, Mikaeus or Melira + Kitchen Finks, Nim Deathmantle + Ashnod's Altar + Grave Titan
    def loops(self, *cards, opp=()):
        from commander_sim.cards.impl import mine
        g = table('seph', 'veyran', 'sauron'); s = g.players[0]
        for c in cards: perm(g, s, c)
        for c in opp: perm(g, g.players[1], c)
        return g, s, {n: kills for n, keys, kills in mine.seph_loops(g, s)}

    def test_mikaeus_triskelion_kills_alone(self):
        g, s, loops = self.loops('Mikaeus, the Unhallowed', 'Triskelion', 'Viscera Seer')
        self.assertEqual(loops, {'Mikaeus + Triskelion': True})

    def test_finks_loops_need_a_payoff(self):
        for enabler in ('Mikaeus, the Unhallowed', 'Melira, Sylvok Outcast'):
            g, s, loops = self.loops(enabler, 'Kitchen Finks', "Ashnod's Altar")
            self.assertEqual(list(loops.values()), [False])
            g, s, loops = self.loops(enabler, 'Kitchen Finks', "Ashnod's Altar", 'Zulaport Cutthroat')
            self.assertEqual(list(loops.values()), [True])
            g, s, loops = self.loops(enabler, 'Kitchen Finks', 'Altar of Dementia')      # mills everyone
            self.assertEqual(list(loops.values()), [True])

    def test_deathmantle_loop(self):
        g, s, loops = self.loops('Nim Deathmantle', "Ashnod's Altar", 'Grave Titan')
        self.assertEqual(list(loops.values()), [False])
        g, s, loops = self.loops('Nim Deathmantle', "Ashnod's Altar", 'Grave Titan', 'Blood Artist')
        self.assertEqual(list(loops.values()), [True])
        g, s, loops = self.loops('Nim Deathmantle', "Ashnod's Altar", 'Grave Titan', 'Triskelion')
        self.assertEqual(list(loops.values()), [True])            # infinite mana keeps returning Triskelion

    def test_finks_loops_need_an_outlet(self):
        g, s, loops = self.loops('Mikaeus, the Unhallowed', 'Kitchen Finks', 'Melira, Sylvok Outcast', 'Blood Artist')
        self.assertEqual(loops, {})

    def test_mikaeus_triskelion_needs_no_outlet(self):
        # undying returns it with four counters: two pings at itself, two at opponents, and it dies a 2/2 with 2 damage
        g, s, loops = self.loops('Mikaeus, the Unhallowed', 'Triskelion')
        self.assertEqual(loops, {'Mikaeus + Triskelion': True})

    def test_a_bigger_triskelion_needs_an_outlet(self):
        # Elesh Norn, Grand Cenobite makes it a counterless 4/4, and Nim Deathmantle on it a 6/6: all four pings (or more)
        # go to itself, so only a free sacrifice outlet keeps it looping
        g, s, loops = self.loops('Mikaeus, the Unhallowed', 'Triskelion', 'Elesh Norn, Grand Cenobite')
        self.assertEqual(loops, {})
        g, s, loops = self.loops('Mikaeus, the Unhallowed', 'Triskelion', 'Elesh Norn, Grand Cenobite', 'Viscera Seer')
        self.assertEqual(loops, {'Mikaeus + Triskelion': True})
        g, s, loops = self.loops('Mikaeus, the Unhallowed', 'Triskelion', 'Nim Deathmantle')
        next(m for m in s.perms if m.name == 'Nim Deathmantle').attached = next(m for m in s.perms if m.name == 'Triskelion')
        from commander_sim.cards.impl import mine
        self.assertEqual({n for n, _, _ in mine.seph_loops(g, s)}, set())

    def test_mikaeus_alone_wants_just_triskelion(self):
        from commander_sim.cards.impl import mine
        g, s, _ = self.loops('Mikaeus, the Unhallowed')
        self.assertEqual(mine.loop_need(g, s), ['Triskelion'])
        g, s, _ = self.loops('Mikaeus, the Unhallowed', 'Elesh Norn, Grand Cenobite', 'Triskelion')
        self.assertEqual(mine.loop_need(g, s), list(mine.OUTLETS))

    def test_rest_in_peace_stops_the_loops(self):
        g, s, loops = self.loops('Mikaeus, the Unhallowed', 'Triskelion', 'Viscera Seer', opp=('Rest in Peace',))
        self.assertEqual(loops, {})

    def test_the_ai_goes_for_a_killing_loop(self):
        from commander_sim.ai import brain
        g, s, _ = self.loops('Mikaeus, the Unhallowed', 'Triskelion', 'Viscera Seer')
        opts = {l: f for u, l, f in brain.main_options(g, s, False)}
        opts['loop: Mikaeus + Triskelion']()
        self.assertTrue(g.over); self.assertIs(g.winner, s)

    def test_a_loop_without_payoff_is_infinite_life(self):
        from commander_sim.cards.impl import mine
        g, s, _ = self.loops('Melira, Sylvok Outcast', 'Kitchen Finks', "Ashnod's Altar")
        (name, keys, kills), = mine.seph_loops(g, s)
        mine.run_loop(g, s, name, keys, kills)
        self.assertFalse(g.over); self.assertGreater(s.life, 1000)

    def test_aura_shards_clears_artifacts_and_enchantments(self):
        from commander_sim.cards.impl import mine
        g, s, _ = self.loops('Melira, Sylvok Outcast', 'Kitchen Finks', "Ashnod's Altar", 'Aura Shards')
        v = g.players[1]
        perm(g, v, 'Sol Ring'); perm(g, v, 'Rite of the Dragoncaller'); snipe = perm(g, v, 'Guttersnipe')
        (name, keys, kills), = mine.seph_loops(g, s)
        mine.run_loop(g, s, name, keys, kills)
        self.assertEqual(v.perms, [snipe])

    def test_tutors_find_the_last_piece(self):
        g, s, _ = self.loops('Mikaeus, the Unhallowed', 'Viscera Seer')
        self.assertEqual(ais.seph_tutor_target(g, s), 'Triskelion')

    def test_reanimation_takes_the_last_piece(self):
        g, s, _ = self.loops('Melira, Sylvok Outcast', 'Viscera Seer', 'Blood Artist')
        s.gy.append(C['Kitchen Finks'])
        self.assertEqual(ais.rean_targets(g, s, 'animate')[0][1].name, 'Kitchen Finks')

    def test_a_piece_that_completes_a_loop_is_cast_first(self):
        g, s, _ = self.loops('Mikaeus, the Unhallowed', 'Viscera Seer')
        self.assertEqual(ais.seph_prio(g, s, hand(s, 'Triskelion')), 85)

    def test_kitchen_finks_persist(self):
        g = table('seph', 'veyran'); s = g.players[0]
        finks = perm(g, s, 'Kitchen Finks')
        self.assertEqual(s.life, 42)                          # enters: gain 2
        E.die(g, finks, 'sac')
        back = next(m for m in s.perms if m.name == 'Kitchen Finks')
        self.assertEqual((back.plus, s.life), (-1, 44))      # persist: back with a -1/-1 counter
        E.die(g, back, 'sac')
        self.assertFalse(any(m.name == 'Kitchen Finks' for m in s.perms))

    def test_melira_keeps_persist_going(self):
        g = table('seph', 'veyran'); s = g.players[0]
        perm(g, s, 'Melira, Sylvok Outcast')
        finks = perm(g, s, 'Kitchen Finks')
        for _ in range(3):
            E.die(g, finks, 'sac')
            finks = next(m for m in s.perms if m.name == 'Kitchen Finks')
            self.assertEqual(finks.plus, 0)
        self.assertFalse(E.minus_counter(g, finks))           # no -1/-1 counters on your creatures (Yawgmoth, Persist)
        self.assertTrue(E.melira(s))

    def test_finks_at_zero_toughness_under_melira_stays_dead(self):
        # Kitchen Finks (3/2) enters under an opponent's Elesh Norn (-2/-2): it dies at once, and under Melira persist
        # returns it without a counter every time, a mandatory loop. It used to recurse until Python gave up.
        g = table('seph', 'veyran'); s, v = g.players
        melira = perm(g, s, 'Melira, Sylvok Outcast'); melira.plus = 2           # 4/4: she survives the Norn
        perm(g, v, 'Elesh Norn, Grand Cenobite')
        perm(g, s, 'Kitchen Finks')
        self.assertIn(melira, s.perms)
        self.assertFalse(any(m.name == 'Kitchen Finks' for m in s.perms))
        self.assertTrue(any(c.name == 'Kitchen Finks' for c in s.gy))

    def test_teferis_protection_against_a_wipe(self):
        g = table('sauron', 'seph'); r, s = g.players
        lands(s, 'Plains', 3); hand(s, "Teferi's Protection"); perm(g, s, 'Grave Titan'); perm(g, s, 'Archon of Cruelty')
        self.assertEqual(ais.wipe_response(g, s, 'destroy', r), 'all')
        self.assertTrue(s.life_locked)

    def test_sephiroth_casts_necropotence_while_healthy(self):
        # Necropotence keeps life above what the table could hit you for (necro_floor: the biggest opposing board plus
        # 6, at least 10); it's cast only when the life above that floor buys two or more cards
        g = table('seph', 'veyran'); s = g.players[0]
        self.assertGreater(ais.seph_prio(g, s, C['Necropotence']), 0)
        s.life = 11
        self.assertEqual(ais.seph_prio(g, s, C['Necropotence']), 0)
        s.life = 30
        for _ in range(3): perm(g, g.players[1], 'Grave Titan')
        self.assertEqual(ais.seph_prio(g, s, C['Necropotence']), 0)        # three Titans: the floor is above 30

    def test_avacyns_pilgrim(self):
        g = table('seph', 'veyran'); s = g.players[0]
        perm(g, s, "Avacyn's Pilgrim")
        self.assertTrue(E.can_pay(g, s, 0, 'W'))


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

    def twinflame(self, n_lands, *creatures, spells=('Lightning Bolt', 'Burst Lightning')):
        from commander_sim.ai import brain
        g = table('veyran', 'seph', 'sauron'); v = g.players[0]
        lands(v, 'Mountain', n_lands)
        for c in creatures: perm(g, v, c)
        hand(v, 'Twinflame', *spells)
        opts = {l: f for u, l, f in brain.main_options(g, v, False) if l.startswith('Twinflame')}
        return g, v, opts

    def test_twinflame_copies_with_haste_until_the_end_step(self):
        g, v, opts = self.twinflame(4, 'Guttersnipe', 'Veyran, Voice of Duality')
        list(opts.values())[0]()
        copies = [m for m in v.perms if m.token]
        self.assertEqual([m.name for m in copies], ['Guttersnipe'])
        self.assertFalse(copies[0].sick)                                 # haste
        self.assertEqual(sum(L.tapped for L in v.lands), 2)             # {1}{R}: mana left for the burn spells
        ais.end_step(g, v)
        self.assertFalse(any(m.token for m in v.perms))                  # exiled at the end step

    def test_twinflame_strive_cost(self):
        g, v, opts = self.twinflame(8, 'Guttersnipe', 'Kessig Flamebreather', 'Thunderdrum Soloist')
        self.assertIn('Twinflame (2 targets)', opts)
        opts['Twinflame (2 targets)']()
        self.assertEqual(sum(L.tapped for L in v.lands), 5)             # {1}{R} + {2}{R}

    def test_twinflame_skips_legends_and_idle_copies(self):
        g, v, opts = self.twinflame(4, 'Veyran, Voice of Duality')
        self.assertEqual(opts, {})                                      # a copy of a legend dies to the legend rule
        g, v, opts = self.twinflame(2, 'Guttersnipe')
        self.assertEqual(opts, {})                                      # no mana left to trigger the copy

    def test_unsummon_returns_a_creature(self):
        from commander_sim.ai import brain
        g = table('veyran', 'sauron'); v, r = g.players
        lands(v, 'Island'); hand(v, 'Unsummon'); k = perm(g, r, 'Kaervek the Merciless')
        opts = {l: f for u, l, f in brain.removal_options(g, v, brain.Situation(g, v))}
        opts['Unsummon -> Kaervek the Merciless']()
        self.assertNotIn(k, r.perms); self.assertIn(C['Kaervek the Merciless'], r.hand)


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

    def test_champions_helm_goes_on_a_legend(self):
        # "Equipped creature gets +2/+2. As long as equipped creature is legendary, it has hexproof. Equip {1}"
        from commander_sim.ai import brain
        g = table('sauron', 'veyran'); r = g.players[0]
        lands(r, 'Island', 3)
        sauron = perm(g, r, 'Sauron, the Dark Lord'); E.amass(g, r, 3); perm(g, r, "Champion's Helm")
        opts = {l: f for u, l, f in brain.main_options(g, r, False)}
        self.assertIn("equip Champion's Helm to Sauron, the Dark Lord", opts)       # no Sword needed
        opts["equip Champion's Helm to Sauron, the Dark Lord"]()
        self.assertEqual(E.epow(g, sauron), 9)
        self.assertTrue(E.untargetable(g, sauron))
        self.assertEqual(sum(L.tapped for L in r.lands), 1)

    def test_sauron_casts_sheoldred_early(self):
        from commander_sim.ai import brain
        g = table('sauron', 'veyran'); r = g.players[0]
        lands(r, 'Swamp', 4); hand(r, 'Sheoldred, the Apocalypse', "Night's Whisper")
        u = {l: u for u, l, f in brain.main_options(g, r, False)}
        self.assertGreater(u['Sheoldred, the Apocalypse'], u["Night's Whisper"])

    def test_slaughter_pact_hits_nonblack_only(self):
        g = table('sauron', 'seph'); r, s = g.players
        perm(g, s, 'Blood Artist'); snipe = perm(g, s, 'Birds of Paradise')
        self.assertEqual(E.legal_targets(g, r, 'destroy', 'c', False, C['Slaughter Pact']), [snipe])

    def test_slaughter_pact_is_paid_at_the_next_upkeep(self):
        g = table('sauron', 'veyran'); r = g.players[0]
        lands(r, 'Swamp', 3)
        E.on_cast(g, r, C['Slaughter Pact'])
        ais.upkeep(g, r)
        self.assertTrue(r.alive)
        self.assertEqual(sum(L.tapped for L in r.lands), 3)
        self.assertEqual(r.pact_debts, [])

    def test_slaughter_pact_unpaid_loses_the_game(self):
        g = table('sauron', 'veyran', 'seph'); r = g.players[0]
        lands(r, 'Island', 3)                                  # no black mana
        E.on_cast(g, r, C['Slaughter Pact'])
        ais.upkeep(g, r)
        self.assertFalse(r.alive)

    def test_the_ai_casts_slaughter_pact_only_when_it_can_pay(self):
        from commander_sim.ai import brain
        g = table('sauron', 'veyran'); r, v = g.players
        hand(r, 'Slaughter Pact'); perm(g, v, 'Guttersnipe')
        self.assertEqual(brain.removal_options(g, r, brain.Situation(g, r)), [])
        lands(r, 'Swamp', 3, tapped=True)                      # tapped now, untapped at the upkeep
        self.assertEqual(len(brain.removal_options(g, r, brain.Situation(g, r))), 1)

    def test_urabrask_on_your_upkeep(self):
        # "At the beginning of your upkeep, exile the top card of your library. You may play it this turn."
        g = table('sauron', 'veyran'); r = g.players[0]
        perm(g, r, 'Urabrask, Heretic Praetor')
        top = r.library[-1]
        ais.upkeep(g, r)
        self.assertEqual(r.hand, [top]); self.assertEqual(r.impulse, [top])

    def test_urabrask_replaces_an_opponents_draw(self):
        # "At the beginning of each opponent's upkeep, the next time they would draw a card this turn, instead they
        #  exile the top card of their library. They may play it this turn."  Exiling isn't drawing: Sheoldred
        #  doesn't trigger, and the card is gone at the end of the turn if it isn't played.
        g = table('sauron', 'veyran'); r, v = g.players
        perm(g, r, 'Urabrask, Heretic Praetor'); perm(g, r, 'Sheoldred, the Apocalypse')
        g.active = v
        ais.upkeep(g, v)
        top = v.library[-1]
        E.draw(g, v, 1, step=True)
        self.assertEqual(v.impulse, [top])
        self.assertEqual(v.life, 40)
        E.draw(g, v, 1)                                        # only the first draw is replaced
        self.assertEqual(v.life, 38)
        ais.end_step(g, v)
        self.assertNotIn(top, v.hand); self.assertIn(top, v.exile)

    def test_kefka_enters(self):
        # "each player discards a card. Then you draw a card for each card type among cards discarded this way"
        g = table('sauron', 'veyran', 'seph'); r, v, s = g.players
        hand(r, 'Sol Ring'); hand(v, 'Island'); hand(s, 'Grave Titan')
        kefka = perm(g, r, 'Kefka, Court Mage // Kefka, Ruler of Ruin')
        self.assertFalse(kefka.fly)                                     # the front face doesn't fly
        self.assertEqual((len(v.hand), len(s.hand)), (0, 0))
        self.assertEqual(len(r.hand), 3)                                # artifact, land, creature

    def test_kefka_transforms(self):
        from commander_sim.ai import brain
        g = table('sauron', 'veyran'); r, v = g.players
        kefka = perm(g, r, 'Kefka, Court Mage // Kefka, Ruler of Ruin')
        lands(r, 'Island', 8); lands(v, 'Island', 5); token(g, v, 1)
        opts = {l: f for u, l, f in brain.main_options(g, r, False)}
        opts['Kefka: {8}, transform']()
        self.assertEqual(v.perms, [])                                   # the opponent sacrificed the token
        self.assertEqual((E.epow(g, kefka), E.etgh(g, kefka), kefka.fly), (5, 7, True))
        n = len(r.hand)
        E.lose_life(g, v, 3, r)                                         # on your turn: draw that many
        self.assertEqual(len(r.hand), n + 3)
        g.active = v
        E.lose_life(g, v, 2, r)
        self.assertEqual(len(r.hand), n + 3)

    def test_brush_off_is_cheaper_against_instants_and_sorceries(self):
        g = table('sauron', 'veyran'); r = g.players[0]
        brush = hand(r, 'Brush Off'); lands(r, 'Island', 2)
        self.assertIs(E.pick_counter(g, r, C['Lightning Bolt']), brush)  # {1}{U}
        self.assertIsNone(E.pick_counter(g, r, C['Grave Titan']))        # {2}{U}{U}
        self.assertTrue(E.cast_counter(g, r, brush, C['Lightning Bolt']))
        self.assertEqual(sum(L.tapped for L in r.lands), 2)

    def test_sauron_values_consecrated_sphinx(self):
        # its priority follows what it will draw: two per opponent draw, so three opponents make it the better card,
        # and it's worth less with one opponent left
        from commander_sim.ai import brain
        g = table('sauron', 'veyran', 'veyran', 'veyran'); r = g.players[0]
        lands(r, 'Island', 5); lands(r, 'Swamp', 2); hand(r, 'Consecrated Sphinx', "Night's Whisper")
        u = {l: u for u, l, f in brain.main_options(g, r, False)}
        self.assertGreater(u['Consecrated Sphinx'], u["Night's Whisper"])
        g1 = table('sauron', 'veyran'); r1 = g1.players[0]
        lands(r1, 'Island', 5); lands(r1, 'Swamp', 2); hand(r1, 'Consecrated Sphinx', "Night's Whisper")
        u1 = {l: u for u, l, f in brain.main_options(g1, r1, False)}
        self.assertLess(u1['Consecrated Sphinx'], u['Consecrated Sphinx'])

    def test_diabolic_intent_needs_a_creature_to_sacrifice(self):
        # "As an additional cost to cast this spell, sacrifice a creature." It used to be free outside Sephiroth.
        from commander_sim.ai import brain
        g = table('sauron', 'veyran'); r = g.players[0]
        lands(r, 'Swamp', 2); c = hand(r, 'Diabolic Intent')
        self.assertFalse(brain.do_cast(g, r, c))
        perm(g, r, 'Orcish Bowmasters')                       # its Army comes too
        army = E.army_of(r); army.plus = 5
        self.assertTrue(brain.do_cast(g, r, c))
        self.assertEqual([m.name for m in r.perms], ['Orc Army'])   # the cheapest creature went, not the Army
        self.assertEqual(len(r.hand), 1)                            # the tutored card

    def test_diabolic_intent_never_sacrifices_the_army(self):
        # the AI used to sacrifice the Army (often with ten or more counters) when it was the only creature,
        # frequently to fetch the Sword that goes on it
        from commander_sim.ai import brain
        g = table('sauron', 'veyran'); r = g.players[0]
        lands(r, 'Swamp', 2); c = hand(r, 'Diabolic Intent')
        E.amass(g, r, 6)
        self.assertFalse(brain.do_cast(g, r, c))
        self.assertIsNotNone(E.army_of(r))

    def test_phyrexian_arena(self):
        g = table('sauron', 'veyran'); r = g.players[0]
        perm(g, r, 'Phyrexian Arena')
        ais.upkeep(g, r)
        self.assertEqual((len(r.hand), r.life), (1, 39))


class Marchesa(unittest.TestCase):
    """Marchesa, the Black Rose: "Dethrone. Other creatures you control have dethrone. Whenever a creature you control
    with a +1/+1 counter on it dies, return that card to the battlefield under your control at the beginning of the
    next end step." """

    def test_dethrone_on_the_life_leader(self):
        g = table('marchesa', 'veyran', 'sauron'); m, v, r = g.players
        a = perm(g, m, 'Marchesa, the Black Rose'); b = perm(g, m, 'Phyrexian Rager')
        v.life = 41
        ais.attack_triggers(g, m, [a, b], r)                  # not the most life: nothing
        self.assertEqual((a.plus, b.plus), (0, 0))
        ais.attack_triggers(g, m, [a, b], v)                  # Rager has dethrone from Marchesa
        self.assertEqual((a.plus, b.plus), (1, 1))

    def test_a_countered_creature_returns_at_the_next_end_step(self):
        g = table('marchesa', 'veyran'); m, v = g.players
        perm(g, m, 'Marchesa, the Black Rose'); rager = perm(g, m, 'Phyrexian Rager')     # enters: draw, lose 1
        rager.plus = 1
        E.die(g, rager, 'destroy')
        self.assertIn(C['Phyrexian Rager'], m.gy)
        ais.end_step(g, v)                                     # anyone's end step
        self.assertNotIn(C['Phyrexian Rager'], m.gy)
        self.assertTrue(any(x.cd.name == 'Phyrexian Rager' and x.plus == 0 for x in m.perms))   # back, no counters
        self.assertEqual((len(m.hand), m.life), (2, 38))        # its enter trigger twice

    def test_no_counter_no_return(self):
        g = table('marchesa', 'veyran'); m, v = g.players
        perm(g, m, 'Marchesa, the Black Rose'); rat = perm(g, m, 'Burglar Rat')
        E.die(g, rat, 'destroy'); ais.end_step(g, m)
        self.assertIn(C['Burglar Rat'], m.gy)

    def test_exiled_creatures_do_not_return(self):
        g = table('marchesa', 'veyran'); m, v = g.players
        perm(g, m, 'Marchesa, the Black Rose'); rat = perm(g, m, 'Burglar Rat'); rat.plus = 2
        E.exile_perm(g, rat); ais.end_step(g, m)
        self.assertIn(C['Burglar Rat'], m.exile)
        self.assertFalse(any(x.cd.name == 'Burglar Rat' for x in m.perms))

    def test_a_wipe_returns_everything_with_counters_marchesa_included(self):
        # the creatures die at the same time as Marchesa, so her ability sees them all
        g = table('marchesa', 'veyran'); m, v = g.players
        a = perm(g, m, 'Marchesa, the Black Rose'); a.is_cmd = True
        rat = perm(g, m, 'Burglar Rat'); rager = perm(g, m, 'Phyrexian Rager'); feeder = perm(g, m, 'Carrion Feeder')
        m.perms.remove(a); m.perms.insert(0, a)                 # Marchesa dies first
        a.plus = rat.plus = rager.plus = 1
        E.apply_wipe(g, v, 'destroy', {})
        self.assertFalse(any(x.creature for x in m.perms))
        ais.end_step(g, v)
        self.assertEqual(sorted(x.cd.name for x in m.perms if x.creature),
                         ['Burglar Rat', 'Marchesa, the Black Rose', 'Phyrexian Rager'])     # not Carrion Feeder
        self.assertTrue(next(x for x in m.perms if x.cd.name.startswith('Marchesa')).is_cmd)
        self.assertEqual(m.tax, 0)

    def test_a_stolen_creature_returns_under_your_control(self):
        from commander_sim.cards.impl import marchesa as IMa
        g = table('marchesa', 'veyran'); m, v = g.players
        perm(g, m, 'Marchesa, the Black Rose'); snipe = perm(g, v, 'Guttersnipe')
        IMa.steal(g, m, snipe, until_eot=True)
        ais.attack_triggers(g, m, [snipe], v)                  # dethrone: a +1/+1 counter
        E.die(g, snipe, 'sac')
        self.assertIn(C['Guttersnipe'], v.gy)                   # its owner's graveyard
        ais.end_step(g, m)
        back = next(x for x in m.perms if x.cd.name == 'Guttersnipe')
        self.assertIs(back.orig, v)
        self.assertNotIn(back, v.perms)

    def test_act_of_treason_then_sacrifice(self):
        from commander_sim.ai import brain
        g = table('marchesa', 'veyran'); m, v = g.players
        lands(m, 'Mountain', 3); perm(g, m, 'Carrion Feeder'); snipe = perm(g, v, 'Guttersnipe')
        c = hand(m, 'Act of Treason')
        E.pay(g, m, 2, 'R'); E.cast_card(g, m, c, 'hand', {})
        self.assertIn(snipe, m.perms)
        self.assertFalse(snipe.sick or snipe.tapped)             # untapped, with haste
        opts = {l: f for u, l, f in brain.main_options(g, m, True)}
        opts['sacrifice stolen Guttersnipe (Carrion Feeder)']()
        self.assertIn(C['Guttersnipe'], v.gy)

    def test_triskelion_loop(self):
        from commander_sim.ai import brain
        g = table('marchesa', 'veyran'); m, v = g.players
        perm(g, m, 'Marchesa, the Black Rose'); perm(g, m, 'Carrion Feeder'); t = perm(g, m, 'Triskelion')
        self.assertEqual(t.plus, 3)
        opts = {l: f for u, l, f in brain.main_options(g, m, True)}
        opts['Triskelion loop (Carrion Feeder)']()
        self.assertEqual(v.life, 38)                            # two shots; sacrificed with one counter left
        ais.end_step(g, m)
        t2 = next(x for x in m.perms if x.cd.name == 'Triskelion')
        self.assertEqual(t2.plus, 3)

    def test_accursed_marauder(self):
        # 2/1: "each player sacrifices a nontoken creature of their choice" - Marchesa's player gives up one she returns
        g = table('marchesa', 'veyran'); m, v = g.players
        perm(g, m, 'Marchesa, the Black Rose'); rager = perm(g, m, 'Phyrexian Rager'); rager.plus = 1
        perm(g, v, 'Guttersnipe'); token(g, v, 1)
        mar = perm(g, m, 'Accursed Marauder')
        self.assertEqual((mar.pow, mar.tgh), (2, 1))
        self.assertIn(C['Phyrexian Rager'], m.gy)
        self.assertIn(mar, m.perms)
        self.assertIn(C['Guttersnipe'], v.gy)                   # the token doesn't count
        self.assertEqual(sum(1 for x in v.perms if x.token), 1)

    def test_tezzerets_gambit_proliferates_every_counter(self):
        g = table('marchesa', 'veyran'); m, v = g.players
        a = perm(g, m, 'Phyrexian Rager'); b = perm(g, m, 'Burglar Rat'); a.plus = 1; b.plus = 2
        lands(m, 'Island', 4)
        c = hand(m, "Tezzeret's Gambit"); E.pay(g, m, 3, 'U'); E.cast_card(g, m, c, 'hand', {})
        self.assertEqual((a.plus, b.plus), (2, 3))

    def test_tezzerets_gambit_pays_life_for_blue(self):
        from commander_sim.ai import brain
        g = table('marchesa', 'veyran'); m = g.players[0]
        lands(m, 'Swamp', 4); c = hand(m, "Tezzeret's Gambit")
        self.assertTrue(brain.do_cast(g, m, c))
        self.assertEqual(m.life, 38)

    def test_removal_restrictions(self):
        g = table('marchesa', 'sauron'); m, r = g.players
        bow = perm(g, r, 'Orcish Bowmasters')                  # black
        mimic = perm(g, r, 'Metallic Mimic')                   # artifact
        self.assertEqual(E.legal_targets(g, m, 'destroy', 'cna', spell=C['Terror']), [])
        self.assertEqual([x for x in E.legal_targets(g, m, 'shrink3', 'c', spell=C['Last Gasp']) if not x.token],
                         [bow, mimic])

    def test_premature_burial_only_hits_what_entered_since_your_last_turn(self):
        g = table('marchesa', 'veyran'); m, v = g.players
        old = perm(g, v, 'Guttersnipe')
        ais.end_step(g, m)                                     # your turn ends
        new = perm(g, v, 'Kessig Flamebreather')
        self.assertEqual(E.legal_targets(g, m, 'destroy', 'c', spell=C['Premature Burial']), [new])

    def test_scorching_dragonfire_exiles(self):
        g = table('marchesa', 'veyran'); m, v = g.players
        snipe = perm(g, v, 'Guttersnipe')
        E.apply_removal(g, m, snipe, 'dmg3', C['Scorching Dragonfire'])
        self.assertIn(C['Guttersnipe'], v.exile)

    def test_last_gasp_ignores_indestructible(self):
        g = table('marchesa', 'veyran'); m, v = g.players
        snipe = perm(g, v, 'Guttersnipe'); v.regen_turn = E.turn_stamp(g)
        E.apply_removal(g, m, snipe, 'shrink3', C['Last Gasp'])
        self.assertIn(C['Guttersnipe'], v.gy)

    def test_gemstone_mine_runs_out(self):
        g = table('marchesa', 'veyran'); m = g.players[0]
        lands(m, 'Gemstone Mine')
        for _ in range(3):
            m.lands[0].tapped = False
            self.assertTrue(E.pay(g, m, 0, 'R'))
        self.assertEqual(m.lands, [])
        self.assertIn(C['Gemstone Mine'], m.gy)

    def test_vivid_land_counters(self):
        g = table('marchesa', 'veyran'); m = g.players[0]
        lands(m, 'Vivid Marsh'); L = m.lands[0]
        for _ in range(3):
            L.tapped = False; E.pay(g, m, 0, 'B')              # its own colour: free
        for _ in range(2):
            L.tapped = False; self.assertTrue(E.pay(g, m, 0, 'R'))
        L.tapped = False
        self.assertFalse(E.can_pay(g, m, 0, 'R'))              # counters gone: black only
        self.assertTrue(E.can_pay(g, m, 0, 'B'))

    def test_soul_enervation(self):
        g = table('marchesa', 'veyran'); m, v = g.players
        mystic = perm(g, v, 'Murmuring Mystic')                # 1/5 survives -4/-4? no: toughness 5
        snipe = perm(g, v, 'Guttersnipe')
        perm(g, m, 'Soul Enervation')
        self.assertIn(C['Guttersnipe'], v.gy)                   # the best target it kills
        m.gy.append(C['Phyrexian Rager']); E.check_state(g)
        m.gy.remove(C['Phyrexian Rager']); E.check_state(g)    # a creature card left your graveyard
        self.assertEqual((m.life, v.life), (41, 39))

    def test_al_bhed_salvagers(self):
        g = table('marchesa', 'veyran'); m, v = g.players
        perm(g, m, 'Al Bhed Salvagers'); rat = perm(g, m, 'Burglar Rat')
        E.die(g, rat, 'sac')
        E.add_treasure(g, m, 1); lands(m, 'Swamp', 0); E.pay(g, m, 0, 'B')   # a Treasure is an artifact dying
        self.assertEqual((m.life, v.life), (42, 38))

    def test_crackling_drake_power(self):
        g = table('marchesa', 'veyran'); m = g.players[0]
        m.gy += [C['Lightning Bolt'], C["Night's Whisper"], C['Burglar Rat']]; m.exile.append(C['Terror'])
        d = perm(g, m, 'Crackling Drake')
        self.assertEqual((E.epow(g, d), E.etgh(g, d), len(m.hand)), (3, 4, 1))

    def test_enslave(self):
        g = table('marchesa', 'veyran'); m, v = g.players
        snipe = perm(g, v, 'Guttersnipe')
        ens = perm(g, m, 'Enslave')
        self.assertIn(snipe, m.perms)
        E.CI.fire(g, 'upkeep', m)
        self.assertEqual(v.life, 39)                            # 1 damage to its owner
        E.die(g, ens, 'destroy')
        self.assertIn(snipe, v.perms)                           # control returns

    def test_lethal_throwdown_draws_with_a_modified_creature(self):
        from commander_sim.ai import brain
        g = table('marchesa', 'veyran'); m, v = g.players
        lands(m, 'Swamp', 1); perm(g, m, 'Marchesa, the Black Rose'); rat = perm(g, m, 'Burglar Rat'); rat.plus = 1
        perm(g, v, 'Guttersnipe'); hand(m, 'Lethal Throwdown')
        opts = {l: f for u, l, f in brain.main_options(g, m, False)}
        opts['Lethal Throwdown (sacrifice Burglar Rat) -> Guttersnipe']()     # the one Marchesa returns
        self.assertIn(C['Guttersnipe'], v.gy)
        self.assertEqual(len(m.hand), 1)

    def test_disembowel_costs_the_targets_mana_value(self):
        from commander_sim.ai import brain
        g = table('marchesa', 'veyran'); m, v = g.players
        lands(m, 'Swamp', 4); perm(g, v, 'Murmuring Mystic')   # X=5 plus {B}: not affordable
        hand(m, 'Disembowel')
        self.assertFalse(any(l.startswith('Disembowel') for u, l, f in brain.main_options(g, m, False)))
        perm(g, v, 'Guttersnipe')                               # mana value 3
        opts = {l: f for u, l, f in brain.main_options(g, m, False)}
        opts['Disembowel X=3 -> Guttersnipe']()
        self.assertEqual(sum(L.tapped for L in m.lands), 4)

    def test_two_notion_thieves_do_not_loop(self):
        # 614.5: each replacement applies once; the draw goes to the other Thief's player and back
        g = table('marchesa', 'sauron'); m, r = g.players
        perm(g, m, 'Notion Thief'); perm(g, r, 'Notion Thief')
        E.draw(g, r, 1)
        self.assertEqual((len(m.hand), len(r.hand)), (0, 1))

    def test_coalition_relic_charge(self):
        g = table('marchesa', 'veyran'); m = g.players[0]
        relic = perm(g, m, 'Coalition Relic')
        E.CI.fire(g, 'end_step', m)
        self.assertTrue(relic.tapped)
        relic.tapped = False; m.floatA = 0
        E.CI.fire(g, 'upkeep', m)
        self.assertEqual(E.total_mana(g, m), 2)                 # the charge counter plus its tap

    def test_nim_deathmantle_leaves_marchesa_returns_alone(self):
        # Nim Deathmantle doesn't pay {4} for a creature Marchesa returns for free
        g = table('marchesa', 'veyran'); m = g.players[0]
        lands(m, 'Swamp', 4); perm(g, m, 'Marchesa, the Black Rose'); perm(g, m, 'Nim Deathmantle')
        rat = perm(g, m, 'Burglar Rat'); rat.plus = 1
        E.die(g, rat, 'destroy')
        self.assertFalse(any(L.tapped for L in m.lands))

    def test_nim_deathmantle_card_exiled_while_paying(self):
        # paying {4} with Treasures sets off Mayhem Devil; its ping kills Dark Confidant, and the state-based check that
        # follows runs Dauthi Voidwalker's sweep: the card has left the graveyard, so nothing returns (the mana is spent)
        g = table('seph', 'veyran', 'veyran'); s, a, b = g.players
        s.treasures = 4; perm(g, s, 'Nim Deathmantle')
        perm(g, a, 'Mayhem Devil'); perm(g, b, 'Dauthi Voidwalker'); perm(g, b, 'Dark Confidant')
        titan = perm(g, s, 'Grave Titan')
        E.die(g, titan, 'destroy')
        self.assertEqual(s.treasures, 0)
        self.assertIn(C['Grave Titan'], s.exile)
        self.assertFalse(any(x.cd is C['Grave Titan'] for x in s.perms))


class NewCards(unittest.TestCase):
    """Ephemerate (Sephiroth) and Sword of Fire and Ice (Sauron)"""

    def test_ephemerate_fizzles_targeted_removal(self):
        # "Exile target creature you control, then return it to the battlefield under its owner's control. Rebound"
        g = table('seph', 'veyran'); s, v = g.players
        lands(s, 'Plains', 1); hand(s, 'Ephemerate')
        grave = perm(g, s, 'Grave Titan')                       # enters: two Zombies
        E.apply_removal(g, v, grave, 'exile', C['Swords to Plowshares'])
        self.assertTrue(any(x.cd is not None and x.cd.name == 'Grave Titan' for x in s.perms))
        self.assertEqual(sum(1 for x in s.perms if x.token), 4)      # entered again: two more Zombies
        self.assertIn(C['Ephemerate'], s.exile)
        self.assertEqual(s.rebound, [C['Ephemerate']])

    def test_ephemerate_answers_theft(self):
        g = table('seph', 'veyran'); s, v = g.players
        lands(s, 'Plains', 1); hand(s, 'Ephemerate')
        grave = perm(g, s, 'Grave Titan')
        self.assertTrue(ais.protect_response(g, s, grave, 'steal', v, C['Zealous Conscripts']))
        self.assertTrue(any(x.cd is not None and x.cd.name == 'Grave Titan' for x in s.perms))

    def test_ephemerate_rebound_blinks_again(self):
        from commander_sim.cards import cardimpl
        g = table('seph', 'veyran'); s, v = g.players
        lands(s, 'Plains', 1); hand(s, 'Ephemerate')
        grave = perm(g, s, 'Grave Titan')
        from commander_sim.cards.impl import mine
        self.assertTrue(mine.ephemerate_cast(g, s, grave, 'value'))
        cardimpl.HOOKS['Ephemerate']['rebound'](g, s, C['Ephemerate'])
        self.assertEqual(sum(1 for x in s.perms if x.token), 6)      # three Grave Titan entries
        self.assertIn(C['Ephemerate'], s.gy)

    def test_ephemerate_value_blink_offered(self):
        from commander_sim.ai import brain
        g = table('seph', 'veyran'); s = g.players[0]
        lands(s, 'Plains', 1); hand(s, 'Ephemerate'); perm(g, s, 'Grave Titan')
        self.assertIn('Ephemerate (blink Grave Titan)', [l for u, l, f in brain.main_options(g, s, True)])

    def test_sword_of_fire_and_ice(self):
        # "+2/+2 and protection from red and from blue. Whenever equipped creature deals combat damage to a player,
        #  Sword of Fire and Ice deals 2 damage to any target and you draw a card. Equip {2}"
        from commander_sim.ai import brain
        g = table('sauron', 'veyran'); r, v = g.players
        lands(r, 'Island', 2); E.amass(g, r, 3); a = E.army_of(r); a.sick = False
        perm(g, r, 'Sword of Fire and Ice')
        opts = {l: f for u, l, f in brain.main_options(g, r, False)}
        opts['equip Sword of Fire and Ice']()
        self.assertEqual((E.epow(g, a), E.etgh(g, a)), (5, 5))
        self.assertTrue(E.protected_from(g, a, 'R'))
        self.assertEqual(E.legal_targets(g, v, 'dmg3', 'c', spell=C['Lightning Bolt']), [])
        ais.resolve_combat(g, r, [a], v, set())
        self.assertEqual(v.life, 40 - 5 - 2)                    # the Army, then the Sword's 2 (no creature to hit)
        self.assertEqual(len(r.hand), 1)

    def test_sword_of_truth_and_justice(self):
        # "Equipped creature gets +2/+2 and has protection from white and from blue. Whenever equipped creature deals
        #  combat damage to a player, put a +1/+1 counter on a creature you control, then proliferate. Equip {2}"
        from commander_sim.ai import brain
        g = table('sauron', 'veyran'); r, v = g.players
        lands(r, 'Island', 2); E.amass(g, r, 3); a = E.army_of(r); a.sick = False
        flux = perm(g, r, 'Flux Channeler'); flux.plus = 1
        perm(g, r, 'Sword of Truth and Justice')
        opts = {l: f for u, l, f in brain.main_options(g, r, False)}
        opts['equip Sword of Truth and Justice']()
        self.assertTrue(E.protected_from(g, a, 'U'))
        ais.resolve_combat(g, r, [a], v, set())
        self.assertEqual(a.plus, 3 + 1 + 1)                     # the counter goes on the Army, then proliferate
        self.assertEqual(flux.plus, 2)



class SauronBreach(unittest.TestCase):
    """Underworld Breach + Brain Freeze / Grapeshot (storm: a copy per spell cast before it this turn)"""

    def board(self, islands, mountains, opp_library=20):
        g = table('sauron', 'veyran', 'veyran', 'veyran'); s = g.players[0]
        for q in g.players[1:]:
            q.hand = []; q.gy += q.library[opp_library:]; del q.library[opp_library:]
        s.hand = []; s.gy = [C['Island']] * 6                          # fuel for escape
        lands(s, 'Island', islands); lands(s, 'Mountain', mountains)
        return g, s

    def test_brain_freeze_copies(self):
        g, s = self.board(2, 0); v = g.players[1]
        E.cast_card(g, s, hand(s, 'Brain Freeze'), 'hand', {'storm': 2, 'mill': [v, v, v]})
        self.assertEqual(len(v.library), 20 - 9)

    def test_grapeshot_kills_the_lowest_life_first(self):
        g, s = self.board(0, 2); a, b = g.players[1], g.players[2]
        a.life = 3
        E.cast_card(g, s, hand(s, 'Grapeshot'), 'hand', {'storm': 4})
        self.assertFalse(a.alive)
        self.assertEqual(sum(40 - q.life for q in g.players[2:]), 2)

    def test_the_line_waits_for_enough_mana(self):
        from commander_sim.cards.impl import mine
        g, s = self.board(6, 6); hand(s, 'Underworld Breach', 'Brain Freeze')
        self.assertFalse(mine.breach_line(g, s))                       # 12 mana: two opponents milled, not three
        g, s = self.board(8, 8); hand(s, 'Underworld Breach', 'Brain Freeze')
        self.assertTrue(mine.breach_line(g, s))
        g, s = self.board(4, 12); hand(s, 'Underworld Breach', 'Brain Freeze')
        self.assertFalse(mine.breach_line(g, s))                       # 16 mana, but only four blue for Brain Freeze

    def test_the_line_mills_the_table(self):
        from commander_sim.cards.impl import mine
        g, s = self.board(8, 8); hand(s, 'Underworld Breach', 'Brain Freeze')
        self.assertTrue(mine.breach_line(g, s, execute=True))
        self.assertEqual([len(q.library) for q in g.players[1:]], [0, 0, 0])
        self.assertGreaterEqual(s.stats['breach_escapes'], 5)

    def test_lotus_petal_makes_it_infinite(self):
        # each Petal escape ({0} plus three graveyard cards) makes one mana of any colour: the {U} for the next Brain
        # Freeze, whose copies mill you for more fuel; from two blue sources it mills out the table
        from commander_sim.cards.impl import mine
        g, s = self.board(2, 1, opp_library=60); lands(s, 'Swamp', 1)
        hand(s, 'Underworld Breach', 'Brain Freeze', 'Lotus Petal')
        self.assertTrue(mine.breach_line(g, s))
        mine.breach_line(g, s, execute=True)
        self.assertEqual([len(q.library) for q in g.players[1:]], [0, 0, 0])

    def test_rituals_alone_run_out_of_blue(self):
        # Dark Ritual and Cabal Ritual make only black mana: storm grows, but each Brain Freeze still needs a real {U}
        from commander_sim.cards.impl import mine
        g, s = self.board(3, 1, opp_library=60); lands(s, 'Swamp', 1)
        hand(s, 'Underworld Breach', 'Brain Freeze', 'Dark Ritual', 'Cabal Ritual')
        self.assertFalse(mine.breach_line(g, s))

    def test_lions_eye_diamond_makes_it_infinite(self):
        # each Diamond escape ({0} plus three graveyard cards) discards the hand and makes three blue: a whole Brain
        # Freeze; from one Island and two Mountains it mills out the table, which the same lands alone can't
        from commander_sim.cards.impl import mine
        g, s = self.board(1, 2, opp_library=60)
        hand(s, 'Underworld Breach', 'Brain Freeze')
        self.assertFalse(mine.breach_line(g, s))
        g, s = self.board(1, 2, opp_library=60)
        hand(s, 'Underworld Breach', 'Brain Freeze', "Lion's Eye Diamond")
        self.assertTrue(mine.breach_line(g, s))
        mine.breach_line(g, s, execute=True)
        self.assertEqual([len(q.library) for q in g.players[1:]], [0, 0, 0])
        self.assertGreaterEqual(s.stats['led_used'], 2)

    def test_laboratory_maniac_mills_yourself_out(self):
        # with Laboratory Maniac out, every Brain Freeze copy mills you; Lotus Petal escapes pay each next Brain
        # Freeze (the copies refill the graveyard), and an escaped Night's Whisper draws from the empty library to win.
        # Milling three 60-card libraries from the same mana fails
        from commander_sim.cards.impl import mine
        g, s = self.board(2, 2, opp_library=60); lands(s, 'Swamp', 2)
        del s.library[30:]
        hand(s, 'Underworld Breach', 'Brain Freeze', 'Lotus Petal', "Night's Whisper")
        self.assertFalse(mine.breach_line(g, s))
        g, s = self.board(2, 2, opp_library=60); lands(s, 'Swamp', 2)
        del s.library[30:]
        hand(s, 'Underworld Breach', 'Brain Freeze', 'Lotus Petal', "Night's Whisper")
        perm(g, s, 'Laboratory Maniac')
        self.assertTrue(mine.breach_line(g, s))
        mine.breach_line(g, s, execute=True)
        self.assertIs(g.winner, s)

    def test_the_line_casts_laboratory_maniac_from_hand(self):
        from commander_sim.cards.impl import mine
        g, s = self.board(4, 2, opp_library=60); lands(s, 'Swamp', 2)
        del s.library[30:]
        hand(s, 'Laboratory Maniac', 'Underworld Breach', 'Brain Freeze', 'Lotus Petal', "Night's Whisper")
        self.assertTrue(mine.breach_line(g, s))
        mine.breach_line(g, s, execute=True)
        self.assertIs(g.winner, s)

    def test_birgi_pays_for_each_brain_freeze(self):
        # Birgi adds {R} for each spell cast: with her out, a Petal escape makes the {U} and Birgi the {1}. Holding her
        # in hand doesn't stop a line that works without casting her first (the dry run tries both)
        from commander_sim.cards.impl import mine
        B = 'Birgi, God of Storytelling // Harnfel, Horn of Bounty'
        g, s = self.board(1, 2, opp_library=60); perm(g, s, B)
        hand(s, 'Underworld Breach', 'Brain Freeze', 'Lotus Petal')
        mine.breach_line(g, s, execute=True)
        self.assertEqual([len(q.library) for q in g.players[1:]], [0, 0, 0])
        self.assertGreaterEqual(s.floatR, 5)                              # Birgi's spare red
        g, s = self.board(1, 2, opp_library=60)
        hand(s, B, 'Underworld Breach', 'Brain Freeze', 'Lotus Petal')
        self.assertTrue(mine.breach_line(g, s))

    def test_jeskas_will_never_spends_the_blue_brain_freeze_needs(self):
        # Jeska's Will makes red for each card in an opponent's hand, but paying for it can eat the Petal's {U}: the
        # dry run falls back to the plain line, so holding it never stops a line that works without it
        from commander_sim.cards.impl import mine
        g, s = self.board(0, 4, opp_library=60); s.gy = []
        for q in g.players[1:]: q.hand = [card('Island')] * 7
        hand(s, "Jeska's Will", 'Underworld Breach', 'Brain Freeze', 'Lotus Petal')
        self.assertTrue(mine.breach_line(g, s))
        mine.breach_line(g, s, execute=True)
        self.assertEqual([len(q.library) for q in g.players[1:]], [0, 0, 0])

    def test_laboratory_maniac_turns_an_empty_draw_into_a_win(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        s.library = []
        E.draw(g, s, 1)
        self.assertTrue(s.decked)
        g = table('sauron', 'veyran'); s = g.players[0]
        s.library = []; perm(g, s, 'Laboratory Maniac')
        E.draw(g, s, 1)
        self.assertIs(g.winner, s)

    def test_the_pieces_are_held_for_the_line(self):
        g, s = self.board(8, 8)
        bf, br, led = hand(s, 'Brain Freeze', 'Underworld Breach', "Lion's Eye Diamond")
        self.assertEqual(ais.sauron_prio(g, s, bf), 0)
        self.assertEqual(ais.gc_prio_sauron(g, s, br), 0)
        self.assertEqual(ais.sauron_prio(g, s, led), 0)

    def test_tutors_find_the_missing_piece(self):
        g, s = self.board(4, 4); hand(s, 'Underworld Breach')
        s.library.append(card('Brain Freeze'))
        self.assertEqual(ais.tutor_pick(g, s, 'any'), 'Brain Freeze')



class Phyrexian(unittest.TestCase):
    """Phyrexian mana ({B/P}): the AI pays 2 life instead of the colour when the mana isn't there and it has more
    than 10 life (Vraska, Betrayal's Sting: {4}{B}{B/P})"""
    def setUp(self):
        self.g = table('sauron', 'veyran'); self.s = self.g.players[0]
        lands(self.s, 'Swamp', 1); lands(self.s, 'Island', 4)
        self.c = hand(self.s, "Vraska, Betrayal's Sting")

    def test_pays_life_for_the_missing_black(self):
        from commander_sim.ai import brain
        self.assertEqual(E.phyrexian(self.c), 'B')
        self.assertEqual(E.cost_of(self.s, self.c), (4, 'B'))
        brain.do_cast(self.g, self.s, self.c)
        self.assertTrue(any(m.name == "Vraska, Betrayal's Sting" for m in self.s.perms))
        self.assertEqual(self.s.life, 38)

    def test_not_at_low_life(self):
        self.s.life = 10
        self.assertEqual(E.cost_of(self.s, self.c), (4, 'BB'))

    def test_full_mana_when_it_has_it(self):
        lands(self.s, 'Swamp', 1)
        self.assertEqual(E.cost_of(self.s, self.c), (4, 'BB'))


class SauronMarchesa(unittest.TestCase):
    """Marchesa, the Black Rose in Sauron's deck (10-03): dethrone for the Army, with Mauhúr's extra counter, and the
    return of a creature with a counter"""
    def test_the_army_gets_dethrone_and_mauhurs_extra_counter(self):
        g = table('sauron', 'veyran', 'seph'); s, v, ph = g.players
        perm(g, s, 'Marchesa, the Black Rose'); perm(g, s, 'Mauhúr, Uruk-hai Captain')
        E.amass(g, s, 2)
        army = ais.army_of(s); army.sick = False
        before = army.plus
        v.life = 45                                                   # the life leader
        ais.attack_triggers(g, s, [army], v)
        self.assertEqual(army.plus, before + 2)                       # dethrone 1, Mauhúr 1

    def test_a_creature_with_a_counter_returns(self):
        g = table('sauron', 'veyran'); s, v = g.players
        perm(g, s, 'Marchesa, the Black Rose')
        b = perm(g, s, 'Orcish Bowmasters'); b.plus = 1
        E.die(g, b, 'destroy')
        ais.end_step(g, s)
        self.assertTrue(any(m.cd is b.cd for m in s.perms))


class Anduril(unittest.TestCase):
    """Andúril, Flame of the West: +3/+1; two tapped 1/1 flying Spirits when the equipped creature attacks, attacking
    only if that creature is legendary (the Ring makes your Ring-bearer legendary)"""
    def setUp(self):
        self.g = table('sauron', 'veyran'); self.p, self.q = self.g.players
        self.a = perm(self.g, self.p, 'Andúril, Flame of the West')

    def attack(self, m):
        self.a.attached = m
        return E.CI.HOOKS['Andúril, Flame of the West']['attack'](self.g, self.a, self.p, [m], self.q) or []

    def spirits(self):
        return [m for m in self.p.perms if m.token and 'spirit' in m.ttypes]

    def test_nonlegendary_spirits_enter_tapped_not_attacking(self):
        army = E.Perm(self.p, None, pw=0, tg=0, name='Orc Army'); army.army = True; army.plus = 3
        self.p.perms.append(army)
        self.assertEqual(self.attack(army), [])
        self.assertEqual(len(self.spirits()), 2)
        self.assertTrue(all(m.tapped and m.fly for m in self.spirits()))
        self.assertEqual(E.epow(self.g, army), 6)

    def test_legendary_spirits_attack(self):
        s = perm(self.g, self.p, 'Sauron, the Dark Lord')
        self.assertEqual(len(self.attack(s)), 2)
        self.assertEqual(E.epow(self.g, s), 10)


if __name__ == '__main__':
    unittest.main()
