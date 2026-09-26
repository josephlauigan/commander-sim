"""Pool mode: test one deck against a fixed field of outside decks (decklists/pool/).

Each game seats the deck under test plus three opponents drawn without replacement from one tier's five
decks, in a random seat order (pools.draw_seats). Opponent draws, seat order, every seat's opening shuffle
and mulligans come from random streams that depend only on the seed and the deck keys, so two versions of
a deck (a --swap) face identical opponents and identical opponent draws: paired, common-random-number runs.

    python3 -m commander_sim --deck seph --pool t3 --games 10000
    python3 -m commander_sim --deck seph --pool t3 --swap "Persist=>Dread Return"      # paired A/B
    python3 -m commander_sim --all-decks --pool all --games 10000 --jobs 24            # deck x tier matrix
    python3 -m commander_sim --deck seph --pool t3 --analyze
    python3 -m commander_sim --calibrate within|ordering --games 10000                 # pool balance checks
"""
import math, sys, time
from collections import Counter, defaultdict
from commander_sim import engine, ais, pools
from commander_sim import compare as C
from commander_sim.decks import DECKS

MINE = ('seph', 'veyran', 'sauron')
TIER_LABEL = {'t1': 'Tier 1 (High B2 / Low B3)', 't2': 'Tier 2 (Mid B3)', 't3': 'Tier 3 (High B3)',
              't4': 'Tier 4 (Low B4)', 't5': 'Tier 5 (High B4)'}


def seat_spec(key, cards=None):
    if key in DECKS: return (key, cards if cards is not None else DECKS[key], ais.CMDS[key])
    d = pools.by_key()[key]
    return (key, cards if cards is not None else d.cards, d.commander)


def pool_keys(tier):
    return [d.key for d in pools.load_pool(tier)]


def play(seed, me, my_cards, opp_keys, k=3):
    seats = pools.draw_seats(seed, opp_keys, me, k)
    return ais.play_pool_game(seed, [seat_spec(s, my_cards if s == me else None) for s in seats])


def _setup(profile, ai, temp, extra_cards=()):
    engine.set_profile(profile); C.set_ai(ai, temp)
    pools.register()
    if extra_cards:
        from commander_sim.cards import sources as _cards; _cards.ensure_cards(list(extra_cards), verbose=False)


# ------------------------------------------------------------------ one deck against a tier
def _chunk(me, my_cards, opp_keys, seeds, profile, ai='adaptive', temp=1.0):
    _setup(profile, ai, temp, my_cards or ())
    R = {'n': 0, 'win': 0, 'win_turns': 0, 'loss_turns': 0, 'plan': 0, 'seated': Counter(), 'opp_won': Counter(),
         'killed_me': Counter(), 'by_seed': {}, 'S1': Counter(), 'S2': Counter(), 'N': Counter(), 'errs': Counter(),
         'wintype': Counter()}
    for s in seeds:
        try:
            g = play(s, me, my_cards, opp_keys)
        except Exception as e:
            R['errs']['__errors__'] += 1
            R['errs'][f'__err__{s}__{e.__class__.__name__}: {str(e)[:60]}'] += 1
            continue
        p = next(x for x in g.players if x.key == me)
        won = g.winner is p
        R['n'] += 1; R['win'] += won; R['by_seed'][s] = won
        if won: R['win_turns'] += p.turns; R['wintype'][g.wintype or 'damage'] += 1
        else: R['loss_turns'] += p.turns
        for q in g.players:
            if q is p: continue
            R['seated'][q.key] += 1; R['opp_won'][q.key] += g.winner is q
        if not p.alive and p.killer is not None and p.killer is not p: R['killed_me'][p.killer.key] += 1
        if me in MINE:
            for k, x in C.axis_values(g, p, me).items():
                R['S1'][k] += x; R['S2'][k] += x * x; R['N'][k] += 1
            if C.plan_turn(p, me): R['plan'] += 1
    return R


def _merge(parts):
    R = parts[0]
    for r in parts[1:]:
        for k, v in r.items():
            if isinstance(v, (Counter, dict)): R[k].update(v)
            else: R[k] += v
    return R


def run(me, my_cards, opp_keys, profile, n, seed0=500000):
    seeds = list(range(seed0, seed0 + n))
    parts = C._run_chunks(_chunk, [(me, my_cards, opp_keys, s, profile, C.AI, C.TEMP) for s in C._chunks(seeds)])
    R = _merge(parts)
    C.ERR.update(R.pop('errs'))
    R['profile'] = profile
    R['axes'] = {k: {'mean': R['S1'][k] / R['N'][k], 'var': max(0.0, R['S2'][k] / R['N'][k] - (R['S1'][k] / R['N'][k]) ** 2),
                     'n': R['N'][k]} for k in R['N']}
    return R


def wilson(k, n, z=1.96):
    if n == 0: return 0.0, 0.0
    p = k / n; d = 1 + z * z / n
    c = (p + z * z / (2 * n)) / d; h = z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n)) / d
    return c - h, c + h


def name_of(key):
    if key in DECKS: return engine.NAME(type('P', (), {'key': key})())
    return pools.short_name(pools.by_key()[key])


def print_run(me, tier, R, label=''):
    n = R['n']; w = R['win']
    lo, hi = wilson(w, n)
    print(f"\n=== {name_of(me)} vs {TIER_LABEL.get(tier, tier)}{label} [{R['profile']}, {C.AI} AI, {n} games] ===")
    print(f"Win rate {100*w/n:5.1f}%   95% CI {100*lo:.1f}-{100*hi:.1f}%   (even share 25%: {100*(w/n-0.25):+.1f} pts)")
    wt = f"{R['win_turns']/w:.1f}" if w else 'n/a'
    lt = f"{R['loss_turns']/(n-w):.1f}" if n - w else 'n/a'
    print(f"Average turn (your own turn count) of wins {wt}, of losses {lt}")
    if me in MINE: print(f"Plan online ({C.PLAN[me]}): {100*R['plan']/n:.1f}% of games")
    if R['wintype']:
        print('Wins by: ' + ', '.join(f'{k} {100*v/w:.0f}%' for k, v in R['wintype'].most_common()))
    print(f"  {'Opponent':42s} {'seated':>7s} {'won when seated':>16s} {'eliminated you':>15s}")
    for k in sorted(R['seated'], key=lambda k: -R['seated'][k]):
        s = R['seated'][k]
        print(f"  {name_of(k) + ' (' + k + ')':42.42s} {100*s/n:6.1f}% {100*R['opp_won'][k]/s:15.1f}% {100*R['killed_me'][k]/n:14.1f}%")
    print("  ('eliminated you' is the share of all games in which that opponent dealt the finishing blow)")


# ------------------------------------------------------------------ paired A/B in pool mode
def paired_delta(b, v):
    """win-rate change and its standard error from per-seed pairs (common random numbers)"""
    seeds = sorted(set(b['by_seed']) & set(v['by_seed']))
    d = [v['by_seed'][s] - b['by_seed'][s] for s in seeds]
    n = len(d)
    if not n: return 0.0, 0.0, 0
    m = sum(d) / n
    var = sum((x - m) ** 2 for x in d) / max(1, n - 1)
    return m, math.sqrt(var / n), n


def print_ab(me, tier, pairs):
    print(f'\n=== {name_of(me)} vs {TIER_LABEL.get(tier, tier)}: baseline vs variant (paired, same opponents and draws) ===')
    deltas, ses = [], []
    for b, v in pairs:
        d, se, n = paired_delta(b, v)
        deltas.append(d); ses.append(se)
        print(f"[{b['profile']:12s}] win {100*b['win']/b['n']:5.1f}% -> {100*v['win']/v['n']:5.1f}%  "
              f"(delta {100*d:+.1f} pts, noise ±{200*se:.1f}, paired)   n={n}")
        opp = sorted(b['seated'], key=lambda k: -b['seated'][k])
        print(f"{'':15s}eliminated by (variant): " + '  '.join(f"{name_of(k)} {100*v['killed_me'][k]/v['n']:.1f}%" for k in opp))
    print('VERDICT:', C.verdict(deltas, ses, [b['profile'] for b, _ in pairs]))
    if C.VERBOSE: C.axis_report(me, pairs)


# ------------------------------------------------------------------ --analyze in pool mode
def _analyze_chunk(me, my_cards, opp_keys, seeds, profile, ai='adaptive', temp=1.0):
    _setup(profile, ai, temp, my_cards or ())
    R = C.analysis_record(); errs = Counter()
    for s in seeds:
        try:
            g = play(s, me, my_cards, opp_keys)
        except Exception as e:
            errs['__errors__'] += 1; errs[f'__err__{s}__{e.__class__.__name__}: {str(e)[:60]}'] += 1; continue
        C.analysis_add(R, g, me)
    R['errs'] = errs
    return R


def analyze(me, my_cards, opp_keys, profile, n, seed0=500000):
    seeds = list(range(seed0, seed0 + n))
    res = C._run_chunks(_analyze_chunk, [(me, my_cards, opp_keys, s, profile, C.AI, C.TEMP) for s in C._chunks(seeds)])
    R = res[0]
    for r in res[1:]:
        for k, v in r.items():
            if isinstance(v, Counter): R[k].update(v)
            else: R[k] += v
    C.ERR.update(R.pop('errs', Counter()))
    return R


# ------------------------------------------------------------------ calibration
def _calib_chunk(me, opp_keys, k, seeds, profile, ai='adaptive', temp=1.0):
    """me=None: every seat drawn from opp_keys (within-tier balance); else me + k from opp_keys"""
    _setup(profile, ai, temp)
    seated, won, errs, rounds, timeouts = Counter(), Counter(), Counter(), 0, 0
    for s in seeds:
        try:
            g = play(s, me, None, opp_keys, k)
        except Exception as e:
            errs['__errors__'] += 1; errs[f'__err__{s}__{e.__class__.__name__}: {str(e)[:60]}'] += 1; continue
        rounds += g.round; timeouts += g.wintype == 'timeout'
        for q in g.players:
            seated[q.key] += 1; won[q.key] += g.winner is q
    return {'seated': seated, 'won': won, 'errs': errs, 'rounds': rounds, 'timeouts': timeouts, 'n': len(seeds)}


def calibrate(me, opp_keys, k, profile, n, seed0=500000):
    seeds = list(range(seed0, seed0 + n))
    parts = C._run_chunks(_calib_chunk, [(me, opp_keys, k, s, profile, C.AI, C.TEMP) for s in C._chunks(seeds)])
    R = _merge(parts)
    C.ERR.update(R.pop('errs'))
    return R


def within_tier(tier, profile, n, seed0=500000):
    """four of the tier's five decks per game; each deck's win rate when seated"""
    R = calibrate(None, pool_keys(tier), 4, profile, n, seed0)
    return {k: (R['won'][k], R['seated'][k]) for k in pool_keys(tier)}, R


def tier_ordering(tier, profile, n, seed0=500000):
    """each deck of `tier` alone in pods of three decks from the tier below"""
    below = 't' + str(int(tier[1]) - 1)
    out = {}
    for d in pool_keys(tier):
        R = calibrate(d, pool_keys(below), 3, profile, n, seed0)
        out[d] = (R['won'][d], R['seated'][d])
    return out


# ------------------------------------------------------------------ command line
def main(a, swaps, base, var):
    """called from compare.main when --pool or --calibrate is given"""
    C.PROG.on = not a.quiet
    profiles = [a.profile] if a.profile else a.profiles.split(',')
    n = a.games or a.n
    seed0 = a.seed
    tiers = list(pools.TIERS) if a.pool == 'all' else [a.pool] if a.pool else []
    pools.register()
    t0 = time.time()
    if a.calibrate:
        run_calibration(a.calibrate, profiles, n, seed0)
    elif a.all_decks or (len(tiers) > 1 and not swaps and not a.analyze):
        decks = list(MINE) if a.all_decks else [a.deck]
        matrix(decks, tiers, profiles, n, seed0)
    elif a.analyze:
        cards = var if swaps else base
        if swaps: print('Analyzing the variant list:', '; '.join(f'{o} -> {i}' for o, i in swaps))
        C.PROG.plan(n * len(profiles) * len(tiers))
        for tier in tiers:
            for prof in profiles:
                C.PROG.label = f'analyze {tier} / {prof}'
                R = analyze(a.deck, cards, pool_keys(tier), prof, n, seed0)
                print(f'\n######## vs {TIER_LABEL[tier]} ########')
                C.print_analysis(a.deck, cards, R, prof)
                print_opponent_names(tier)
    elif swaps:
        C.PROG.plan(n * len(profiles) * len(tiers) * 2)
        for tier in tiers:
            pairs = []
            for prof in profiles:
                C.PROG.label = f'{tier} current / {prof}'; b = run(a.deck, base, pool_keys(tier), prof, n, seed0)
                C.PROG.label = f'{tier} swaps / {prof}'; v = run(a.deck, var, pool_keys(tier), prof, n, seed0)
                pairs.append((b, v))
            print('Swaps:', '; '.join(f'{o} -> {i}' for o, i in swaps))
            print_ab(a.deck, tier, pairs)
    else:
        C.PROG.plan(n * len(profiles) * len(tiers))
        for tier in tiers:
            for prof in profiles:
                C.PROG.label = f'{tier} / {prof}'
                print_run(a.deck, tier, run(a.deck, base, pool_keys(tier), prof, n, seed0))
    print(f'\n({time.time()-t0:.0f}s total)')


def print_opponent_names(tier):
    print('  Opponent keys: ' + ', '.join(f'{k} = {name_of(k)}' for k in pool_keys(tier)))


def matrix(decks, tiers, profiles, n, seed0, out=None):
    C.PROG.plan(n * len(profiles) * len(tiers) * len(decks))
    cells = {}
    for prof in profiles:
        for d in decks:
            for tier in tiers:
                C.PROG.label = f'{d} {tier} / {prof}'
                R = run(d, None, pool_keys(tier), prof, n, seed0)
                cells[(prof, d, tier)] = R
                if C.PROG.on:
                    lo, hi = wilson(R['win'], R['n'])
                    print(f"  cell {d} vs {tier} [{prof}]: {100*R['win']/R['n']:5.1f}% ({100*lo:.1f}-{100*hi:.1f})",
                          file=sys.stderr, flush=True)
    for prof in profiles:
        print(f'\n=== Win rate by deck and tier [{prof} profile, {C.AI} AI, n={n} per cell; even share 25%] ===')
        print(f"  {'deck':10s} " + ' '.join(f'{t:>18s}' for t in tiers))
        for d in decks:
            row = []
            for t in tiers:
                R = cells[(prof, d, t)]; lo, hi = wilson(R['win'], R['n'])
                row.append(f"{100*R['win']/R['n']:5.1f}% ({100*lo:4.1f}-{100*hi:4.1f})")
            print(f"  {d:10s} " + ' '.join(f'{x:>18s}' for x in row))
    return cells


def run_calibration(which, profiles, n, seed0):
    if which in ('within', 'all'):
        C.PROG.plan(n * len(profiles) * len(pools.TIERS))
        for prof in profiles:
            print(f'\n=== Within-tier balance [{prof}, n={n} games per tier; each deck is seated in ~80%] ===')
            for tier in pools.TIERS:
                C.PROG.label = f'within {tier} / {prof}'
                res, R = within_tier(tier, prof, n, seed0)
                print(f"  {TIER_LABEL[tier]}: avg game {R['rounds']/max(1,R['n']):.1f} rounds, timeouts {100*R['timeouts']/max(1,R['n']):.1f}%")
                for k, (w, s) in res.items():
                    lo, hi = wilson(w, s)
                    flag = '  HIGH' if w / s > 0.35 else '  LOW' if w / s < 0.15 else ''
                    print(f"    {name_of(k):28s} {100*w/s:5.1f}% ({100*lo:.1f}-{100*hi:.1f}){flag}")
    if which in ('ordering', 'all'):
        C.PROG.plan(n * len(profiles) * 20)
        for prof in profiles:
            print(f'\n=== Tier ordering: each deck alone against three decks of the tier below [{prof}, n={n}] ===')
            for tier in pools.TIERS[1:]:
                C.PROG.label = f'ordering {tier} / {prof}'
                res = tier_ordering(tier, prof, n, seed0)
                tot_w = sum(w for w, s in res.values()); tot_s = sum(s for w, s in res.values())
                print(f"  {TIER_LABEL[tier]} into {TIER_LABEL['t' + str(int(tier[1]) - 1)]}: {100*tot_w/tot_s:.1f}% overall")
                for k, (w, s) in res.items():
                    lo, hi = wilson(w, s)
                    flag = '  (not above 25%)' if hi < 0.25 or w / s <= 0.25 else ''
                    print(f"    {name_of(k):28s} {100*w/s:5.1f}% ({100*lo:.1f}-{100*hi:.1f}){flag}")


def trace(a, cards):
    """--trace N with --pool: play-by-play of one pool game (seed = --seed + N)"""
    pools.register()
    prof = a.profile or a.profiles.split(',')[0]
    _setup(prof, C.AI, C.TEMP, cards or ())
    tier = a.pool if a.pool != 'all' else 't1'
    seed = a.seed + a.trace
    seats = pools.draw_seats(seed, pool_keys(tier), a.deck, 3)
    g = ais.play_pool_game(seed, [seat_spec(s, cards if s == a.deck else None) for s in seats], trace=True)
    print(f'Play-by-play of pool game {a.trace} (seed {seed}): {name_of(a.deck)} vs {TIER_LABEL[tier]}, {prof} profile, {C.AI} AI')
    print('\n'.join(g.log))
    print(f"Winner: {name_of(g.winner.key) if g.winner else 'none'} ({g.wintype})")


def swap_in_place(cards, swaps):
    """apply swaps keeping every other card at its list position, so both versions of the deck get the same
    library permutation from the same seed (compare.apply_swaps appends instead; old mode keeps that)"""
    out = list(cards)
    for o, i in swaps:
        out[out.index(o)] = i
    return out
