"""Game Changer cast-rate sweep: play each of your decks against every tier and record, for every deck at the table
(the pool opponents included), how often each Game Changer it drew was cast (or, for lands, put onto the battlefield).
A Game Changer that is drawn often but almost never cast is either held on purpose (counters, protection) or an AI gap.

    PYTHONPATH=. python3 audit/gc_sweep.py 100 out.json      # games per (deck, tier); runs on every core
    PYTHONPATH=. python3 audit/gc_sweep.py report out.json
"""
import collections, json, multiprocessing, os, sys


def _gcs():
    from commander_sim.decks import DECKS
    from commander_sim import pools
    from commander_sim.cards import scryfall
    names = {n for v in DECKS.values() for n in v} | {n for d in pools.load_pool() for n in d.cards}
    recs = scryfall.fetch(sorted(names), verbose=False)
    return sorted(n for n in names if recs.get(n, {}).get('game_changer'))


def _chunk(args):
    me, tier, seeds, gcs = args
    from commander_sim import poolmode
    poolmode._setup('loose', 'adaptive', 1.0)
    gcs = set(gcs)
    ks = poolmode.pool_keys(tier)
    out = collections.Counter()
    for s in seeds:
        try:
            g = poolmode.play(s, me, None, ks)
        except Exception as e:
            import traceback
            tb = [f'{fr.name}:{fr.lineno}' for fr in traceback.extract_tb(e.__traceback__)]
            out[('__error__', type(e).__name__, f'{me} {tier} {s} ' + ' < '.join(tb[-12:]))] += 1
            continue
        for p in g.players:
            mine = set(p.deck_names) & gcs
            if not mine: continue
            onbf = {m.cd.name for m in p.perms if m.cd is not None} | {L.cd.name for L in p.lands}
            gone = {c.name for c in p.gy} | {c.name for c in p.exile}
            for n in mine:
                drawn = n in p.seen_names or n in p.cast_names or n in onbf
                if not drawn: continue
                used = n in p.cast_names or n in onbf or (n in gone and n not in {c.name for c in p.hand})
                out[(p.key, n, 'drawn')] += 1
                out[(p.key, n, 'used')] += used
                out[(p.key, n, 'won')] += g.winner is p
                out[(p.key, n, 'won_used')] += (g.winner is p) and used
    return [[list(k), v] for k, v in out.items()]


def run(n, path):
    from commander_sim.decks import DECKS
    gcs = _gcs()
    jobs = []
    for me in DECKS:
        for t in ('t1', 't2', 't3', 't4', 't5'):
            for i in range(0, n, 25):
                jobs.append((me, t, list(range(900000 + i, 900000 + min(n, i + 25))), gcs))
    tot = collections.Counter()
    with multiprocessing.Pool(os.cpu_count()) as pool:
        for res in pool.imap_unordered(_chunk, jobs):
            for k, v in res: tot[tuple(k)] += v
    json.dump([[list(k), v] for k, v in tot.items()], open(path, 'w'))


def report(path):
    tot = {tuple(k): v for k, v in json.load(open(path))}
    errs = sum(v for k, v in tot.items() if k[0] == '__error__')
    rows = collections.defaultdict(dict)
    for (key, n, what), v in tot.items():
        if key == '__error__': continue
        rows[(n, key)][what] = v
    print(f'errors: {errs}')
    print(f'{"Game Changer":36s} {"deck":40s} {"drawn":>6s} {"used%":>6s} {"win% used":>9s} {"win% held":>9s}')
    for (n, key), r in sorted(rows.items(), key=lambda x: (x[0][0], x[0][1])):
        d, u = r.get('drawn', 0), r.get('used', 0)
        wu = r.get('won_used', 0); wh = r.get('won', 0) - wu
        print(f'{n[:36]:36s} {key[:40]:40s} {d:6d} {u / d if d else 0:6.0%} {wu / u if u else 0:9.0%} '
              f'{wh / (d - u) if d > u else 0:9.0%}')


if __name__ == '__main__':
    if sys.argv[1] == 'report': report(sys.argv[2])
    else: run(int(sys.argv[1]), sys.argv[2])
