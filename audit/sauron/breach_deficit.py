"""What the Breach line is short of: at the first check of each Sauron turn where Breach and a storm piece are both
available and the dry run says no, replay the dry run with extra mana (+1..+6) and, separately, extra graveyard
fuel (+3/+6), and record the smallest boost that would have made it work.

    PYTHONPATH=. python3 audit/sauron/breach_deficit.py t3 200 ["Out=>In" ...]
"""
import collections, sys
from commander_sim import poolmode, compare, engine as E
from commander_sim.decks import DECKS
from commander_sim.cards.impl import mine
from commander_sim.ai import search


def main(tier, n, swaps_raw):
    swaps, _ = compare.parse_swaps(swaps_raw) if swaps_raw else ([], [])
    cards = poolmode.swap_in_place(list(DECKS['sauron']), swaps) if swaps else None
    poolmode._setup('loose', 'adaptive', 1.0, [b for _, b in swaps])
    orig = mine._dry_run
    cur = {}
    out = collections.Counter()

    def boosted(g, p, mana=0, fuel=0):
        g2 = search.clone(g); p2 = g2.players[g.players.index(p)]
        p2.floatU += mana                       # blue: pays Brain Freeze and any generic cost
        if fuel:
            filler = [c for c in p2.library if c.name not in mine.LINE_CARDS][:fuel]
            for c in filler: p2.library.remove(c); p2.gy.append(c)
        g2.breach_dry = None
        return orig(g2, p2)

    def dry(g, p):
        r = orig(g, p)
        if g is cur.get('real') and p.key == 'sauron' and g.active is p and p.turns not in cur['seen']:
            cur['seen'].add(p.turns)
            if r: out['works at the first check'] += 1
            else:
                k = next((k for k in range(1, 7) if boosted(g, p, mana=k)), None)
                f = next((f for f in (3, 6) if boosted(g, p, fuel=f)), None)
                out[f'+{k} mana would do' if k else 'more than +6 mana'] += 1
                out[f'+{f} graveyard cards would do' if f else 'more than +6 graveyard cards'] += 1
                out['checks that failed'] += 1
        return r
    mine._dry_run = dry
    ot = E.tick

    def tick(g):
        cur.setdefault('real', g); return ot(g)
    for mod in list(sys.modules.values()):
        if mod and getattr(mod, '__name__', '').startswith('commander_sim') and getattr(mod, 'tick', None) is ot:
            mod.tick = tick
    ks = poolmode.pool_keys(tier)
    for i in range(n):
        cur.clear(); cur['seen'] = set()
        poolmode.play(800000 + i, 'sauron', cards, ks)
    print(f'{tier}, {n} games, swaps: {swaps_raw or "none"}')
    for k, v in sorted(out.items(), key=lambda x: (not x[0].startswith('+'), x[0])): print(f'  {k:32s} {v}')


if __name__ == '__main__':
    main(sys.argv[1], int(sys.argv[2]), sys.argv[3:])
