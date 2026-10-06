"""Crash-proof overnight look-ahead run: every game is its own process, so one game that crashes or hangs costs only
that game, never the run.

    python3 audit/overnight/run.py run  OUT --decks sauron,seph,veyran,yshtola --stop-launch 06:00 --deadline 06:28
    python3 audit/overnight/run.py game DECK TIER SEED          # one game, prints one JSON line (used by `run`)
    python3 audit/overnight/run.py report OUT                   # the deck x tier table from OUT/results.jsonl

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
def game(deck, tier, seed, profile, temp):
    sys.path.insert(0, ROOT)
    from commander_sim import poolmode as PM, compare as C
    PM._setup(profile, 'lookahead', temp)
    t0 = time.time()
    g = PM.play(seed, deck, None, PM.pool_keys(tier))
    p = next(x for x in g.players if x.key == deck)
    won = g.winner is p
    out = dict(deck=deck, tier=tier, seed=seed, ok=True, won=won, turns=p.turns, wintype=(g.wintype or 'damage') if won else None,
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
            rows.append(r); done.add((r['deck'], r['tier'], r['seed']))
    return done, rows


def queue(decks, done):
    i = 0
    while True:
        for tier in TIERS:
            for d in decks:
                k = (d, tier, SEED0 + i)
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
    done, rows = load_done(out)
    q = queue(decks, done)
    res = open(os.path.join(out, 'results.jsonl'), 'a')
    os.makedirs(os.path.join(out, 'tmp'), exist_ok=True)
    running = {}                     # proc -> (key, start, logfile)
    stats = Counter(ok=sum(1 for r in rows if r.get('ok')), crash=0, timeout=0)
    event(out, f"start: {len(done)} games already done; decks {decks}; jobs {a.jobs}; timeout {a.timeout}s; "
               f"stop launching {dt.datetime.fromtimestamp(stop_launch):%H:%M}, deadline {dt.datetime.fromtimestamp(deadline):%H:%M}")
    last_status = last_hour = time.time()

    def record(r):
        res.write(json.dumps(r) + '\n'); res.flush(); os.fsync(res.fileno())

    while True:
        now = time.time()
        while len(running) < a.jobs and now < stop_launch:
            key = next(q)
            name = f'{key[0]}-{key[1]}-{key[2]}'
            errf = open(os.path.join(out, 'tmp', name + '.err'), 'w')
            outf = open(os.path.join(out, 'tmp', name + '.out'), 'w')      # a file, not a pipe: a chatty game can't block
            p = subprocess.Popen([sys.executable, os.path.abspath(__file__), 'game', *map(str, key),
                                  '--profile', a.profile, '--temp', str(a.temp)], cwd=ROOT, stdout=outf,
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
                        record(dict(deck=key[0], tier=key[1], seed=key[2], ok=False, error='timeout', secs=round(now - t0)))
                        event(out, f"TIMEOUT {key[0]} {key[1]} seed {key[2]} after {now - t0:.0f}s (killed; run continues)")
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
                record(dict(deck=key[0], tier=key[1], seed=key[2], ok=False, error=f'exit {rc}: {tail[0][:200]}',
                            secs=round(now - t0)))
                event(out, f"CRASH {key[0]} {key[1]} seed {key[2]} exit {rc}: {tail[0][:160]} (run continues)")
        if now - last_status > 30:
            last_status = now
            json.dump(dict(time=f'{dt.datetime.now():%H:%M:%S}', running=len(running), **stats,
                           launching=now < stop_launch), open(os.path.join(out, 'status.json'), 'w'))
        if now - last_hour > 3600:
            last_hour = now
            event(out, f"progress: {stats['ok']} games done, {stats['crash']} crashes, {stats['timeout']} timeouts, "
                       f"{len(running)} running")
        if not running and now >= stop_launch: break
        time.sleep(1)
    event(out, f"finished: {stats['ok']} games done, {stats['crash']} crashes, {stats['timeout']} timeouts")
    report(out)


# ------------------------------------------------------------------ the report
def wilson(k, n, z=1.96):
    import math
    if n == 0: return 0.0, 0.0
    p = k / n; d = 1 + z * z / n
    c = (p + z * z / (2 * n)) / d; h = z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n)) / d
    return c - h, c + h


def report(out):
    _, rows = load_done(out)
    cells = defaultdict(list); bad = defaultdict(Counter)
    for r in rows:
        if r.get('ok'): cells[(r['deck'], r['tier'])].append(r)
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
    lines += ['', '| Deck | games | avg minutes per game | slowest (min) | crashes | timeouts |', '|---|---|---|---|---|---|']
    for d in decks:
        rs = [r for (dd, _), v in cells.items() if dd == d for r in v]
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
    r = sub.add_parser('run'); r.add_argument('out'); r.add_argument('--decks', default='sauron,seph,veyran,yshtola')
    r.add_argument('--jobs', type=int, default=os.cpu_count()); r.add_argument('--timeout', type=int, default=2700)
    r.add_argument('--stop-launch', default='06:00'); r.add_argument('--deadline', default='06:28')
    r.add_argument('--profile', default='loose'); r.add_argument('--temp', type=float, default=1.0)
    p = sub.add_parser('report'); p.add_argument('out')
    a = ap.parse_args()
    if a.cmd == 'game': game(a.deck, a.tier, a.seed, a.profile, a.temp)
    elif a.cmd == 'run': run(a)
    else: report(a.out)
