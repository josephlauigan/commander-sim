"""Your Y'shtola, Night's Blessed deck (cards/impl/yshtola.py): Y'shtola's two triggers, the drain package, the cards
taken from opponents, the deck's other modeled cards, the AI's choices, and the person's choices in practice mode."""
import unittest
from tests.table import table, hand, perm, lands, card
from commander_sim import engine as E, ais
from commander_sim.play import human
from commander_sim.play.controller import ScriptController


Y = None


def setUpModule():
    """the card module is imported once the card database is set up (see test_zur)"""
    global Y
    table('yshtola', 'veyran')
    from commander_sim.cards.impl import yshtola as _y
    Y = _y


def ysh(g, p):
    m = E.enter(g, p, p.cmd); m.is_cmd = True; m.sick = False; p.cmd_in_zone = False
    return m


def cast(g, p, name, **ctx):
    """cast a card from hand (already paid for)"""
    c = hand(p, name)
    E.cast_card(g, p, c, 'hand', ctx)
    return c


def seat(g, p, answers):
    g.controllers = {p.key: ScriptController(answers)}
    return g.controllers[p.key]


def by_text(word):
    return lambda req: next(i for i, x in enumerate(req.choices) if word in x)


def stack_top(p, *names):
    """these cards on top of p's library, the last one on top"""
    for n in names:
        c = card(n)
        if c in p.library: p.library.remove(c)
        p.library.append(c)


class Yshtola(unittest.TestCase):
    def test_a_noncreature_spell_of_mana_value_three_drains_each_opponent(self):
        g = table('yshtola', 'veyran', 'sauron'); y, v, s = g.players
        ysh(g, y)
        cast(g, y, 'Propaganda')
        self.assertEqual((y.life, v.life, s.life), (42, 38, 38))
        self.assertEqual(y.milestone.get('yshtola'), y.turns)

    def test_small_spells_and_creatures_dont(self):
        g = table('yshtola', 'veyran', 'sauron'); y, v, s = g.players
        ysh(g, y)
        cast(g, y, 'Arcane Signet')                              # mana value 2
        cast(g, y, 'Marauding Blight-Priest')                    # a creature
        self.assertEqual((y.life, v.life, s.life), (40, 40, 40))

    def test_x_counts_toward_the_mana_value(self):
        g = table('yshtola', 'veyran', 'sauron'); y, v, s = g.players
        ysh(g, y)
        cast(g, y, 'Exsanguinate', x=1)                          # {1}{B}{B}: mana value 3
        self.assertEqual((v.life, s.life), (37, 37))             # 1 from Exsanguinate, 2 from Y'shtola
        self.assertEqual(y.life, 44)

    def test_opponents_spells_dont(self):
        g = table('yshtola', 'veyran', 'sauron'); y, v, s = g.players
        ysh(g, y)
        cast(g, v, 'Propaganda')
        self.assertEqual((y.life, v.life, s.life), (40, 40, 40))

    def test_end_step_draw_when_a_player_lost_four_life(self):
        g = table('yshtola', 'veyran', 'sauron'); y, v, s = g.players
        ysh(g, y)
        n = len(y.hand)
        E.lose_life(g, s, 2, v); E.lose_life(g, s, 1, v)
        self.assertEqual(Y.lost_this_turn(g, s), 3)              # counted once each (it used to double with hooks)
        ais.end_step(g, v)
        self.assertEqual(len(y.hand), n)
        E.lose_life(g, s, 1, v)
        ais.end_step(g, v)                                       # anyone's end step; any player, you included
        self.assertEqual(len(y.hand), n + 1)

    def test_vigilance(self):
        g = table('yshtola', 'veyran'); y, v = g.players
        self.assertTrue(ysh(g, y).vig)


class Curiosity(unittest.TestCase):
    def test_on_yshtola_it_draws_for_each_opponent_her_trigger_hits(self):
        g = table('yshtola', 'veyran', 'sauron'); y, v, s = g.players
        m = ysh(g, y)
        perm(g, y, 'Curiosity')
        self.assertTrue(any(a.cd.name == 'Curiosity' for a in E.CI.auras_on(g, m)))      # the AI's host
        n = len(y.hand)
        cast(g, y, 'Propaganda')
        self.assertEqual(len(y.hand), n + 2)

    def test_combat_damage_draws(self):
        g = table('yshtola', 'veyran'); y, v = g.players
        t = perm(g, y, 'Thief of Sanity'); t.sick = False
        g.attach_to = t
        perm(g, y, 'Curiosity')
        g.attach_to = None
        n = len(y.hand)
        E.CI.fire(g, 'combat_damage', y, t, v, 2)
        self.assertGreaterEqual(len(y.hand), n + 1)


class Drain(unittest.TestCase):
    def test_blight_priest_and_sanguine_bond(self):
        g = table('yshtola', 'veyran', 'sauron'); y, v, s = g.players
        perm(g, y, 'Marauding Blight-Priest'); perm(g, y, 'Sanguine Bond')
        E.gain(y, 3)
        self.assertEqual(sorted([v.life, s.life]), [36, 39])     # 1 each (Priest), 3 more to one (Bond)

    def test_sanguine_bond_takes_the_kill(self):
        g = table('yshtola', 'veyran', 'sauron'); y, v, s = g.players
        perm(g, y, 'Sanguine Bond')
        perm(g, s, 'Grave Titan')                                # Sauron is the bigger threat
        v.life = 3
        E.gain(y, 3)
        E.check_state(g)
        self.assertFalse(v.alive)
        self.assertEqual(s.life, 40)

    def test_debt_to_the_deathless(self):
        g = table('yshtola', 'veyran', 'sauron'); y, v, s = g.players
        cast(g, y, 'Debt to the Deathless', x=3)
        self.assertEqual((v.life, s.life, y.life), (34, 34, 52))

    def test_the_ai_casts_an_x_drain_for_the_kill(self):
        g = table('yshtola', 'veyran', 'sauron'); y, v, s = g.players
        lands(y, 'Swamp', 6)
        c = hand(y, 'Exsanguinate')
        v.life = 4
        u, label, go = Y.x_drain_option(g, y, c)
        self.assertIn('X = 4', label)
        self.assertGreater(u, 8)
        self.assertTrue(go())
        self.assertFalse(v.alive)
        self.assertEqual(s.life, 36)

    def test_no_small_x_without_a_kill(self):
        g = table('yshtola', 'veyran', 'sauron'); y, v, s = g.players
        lands(y, 'Swamp', 4)
        self.assertIsNone(Y.x_drain_option(g, y, hand(y, 'Exsanguinate')))

    def test_ill_gotten_inheritance(self):
        g = table('yshtola', 'veyran', 'sauron'); y, v, s = g.players
        igi = perm(g, y, 'Ill-Gotten Inheritance')
        E.CI.fire(g, 'upkeep', y)
        self.assertEqual((y.life, v.life, s.life), (41, 39, 39))
        lands(y, 'Swamp', 6)
        v.life = 4
        opts = Y._inheritance_sac(g, igi, y, None, True)
        self.assertEqual(len(opts), 1)
        self.assertTrue(opts[0][2]())
        self.assertFalse(v.alive)
        self.assertNotIn(igi, y.perms)
        self.assertEqual(y.life, 45)

    def test_urborg_syphon_mage(self):
        g = table('yshtola', 'veyran', 'sauron'); y, v, s = g.players
        m = perm(g, y, 'Urborg Syphon-Mage'); m.sick = False
        lands(y, 'Swamp', 3)
        hand(y, 'Island')
        v.life = 2
        opts = Y._syphon(g, m, y, None, True)
        self.assertTrue(opts and opts[0][0] > 8)                 # a kill
        self.assertTrue(opts[0][2]())
        self.assertFalse(v.alive)
        self.assertEqual(s.life, 38)
        self.assertEqual(y.life, 44)
        self.assertTrue(m.tapped)
        self.assertIn(card('Island'), y.gy)


class Stolen(unittest.TestCase):
    def test_gonti_takes_a_card_castable_with_any_mana(self):
        g = table('yshtola', 'veyran'); y, v = g.players
        stack_top(v, 'Guttersnipe', 'Island', 'Island', 'Island')
        perm(g, y, 'Gonti, Lord of Luxury')
        c = card('Guttersnipe')
        self.assertIn(c, y.hand)
        self.assertEqual(E.cost_of(y, c), (3, ''))               # {2}{R}: the red paid with any mana
        self.assertEqual([x.name for x in v.library[:3]].count('Island'), 3)    # the rest on the bottom
        lands(y, 'Swamp', 3)
        E.pay(g, y, 3, '')
        E.cast_card(g, y, c, 'hand')
        m = next(x for x in y.perms if x.cd is c)
        self.assertIs(m.orig, v)                                 # still Veyran's card
        E.die(g, m, 'destroy')
        self.assertIn(c, v.gy)
        self.assertNotIn(c, y.gy)

    def test_a_taken_spell_goes_to_its_owners_graveyard(self):
        g = table('yshtola', 'veyran'); y, v = g.players
        c = card('Night\'s Whisper')
        Y.take_card(g, y, c, v, 'test')
        E.cast_card(g, y, c, 'hand')
        self.assertIn(c, v.gy)

    def test_taken_cards_arent_discarded_to_hand_size(self):
        g = table('yshtola', 'veyran'); y, v = g.players
        c = card('Guttersnipe')
        Y.take_card(g, y, c, v, 'test')
        hand(y, 'Island', 'Plains', 'Swamp', 'Sol Ring', 'Arrest', 'Curiosity', 'Propaganda')
        ais.end_step(g, y)
        self.assertIn(c, y.hand)
        self.assertEqual(len(y.hand), 8)

    def test_hostage_taker_returns_the_creature_when_it_leaves(self):
        g = table('yshtola', 'veyran'); y, v = g.players
        gs = perm(g, v, 'Guttersnipe')
        t = perm(g, y, 'Hostage Taker')
        self.assertNotIn(gs, v.perms)
        self.assertIn(card('Guttersnipe'), y.hand)
        E.die(g, t, 'destroy')
        self.assertTrue(any(m.cd is card('Guttersnipe') for m in v.perms))
        self.assertNotIn(card('Guttersnipe'), y.hand)

    def test_hostage_taker_cast_it_and_keep_it(self):
        g = table('yshtola', 'veyran'); y, v = g.players
        perm(g, v, 'Guttersnipe')
        t = perm(g, y, 'Hostage Taker')
        lands(y, 'Swamp', 3); E.pay(g, y, 3, '')
        E.cast_card(g, y, card('Guttersnipe'), 'hand')
        E.die(g, t, 'destroy')
        m = next(x for x in y.perms if x.cd is card('Guttersnipe'))
        self.assertIs(m.orig, v)
        self.assertFalse(any(x.cd is card('Guttersnipe') for x in v.perms))

    def test_thief_of_sanity(self):
        g = table('yshtola', 'veyran'); y, v = g.players
        stack_top(v, 'Island', 'Guttersnipe', 'Island')
        t = perm(g, y, 'Thief of Sanity'); t.sick = False
        E.CI.fire(g, 'combat_damage', y, t, v, 2)
        self.assertIn(card('Guttersnipe'), y.hand)
        self.assertEqual([c.name for c in v.gy].count('Island'), 2)


class OtherCards(unittest.TestCase):
    def test_jesters_cap(self):
        g = table('yshtola', 'veyran'); y, v = g.players
        cap = perm(g, y, "Jester's Cap")
        n = len(v.library)
        Y.cap_use(g, y, cap, v)
        self.assertEqual(len(v.library), n - 3)
        self.assertEqual(len(v.exile), 3)
        self.assertNotIn(cap, y.perms)

    def test_dark_petition_spell_mastery(self):
        g = table('yshtola', 'veyran'); y, v = g.players
        n = len(y.hand)
        cast(g, y, 'Dark Petition')
        self.assertEqual(len(y.hand), n + 1)
        self.assertEqual(y.floatB, 0)                            # no instants or sorceries in the graveyard yet
        y.gy += [card('Night\'s Whisper'), card('Read the Bones')]
        cast(g, y, 'Diabolic Tutor')                             # (a plain tutor, for comparison)
        y.hand.append(card('Dark Petition')); y.gy.remove(card('Dark Petition'))
        E.cast_card(g, y, card('Dark Petition'), 'hand')
        self.assertEqual(y.floatB, 3)

    def test_plea_for_guidance_finds_two_enchantments(self):
        g = table('yshtola', 'veyran'); y, v = g.players
        cast(g, y, 'Plea for Guidance')
        self.assertEqual(sum(1 for c in y.hand if 'E' in c.types), 2)

    def test_take_up_the_shield_saves_yshtola(self):
        g = table('yshtola', 'veyran'); y, v = g.players
        m = ysh(g, y)
        lands(y, 'Plains', 2)
        hand(y, 'Take Up the Shield')
        E.apply_removal(g, v, m, 'destroy', card('Go for the Throat'))
        self.assertIn(m, y.perms)
        self.assertEqual(m.plus, 1)
        self.assertTrue(E.CI.granted_kw(g, m, 'lifelink') or 'lifelink' in g.eot_kw.get(id(m), ()))

    def test_zur_in_the_99_fetches_for_this_deck(self):
        g = table('yshtola', 'veyran'); y, v = g.players
        ysh(g, y)
        z = perm(g, y, 'Zur the Enchanter'); z.sick = False
        y.library = [c for c in y.library if not ('E' in c.types and c.cmc <= 3) or c.name in ('Curiosity', 'Propaganda')]
        ais.attack_triggers(g, y, [z], v)
        got = [m for m in y.perms if m.cd is not None and 'E' in m.cd.types]
        self.assertEqual([m.cd.name for m in got], ['Curiosity'])          # on Y'shtola: it draws off every trigger
        self.assertIs(got[0].attached, Y.ysh_perm(y))

    def test_yshtola_doesnt_chump_block_unless_it_would_kill_you(self):
        g = table('veyran', 'yshtola'); v, y = g.players
        m = ysh(g, y)
        big = perm(g, v, 'Grave Titan'); big.sick = False
        for i in range(20):                                      # 6 damage at 10 life: other decks chump half the time
            g.rng.seed(i); y.life = 10
            ais.resolve_combat(g, v, [big], y, set())
            self.assertIn(m, y.perms)
            self.assertEqual(y.life, 4)


class AI(unittest.TestCase):
    def test_a_spell_that_triggers_yshtola_ranks_higher(self):
        g = table('yshtola', 'veyran', 'sauron'); y, v, s = g.players
        c = card('Read the Bones')
        before = Y.yshtola_prio(g, y, c)
        ysh(g, y)
        self.assertGreater(Y.yshtola_prio(g, y, c), before)

    def test_tutors_find_sanguine_bond(self):
        g = table('yshtola', 'veyran'); y, v = g.players
        y.turns = 8; y.life = 20
        self.assertEqual(ais.tutor_pick(g, y, 'any'), 'Sanguine Bond')
        perm(g, y, 'Sanguine Bond')
        self.assertNotEqual(ais.tutor_pick(g, y, 'ench'), 'Sanguine Bond')

    def test_idyllic_tutor_uses_the_wish_list(self):
        g = table('yshtola', 'veyran'); y, v = g.players
        y.turns = 8; y.life = 20
        cast(g, y, 'Idyllic Tutor')
        self.assertIn(card('Sanguine Bond'), y.hand)

    def test_tutors_find_the_finisher_when_it_kills(self):
        g = table('yshtola', 'veyran', 'sauron'); y, v, s = g.players
        lands(y, 'Swamp', 9)
        v.life = s.life = 6
        self.assertIn(ais.tutor_pick(g, y, 'any'), ('Debt to the Deathless', 'Exsanguinate'))


class Practice(unittest.TestCase):
    def test_your_spell_triggers_yshtola(self):
        g = table('yshtola', 'veyran', 'sauron'); y, v, s = g.players
        ysh(g, y)
        from commander_sim.play import mana
        mana.pool_of(y).add('U', 1); mana.pool_of(y).add('C', 2)
        hand(y, 'Propaganda')
        seat(g, y, [{'do': 'cast', 'card': 0}, {'do': 'pass'}])
        human.human_main(g, y, False)
        self.assertEqual((y.life, v.life, s.life), (42, 38, 38))

    def test_you_choose_x(self):
        g = table('yshtola', 'veyran', 'sauron'); y, v, s = g.players
        from commander_sim.play import mana
        mana.pool_of(y).add('B', 2); mana.pool_of(y).add('C', 4)
        hand(y, 'Exsanguinate')
        seat(g, y, [{'do': 'cast', 'card': 0}, by_text('X = 3'), {'do': 'pass'}])
        human.human_main(g, y, False)
        self.assertEqual((v.life, s.life, y.life), (37, 37, 46))

    def test_syphon_mage_for_the_person(self):
        g = table('yshtola', 'veyran', 'sauron'); y, v, s = g.players
        m = perm(g, y, 'Urborg Syphon-Mage'); m.sick = False
        from commander_sim.play import mana
        mana.pool_of(y).add('B', 1); mana.pool_of(y).add('C', 2)
        hand(y, 'Island', 'Arrest')
        seat(g, y, [0, by_text('Island')])
        self.assertIsNone(human.apply(g, y, {'do': 'use', 'perm': y.perms.index(m)}))
        self.assertEqual((v.life, s.life, y.life), (38, 38, 44))
        self.assertEqual([c.name for c in y.hand], ['Arrest'])

    def test_inheritance_and_cap_for_the_person(self):
        g = table('yshtola', 'veyran', 'sauron'); y, v, s = g.players
        igi = perm(g, y, 'Ill-Gotten Inheritance'); cap = perm(g, y, "Jester's Cap")
        from commander_sim.play import mana
        mana.pool_of(y).add('B', 1); mana.pool_of(y).add('C', 7)
        seat(g, y, [0, by_text('Sauron'), 0, by_text('Veyran'), 0, 0, 0])
        self.assertIsNone(human.apply(g, y, {'do': 'use', 'perm': y.perms.index(igi)}))
        self.assertEqual((s.life, y.life), (36, 44))
        n = len(v.library)
        self.assertIsNone(human.apply(g, y, {'do': 'use', 'perm': y.perms.index(cap)}))
        self.assertEqual(len(v.library), n - 3)

    def test_you_pick_gontis_card(self):
        g = table('yshtola', 'veyran'); y, v = g.players
        stack_top(v, 'Guttersnipe', 'Counterspell', 'Island', 'Island')
        seat(g, y, [by_text('Guttersnipe')])
        perm(g, y, 'Gonti, Lord of Luxury')
        self.assertIn(card('Guttersnipe'), y.hand)
        self.assertNotIn(card('Counterspell'), y.hand)

    def test_you_pick_hostage_takers_target(self):
        g = table('yshtola', 'veyran'); y, v = g.players
        a = perm(g, v, 'Guttersnipe'); b = perm(g, v, 'Sol Ring')
        seat(g, y, [by_text('Sol Ring')])
        perm(g, y, 'Hostage Taker')
        self.assertIn(a, v.perms); self.assertNotIn(b, v.perms)
        self.assertIn(card('Sol Ring'), y.hand)

    def test_you_search_with_idyllic_tutor(self):
        g = table('yshtola', 'veyran'); y, v = g.players
        seat(g, y, [by_text('Propaganda')])
        cast(g, y, 'Idyllic Tutor')
        self.assertIn(card('Propaganda'), y.hand)

    def test_take_up_the_shield_target(self):
        g = table('yshtola', 'veyran'); y, v = g.players
        m = ysh(g, y)
        from commander_sim.play import mana
        mana.pool_of(y).add('W', 1); mana.pool_of(y).add('C', 1)
        hand(y, 'Take Up the Shield')
        seat(g, y, [{'do': 'cast', 'card': 0}, by_text("Y'shtola"), {'do': 'pass'}])
        human.human_main(g, y, False)
        self.assertEqual(m.plus, 1)


if __name__ == '__main__':
    unittest.main()
