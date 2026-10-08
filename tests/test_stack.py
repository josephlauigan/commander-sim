"""The stack (engine.stack_window): every player gets priority in turn order on each spell, responses go on top and
resolve first, a counterspell can be countered in turn, and the stack is plain data the look-ahead can copy."""
import copy
import unittest
from tests.table import table, hand, lands, card
from commander_sim import engine as E
from commander_sim.play.controller import ScriptController


def seat(g, p, answers):
    g.controllers = {p.key: ScriptController(answers)}
    return g.controllers[p.key]


class Stack(unittest.TestCase):
    def test_a_counter_war_between_three_players(self):
        g = table('sauron', 'veyran', 'seph'); s, v, ph = g.players
        sheo = card('Sheoldred, the Apocalypse')
        hand(v, 'Counterspell'); lands(v, 'Island', 2)
        hand(s, 'Counterspell'); lands(s, 'Island', 2)
        item = E.StackItem(s, sheo, {}, 'hand', 7, {})
        g.stack.append(item)
        self.assertTrue(E.cast_counter_spell(g, v, v.hand[-1], item))     # Veyran counters Sheoldred ...
        self.assertFalse(item.countered)                                   # ... Sauron counters that back
        self.assertEqual(s.stats['counterwar_won'], 1)
        self.assertEqual(sum(1 for c in s.gy + v.gy if c.name == 'Counterspell'), 2)

    def test_a_countered_counterspell_does_nothing(self):
        g = table('sauron', 'veyran'); s, v = g.players
        item = E.StackItem(s, card('Sheoldred, the Apocalypse'), {}, 'hand', 7, {})
        g.stack.append(item)
        ctr_item = E.StackItem(v, card('Counterspell'), {'counter': item}, 'hand', 7, {}, generic=False)
        g.stack.append(ctr_item)
        E.resolve_counter(g, s, card('Counterspell'), ctr_item)           # Sauron's counter resolves first
        self.assertTrue(ctr_item.countered); self.assertFalse(item.countered)

    def test_responses_resolve_first_and_priority_goes_round(self):
        g = table('veyran', 'sauron', 'seph'); v, s, ph = g.players
        lands(v, 'Mountain', 3)
        ctl = seat(g, ph, [{'do': 'pass'}])
        c = hand(v, 'Guttersnipe')
        self.assertTrue(E.cast_card(g, v, c, 'hand', {}))
        self.assertEqual(len(ctl.asked), 1)                                 # the person got priority once
        self.assertEqual(g.stack, [])

    def test_the_person_sees_the_whole_stack(self):
        g = table('veyran', 'sauron', 'seph'); v, s, ph = g.players
        ctl = seat(g, ph, [{'do': 'pass'}])
        a = E.StackItem(v, card('Guttersnipe'), {}, 'hand', 5, {})
        g.stack.append(a)
        b = E.StackItem(s, card('Counterspell'), {'counter': a}, 'hand', 6, {}, generic=False)
        E.stack_window(g, s, b)
        d = ctl.asked[0].data['stack']
        self.assertEqual([x['name'] for x in d], ['Counterspell', 'Guttersnipe'])           # top first
        self.assertEqual(d[0]['target'], 'Guttersnipe')

    def test_the_stack_copies_with_the_game(self):
        g = table('veyran', 'sauron'); v, s = g.players
        it = E.StackItem(v, card('Guttersnipe'), {'target': None}, 'hand', 5, {})
        g.stack.append(it)
        from commander_sim.ai import search
        g2 = search.clone(g)
        self.assertEqual(len(g2.stack), 1)
        self.assertIs(g2.stack[0].controller, g2.players[0])
        E.settle_stack(g2)
        self.assertEqual(g2.stack, [])
        self.assertTrue(any(m.name == 'Guttersnipe' for m in g2.players[0].perms))
        self.assertEqual(len(g.stack), 1)                                   # the original is untouched


if __name__ == '__main__':
    unittest.main()


class Abilities(unittest.TestCase):
    """activated abilities go on the stack once their cost is paid; mana abilities don't"""
    def test_the_ai_counters_an_important_ability_with_azorius_guildmage(self):
        from tests.table import perm
        g = table('veyran', 'yshtola'); v, z = g.players
        perm(g, z, 'Azorius Guildmage'); lands(z, 'Island', 3)
        src = perm(g, v, 'Triskelion')
        self.assertFalse(E.ability_window(g, v, src, '1 damage', imp=8))         # countered
        self.assertTrue(E.ability_window(g, v, src, '1 damage', imp=2))          # not worth the mana

    def test_no_window_when_nobody_could_answer(self):
        from tests.table import perm
        g = table('veyran', 'sauron'); v, s = g.players
        src = perm(g, v, 'Triskelion')
        n = g.stack_pushes
        self.assertTrue(E.ability_window(g, v, src, '1 damage', imp=9))
        self.assertEqual(g.stack_pushes, n)

    def test_a_countered_equip_does_not_attach(self):
        from tests.table import perm
        g = table('sauron', 'yshtola'); s, z = g.players
        perm(g, z, 'Azorius Guildmage'); lands(z, 'Island', 3)
        a = perm(g, s, 'Orcish Bowmasters'); e = perm(g, s, 'Lightning Greaves')
        lands(s, 'Swamp', 2)
        E.equip_to(g, s, e, a, 0)                       # imp 3 + Greaves' value: below the AI's bar, so it resolves
        self.assertIs(e.attached, a)

    def test_you_counter_an_ability_with_azorius_guildmage(self):
        from tests.table import perm
        from commander_sim.play import mana
        g = table('veyran', 'yshtola'); v, z = g.players
        gm = perm(g, z, 'Azorius Guildmage')
        mana.pool_of(z).add('U', 1); mana.pool_of(z).add('C', 2)
        src = perm(g, v, 'Triskelion')
        idx = z.perms.index(gm)
        seat(g, z, [{'do': 'use', 'perm': idx}, lambda req: next(i for i, x in enumerate(req.choices) if 'counter' in x)])
        self.assertFalse(E.ability_window(g, v, src, '1 damage', imp=8))

    def test_an_ability_resolves_even_if_its_source_dies_in_response(self):
        from tests.table import perm
        g = table('veyran', 'sauron', 'seph'); v, s, ph = g.players
        b = perm(g, v, 'Triskelion'); b.plus = 3
        hand(ph, 'Lightning Bolt'); lands(ph, 'Mountain', 1)
        bolt = len(ph.hand) - 1
        from commander_sim.play import mana
        mana.pool_of(ph).add('R', 1)
        seat(g, ph, [{'do': 'cast', 'card': bolt}, lambda req: next(i for i, x in enumerate(req.choices) if 'Triskelion' in x), {'do': 'pass'}])
        life = s.life
        b.plus -= 1                                      # the cost: a counter removed
        resolves = E.ability_window(g, v, b, '1 damage', imp=8)
        self.assertTrue(resolves)                        # the Bolt killed Triskelion, the ping still happens
        self.assertNotIn(b, v.perms)
