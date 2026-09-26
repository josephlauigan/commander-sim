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
import copy, math, random, re, zlib
from commander_sim import engine as E

KEYS = set()            # decks that search ('*' in KEYS: every deck)
ROLLOUTS = 6
TOP_K = 5
HORIZON = 1             # play out to the end of p's next turn (2: the turn after that)
COMBO_W = 12.0          # value of holding every piece of a combo the deck plays (scaled by the share of pieces held)
PLAYOUT_WORK = 2_000    # engine steps one playout may take (see E.tick); a playout that runs out is scored where it stands
GAME_DECISIONS = 1000   # look-ahead decisions per game (a game takes about 125); past this the heuristic AI plays on
BOARD_LIMIT = 150       # permanents on the table past which copies get too slow (a 250-Goblin lock): heuristic AI
GAME_SEARCH_WORK = 3_000_000   # engine steps of look-ahead playouts per game (a game uses about 270,000); past it, heuristic
STATS = {'decisions': 0, 'playouts': 0, 'changed': 0, 'cut': 0, 'max_work': 0, 'capped': 0, 'big_board': 0}   # capped: games past GAME_DECISIONS or GAME_SEARCH_WORK
CUTS = []               # where playouts ran out of steps: (deck, round, innermost frames), first few only

_SHARED = None


def enabled(g, p):
    if getattr(g, 'in_search', False): return False
    if not ('*' in KEYS or p.key in KEYS): return False
    if sum(len(q.perms) for q in g.players) > BOARD_LIMIT:
        STATS['big_board'] = STATS.get('big_board', 0) + 1
        return False
    if getattr(g, 'search_work', 0) > GAME_SEARCH_WORK:
        STATS['capped'] += 1
        return False
    if getattr(g, 'search_n', 0) >= GAME_DECISIONS:
        if g.search_n == GAME_DECISIONS: STATS['capped'] += 1; g.search_n += 1
        return False
    return True


def _decision_seed(g, *parts):
    """a new decision in game g: its random seed, the same in every process (Python's string hashing is not)"""
    STATS['decisions'] += 1
    g.search_n = getattr(g, 'search_n', 0) + 1
    return zlib.crc32(repr((g.search_n, g.round) + parts).encode())


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
    of p2's next turn (HORIZON of p2's turns)"""
    from commander_sim import ais
    from commander_sim.ai import brain
    act = active or p2
    if act.alive: ais.continue_turn(g2, act, g2.step)
    ps = g2.players
    k = ps.index(act)
    left = HORIZON
    while not g2.over and p2.alive:
        k = (k + 1) % len(ps)
        if k == 0: g2.round += 1
        if g2.round > stop_rounds: break
        q = ps[k]
        if not q.alive: continue
        if getattr(q, 'skip_turns', 0) > 0:                   # Ral Zarek -7
            q.skip_turns -= 1; continue
        brain.end_of_turn_window(g2, q)
        if g2.over: break
        if q.alive: ais.take_turn(g2, q)
        if q is p2:
            left -= 1
            if left <= 0: break


def evaluate(g, p):
    """how good the position is for p: winning / losing, else p's strength against the strongest opponents"""
    if g.over: return 100.0 if g.winner is p else -100.0
    if not p.alive: return -100.0
    opps = [q for q in g.players if q is not p and q.alive]
    dead = sum(1 for q in g.players if q is not p and not q.alive)
    if not opps: return 100.0
    s = strength(g, p)
    so = sorted((strength(g, q) for q in opps), reverse=True)
    raw = s - 0.6 * so[0] - 0.4 * (sum(so) / len(so)) + 12.0 * dead
    return 95.0 * math.tanh(raw / 100.0)     # below a win however big the board (250 goblins must still attack)


def strength(g, q):
    life = max(0, min(q.life, 60))
    board = sum(E.pval(g, m) for m in q.perms if not m.phased)
    power = sum(E.epow(g, m) for m in q.perms if m.creature and not m.phased)
    return (0.25 * life + board + 0.35 * power + 0.9 * len(q.hand) + 0.6 * len(q.lands) + 0.4 * q.treasures
            + COMBO_W * combo_progress(q))


_DECK_COMBOS = {}


def deck_combos(q):
    """the modeled combos whose pieces are all in q's deck (fixed per deck)"""
    v = _DECK_COMBOS.get(q.key)
    if v is None:
        from commander_sim.cards.impl import combos as impl_combos
        names = {c.name for c in impl_combos.full_deck_names(q)} | {q.cmd.name}
        v = _DECK_COMBOS[q.key] = [c for c in impl_combos.COMBOS if all(any(n in names for n in grp) for grp in c.groups)]
    return v


def combo_progress(q):
    """share of q's closest combo that q holds (in hand, on the battlefield, or the commander in the command zone),
    squared: the last pieces count most. 0 for a deck without a modeled combo."""
    cmbs = deck_combos(q)
    if not cmbs: return 0.0
    have = {m.cd.name for m in q.perms if m.cd is not None and not m.phased} | {c.name for c in q.hand}
    if q.cmd_in_zone: have.add(q.cmd.name)
    best = max(sum(1 for grp in c.groups if any(n in have for n in grp)) / len(c.groups) for c in cmbs)
    return best * best


def _start(g2, g):
    """a copy g2 of the real game g begins a playout: step budget, and its work is booked to g (GAME_SEARCH_WORK)"""
    g2.in_search = True
    g2.search_parent = g
    g2.work_start = g2.work; g2.work_cap = g2.work + PLAYOUT_WORK
    g2.board_cap = BOARD_LIMIT                 # a board that explodes mid-playout makes every step slow


def _done(g2, err=None):
    """book-keeping after a playout; err: the OutOfWork that stopped it"""
    STATS['playouts'] += 1
    w = g2.work - g2.work_start
    STATS['max_work'] = max(STATS['max_work'], w)
    real = getattr(g2, 'search_parent', None)
    if real is not None: real.search_work = getattr(real, 'search_work', 0) + w
    if err is not None:
        STATS['cut'] += 1
        if len(CUTS) < 20:
            import traceback
            fr = traceback.extract_tb(err.__traceback__)[-8:]
            CUTS.append((E.NAME(g2.active) if g2.active else '?', g2.round,
                         ' <- '.join(f'{f.name}:{f.lineno}' for f in reversed(fr))))


def _find(opts, label, n):
    k = 0
    for o in opts:
        if o[1] == label:
            if k == n: return o
            k += 1
    return None


def choose(g, p, post, opts):
    """pick a main-phase play by look-ahead; None to let the heuristic choose"""
    from commander_sim.ai import brain
    real = [o for o in opts if o[2] is not None]
    if not real: return None
    ranked = sorted(real, key=lambda o: -o[0])[:TOP_K]
    stop = next(o for o in opts if o[2] is None)
    cands = ranked + [stop]
    keyed = []
    for o in cands:                                     # (label, occurrence) identifies an option in a copy
        n = sum(1 for x in opts[:opts.index(o)] if x[1] == o[1])
        keyed.append((o, o[1], n))
    base_seed = _decision_seed(g, p.key, len(p.hand), len(p.perms))
    saved = (E.CUR_G, E.LAST_COUNTER, E.PAY_FOR)
    scores = {id(o): 0.0 for o in cands}
    try:
        for r in range(ROLLOUTS):
            for o, label, n in keyed:
                rng = random.Random(base_seed * 31 + r)
                g2 = clone(g)
                _start(g2, g)
                p2 = g2.players[g.players.index(p)]
                determinize(g2, p2, rng)
                g2.rng = random.Random(rng.random())
                E.CUR_G = g2
                err = None
                try:
                    o2 = _find(brain.main_options(g2, p2, post), label, n)
                    if o2 is None: scores[id(o)] -= 50.0; continue
                    if o2[2] is not None:
                        if not o2[2](): scores[id(o)] -= 50.0; continue
                        brain.main(g2, p2, post)        # the rest of this phase, heuristically
                    play_on_after_phase(g2, p2, post)
                except E.OutOfWork as e:
                    err = e
                scores[id(o)] += evaluate(g2, p2)
                _done(g2, err)
    finally:
        E.CUR_G, E.LAST_COUNTER, E.PAY_FOR = saved
    best = max(cands, key=lambda o: scores[id(o)])
    if best is not max(opts, key=lambda o: o[0]): STATS['changed'] += 1
    return best


def play_on_after_phase(g2, p2, post):
    """the chosen play is made and the phase finished: continue from the next step"""
    from commander_sim import ais
    nxt = {'main1': 'combat', 'main2': 'end'}.get(g2.step, 'end')
    g2.step = nxt
    play_on(g2, p2)


# ------------------------------------------------------------------ attacks
def choose_attack(g, p):
    """at the first combat of p's turn: (defender index, 'filtered' | 'all' | 'none') by look-ahead"""
    opps = [i for i, q in enumerate(g.players) if q is not p and q.alive]
    if not opps: return None
    cands = [(i, m) for i in opps for m in ('filtered', 'all')] + [(opps[0], 'none')]
    base_seed = _decision_seed(g, p.key, 'atk')
    saved = (E.CUR_G, E.LAST_COUNTER, E.PAY_FOR)
    scores = {c: 0.0 for c in cands}
    try:
        for r in range(ROLLOUTS):
            for c in cands:
                rng = random.Random(base_seed * 31 + r)
                g2 = clone(g)
                _start(g2, g)
                p2 = g2.players[g.players.index(p)]
                determinize(g2, p2, rng)
                g2.rng = random.Random(rng.random())
                E.CUR_G = g2
                g2.forced_attack = c
                g2.step = 'combat'
                err = None
                try:
                    play_on(g2, p2)
                except E.OutOfWork as e:
                    err = e
                scores[c] += evaluate(g2, p2)
                _done(g2, err)
    finally:
        E.CUR_G, E.LAST_COUNTER, E.PAY_FOR = saved
    best = max(cands, key=lambda c: scores[c])
    E.log(f'      [{E.NAME(p)} attack plan by search: {best[1]} at {E.NAME(g.players[best[0]])}]', g)
    return best


# ------------------------------------------------------------------ counterspells
def choose_counter(g, q, p, c, ctx, zone):
    """q may counter p's spell c (cast in p's main phase): True to counter, by look-ahead to the end of q's next turn"""
    base_seed = _decision_seed(g, q.key, 'ctr')
    saved = (E.CUR_G, E.LAST_COUNTER, E.PAY_FOR)
    scores = {True: 0.0, False: 0.0}
    try:
        for r in range(ROLLOUTS):
            for counter in (True, False):
                rng = random.Random(base_seed * 31 + r)
                g2, memo = clone(g, want_memo=True)
                _start(g2, g)
                q2 = g2.players[g.players.index(q)]; p2 = g2.players[g.players.index(p)]
                determinize(g2, q2, rng)
                g2.rng = random.Random(rng.random())
                E.CUR_G = g2
                ctx2 = {k: (memo.get(id(v), v) if isinstance(v, (E.Perm, E.Player)) else v) for k, v in (ctx or {}).items()}
                err = None
                try:
                    if counter:
                        ctr = E.pick_counter(g2, q2, c)
                        if ctr is None or not E.cast_counter(g2, q2, ctr, c): scores[counter] -= 50.0; continue
                        E.counter_side_effects(g2, q2, p2, ctr)
                        if ctr.name == 'Mana Drain': q2.drain_mana = getattr(q2, 'drain_mana', 0) + c.cmc
                        if c is p2.cmd: p2.cmd_in_zone = True
                        elif zone == 'gy' or ctx2.get('exile_after'): p2.exile.append(c)
                        elif not c.land: p2.gy.append(c)
                    else:
                        E.resolve(g2, p2, c, ctx2, zone); E.check_state(g2)
                    if not g2.over: play_on(g2, q2, active=p2)
                except E.OutOfWork as e:
                    err = e
                scores[counter] += evaluate(g2, q2)
                _done(g2, err)
    finally:
        E.CUR_G, E.LAST_COUNTER, E.PAY_FOR = saved
    return scores[True] > scores[False]
