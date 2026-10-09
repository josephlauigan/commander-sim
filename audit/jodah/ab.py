"""Paired A/B for the Jodah AI fixes (same seeds, the switches in JODAH_AI; see cards/impl/jodah.py).

    JODAH_AI=none        PYTHONPATH=. python3 audit/jodah/ab.py run t1,t2,t3,t4,t5 1000 base.json [--jobs 15] [--ai adaptive]
    JODAH_AI=first,tutor PYTHONPATH=. python3 audit/jodah/ab.py run t1,t2,t3,t4,t5 1000 fix.json
    PYTHONPATH=. python3 audit/jodah/ab.py compare base.json fix.json
    python3 audit/jodah/ab.py merge out.json results.jsonl        # an overnight run's Jodah games as an A/B file
    python3 audit/jodah/ab.py merge out.json t1.json t2.json ...  # single-tier runs as one file

--diag (diagnostics, not fixes: upper bounds on what a problem costs) takes a comma list:
    colourless    every Jodah spell costs its mana value in generic mana (no colour problems)
    cmdcolourless the same for Jodah alone (the commander's WUBRG)
    temp=X        Jodah's AI temperature (style), aggr=X / caution=X its aggression and caution
    safejodah     Jodah can't be targeted or destroyed (removal never touches it)

--swap "Out=>In" (repeatable) plays a changed list (suggestions only; the deck file is not edited).

compare prints each tier's win rate before and after, the paired change and its noise band (2 standard errors of
the per-seed differences), and the change over all tiers.
"""
import json, math, os, sys


def diagnostics(diag):
    """patch the engine in this process before the worker pool forks"""
    from commander_sim import engine as E, poolmode
    from commander_sim.ai import brain
    poolmode._setup('loose', 'adaptive', 1.0)
    for d in diag:
        k, _, v = d.partition('=')
        if k in ('temp', 'aggr', 'caution'):
            brain.STYLE['jodah'][{'aggr': 'aggression'}.get(k, k)] = float(v)
        elif k in ('colourless', 'cmdcolourless'):
            orig = E.cost_of

            def cost_of(p, c, orig=orig, cmd=k == 'cmdcolourless'):
                gen, pips = orig(p, c)
                return (gen + len(pips), '') if p.key == 'jodah' and (c is p.cmd or not cmd) else (gen, pips)
            for m in list(sys.modules.values()):
                if getattr(m, '__name__', '').startswith('commander_sim') and getattr(m, 'cost_of', None) is orig:
                    m.cost_of = cost_of
        elif k == 'safejodah':
            for name in ('untargetable', 'protected_from'):
                orig = getattr(E, name)

                def f(g, m, *a, orig=orig):
                    return True if (m.cd is not None and m.cd.name == 'Jodah, the Unifier') else orig(g, m, *a)
                for mod in list(sys.modules.values()):
                    if getattr(mod, '__name__', '').startswith('commander_sim') and getattr(mod, name, None) is orig:
                        setattr(mod, name, f)
            E.CI.SELF_REGEN['Jodah, the Unifier'] = lambda g, m: True
        else:
            raise SystemExit(f'unknown --diag {d}')


def count_protection():
    """per-game counts for the Jodah seat, saved with each tier (S1: sums over the games): Jodah leaving the
    battlefield, and the protection stats the Jodah AI records (jprot_*: removal or a wipe that a protection card
    stopped, by card; jprot_aimed: removal that reached Jodah while it was already hexproof or indestructible;
    jr_shalai_rem / jr_shalai_taken: opposing removal while Shalai and Jodah were both out / that took Shalai)"""
    from commander_sim import engine as E, compare as C, poolmode
    from commander_sim.cards.impl import jodah as J
    poolmode._setup('loose', 'adaptive', 1.0)                 # every card module loaded (they import apply_removal)
    o_leave, o_axes = E.leave, C.axis_values

    def leave(g, m):
        if m.cd is not None and m.cd.name == 'Jodah, the Unifier' and m in m.owner.perms and m.owner.key == 'jodah':
            m.owner.stats['jodah_left'] += 1
        return o_leave(g, m)

    def axis_values(g, me, deck):
        v = o_axes(g, me, deck)
        if deck == 'jodah':
            v.update({k: x for k, x in me.stats.items() if k.startswith(('jprot', 'jr_')) or k in ('jodah_left', 'jodah_cascades')})
            v['jodah_left'] = me.stats['jodah_left']
        return v
    o_rem = E.apply_removal

    def apply_removal(g, actor, m, kind, spell=None):
        if m.cd is not None and m.cd.name == 'Jodah, the Unifier' and m.owner.key == 'jodah' and m in m.owner.perms \
                and (E.untargetable(g, m) or ((kind == 'destroy' or kind.startswith('dmg')) and E.indestructible(g, m))):
            m.owner.stats['jprot_aimed'] += 1
        o = m.owner
        out = {x.cd.name for x in o.perms if x.cd is not None and not x.phased}
        if o.key == 'jodah' and actor is not o and m in o.perms and {J.SHALAI, 'Jodah, the Unifier'} <= out:
            o.stats['jr_shalai_rem'] += 1                 # opposing removal that had to land elsewhere than Jodah
            if m.cd is not None and m.cd.name == J.SHALAI: o.stats['jr_shalai_taken'] += 1
        return o_rem(g, actor, m, kind, spell)
    for mod in list(sys.modules.values()):
        if getattr(mod, '__name__', '').startswith('commander_sim') and getattr(mod, 'apply_removal', None) is o_rem:
            mod.apply_removal = apply_removal
    E.leave, C.axis_values = leave, axis_values


def run(tiers, n, path, jobs=15, ai='adaptive', seed0=500000, diag=(), swaps=()):
    from commander_sim import poolmode, compare as C
    from commander_sim.decks import DECKS
    if diag: diagnostics(diag)
    count_protection()
    cards = None
    if swaps:                                       # deck-list suggestions, tested paired (the list file is untouched)
        cards = list(DECKS['jodah'])
        for sw in swaps:
            a, b = sw.split('=>'); cards[cards.index(a.strip())] = b.strip()
        poolmode._setup('loose', ai, 1.0, cards)
    C.JOBS = jobs; C.AI, C.TEMP = ai, 1.0; C.set_ai(ai, 1.0)
    out = {'JODAH_AI': os.environ.get('JODAH_AI', '(default)') + (' diag=' + ','.join(diag) if diag else '') + (' swap=' + '; '.join(swaps) if swaps else ''), 'ai': ai}
    for t in tiers:
        R = poolmode.run('jodah', cards, poolmode.pool_keys(t), 'loose', n, seed0)
        out[t] = {'n': R['n'], 'win': R['win'], 'plan': R['plan'], 'by_seed': {str(k): v for k, v in R['by_seed'].items()},
                  'win_turns': R['win_turns'], 'killed_me': dict(R['killed_me']),
                  'S1': {k: v for k, v in R['S1'].items() if k.startswith(('jprot', 'jr_')) or k in ('jodah_left', 'plan_reached', 'jodah_cascades')}}
        print(f"{t}: {100 * R['win'] / R['n']:.1f}% of {R['n']}, plan {100 * R['plan'] / R['n']:.0f}%", flush=True)
        json.dump(out, open(path, 'w'))                 # after each tier: a stopped run keeps what it finished


def paired(b, v):
    seeds = sorted(set(b['by_seed']) & set(v['by_seed']))
    d = [v['by_seed'][s] - b['by_seed'][s] for s in seeds]
    n = len(d)
    m = sum(d) / n
    var = sum((x - m) ** 2 for x in d) / max(1, n - 1)
    return m, math.sqrt(var / n), d


def compare(a, b):
    A, B = json.load(open(a)), json.load(open(b))
    print(f"A: JODAH_AI={A['JODAH_AI']} ({A['ai']})   B: JODAH_AI={B['JODAH_AI']} ({B['ai']})")
    alld = []
    for t in sorted(k for k in A if k.startswith('t') and k in B):
        m, se, d = paired(A[t], B[t]); alld += d
        x, y = A[t], B[t]
        print(f"{t}: {100 * x['win'] / x['n']:5.1f}% -> {100 * y['win'] / y['n']:5.1f}%   delta {100 * m:+.1f} pts "
              f"(noise ±{200 * se:.1f})   plan {100 * x['plan'] / x['n']:.0f}% -> {100 * y['plan'] / y['n']:.0f}%   n={len(d)}")
    n = len(alld); m = sum(alld) / n
    se = math.sqrt(sum((x - m) ** 2 for x in alld) / max(1, n - 1) / n)
    print(f"all: delta {100 * m:+.1f} pts (noise ±{200 * se:.1f}), n={n}")


def from_results(src, out, *more):
    """an overnight results.jsonl (or several ab.py files of single tiers) as one A/B file: the baseline of a paired
    look-ahead check (games are deterministic per seed, so the overnight run's games are the old code's games)"""
    res = {'JODAH_AI': 'overnight run' if src.endswith('.jsonl') else 'merged', 'ai': 'lookahead'}
    if src.endswith('.jsonl'):
        for line in open(src):
            r = json.loads(line)
            if r.get('deck') != 'jodah' or not r.get('ok'): continue
            t = res.setdefault(r['tier'], {'n': 0, 'win': 0, 'plan': 0, 'by_seed': {}})
            t['n'] += 1; t['win'] += bool(r['won']); t['plan'] += bool(r.get('plan')); t['by_seed'][str(r['seed'])] = bool(r['won'])
    else:
        for f in (src,) + more:
            d = json.load(open(f)); res['ai'] = d['ai']; res['JODAH_AI'] = d['JODAH_AI']
            res.update({k: v for k, v in d.items() if k.startswith('t')})
    json.dump(res, open(out, 'w'))


if __name__ == '__main__':
    a = sys.argv[1:]
    if a[0] == 'merge':
        from_results(a[2], a[1], *a[3:]); sys.exit()
    if a[0] == 'run':
        kw = {}
        if '--jobs' in a: kw['jobs'] = int(a[a.index('--jobs') + 1])
        if '--ai' in a: kw['ai'] = a[a.index('--ai') + 1]
        if '--seed0' in a: kw['seed0'] = int(a[a.index('--seed0') + 1])
        if '--diag' in a: kw['diag'] = a[a.index('--diag') + 1].split(',')
        kw['swaps'] = [a[i + 1] for i, x in enumerate(a) if x == '--swap']
        run(a[1].split(','), int(a[2]), a[3], **kw)
    else:
        compare(a[1], a[2])
