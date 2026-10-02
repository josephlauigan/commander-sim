"""Your Zur the Enchanter deck (cards/impl/zur.py): the Auras that lock a permanent down, Zur's attack trigger, the
deck's other modeled cards, the AI's responses, and the person's choices in practice mode."""
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
    table('zur', 'veyran')
    from commander_sim.cards.impl import zur as _z, common as _c
    Z, IC = _z, _c


def zur(g, p, sick=False):
    m = E.enter(g, p, p.cmd, sick=sick); m.is_cmd = True; p.cmd_in_zone = False
    return m


def cast(g, p, name, target=None):
    """cast a card from hand (already paid for), with the target chosen as it's cast"""
    c = hand(p, name)
    E.cast_card(g, p, c, 'hand', {'target': target} if target is not None else {})
    return next((m for m in p.perms if m.cd is c), None)


def attackers(g, p):
    return [m for m in p.perms if m.creature and not m.tapped and not E.CI.locked(g, m, 'pacify')]


def seat(g, p, answers):
    g.controllers = {p.key: ScriptController(answers)}
    return g.controllers[p.key]


def by_text(word):
    return lambda req: next(i for i, x in enumerate(req.choices) if word in x)


class Locks(unittest.TestCase):
    def test_arrest_stops_attacks_blocks_and_abilities_until_it_leaves(self):
        g = table('zur', 'veyran', 'sauron'); z, v, s = g.players
        gm = perm(g, v, 'Harmonic Prodigy')
        a = cast(g, z, 'Arrest', gm)
        self.assertIs(a.attached, gm)
        self.assertFalse(combat.can_attack(g, v, gm))
        self.assertFalse(ais.can_block(g, gm, zur(g, z)))
        from commander_sim.cards.impl import rules
        self.assertTrue(rules.ability_locked(g, gm, v))
        E.die(g, a, 'destroy')                                   # the Aura goes: the creature is free again
        gm.sick = False
        self.assertTrue(combat.can_attack(g, v, gm))
        self.assertFalse(E.CI.locked(g, gm, 'pacify'))

    def test_luminous_bonds_leaves_abilities_alone(self):
        g = table('zur', 'veyran', 'sauron'); z, v, s = g.players
        m = perm(g, v, 'Guttersnipe'); m.sick = False
        cast(g, z, 'Luminous Bonds', m)
        self.assertFalse(combat.can_attack(g, v, m))
        self.assertTrue(E.CI.locked(g, m, 'pacify'))
        self.assertFalse(E.CI.locked(g, m, 'noact'))

    def test_encrust_stops_mana_and_untapping(self):
        g = table('zur', 'veyran', 'sauron'); z, v, s = g.players
        rock = perm(g, v, 'Sol Ring')
        before = E.total_mana(g, v)
        cast(g, z, 'Encrust', rock)
        self.assertEqual(E.total_mana(g, v), before - 2)          # its mana ability can't be activated
        cr = perm(g, s, 'Orcish Bowmasters'); cr.tapped = True
        hand(z, 'Encrust')                                       # (a second copy, from the table helper's card)
        E.cast_card(g, z, z.hand[-1], 'hand', {'target': cr})
        ais._step_start(g, s)
        self.assertTrue(cr.tapped)                               # doesn't untap

    def test_kasmina_makes_a_vanilla_1_1_until_it_leaves(self):
        g = table('zur', 'veyran', 'sauron'); z, v, s = g.players
        m = perm(g, v, 'Niv-Mizzet, Parun')
        self.assertTrue(m.fly)
        a = cast(g, z, "Kasmina's Transmutation", m)
        self.assertEqual((E.epow(g, m), E.etgh(g, m)), (1, 1))
        self.assertFalse(m.fly); self.assertTrue(m.neutered)
        E.die(g, a, 'destroy')
        self.assertEqual(E.epow(g, m), 5); self.assertTrue(m.fly); self.assertFalse(m.neutered)

    def test_a_locked_creature_is_worth_less(self):
        g = table('zur', 'veyran', 'sauron'); z, v, s = g.players
        m = perm(g, v, 'Niv-Mizzet, Parun')
        before = E.pval(g, m)
        cast(g, z, 'Arrest', m)
        self.assertLess(E.pval(g, m), before * 0.5)
        self.assertEqual(E.legal_targets(g, z, 'arrest', 'c', spell=card('Prison Sentence')), [])   # not twice

    def test_no_target_no_aura(self):
        g = table('zur', 'veyran', 'sauron'); z, v, s = g.players
        c = hand(z, 'Bound in Silence')
        E.cast_card(g, z, c, 'hand', {})
        self.assertIn(c, z.gy)
        self.assertFalse(any(m.cd is c for m in z.perms))

    def test_phyrexian_boon_kills_a_small_nonblack_creature_or_pumps_zur(self):
        g = table('zur', 'veyran', 'sauron'); z, v, s = g.players
        m = perm(g, v, 'Guttersnipe')                            # 2/2, red
        perm(g, z, 'Phyrexian Boon')
        self.assertNotIn(m, v.perms)
        g2 = table('zur', 'veyran', 'sauron'); z2 = g2.players[0]
        zz = zur(g2, z2)
        perm(g2, z2, 'Phyrexian Boon')
        self.assertEqual((E.epow(g2, zz), E.etgh(g2, zz)), (3, 5))


class ZurTrigger(unittest.TestCase):
    def test_zur_fetches_an_enchantment_onto_the_battlefield(self):
        g = table('zur', 'veyran', 'sauron'); z, v, s = g.players
        zz = zur(g, z)
        n = len(z.library)
        ais.attack_triggers(g, z, [zz], v)
        self.assertEqual(len(z.library), n - 1)
        got = [m for m in z.perms if m.cd is not None and 'E' in m.cd.types]
        self.assertEqual(len(got), 1)
        self.assertLessEqual(got[0].cd.cmc, 3)
        self.assertEqual(z.milestone.get('zurfetch'), z.turns)

    def test_a_fetched_lock_aura_ignores_hexproof(self):
        g = table('zur', 'veyran', 'sauron'); z, v, s = g.players
        zz = zur(g, z)
        big = perm(g, v, 'Niv-Mizzet, Parun')
        g.eot_kw[id(big)] = {'hexproof'}
        self.assertTrue(E.untargetable(g, big))
        self.assertIsNone(Z.lock_host(g, z, 'Arrest', targeted=True))      # a cast Arrest can't target it
        z.library = [c for c in z.library if not ('E' in c.types and c.cmc <= 3) or c.name == 'Arrest']
        ais.attack_triggers(g, z, [zz], v)
        self.assertTrue(E.CI.locked(g, big, 'pacify'))

    def test_power_auras_go_on_zur(self):
        g = table('zur', 'veyran', 'sauron'); z, v, s = g.players
        zz = zur(g, z)
        z.library = [c for c in z.library if not ('E' in c.types and c.cmc <= 3) or c.name == 'Ethereal Armor']
        ais.attack_triggers(g, z, [zz], v)
        self.assertEqual(E.epow(g, zz), 2)                       # 1 + one enchantment (the Armor itself)
        self.assertTrue(ais.first_strike(zz))

    def test_never_more_than_mana_value_three(self):
        g = table('zur', 'veyran', 'sauron'); z, v, s = g.players
        zz = zur(g, z)
        z.library = [c for c in z.library if not ('E' in c.types and c.cmc <= 3)]
        ais.attack_triggers(g, z, [zz], v)
        self.assertFalse(any(m.cd is not None and 'E' in m.cd.types for m in z.perms))


class Cards(unittest.TestCase):
    def test_gift_of_immortality_brings_zur_back_and_returns_itself(self):
        g = table('zur', 'veyran', 'sauron'); z, v, s = g.players
        zz = zur(g, z)
        g.attach_to = zz
        perm(g, z, 'Gift of Immortality')
        g.attach_to = None
        E.die(g, zz, 'destroy')
        back = next(m for m in z.perms if m.cd is z.cmd)
        self.assertFalse(z.cmd_in_zone)
        self.assertIn(card('Gift of Immortality'), z.gy)
        ais.end_step(g, v)
        self.assertTrue(any(a.cd.name == 'Gift of Immortality' for a in IC.auras_on(g, back)))

    def test_duelists_heritage_gives_zur_double_strike(self):
        g = table('zur', 'veyran', 'sauron'); z, v, s = g.players
        zz = zur(g, z)
        perm(g, z, "Duelist's Heritage")
        z.library = [c for c in z.library if not ('E' in c.types and c.cmc <= 3)]
        ais.attack_triggers(g, z, [zz], v)
        self.assertTrue(ais.double_strike(z, zz))

    def test_bastion_protector_guards_only_the_commander(self):
        g = table('zur', 'veyran', 'sauron'); z, v, s = g.players
        zz = zur(g, z); bp = perm(g, z, 'Bastion Protector')
        self.assertEqual((E.epow(g, zz), E.etgh(g, zz)), (3, 6))
        self.assertTrue(E.indestructible(g, zz))
        self.assertFalse(E.indestructible(g, bp))

    def test_ministrant_leaves_two_spirits(self):
        g = table('zur', 'veyran', 'sauron'); z, v, s = g.players
        m = perm(g, z, 'Ministrant of Obligation')
        E.die(g, m, 'destroy')
        sp = [x for x in z.perms if x.token]
        self.assertEqual(len(sp), 2)
        self.assertTrue(all(x.fly for x in sp))

    def test_recruitment_officer_digs_for_a_small_creature(self):
        g = table('zur', 'veyran', 'sauron'); z, v, s = g.players
        z.library.append(card('Esper Sentinel'))
        Z.officer_dig(g, z)
        self.assertIn('Esper Sentinel', [c.name for c in z.hand])

    def test_the_eternal_wanderer(self):
        g = table('zur', 'veyran', 'sauron'); z, v, s = g.players
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
        zz = zur(g, z); perm(g, z, 'Esper Sentinel'); perm(g, v, 'Guttersnipe')
        Z.wanderer_ult(g, z)
        self.assertEqual([m.cd.name for m in z.perms if m.creature], ['Zur the Enchanter'])
        self.assertEqual(sum(1 for m in v.perms if m.creature), 1)

    def test_prayer_of_binding_until_it_leaves(self):
        g = table('zur', 'veyran', 'sauron'); z, v, s = g.players
        big = perm(g, v, 'Niv-Mizzet, Parun')
        life = z.life
        pr = perm(g, z, 'Prayer of Binding')
        self.assertNotIn(big, v.perms); self.assertEqual(z.life, life + 2)
        E.die(g, pr, 'destroy')
        self.assertTrue(any(m.cd is not None and m.cd.name == 'Niv-Mizzet, Parun' for m in v.perms))

    def test_destroy_evil_and_disenchant_targets(self):
        g = table('zur', 'veyran', 'sauron'); z, v, s = g.players
        small = perm(g, v, 'Guttersnipe'); big = perm(g, v, 'Niv-Mizzet, Parun'); rock = perm(g, v, 'Sol Ring')
        evil = E.legal_targets(g, z, 'destroy', 'ce', spell=card('Destroy Evil'))
        self.assertIn(big, evil); self.assertNotIn(small, evil)
        dis = E.legal_targets(g, z, 'destroy', 'ae', spell=card('Disenchant'))
        self.assertIn(rock, dis); self.assertNotIn(big, dis)

    def test_divine_verdict_on_an_attacker(self):
        g = table('veyran', 'zur', 'sauron'); v, z, s = g.players
        lands(z, 'Plains', 4)
        hand(z, 'Divine Verdict')
        big = perm(g, v, 'Niv-Mizzet, Parun'); big.sick = False
        g.in_combat = {big}
        Z.verdict_response(g, z, v, [big])
        self.assertNotIn(big, v.perms)

    def test_rootborn_defenses_against_a_wipe(self):
        g = table('zur', 'veyran', 'sauron'); z, v, s = g.players
        lands(z, 'Plains', 3)
        zz = zur(g, z); perm(g, z, 'Restoration Angel'); hand(z, 'Rootborn Defenses')
        self.assertEqual(ais.wipe_response(g, z, 'destroy', v), 'indes')
        self.assertTrue(E.indestructible(g, zz))

    def test_demonic_embrace_from_the_graveyard(self):
        g = table('zur', 'veyran', 'sauron'); z, v, s = g.players
        lands(z, 'Swamp', 3)
        zz = zur(g, z); z.gy.append(card('Demonic Embrace')); hand(z, 'Island')
        opts = Z._embrace_gy(g, card('Demonic Embrace'), z, None, False)
        self.assertTrue(opts)
        self.assertTrue(opts[0][2]())
        self.assertEqual(E.epow(g, zz), 4)
        self.assertEqual(z.life, 37)

    def test_vivid_meadow(self):
        g = table('zur', 'veyran', 'sauron'); z = g.players[0]
        lands(z, 'Vivid Meadow', 1)
        L = z.lands[-1]
        self.assertEqual(E.CI.LAND_COLS['Vivid Meadow'](g, z, L), z.ident)


class Practice(unittest.TestCase):
    def test_you_cast_arrest_and_choose_its_target_once(self):
        g = table('zur', 'veyran', 'sauron'); z, v, s = g.players
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
        g = table('zur', 'veyran', 'sauron'); z, v, s = g.players
        zz = zur(g, z)
        m = perm(g, v, 'Guttersnipe')
        seat(g, z, [by_text('Luminous Bonds'), by_text('Guttersnipe')])
        ais.attack_triggers(g, z, [zz], v)
        self.assertTrue(E.CI.locked(g, m, 'pacify'))

    def test_clever_concealment_with_convoke(self):
        g = table('zur', 'veyran', 'sauron'); z, v, s = g.players
        zz = zur(g, z); perm(g, z, 'Esper Sentinel'); perm(g, z, 'Restoration Angel')
        from commander_sim.play import mana, cards
        mana.pool_of(z).add('W', 1); mana.pool_of(z).add('C', 1)
        c = hand(z, 'Clever Concealment')
        seat(g, z, [by_text('Zur'), 'cancel', by_text('Esper Sentinel'), by_text('Restoration Angel'), 'cancel'])
        self.assertIsNone(cards.cast_from_hand(g, z, c))
        self.assertTrue(zz.phased)
        self.assertIn(c, z.gy)

    def test_momentary_blink_flashback(self):
        g = table('zur', 'veyran', 'sauron'); z, v, s = g.players
        perm(g, z, 'Esper Sentinel'); perm(g, v, 'Guttersnipe'); perm(g, v, 'Harmonic Prodigy')
        perm(g, z, 'Accursed Marauder')                          # each player sacrifices one: Sentinel, and one of theirs
        z.gy.append(card('Momentary Blink'))
        from commander_sim.play import mana
        mana.pool_of(z).add('U', 1); mana.pool_of(z).add('C', 3)
        seat(g, z, [by_text('Accursed Marauder'), 0])            # blink it; then its edict: you sacrifice it
        why = human.apply(g, z, {'do': 'cast', 'zone': 'gy', 'card': len(z.gy) - 1})
        self.assertIsNone(why)
        self.assertIn(card('Momentary Blink'), z.exile)
        self.assertFalse(any(m.creature and not m.token for m in v.perms))      # the Marauder's edict again

    def test_the_wanderer_for_the_person(self):
        g = table('zur', 'veyran', 'sauron'); z, v, s = g.players
        w = perm(g, z, 'The Eternal Wanderer')
        seat(g, z, [by_text('0:')])
        self.assertIsNone(human.use(g, z, w))
        sam = [m for m in z.perms if m.token]
        self.assertEqual(len(sam), 1)
        self.assertTrue(ais.double_strike(z, sam[0]))


if __name__ == '__main__':
    unittest.main()
