"""The differential harness: the same positions in the Python and the Rust engines, and what each AI reads from them.

    python3 rust/tools/difftest.py                    # the hand-built positions plus 200 random ones (seed 1)
    python3 rust/tools/difftest.py --gen 500 --seed 4 --show 20
    python3 rust/tools/difftest.py --keep pos.json    # also write the positions (rerun one with --load)

A position is a recipe both engines follow with their test helpers (tests/table.py, testkit.rs): decks, then per
seat its life, turns, lands, graveyard, hand, permanents and tokens. Libraries are shuffled differently, so a card
whose entering effect draws or searches can build a different position; each report starts with the position as
built, and a position built differently is counted apart.

Compared, for the seat to move: the board reading (brain.Situation), removal risk, the card it holds mana for, each
candidate's cast priority and utility, the tutor target, and every main-phase option with its utility; for every seat,
the position score (search.evaluate, search.strength). Nothing here is random, so the numbers should match exactly.

Differences are split by cause: 'card code' when a card with hand-written Python code (cards/impl, which the Rust
ports in M5) is involved, 'phase 6' for your decks whose AI the Rust ports in phase 6, 'random' when a land search
picked different basics (each engine has its own random numbers), 'build' when the position came out differently
for another reason, and 'AI' for the rest. 'AI' and 'build' differences are bugs.
"""
import argparse, json, os, random, subprocess, sys, tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, ROOT)

from tests.table import table, hand, lands, perm, token, take, setup          # noqa: E402
from commander_sim import engine as E, ais as A                             # noqa: E402
from commander_sim.ai import brain, search                                  # noqa: E402
from commander_sim.cards import cardimpl as CI                              # noqa: E402

TOL = 1e-6

# ------------------------------------------------------------------ positions
HAND_BUILT = [
    {'name': 'sauron: sol ring and assault', 'decks': ['sauron', 'veyran'],
     'seats': [{'lands': [['Island', 2]], 'hand': ['Sol Ring', 'Aggravated Assault']}, {}]},
    {'name': 'seph removal on the biggest threat', 'decks': ['sauron', 'veyran'],
     'seats': [{'lands': [['Plains', 1], ['Swamp', 3]], 'hand': ['Swords to Plowshares']},
               {'tokens': [1], 'perms': ['Hellkite Tyrant']}]},
    {'name': 'search decision table', 'decks': ['veyran', 'sauron'], 'seed': 7,
     'seats': [{'lands': [['Island', 2], ['Mountain', 2]], 'hand': ['Guttersnipe', 'Think Twice', 'Lightning Bolt']},
               {'perms': ["Jace's Archivist"]}]},
    {'name': 'sauron sword out', 'decks': ['sauron', 'veyran', 'seph'],
     'seats': [{'turns': 6, 'lands': [['Island', 3], ['Mountain', 3]], 'perms': ['Sword of Feast and Famine'],
                'hand': ['Counterspell', 'Arcane Signet']}, {'tokens': [3, 3]}, {'life': 12}]},
]


def load_decks():
    d = json.load(open(os.path.join(ROOT, 'data', 'decks.json')))
    return {x['key']: x for x in d['mine'] + d['pool']}


def random_position(rng, i, decks, me_keys, opp_keys):
    """decks, lands, hands, graveyards and boards drawn from the decklists; in most positions no permanent has card
    code (what Python's card code does as a permanent enters waits for M5 in the Rust)"""
    plain = rng.random() < 0.8
    me = rng.choice(me_keys)
    opps = rng.sample([k for k in opp_keys if k != me], rng.choice([1, 2, 3]))
    keys = [me] + opps
    turn = rng.randint(1, 10)
    seats = []
    for j, k in enumerate(keys):
        d = decks[k]
        pile = [n for n in d['cards'] if n != d['commander']]
        rng.shuffle(pile)
        land_names = [n for n in pile if E.DB[n].land and not (plain and n in CARD_CODE)]
        spells = [n for n in pile if not E.DB[n].land]
        nl = min(len(land_names), rng.randint(max(0, turn - 2), turn + 1))
        tally = {}
        for n in land_names[:nl]:
            tally[n] = tally.get(n, 0) + 1
        s = {'life': rng.choice([40, 40, 30, 20, 12, 5]), 'turns': max(1, turn - (j > 0 and rng.random() < 0.5)),
             'lands': [[n, c, rng.random() < 0.2] for n, c in tally.items()]}
        perms = [n for n in spells if E.DB[n].perm and not (plain and n in CARD_CODE)][:rng.randint(0, 4)]
        rest = [n for n in spells if n not in perms]
        s['perms'] = perms
        s['hand'] = rest[:rng.randint(0, 7 if j == 0 else 5)]
        rest = rest[len(s['hand']):]
        s['gy'] = rest[:rng.randint(0, 4)]
        if rng.random() < 0.3: s['tokens'] = [rng.randint(1, 5) for _ in range(rng.randint(1, 3))]
        if rng.random() < 0.15: s['treasures'] = rng.randint(1, 3)
        s['library'] = pile                      # the library's order (the cards taken out of it keep theirs)
        seats.append(s)
    return {'name': f'random {i}', 'decks': keys, 'seed': rng.randint(1, 10**6), 'post': rng.random() < 0.3,
            'seats': seats}


# ------------------------------------------------------------------ the Python report
def build(pos):
    g = table(*pos['decks'], seed=pos.get('seed', 1))
    for p in g.players: brain.full_deck(p)         # the whole decklist, cached as a real game does on its first turn
    for p, s in zip(g.players, pos['seats']):
        if 'library' in s:                         # the same order in both engines (the shuffles differ)
            at = {}
            for i, n in enumerate(s['library']): at.setdefault(n, i)
            p.library.sort(key=lambda c: at.get(c.name, len(at)))
        if 'life' in s: p.life = s['life']
        if 'turns' in s: p.turns = s['turns']
        if 'treasures' in s: p.treasures = s['treasures']
        for ln in s.get('lands', ()):
            lands(p, ln[0], ln[1], ln[2] if len(ln) > 2 else False)
        for n in s.get('gy', ()):
            p.gy.append(take(p, n))
        if s.get('hand'): hand(p, *s['hand'])
        for n in s.get('perms', ()):
            perm(g, p, n)
        for t in s.get('tokens', ()):
            token(g, p, t)
    return g


def built(g):
    """the position as built, comparable across engines"""
    return [{'life': p.life, 'hand': sorted(c.name for c in p.hand), 'perms': sorted(m.name for m in p.perms),
             'lands': sorted(L.cd.name for L in p.lands), 'gy': sorted(c.name for c in p.gy),
             'library': len(p.library), 'ends': [c.name for c in p.library[:3] + p.library[-3:]],
             'treasures': p.treasures} for p in g.players]


def py_report(pos):
    g = build(pos)
    me = g.players[pos.get('me', 0)]
    post = pos.get('post', False)
    s = brain.Situation(g, me)
    hc, hv = brain.hold_value(g, me, s)
    cands = list(me.hand) + ([me.cmd] if me.cmd_in_zone else [])
    seat = {q: i for i, q in enumerate(g.players)}
    rep = {
        'built': built(g),
        'evaluate': [search.evaluate(g, q) for q in g.players],
        'strength': [search.strength(g, q) for q in g.players],
        'situation': {'mana': s.mana, 'hand': s.hand, 'lands_in_hand': s.lands_in_hand,
                      'threat': {str(seat[q]): v for q, v in s.threat.items()},
                      'leader': seat[s.leader] if s.leader is not None else None, 'max_threat': s.max_threat,
                      'incoming': s.incoming, 'danger': s.danger, 'combo_near': s.combo_near, 'ctr_risk': s.ctr_risk},
        'removal_risk': brain.removal_risk(g, me),
        'detail': [{'board': [[m.name, E.pval(g, m), E.epow(g, m)] for m in q.perms], 'mana': brain.open_mana(g, q),
                    'open_u': brain.open_mana(g, q, 'U'), 'p_counter': brain.prob_holding(g, q, brain.is_counter)}
                   for q in g.players],
        'hold': [hc.name if hc is not None else None, hv],
        'card_utility': {c.name: brain.card_utility(g, me, s, c) for c in cands},
        'deck_prio': {c.name: A.deck_prio(g, me, c) for c in cands},
        'tutor_pick': A.tutor_pick(g, me, 'any'),
        'tutor_cc': {(c.name if c is not None else '*any*'): tutor_card_code(g, me, c) for c in cands + [None]},
    }
    try:
        rep['main_options'] = [[lbl, u] for u, lbl, _ in brain.main_options(g, me, post)]
    except Exception as e:                                                  # noqa: BLE001
        rep['main_options'] = f'error: {e!r}'
    return rep


TUTOR_TAGS = ('tut', 'seal', 'intuition', 'gifts')


def tutor_card_code(g, p, c):
    """a tutor (c None: any tutor) whose best targets include a card with card code: its priority and pick depend
    on that code (its hooked priority, say), so a difference there waits for M5"""
    if c is not None and not any(c.tags.get(k) for k in TUTOR_TAGS): return False
    ok = A.TUTOR_OK.get(c.tags.get('tut') if c is not None else 'any', A.TUTOR_OK['any'])
    cands = [x for x in E.searchable(g, p) if ok(x) and not x.land]
    if not cands: return False
    vals = {x.name: (A.tutor_value(g, p, x), A.deck_prio(g, p, x)) for x in cands}
    top = max(v[0] for v in vals.values()), max(v[1] for v in vals.values())
    return any(n in CARD_CODE for n, v in vals.items() if v[0] == top[0] or v[1] == top[1])


def rust_reports(positions):
    with tempfile.NamedTemporaryFile('w', suffix='.json', delete=False) as f:
        json.dump(positions, f)
    try:
        out = subprocess.run(['cargo', 'run', '-q', '--release', '-p', 'sim-core', '--example', 'difftest', f.name],
                             cwd=os.path.join(ROOT, 'rust'), capture_output=True, text=True)
        if out.returncode != 0:
            sys.exit('the Rust side failed:\n' + out.stderr[-4000:])
        return json.loads(out.stdout)
    finally:
        os.unlink(f.name)


# ------------------------------------------------------------------ comparing
SHORT = {}               # 'Jaxis' -> 'Jaxis, the Troublemaker' (option labels use short names)
CARD_CODE = set()        # cards with hand-written Python code (filled after the card modules register: setup())
PHASE6 = ('seph', 'veyran', 'galadriel', 'yshtola', 'alela', 'jodah')   # your decks whose AI the Rust ports in phase 6


def load_card_code():
    CARD_CODE.update(set(CI.HOOKS) | set(CI.SPELL_PRIO) | set(CI.LAND_ETB) | set(CI.DYN_MANA) | set(CI.AS_ENTERS)
                     | set(CI.SAGA) | set(CI.SELF_REGEN) | set(CI.SELF_CAST) | set(CI.LAND_COLS) | set(CI.ON_TAP)
                     | set(CI.PVAL) | set(CI.SPELL_IMP))
    from commander_sim.cards.impl import combos
    CARD_CODE.update(combos.PIECES)                  # combo pieces: their threat bonus (combos.piece_threat)
    # card code reached by tag: removal that transforms or locks (engine/removal.rs: transform_away, apply_lock) and
    # Auras (attaching, falling off: cardcode::aura_fall)
    CARD_CODE.update(n for n, c in E.DB.items()
                     if c.tags.get('rem') in ('elk', 'mutate', 'forest', 'arrest', 'pacify', 'encrust', 'kasmina')
                     or 'aura' in (getattr(c, 'subtypes', None) or ()) or 'aura' in c.tags)
    SHORT.update({n.split(',')[0]: n for n in CARD_CODE if len(n.split(',')[0]) >= 4})


def same(a, b):
    if isinstance(a, bool) or isinstance(b, bool) or a is None or b is None or isinstance(a, str):
        return a == b
    if isinstance(a, (int, float)) and isinstance(b, (int, float)):
        return abs(a - b) <= TOL * max(1.0, abs(a), abs(b))
    return a == b


def diff(path, a, b, out):
    """the differences between two reports' values, as (path, python, rust)"""
    if isinstance(a, dict) and isinstance(b, dict):
        for k in sorted(set(a) | set(b)):
            diff(f'{path}.{k}', a.get(k, '<missing>'), b.get(k, '<missing>'), out)
    elif isinstance(a, list) and isinstance(b, list) and len(a) == len(b):
        for i, (x, y) in enumerate(zip(a, b)):
            diff(f'{path}[{i}]', x, y, out)
    elif not same(a, b):
        out.append((path, a, b))


def options(rep):
    m = rep['main_options']
    if isinstance(m, str): return m
    out = {}
    for lbl, u in m:
        out.setdefault(lbl, []).append(u)
    return {k: sorted(v) for k, v in out.items()}


# option labels from card code the Rust ports in M5 (cardcode.rs): Underworld Breach (breach_gc_options, Sauron's
# breach_options), partials' hand options (blitz, bestow), the outside decks' card plays (pool_card_options: evoke,
# aristocrats' sacrifices)
M5_OPTIONS = ('escape ', 'blitz ', 'bestow ', 'evoke ', 'sacrifice ', 'Underworld Breach line')


BASICS = ('Plains', 'Island', 'Swamp', 'Mountain', 'Forest', 'Wastes')


def basics_only(a, b):
    """a land search picked different basics (at random), and everything else that differs follows from it (the
    library's order changed: a mill or a draw later hit different cards)"""
    for x, y in zip(a, b):
        if len(x['lands']) != len(y['lands']): return False
        if [n for n in x['lands'] if n not in BASICS] != [n for n in y['lands'] if n not in BASICS]: return False
    return any(x['lands'] != y['lands'] for x, y in zip(a, b))


def board_card_code(pos, built):
    """the card-coded permanents and lands on the battlefield (as asked for, and as Python built it)"""
    names = [n for s in pos['seats'] for n in list(s.get('perms', ())) + [ln[0] for ln in s.get('lands', ())]]
    names += [n for s in built for n in s['perms'] + s['lands']]
    return sorted({n for n in names if n in CARD_CODE})


def involved(text):
    return [n for n in CARD_CODE if n in text] + [n for k, n in SHORT.items() if k in text]


def classify(pos, py, rs):
    """[(cause, what, python, rust)]"""
    board = board_card_code(pos, py['built'])
    for i, seat in enumerate(pos['seats']):             # without a fixed order the libraries are shuffled differently
        if 'library' not in seat:
            for r in (py, rs): r['built'][i].pop('ends', None)
    if py['built'] != rs['built']:
        d = []
        diff('built', py['built'], rs['built'], d)
        cards = [n for _, a, b in d for n in involved(str(a)) + involved(str(b))]
        cause = ('card code' if board or cards else 'random' if basics_only(py['built'], rs['built']) else 'build')
        return [(cause, p, a, b) for p, a, b in d]
    me = pos['decks'][pos.get('me', 0)]
    found = []
    for key in ('detail', 'evaluate', 'strength', 'situation', 'removal_risk', 'hold', 'card_utility', 'deck_prio', 'tutor_pick'):
        d = []
        diff(key, py[key], rs.get(key, '<missing>'), d)
        for p, a, b in d:
            cards = involved(p) + involved(str(a)) + involved(str(b))
            tut = py['tutor_cc'].get(p.split('.', 1)[1] if '.' in p else '') or (key == 'tutor_pick' and py['tutor_cc']['*any*'])
            cause = 'card code' if board or cards or tut else 'AI'
            if me in PHASE6 and key in ('card_utility', 'deck_prio', 'tutor_pick', 'hold'): cause = 'phase 6'
            found.append((cause, p, a, b))
    po, ro = options(py), options(rs)
    if isinstance(po, str) or isinstance(ro, str):
        if po != ro: found.append(('AI', 'main_options', po, ro))
        return found
    for lbl in sorted(set(po) | set(ro)):
        a, b = po.get(lbl), ro.get(lbl)
        if a is not None and b is not None and len(a) == len(b) and all(same(x, y) for x, y in zip(a, b)): continue
        cause = ('card code' if board or involved(lbl) or lbl.startswith(M5_OPTIONS) or py['tutor_cc'].get(lbl)
                 else 'phase 6' if me in PHASE6 else 'AI')
        found.append((cause, f'main_options[{lbl}]', a, b))
    return found


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--gen', type=int, default=200, help='random positions to add (default 200)')
    ap.add_argument('--seed', type=int, default=1)
    ap.add_argument('--show', type=int, default=12, help='AI differences to print in full (default 12)')
    ap.add_argument('--load', help='positions from this file instead (from --keep)')
    ap.add_argument('--keep', help='write the positions to this file')
    ap.add_argument('--json', help='write every difference to this file')
    ap.add_argument('--decks', help="the decks random positions are drawn from: comma-separated deck keys or tiers "
                                    "('t2,alela'); default: sauron and Tier 1")
    a = ap.parse_args()
    setup()
    load_card_code()
    if a.load:
        positions = json.load(open(a.load))
    else:
        decks = load_decks()
        keys = []
        for x in (a.decks or 'sauron,t1').split(','):
            keys += [k for k, d in decks.items() if d.get('tier') == x] if x in ('t1', 't2', 't3', 't4', 't5') else [x]
        rng = random.Random(a.seed)
        positions = list(HAND_BUILT) + [random_position(rng, i, decks, keys, keys) for i in range(a.gen)]
    if a.keep: json.dump(positions, open(a.keep, 'w'), indent=1)
    py = [py_report(p) for p in positions]
    rs = rust_reports(positions)
    by_cause, rows = {}, []
    clean = 0
    for pos, x, y in zip(positions, py, rs):
        found = classify(pos, x, y)
        if not found: clean += 1
        causes = {c for c, *_ in found}
        for c in causes: by_cause[c] = by_cause.get(c, 0) + 1
        rows += [(pos['name'], *f) for f in found]
    n = len(positions)
    items = sum(1 for _ in rows)
    print(f'{n} positions: {clean} match in full')
    for c in ('AI', 'card code', 'phase 6', 'random', 'build'):
        print(f'  {c:10s} differences in {by_cause.get(c, 0)} positions ({sum(1 for r in rows if r[1] == c)} values)')
    ai = [r for r in rows if r[1] == 'AI']
    if ai:
        print('\nAI differences (python | rust):')
        for name, _, what, a_, b_ in ai[:a.show]:
            print(f'  {name}: {what}: {a_!r} | {b_!r}')
        kinds = {}
        for r in ai:
            k = r[2].split('[')[0].split('.')[0]
            kinds[k] = kinds.get(k, 0) + 1
        print('  by value:', ', '.join(f'{k} {v}' for k, v in sorted(kinds.items(), key=lambda x: -x[1])))
    if a.json:
        json.dump([{'position': r[0], 'cause': r[1], 'what': r[2], 'python': r[3], 'rust': r[4]} for r in rows],
                  open(a.json, 'w'), indent=1, default=str)
    return 1 if ai else 0


if __name__ == '__main__':
    sys.exit(main())
