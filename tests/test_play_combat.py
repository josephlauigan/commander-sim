"""Practice mode: combat declarations for the human seat. Attackers are any creatures that can legally attack;
you pick whom to attack; when attacked, you pick blockers. The engine resolves the damage."""
import unittest
from tests.table import table, perm
from commander_sim import engine as E, ais
from commander_sim.play import combat
from commander_sim.play.controller import ScriptController


def seat(g, p, answers):
    g.controllers = {p.key: ScriptController(answers)}
    return g.controllers[p.key]


class Attacking(unittest.TestCase):
    def test_who_can_attack(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        arch = perm(g, s, "Jace's Archivist"); arch.sick = False          # the AI never attacks with it; you may
        new = perm(g, s, 'Orcish Bowmasters', sick=True)
        tapped = perm(g, s, 'Grave Titan'); tapped.sick = False; tapped.tapped = True
        c = combat.attack_candidates(g, s)
        self.assertIn(arch, c); self.assertNotIn(new, c); self.assertNotIn(tapped, c)

    def test_attack_the_only_opponent(self):
        g = table('sauron', 'veyran'); s, v = g.players
        t = perm(g, s, 'Grave Titan'); t.sick = False
        seat(g, s, [[combat.attack_candidates(g, s).index(t)]])
        ais.combat(g, s)
        self.assertEqual(v.life, 40 - 6); self.assertTrue(t.tapped)

    def test_choose_the_defending_player(self):
        g = table('sauron', 'veyran', 'seph'); s, v, x = g.players
        t = perm(g, s, 'Grave Titan'); t.sick = False
        ctl = seat(g, s, [[0], 1])
        ais.combat(g, s)
        self.assertEqual((v.life, x.life), (40, 34))
        self.assertEqual([r.kind for r in ctl.asked], ['attack', 'target'])

    def test_split_attackers_between_players(self):
        g = table('sauron', 'veyran', 'seph'); s, v, x = g.players
        t = perm(g, s, 'Grave Titan'); t.sick = False
        b = perm(g, s, 'Orcish Bowmasters'); b.sick = False
        c = combat.attack_candidates(g, s)
        ctl = seat(g, s, [[c.index(t), c.index(b)], 2, 0, 1])   # split: Titan at Veyran, Bowmasters at Sephiroth
        lv, lx = v.life, x.life                                  # (Bowmasters' enter damage already dealt)
        ais.combat(g, s)
        self.assertEqual((v.life, x.life), (lv - 6, lx - 1))
        self.assertTrue(t.tapped and b.tapped)
        self.assertIn(combat.SPLIT, ctl.asked[1].choices)

    def test_split_to_one_player_is_one_attack(self):
        g = table('sauron', 'veyran', 'seph'); s, v, x = g.players
        t = perm(g, s, 'Grave Titan'); t.sick = False
        b = perm(g, s, 'Orcish Bowmasters'); b.sick = False
        c = combat.attack_candidates(g, s)
        seat(g, s, [[c.index(t), c.index(b)], 2, 1, 1])
        self.assertEqual(combat.human_attack(g, s), [(x, [t, b])])

    def test_a_single_attacker_isnt_offered_a_split(self):
        g = table('sauron', 'veyran', 'seph'); s, v, x = g.players
        t = perm(g, s, 'Grave Titan'); t.sick = False
        ctl = seat(g, s, [[0], 0])
        combat.human_attack(g, s)
        self.assertNotIn(combat.SPLIT, ctl.asked[1].choices)

    def test_no_attack(self):
        g = table('sauron', 'veyran'); s, v = g.players
        t = perm(g, s, 'Grave Titan'); t.sick = False
        seat(g, s, [[]])
        ais.combat(g, s)
        self.assertEqual(v.life, 40); self.assertFalse(t.tapped)


class AISplits(unittest.TestCase):
    """the AI's attackers may go at different players (ais.split_attack)"""
    def test_an_attacker_avoids_a_bad_block(self):
        g = table('sauron', 'veyran', 'seph'); s, v, x = g.players
        a = perm(g, s, 'Orcish Bowmasters'); a.sick = False      # 1/1
        perm(g, v, 'Grave Titan')                                # Veyran's 6/6 would eat it
        self.assertEqual(ais.split_attack(g, s, v, [a]), [(x, [a])])

    def test_one_blocker_stops_only_one_attacker(self):
        g = table('sauron', 'veyran', 'seph'); s, v, x = g.players
        a = perm(g, s, 'Orcish Bowmasters'); b = perm(g, s, 'Orcish Bowmasters'); a.sick = b.sick = False
        perm(g, v, 'Niv-Mizzet, Parun')                          # one blocker that eats a 1/1
        groups = dict((q.key, xs) for q, xs in ais.split_attack(g, s, v, [a, b]))
        self.assertEqual((len(groups['veyran']), len(groups['seph'])), (1, 1))

    def test_finish_off_a_player_who_cant_block(self):
        g = table('sauron', 'veyran', 'seph'); s, v, x = g.players
        t = perm(g, s, 'Grave Titan'); t.sick = False
        perm(g, v, 'Grave Titan')                                # Veyran can trade with it; Sephiroth has nothing
        x.life = 5
        self.assertEqual(ais.split_attack(g, s, v, [t]), [(x, [t])])

    def test_nothing_moves_without_a_reason(self):
        g = table('sauron', 'veyran', 'seph'); s, v, x = g.players
        t = perm(g, s, 'Grave Titan'); t.sick = False
        self.assertEqual(ais.split_attack(g, s, v, [t]), [(v, [t])])

    def test_the_ai_attacks_two_players_in_one_combat(self):
        g = table('sauron', 'veyran', 'seph'); s, v, x = g.players
        t = perm(g, s, 'Grave Titan'); t.sick = False
        b = perm(g, s, 'Orcish Bowmasters'); b.sick = False
        perm(g, x, 'Grave Titan', sick=False)                    # Sephiroth can eat the Bowmasters
        lv, lx = v.life, x.life
        g.forced_attack = (g.players.index(x), 'filtered')       # the plan names Sephiroth
        ais.combat(g, s)
        self.assertLess(v.life, lv)                              # the Bowmasters went at Veyran instead
        self.assertTrue(t.tapped and b.tapped)


class Blocking(unittest.TestCase):
    def test_block(self):
        g = table('sauron', 'veyran'); s, v = g.players
        wall = perm(g, s, 'Grave Titan'); wall.sick = False
        a = perm(g, v, 'Guttersnipe'); a.sick = False
        ctl = seat(g, s, [{'do': 'pass'}, 0])
        g.active = v
        ais.resolve_combat(g, v, [a], s, set())
        self.assertEqual(s.life, 40); self.assertNotIn(a, v.perms)
        self.assertEqual([r.kind for r in ctl.asked], ['priority', 'block'])

    def test_no_block(self):
        g = table('sauron', 'veyran'); s, v = g.players
        perm(g, s, 'Grave Titan').sick = False
        a = perm(g, v, 'Guttersnipe'); a.sick = False
        seat(g, s, [{'do': 'pass'}, 'cancel'])
        g.active = v
        ais.resolve_combat(g, v, [a], s, set())
        self.assertEqual(s.life, 38)

    def test_unblockable_attackers_are_not_asked_about(self):
        g = table('sauron', 'veyran'); s, v = g.players
        perm(g, s, 'Grave Titan').sick = False
        a = perm(g, v, 'Guttersnipe'); a.sick = False
        ctl = seat(g, s, [{'do': 'pass'}])
        g.active = v
        ais.resolve_combat(g, v, [a], s, {a})
        self.assertEqual([r.kind for r in ctl.asked], ['priority']); self.assertEqual(s.life, 38)



class ForTheBrowser(unittest.TestCase):
    """with a person at the browser, combat requests say where the creatures are, so the table can show them"""
    def run_with(self, g, p, fn, answers):
        import threading
        from commander_sim.play.controller import HumanController
        got = []
        ctl = HumanController(notify=got.append); g.controllers = {p.key: ctl}

        def feed():
            for a in answers:
                while len([e for e in got if e['kind'] == 'request']) <= answers.index(a): threading.Event().wait(0.02)
                ctl.answer(a)
        threading.Thread(target=feed, daemon=True).start()
        out = fn()
        return out, [e['request'] for e in got if e['kind'] == 'request']

    def test_attackers_point_at_your_creatures(self):
        g = table('sauron', 'veyran'); s = g.players[0]
        t = perm(g, s, 'Grave Titan'); t.sick = False
        out, reqs = self.run_with(g, s, lambda: combat.human_attack(g, s), [[0]])
        self.assertEqual(reqs[0].data['refs'], [{'seat': 'sauron', 'perm': s.perms.index(t)}])
        self.assertEqual(out, [(g.players[1], [t])])

    def test_blocks_show_the_attackers(self):
        g = table('veyran', 'sauron'); v, s = g.players
        a = perm(g, v, 'Guttersnipe'); b = perm(g, s, 'Orcish Bowmasters')
        out, reqs = self.run_with(g, s, lambda: combat.human_blocks(g, v, [a], s, set()), ['cancel'])
        d = reqs[0].data
        self.assertEqual(d['attacker'], {'seat': 'veyran', 'perm': v.perms.index(a)})
        self.assertEqual(d['attackers'], [d['attacker']])
        self.assertIn({'seat': 'sauron', 'perm': s.perms.index(b)}, d['refs'])          # (and the Orc Army it made)

if __name__ == '__main__':
    unittest.main()
