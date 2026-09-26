"""The card ability language: Oracle text compiled into ability data, and the interpreter running it in a game."""
import unittest
from tests.table import table, perm
from commander_sim import engine as E
from commander_sim.cards import dsl_parse


class Compiler(unittest.TestCase):
    def test_a_spell(self):
        ab, bad = dsl_parse.compile_card('Divination', 'Draw two cards.', 'S')
        self.assertEqual(ab, [{'type': 'spell', 'effects': [{'do': 'draw', 'n': 2, 'who': 'you'}]}])
        self.assertEqual(bad, [])

    def test_a_dies_trigger(self):
        ab, _ = dsl_parse.compile_card(
            'Grave Pact', 'Whenever a creature you control dies, each other player sacrifices a creature.', 'E')
        self.assertEqual(len(ab), 1)
        a = ab[0]
        self.assertEqual((a['type'], a['event'], a['source']), ('triggered', 'dies', 'you_creature'))
        self.assertEqual(a['effects'][0]['do'], 'sacrifice')
        self.assertEqual(a['effects'][0]['who'], 'each_opponent')

    def test_an_enters_trigger_with_a_target(self):
        ab, _ = dsl_parse.compile_card(
            'Test Beast', 'When Test Beast enters, destroy target artifact an opponent controls.', 'C')
        e = ab[0]['effects'][0]
        self.assertEqual((ab[0]['event'], e['do']), ('etb', 'destroy'))
        self.assertEqual(e['what']['filter'], {'type': 'artifact', 'controller': 'opp'})

    def test_text_it_cannot_read_is_reported(self):
        line = 'Gain control of all lands named Plains until the next full moon.'
        ab, bad = dsl_parse.compile_card('Weird', line, 'S')
        self.assertEqual((ab, bad), ([], [line]))


class Interpreter(unittest.TestCase):
    def test_niv_mizzet_the_firemind(self):
        # compiled from Scryfall: "Whenever you draw a card, Niv-Mizzet deals 1 damage to any target."
        self.assertIsNotNone(E.DB['Niv-Mizzet, the Firemind'].dsl)
        g = table('veyran', 'seph'); v, s = g.players
        perm(g, v, 'Niv-Mizzet, the Firemind')
        E.draw(g, v, 1)
        self.assertEqual(s.life, 39)


if __name__ == '__main__':
    unittest.main()
