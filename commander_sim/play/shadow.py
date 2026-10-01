"""The AI comparison log (practice mode's shadow log): at each decision the AI would weigh, what you did, what the AI
would have done, and how the two score when the game is played on from there.

When you answer a decision worth comparing (a play in your main phase, or your attack), the game is copied before
your answer takes effect. The comparison is worked out on that copy later, a playout at a time, on the engine
thread while it waits for your next answer (the engine keeps its current game in module-level state, so a second
thread can't play copies at the same time). Your answer is never held up by more than one playout. Whatever is left
when the game ends is finished before the review opens.

Scores are the look-ahead's: the mean over six playouts of where the position stands at the end of your next turn,
from -100 (lost) to +100 (won). The same six futures are used for every option of a decision, so the options are
compared on equal terms. A decision with one sensible option, or one the AI doesn't weigh this way (tapping mana,
playing a land, an ability), is recorded without a score.
"""
from commander_sim import engine as E


class Entry:
    def __init__(s, n, rnd, step, kind, situation, yours):
        s.n, s.round, s.step, s.kind, s.situation, s.yours = n, rnd, step, kind, situation, yours
        s.ai = None                  # the AI's choice (a label)
        s.scores = {}                # label -> mean score
        s.scored = False             # comparison finished
        s.note = ''
        s.yours_key = None           # your choice as the AI's label for it (the key into scores)
        s.hand, s.cmd = [], None     # your hand and commander (if in the command zone) at the decision
        s.ai_plan, s.n_choices = None, 0
        s.job = None                 # the work left: a generator, one playout per step

    def ai_answer(s):
        """the AI's choice as your answer at this decision, or None when it isn't a single action"""
        if s.kind == 'main':
            if s.ai.startswith('stop'): return {'do': 'pass'}
            name = s.ai.split(' -> ')[0]
            if name in s.hand: return {'do': 'cast', 'card': s.hand.index(name)}
            if name == s.cmd: return {'do': 'cast', 'zone': 'cmd'}
            return None
        if s.kind == 'attack':
            mode = s.ai_plan[1] if s.ai_plan else None
            if mode == 'none': return []
            if mode == 'all': return list(range(s.n_choices))
        return None

    def ai_text(s):
        if s.ai is None: return None
        if s.kind == 'main':
            if s.ai.startswith('stop'): return 'Pass (nothing more this phase)'
            name, _, tgt = s.ai.partition(' -> ')
            return f'Cast {name}' + (f' on {tgt}' if tgt else '')
        return s.ai

    def as_dict(s):
        return {'n': s.n, 'round': s.round, 'step': s.step, 'kind': s.kind, 'situation': s.situation,
                'key': s.yours_key, 'ai_text': s.ai_text(), 'can_try': s.scored and s.ai is not None,
                'yours': s.yours, 'ai': s.ai, 'score_yours': s.scores.get(s.yours_key) if s.scored else None,
                'score_ai': s.scores.get(s.ai) if s.scored and s.ai is not None else None, 'scored': s.scored,
                'note': s.note, 'scores': dict(s.scores), 'step_text': STEP_TEXT.get(s.step, s.step),
                'differs': bool(s.scored and s.ai != s.yours_key and s.yours_key in s.scores
                                and s.scores[s.ai] - s.scores[s.yours_key] >= SAME)}


STEPS = {'main1': 'main phase 1', 'main2': 'main phase 2', 'combat': 'combat'}
STEP_TEXT = {'main1': 'Main 1', 'main2': 'Main 2', 'combat': 'Combat'}


def situation(g, p):
    opps = ', '.join(f'{E.NAME(q)} {q.life}' for q in g.players if q is not p and q.alive)
    return (f"{p.life} life, {len(p.hand)} in hand, {len(p.lands)} lands, {sum(1 for m in p.perms if m.creature)} "
            f"creatures; opponents: {opps}")


SAME = 0.1          # options scoring within this of each other count as the same choice (no difference shown)


class Shadow:
    def __init__(s):
        s.entries = []

    # ------------------------------------------------------------------ recording (engine thread, before your answer applies)
    def record(s, g, p, req, ans, n):
        """decision n: request req answered with ans. Copies the game if the decision is one to compare"""
        if req.kind == 'priority' and isinstance(ans, dict):
            stack = req.data.get('stack') or []
            if g.active is not p or getattr(g, 'step', None) not in ('main1', 'main2') or stack: return
            do = ans.get('do')
            if do == 'tap': return                                     # getting mana ready: not a decision of its own
            label = _main_label(p, ans)
            e = Entry(n, g.round, g.step, 'main', situation(g, p), _yours_text(p, ans, label))
            e.hand, e.cmd = [c.name for c in p.hand], (p.cmd.name if p.cmd_in_zone else None)
            if label is None:
                e.note = 'not compared: the AI weighs spells and passing here, not this'
                s.entries.append(e); return
            e.yours_key = label
            e.job = _main_job(e, g, p, label)
            s.entries.append(e)
        elif req.kind == 'attack':
            label = _attack_label(g, p, req, ans)
            e = Entry(n, g.round, 'combat', 'attack', situation(g, p), _attack_text(g, p, req, ans))
            e.n_choices = len(req.choices)
            e.yours_key = label
            e.job = _attack_job(e, g, p, label)
            s.entries.append(e)

    def forget_from(s, n):
        """Undo: the decisions from n on are gone"""
        s.entries = [e for e in s.entries if e.n < n]

    # ------------------------------------------------------------------ working (engine thread, while idle)
    def pending(s):
        return [e for e in s.entries if e.job is not None]

    def step(s):
        """do one playout of the oldest unfinished comparison; False when there's nothing left"""
        for e in s.entries:
            if e.job is None: continue
            try:
                next(e.job)
            except StopIteration:
                e.job = None
            return True
        return False

    def finish(s, progress=None):
        total = sum(1 for _ in s.pending())
        done = 0
        while s.pending():
            e = s.pending()[0]
            for _ in e.job: pass
            e.job = None
            done += 1
            if progress: progress(done, total)

    def review(s):
        return [e.as_dict() for e in s.entries]


# ------------------------------------------------------------------ main phase
def _main_label(p, ans):
    """the AI's label for your play: a card's name, or 'stop' for passing; None for something it doesn't weigh"""
    do = ans.get('do')
    if do == 'pass': return 'stop'
    if do == 'cast':
        if ans.get('zone') == 'cmd': return p.cmd.name
        i = ans.get('card')
        zone = p.gy if ans.get('zone') == 'gy' else p.hand
        if isinstance(i, int) and 0 <= i < len(zone): return zone[i].name
    return None


def _yours_text(p, ans, label):
    do = ans.get('do')
    if do == 'pass': return 'Pass (nothing more this phase)'
    if label: return f'Cast {label}'
    if do == 'land':
        i = ans.get('card'); zone = p.gy if ans.get('zone') == 'gy' else p.hand
        return f"Play {zone[i].name}" if isinstance(i, int) and 0 <= i < len(zone) else 'Play a land'
    if do == 'use':
        if 'land' in ans: return 'Use a land ability'
        i = ans.get('perm')
        return f"Use {p.perms[i].name}" if isinstance(i, int) and 0 <= i < len(p.perms) else 'Use an ability'
    return str(do)


def _snapshot(g, p):
    from commander_sim.ai import search
    from commander_sim.play import mana
    g2 = search.clone(g)
    p2 = g2.players[g.players.index(p)]
    p2.floatA = getattr(p2, 'floatA', 0) + (mana.pool_of(p).total() if getattr(p, 'pool', None) is not None else 0)
    g2.search_n = 0
    return g2, p2


def _main_job(e, g, p, label):
    """a generator: the comparison for main-phase decision e, one playout per step (the copy is taken now)"""
    from commander_sim.ai import brain, search
    snap, sp = _snapshot(g, p)
    post = getattr(g, 'step', None) == 'main2'

    def run():
        saved = E.CUR_G
        E.CUR_G = snap
        try:
            opts = brain.main_options(snap, sp, post)
        finally:
            E.CUR_G = saved
        real = [o for o in opts if o[2] is not None]
        stop = next(o for o in opts if o[2] is None)
        ranked = sorted(real, key=lambda o: -o[0])[:search.TOP_K]
        cands = ranked + [stop]
        labels = [o[1] for o in cands]
        mine_label = stop[1] if label == 'stop' else label          # passing: 'stop' or 'stop (hold mana)'
        e.yours_key = mine_label
        if mine_label not in labels:
            mine = next((o for o in opts if o[1] == mine_label), None)
            if mine is None:
                e.note = 'not compared: the AI found no way to make this play from here'
                e.scored = False; return
            cands.append(mine); labels.append(mine_label)
        if len(cands) < 2:
            e.ai = labels[0]; e.note = 'forced: only one option'; return
        base = 0x5EED + e.n                                            # the same futures for every option
        tot = {lb: 0.0 for lb in labels}
        for r in range(search.ROLLOUTS):
            for lb in labels:
                tot[lb] += search.playout_main(snap, sp, post, lb, 0, base, r)
                yield
        e.scores = {lb: tot[lb] / search.ROLLOUTS for lb in labels}
        ai_set = [o[1] for o in ranked + [stop]]
        e.ai = max(ai_set, key=lambda lb: e.scores[lb])
        e.scored = True
    return run()


# ------------------------------------------------------------------ attacks
def _attack_label(g, p, req, ans):
    """your declaration as the look-ahead's plans see attacks: (defender index, 'none' | 'all' | 'filtered')"""
    opps = [i for i, q in enumerate(g.players) if q is not p and q.alive]
    picked = ans if isinstance(ans, list) else []
    if not picked: return (opps[0], 'none') if opps else None
    mode = 'all' if len(set(picked)) == len(req.choices) else 'filtered'
    return (opps[0], mode)            # the defender is asked next; the first opponent stands in for it


def _attack_text(g, p, req, ans):
    picked = ans if isinstance(ans, list) else []
    if not picked: return 'No attack'
    return 'Attack with ' + ', '.join(req.choices[i].split(' (')[0] for i in picked if 0 <= i < len(req.choices))


def _attack_job(e, g, p, label):
    from commander_sim.ai import search
    snap, sp = _snapshot(g, p)
    opps = [i for i, q in enumerate(g.players) if q is not p and q.alive]

    def run():
        if label is None or not opps:
            e.note = 'not compared'; return
        cands = [(i, m) for i in opps for m in ('filtered', 'all')] + [(opps[0], 'none')]
        base = 0xA77 + e.n
        tot = {c: 0.0 for c in cands}
        for r in range(search.ROLLOUTS):
            for c in cands:
                tot[c] += search.playout_attack(snap, sp, c, base, r)
                yield
        e.scores = {_attack_name(g, c): tot[c] / search.ROLLOUTS for c in cands}
        best = max(cands, key=lambda c: tot[c])
        e.ai = _attack_name(g, best); e.ai_plan = best
        e.yours_key = _attack_name(g, label) if label in cands else _attack_name(g, (opps[0], label[1]))
        e.scored = True
    return run()


def _attack_name(g, c):
    who = E.NAME(g.players[c[0]])
    return {'none': 'No attack', 'all': f'Attack {who} with everything', 'filtered': f'Attack {who} (safe attackers)'}[c[1]]
