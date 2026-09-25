"""Look-ahead for main-phase decisions: try each candidate play on copies of the game and keep the one that leads to
the best position.

For a decision of player p:
  1. the candidates are the heuristic AI's top plays (by its own utility) plus 'stop' (hold the mana / go on);
  2. each candidate is tried in `ROLLOUTS` copies of the game. Each copy hides what p can't see: every opponent's
     hand is re-dealt from their hand and library, and p's own library is shuffled;
  3. in the copy the play is made, and the game continues with the fast heuristic AI until the end of p's next turn;
  4. the final position is scored from p's point of view (`evaluate`). All candidates share the same copies' random
     seeds, so they are compared on the same futures.
The candidate with the best mean score is played in the real game.
"""
import copy, random, re
import engine as E

KEYS = set()            # decks that search ('*' in KEYS: every deck)
ROLLOUTS = 6
TOP_K = 5
HORIZON = 'next_turn'   # play out to the end of p's next turn
STATS = {'decisions': 0, 'playouts': 0, 'changed': 0}

_SHARED = None


def enabled(g, p):
    if getattr(g, 'in_search', False) or not E.POOL_RULES: return False
    return '*' in KEYS or p.key in KEYS


# ------------------------------------------------------------------ copying a game
def _shared():
    global _SHARED
    if _SHARED is None or len(_SHARED) != len(E.DB):
        _SHARED = {id(cd): cd for cd in E.DB.values()}      # card definitions are shared, never copied
    return _SHARED


_IDS = re.compile(r'\d{6,}')


def _remap_key(k, memo):
    if isinstance(k, int) and k in memo: return id(memo[k])
    if isinstance(k, str) and _IDS.search(k):
        return _IDS.sub(lambda m: str(id(memo[int(m.group())])) if int(m.group()) in memo else m.group(), k)
    if isinstance(k, tuple): return tuple(_remap_key(x, memo) for x in k)
    return k


def _remap_dict(d, memo):
    if not isinstance(d, dict): return d
    if not any(isinstance(k, int) and k in memo or isinstance(k, (str, tuple)) and _IDS.search(str(k)) for k in d):
        return d
    items = [(_remap_key(k, memo), v) for k, v in d.items()]
    out = copy.copy(d); out.clear(); out.update(items)       # keep the type (Counter, defaultdict)
    return out


def clone(g, want_memo=False):
    """a deep copy of game g that can be played on without touching g. Object-id keyed tables (end-of-turn pumps,
    once-per-turn flags) are re-keyed to the copies; caches are dropped."""
    memo = dict(_shared())
    saved_log, g.log = g.log, None
    try:
        g2 = copy.deepcopy(g, memo)
    finally:
        g.log = saved_log
    for obj in [g2] + list(g2.players):
        for k, v in list(vars(obj).items()):
            if isinstance(v, dict) and v: setattr(obj, k, _remap_dict(v, memo))
    for q in g2.players:
        for m in q.perms:
            if m.data: m.data = _remap_dict(m.data, memo)
    g2.hook_cache = None; g2.static_idx = None; g2.coat_cache = None; g2.cur_cast = None
    return (g2, memo) if want_memo else g2


def determinize(g2, me, rng):
    """hide what `me` can't know: re-deal opponents' hands from hand + library, shuffle me's library"""
    for q in g2.players:
        if q is me:
            rng.shuffle(q.library)
        elif q.alive:
            pool = q.hand + q.library
            rng.shuffle(pool)
            n = len(q.hand)
            q.hand, q.library = pool[:n], pool[n:]


# ------------------------------------------------------------------ playing a copy forward
def play_on(g2, p2, stop_rounds=20, active=None):
    """finish the active player's turn (p2's by default) from the current step, then everyone's turns until the end
    of p2's next turn"""
    import ais, brain
    act = active or p2
    if act.alive: ais.continue_turn(g2, act, g2.step)
    ps = g2.players
    k = ps.index(act)
    while not g2.over and p2.alive:
        k = (k + 1) % len(ps)
        if k == 0: g2.round += 1
        if g2.round > stop_rounds: break
        q = ps[k]
        if not q.alive: continue
        brain.end_of_turn_window(g2, q)
        if g2.over: break
        if q.alive: ais.take_turn(g2, q)
        if q is p2: break


def evaluate(g, p):
    """how good the position is for p: winning / losing, else p's strength against the strongest opponents"""
    if g.over: return 100.0 if g.winner is p else -100.0
    if not p.alive: return -100.0
    opps = [q for q in g.players if q is not p and q.alive]
    dead = sum(1 for q in g.players if q is not p and not q.alive)
    if not opps: return 100.0
    s = strength(g, p)
    so = sorted((strength(g, q) for q in opps), reverse=True)
    return s - 0.6 * so[0] - 0.4 * (sum(so) / len(so)) + 12.0 * dead


def strength(g, q):
    life = max(0, min(q.life, 60))
    board = sum(E.pval(g, m) for m in q.perms if not m.phased)
    power = sum(E.epow(g, m) for m in q.perms if m.creature and not m.phased)
    return 0.25 * life + board + 0.35 * power + 0.9 * len(q.hand) + 0.6 * len(q.lands) + 0.4 * q.treasures


def _find(opts, label, n):
    k = 0
    for o in opts:
        if o[1] == label:
            if k == n: return o
            k += 1
    return None


def choose(g, p, post, opts):
    """pick a main-phase play by look-ahead; None to let the heuristic choose"""
    import brain
    real = [o for o in opts if o[2] is not None]
    if not real: return None
    ranked = sorted(real, key=lambda o: -o[0])[:TOP_K]
    stop = next(o for o in opts if o[2] is None)
    cands = ranked + [stop]
    keyed = []
    for o in cands:                                     # (label, occurrence) identifies an option in a copy
        n = sum(1 for x in opts[:opts.index(o)] if x[1] == o[1])
        keyed.append((o, o[1], n))
    STATS['decisions'] += 1
    saved = (E.CUR_G, E.LAST_COUNTER, E.PAY_FOR)
    base_seed = hash((g.round, p.key, len(p.hand), len(p.perms), STATS['decisions'])) & 0xffffffff
    scores = {id(o): 0.0 for o in cands}
    try:
        for r in range(ROLLOUTS):
            for o, label, n in keyed:
                rng = random.Random(base_seed * 31 + r)
                g2 = clone(g)
                g2.in_search = True
                p2 = g2.players[g.players.index(p)]
                determinize(g2, p2, rng)
                g2.rng = random.Random(rng.random())
                E.CUR_G = g2
                o2 = _find(brain.main_options(g2, p2, post), label, n)
                if o2 is None: scores[id(o)] -= 50.0; continue
                if o2[2] is not None:
                    if not o2[2](): scores[id(o)] -= 50.0; continue
                    brain.main(g2, p2, post)            # the rest of this phase, heuristically
                play_on_after_phase(g2, p2, post)
                scores[id(o)] += evaluate(g2, p2)
                STATS['playouts'] += 1
    finally:
        E.CUR_G, E.LAST_COUNTER, E.PAY_FOR = saved
    best = max(cands, key=lambda o: scores[id(o)])
    if best is not max(opts, key=lambda o: o[0]): STATS['changed'] += 1
    return best


def play_on_after_phase(g2, p2, post):
    """the chosen play is made and the phase finished: continue from the next step"""
    import ais
    nxt = {'main1': 'combat', 'main2': 'end'}.get(g2.step, 'end')
    g2.step = nxt
    play_on(g2, p2)


# ------------------------------------------------------------------ attacks
def choose_attack(g, p):
    """at the first combat of p's turn: (defender index, 'filtered' | 'all' | 'none') by look-ahead"""
    opps = [i for i, q in enumerate(g.players) if q is not p and q.alive]
    if not opps: return None
    cands = [(i, m) for i in opps for m in ('filtered', 'all')] + [(opps[0], 'none')]
    STATS['decisions'] += 1
    saved = (E.CUR_G, E.LAST_COUNTER, E.PAY_FOR)
    base_seed = hash((g.round, p.key, 'atk', STATS['decisions'])) & 0xffffffff
    scores = {c: 0.0 for c in cands}
    try:
        for r in range(ROLLOUTS):
            for c in cands:
                rng = random.Random(base_seed * 31 + r)
                g2 = clone(g)
                g2.in_search = True
                p2 = g2.players[g.players.index(p)]
                determinize(g2, p2, rng)
                g2.rng = random.Random(rng.random())
                E.CUR_G = g2
                g2.forced_attack = c
                g2.step = 'combat'
                play_on(g2, p2)
                scores[c] += evaluate(g2, p2)
                STATS['playouts'] += 1
    finally:
        E.CUR_G, E.LAST_COUNTER, E.PAY_FOR = saved
    best = max(cands, key=lambda c: scores[c])
    E.log(f'      [{E.NAME(p)} attack plan by search: {best[1]} at {E.NAME(g.players[best[0]])}]', g)
    return best


# ------------------------------------------------------------------ counterspells
def choose_counter(g, q, p, c, ctx, zone):
    """q may counter p's spell c (cast in p's main phase): True to counter, by look-ahead to the end of q's next turn"""
    STATS['decisions'] += 1
    saved = (E.CUR_G, E.LAST_COUNTER, E.PAY_FOR)
    base_seed = hash((g.round, q.key, 'ctr', STATS['decisions'])) & 0xffffffff
    scores = {True: 0.0, False: 0.0}
    try:
        for r in range(ROLLOUTS):
            for counter in (True, False):
                rng = random.Random(base_seed * 31 + r)
                g2, memo = clone(g, want_memo=True)
                g2.in_search = True
                q2 = g2.players[g.players.index(q)]; p2 = g2.players[g.players.index(p)]
                determinize(g2, q2, rng)
                g2.rng = random.Random(rng.random())
                E.CUR_G = g2
                ctx2 = {k: (memo.get(id(v), v) if isinstance(v, (E.Perm, E.Player)) else v) for k, v in (ctx or {}).items()}
                if counter:
                    ctr = E.pick_counter(g2, q2, c)
                    if ctr is None or not E.cast_counter(g2, q2, ctr): scores[counter] -= 50.0; continue
                    E.counter_side_effects(g2, q2, p2, ctr)
                    if ctr.name == 'Mana Drain': q2.drain_mana = getattr(q2, 'drain_mana', 0) + c.cmc
                    if c is p2.cmd: p2.cmd_in_zone = True
                    elif zone == 'gy' or ctx2.get('exile_after'): p2.exile.append(c)
                    elif not c.land: p2.gy.append(c)
                else:
                    E.resolve(g2, p2, c, ctx2, zone); E.check_state(g2)
                if not g2.over: play_on(g2, q2, active=p2)
                scores[counter] += evaluate(g2, q2)
                STATS['playouts'] += 1
    finally:
        E.CUR_G, E.LAST_COUNTER, E.PAY_FOR = saved
    return scores[True] > scores[False]
