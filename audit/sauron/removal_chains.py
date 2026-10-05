import collections, sys, inspect
from commander_sim import poolmode, pools, ais, engine as E
pools.register(); poolmode._setup('loose', 'adaptive', 1.0)
chains = collections.Counter()
orig_leave = E.leave
SKIP = {'leave','die','apply_removal','apply_wipe','_resolve_combat','edict'}
def leave(g, m):
    p = m.owner
    if p.key == 'sauron' and m in p.perms and (m.cd is p.cmd or m.army):
        fs = [fr.function for fr in inspect.stack()[1:14]]
        if not any(f in ('apply_wipe','apply_removal','_resolve_combat','edict') for f in fs):
            chains[('Sauron' if m.cd is p.cmd else 'Army', ' < '.join(fs[:6]))] += 1
    return orig_leave(g, m)
E.leave = leave
for t in ('t1', 't3', 't5'):
    ks = poolmode.pool_keys(t)
    for i in range(150): poolmode.play(600000 + i, 'sauron', None, ks)
for k, v in chains.most_common(14): print(v, k)
