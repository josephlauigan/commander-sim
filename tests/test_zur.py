"""Cards first written for the Zur deck (removed 2026-10-07) that other lists still run (cards/impl/zur.py): the Auras
that lock a permanent down, Zur's attack trigger in your Y'shtola deck's 99, the other modeled cards, the AI's
responses, and the person's choices in practice mode. The seat is your Y'shtola deck, which runs most of them."""
import unittest
from tests.table import table, hand, perm, lands, token, card
from commander_sim import engine as E, ais
from commander_sim.play import combat, human
from commander_sim.play.controller import ScriptController


Z = IC = None


def setUpModule():
    """the card modules are imported once the card database is set up (a top-level import here would run at test
    discovery, before the engine's own load order, and change which card definitions win)"""
    global Z, IC
    table('yshtola', 'veyran')
    from commander_sim.cards.impl import zur as _z, common as _c
    Z, IC = _z, _c


def cmd(g, p, sick=False):
    """your commander onto the battlefield"""
    m = E.enter(g, p, p.cmd, sick=sick); m.is_cmd = True; p.cmd_in_zone = False
    return m


def zur(g, p):
    """Zur the Enchanter from the 99, ready to attack"""
    z = perm(g, p, 'Zur the Enchanter'); z.sick = False
    return z


def cast(g, p, name, target=None):
    """cast a card from hand (already paid for), with the target chosen as it's cast"""
    c = hand(p, name)
    E.cast_card(g, p, c, 'hand', {'target': target} if target is not None else {})
    return next((m for m in p.perms if m.cd is c), None)


def seat(g, p, answers):
    g.controllers = {p.key: ScriptController(answers)}
    return g.controllers[p.key]


def by_text(word):
    return lambda req: next(i for i, x in enumerate(req.choices) if word in x)


class Locks(unittest.TestCase):
    def test_arrest_stops_attacks_blocks_and_abilities_until_it_leaves(self):
        g = table('yshtola', 'veyran', 'sauron'); z, v, s = g.players
        gm = perm(g, v, 'Harmonic Prodigy')
        a = cast(g, z, 'Arrest', gm)
        self.assertIs(a.attached, gm)
        self.assertFalse(combat.can_attack(g, v, gm))
        self.assertFalse(ais.can_block(g, gm, cmd(g, z)))
        from commander_sim.cards.impl import rules
        self.assertTrue(rules.ability_locked(g, gm, v))
        E.die(g, a, 'destroy')                                   # the Aura goes: the creature is free again
        gm.sick = False
        self.assertTrue(combat.can_attack(g, v, gm))
        self.assertFalse(E.CI.locked(g, gm, 'pacify'))

    def test_luminous_bonds_leaves_abilities_alone(self):
        g = table('yshtola', 'veyran', 'sauron'); z, v, s = g.players
        m = perm(g, v, 'Guttersnipe'); m.sick = False
        cast(g, z, 'Luminous Bonds', m)
        self.assertFalse(combat.can_attack(g, v, m))
        self.assertTrue(E.CI.locked(g, m, 'pacify'))
        self.assertFalse(E.CI.locked(g, m, 'noact'))

    def test_encrust_stops_mana_and_untapping(self):
        g = table('yshtola', 'veyran', 'sauron'); z, v, s = g.players
        rock = perm(g, v, 'Sol Ring')
        before = E.total_mana(g, v)
        cast(g, z, 'Encrust', rock)
        self.assertEqual(E.total_mana(g, v), before - 2)          # its mana ability can't be activated
        cr = perm(g, s, 'Orcish Bowmasters'); cr.tapped = True
        hand(z, 'Encrust')                                       # (a second copy, from the table helper's card)
        E.cast_card(g, z, z.hand[-1], 'hand', {'target': cr})
        ais._step_start(g, s)
        self.assertTrue(cr.tapped)                               # doesn't untap

    def test_a_locked_creature_is_worth_less(self):
        g = table('yshtola', 'veyran', 'sauron'); z, v, s = g.players
        m = perm(g, v, 'Niv-Mizzet, Parun')
        before = E.pval(g, m)
        cast(g, z, 'Arrest', m)
        self.assertLess(E.pval(g, m), before * 0.5)
        self.assertEqual(E.legal_targets(g, z, 'arrest', 'c', spell=card('Prison Sentence')), [])   # not twice

    def test_no_target_no_aura(self):
        g = table('yshtola', 'veyran', 'sauron'); z, v, s = g.players
        c = hand(z, 'Bound in Silence')
        E.cast_card(g, z, c, 'hand', {})
        self.assertIn(c, z.gy)
        self.assertFalse(any(m.cd is c for m in z.perms))


class ZurTrigger(unittest.TestCase):
    def test_a_fetched_lock_aura_ignores_hexproof(self):
        g = table('yshtola', 'veyran', 'sauron'); z, v, s = g.players
        zz = zur(g, z)
        big = perm(g, v, 'Niv-Mizzet, Parun')
        g.eot_kw[id(big)] = {'hexproof'}
        self.assertTrue(E.untargetable(g, big))
        self.assertIsNone(Z.lock_host(g, z, 'Arrest', targeted=True))      # a cast Arrest can't target it
        z.library = [c for c in z.library if not ('E' in c.types and c.cmc <= 3) or c.name == 'Arrest']
        ais.attack_triggers(g, z, [zz], v)
        self.assertTrue(E.CI.locked(g, big, 'pacify'))

    def test_never_more_than_mana_value_three(self):
        g = table('yshtola', 'veyran', 'sauron'); z, v, s = g.players
        zz = zur(g, z)
        z.library = [c for c in z.library if not ('E' in c.types and c.cmc <= 3)]
        ais.attack_triggers(g, z, [zz], v)
        self.assertFalse(any(m.cd is not None and 'E' in m.cd.types for m in z.perms))


class Cards(unittest.TestCase):
    def test_bastion_protector_guards_only_the_commander(self):
        g = table('yshtola', 'veyran', 'sauron'); z, v, s = g.players
        y = cmd(g, z); bp = perm(g, z, 'Bastion Protector')
        self.assertEqual((E.epow(g, y), E.etgh(g, y)), (4, 6))     # Y'shtola is 2/4
        self.assertTrue(E.indestructible(g, y))
        self.assertFalse(E.indestructible(g, bp))

    def test_the_eternal_wanderer(self):
        g = table('yshtola', 'veyran', 'sauron'); z, v, s = g.players
        w = perm(g, z, 'The Eternal Wanderer')
        self.assertEqual(w.loyalty, 5)
        big = perm(g, v, 'Niv-Mizzet, Parun')
        Z.wanderer_exile(g, z, big)
        self.assertNotIn(big, v.perms)
        ais.end_step(g, s)                                       # not its owner's end step
        self.assertFalse(any(m.cd is not None and m.cd.name == 'Niv-Mizzet, Parun' for m in v.perms))
        ais.end_step(g, v)
        self.assertTrue(any(m.cd is not None and m.cd.name == 'Niv-Mizzet, Parun' for m in v.perms))
        for x in E.make_tokens(g, z, 1, 2, color='W'): x.data = {'kws': ('double strike',)}
        self.assertTrue(ais.double_strike(z, [m for m in z.perms if m.token][0]))
        y = cmd(g, z); perm(g, z, 'Esper Sentinel'); perm(g, v, 'Guttersnipe')
        Z.wanderer_ult(g, z)
        self.assertEqual([m.cd.name for m in z.perms if m.creature], [y.cd.name])
        self.assertEqual(sum(1 for m in v.perms if m.creature), 1)

    def test_prayer_of_binding_until_it_leaves(self):
        g = table('yshtola', 'veyran', 'sauron'); z, v, s = g.players
        big = perm(g, v, 'Niv-Mizzet, Parun')
        life = z.life
        pr = perm(g, z, 'Prayer of Binding')
        self.assertNotIn(big, v.perms); self.assertEqual(z.life, life + 2)
        E.die(g, pr, 'destroy')
        self.assertTrue(any(m.cd is not None and m.cd.name == 'Niv-Mizzet, Parun' for m in v.perms))

    def test_disenchant_targets(self):
        g = table('yshtola', 'veyran', 'sauron'); z, v, s = g.players
        big = perm(g, v, 'Niv-Mizzet, Parun'); rock = perm(g, v, 'Sol Ring')
        dis = E.legal_targets(g, z, 'destroy', 'ae', spell=card('Disenchant'))
        self.assertIn(rock, dis); self.assertNotIn(big, dis)

    def test_rootborn_defenses_against_a_wipe(self):
        g = table('yshtola', 'veyran', 'sauron'); z, v, s = g.players
        lands(z, 'Plains', 3)
        y = cmd(g, z); perm(g, z, 'Restoration Angel'); hand(z, 'Rootborn Defenses')
        self.assertEqual(ais.wipe_response(g, z, 'destroy', v), 'indes')
        self.assertTrue(E.indestructible(g, y))


class Practice(unittest.TestCase):
    def test_you_cast_arrest_and_choose_its_target_once(self):
        g = table('yshtola', 'veyran', 'sauron'); z, v, s = g.players
        m = perm(g, v, 'Guttersnipe'); perm(g, s, 'Orcish Bowmasters')
        lands(z, 'Plains', 3)
        from commander_sim.play import mana
        mana.pool_of(z).add('W', 1); mana.pool_of(z).add('C', 2)
        hand(z, 'Arrest')
        ctl = seat(g, z, [{'do': 'cast', 'card': 0}, by_text('Guttersnipe'), {'do': 'pass'}])
        human.human_main(g, z, False)
        self.assertTrue(E.CI.locked(g, m, 'pacify'))
        self.assertEqual(sum(1 for r in ctl.asked if r.kind == 'target'), 1)

    def test_zur_fetch_is_yours_to_choose(self):
        g = table('yshtola', 'veyran', 'sauron'); z, v, s = g.players
        zz = zur(g, z)
        m = perm(g, v, 'Guttersnipe')
        seat(g, z, [by_text('Luminous Bonds'), by_text('Guttersnipe')])
        ais.attack_triggers(g, z, [zz], v)
        self.assertTrue(E.CI.locked(g, m, 'pacify'))

    def test_clever_concealment_with_convoke(self):
        g = table('yshtola', 'veyran', 'sauron'); z, v, s = g.players
        y = cmd(g, z); perm(g, z, 'Esper Sentinel'); perm(g, z, 'Restoration Angel')
        from commander_sim.play import mana, cards
        mana.pool_of(z).add('W', 1); mana.pool_of(z).add('C', 1)
        c = hand(z, 'Clever Concealment')
        seat(g, z, [by_text("Y'shtola"), 'cancel', by_text('Esper Sentinel'), by_text('Restoration Angel'), 'cancel'])
        self.assertIsNone(cards.cast_from_hand(g, z, c))
        self.assertTrue(y.phased)
        self.assertIn(c, z.gy)

    def test_the_wanderer_for_the_person(self):
        g = table('yshtola', 'veyran', 'sauron'); z, v, s = g.players
        w = perm(g, z, 'The Eternal Wanderer')
        seat(g, z, [by_text('0:')])
        self.assertIsNone(human.use(g, z, w))
        sam = [m for m in z.perms if m.token]
        self.assertEqual(len(sam), 1)
        self.assertTrue(ais.double_strike(z, sam[0]))


class Necropotence(unittest.TestCase):
    def test_you_pay_life_and_get_the_cards_at_your_end_step(self):
        g = table('yshtola', 'veyran', 'sauron'); z = g.players[0]
        necro = perm(g, z, 'Necropotence')
        hand_n, lib_n, life = len(z.hand), len(z.library), z.life
        seat(g, z, [0, by_text('pay 3 life')])
        self.assertIsNone(human.use(g, z, necro))
        self.assertEqual((z.life, len(z.library), len(z.hand)), (life - 3, lib_n - 3, hand_n))   # face down for now
        ais.end_step(g, z)
        self.assertEqual(len(z.hand), hand_n + 3)

    def test_paid_in_your_end_step_they_wait_a_turn(self):
        g = table('yshtola', 'veyran', 'sauron'); z = g.players[0]
        necro = perm(g, z, 'Necropotence')
        g.step = 'end'
        n = len(z.hand)
        seat(g, z, [0, by_text('pay 2 life')])
        human.use(g, z, necro)
        ais.end_step(g, z)
        self.assertEqual(len(z.hand), n)
        z.turns += 1
        ais.end_step(g, z)
        self.assertEqual(len(z.hand), n + 2)

    def test_the_ai_does_not_pay_for_you(self):
        g = table('yshtola', 'veyran', 'sauron'); z = g.players[0]
        perm(g, z, 'Necropotence')
        seat(g, z, [])
        life = z.life
        ais.end_step(g, z)
        self.assertEqual(z.life, life)


if __name__ == '__main__':
    unittest.main()
