"""How close Sauron gets to the Underworld Breach line, per game: Breach and a storm piece both available on Sauron's
main phase, the dry run saying yes, the line played, the game won after it.

    PYTHONPATH=. python3 audit/sauron/breach_stats.py t3 300 ["Out=>In" ...]
"""
import collections, sys
from commander_sim import poolmode, pools, compare, engine as E
from commander_sim.decks import DECKS
from commander_sim.cards.impl import mine


def main(tier, n, swaps_raw):
    swaps, _ = compare.parse_swaps(swaps_raw) if swaps_raw else ([], [])
    cards = poolmode.swap_in_place(list(DECKS['sauron']), swaps) if swaps else None
    poolmode._setup('loose', 'adaptive', 1.0, [b for _, b in swaps])
    cur = {}
    orig_line = mine.breach_line

    def breach_line(g, p, execute=False):
        r = orig_line(g, p, execute)
        if p.key == 'sauron' and g is cur.get('real'):          # not the dry run's copy
            if execute: cur['played'] = True
            elif r: cur['dry_yes'] = True
        return r
    mine.breach_line = breach_line
    orig_tick = E.tick

    def tick(g):
        cur.setdefault('real', g)                       # the first game seen is the real one; copies come later
        if g is not cur['real']: return orig_tick(g)
        p = next((q for q in g.players if q.key == 'sauron'), None)
        if p is not None and p.alive and g.active is p:
            names = {c.name for c in p.hand} | {c.name for c in p.gy}
            br = E.has(p, 'breach') or 'Underworld Breach' in names
            stp = bool(names & {'Brain Freeze', 'Grapeshot'})
            if br and stp: cur.setdefault('pieces', p.turns)
            if br: cur.setdefault('breach', p.turns)
        return orig_tick(g)
    for mod in list(sys.modules.values()):
        if mod and getattr(mod, '__name__', '').startswith('commander_sim') and getattr(mod, 'tick', None) is orig_tick:
            mod.tick = tick
    ks = poolmode.pool_keys(tier)
    C = collections.Counter(); W = collections.Counter()
    for i in range(n):
        cur.clear()
        g = poolmode.play(800000 + i, 'sauron', cards, ks)
        p = next(x for x in g.players if x.key == 'sauron')
        won = g.winner is p
        C['games'] += 1; W['games'] += won
        for k in ('breach', 'pieces', 'dry_yes', 'played'):
            if k in cur: C[k] += 1; W[k] += won
        if 'pieces' in cur and 'dry_yes' not in cur: C['pieces_no_dry'] += 1; W['pieces_no_dry'] += won
    print(f'{tier}, {n} games, swaps: {swaps_raw or "none"}')
    for k, lab in (('games', 'all games'), ('breach', 'Breach in hand/gy/battlefield on own turn'),
                   ('pieces', '...and Brain Freeze/Grapeshot too'), ('pieces_no_dry', '...but the dry run never said yes'),
                   ('dry_yes', 'dry run said yes'), ('played', 'line played')):
        print(f'  {lab:45s} {C[k]/n:6.1%} of games, win {W[k]/C[k] if C[k] else 0:6.1%}')


if __name__ == '__main__':
    main(sys.argv[1], int(sys.argv[2]), sys.argv[3:])
