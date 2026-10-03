"""Your Galadriel deck (cards/impl/galadriel.py): Galadriel's Alliance, the Rebel searchers, the creature-type cards,
Panharmonicon, the deck's other modeled cards, and the person's choices and abilities in practice mode."""
import unittest
from tests.table import table, hand, perm, lands, card
from commander_sim import engine as E, ais
from commander_sim.play import human, mana
from commander_sim.play.controller import ScriptController


G = None


def setUpModule():
    """the card module is imported once the card database is set up (see test_zur)"""
    global G
    table('galadriel', 'veyran')
    from commander_sim.cards.impl import galadriel as _g
    G = _g


def galadriel(g, p):
    m = E.enter(g, p, p.cmd); m.is_cmd = True; p.cmd_in_zone = False
    return m


def seat(g, p, answers):
    g.controllers = {p.key: ScriptController(answers)}
    return g.controllers[p.key]


def by_text(word):
    return lambda req: next(i for i, x in enumerate(req.choices) if word in x)


def opts(g, m, p, post=True):
    from commander_sim.ai import brain
    return E.CI.HOOKS[m.cd.name]['options'](g, m, p, brain.Situation(g, p), post)


class Alliance(unittest.TestCase):
    def test_each_mode_once_a_turn(self):
        g = table('galadriel', 'veyran', 'sauron'); p = g.players[0]
        gal = galadriel(g, p)
        for n in ('Llanowar Elves', 'Fyndhorn Elves', 'Elvish Mystic', 'Jhovall Queen'):
            perm(g, p, n)
        self.assertEqual(sorted(G._chosen(g, gal)), ['counters', 'draw', 'mana'])      # the fourth: nothing left
        self.assertEqual(p.milestone.get('alliance'), p.turns)

    def test_you_choose_the_mode(self):
        g = table('galadriel', 'veyran', 'sauron'); p = g.players[0]
        galadriel(g, p)
        seat(g, p, [by_text('add {G}{G}{G}')])
        perm(g, p, 'Llanowar Elves')
        self.assertEqual(mana.pool_of(p).m['G'], 3)

    def test_panharmonicon_doubles_it(self):
        g = table('galadriel', 'veyran', 'sauron'); p = g.players[0]
        gal = galadriel(g, p)
        perm(g, p, 'Panharmonicon')
        perm(g, p, 'Llanowar Elves')
        self.assertEqual(len(G._chosen(g, gal)), 2)

    def test_panharmonicon_doubles_cathars_crusade(self):
        g = table('galadriel', 'veyran', 'sauron'); p = g.players[0]
        perm(g, p, "Cathars' Crusade"); perm(g, p, 'Panharmonicon')
        a = perm(g, p, 'Jhovall Queen')
        perm(g, p, 'Llanowar Elves')
        self.assertEqual(a.plus, 4)                       # two for itself, two for the Elves


class Rebels(unittest.TestCase):
    def test_a_searcher_finds_a_rebel_within_its_cap(self):
        g = table('galadriel', 'veyran', 'sauron'); p = g.players[0]
        lands(p, 'Plains', 3)
        m = perm(g, p, 'Ramosian Sergeant')
        before = set(id(x) for x in p.perms)
        o = opts(g, m, p)
        self.assertTrue(o and o[0][2]())
        new = [x for x in p.perms if id(x) not in before]
        self.assertEqual(len(new), 1)
        self.assertIn('rebel', new[0].cd.subtypes); self.assertLessEqual(new[0].cd.cmc, 2)
        self.assertTrue(m.tapped)

    def test_maskwood_makes_every_creature_a_rebel(self):
        g = table('galadriel', 'veyran', 'sauron'); p = g.players[0]
        elves = card('Llanowar Elves')
        self.assertFalse(G.card_is(p, elves, 'rebel'))
        perm(g, p, 'Maskwood Nexus')
        self.assertTrue(G.card_is(p, elves, 'rebel'))
        self.assertTrue(E.has_type(perm(g, p, 'Fyndhorn Elves'), 'rebel'))

    def test_you_search_with_lin_sivvi(self):
        g = table('galadriel', 'veyran', 'sauron'); p = g.players[0]
        lin = perm(g, p, 'Lin Sivvi, Defiant Hero')
        mana.pool_of(p).add('C', 3)
        seat(g, p, [0, by_text('X = 3'), by_text('Ramosian Captain')])
        self.assertIsNone(human.use(g, p, lin))
        self.assertTrue(any(x.cd is not None and x.cd.name == 'Ramosian Captain' for x in p.perms))
        self.assertTrue(lin.tapped); self.assertEqual(mana.pool_of(p).total(), 0)

    def test_revivalist_returns_a_rebel(self):
        g = table('galadriel', 'veyran', 'sauron'); p = g.players[0]
        lands(p, 'Plains', 6)
        m = perm(g, p, 'Ramosian Revivalist')
        c = hand(p, 'Ramosian Captain'); p.hand.remove(c); p.gy.append(c)
        o = opts(g, m, p)
        self.assertTrue(o and o[0][2]())
        self.assertTrue(any(x.cd is c for x in p.perms))


class CreatureTypes(unittest.TestCase):
    def test_kindred_discovery_names_a_type(self):
        g = table('galadriel', 'veyran', 'sauron'); p = g.players[0]
        kd = perm(g, p, 'Kindred Discovery')
        t = G.ctype(kd)
        self.assertIn(t, ('human', 'rebel'))
        n = len(p.hand)
        perm(g, p, 'Ramosian Sergeant')                    # a Human Rebel
        self.assertEqual(len(p.hand), n + 1)

    def test_sauron_still_names_orc(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        self.assertEqual(G.ctype(perm(g, s, 'Kindred Discovery')), 'orc')

    def test_door_of_destinies_and_the_banner(self):
        g = table('galadriel', 'veyran', 'sauron'); p = g.players[0]
        door = perm(g, p, 'Door of Destinies'); door.data['ctype'] = 'rebel'
        ban = perm(g, p, "Vanquisher's Banner"); ban.data['ctype'] = 'rebel'
        r = perm(g, p, 'Ramosian Sergeant')
        n = len(p.hand)
        E.cast_card(g, p, hand(p, 'Ramosian Lieutenant'), 'hand', {})
        self.assertEqual(door.data.get('charge'), 1)
        self.assertEqual(len(p.hand), n + 1)                # took one into hand to cast it, drew one (the Banner)
        self.assertEqual(E.epow(g, r), 1 + 1 + 1)           # base 1, the Door's counter, the Banner

    def test_secluded_courtyard_colours_only_for_the_type(self):
        g = table('galadriel', 'veyran', 'sauron'); p = g.players[0]
        L = E.Land(card('Secluded Courtyard'), False); L.data = {'ctype': 'elf'}; p.lands.append(L)
        E.PAY_FOR = card('Llanowar Elves')
        try:
            self.assertIn('G', E.land_cols(p, L, False))
            E.PAY_FOR = card('Counterspell')
            self.assertEqual(E.land_cols(p, L, False), '')
        finally:
            E.PAY_FOR = None

    def test_courtyard_mana_in_your_pool(self):
        g = table('galadriel', 'veyran', 'sauron'); p = g.players[0]
        L = E.Land(card('Secluded Courtyard'), False); L.data = {'ctype': 'elf'}; p.lands.append(L)
        src = next(s for s in mana.sources(g, p) if s.get('restricted'))
        self.assertIsNone(mana.tap(g, p, src['id'], 'G'))
        pool = mana.pool_of(p)
        self.assertFalse(pool.can_pay(0, 'G'))              # nothing being paid for: not usable
        mana.SPENDING = card('Llanowar Elves')
        try:
            self.assertTrue(pool.pay(0, 'G'))
        finally:
            mana.SPENDING = None


class OtherCards(unittest.TestCase):
    def test_crackdown_keeps_big_nonwhite_creatures_tapped(self):
        g = table('galadriel', 'sauron'); p, s = g.players
        perm(g, p, 'Crackdown')
        big = perm(g, s, 'Grave Titan'); big.tapped = True
        small = perm(g, s, 'Orcish Bowmasters'); small.tapped = True
        ais._step_start(g, s)
        self.assertTrue(big.tapped); self.assertFalse(small.tapped)

    def test_cho_manno_takes_no_damage(self):
        g = table('galadriel', 'veyran'); p, v = g.players
        cho = perm(g, p, 'Cho-Manno, Revolutionary')
        E.apply_removal(g, v, cho, 'dmg3')
        self.assertIn(cho, p.perms)

    def test_knight_of_the_holy_nimbus_regenerates_unless_paid(self):
        g = table('galadriel', 'veyran'); p, v = g.players
        k = perm(g, p, 'Knight of the Holy Nimbus')
        E.die(g, k, 'destroy')
        self.assertIn(k, p.perms); self.assertTrue(k.tapped)        # the opponent had no mana
        k2 = perm(g, p, 'Knight of the Holy Nimbus')
        lands(v, 'Island', 2)
        E.die(g, k2, 'destroy')
        self.assertNotIn(k2, p.perms)                               # paid {2}

    def test_defiant_vanguard_kills_what_it_blocks(self):
        g = table('sauron', 'galadriel'); s, p = g.players
        t = perm(g, s, 'Grave Titan'); t.sick = False
        vg = perm(g, p, 'Defiant Vanguard')
        ais.resolve_combat(g, s, [t], p, set())
        self.assertNotIn(t, s.perms); self.assertNotIn(vg, p.perms)

    def test_amrou_seekers_blockers(self):
        g = table('galadriel', 'veyran'); p, v = g.players
        a = perm(g, p, 'Amrou Seekers')
        self.assertFalse(ais.can_block(g, perm(g, v, 'Guttersnipe'), a))
        self.assertTrue(ais.can_block(g, perm(g, p, 'Jhovall Queen'), a))

    def test_abduction_steals_and_returns_on_death(self):
        g = table('galadriel', 'sauron'); p, s = g.players
        t = perm(g, s, 'Grave Titan'); t.tapped = True
        perm(g, p, 'Abduction')
        self.assertIn(t, p.perms); self.assertFalse(t.tapped)
        E.die(g, t, 'destroy')
        self.assertTrue(any(x.cd is t.cd for x in s.perms))           # back under its owner's control

    def test_bribery(self):
        g = table('galadriel', 'sauron'); p, s = g.players
        E.cast_card(g, p, hand(p, 'Bribery'), 'hand', {})
        self.assertTrue(any(x.orig is s for x in p.perms if x.creature))

    def test_eerie_interlude_returns_at_the_end_step(self):
        g = table('galadriel', 'veyran'); p = g.players[0]
        m = perm(g, p, 'Recruiter of the Guard')
        E.cast_card(g, p, hand(p, 'Eerie Interlude'), 'hand', {'targets': [m]})
        self.assertNotIn(m, p.perms)
        ais.end_step(g, p)
        self.assertTrue(any(x.cd is m.cd for x in p.perms))

    def test_voice_of_resurgence_token(self):
        g = table('galadriel', 'veyran'); p, v = g.players
        perm(g, p, 'Voice of Resurgence'); perm(g, p, 'Jhovall Queen')
        E.on_cast(g, v, card('Lightning Bolt'))                       # during your turn
        tok = next(x for x in p.perms if x.token)
        self.assertEqual(E.epow(g, tok), 3)

    def test_mangara_draws_on_the_second_spell(self):
        g = table('galadriel', 'veyran'); p, v = g.players
        perm(g, p, 'Mangara, the Diplomat')
        n = len(p.hand)
        v.spells_this_turn = 2
        E.on_cast(g, v, card('Lightning Bolt'))
        self.assertEqual(len(p.hand), n + 1)

    def test_mana_springleaf_drum_and_elvish_archdruid(self):
        g = table('galadriel', 'veyran'); p = g.players[0]
        d = perm(g, p, 'Springleaf Drum')
        self.assertEqual(E.CI.dyn_mana(g, p, d), 0)                  # no creature to tap
        q = perm(g, p, 'Jhovall Queen')
        self.assertEqual(E.CI.dyn_mana(g, p, d), 1)
        self.assertTrue(E.pay(g, p, 1, ''))
        self.assertTrue(q.tapped)
        a = perm(g, p, 'Elvish Archdruid'); perm(g, p, 'Farhaven Elf')
        self.assertEqual(E.CI.dyn_mana(g, p, a), 2)

    def test_return_to_dust_takes_two_in_your_main_phase(self):
        g = table('galadriel', 'veyran'); p, v = g.players
        a, b = perm(g, v, 'Sol Ring'), perm(g, v, 'Mind Stone')
        E.cast_card(g, p, hand(p, 'Return to Dust'), 'hand', {'target': a})
        self.assertNotIn(a, v.perms); self.assertNotIn(b, v.perms)

    def test_unbreakable_formation_addendum(self):
        g = table('galadriel', 'veyran'); p = g.players[0]
        m = perm(g, p, 'Jhovall Queen')
        E.cast_card(g, p, hand(p, 'Unbreakable Formation'), 'hand', {})
        self.assertEqual(m.plus, 1); self.assertTrue(E.indestructible(g, m))
        g.step = 'combat'                                            # outside your main phase: no addendum
        G.formation(g, p, g.step in ('main1', 'main2'))
        self.assertEqual(m.plus, 1)

    def test_elspeth_emblem_flies(self):
        g = table('galadriel', 'veyran'); p, v = g.players
        m = perm(g, p, 'Jhovall Queen')
        E.CI.elspeth_emblem(g, p)
        self.assertTrue(E.DSLMOD.has_kw(g, m, 'flying'))
        self.assertEqual(E.epow(g, m), 4 + 2)


class Practice(unittest.TestCase):
    def test_a_rebel_searcher(self):
        g = table('galadriel', 'veyran', 'sauron'); p = g.players[0]
        m = perm(g, p, 'Ramosian Lieutenant')
        mana.pool_of(p).add('C', 4)
        seat(g, p, [0, by_text('Ramosian Sergeant')])
        self.assertIsNone(human.use(g, p, m))
        self.assertTrue(any(x.cd is not None and x.cd.name == 'Ramosian Sergeant' for x in p.perms))

    def test_whipcorder_morph(self):
        from commander_sim.play import cards
        g = table('galadriel', 'veyran', 'sauron'); p = g.players[0]
        c = hand(p, 'Whipcorder')
        mana.pool_of(p).add('C', 3)
        seat(g, p, [by_text('face down')])
        self.assertIsNone(cards.cast_from_hand(g, p, c))
        m = next(x for x in p.perms if x.cd is c)
        self.assertTrue(m.data.get('facedown')); self.assertTrue(m.neutered)
        mana.pool_of(p).add('W', 1)
        ctl = seat(g, p, [0])
        self.assertIsNone(human.use(g, p, m))                         # its one ability: turn it face up
        self.assertIn('face up', ctl.asked[0].choices[0])
        self.assertFalse(m.neutered)

    def test_mirror_entity(self):
        g = table('galadriel', 'veyran', 'sauron'); p = g.players[0]
        me = perm(g, p, 'Mirror Entity'); q = perm(g, p, 'Jhovall Queen')
        mana.pool_of(p).add('C', 5)
        seat(g, p, [0, by_text('X = 5')])
        self.assertIsNone(human.use(g, p, me))
        self.assertEqual((E.epow(g, q), E.etgh(g, q)), (5, 5))
        self.assertTrue(E.has_type(q, 'goblin'))

    def test_austere_command_modes(self):
        g = table('galadriel', 'veyran'); p = g.players[0]
        seat(g, p, [by_text('artifacts'), by_text('4 or greater')])
        self.assertEqual(ais.wipe_modes(g, p, 'austere2'), {'art', 'ge4'})

    def test_mentor_of_the_meek_asks(self):
        g = table('galadriel', 'veyran'); p = g.players[0]
        perm(g, p, 'Mentor of the Meek'); lands(p, 'Plains', 1)
        ctl = seat(g, p, [0])                                          # yes, pay {1}
        n = len(p.hand)
        perm(g, p, 'Llanowar Elves')
        self.assertEqual(len(p.hand), n + 1)
        self.assertTrue(any('Mentor' in r.prompt for r in ctl.asked))

    def test_type_choice_asks_you(self):
        g = table('galadriel', 'veyran'); p = g.players[0]
        seat(g, p, [by_text('Elf')])
        self.assertEqual(G.ctype(perm(g, p, 'Patchwork Banner')), 'elf')


if __name__ == '__main__':
    unittest.main()
