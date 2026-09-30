"""Practice mode: targets and activated abilities for the human seat. Targets follow the rules (your own
permanents, players for burn; hexproof and protection respected), are chosen before paying, and can be cancelled.
Equip and card abilities (planeswalkers) work from the permanent."""
import unittest
from tests.table import table, hand, lands, perm, card
from commander_sim import engine as E
from commander_sim.play import legal, mana, human
from commander_sim.play.controller import ScriptController


class Pick:
    """an answer that picks the choice describing x (targets are listed by description, so tests pick by object)"""
    def __init__(s, x): s.x = x


def seat(g, p, answers):
    ctl = ScriptController(answers)
    plain = ctl.ask

    def ask(req):
        ans = plain(req)
        return req.choices.index(legal.describe_target(g, p, ans.x)) if isinstance(ans, Pick) else ans
    ctl.ask = ask
    g.controllers = {p.key: ctl}
    return ctl


def tap_all(g, p):
    while mana.sources(g, p): mana.tap(g, p, mana.sources(g, p)[0]['id'])


class Targets(unittest.TestCase):
    def test_what_a_removal_spell_can_target(self):
        g = table('sauron', 'veyran'); s, v = g.players
        c = hand(s, 'Terminate')
        mine = perm(g, s, 'Orcish Bowmasters'); theirs = perm(g, v, 'Guttersnipe')
        cloaked = perm(g, v, 'Young Pyromancer'); cloak = perm(g, v, 'Whispersilk Cloak'); cloak.attached = cloaked
        tg = legal.spell_targets(g, s, c)
        self.assertIn(mine, tg); self.assertIn(theirs, tg); self.assertNotIn(cloaked, tg)
        self.assertNotIn(v, tg)                                  # Terminate can't target players

    def test_burn_can_target_players(self):
        g = table('veyran', 'sauron'); v, s = g.players
        tg = legal.spell_targets(g, v, card('Lightning Bolt'))
        self.assertIn(s, tg); self.assertIn(v, tg)
        s.life_locked = True                                     # Teferi's Protection
        self.assertNotIn(s, legal.spell_targets(g, v, card('Lightning Bolt')))

    def test_cast_with_a_target(self):
        g = table('sauron', 'veyran'); s, v = g.players
        hand(s, 'Terminate'); lands(s, 'Swamp', 1); lands(s, 'Mountain', 1); tap_all(g, s)
        gs = perm(g, v, 'Guttersnipe')
        ctl = seat(g, s, [{'do': 'cast', 'card': 0}, Pick(gs), {'do': 'pass'}])
        human.human_main(g, s, False)
        self.assertNotIn(gs, v.perms)
        self.assertEqual(ctl.asked[1].kind, 'target')

    def test_cancelling_a_target_costs_nothing(self):
        g = table('sauron', 'veyran'); s, v = g.players
        hand(s, 'Terminate'); lands(s, 'Swamp', 1); lands(s, 'Mountain', 1); tap_all(g, s)
        perm(g, v, 'Guttersnipe')
        seat(g, s, [{'do': 'cast', 'card': 0}, 'cancel', {'do': 'pass'}])
        human.human_main(g, s, False)
        self.assertIn(E.DB['Terminate'], s.hand)
        self.assertEqual(mana.pool_of(s).total(), 2)

    def test_bloodchiefs_thirst_kicker(self):
        g = table('sauron', 'veyran'); s, v = g.players
        c = hand(s, "Bloodchief's Thirst"); lands(s, 'Swamp', 1); tap_all(g, s)
        self.assertIsNone(legal.check_cast(g, s, c))              # {B} unkicked
        big = perm(g, v, 'Sheoldred, the Apocalypse')
        ctl = seat(g, s, [{'do': 'cast', 'card': 0}, Pick(big), {'do': 'pass'}])
        human.human_main(g, s, False)
        self.assertIn(big, v.perms)                              # mana value 4 needs the kicker: {2}{B} more
        self.assertIn('Missing {2}{B}', ctl.told[0][1])


class Abilities(unittest.TestCase):
    def test_equip(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        E.amass(g, s, 3); army = E.army_of(s)
        sword = perm(g, s, 'Sword of Feast and Famine'); lands(s, 'Swamp', 2); tap_all(g, s)
        i = s.perms.index(sword)
        seat(g, s, [{'do': 'use', 'perm': i}, 0, Pick(army), {'do': 'pass'}])
        human.human_main(g, s, False)
        self.assertIs(sword.attached, army)
        self.assertEqual(mana.pool_of(s).total(), 0)

    def test_equip_needs_mana_and_your_main_phase(self):
        g = table('sauron', 'veyran'); s, v = g.players
        E.amass(g, s, 3); army = E.army_of(s)
        sword = perm(g, s, 'Sword of Feast and Famine')
        self.assertIn('Missing {2}', legal.check_equip(g, s, sword, army))
        g.active = v
        self.assertEqual(legal.check_equip(g, s, sword, army), "It isn't your turn.")

    def test_a_planeswalker_ability(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        vr = perm(g, s, "Vraska, Betrayal's Sting")
        abil = human.abilities_of(g, s, vr)
        self.assertTrue(abil, 'Vraska offers loyalty abilities')
        start = vr.loyalty
        seat(g, s, [{'do': 'use', 'perm': s.perms.index(vr)}, 0, {'do': 'pass'}])
        human.human_main(g, s, False)
        self.assertNotEqual(vr.loyalty, start)

    def test_nothing_to_activate(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        perm(g, s, 'Orcish Bowmasters')
        ctl = seat(g, s, [{'do': 'use', 'perm': 0}, {'do': 'pass'}])
        human.human_main(g, s, False)
        self.assertIn('no ability', ctl.told[0][1])


if __name__ == '__main__':
    unittest.main()
