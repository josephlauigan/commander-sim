"""Deck validator: structural checks (offline) and card-data checks (synthetic Scryfall records)."""
import os, sys, unittest
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import pools


def rec(name, ident='', gc=False, banned=False, type_line='Instant', faces=None):
    r = {'name': name, 'color_identity': list(ident), 'game_changer': gc, 'type_line': type_line,
         'legalities': {'commander': 'banned' if banned else 'legal'}}
    if faces: r['card_faces'] = [{'name': f} for f in faces]
    return r


class FakeDeck:
    def __init__(s, cards, commander='Cmdr', tier='t1', gc_claimed=None):
        s.cards, s.commander, s.tier, s.gc_claimed = cards, commander, tier, gc_claimed


def deck_of(spells, commander='Cmdr', basics='Island'):
    return [commander] + spells + [basics] * (99 - len(spells))


class Structural(unittest.TestCase):
    def test_good_deck(self):
        self.assertEqual(pools.structural_problems(deck_of(['A', 'B']), 'Cmdr'), [])

    def test_wrong_size(self):
        self.assertIn('99 cards, expected 100', pools.structural_problems(deck_of(['A'])[:-1], 'Cmdr'))

    def test_duplicate_nonbasic(self):
        p = pools.structural_problems(deck_of(['A', 'A']), 'Cmdr')
        self.assertTrue(any('not singleton: A' in x for x in p))

    def test_basics_may_repeat(self):
        self.assertEqual(pools.structural_problems(deck_of([], basics='Snow-Covered Island'), 'Cmdr'), [])

    def test_commander_missing(self):
        p = pools.structural_problems(deck_of(['A']), 'Someone Else')
        self.assertTrue(any('not in the import list' in x for x in p))
        self.assertTrue(pools.structural_problems(deck_of(['A']), None))


class CardData(unittest.TestCase):
    def recs(self, **extra):
        r = {'Cmdr': rec('Cmdr', 'U', type_line='Legendary Creature — Human'),
             'A': rec('A', 'U'), 'Island': rec('Island', type_line='Basic Land — Island')}
        r.update(extra)
        return r

    def test_clean(self):
        probs, info = pools.card_problems(FakeDeck(deck_of(['A'])), self.recs())
        self.assertEqual(probs, []); self.assertEqual(info['identity'], 'U')

    def test_unresolved_name(self):
        probs, _ = pools.card_problems(FakeDeck(deck_of(['A', 'Nope'])), self.recs())
        self.assertTrue(any('not found on Scryfall: Nope' in p for p in probs))

    def test_fuzzy_match_flagged(self):
        probs, _ = pools.card_problems(FakeDeck(deck_of(['Counterspel'])), self.recs(Counterspel=rec('Counterspell', 'U')))
        self.assertTrue(any('fuzzy' in p for p in probs))

    def test_front_face_name_accepted(self):   # Adventure / MDFC written by front face
        r = rec('Bonecrusher Giant // Stomp', 'R', faces=['Bonecrusher Giant', 'Stomp'])
        self.assertTrue(pools.resolved_name_ok('Bonecrusher Giant', r))
        self.assertTrue(pools.resolved_name_ok('Bonecrusher Giant // Stomp', r))

    def test_colour_identity(self):
        probs, _ = pools.card_problems(FakeDeck(deck_of(['A', 'Bolt'])), self.recs(Bolt=rec('Bolt', 'R')))
        self.assertTrue(any('outside colour identity' in p and 'Bolt' in p for p in probs))

    def test_banned(self):
        probs, _ = pools.card_problems(FakeDeck(deck_of(['Ban'])), self.recs(Ban=rec('Ban', 'U', banned=True)))
        self.assertTrue(any('banned' in p for p in probs))

    def test_commander_must_be_legendary_creature(self):
        probs, _ = pools.card_problems(FakeDeck(deck_of(['A'])), self.recs(Cmdr=rec('Cmdr', 'U', type_line='Instant')))
        self.assertTrue(any('not a legendary creature' in p for p in probs))

    def test_game_changer_tier_rules(self):
        r = self.recs(G1=rec('G1', 'U', gc=True), G2=rec('G2', 'U', gc=True), G3=rec('G3', 'U', gc=True))
        for tier, spells, ok in (('t1', [], True), ('t1', ['G1'], False), ('t2', ['G1'], True),
                                 ('t2', ['G1', 'G2', 'G3'], False), ('t3', ['G1', 'G2', 'G3'], True),
                                 ('t3', ['G1', 'G2'], False), ('t4', ['G1', 'G2', 'G3'], True)):
            with self.subTest(tier=tier, n=len(spells)):
                probs, _ = pools.card_problems(FakeDeck(deck_of(spells), tier=tier), r)
                self.assertEqual(not any('Game Changers, tier' in p for p in probs), ok)

    def test_claimed_gc_count_mismatch(self):
        probs, _ = pools.card_problems(FakeDeck(deck_of([]), gc_claimed=2), self.recs())
        self.assertTrue(any('file says 2' in p for p in probs))


class RealPool(unittest.TestCase):
    def test_pool_layout(self):
        decks = pools.load_pool()
        self.assertEqual(len(decks), 25)
        for t in pools.TIERS:
            self.assertEqual(len(pools.load_pool(t)), 5, t)
        for d in decks:
            with self.subTest(deck=d.key):
                self.assertEqual(pools.structural_problems(d.cards, d.commander), [])
                self.assertTrue(d.modeling_notes)


if __name__ == '__main__':
    unittest.main()
