"""Compare a deck change under both AI interaction profiles.

Examples
  python3 compare.py --deck seph --swap "Farewell=>Damnation"
  python3 compare.py --deck seph --swap "Persist=>Dread Return" --swap "Swamp=>Island" --n 3000
  python3 compare.py --deck seph --swap "A=>B" --swap "C=>D" --ablate      # each swap alone too
  python3 compare.py --deck najeela --swap "A=>B" --goldfish               # add solo speed metrics
Long runs (to stay under a tool timeout), one piece at a time, then merge:
  python3 compare.py --deck seph --swap "A=>B" --n 10000 --part baseline --profile loose --out b_loose.json
  python3 compare.py --report b_loose.json v_loose.json b_cons.json v_cons.json
Baseline = the deck list currently in the project .md file.  Same seeds are used for
baseline and variant so the comparison is paired.
"""
import argparse, json, math, os, sys, time
if '--dsl-all' in sys.argv: os.environ['SIM_DSL_ALL'] = '1'      # must be set before the decks load
from collections import Counter
import engine, ais
from engine import DB
from decks import DECKS

KEYS = ('seph', 'veyran', 'sauron', 'najeela')


def apply_swaps(deck, swaps):
    d = list(DECKS[deck])
    for out, inn in swaps:
        if out not in d: sys.exit(f"'{out}' is not in the current {deck} list")
        d.remove(out); d.append(inn)
    return d


def parse_swaps(raw):
    swaps = []
    for s in raw or []:
        if '=>' not in s: sys.exit(f"swap must look like 'Out Card=>In Card': {s}")
        a, b = [x.strip() for x in s.split('=>', 1)]
        swaps.append((a, b))
    import cards
    added, missing = cards.ensure_cards([b for _, b in swaps])
    if missing:
        sys.exit('Not found on Scryfall (check the spelling): ' + ', '.join(missing))
    auto = [DB[b] for _, b in swaps if DB[b].source != 'manual']
    return swaps, auto


def check_identity(deck, swaps):
    import cards
    from engine import IDENT
    bad = [b for _, b in swaps if not cards.identity(DB[b]) <= set(IDENT[deck])]
    if bad: sys.exit(f"Outside {deck}'s color identity ({IDENT[deck]}): " + ', '.join(bad))


def show_auto(auto):
    import cards
    if auto:
        print('Cards modeled from Scryfall text or cards_dsl.json (check the abilities below):')
        for cd in auto: print('   ' + cards.describe(cd))


# ------------------------------------------------------------------ strength axes
# (key, label, axis, which direction is better, applies to)
METRICS = [
    ('win',           'Win rate',                               'Outcome',              'up',   None),
    ('survived',      'Rounds survived (or game length)',       'Outcome',              'up',   None),
    ('mulls',         'Extra mulligans per game',               'Mana & consistency',   'down', None),
    ('land_miss',     'Missed land drops, turns 1-5',           'Mana & consistency',   'down', None),
    ('mana_T4',       'Mana available on your turn 4',          'Mana & consistency',   'up',   None),
    ('mana_T6',       'Mana available on your turn 6',          'Mana & consistency',   'up',   None),
    ('plan_reached',  'Games where the game plan came online',  'Speed',                'up',   None),
    ('plan_turn',     'Turn the game plan came online',         'Speed',                'down', None),
    ('cards_drawn',   'Cards drawn per game (incl. draw step)', 'Card flow',            'up',   None),
    ('tutored',       'Tutors resolved per game',               'Card flow',            'up',   None),
    ('spells_cast',   'Spells cast per game',                   'Card flow',            'up',   None),
    ('counters',      'Counterspells cast',                     'Interaction',          'up',   None),
    ('removal',       'Spot removal cast',                      'Interaction',          'up',   None),
    ('wipes',         'Board wipes cast',                       'Interaction',          'up',   None),
    ('protection',    'Protection used (HI/sac/Tidebinder/counter-war)', 'Resilience', 'up', None),
    ('threats_lost',  'Your key permanents removed by opponents', 'Resilience',         'down', None),
    ('countered',     'Your spells countered',                  'Resilience',           'down', None),
    ('recovered',     'Removed bombs brought back',             'Resilience',           'up',   'seph'),
    ('bombs_landed',  'Bombs put on the battlefield',           'Threat & pressure',    'up',   'seph'),
    ('dmg_dealt',     'Damage dealt to opponents per game',     'Threat & pressure',    'up',   None),
    ('dmg_combat',    '  ...of which combat damage',            'Threat & pressure',    'up',   None),
    ('dmg_burn',      '  ...of which magecraft / burn',         'Threat & pressure',    'up',   None),
    ('dmg_drain',     '  ...of which drain triggers',           'Threat & pressure',    'up',   None),
    ('dmg_aether',    '  ...of which Aetherflux shots',         'Threat & pressure',    'up',   None),
    ('dmg_triggers',  '  ...of which other triggers',           'Threat & pressure',    'up',   None),
    ('combo_rate',    'Games with a combo attempt',             'Speed',                'up',   'combo'),
    ('aether_shots',  'Aetherflux shots per game',              'Threat & pressure',    'up',   'veyran'),
]
PLAN = {'seph': 'first 6+ power bomb on the battlefield', 'veyran': 'Veyran + a magecraft payoff on the battlefield',
        'sauron': 'Sword + Assault combo attempted', 'najeela': 'first WUBRG extra combat'}
AXIS_NOTE = {
    'Outcome': 'the bottom line; everything else explains why it moved.',
    'Mana & consistency': 'can the deck cast its spells on time? Missed land drops and low mana stall everything else.',
    'Speed': 'how fast the deck does the thing it is built to do.',
    'Card flow': 'how many resources the deck sees; more cards seen means more options and fewer dead draws.',
    'Interaction': 'how often the deck answers what opponents do.',
    'Resilience': 'how well the plan survives opposing interaction.',
    'Threat & pressure': 'how much the deck pushes the game toward a win.',
}


def plan_turn(me, deck):
    if deck == 'seph': return me.first_bomb
    if deck == 'veyran': return me.milestone.get('engine')
    if deck == 'najeela': return me.milestone.get('act')
    return me.milestone.get('combo')


def axis_values(g, me, deck):
    s = me.stats
    v = {'win': 1.0 if g.winner is me else 0.0,
         'survived': s.get('elim_round', g.round),
         'mulls': s['mulls'], 'land_miss': s['land_miss'],
         'cards_drawn': s['cards_drawn'], 'tutored': s['tutored'], 'spells_cast': s['spells_cast'],
         'counters': s['counters_cast'], 'removal': s['removal_cast'], 'wipes': s['wipes_cast'],
         'protection': s['hi_used'] + s['sac_saves'] + s['counterwar_won'] + sum(x for k, x in s.items() if k.startswith('tide_')),
         'threats_lost': s['threats_lost'], 'countered': s['spells_countered'],
         'recovered': s['bomb_recovered'], 'bombs_landed': s['bombs_landed'], 'dmg_dealt': s['dmg_dealt']}
    if s.get('had_T4'): v['mana_T4'] = s['mana_T4']
    if s.get('had_T6'): v['mana_T6'] = s['mana_T6']
    for k in ('combat', 'burn', 'drain', 'aether', 'triggers'): v['dmg_' + k] = s['dmgk_' + k]
    v['combo_rate'] = 1.0 if s['combo_attempt'] else 0.0
    v['aether_shots'] = s['aether_shots']
    plan = plan_turn(me, deck)
    v['plan_reached'] = 1.0 if plan else 0.0
    if plan: v['plan_turn'] = plan
    return v


def _safe_game(seed, decks, counter):
    """play one game; if the engine hits an internal error, record the seed and skip the game"""
    try:
        return ais.play_game(seed, decks)
    except Exception as e:
        counter['__errors__'] += 1
        counter[f'__err__{seed}__{e.__class__.__name__}: {str(e)[:60]}'] += 1
        return None


def _pod_chunk(deck, cards, profile, seeds, ai='adaptive', temp=1.0):
    engine.set_profile(profile); set_ai(ai, temp)
    import cards as _cards; _cards.ensure_cards(cards, verbose=False)     # worker processes: load auto-tagged cards from cache
    decks = dict(DECKS); decks[deck] = cards
    W, killer = Counter(), Counter()
    S1, S2, N = Counter(), Counter(), Counter()
    for s in seeds:
        g = _safe_game(s, decks, W)
        if g is None: continue
        W[g.winner.key if g.winner else 'none'] += 1
        me = next(p for p in g.players if p.key == deck)
        if not me.alive: killer[me.killer.key if me.killer else 'none'] += 1
        for k, x in axis_values(g, me, deck).items():
            S1[k] += x; S2[k] += x * x; N[k] += 1
    return W, killer, S1, S2, N


JOBS = 1
VERBOSE = True
_POOL = None


class Progress:
    """One progress bar for the whole command: games done / planned, elapsed, estimated time left."""
    def __init__(s):
        s.total = 0; s.done = 0; s.t0 = None; s.label = ''; s.on = True; s.last = 0.0
        s.tty = sys.stderr.isatty(); s.next_pct = 10

    def plan(s, games):
        s.total += games

    def update(s, k):
        if s.t0 is None: s.t0 = time.time()
        s.done += k
        if not s.on or not s.total: return
        now = time.time()
        pct = 100.0 * s.done / s.total
        el = now - s.t0
        left = el / s.done * (s.total - s.done) if s.done else 0
        fmt = lambda x: f'{int(x // 60)}m{int(x % 60):02d}s'
        if s.tty:
            if now - s.last < 0.25 and s.done < s.total: return
            s.last = now
            bar = '#' * int(pct / 4) + '-' * (25 - int(pct / 4))
            sys.stderr.write(f'\r[{bar}] {pct:5.1f}%  {s.done:,}/{s.total:,} games  '
                             f'{fmt(el)} elapsed, ~{fmt(left)} left  {s.label[:28]:28s}')
            if s.done >= s.total: sys.stderr.write('\n')
            sys.stderr.flush()
        elif pct >= s.next_pct or s.done >= s.total:           # redirected output: a line every 10%
            s.next_pct = int(pct // 10) * 10 + 10
            print(f'  progress {pct:5.1f}%  {fmt(el)} elapsed, ~{fmt(left)} left', file=sys.stderr, flush=True)


PROG = Progress()
ERR = Counter()


def report_errors():
    n = ERR.get('__errors__', 0)
    if not n: return
    seeds = [k.split('__')[2] + '  ' + k.split('__', 3)[3] for k in ERR if k.startswith('__err__')][:5]
    print(f'\nWARNING: {n} game(s) hit an internal simulator error and were skipped (results use the rest).')
    print('  Replay one to see what happened:  python3 compare.py --deck <deck> [--swap ...] --trace <seed - 500000>')
    for s in seeds: print('   seed', s)
    print('  Please send me that output so I can fix the cause.')


def _chunks(seeds):
    """small chunks so progress updates often; results are identical to one big chunk"""
    size = max(25, min(250, len(seeds) // (max(1, JOBS) * 8) or 25))
    return [seeds[i:i + size] for i in range(0, len(seeds), size)]


def _run_chunks(fn, args_for):
    global _POOL
    seeds_parts = args_for
    if JOBS > 1:
        if _POOL is None:
            from multiprocessing import Pool
            _POOL = Pool(JOBS)
        out = []
        for r in _POOL.imap_unordered(_star, [(fn,) + a for a in seeds_parts]):
            out.append(r[1]); PROG.update(r[0])
        return out
    out = []
    for a in seeds_parts:
        out.append(fn(*a)); PROG.update(len(a[3]))
    return out


def _star(a):
    fn, rest = a[0], a[1:]
    return len(rest[3]), fn(*rest)
AI, TEMP = 'adaptive', 1.0


def set_ai(mode, temp):
    engine.AI_MODE = mode
    import brain
    brain.TEMP_SCALE = temp


def pod(deck, cards, profile, n, seed0=500000):
    seeds = list(range(seed0, seed0 + n))
    res = _run_chunks(_pod_chunk, [(deck, cards, profile, s, AI, TEMP) for s in _chunks(seeds)])
    W, killer, S1, S2, N = Counter(), Counter(), Counter(), Counter(), Counter()
    for w, k, s1, s2, nn in res:
        W.update(w); killer.update(k); S1.update(s1); S2.update(s2); N.update(nn)
    axes = {}
    ERR.update({k: v for k, v in W.items() if k.startswith('__err')})
    for k in [k for k in W if k.startswith('__err')]: del W[k]
    n_ok = sum(W.values())
    for k in N:
        m = S1[k] / N[k]
        var = max(0.0, S2[k] / N[k] - m * m)
        axes[k] = {'mean': m, 'var': var, 'n': N[k]}
    return {'profile': profile, 'n': n_ok, 'win': dict(W), 'killed_by': dict(killer), 'axes': axes}


def axis_report(deck, pairs):
    """Per-axis table: baseline -> variant for each profile, * marks a change beyond noise (2 SE)."""
    profs = [b['profile'] for b, _ in pairs]
    print(f"\n--- What changed, by strength axis ({deck}) ---")
    print("  Each cell: current -> new, change, and the noise band (±2 standard errors) for that change.")
    print("  * = the change is larger than its noise band, so it is unlikely to be luck.")
    print(f"  game plan for {deck}: {PLAN[deck]}")
    print(f"    {'':48s} " + '   '.join(f"{'[' + p + ']':^39s}" for p in profs))
    moved = {}
    cur_axis = None
    for key, label, axis, better, only in METRICS:
        if only == 'combo' and deck not in ('veyran', 'sauron'): continue
        if only and only not in ('combo', deck): continue
        if key.startswith('dmg_') and key != 'dmg_dealt' and all(
                b['axes'].get(key, {}).get('mean', 0) == 0 and v['axes'].get(key, {}).get('mean', 0) == 0 for b, v in pairs):
            continue
        cells = []
        for b, v in pairs:
            ab, av = b['axes'].get(key), v['axes'].get(key)
            if not ab or not av:
                cells.append(f"{'n/a':>36s}"); continue
            d = av['mean'] - ab['mean']
            se = math.sqrt(ab['var'] / ab['n'] + av['var'] / av['n'])
            sig = se > 0 and abs(d) > 2 * se
            pctfmt = key in ('win', 'plan_reached', 'combo_rate')
            f = (lambda x: f'{100*x:5.1f}%') if pctfmt else (lambda x: f'{x:6.2f}')
            dd = f'{100*d:+.1f}pt' if pctfmt else f'{d:+.2f}'
            nz = f'±{200*se:.1f}' if pctfmt else f'±{2*se:.2f}'
            cells.append(f"{f(ab['mean'])} -> {f(av['mean'])} {dd:>7s} {nz:>6s}{'*' if sig else ' '}")
            if sig:
                good = (d > 0) == (better == 'up')
                moved.setdefault((axis, label, good), []).append(b['profile'])
        if axis != cur_axis:
            print(f"\n  {axis}")
            cur_axis = axis
        print(f"    {label:48s} " + '   '.join(cells))
    # narrative
    print("\n--- Reading it ---")
    if not moved:
        print("  No axis moved beyond noise under either profile: the swap is roughly a like-for-like")
        print("  trade in this pod, or the new card rarely matters. Try --n 10000 to resolve smaller effects.")
        return
    by_axis = {}
    for (axis, label, good), ps in moved.items():
        by_axis.setdefault(axis, []).append((label, good, ps))
    for axis, items in by_axis.items():
        print(f"  {axis}: {AXIS_NOTE[axis]}")
        for label, good, ps in items:
            where = 'both profiles' if len(ps) == len(profs) else ps[0] + ' profile only'
            print(f"    {'better' if good else 'worse':6s} - {label} ({where})")
    print("  With ~40 checks per run, one or two stars can appear by chance; trust the ones that show")
    print("  up under both profiles, or that line up with the card's job.")
    print("  Note: per-game counts rise with game length, so check 'Rounds survived' before reading")
    print("  more cards drawn or more spells cast as a pure gain.")


# ------------------------------------------------------------------ deck analysis (--analyze)
KINDS = ('combat', 'burn', 'drain', 'aether', 'triggers', 'combo', 'commander damage', 'decked', 'other')


def _analyze_chunk(deck, cards, profile, seeds, ai='adaptive', temp=1.0):
    engine.set_profile(profile); set_ai(ai, temp)
    import cards as _cards; _cards.ensure_cards(cards, verbose=False)
    decks = dict(DECKS); decks[deck] = cards
    R = analysis_record()
    errs = Counter()
    for s in seeds:
        g = _safe_game(s, decks, errs)
        if g is None: continue
        analysis_add(R, g, deck)
    R['errs'] = errs
    return R


def analysis_record():
    return {'n': 0, 'win': 0, 'wintype': Counter(), 'kills': Counter(), 'death': Counter(), 'death_round': 0,
            'survived': 0, 'plan': Counter(), 'plan_n': 0, 'combo': 0, 'aether': 0,
            'seen': Counter(), 'cast': Counter(), 'win_seen': Counter(), 'win_cast': Counter(), 'lost': Counter(),
            'dmg': Counter()}


def analysis_add(R, g, deck):
    """add one finished game to an --analyze record, from `deck`'s point of view"""
    me = next(p for p in g.players if p.key == deck)
    won = g.winner is me
    R['n'] += 1; R['win'] += won
    if won: R['wintype'][g.wintype or 'damage'] += 1
    for k in KINDS: R['kills'][k] += me.stats['kills_' + k]
    for k in ('combat', 'burn', 'drain', 'aether', 'triggers'): R['dmg'][k] += me.stats['dmgk_' + k]
    if not me.alive:
        R['death'][(me.killer.key if me.killer is not None else 'self', getattr(me, 'death_kind', 'other'))] += 1
        R['death_round'] += me.stats.get('elim_round', g.round)
    R['survived'] += me.stats.get('elim_round', g.round)
    t = plan_turn(me, deck)
    if t: R['plan'][t] += 1
    R['combo'] += 1 if me.stats['combo_attempt'] else 0
    R['aether'] += 1 if me.stats['aether_shots'] else 0
    seen = me.seen_names | me.cast_names
    for nme in seen:
        R['seen'][nme] += 1; R['win_seen'][nme] += won
    for nme in me.cast_names:
        R['cast'][nme] += 1; R['win_cast'][nme] += won
    for nme, k in me.lost_names.items(): R['lost'][nme] += k


def analyze(deck, cards, profile, n, seed0=500000):
    seeds = list(range(seed0, seed0 + n))
    res = _run_chunks(_analyze_chunk, [(deck, cards, profile, s, AI, TEMP) for s in _chunks(seeds)])
    R = res[0]
    for r in res[1:]:
        for k, v in r.items():
            if isinstance(v, Counter): R[k].update(v)
            else: R[k] += v
    ERR.update(R.pop('errs', Counter()))
    return R


def print_analysis(deck, cards, R, profile):
    N = R['n']; base = R['win'] / N
    card_types = {c.name: c for c in (DB[x] for x in set(cards))}
    print(f'\n=========== {deck} analysis [{profile} profile, {AI} AI, {N} games] ===========')
    print(f'Win rate {100*base:.1f}%   (4-player baseline 25%)   average rounds survived {R["survived"]/N:.1f}')

    print('\n-- How it wins --')
    if R['win']:
        for k, v in R['wintype'].most_common():
            print(f'  {k:10s} {100*v/R["win"]:5.1f}% of wins')
    tk = sum(R['kills'].values())
    if tk:
        print(f'  Opponents it eliminated: {tk/N:.2f} per game, by')
        for k, v in R['kills'].most_common():
            if v: print(f'    {k:17s} {100*v/tk:5.1f}%')
    td = sum(R['dmg'].values())
    if td:
        print(f'  Damage dealt per game: {td/N:.1f}, from')
        for k, v in R['dmg'].most_common():
            if v: print(f'    {k:17s} {v/N:6.1f}  ({100*v/td:.0f}%)')

    print('\n-- How it loses --')
    nd = sum(R['death'].values())
    if nd:
        print(f'  Eliminated in {100*nd/N:.1f}% of games, on average in round {R["death_round"]/nd:.1f}')
        for (who, how), v in R['death'].most_common(8):
            if who not in KEYS and who != 'self': who = engine.SEATS.get(who, {}).get('name', who)   # pool deck
            print(f'    by {who:8s} via {how:17s} {100*v/N:5.1f}% of games')

    print('\n-- Game plan timing --')
    print(f'  Plan: {PLAN[deck]}')
    cum = 0; pn = sum(R['plan'].values())
    line = []
    for t in range(2, 13):
        cum += R['plan'].get(t, 0)
        line.append(f'T{t} {100*cum/N:.0f}%')
    print('  Online by turn:  ' + '  '.join(line))
    print(f'  Online at all: {100*pn/N:.1f}% of games')
    if deck in ('veyran', 'sauron'): print(f'  Combo attempted in {100*R["combo"]/N:.1f}% of games')
    if deck == 'veyran': print(f'  Aetherflux fired in {100*R["aether"]/N:.1f}% of games')

    print('\n-- Card report (correlation, not causation: see notes) --')
    rows = []
    for nme, c in card_types.items():
        if c.land: continue
        seen = R['seen'][nme]; cast = R['cast'][nme]
        rows.append((nme, seen, cast, R['win_cast'][nme], R['lost'][nme], c))
    minn = max(100, N // 50)
    rated = [r for r in rows if r[2] >= minn]
    rated.sort(key=lambda r: -(r[3] / r[2]))
    fmt = lambda r: (f'    {r[0][:42]:42s} cast in {100*r[2]/N:5.1f}% of games,'
                     f' win rate when cast {100*r[3]/r[2]:5.1f}% ({100*(r[3]/r[2]-base):+.1f})')
    print(f'  Highest win rate when cast (cast in at least {minn} games):')
    for r in rated[:10]: print(fmt(r))
    print('  Lowest win rate when cast:')
    for r in rated[-10:][::-1]: print(fmt(r))
    held = lambda c: 'ctr' in c.tags or c.tags.get('prot') in ('hi', 'phase', 'indes', 'blink', 'notw') or 'tide' in c.tags
    dead = [r for r in rows if r[1] >= minn and r[2] / r[1] < 0.5]
    dead.sort(key=lambda r: r[2] / r[1])
    print('  Often stuck in hand (drawn, but cast in under half of those games):')
    for r in dead[:12]:
        why = ' [reactive: only cast in response]' if held(r[5]) else (' [never cast by the sim: untagged]' if r[2] == 0 else '')
        print(f'    {r[0][:42]:42s} cast {100*r[2]/max(1,r[1]):5.1f}% of the games it was drawn{why}')
    lost = sorted([r for r in rows if r[4]], key=lambda r: -r[4])
    if lost:
        print('  Most often removed by opponents (per game):')
        for r in lost[:8]: print(f'    {r[0][:42]:42s} {r[4]/N:.2f}')
    print('\n  Notes: a high win rate when cast can mean the card helps, or that it only gets cast in')
    print('  games that were already going well (expensive cards, finishers). Low numbers on cheap cards')
    print('  that are cast early are the most telling. Use the list to pick swaps, then confirm each one')
    print('  with a normal paired comparison.')


def goldfish(deck, cards, n=10000, turns=10):
    engine.set_profile('conservative')
    key = {'seph': 'bomb_by', 'najeela': 'act'}.get(deck, 'combo')
    ms = []
    for s in range(n):
        g, p = ais.goldfish(s, deck, cards, turns)
        ms.append(p.milestone.get(key))
    return {t: sum(1 for m in ms if m is not None and m <= t) / n for t in range(3, turns + 1)}


def pct(x): return f'{100 * x:5.1f}%'


def verdict(deltas, ses, profiles):
    clear = [abs(d) > 2 * s for d, s in zip(deltas, ses)]
    if all(clear) and len({d > 0 for d in deltas}) == 1:
        return 'BETTER under both profiles' if deltas[0] > 0 else 'WORSE under both profiles'
    if all(clear):
        return 'TABLE-DEPENDENT (clear under both profiles, but in opposite directions)'
    if any(clear):
        i = clear.index(True)
        word = 'BETTER' if deltas[i] > 0 else 'WORSE'
        return f'LEANS {word} (clear only under {profiles[i]}; rerun with a larger --n to confirm)'
    return 'NO CLEAR EFFECT (within noise)'


def report(deck, pairs, label='variant'):
    """pairs: list of (baseline_result, variant_result) per profile"""
    print(f'\n=== {deck}: baseline vs {label} ===')
    deltas, ses = [], []
    for b, v in pairs:
        n = min(b['n'], v['n'])
        wb, wv = b['win'].get(deck, 0) / b['n'], v['win'].get(deck, 0) / v['n']
        se = math.sqrt(wb * (1 - wb) / b['n'] + wv * (1 - wv) / v['n'])
        deltas.append(wv - wb); ses.append(se)
        print(f"[{b['profile']:12s}] {deck} win {pct(wb)} -> {pct(wv)}  "
              f"(delta {100*(wv-wb):+.1f} pts, noise ±{200*se:.1f})   n={n}")
        others = '  '.join(f"{k} {pct(b['win'].get(k,0)/b['n'])}->{pct(v['win'].get(k,0)/v['n'])}"
                           for k in KEYS if k != deck)
        print(f"{'':15s}others: {others}")
        kb = '  '.join(f"{k} {pct(v['killed_by'].get(k,0)/v['n'])}" for k in KEYS if k != deck)
        print(f"{'':15s}{deck} (variant) eliminated by: {kb}")
    print('VERDICT:', verdict(deltas, ses, [b['profile'] for b, _ in pairs]))
    if VERBOSE and all('axes' in b and 'axes' in v for b, v in pairs): axis_report(deck, pairs)
    if min(b['n'] for b, _ in pairs) < 1500: print('(small sample: treat as a quick look only)')
    return deltas


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--deck', choices=KEYS)
    ap.add_argument('--swap', action='append')
    ap.add_argument('--n', type=int, default=1500)
    ap.add_argument('--profiles', default='conservative,loose')
    ap.add_argument('--ablate', action='store_true')
    ap.add_argument('--goldfish', action='store_true')
    ap.add_argument('--part', choices=('baseline', 'variant'))
    ap.add_argument('--profile')
    ap.add_argument('--out')
    ap.add_argument('--report', nargs='+')
    ap.add_argument('--dsl-all', action='store_true', help='run every card from its Scryfall Oracle text through the ability interpreter (ignores hand tags)')
    ap.add_argument('--cards', action='store_true', help='list every card in --deck with its tags, source and unmodeled text')
    ap.add_argument('--analyze', action='store_true', help='deep report on one deck: how it wins/loses, plan timing, card report')
    ap.add_argument('--brief', action='store_true', help='skip the per-axis breakdown')
    ap.add_argument('--trace', type=int, metavar='GAME', help='print a play-by-play log of one game (seed number) and exit')
    ap.add_argument('--ai', choices=('adaptive', 'rigid'), default='adaptive', help='AI decision model (default adaptive)')
    ap.add_argument('--temp', type=float, default=1.0, help='adaptive AI randomness multiplier (lower = sharper play)')
    ap.add_argument('--quiet', action='store_true', help='no progress bar')
    ap.add_argument('--jobs', type=int, default=1, help='parallel worker processes (e.g. number of CPU cores)')
    ap.add_argument('--pool', choices=('t1', 't2', 't3', 't4', 't5', 'all'),
                    help='pool mode: play --deck against three outside decks drawn from this tier (see opponents/)')
    ap.add_argument('--all-decks', action='store_true', help='pool mode: run each of the four decks (deck x tier matrix)')
    ap.add_argument('--calibrate', choices=('within', 'ordering', 'all'), help='pool balance checks (no --deck needed)')
    ap.add_argument('--games', type=int, help='pool mode: games per run (same as --n)')
    ap.add_argument('--seed', type=int, default=500000, help='pool mode: first seed (games use seed .. seed+n-1)')
    a = ap.parse_args()
    global JOBS, VERBOSE
    JOBS = max(1, a.jobs); VERBOSE = not a.brief
    global AI, TEMP
    AI, TEMP = a.ai, a.temp
    set_ai(AI, TEMP)

    if a.report:
        rs = [json.load(open(f)) for f in a.report]
        deck = rs[0]['deck']
        byp = {}
        for r in rs: byp.setdefault(r['profile'], {})[r['part']] = r
        report(deck, [(d['baseline'], d['variant']) for d in byp.values() if len(d) == 2])
        return

    if a.pool or a.calibrate or a.all_decks:
        import poolmode
        poolmode.C = sys.modules[__name__]      # this module's settings (--jobs, --brief, --ai), even when run as __main__
        if a.all_decks and not a.pool: a.pool = 'all'
        swaps, base, var = [], None, None
        if a.deck and not a.all_decks:
            swaps, auto = parse_swaps(a.swap)
            check_identity(a.deck, swaps)
            show_auto(auto)
            apply_swaps(a.deck, swaps)                                    # validates the swaps
            base, var = DECKS[a.deck], poolmode.swap_in_place(DECKS[a.deck], swaps)
        elif not a.calibrate and not a.all_decks:
            sys.exit('--deck is required (or use --all-decks)')
        if a.trace is not None:
            poolmode.trace(a, var if swaps else base); return
        poolmode.main(a, swaps, base, var)
        return

    if not a.deck: sys.exit('--deck is required')
    swaps, auto = parse_swaps(a.swap)
    check_identity(a.deck, swaps)
    show_auto(auto)
    base, var = DECKS[a.deck], apply_swaps(a.deck, swaps)
    profiles = a.profiles.split(',')

    if a.trace is not None:
        engine.set_profile(profiles[0])
        decks = dict(DECKS); decks[a.deck] = var
        g = ais.play_game(500000 + a.trace, decks, trace=True)
        which = 'variant list' if swaps else 'current list'
        print(f'Play-by-play of game {a.trace} ({a.deck}: {which}, {profiles[0]} profile, {AI} AI)')
        print('\n'.join(g.log))
        print(f"Winner: {g.winner.key if g.winner else 'none'} ({g.wintype})")
        return

    if a.cards:
        import cards
        names = sorted(set(var if swaps else base))
        auto = [n for n in names if DB[n].source != 'manual']
        print(f'{a.deck}: {len(names)} unique cards, {len(names) - len(auto)} hand-tagged, {len(auto)} from Scryfall / ability data')
        for n in names: print('  ' + cards.describe(DB[n]))
        try:
            gcs = cards.game_changers(names)
            if gcs is None: print('\n(Game Changer check unavailable: could not reach Scryfall for every card)')
            else: print(f'\nGame Changers per Scryfall ({len(gcs)}): ' + (', '.join(gcs) or 'none'))
        except SystemExit as e:
            print(f'\n(Game Changer check skipped: {e})')
        return

    PROG.on = not a.quiet
    if a.analyze:
        t = time.time()
        PROG.plan(a.n * len(profiles))
        cards = var if swaps else base
        if swaps: print('Analyzing the variant list:', '; '.join(f'{o} -> {i}' for o, i in swaps))
        results = []
        for prof in profiles:
            PROG.label = f'analyze / {prof}'
            results.append((prof, analyze(a.deck, cards, prof, a.n)))
        for prof, R in results: print_analysis(a.deck, cards, R, prof)
        print(f'\n({time.time()-t:.0f}s total)')
        return

    if a.part:
        t = time.time(); PROG.plan(a.n); PROG.label = f'{a.part} / {a.profile}'
        r = pod(a.deck, base if a.part == 'baseline' else var, a.profile, a.n)
        r.update(deck=a.deck, part=a.part)
        json.dump(r, open(a.out or f'{a.part}_{a.profile}.json', 'w'))
        print(f'{a.part}/{a.profile} done in {time.time()-t:.0f}s')
        return

    print('Swaps:', '; '.join(f'{o} -> {i}' for o, i in swaps) or '(none)')
    print(f'AI: {AI}' + (f' (temperature x{TEMP})' if AI == 'adaptive' else ''))
    t = time.time()
    abl = swaps if (a.ablate and len(swaps) > 1) else []
    PROG.plan(a.n * len(profiles) * (2 + len(abl)))
    baselines = {}
    for p in profiles:
        PROG.label = f'current list / {p}'; baselines[p] = pod(a.deck, base, p, a.n)
    variants = {}
    for p in profiles:
        PROG.label = f'with swaps / {p}'; variants[p] = pod(a.deck, var, p, a.n)
    singles = []
    for sw in abl:
        rs = {}
        for p in profiles:
            PROG.label = f'only {sw[1]} / {p}'; rs[p] = pod(a.deck, apply_swaps(a.deck, [sw]), p, a.n)
        singles.append((sw, rs))
    report(a.deck, [(baselines[p], variants[p]) for p in profiles])
    for sw, rs in singles:
        report(a.deck, [(baselines[p], rs[p]) for p in profiles], label=f'only {sw[0]} -> {sw[1]}')
    if a.goldfish:
        gb, gv = goldfish(a.deck, base), goldfish(a.deck, var)
        what = {'seph': 'bomb on battlefield', 'najeela': 'first WUBRG activation'}.get(a.deck, 'combo ready')
        print(f'\nGoldfish ({what}) by turn:')
        for t_ in gb: print(f'  T{t_:2d}  baseline {pct(gb[t_])}   variant {pct(gv[t_])}')
    print(f'\n({time.time()-t:.0f}s total)')


if __name__ == '__main__':
    try:
        main()
    finally:
        if _POOL is not None: _POOL.close(); _POOL.join()
        report_errors()