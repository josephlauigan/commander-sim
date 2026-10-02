"""Practice mode, Veyran's spells: the choices the card code makes for the AI are yours as they resolve (Crackle
with Power's X and targets, Mizzix's Mastery, Flashback, Jeska's Will, Expressive Iteration, Flow State, Stock Up,
Prismari Charm) and copies of your spells take new targets; Alania, Divergent Storm's copies (the AI's and yours)."""
import unittest
from tests.table import table, hand, perm
from commander_sim import engine as E
from commander_sim.play import mana, human
from commander_sim.play.controller import ScriptController


def seat(g, p, answers):
    g.controllers = {p.key: ScriptController(answers)}
    return g.controllers[p.key]


def by_text(word):
    return lambda req: next(i for i, x in enumerate(req.choices) if word in x)


def pool(p, **kw):
    for c, n in kw.items(): mana.pool_of(p).add(c, n)


def main(g, p, *answers):
    ctl = seat(g, p, list(answers) + [{'do': 'pass'}])
    human.human_main(g, p, False)
    return ctl


class Spells(unittest.TestCase):
    def test_crackle_with_power_x_two(self):
        g = table('veyran', 'seph', 'sauron'); v, s, o = g.players
        hand(v, 'Crackle with Power'); pool(v, R=2, C=6)
        main(g, v, {'do': 'cast', 'card': 0}, by_text('X = 2'), by_text('Sephiroth'), by_text('Sauron'))
        self.assertEqual((s.life, o.life), (30, 30))

    def test_crackle_one_target_is_enough(self):
        g = table('veyran', 'seph', 'sauron'); v, s, o = g.players
        hand(v, 'Crackle with Power'); pool(v, R=2, C=6)
        main(g, v, {'do': 'cast', 'card': 0}, by_text('X = 2'), by_text('Sephiroth'), by_text('no more'))
        self.assertEqual((s.life, o.life), (30, 40))

    def test_mizzixs_mastery_your_target(self):
        g = table('veyran', 'seph'); v = g.players[0]
        hand(v, "Mizzix's Mastery"); v.gy += [E.DB['Quick Study'], E.DB['Lightning Bolt']]; pool(v, R=1, C=3)
        n = len(v.hand)
        main(g, v, {'do': 'cast', 'card': 0}, by_text('one target'), by_text('Quick Study'))
        self.assertEqual(len(v.hand), n - 1 + 2)
        self.assertIn(E.DB['Quick Study'], v.exile); self.assertIn(E.DB['Lightning Bolt'], v.gy)

    def test_flashback_grants_flashback(self):
        g = table('veyran', 'seph'); v = g.players[0]
        hand(v, 'Flashback'); v.gy.append(E.DB['Quick Study']); pool(v, R=1, U=1, C=2)
        n = len(v.hand)
        main(g, v, {'do': 'cast', 'card': 0}, 0, {'do': 'cast', 'zone': 'gy', 'card': 0})
        self.assertEqual(len(v.hand), n - 1 + 2)
        self.assertIn(E.DB['Quick Study'], v.exile)

    def test_jeskas_will_mana(self):
        g = table('veyran', 'seph'); v, s = g.players
        hand(v, "Jeska's Will"); hand(s, 'Ephemerate', 'Necromancy', 'Entomb'); pool(v, R=1, C=2)
        main(g, v, {'do': 'cast', 'card': 0}, by_text('add {R}'), 0)
        self.assertEqual(mana.pool_of(v).m['R'], 3)

    def test_jeskas_will_exile_does_not_play_a_land_for_you(self):
        g = table('veyran', 'seph'); v = g.players[0]
        hand(v, "Jeska's Will"); pool(v, R=1, C=2)
        v.library += [E.DB['Island']] * 3
        main(g, v, {'do': 'cast', 'card': 0}, by_text('exile'))
        self.assertEqual(len(v.lands), 0); self.assertEqual(sum(1 for c in v.hand if c.name == 'Island'), 3)

    def test_expressive_iteration(self):
        g = table('veyran', 'seph'); v = g.players[0]
        hand(v, 'Expressive Iteration'); pool(v, U=1, R=1)
        top3 = v.library[-3:]
        main(g, v, {'do': 'cast', 'card': 0}, 0, 0)
        self.assertIn(top3[-1], v.hand); self.assertIn(top3[-2], v.impulse); self.assertIs(v.library[0], top3[0])

    def test_stock_up(self):
        g = table('veyran', 'seph'); v = g.players[0]
        hand(v, 'Stock Up'); pool(v, U=1, C=2); n = len(v.hand)
        main(g, v, {'do': 'cast', 'card': 0}, 0, 0)
        self.assertEqual(len(v.hand), n - 1 + 2)

    def test_prismari_charm_two_pings(self):
        g = table('veyran', 'seph', 'sauron'); v, s, o = g.players
        hand(v, 'Prismari Charm'); pool(v, U=1, R=1)
        main(g, v, {'do': 'cast', 'card': 0}, by_text('1 damage'), by_text('Sephiroth'), by_text('Sauron'))
        self.assertEqual((s.life, o.life), (39, 39))

    def test_thousand_year_storm_copies_take_new_targets(self):
        g = table('veyran', 'seph', 'sauron'); v, s, o = g.players
        perm(g, v, 'Thousand-Year Storm'); hand(v, 'Quick Study', 'Lightning Bolt'); pool(v, U=1, C=2, R=1)
        main(g, v, {'do': 'cast', 'card': 0}, {'do': 'cast', 'card': 0}, by_text('Sephiroth'), by_text('Sauron'))
        self.assertEqual((s.life, o.life), (37, 37))



class Alania(unittest.TestCase):
    """Alania, Divergent Storm: your first instant and first sorcery each turn, for an opponent's card per copy"""
    def bolt(self, g, v, target):
        E.cast_card(g, v, hand(v, 'Lightning Bolt'), 'hand', {'target': None, 'face': target})

    def test_the_first_instant_is_copied_once(self):
        g = table('veyran', 'seph', 'sauron'); v, s, o = g.players
        perm(g, v, 'Alania, Divergent Storm')
        before = len(s.hand) + len(o.hand)
        self.bolt(g, v, s)
        self.assertEqual(len(s.hand) + len(o.hand), before + 1)           # one opponent drew for the copy
        self.assertEqual(s.life + o.life, 80 - 6)                          # the Bolt and its copy
        E.cast_card(g, v, hand(v, 'Burst Lightning'), 'hand', {'face': s})   # not the first instant: no copy
        self.assertEqual(len(s.hand) + len(o.hand), before + 1)

    def test_veyran_and_harmonic_prodigy_each_add_a_copy(self):
        g = table('veyran', 'seph', 'sauron'); v, s, o = g.players
        for n in ('Alania, Divergent Storm', 'Veyran, Voice of Duality', 'Harmonic Prodigy'): perm(g, v, n)
        before = len(s.hand) + len(o.hand)
        self.bolt(g, v, s)
        self.assertEqual(len(s.hand) + len(o.hand), before + 3)

    def test_the_first_sorcery_too_and_not_a_counterspell(self):
        g = table('veyran', 'seph', 'sauron'); v, s, o = g.players
        perm(g, v, 'Alania, Divergent Storm')
        before = len(s.hand) + len(o.hand)
        E.cast_card(g, v, hand(v, 'Stock Up'), 'hand')
        self.assertEqual(len(s.hand) + len(o.hand), before + 1)
        c = hand(v, 'Counterspell')
        g.log = []
        E.on_cast(g, v, c)                                                 # the first instant: a counterspell
        self.assertFalse(any('Alania' in x for x in g.log))

    def test_a_spell_before_alania_counts(self):
        g = table('veyran', 'seph', 'sauron'); v, s, o = g.players
        self.bolt(g, v, s)
        perm(g, v, 'Alania, Divergent Storm')
        before = len(s.hand) + len(o.hand)
        E.cast_card(g, v, hand(v, 'Burst Lightning'), 'hand', {'face': s})
        self.assertEqual(len(s.hand) + len(o.hand), before)

    def test_you_choose_who_draws_or_no_copy(self):
        g = table('veyran', 'seph', 'sauron'); v, s, o = g.players
        for n in ('Alania, Divergent Storm', 'Veyran, Voice of Duality'): perm(g, v, n)
        hand(v, 'Lightning Bolt'); pool(v, R=1)
        ns, no = len(s.hand), len(o.hand)
        main(g, v, {'do': 'cast', 'card': 0}, by_text('Sephiroth'),       # the Bolt's target
             by_text('Sauron'), by_text('Sephiroth'),                     # first trigger: Sauron draws; the copy hits Sephiroth
             'cancel')                                                    # second trigger: no copy
        self.assertEqual((len(s.hand), len(o.hand)), (ns, no + 1))
        self.assertEqual(s.life, 40 - 6)


if __name__ == '__main__':
    unittest.main()
