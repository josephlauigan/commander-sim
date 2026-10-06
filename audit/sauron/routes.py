"""How Sauron wins and how it loses, per game: damage to opponents split by source (Army / Sauron / other creatures /
noncombat), who eliminated each opponent and how, which plan pieces were ever assembled, and for losses: round, killer,
cause, and what Sauron had on the battlefield when it died.

    PYTHONPATH=. python3 audit/sauron/routes.py run t3 0 200 out.json    # one chunk (tier, first seed offset, games)
    PYTHONPATH=. python3 audit/sauron/routes.py report out1.json out2.json ...
"""
import collections, inspect, json, statistics, sys
from commander_sim import poolmode, pools, engine as E


def run(tier, off, n, out):
    pools.register(); poolmode._setup('loose', 'adaptive', 1.0)
    cur = {}
    orig_ll, orig_elim = E.lose_life, E.eliminate

    def src_name(fr):
        for _ in range(10):
            fr = fr.f_back
            if fr is None: return '?'
            L = fr.f_locals
            for k in ('src', 'c', 'card', 'm'):
                v = L.get(k); nm = getattr(v, 'name', None)
                if isinstance(nm, str) and not hasattr(v, 'perms') and nm in cur['names']: return nm
        return '?'

    def lose_life(g, p, n, src, kind='other', damage=None):
        before = p.life if p.alive else None
        orig_ll(g, p, n, src, kind, damage)
        me = cur.get('me')
        if me is None or src is not me or p is me or before is None: return
        lost = before - p.life
        if lost <= 0: return
        if kind == 'combat':
            a = inspect.currentframe().f_back.f_locals.get('a')
            if a is not None and getattr(a, 'army', False): cat = 'army'
            elif a is not None and a.cd is me.cmd: cat = 'sauron'
            else: cat = 'creature:' + getattr(a, 'name', '?')
        else:
            cat = f'{kind}:{src_name(inspect.currentframe())}'
        cur['dmg'][cat] += lost

    def eliminate(g, p):
        me = cur.get('me')
        if me is not None and p is me:
            a = E.army_of(p)
            cur['death'] = {'round': g.round, 'turns': p.turns, 'sauron_out': any(m.cd is p.cmd for m in p.perms),
                            'army_pow': E.epow(g, a) if a else 0, 'life': p.life, 'creatures': sum(1 for m in p.perms if m.creature),
                            'lands': len(p.lands)}
        orig_elim(g, p)
        if me is not None and p is me:
            cur['death'].update(kind=p.death_kind, killer=getattr(p.killer, 'key', None) if p.killer is not p else 'self')
        elif me is not None and p.killer is me:
            cur['kills'].append([p.death_kind, g.round])

    orig_tick = E.tick
    def tick(g):
        if cur.get('me') is None:                     # the Sauron player exists once the game is built
            cur['me'] = me = next((q for q in g.players if q.key == 'sauron'), None)
            cur['names'] = set(me.deck_names) | {'Token'}
        me = cur.get('me')
        if me is not None and me.alive:
            a = E.army_of(me)
            if a:
                cur['army_max'] = max(cur['army_max'], E.epow(g, a))
                if E.equipped(a, 'sword'): cur['flags'].add('sword_on_army')
                if E.equipped(a, 'sword') and E.has(me, 'assault'): cur['flags'].add('sword+assault')
            if E.has(me, 'assault'): cur['flags'].add('assault_out')
            if E.has(me, 'breach'): cur['flags'].add('breach_out')
            if any(m.cd is me.cmd for m in me.perms) and 'cmd_turn' not in cur: cur['cmd_turn'] = me.turns
        return orig_tick(g)

    for mod in list(sys.modules.values()):
        if not mod or not getattr(mod, '__name__', '').startswith('commander_sim'): continue
        if getattr(mod, 'lose_life', None) is orig_ll: mod.lose_life = lose_life
        if getattr(mod, 'eliminate', None) is orig_elim: mod.eliminate = eliminate
        if getattr(mod, 'tick', None) is orig_tick: mod.tick = tick

    ks = poolmode.pool_keys(tier)
    rows = []
    for i in range(off, off + n):
        seed = 700000 + i
        cur.clear(); cur.update(dmg=collections.Counter(), kills=[], flags=set(), army_max=0, names=set())
        try:
            g = poolmode.play(seed, 'sauron', None, ks)
        except Exception as e:
            rows.append({'seed': seed, 'err': f'{e.__class__.__name__}: {e}'[:120]}); continue
        p = next(x for x in g.players if x.key == 'sauron')
        rows.append({'seed': seed, 'tier': tier, 'won': g.winner is p, 'wintype': g.wintype if g.winner is p else None,
                     'winner': getattr(g.winner, 'key', None), 'rounds': g.round, 'dmg': dict(cur['dmg']), 'kills': cur['kills'],
                     'flags': sorted(cur['flags']), 'army_max': cur['army_max'], 'cmd_turn': cur.get('cmd_turn'),
                     'death': cur.get('death'), 'opps': [q.key for q in g.players if q is not p]})
    json.dump(rows, open(out, 'w'))


def bucket(cat):
    if cat in ('army', 'sauron'): return cat
    if cat.startswith('creature:'): return 'other creatures'
    return 'noncombat'


def report(files):
    rows = [r for f in files for r in json.load(open(f)) if 'err' not in r]
    errs = sum(1 for f in files for r in json.load(open(f)) if 'err' in r)
    by = collections.defaultdict(list)
    for r in rows: by[r['tier']].append(r)
    print(f'{len(rows)} games ({errs} errors)\n')
    for t in sorted(by) + ['all']:
        R = rows if t == 'all' else by[t]
        W = [r for r in R if r['won']]; Lo = [r for r in R if not r['won']]
        print(f'===== {t}: {len(R)} games, win {len(W)/len(R):.1%}')
        for lab, S in (('wins', W), ('losses', Lo)):
            tot = collections.Counter()
            for r in S:
                for k, v in r['dmg'].items(): tot[bucket(k)] += v
            s = sum(tot.values()) or 1
            print(f'  damage to opponents in {lab} ({sum(tot.values())/max(1,len(S)):.0f}/game): ' +
                  ', '.join(f'{k} {v/s:.0%}' for k, v in tot.most_common()))
        wt = collections.Counter(r['wintype'] for r in W)
        print('  win type: ' + ', '.join(f'{k} {v/len(W):.0%}' for k, v in wt.most_common()) if W else '  no wins')
        kk = collections.Counter(k for r in R for k, _ in r['kills'])
        print(f'  opponents Sauron eliminated: {sum(kk.values())/len(R):.2f}/game: ' + ', '.join(f'{k} {v}' for k, v in kk.most_common()))
        # main damage source per win
        prim = collections.Counter()
        for r in W:
            b = collections.Counter()
            for k, v in r['dmg'].items(): b[bucket(k)] += v
            prim[b.most_common(1)[0][0] if b else 'none (others did the work)'] += 1
        if W: print('  biggest damage source in each win: ' + ', '.join(f'{k} {v/len(W):.0%}' for k, v in prim.most_common()))
        for fl in ('sword_on_army', 'assault_out', 'sword+assault', 'breach_out'):
            a = [r for r in R if fl in r['flags']]; b = [r for r in R if fl not in r['flags']]
            wa = sum(r['won'] for r in a) / len(a) if a else 0; wb = sum(r['won'] for r in b) / len(b) if b else 0
            print(f'  {fl:14s} in {len(a)/len(R):5.1%} of games: win {wa:5.1%} with vs {wb:5.1%} without')
        for lo, hi in ((0, 5), (5, 10), (10, 16), (16, 999)):
            a = [r for r in R if lo <= r['army_max'] < hi]
            if a: print(f'  Army peak {lo:>2}-{hi if hi < 999 else "+":<3}: {len(a)/len(R):5.1%} of games, win {sum(r["won"] for r in a)/len(a):5.1%}')
        ct = [r['cmd_turn'] for r in R if r['cmd_turn']]
        for lo, hi in ((0, 6), (6, 7), (7, 9), (9, 99)):
            a = [r for r in R if r['cmd_turn'] and lo <= r['cmd_turn'] < hi]
            if a: print(f'  Sauron cast turn {lo}-{hi-1 if hi < 99 else "+"}: {len(a)/len(R):5.1%} of games, win {sum(r["won"] for r in a)/len(a):5.1%}')
        a = [r for r in R if not r['cmd_turn']]
        if a: print(f'  Sauron never cast:   {len(a)/len(R):5.1%} of games, win {sum(r["won"] for r in a)/len(a):5.1%}')
        D = [r['death'] for r in Lo if r.get('death')]
        if D:
            print(f'  eliminated in {len(D)/len(R):.0%} of games, median round {statistics.median(d["round"] for d in D)}, '
                  f'Sauron out at death {sum(d["sauron_out"] for d in D)/len(D):.0%}, Army power at death median {statistics.median(d["army_pow"] for d in D)}, '
                  f'creatures {statistics.median(d["creatures"] for d in D)}')
            dk = collections.Counter(d['kind'] for d in D)
            print('  cause of death: ' + ', '.join(f'{k} {v/len(D):.0%}' for k, v in dk.most_common()))
            kl = collections.Counter(d['killer'] for d in D)
            print('  killed by: ' + ', '.join(f'{k} {v/len(D):.0%}' for k, v in kl.most_common(6)))
            rd = collections.Counter(min(d['round'] // 3 * 3, 15) for d in D)
            print('  death round: ' + ', '.join(f'{k}-{k+2 if k < 15 else "+"}: {v/len(D):.0%}' for k, v in sorted(rd.items())))
        Ls = [r for r in Lo if not r.get('death')]
        if Ls: print(f'  lost but survived to the end (someone else won): {len(Ls)/len(R):.0%}')
        if t == 'all':
            nc = collections.Counter()
            for r in rows:
                for k, v in r['dmg'].items():
                    if bucket(k) in ('noncombat', 'other creatures'): nc[k] += v
            print('  top non-Army damage sources (total): ' + ', '.join(f'{k} {v}' for k, v in nc.most_common(12)))
        print()


if __name__ == '__main__':
    if sys.argv[1] == 'run': run(sys.argv[2], int(sys.argv[3]), int(sys.argv[4]), sys.argv[5])
    else: report(sys.argv[2:])
