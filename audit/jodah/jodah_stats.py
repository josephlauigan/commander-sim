"""How the AI pilots Jodah: casts, cascades, attacks, mana, tutors, counters, mulligans, dead cards.

    PYTHONPATH=. python3 audit/jodah/jodah_stats.py run t1,t2,t3,t4,t5 300 out.json [--jobs 15] [--ai adaptive] [--seed0 700000]
    PYTHONPATH=. python3 audit/jodah/jodah_stats.py report out.json [more.json ...]
    PYTHONPATH=. python3 audit/jodah/jodah_stats.py log t2 700012              # print one game's trace

Each game is traced (g.log) and a few engine functions are wrapped to record, for the Jodah seat:
turn-by-turn snapshots (mana left after the second main phase, cards stuck on colours, legends in hand, whether
Jodah is out), every spell it casts (zone, legendary, Jodah out or not), every cascade, every opposing spell it
could have countered, tutor targets, attacks and their size, and what finished it.
"""
import collections, json, re, statistics, sys
from multiprocessing import Pool

JODAH = 'Jodah, the Unifier'
TUTORS = ('Demonic Tutor', 'Vampiric Tutor', 'Enlightened Tutor', 'Mystical Tutor', 'Worldly Tutor', 'Profane Tutor')
_INST = False


def instrument():
    global _INST
    if _INST: return
    _INST = True
    from commander_sim import engine as E, ais as A
    from commander_sim.cards.impl import jodah as J

    def rec(g):
        return getattr(g, 'jrec', None)

    def me(g):
        return next((x for x in g.players if x.key == 'jodah'), None)

    def jodah_out(p):
        return any(m.cd is not None and m.cd.name == JODAH and not m.phased for m in p.perms)

    def legends(g, p):
        return sum(1 for m in p.perms if m.creature and not m.phased and J.legendary(g, m))

    o_cast = E.on_cast

    def on_cast(g, p, c):
        r = rec(g)
        if r is not None and p.key == 'jodah':
            cur = getattr(g, 'cur_cast', None)
            zone = cur[2] if cur is not None and cur[0] is c else '?'
            if c is p.cmd: zone = 'cmd'
            r['casts'].append((p.turns, c.name, zone, J.legend_card(c), jodah_out(p), c.cmc, g.active is p))
        return o_cast(g, p, c)
    E.on_cast = on_cast

    o_leave = E.leave

    def leave(g, m):
        r = rec(g)
        if r is not None and m.cd is not None and m.cd.name == JODAH and m in m.owner.perms and m.owner.key == 'jodah':
            ctx = [x[4:].strip() for x in (g.log or [])[-12:] if not x[4:].lstrip().startswith('[')][-4:]
            r['removed'].append({'turn': m.owner.turns, 'active': g.active.key if g.active else None,
                                 'step': getattr(g, 'step', None), 'ctx': ctx,
                                 'stack': [(it.card.name, it.controller.key) for it in g.stack if getattr(it, 'card', None) is not None]})
        return o_leave(g, m)
    E.leave = leave

    o_resp = E.ai_respond

    def ai_respond(g, q, item):
        r = rec(g)
        top = g.stack[-1] if g.stack else None
        if r is None or q.key != 'jodah' or top is None or top.controller is q or top.kind != 'spell' \
                or top.ctx.get('counter') is not None or q.key in top.passed:
            return o_resp(g, q, item)
        ctr = E.pick_counter(g, q, top.card)
        n0 = q.stats['counters_cast']
        res = o_resp(g, q, item)
        if ctr is not None:
            t = top.ctx.get('target')
            r['ctr_chances'].append((q.turns, top.card.name, top.controller.key, round(top.aff.get(q, top.imp), 1),
                                     q.stats['counters_cast'] > n0, ctr.name, getattr(t, 'name', None)))
        return res
    E.ai_respond = ai_respond

    def wrap(step):
        f = A.STEP_FN[step]

        def fn(g, p):
            r = rec(g)
            j = me(g)
            if r is not None and j is not None and j.alive:
                out = jodah_out(j)
                if r['out'] and not out: r['jodah_left'].append((j.turns, g.active.key == 'jodah', step))
                r['out'] = out
            if r is None or p.key != 'jodah':
                return f(g, p)
            if step == 'main1':
                gen, pips = E.cost_of(p, p.cmd)
                r['turns'].append({'t': p.turns, 'lands': len(p.lands), 'mana': E.total_mana(g, p),
                                   'jodah_out': jodah_out(p), 'cmd_zone': p.cmd_in_zone,
                                   'jodah_payable': p.cmd_in_zone and E.can_pay(g, p, gen, pips),
                                   'hand': len(p.hand), 'leg_hand': sum(1 for c in p.hand if J.legend_card(c)),
                                   'legends_bf': legends(g, p)})
            if step == 'combat':
                t = r['turns'][-1] if r['turns'] else {}
                cs = [m for m in p.perms if m.creature and not m.phased and not m.sick and not m.tapped]
                t['can_attack'] = len(cs); t['pow_ready'] = sum(E.epow(g, m) for m in cs)
                t['legends_bf'] = legends(g, p); t['jodah_out_c'] = jodah_out(p)
                bb = [m for m in p.perms if m.cd is not None and m.cd.name == 'Blackblade Reforged']
                t['blackblade'] = bool(bb); t['bb_on'] = bool(bb and bb[0].attached is not None and bb[0].attached in p.perms)
                t['kaldra_pieces'] = sum(1 for m in p.perms if m.cd is not None and m.cd.name in J.KALDRA)
                t['kaldra'] = any(m.cd is None and m.data and m.data.get('kaldra') for m in p.perms)
                n0 = len(g.log)
                res = f(g, p)
                dmg = 0; atk = 0
                for ln in g.log[n0:]:
                    mm = re.search(r'Jodah attacks .* with (\d+) creature\(s\): (\d+) damage', ln)
                    if mm: atk += int(mm.group(1)); dmg += int(mm.group(2))
                t['attackers'] = atk; t['dmg'] = dmg
                return res
            res = f(g, p)
            if step == 'main2' and p.alive and r['turns']:
                t = r['turns'][-1]
                left = E.total_mana(g, p)
                t['unspent'] = left
                stuck, held = [], []
                for c in p.hand:
                    if c.land: continue
                    gen, pips = E.cost_of(p, c)
                    if gen + len(pips) <= left and not E.can_pay(g, p, gen, pips): stuck.append(c.name)
                    elif E.can_pay(g, p, gen, pips) and not (c.instant or 'ctr' in c.tags): held.append(c.name)
                t['colour_stuck'] = stuck; t['castable_held'] = held
                t['hand_end'] = [c.name for c in p.hand]
                gen, pips = E.cost_of(p, p.cmd)
                t['jodah_held'] = p.cmd_in_zone and E.can_pay(g, p, gen, pips)
            return res
        A.STEP_FN[step] = fn
    for s in ('start', 'main1', 'combat', 'main2'): wrap(s)


def parse_log(lines):
    """cascades, tutor targets, Kaldra, what finished Jodah"""
    out = {'cascade_hits': [], 'cascade_miss': 0, 'cascade_declined': 0, 'tutors': [], 'kaldra_made': 0, 'death': None,
           'countered': []}
    last_tutor = None
    for i, ln in enumerate(lines):
        m = re.search(r'Jodah reveals (.+) \((\d+) other cards\): cast free', ln)
        if m: out['cascade_hits'].append(m.group(1)); continue
        if 'Jodah reveals' in ln and 'not cast' in ln: out['cascade_declined'] += 1; continue
        if 'Jodah exiles' in ln and 'finds no legendary' in ln: out['cascade_miss'] += 1; continue
        m = re.search(r'  Jodah casts (.+?)( ->|$| from suspend)', ln)
        if m and m.group(1) in TUTORS: last_tutor = (m.group(1), ln[:3].strip()); continue
        m = re.search(r'Jodah (?:tutors|puts|searches for) (.+?)(?: on top of their library)?$', ln) or \
            re.search(r"Opposition Agent: .* takes (.+) from Jodah's search", ln)
        if m and last_tutor is not None:
            out['tutors'].append((last_tutor[0], m.group(1), last_tutor[1])); last_tutor = None; continue
        if 'Jodah creates Kaldra' in ln: out['kaldra_made'] += 1
        if '*** Jodah is eliminated' in ln:
            nxt = lines[i + 1] if i + 1 < len(lines) else ''
            prev = [x for x in lines[max(0, i - 6):i] if not x.lstrip('R0123456789 ').startswith('[')]
            out['death'] = {'line': ln, 'next': nxt, 'prev': prev[-3:]}
    return out


def one(tier, seed, keys, ai):
    from commander_sim import poolmode, pools, ais
    seats = pools.draw_seats(seed, keys, 'jodah', 3)
    from commander_sim import engine as E
    g0 = ais.setup_pool_game(seed, [poolmode.seat_spec(s) for s in seats], trace=True)
    g0.jrec = {'casts': [], 'turns': [], 'ctr_chances': [], 'jodah_left': [], 'out': False, 'removed': []}
    g = ais._run_rounds(g0, g0.players, 20)
    p = next(x for x in g.players if x.key == 'jodah')
    r = g.jrec
    lg = parse_log(g.log)
    d = {'tier': tier, 'seed': seed, 'won': g.winner is p, 'turns': p.turns, 'wintype': g.wintype,
         'killer': p.killer.key if (not p.alive and p.killer is not None and p.killer is not p) else None,
         'winner': g.winner.key if g.winner else None, 'alive': p.alive, 'round': g.round,
         'mulls': p.stats['mulls'], 'land_miss': p.stats['land_miss'], 'drawn': p.stats['cards_drawn'],
         'seen': sorted(p.seen_names), 'cast_names': sorted(p.cast_names), 'plan': p.milestone.get('jodah'),
         'casts': r['casts'], 'removed': r['removed'], 'turn_rows': r['turns'], 'ctr_chances': r['ctr_chances'], 'jodah_left': r['jodah_left'],
         'life_end': p.life, 'opps': [x.key for x in g.players if x is not p], 'deck': list(p.deck_names)}
    d.update(lg)
    return d


def _work(args):
    tier, seeds, ai, swaps = args
    from commander_sim import poolmode, pools
    poolmode._setup('loose', ai, 1.0)
    instrument()
    keys = poolmode.pool_keys(tier)
    out = []
    for s in seeds:
        try: out.append(one(tier, s, keys, ai))
        except Exception as e: out.append({'tier': tier, 'seed': s, 'error': repr(e)[:200]})
    return out


def run(tiers, n, path, jobs=15, ai='adaptive', seed0=700000):
    tasks = []
    for t in tiers:
        seeds = list(range(seed0, seed0 + n))
        for i in range(0, n, 10): tasks.append((t, seeds[i:i + 10], ai, ()))
    with Pool(jobs) as pool:
        res = [x for part in pool.imap_unordered(_work, tasks) for x in part]
    json.dump(res, open(path, 'w'))
    print(f'{len(res)} games -> {path}')


def pct(a, b): return f'{100 * a / b:.0f}%' if b else '-'


def med(xs): return statistics.median(xs) if xs else '-'


def mean(xs): return f'{statistics.mean(xs):.2f}' if xs else '-'


def report(paths):
    G = [d for pth in paths for d in json.load(open(pth)) if 'error' not in d]
    from commander_sim import poolmode; poolmode._setup("loose", "adaptive", 1.0)
    tiers = sorted({d["tier"] for d in G})
    print(f'{len(G)} games, tiers {tiers}')
    groups = [(t, [d for d in G if d['tier'] == t]) for t in tiers] + [('all', G)]
    print('\n== Outcome and Jodah')
    print(f"{'tier':5s} {'n':>4s} {'win':>5s} {'cast':>5s} {'t(med)':>6s} {'by t5':>5s} {'by t6':>5s} {'casts':>5s} "
          f"{'left':>5s} {'plan':>5s} {'winP':>5s} {'win!P':>5s} {'loss t':>6s} {'mull':>5s}")
    for t, gs in groups:
        n = len(gs)
        first = [min((c[0] for c in d['casts'] if c[1] == JODAH), default=None) for d in gs]
        fc = [x for x in first if x is not None]
        ncast = [sum(1 for c in d['casts'] if c[1] == JODAH) for d in gs]
        left = [len(d['jodah_left']) for d in gs]
        pl = [d for d in gs if d['plan']]; npl = [d for d in gs if not d['plan']]
        lt = [d['turns'] for d in gs if not d['won']]
        print(f"{t:5s} {n:4d} {pct(sum(d['won'] for d in gs), n):>5s} {pct(len(fc), n):>5s} {med(fc):>6} "
              f"{pct(sum(1 for x in fc if x <= 5), n):>5s} {pct(sum(1 for x in fc if x <= 6), n):>5s} {mean(ncast):>5s} "
              f"{mean(left):>5s} {pct(len(pl), n):>5s} {pct(sum(d['won'] for d in pl), len(pl)):>5s} "
              f"{pct(sum(d['won'] for d in npl), len(npl)):>5s} {med(lt):>6} {mean([d['mulls'] for d in gs]):>5s}")
    print('  cast: Jodah cast at all; t(med): its first cast turn; casts: Jodah casts per game; left: times it left the '
          'battlefield; plan: a cascade happened; winP/win!P: win rate with/without a cascade')

    print('\n== Jodah castable but not cast (own turns with Jodah in the command zone and payable after main 2)')
    for t, gs in groups:
        rows = [r for d in gs for r in d['turn_rows'] if 'jodah_held' in r]
        held = [r for r in rows if r['jodah_held']]
        first_ok = []
        for d in gs:
            ok = next((r['t'] for r in d['turn_rows'] if r.get('jodah_payable')), None)
            cast = min((c[0] for c in d['casts'] if c[1] == JODAH), default=None)
            if ok is not None: first_ok.append((cast - ok) if cast is not None else None)
        late = [x for x in first_ok if x is not None]
        print(f"  {t:5s} turns where Jodah was payable and still in zone at end: {len(held)} "
              f"({mean([1 if r['jodah_held'] else 0 for r in rows])} of turns); first cast after first payable turn: "
              f"same turn {pct(sum(1 for x in late if x == 0), len(first_ok))}, later {pct(sum(1 for x in late if x > 0), len(first_ok))}, "
              f"never {pct(sum(1 for x in first_ok if x is None), len(first_ok))}")

    print('\n== Legends from hand: cast with Jodah out (cascade) vs before Jodah')
    for t, gs in groups:
        leg = [c for d in gs for c in d['casts'] if c[3] and c[2] == 'hand']
        withj = [c for c in leg if c[4]]
        # legends cast from hand on a turn where Jodah was in the command zone and payable at main 1
        payable = {(d['tier'], d['seed'], r['t']) for d in gs for r in d['turn_rows'] if r.get('jodah_payable')}
        pre = [c for d in gs for c in d['casts'] if c[3] and c[2] == 'hand' and not c[4] and (d['tier'], d['seed'], c[0]) in payable]
        n = len(gs)
        hits = [h for d in gs for h in d['cascade_hits']]
        print(f"  {t:5s} legends from hand/game {len(leg)/n:.2f}, with Jodah out {len(withj)/n:.2f} ({pct(len(withj), len(leg))}); "
              f"legend cast instead of a payable Jodah {len(pre)/n:.2f}/game; cascades/game {len(hits)/n:.2f} "
              f"(games with any {pct(sum(1 for d in gs if d['cascade_hits']), n)}), misses {sum(d['cascade_miss'] for d in gs)/n:.2f}/game")
    hits = collections.Counter(h for d in G for h in d['cascade_hits'])
    print('  cascade hits:', ', '.join(f'{k} {v}' for k, v in hits.most_common(20)))
    trig = collections.Counter(c[1] for d in G for c in d['casts'] if c[3] and c[2] == 'hand' and c[4])
    print('  legends cast from hand with Jodah out:', ', '.join(f'{k} {v}' for k, v in trig.most_common(15)))

    print('\n== Board and attacks (own combat steps)')
    for t, gs in groups:
        rows = [r for d in gs for r in d['turn_rows'] if 'attackers' in r]
        jo = [r for r in rows if r.get('jodah_out_c')]
        could = [r for r in rows if r['can_attack'] > 0]
        print(f"  {t:5s} combats {len(rows)}; with creatures ready {pct(len(could), len(rows))}, attacked in "
              f"{pct(sum(1 for r in could if r['attackers']), len(could))} of those; dmg/attack turn "
              f"{mean([r['dmg'] for r in could if r['attackers']])}; with Jodah out: legends {mean([r['legends_bf'] for r in jo])}, "
              f"ready power {mean([r['pow_ready'] for r in jo])}, attacked {pct(sum(1 for r in jo if r['attackers']), len(jo))}")
        bb = [r for r in rows if r.get('blackblade')]
        print(f"        Blackblade out {len(bb)} combats, equipped {pct(sum(1 for r in bb if r['bb_on']), len(bb))}; "
              f"Kaldra pieces 3/3 in {sum(1 for r in rows if r['kaldra_pieces'] == 3)} combats, Kaldra token in "
              f"{sum(1 for r in rows if r['kaldra'])}; games with Kaldra made {sum(1 for d in gs if d['kaldra_made'])}")
    dmg_total = collections.defaultdict(list)
    for d in G:
        dmg_total[d['tier']].append(sum(r.get('dmg', 0) for r in d['turn_rows']))
    print('  combat damage dealt per game:', ', '.join(f'{t} {mean(v)}' for t, v in sorted(dmg_total.items())))

    print('\n== Mana')
    for t, gs in groups:
        rows = [r for d in gs for r in d['turn_rows'] if 'unspent' in r]
        by = collections.defaultdict(list)
        for r in rows: by[min(r['t'], 9)].append(r)
        l4 = [r['lands'] for d in gs for r in d['turn_rows'] if r['t'] == 4]
        l6 = [r['lands'] for d in gs for r in d['turn_rows'] if r['t'] == 6]
        m4 = [r['mana'] for d in gs for r in d['turn_rows'] if r['t'] == 4]
        m6 = [r['mana'] for d in gs for r in d['turn_rows'] if r['t'] == 6]
        st = [r for r in rows if r['colour_stuck']]
        print(f"  {t:5s} lands t4 {mean(l4)} / t6 {mean(l6)}; mana t4 {mean(m4)} / t6 {mean(m6)}; land drops missed/game "
              f"{mean([d['land_miss'] for d in gs])}; unspent after main 2 (mean) {mean([r['unspent'] for r in rows])}; "
              f"turns with a card stuck on colours {pct(len(st), len(rows))}; turns ending with a castable sorcery-speed "
              f"card in hand {pct(sum(1 for r in rows if r['castable_held']), len(rows))}")
    stuck = collections.Counter(x for d in G for r in d['turn_rows'] for x in r.get('colour_stuck', ()))
    print('  stuck on colours (card-turns):', ', '.join(f'{k} {v}' for k, v in stuck.most_common(12)))
    heldc = collections.Counter(x for d in G for r in d['turn_rows'] for x in r.get('castable_held', ()))
    print('  castable but held at end of turn (card-turns):', ', '.join(f'{k} {v}' for k, v in heldc.most_common(15)))

    print('\n== Tutors')
    tt = collections.Counter((a, b) for d in G for a, b, _ in d['tutors'])
    byt = collections.defaultdict(collections.Counter)
    for (a, b), v in tt.items(): byt[a][b] += v
    cast_t = collections.Counter(c[1] for d in G for c in d['casts'] if c[1] in TUTORS)
    for a in TUTORS:
        print(f"  {a} (cast {cast_t[a]}): " + ', '.join(f'{k} {v}' for k, v in byt[a].most_common(8)))

    print('\n== Counterspells: opposing spells Jodah had a counter for')
    ch = [c for d in G for c in d['ctr_chances']]
    print(f"  chances {len(ch)}, countered {pct(sum(1 for c in ch if c[4]), len(ch))}")
    cn = collections.Counter(c[1] for c in ch if c[4])
    print('  countered:', ', '.join(f'{k} {v}' for k, v in cn.most_common(15)))
    ln = collections.Counter(c[1] for c in ch if not c[4])
    print('  let through:', ', '.join(f'{k} {v}' for k, v in ln.most_common(15)))
    by_ctr = collections.Counter(c[5] for c in ch if c[4])
    print('  counters used:', ', '.join(f'{k} {v}' for k, v in by_ctr.most_common()))

    print('\n== Mulligans')
    for t, gs in groups:
        mc = collections.Counter(d['mulls'] for d in gs)
        print(f"  {t:5s} " + ', '.join(f"{k} mulls {pct(v, len(gs))} (win {pct(sum(d['won'] for d in gs if d['mulls'] == k), v)})"
                                     for k, v in sorted(mc.items())))

    print('\n== Cards: seen vs cast (all tiers), and hand at the end of own turns')
    seen = collections.Counter(n for d in G for n in d['seen'])
    cast = collections.Counter(n for d in G for n in d['seen'] if n in d['cast_names'])
    endh = collections.Counter(x for d in G for r in d['turn_rows'] for x in r.get('hand_end', ()))
    rows = []
    for n, s in seen.items():
        from commander_sim.engine import DB
        if n in DB and DB[n].land: continue
        rows.append((cast[n] / s, n, s, endh[n]))
    rows.sort()
    for f, n, s, e in rows[:25]:
        print(f'  {n:34s} seen {s:4d}  cast when seen {100*f:3.0f}%  end-of-turn card-turns in hand {e}')
    print('\n== How Jodah dies, and how Jodah (the card) leaves the battlefield')
    for t, gs in groups:
        dead = [d for d in gs if d['death']]
        comb = sum(1 for d in dead if 'attacks Jodah' in d['death']['next'])
        out_last = sum(1 for d in dead if d['turn_rows'] and d['turn_rows'][-1].get('jodah_out'))
        n = len(gs)
        rm = collections.Counter()
        for d in gs:
            for r in d.get('removed', ()):
                last = r['ctx'][-1] if r['ctx'] else ''
                wipe = any(k in x for x in r['ctx'] for k in ('wipe', 'Verdict', 'Wrath', 'Damnation', 'Sunfall', 'Deed'))
                rm['targeted' if ('is removed' in last or 'exiles Jodah' in last) else 'wipe' if wipe
                   else 'combat' if r['step'] == 'combat' else 'other'] += 1
        print(f"  {t:5s} eliminated {len(dead)} of {n}: by combat damage {pct(comb, len(dead))}, median turn "
              f"{med([d['turns'] for d in dead])}, Jodah out on its last turn {pct(out_last, len(dead))}; Jodah removed "
              f"{sum(rm.values()) / n:.2f}/game (" + ', '.join(f'{k} {v / n:.2f}' for k, v in rm.most_common()) + ')')
    tj = [c for d in G for c in d['ctr_chances'] if len(c) > 6 and c[6] == JODAH]
    print(f"  removal aimed at Jodah while holding a counter: {len(tj)} times in {len(G)} games, countered {sum(c[4] for c in tj)}")
    by_kill = collections.Counter(d['killer'] for d in G if not d['won'])
    print('\n== Killers:', ', '.join(f'{k} {v}' for k, v in by_kill.most_common(12)))


def show(tier, seed):
    from commander_sim import poolmode, pools, ais
    poolmode._setup('loose', 'adaptive', 1.0)
    keys = poolmode.pool_keys(tier)
    seats = pools.draw_seats(seed, keys, 'jodah', 3)
    g = ais.play_pool_game(seed, [poolmode.seat_spec(s) for s in seats], trace=True)
    print('\n'.join(g.log))
    p = next(x for x in g.players if x.key == 'jodah')
    print('WINNER', g.winner.key if g.winner else None, g.wintype, dict(p.stats))


if __name__ == '__main__':
    a = sys.argv[1:]
    if a[0] == 'run':
        kw = {}
        if '--jobs' in a: kw['jobs'] = int(a[a.index('--jobs') + 1])
        if '--ai' in a: kw['ai'] = a[a.index('--ai') + 1]
        if '--seed0' in a: kw['seed0'] = int(a[a.index('--seed0') + 1])
        run(a[1].split(','), int(a[2]), a[3], **kw)
    elif a[0] == 'report': report(a[1:])
    elif a[0] == 'log': show(a[1], int(a[2]))
