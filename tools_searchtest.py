"""Paired test of the look-ahead AI: a pool deck alone against three decks of a tier, the same seeds played with
the heuristic AI and with search, for that deck only.

    python3 tools_searchtest.py <deck key> <tier> <games> [rollouts] [top_k] [all]

With 'all', every deck at the table searches in the second run (not only the tested deck).
Environment: SEARCH_HORIZON (own turns to look ahead, default search.HORIZON), SEARCH_COMBO_W (search.COMBO_W).
One task per seed; a worker that dies stops the run with an error (never a silent hang), a seed that takes
longer than SEED_LIMIT seconds or raises an error is reported and left out.
"""
import os, sys, time
import engine as E, ais, compare, pools, poolmode, search
from concurrent.futures import ProcessPoolExecutor, wait, FIRST_COMPLETED

SEED_LIMIT = 600


def _init(rollouts, top_k):
    pools.register(); compare.set_ai('adaptive', 1.0); E.set_profile('conservative')
    search.ROLLOUTS, search.TOP_K = rollouts, top_k
    search.HORIZON = int(os.environ.get('SEARCH_HORIZON', search.HORIZON))
    search.COMBO_W = float(os.environ.get('SEARCH_COMBO_W', search.COMBO_W))


def _run(me, tier, s, everyone):
    try:
        return _run1(me, tier, s, everyone)
    except Exception:
        import traceback
        return s, None, {}, [], [], traceback.format_exc(limit=-6)


def _run1(me, tier, s, everyone):
    for k in search.STATS: search.STATS[k] = 0
    search.CUTS.clear(); ais.STOPPED.clear()
    r = []
    for keys in (set(), {'*'} if everyone else {me}):
        search.KEYS = keys
        seats = pools.draw_seats(s, poolmode.pool_keys(tier), me, 3)
        g = ais.play_pool_game(s, [poolmode.seat_spec(k) for k in seats])
        r.append(g.winner is not None and g.winner.key == me)
    return s, r, dict(search.STATS), list(search.CUTS), list(ais.STOPPED), None


if __name__ == '__main__':
    me, tier, n = sys.argv[1], sys.argv[2], int(sys.argv[3])
    rollouts = int(sys.argv[4]) if len(sys.argv) > 4 else 6
    top_k = int(sys.argv[5]) if len(sys.argv) > 5 else 5
    everyone = len(sys.argv) > 6 and sys.argv[6] == 'all'
    seeds = list(range(500000, 500000 + n))
    res, st, cuts, stopped, late, errors = [], {}, [], [], [], []
    with ProcessPoolExecutor(24, initializer=_init, initargs=(rollouts, top_k)) as P:
        started = {P.submit(_run, me, tier, s, everyone): s for s in seeds}
        pending, since = set(started), {}             # since: when a task was first seen running
        while pending:
            done, pending = wait(pending, timeout=10, return_when=FIRST_COMPLETED)
            for f in done:
                s, r, stt, c, sp, err = f.result()     # raises if the worker died
                if err: errors.append((s, err)); continue
                res.append(r); cuts += c; stopped += [(s,) + x for x in sp]
                for k, v in stt.items(): st[k] = max(st.get(k, 0), v) if k == 'max_work' else st.get(k, 0) + v
            now = time.time()
            for f in list(pending):
                s = started[f]
                if f.running(): since.setdefault(f, now)
                if f in since and now - since[f] > SEED_LIMIT and s not in late:
                    late.append(s); print(f'  seed {s} past {SEED_LIMIT}s', flush=True)
            if late and all(started[f] in late for f in pending):
                procs = list((P._processes or {}).values())
                for p in procs: p.kill()
                P.shutdown(wait=False, cancel_futures=True)
                break
    m = len(res)
    b = sum(r[0] for r in res) / m; v = sum(r[1] for r in res) / m
    both = sum(1 for r in res if r[0] != r[1])
    print(f'[horizon {os.environ.get("SEARCH_HORIZON", search.HORIZON)}, combo_w {os.environ.get("SEARCH_COMBO_W", search.COMBO_W)}] '
          f'{me} alone vs {tier}: heuristic {100*b:.1f}%, search {100*v:.1f}% ({100*(v-b):+.1f} pts, n={m}, '
          f'{both} games differ in outcome)  {st}')
    for c in cuts[:10]: print('  playout cut:', c)
    for c in stopped: print('  game stopped:', c)
    if late: print('  seeds left out (too slow):', late)
    for s, err in errors: print(f'  seed {s} left out, error:\n{err}')
