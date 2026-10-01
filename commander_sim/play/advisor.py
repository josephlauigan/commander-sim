"""What the AI would do at your decision (practice mode's Hint; the AI comparison in 3d builds on it).

The AI is asked on a copy of the game (search.clone, as the look-ahead does), never the real one, so asking changes
nothing: the game, its random streams and Undo's replay stay exactly as they were. In the copy your seat has no mana
pool; mana you have floating counts as mana of any colour there.

    hint(g, p, req) -> {'text': one line, 'detail': [lines], 'choice': the AI's pick or None}
"""
from commander_sim import engine as E


def _copy(g, p):
    from commander_sim.ai import search
    from commander_sim.play import mana
    pool = mana.pool_of(p).total() if getattr(p, 'pool', None) is not None else 0
    g2 = search.clone(g)
    p2 = g2.players[g.players.index(p)]
    p2.floatA = getattr(p2, 'floatA', 0) + pool
    return g2, p2


def _label(lbl, hand_names):
    if lbl.startswith('stop'): return 'Pass: play nothing more this phase' + (' (hold your mana up)' if 'hold' in lbl else '')
    if lbl in hand_names: return f'Cast {lbl}'
    if ' -> ' in lbl and lbl.split(' -> ')[0] in hand_names:
        spell, target = lbl.split(' -> ', 1)
        return f'Cast {spell} on {target}'
    return lbl


def main_phase(g, p):
    from commander_sim.ai import brain, search
    g2, p2 = _copy(g, p)
    post = getattr(g, 'step', None) == 'main2'
    saved = E.CUR_G
    E.CUR_G = g2
    try:
        opts = brain.main_options(g2, p2, post)
        names = {c.name for c in p2.hand} | ({p2.cmd.name} if p2.cmd_in_zone else set())
        if len(opts) < 2:
            o = opts[0] if opts else None
            return {'text': _label(o[1], names) if o else 'Nothing to do: pass.', 'detail': [], 'choice': o and o[1]}
        if search.enabled(g2, p2):
            search.LAST_SCORES[:] = []
            o = search.choose(g2, p2, post, opts)
            ranked = sorted(search.LAST_SCORES, key=lambda x: -x[1])
            detail = ['Look-ahead (each play tried on six copies of the game, played on to the end of your next turn):'] + \
                     [f'{_label(l, names)}: {v:+.1f}' for l, v in ranked]
        else:
            o = max(opts, key=lambda x: x[0])
            ranked = sorted(opts, key=lambda x: -x[0])[:4]
            detail = ["The AI's own ratings (no look-ahead in this game):"] + [f'{_label(l, names)}: {u:.1f}' for u, l, _ in ranked]
        if o is None: o = max(opts, key=lambda x: x[0])
        return {'text': _label(o[1], names), 'detail': detail, 'choice': o[1]}
    finally:
        E.CUR_G = saved


def attack(g, p):
    from commander_sim.ai import search
    g2, p2 = _copy(g, p)
    if not search.enabled(g2, p2):
        return {'text': 'Attack with the creatures that can attack safely.', 'detail': ['(no look-ahead in this game)'],
                'choice': None}
    saved = E.CUR_G
    E.CUR_G = g2
    try:
        res = search.choose_attack(g2, p2)
    finally:
        E.CUR_G = saved
    if res is None: return {'text': 'No attack.', 'detail': [], 'choice': None}
    who = E.NAME(g.players[res[0]])
    text = {'none': "Don't attack.", 'all': f'Attack {who} with everything.',
            'filtered': f'Attack {who} with the creatures that can attack safely.'}[res[1]]
    return {'text': text, 'detail': ['Look-ahead over attacking each opponent (safe attackers, or everything) and not attacking.'],
            'choice': list(res)}


def counter(g, p, spell):
    from commander_sim.ai import search
    cc = getattr(g, 'cur_cast', None)
    if cc is None or cc[0] is not spell: return None
    ctr = E.pick_counter(g, p, spell)
    if ctr is None: return {'text': f"Let {spell.name} resolve: you can't counter it now.", 'detail': [], 'choice': False}
    from commander_sim.play import mana
    g2, memo = search.clone(g, want_memo=True)                 # asked on a copy: the real game's counters stay put
    p2 = g2.players[g.players.index(p)]
    p2.floatA = getattr(p2, 'floatA', 0) + (mana.pool_of(p).total() if getattr(p, 'pool', None) is not None else 0)
    if not search.enabled(g2, p2):
        return {'text': f'You could counter {spell.name} with {ctr.name}; the hint needs the look-ahead AI to weigh it.',
                'detail': [], 'choice': None}
    caster2 = g2.players[g.players.index(g.active)]
    ctx2 = {k: (memo.get(id(v), v) if isinstance(v, (E.Perm, E.Player)) else v) for k, v in (cc[1] or {}).items()}
    saved = E.CUR_G
    E.CUR_G = g2
    try:
        yes = search.choose_counter(g2, p2, caster2, spell, ctx2, cc[2])
    finally:
        E.CUR_G = saved
    return {'text': f'Counter {spell.name} with {ctr.name}.' if yes else f'Let {spell.name} resolve.',
            'detail': ['Look-ahead: the game played on with and without countering it, to the end of your next turn.'],
            'choice': yes}


def hint(g, p, req):
    """the AI's advice for request req (waiting on the engine thread, so the game holds still)"""
    if req.kind == 'priority':
        stack = req.data.get('stack') or []
        if stack:
            spell = getattr(g, 'cur_cast', None)
            res = counter(g, p, spell[0]) if spell else None
            if res: return res
        if g.active is p and getattr(g, 'step', None) in ('main1', 'main2') and not stack:
            return main_phase(g, p)
        return {'text': 'Pass, unless you have an instant you want to use now.',
                'detail': ['(the AI only plays instants at this point in answer to something)'], 'choice': None}
    if req.kind == 'attack': return attack(g, p)
    return {'text': 'No hint for this kind of decision yet.', 'detail': [], 'choice': None}
