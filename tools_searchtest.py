"""Paired test of the look-ahead AI: a pool deck alone against three decks of a tier, the same seeds played with
the heuristic AI and with search, for that deck only.

    python3 tools_searchtest.py <deck key> <tier> <games> [rollouts] [top_k] [all]

With 'all', every deck at the table searches in the second run (not only the tested deck).
"""
import sys
import engine as E, ais, compare, pools, poolmode, search
from multiprocessing import Pool


def _run(args):
    me, tier, seeds, rollouts, top_k, everyone = args
    pools.register(); compare.set_ai('adaptive', 1.0); E.set_profile('conservative')
    search.ROLLOUTS, search.TOP_K = rollouts, top_k
    out = []
    for s in seeds:
        r = []
        for keys in (set(), {'*'} if everyone else {me}):
            search.KEYS = keys
            seats = pools.draw_seats(s, poolmode.pool_keys(tier), me, 3)
            g = ais.play_pool_game(s, [poolmode.seat_spec(k) for k in seats])
            r.append(g.winner is not None and g.winner.key == me)
        out.append(r)
    return out, dict(search.STATS)


if __name__ == '__main__':
    me, tier, n = sys.argv[1], sys.argv[2], int(sys.argv[3])
    rollouts = int(sys.argv[4]) if len(sys.argv) > 4 else 6
    top_k = int(sys.argv[5]) if len(sys.argv) > 5 else 5
    everyone = len(sys.argv) > 6 and sys.argv[6] == 'all'
    seeds = list(range(500000, 500000 + n)); chunks = [seeds[i::24] for i in range(24)]
    with Pool(24) as P: parts = P.map(_run, [(me, tier, c, rollouts, top_k, everyone) for c in chunks])
    res = [r for part, _ in parts for r in part]
    st = {k: sum(p[1][k] for p in parts) for k in parts[0][1]}
    b = sum(r[0] for r in res) / n; v = sum(r[1] for r in res) / n
    both = sum(1 for r in res if r[0] != r[1])
    print(f'{me} alone vs {tier}: heuristic {100*b:.1f}%, search {100*v:.1f}% ({100*(v-b):+.1f} pts, n={n}, '
          f'{both} games differ in outcome)  {st}')
