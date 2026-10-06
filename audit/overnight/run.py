"""Crash-proof overnight look-ahead run: every game is its own process, so one game that crashes or hangs costs only
that game, never the run.

    python3 audit/overnight/run.py run  OUT --decks sauron,seph,veyran,yshtola --stop-launch 06:00 --deadline 06:28
    python3 audit/overnight/run.py run  OUT --decks seph --tiers t4,t5 --games 480 --swap "Phyrexian Arena=>Bolas's Citadel"
    python3 audit/overnight/run.py game DECK TIER SEED [--swap ...]   # one game, prints one JSON line (used by `run`)
    python3 audit/overnight/run.py report OUT                   # the deck x tier table from OUT/results.jsonl

With --swap, every seed is played twice, by the current list and by the swapped one (each swapped card keeps its
list position, as in `--pool --swap` runs), and the report adds the paired difference per cell. --games stops each
cell after that many seeds.

`run` plays the decks' games in rounds (seed 500000, 500001, ... for every deck and tier in turn), so whenever it stops
every cell has about the same number of games. The seeds are the same as `--pool` runs use (poolmode.play), so the
results pair with later CLI runs. Each game gets a wall-clock limit (--timeout); a game over it is killed and recorded
as a timeout. Results are appended to OUT/results.jsonl as they finish; restarting `run` resumes, skipping finished
games. OUT/events.log has one line per crash, timeout and hourly progress; OUT/status.json is rewritten every 30 s.
"""
import argparse, datetime as dt, json, os, signal, subprocess, sys, time
from collections import Counter, defaultdict

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
TIERS = ('t1', 't2', 't3', 't4', 't5')
SEED0 = 500000


# ------------------------------------------------------------------ one game
def game(deck, tier, seed, profile, temp, swap=None):
    sys.path.insert(0, ROOT)
    from commander_sim import poolmode as PM, compare as C
    from commander_sim.decks import DECKS
    PM._setup(profile, 'lookahead', temp)
    cards = None
    if swap:
        swaps, _ = C.parse_swaps(swap)
        C.check_identity(deck, swaps)
        cards = PM.swap_in_place(DECKS[deck], swaps)
    t0 = time.time()
    g = PM.play(seed, deck, cards, PM.pool_keys(tier))
    p = next(x for x in g.players if x.key == deck)
    won = g.winner is p
    out = dict(deck=deck, tier=tier, seed=seed, variant='swap' if swap else 'base', ok=True, won=won, turns=p.turns, wintype=(g.wintype or 'damage') if won else None,
               opps=[q.key for q in g.players if q is not p], winner=g.winner.key if g.winner is not None else None,
               killer=p.killer.key if (not p.alive and p.killer is not None and p.killer is not p) else None,
               plan=bool(C.plan_turn(p, deck)), secs=round(time.time() - t0, 1))
    print(json.dumps(out), flush=True)


# ------------------------------------------------------------------ the supervisor
def at(hhmm):
    """the next time it is hh:mm (today, or tomorrow if that has passed)"""
    h, m = map(int, hhmm.split(':'))
    now = dt.datetime.now()
    t = now.replace(hour=h, minute=m, second=0, microsecond=0)
    if t <= now: t += dt.timedelta(days=1)
    return t.timestamp()


def event(out, msg):
    line = f"{dt.datetime.now():%H:%M:%S} {msg}"
    with open(os.path.join(out, 'events.log'), 'a') as fh: fh.write(line + '\n')
    print(line, flush=True)


def load_done(out):
    done, rows = set(), []
    path = os.path.join(out, 'results.jsonl')
    if os.path.exists(path):
        for line in open(path):
            try: r = json.loads(line)
            except ValueError: continue
            rows.append(r); done.add(key_of(r))
    return done, rows


def key_of(r):
    return (r['deck'], r['tier'], r['seed'], r.get('variant', 'base'))


def queue(decks, tiers, variants, done, games=None):
    """(deck, tier, seed, variant) in rounds of seeds; a seed's variants next to each other (a pair finishes together)"""
    i = 0
    while games is None or i < games:
        for tier in tiers:
            for d in decks:
                for v in variants:
                    k = (d, tier, SEED0 + i, v)
                    if k not in done: yield k
        i += 1


def kill(proc):
    try: os.killpg(proc.pid, signal.SIGKILL)
    except (ProcessLookupError, PermissionError): pass


def run(a):
    out = os.path.abspath(a.out); os.makedirs(out, exist_ok=True)
    stop_launch, deadline = at(a.stop_launch), at(a.deadline)
    if stop_launch > deadline: stop_launch = deadline
    decks = a.decks.split(',')
    tiers = a.tiers.split(',')
    variants = ('base', 'swap') if a.swap else ('base',)
    done, rows = load_done(out)
    q = queue(decks, tiers, variants, done, a.games)
    exhausted = False
    res = open(os.path.join(out, 'results.jsonl'), 'a')
    os.makedirs(os.path.join(out, 'tmp'), exist_ok=True)
    running = {}                     # proc -> (key, start, logfile)
    stats = Counter(ok=sum(1 for r in rows if r.get('ok')), crash=0, timeout=0)
    event(out, f"start: {len(done)} games already done; decks {decks}; tiers {tiers}; swap {a.swap or 'none'}; "
               f"games per cell {a.games or 'until stop'}; jobs {a.jobs}; timeout {a.timeout}s; "
               f"stop launching {dt.datetime.fromtimestamp(stop_launch):%H:%M}, deadline {dt.datetime.fromtimestamp(deadline):%H:%M}")
    last_status = last_hour = time.time()

    def record(r):
        res.write(json.dumps(r) + '\n'); res.flush(); os.fsync(res.fileno())

    while True:
        now = time.time()
        while len(running) < a.jobs and now < stop_launch and not exhausted:
            key = next(q, None)
            if key is None: exhausted = True; break
            name = '-'.join(map(str, key))
            errf = open(os.path.join(out, 'tmp', name + '.err'), 'w')
            outf = open(os.path.join(out, 'tmp', name + '.out'), 'w')      # a file, not a pipe: a chatty game can't block
            extra = [x for sw in a.swap for x in ('--swap', sw)] if key[3] == 'swap' else []
            p = subprocess.Popen([sys.executable, os.path.abspath(__file__), 'game', *map(str, key[:3]),
                                  '--profile', a.profile, '--temp', str(a.temp), *extra], cwd=ROOT, stdout=outf,
                                 stderr=errf, text=True, start_new_session=True)
            outf.close()
            running[p] = (key, now, errf)
        for p in list(running):
            key, t0, errf = running[p]
            rc = p.poll()
            if rc is None:
                if now - t0 > a.timeout or now > deadline:
                    kill(p); p.wait()
                    errf.close()
                    for f in (errf.name, errf.name[:-4] + '.out'):
                        if os.path.exists(f): os.remove(f)
                    why = 'deadline' if now > deadline else 'timeout'
                    del running[p]
                    if why == 'timeout':
                        stats['timeout'] += 1
                        record(dict(deck=key[0], tier=key[1], seed=key[2], variant=key[3], ok=False, error='timeout',
                                    secs=round(now - t0)))
                        event(out, f"TIMEOUT {' '.join(map(str, key))} after {now - t0:.0f}s (killed; run continues)")
                continue
            errf.close()
            outname = errf.name[:-4] + '.out'
            line = open(outname).read().strip().splitlines()
            del running[p]
            r = None
            if rc == 0 and line:
                try: r = json.loads(line[-1])
                except ValueError: r = None
            if r is not None:
                stats['ok'] += 1; record(r)
                os.remove(errf.name); os.remove(outname)
            else:
                stats['crash'] += 1
                tail = open(errf.name).read().strip().splitlines()[-1:] or ['(no output)']
                record(dict(deck=key[0], tier=key[1], seed=key[2], variant=key[3], ok=False,
                            error=f'exit {rc}: {tail[0][:200]}', secs=round(now - t0)))
                event(out, f"CRASH {' '.join(map(str, key))} exit {rc}: {tail[0][:160]} (run continues)")
        if now - last_status > 30:
            last_status = now
            json.dump(dict(time=f'{dt.datetime.now():%H:%M:%S}', running=len(running), **stats,
                           launching=now < stop_launch), open(os.path.join(out, 'status.json'), 'w'))
        if now - last_hour > 3600:
            last_hour = now
            event(out, f"progress: {stats['ok']} games done, {stats['crash']} crashes, {stats['timeout']} timeouts, "
                       f"{len(running)} running")
        if not running and (now >= stop_launch or exhausted): break
        time.sleep(1)
    event(out, f"finished: {stats['ok']} games done, {stats['crash']} crashes, {stats['timeout']} timeouts")
    report(out)


# ------------------------------------------------------------------ the report
def _pair_row(d, t, ds):
    import math
    n = len(ds)
    if not n: return f'| {d} | {t} | 0 | | | | |'
    diff = [s - b for b, s in ds]
    m = sum(diff) / n
    se = math.sqrt(sum((x - m) ** 2 for x in diff) / max(1, n - 1) / n)
    b = sum(x for x, _ in ds) / n; s = sum(y for _, y in ds) / n
    return (f'| {d} | {t} | {n} | {100*b:.1f}% | {100*s:.1f}% | {100*m:+.1f} pts | '
            f'{100*(m - 1.96*se):+.1f} to {100*(m + 1.96*se):+.1f} |')


def wilson(k, n, z=1.96):
    import math
    if n == 0: return 0.0, 0.0
    p = k / n; d = 1 + z * z / n
    c = (p + z * z / (2 * n)) / d; h = z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n)) / d
    return c - h, c + h


def report(out):
    _, rows = load_done(out)
    cells = defaultdict(list); bad = defaultdict(Counter)
    pairs = defaultdict(dict)                         # (deck, tier) -> seed -> {variant: won}
    for r in rows:
        if r.get('ok'):
            pairs[(r['deck'], r['tier'])].setdefault(r['seed'], {})[r.get('variant', 'base')] = r['won']
            if r.get('variant', 'base') == 'base': cells[(r['deck'], r['tier'])].append(r)
        else: bad[r['deck']][r['error'].split(':')[0]] += 1
    decks = list(dict.fromkeys(r['deck'] for r in rows))
    lines = [f"# Overnight look-ahead run ({os.path.basename(out)})", '',
             'Look-ahead AI, loose profile, each deck against three decks of the tier (25% = an even share). '
             'Win rate (95% interval), games in the cell.', '',
             '| Deck | ' + ' | '.join(TIERS) + ' |', '|---|' + '---|' * len(TIERS)]
    for d in decks:
        row = []
        for t in TIERS:
            rs = cells.get((d, t), []); n = len(rs); w = sum(r['won'] for r in rs)
            lo, hi = wilson(w, n)
            row.append(f"{100*w/n:.1f}% ({100*lo:.0f}-{100*hi:.0f}), n={n}" if n else '-')
        lines.append(f'| {d} | ' + ' | '.join(row) + ' |')
    if any(r.get('variant') == 'swap' for r in rows):
        lines += ['', '**Swap: paired difference** (same seeds, opponents and draws; swapped list minus current list; '
                  '95% interval = 1.96 standard errors of the per-seed differences)', '',
                  '| Deck | Tier | pairs | current | swapped | difference | 95% interval |', '|---|---|---|---|---|---|---|']
        allp = defaultdict(list)
        for (d, t), by in sorted(pairs.items(), key=lambda x: (x[0][0], TIERS.index(x[0][1]))):
            ds = [(v['base'], v['swap']) for v in by.values() if 'base' in v and 'swap' in v]
            allp[d] += ds
            lines.append(_pair_row(d, t, ds))
        for d, ds in allp.items():
            if len({t for (dd, t) in pairs if dd == d}) > 1: lines.append(_pair_row(d, 'all', ds))
    lines += ['', '| Deck | games | avg minutes per game | slowest (min) | crashes | timeouts |', '|---|---|---|---|---|---|']
    for d in decks:
        rs = [r for r in rows if r.get('ok') and r['deck'] == d]
        secs = [r['secs'] for r in rs]
        lines.append(f"| {d} | {len(rs)} | {sum(secs)/len(secs)/60:.1f} | {max(secs)/60:.1f} | "
                     f"{sum(v for k, v in bad[d].items() if k != 'timeout')} | {bad[d]['timeout']} |" if rs else f'| {d} | 0 | | | | |')
    txt = '\n'.join(lines) + '\n'
    open(os.path.join(out, 'report.md'), 'w').write(txt)
    print(txt)


if __name__ == '__main__':
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest='cmd', required=True)
    g = sub.add_parser('game'); g.add_argument('deck'); g.add_argument('tier'); g.add_argument('seed', type=int)
    g.add_argument('--profile', default='loose'); g.add_argument('--temp', type=float, default=1.0)
    g.add_argument('--swap', action='append')
    r = sub.add_parser('run'); r.add_argument('out'); r.add_argument('--decks', default='sauron,seph,veyran,yshtola')
    r.add_argument('--jobs', type=int, default=os.cpu_count()); r.add_argument('--timeout', type=int, default=2700)
    r.add_argument('--stop-launch', default='06:00'); r.add_argument('--deadline', default='06:28')
    r.add_argument('--profile', default='loose'); r.add_argument('--temp', type=float, default=1.0)
    r.add_argument('--tiers', default=','.join(TIERS)); r.add_argument('--games', type=int, default=None)
    r.add_argument('--swap', action='append', default=[])
    p = sub.add_parser('report'); p.add_argument('out')
    a = ap.parse_args()
    if a.cmd == 'game': game(a.deck, a.tier, a.seed, a.profile, a.temp, a.swap)
    elif a.cmd == 'run': run(a)
    else: report(a.out)
