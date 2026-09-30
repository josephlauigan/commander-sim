"""python3 -m commander_sim.play --text --deck sauron --tier t4: a practice game (see documents/practice-mode.md)."""
import argparse
import sys

from commander_sim.play.session import Session, MY_DECKS, TIERS


def main(argv=None):
    ap = argparse.ArgumentParser(prog='python3 -m commander_sim.play', description='Play one of your decks against '
                                 'three AI opponents from a tier.')
    ap.add_argument('--text', action='store_true', help='play in the terminal (the browser table comes later)')
    ap.add_argument('--deck', choices=MY_DECKS, default='sauron')
    ap.add_argument('--tier', choices=TIERS, default='t3')
    ap.add_argument('--seed', type=int, help='same seed, same shuffles and seats (default: random)')
    ap.add_argument('--seat', type=int, choices=(1, 2, 3, 4), help='your seat in turn order (default: random)')
    ap.add_argument('--opponents', help='three decks of the tier, comma-separated (default: drawn at random)')
    ap.add_argument('--profile', choices=('conservative', 'loose'), default='loose',
                    help='how much the opponents interact')
    ap.add_argument('--ai', choices=('lookahead', 'adaptive'), default='lookahead',
                    help='opponent AI (adaptive is much faster and a little weaker)')
    ap.add_argument('--step', action='store_true', help='pause at the start of every turn')
    a = ap.parse_args(argv)
    if not a.text:
        ap.error('only the text client exists so far: add --text')
    from commander_sim.play import text
    s = Session(a.deck, a.tier, seed=a.seed, seat=a.seat,
                opponents=a.opponents.split(',') if a.opponents else None, profile=a.profile, ai=a.ai, step=a.step)
    ev = text.run(s)
    return 1 if ev['kind'] == 'error' else 0


if __name__ == '__main__':
    sys.exit(main())
