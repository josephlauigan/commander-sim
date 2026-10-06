"""Avery's Alela deck (cards/impl/alela.py): Alela's Faeries, the Jace token, the removal and control cards, energy,
the creatures with triggers, Venser and the deck's other modeled cards."""
import unittest
from tests.table import table, hand, perm, lands, card, token
from commander_sim import engine as E, ais


A = None


def setUpModule():
    global A
    table('alela', 'veyran')
    from commander_sim.cards.impl import alela as _a
    A = _a


def alela(g, p):
    m = E.enter(g, p, p.cmd); m.is_cmd = True; p.cmd_in_zone = False
    return m


def names(p):
    return [m.name for m in p.perms]


def opts(g, c, p, event, post=False, src=None):
    from commander_sim.ai import brain
    fn = E.CI.HOOKS[c.name][event]
    return fn(g, src, p, brain.Situation(g, p), post) if src is not None else fn(g, c, p, brain.Situation(g, p), post)


class Faeries(unittest.TestCase):
    def test_artifact_spell_makes_a_faerie_with_alela_out(self):
        g = table('alela', 'veyran'); p = g.players[0]
        alela(g, p)
        lands(p, 'Plains', 2)
        c = hand(p, 'Mind Stone')
        E.pay(g, p, 2, ''); E.cast_card(g, p, c)
        toks = [m for m in p.perms if m.token]
        self.assertEqual(len(toks), 1)
        self.assertTrue(toks[0].fly)
        self.assertEqual(E.epow(g, toks[0]), 2)          # Alela's +1/+0 to other fliers


class JaceToken(unittest.TestCase):
    def test_empower_creates_then_grows(self):
        g = table('alela', 'veyran'); p = g.players[0]
        A.empower(g, p, 2)
        j = A.jace_token(p)
        self.assertEqual(j.loyalty, 2)
        A.empower(g, p, 1)
        self.assertIs(A.jace_token(p), j)
        self.assertEqual(j.loyalty, 3)

    def test_keeper_empowers_two(self):
        g = table('alela', 'veyran'); p = g.players[0]
        perm(g, p, 'Keeper of the Quiet Hour')
        self.assertEqual(A.jace_token(p).loyalty, 2)

    def test_annex_enters_untapped_with_a_planeswalker(self):
        g = table('alela', 'veyran'); p = g.players[0]
        lands(p, 'Plains', 3)
        annex = card('Fatehold Annex')
        self.assertTrue(ais.land_enters_tapped(p, annex))
        A.empower(g, p, 1)
        self.assertFalse(ais.land_enters_tapped(p, annex))

    def test_the_token_leaves_no_card_behind(self):
        g = table('alela', 'veyran'); p = g.players[0]
        A.empower(g, p, 1)
        j = A.jace_token(p)
        gy = len(p.gy)
        E.bounce(g, j)
        self.assertIsNone(A.jace_token(p))
        self.assertEqual(len(p.gy), gy)
        self.assertFalse(any(c.name == A.JACE for c in p.hand))

    def test_minus_three_draws(self):
        g = table('alela', 'veyran'); p = g.players[0]
        A.empower(g, p, 3)
        j = A.jace_token(p)
        o = opts(g, j.cd, p, 'options', src=j)
        best = max(o, key=lambda x: x[0])
        self.assertIn('-3', best[1])
        n = len(p.hand); best[2]()
        self.assertEqual(len(p.hand), n + 1)
        self.assertIsNone(A.jace_token(p))                # loyalty 0: gone

    def test_plan_empowers_on_first_noncreature_spell_only(self):
        g = table('alela', 'veyran'); p = g.players[0]
        perm(g, p, 'Plan for All Outcomes')
        for n in ('Night\'s Whisper', 'Sol Ring'):
            E.on_cast(g, p, card(n))
        self.assertEqual(A.jace_token(p).loyalty, 1)


class Removal(unittest.TestCase):
    def test_multiply_by_zero_kills_unless_counters(self):
        g = table('alela', 'veyran'); p, q = g.players
        a = perm(g, q, 'Grave Titan'); b = perm(g, q, 'Serra Angel'); b.plus = 1
        self.assertIn(a, E.legal_targets(g, p, 'zero', 'c'))
        self.assertNotIn(b, E.legal_targets(g, p, 'zero', 'c'))
        E.apply_removal(g, p, a, 'zero')
        self.assertNotIn(a, q.perms)

    def test_prophesied_end_draws_unless_attacking(self):
        g = table('alela', 'veyran'); p, q = g.players
        spell = card('Prophesied End')
        a = perm(g, q, 'Grave Titan'); n = len(q.hand)
        E.apply_removal(g, p, a, 'destroy', spell)
        self.assertEqual(len(q.hand), n + 1)
        b = perm(g, q, 'Serra Angel'); g.in_combat = {b}
        E.apply_removal(g, p, b, 'destroy', spell)
        g.in_combat = ()
        self.assertEqual(len(q.hand), n + 1)

    def test_plan_puts_the_permanent_on_top(self):
        g = table('alela', 'veyran'); p, q = g.players
        t = perm(g, q, 'Grave Titan')
        perm(g, p, 'Plan for All Outcomes')
        self.assertNotIn(t, q.perms)
        self.assertEqual(q.library[-1].name, 'Grave Titan')

    def test_karmic_justice_answers_an_opponents_destroy(self):
        g = table('alela', 'veyran'); p, q = g.players
        perm(g, p, 'Karmic Justice'); stone = perm(g, p, 'Mind Stone')
        t = perm(g, q, 'Grave Titan')
        E.apply_removal(g, q, stone, 'destroy', card('Abrade'))
        self.assertNotIn(t, q.perms)

    def test_karmic_justice_ignores_creatures_and_your_own(self):
        g = table('alela', 'veyran'); p, q = g.players
        perm(g, p, 'Karmic Justice'); c = perm(g, p, 'Serra Angel'); stone = perm(g, p, 'Mind Stone')
        t = perm(g, q, 'Grave Titan')
        E.apply_removal(g, q, c, 'destroy')
        E.apply_removal(g, p, stone, 'destroy')
        self.assertIn(t, q.perms)

    def test_ray_of_command_steals_and_returns_tapped(self):
        g = table('alela', 'veyran'); p, q = g.players
        lands(p, 'Island', 4)
        t = perm(g, q, 'Grave Titan'); t.tapped = True
        c = hand(p, 'Ray of Command')
        o = opts(g, c, p, 'hand_options', post=False)
        self.assertTrue(o and o[0][2]())
        self.assertIn(t, p.perms)
        self.assertFalse(t.tapped)
        ais.end_step(g, p)
        self.assertIn(t, q.perms)
        self.assertTrue(t.tapped)

    def test_malice_destroys_a_nonblack_creature(self):
        g = table('alela', 'veyran'); p, q = g.players
        lands(p, 'Swamp', 4)
        t = perm(g, q, 'Consecrated Sphinx'); perm(g, q, 'Grave Titan')       # Grave Titan is black
        c = hand(p, 'Spite // Malice')
        o = opts(g, c, p, 'hand_options', post=False)
        self.assertEqual(len(o), 1)
        o[0][2]()
        self.assertNotIn(t, q.perms)

    def test_muddle_counters_only_instants_and_sorceries(self):
        m = card('Muddle the Mixture')
        self.assertTrue(E.counter_ok(m, card('Counterspell')))
        self.assertFalse(E.counter_ok(m, card('Rhystic Study')))


class Opposition(unittest.TestCase):
    def test_taps_their_mana_in_their_upkeep(self):
        g = table('alela', 'veyran'); p, q = g.players
        perm(g, p, 'Opposition')
        for _ in range(2): token(g, p, 1, fly=True)
        lands(q, 'Island', 3)
        g.active = q
        E.CI.fire(g, 'upkeep', q)
        self.assertEqual(sum(1 for L in q.lands if L.tapped), 2)

    def test_taps_attackers_at_their_combat(self):
        g = table('alela', 'veyran'); p, q = g.players
        perm(g, p, 'Opposition')
        token(g, p, 1, fly=True)
        t = perm(g, q, 'Grave Titan')
        g.active = q
        A.opposition_precombat(g, p)
        self.assertTrue(t.tapped)


class Energy(unittest.TestCase):
    def test_static_prison_pays_then_goes(self):
        g = table('alela', 'veyran'); p, q = g.players
        t = perm(g, q, 'Grave Titan')
        sp = perm(g, p, 'Static Prison')
        self.assertNotIn(t, q.perms)
        self.assertEqual(p.energy, 2)
        for _ in range(2): E.CI.fire(g, 'upkeep', p)
        self.assertIn(sp, p.perms)
        E.CI.fire(g, 'upkeep', p)
        self.assertNotIn(sp, p.perms)
        self.assertTrue(any(m.name == 'Grave Titan' for m in q.perms))

    def test_aether_hub_colour_costs_energy(self):
        g = table('alela', 'veyran'); p = g.players[0]
        hub = card('Aether Hub')
        L = E.Land(hub, False); p.lands.append(L)
        E.CI.LAND_ETB['Aether Hub'](g, p, L)
        self.assertEqual(E.land_cols(p, L, False), p.ident)
        E.pay(g, p, 0, 'W')
        self.assertEqual(p.energy, 0)
        L.tapped = False
        self.assertEqual(E.land_cols(p, L, False), '')


class Creatures(unittest.TestCase):
    def test_plumecreed_counts_faerie_tokens(self):
        g = table('alela', 'veyran'); p = g.players[0]
        perm(g, p, 'Plumecreed Mentor')
        page = perm(g, p, 'Page, Loose Leaf')
        token(g, p, 1, fly=True)
        self.assertEqual(page.plus, 1)

    def test_jackdaw_returns_a_cheaper_creature(self):
        g = table('alela', 'veyran'); p = g.players[0]
        p.gy.append(card('Initiates of the Ebon Hand'))
        j = perm(g, p, 'Jackdaw Savior')
        E.die(g, j, 'destroy')
        self.assertIn('Initiates of the Ebon Hand', names(p))

    def test_malcator_golem_at_end_step_after_three_artifacts(self):
        g = table('alela', 'veyran'); p = g.players[0]
        perm(g, p, 'Malcator, Purity Overseer')
        golems = lambda: sum(1 for m in p.perms if m.token and 'golem' in m.ttypes)
        self.assertEqual(golems(), 1)
        for n in ('Mind Stone', 'Sol Ring'): perm(g, p, n)
        E.CI.fire(g, 'end_step', p)
        self.assertEqual(golems(), 2)                     # its own Golem was the third artifact

    def test_airlift_chaplain_keeps_a_small_creature(self):
        g = table('alela', 'veyran'); p = g.players[0]
        for n in ('Island', 'Island', 'Initiates of the Ebon Hand'): p.library.append(card(n))
        ch = perm(g, p, 'Airlift Chaplain')
        self.assertEqual([c.name for c in p.hand], ['Initiates of the Ebon Hand'])
        self.assertEqual(ch.plus, 0)

    def test_airlift_chaplain_counter_when_nothing_fits(self):
        g = table('alela', 'veyran'); p = g.players[0]
        for n in ('Island', 'Island', 'Island'): p.library.append(card(n))
        ch = perm(g, p, 'Airlift Chaplain')
        self.assertEqual(p.hand, [])
        self.assertEqual(ch.plus, 1)

    def test_skycoach_crews_with_a_sick_creature(self):
        g = table('alela', 'veyran'); p = g.players[0]
        sc = perm(g, p, 'Strixhaven Skycoach')
        perm(g, p, 'Plumecreed Mentor', sick=True)
        E.CI.fire(g, 'crew', p)
        self.assertTrue(sc.creature)
        self.assertEqual(E.epow(g, sc), 3)


class Spells(unittest.TestCase):
    def test_helping_hand_returns_tapped(self):
        g = table('alela', 'veyran'); p = g.players[0]
        p.gy.append(card('Plumecreed Mentor'))
        A._helping_hand(g, p, card('Helping Hand'), {})
        m = next(m for m in p.perms if m.name == 'Plumecreed Mentor')
        self.assertTrue(m.tapped)

    def test_daydream_blinks_with_a_counter(self):
        g = table('alela', 'veyran'); p = g.players[0]
        perm(g, p, 'Keeper of the Quiet Hour')
        A._daydream(g, p, card('Daydream'), {})
        k = next(m for m in p.perms if m.name == 'Keeper of the Quiet Hour')
        self.assertEqual(k.plus, 1)
        self.assertEqual(A.jace_token(p).loyalty, 4)       # empowered again

    def test_ajani_chapters(self):
        g = table('alela', 'veyran'); p, q = g.players
        t = perm(g, q, 'Grave Titan')
        aj = perm(g, p, 'Ajani Fells the Godsire')
        self.assertNotIn(t, q.perms)
        E.CI.fire(g, 'upkeep', p)
        self.assertTrue(any(m.token and 'cat' in m.ttypes for m in p.perms))
        E.CI.fire(g, 'upkeep', p)
        self.assertNotIn(aj, p.perms)

    def test_venser_emblem_exiles_per_spell(self):
        g = table('alela', 'veyran'); p, q = g.players
        from commander_sim.cards.impl import rules
        rules.give_emblem(p, 'venser')
        t = perm(g, q, 'Grave Titan')
        E.on_cast(g, p, card('Sol Ring'))
        self.assertNotIn(t, q.perms)


if __name__ == '__main__':
    unittest.main()
