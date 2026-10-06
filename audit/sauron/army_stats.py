import collections, statistics, sys
from commander_sim import poolmode, pools, ais, engine as E
pools.register(); poolmode._setup('loose', 'adaptive', 1.0)
N = int(sys.argv[1]) if len(sys.argv) > 1 else 300
st = collections.defaultdict(list)
cur = {}
orig_enter, orig_leave = E.enter, E.leave
def enter(g, p, cd, *a, **k):
    m = orig_enter(g, p, cd, *a, **k)
    if p.key == 'sauron' and cd is p.cmd and 'sauron_turn' not in cur: cur['sauron_turn'] = p.turns
    return m
def leave(g, m):
    p = m.owner
    if p.key == 'sauron' and m in p.perms:
        if m.cd is p.cmd: cur['sauron_removed'] = cur.get('sauron_removed', 0) + 1
        if m.army: cur['army_lost'] = cur.get('army_lost', 0) + 1; cur['army_lost_pow'] = cur.get('army_lost_pow', 0) + E.epow(g, m)
    return orig_leave(g, m)
E.enter = enter; E.leave = leave
orig_tick = E.tick
def tick(g):
    for p in g.players:
        if p.key == 'sauron':
            a = E.army_of(p)
            if a: cur['army_max'] = max(cur.get('army_max', 0), E.epow(g, a))
            cur['lands6'] = cur.get('lands6') if p.turns != 6 else len(p.lands)
    return orig_tick(g)
E.tick = tick
keys = {t: poolmode.pool_keys(t) for t in ('t1', 't3', 't5')}
for t, ks in keys.items():
    for i in range(N):
        cur.clear()
        g = poolmode.play(600000 + i, 'sauron', None, ks)
        p = next(x for x in g.players if x.key == 'sauron')
        won = g.winner is p
        st[t, 'won'].append(won)
        st[t, 'sauron_turn'].append(cur.get('sauron_turn'))
        st[t, 'sauron_removed'].append(cur.get('sauron_removed', 0))
        st[t, 'army_max'].append(cur.get('army_max', 0))
        st[t, 'army_lost'].append(cur.get('army_lost', 0))
        st[t, 'lands_t6'].append(cur.get('lands6'))
        st[t, 'turns'].append(p.turns)
        st[t, 'mulls'].append(p.stats['mulls'])
        st[t, 'cmd_dmg'].append(sum(q.cmd_dmg.get('sauron', 0) for q in g.players))
for t in keys:
    s = lambda k: [x for x in st[t, k] if x is not None]
    tt = s('sauron_turn')
    print(f"== {t}: win {sum(st[t,'won'])/N:.1%}; Sauron cast in {len(tt)/N:.0%} of games, median turn {statistics.median(tt)}, "
          f"by turn 6 {sum(1 for x in tt if x <= 6)/N:.0%}; removed {statistics.mean(st[t,'sauron_removed']):.2f}/game")
    am = st[t, 'army_max']
    print(f"   army max power: median {statistics.median(am)}, >=10 in {sum(1 for x in am if x >= 10)/N:.0%}, >=20 in {sum(1 for x in am if x >= 20)/N:.0%}; "
          f"armies lost {statistics.mean(st[t,'army_lost']):.2f}/game; lands on turn 6 median {statistics.median(s('lands_t6')) if s('lands_t6') else '-'}; "
          f"turns played median {statistics.median(st[t,'turns'])}")
    w = [a for a, x in zip(am, st[t,'won']) if x]; l = [a for a, x in zip(am, st[t,'won']) if not x]
    print(f"   army max in wins {statistics.median(w) if w else '-'} vs losses {statistics.median(l)}")
