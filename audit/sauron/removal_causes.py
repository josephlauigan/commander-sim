import collections, sys, inspect
from commander_sim import poolmode, pools, ais, engine as E
pools.register(); poolmode._setup('loose', 'adaptive', 1.0)
N = int(sys.argv[1])
causes = collections.Counter(); wipes_own = collections.Counter()
orig_leave = E.leave
def leave(g, m):
    p = m.owner
    if p.key == 'sauron' and m in p.perms and (m.cd is p.cmd or m.army):
        what = 'Sauron' if m.cd is p.cmd else 'Army'
        why = 'other'
        for fr in inspect.stack()[1:25]:
            f, loc = fr.function, fr.frame.f_locals
            if f == 'apply_wipe':
                who = loc.get('p'); why = 'own wipe' if who is p else 'opponent wipe'
                if who is p: wipes_own[(what, (g.cur_cast[0].name if getattr(g, 'cur_cast', None) else '?'))] += 1
                break
            if f == 'apply_removal':
                who = loc.get('actor'); why = 'own removal' if who is p else 'opponent spot removal'; break
            if f in ('_resolve_combat',): why = 'combat'; break
            if f in ('edict',): why = 'edict'; break
            if f in ('seph_sac', 'die') and loc.get('cause') == 'sac': why = 'sacrificed'; break
        causes[(what, why)] += 1
    return orig_leave(g, m)
E.leave = leave
for t in ('t1', 't3', 't5'):
    ks = poolmode.pool_keys(t)
    for i in range(N): poolmode.play(600000 + i, 'sauron', None, ks)
for k, v in sorted(causes.items(), key=lambda x: -x[1]): print(f'{k[0]:7s} {k[1]:22s} {v/(3*N):.2f} per game')
print('own wipes that hit Sauron/Army:', wipes_own.most_common(8))
