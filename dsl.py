"""Card ability language (DSL) -- part 2: the interpreter.

Cards whose CD has a `.dsl` ability list are run entirely by this module:
  - fire(g, event, ...)        triggered abilities (enters, dies, attacks, cast, draw, upkeep, ...)
  - resolve_spell(g, p, c, ctx) instants and sorceries
  - pt / has_kw / cost_delta / token_mult / no_lifegain   static abilities
  - ability_options(g, p, ...)  activated and loyalty abilities as choices for the AI
  - card_value(g, p, c)        how much the AI wants to cast a card it has no hand-written priority for
Hand-tagged cards keep using the original engine code; both kinds play together in one game.
"""
import json, os, sys
import engine as E
from engine import (has, army_of, epow, etgh, pval, threat, lose_life, gain, draw, mill, amass, make_tokens,
                    enter, leave, die, bounce, exile_perm, tuck, apply_removal, land_ramp, untargetable, can_pay,
                    pay, total_mana, log, NAME, check_state, discard_worst, edict, once_per_turn)
from dsl_parse import compile_card

MAX_DEPTH = 6


# ------------------------------------------------------------------ small helpers
def active(g):
    return bool(getattr(g, 'dsl_on', False))


def abil(cd, kind=None):
    ab = getattr(cd, 'dsl', None) or []
    return ab if kind is None else [a for a in ab if a.get('type') == kind]


def controller(m):
    return m.owner


def num(g, p, n, ctx=None, src=None):
    ctx = ctx or {}
    if isinstance(n, int): return n
    if n is None: return 1
    if n == 'X': return max(1, ctx.get('x', 3))
    if n == 'creatures_you_control': return sum(1 for m in p.perms if m.creature and not m.phased)
    if n == 'cards_in_hand': return len(p.hand)
    if n == 'opponents': return len(g.opps(p))
    if n == 'spells_this_turn': return p.spells_this_turn
    if n == 'devotion_B': return min(10, sum(x.cd.pips.count('B') for x in p.perms if x.cd is not None and x.cd.perm))
    if n.startswith('target_'):
        t = ctx.get('target_snapshot') or {}
        return {'target_power': t.get('pow', 2), 'target_toughness': t.get('tgh', 2), 'target_mv': t.get('mv', 2)}.get(n, 2)
    if n == 'last_lost': return ctx.get('lost', 0)
    if n == 'event_spell_mv': return ctx.get('spell').cmc if ctx.get('spell') is not None else 2
    if n == 'source_power': return epow(g, src) if src is not None and hasattr(src, 'owner') else 2
    if n == 'counters_on_self': return max(0, src.plus) if src is not None and hasattr(src, 'plus') else 0
    if n.startswith('type_you:') or n.startswith('type_all:'):
        t = n.split(':', 1)[1]
        qs = [p] if n.startswith('type_you') else [q for q in g.players if q.alive]
        return sum(1 for q in qs for m in q.perms if not m.phased and E.has_type(m, t))
    return 2


TYPE_OK = {
    'creature': lambda m: m.creature, 'artifact': lambda m: m.cd is not None and 'A' in m.cd.types,
    'enchantment': lambda m: m.cd is not None and 'E' in m.cd.types, 'planeswalker': lambda m: m.cd is not None and 'P' in m.cd.types,
    'land': lambda m: False, 'nonland': lambda m: True, 'permanent': lambda m: True,
    'cp': lambda m: m.creature or (m.cd is not None and 'P' in m.cd.types),
    'ac': lambda m: m.creature or (m.cd is not None and 'A' in m.cd.types),
    'acp': lambda m: m.creature or (m.cd is not None and ('A' in m.cd.types or 'P' in m.cd.types)),
    'ce': lambda m: m.creature or (m.cd is not None and 'E' in m.cd.types),
    'ae': lambda m: m.cd is not None and ('A' in m.cd.types or 'E' in m.cd.types),
    'nonartifact_creature': lambda m: m.creature and not (m.cd is not None and 'A' in m.cd.types),
    'noncreature': lambda m: not m.creature,
}


def matches(g, p, m, f, src=None):
    """does permanent m satisfy filter f, from the point of view of player p (the ability's controller)?"""
    if m.phased: return False
    f = f or {}
    t = f.get('type')
    if t and t in TYPE_OK and not TYPE_OK[t](m): return False
    c = f.get('controller')
    if c == 'you' and m.owner is not p: return False
    if c == 'opp' and m.owner is p: return False
    if f.get('other') and src is not None and m is src: return False
    cd = m.cd
    if 'max_mv' in f and cd is not None and cd.cmc > f['max_mv']: return False
    if 'min_mv' in f and (cd is None or cd.cmc < f['min_mv']): return False
    if 'max_pow' in f and epow(g, m) > f['max_pow']: return False
    if 'min_pow' in f and epow(g, m) < f['min_pow']: return False
    if f.get('nontoken') and m.token: return False
    if f.get('nonlegendary') and cd is not None and 'leg' in cd.tags: return False
    if f.get('legendary') and not (cd is not None and 'leg' in cd.tags): return False
    if f.get('subtype') and f['subtype'] not in ('', None):
        st = f['subtype']
        if st == 'warrior':
            if not m.warrior and not E.has_type(m, 'warrior'): return False
        elif not E.has_type(m, st): return False
    return True


def card_matches(c, f):
    f = f or {}
    t = f.get('type')
    if t in (None, 'any', 'permanent'): ok = True if t != 'permanent' else (c.perm or c.land)
    elif t == 'creature': ok = c.creature
    elif t == 'land': ok = c.land and (not f.get('basic') or c.name in ('Forest', 'Island', 'Plains', 'Swamp', 'Mountain'))
    elif t == 'artifact': ok = 'A' in c.types
    elif t == 'enchantment': ok = 'E' in c.types
    elif t == 'instant_or_sorcery': ok = c.instant or c.sorcery
    elif t == 'noncreature': ok = not c.creature and not c.land
    else: ok = True
    if not ok: return False
    if 'max_mv' in f and c.cmc > f['max_mv']: return False
    if 'min_mv' in f and c.cmc < f['min_mv']: return False
    if 'max_pow' in f and c.pow > f['max_pow']: return False
    if f.get('nonlegendary') and 'leg' in c.tags: return False
    return True


def players(g, p, w, ctx=None):
    ctx = ctx or {}
    opps = g.opps(p)
    if w == 'you': return [p]
    if w == 'each_opponent': return opps
    if w == 'each_player': return [q for q in g.players if q.alive]
    if w in ('target_opponent', 'target_player'):
        return [max(opps, key=lambda q: threat(g, p, q))] if opps else []
    if w == 'target_controller':
        t = ctx.get('target_owner'); return [t] if t is not None and t.alive else []
    if w in ('event_player', 'defending_player'):
        t = ctx.get('event_player'); return [t] if t is not None and t.alive else []
    return [p]


HARMFUL = ('destroy', 'exile', 'bounce', 'tuck', 'damage', 'tap')


def choose_target(g, p, sel, harmful, ctx, src=None, spell=None):
    """pick the most sensible legal target (or reuse the one the AI picked when casting)"""
    f = dict(sel.get('filter') or {})
    pre = ctx.get('target')
    if pre is not None and pre in pre.owner.perms and matches(g, p, pre, f, src): return pre
    cands = []
    for q in g.players:
        if not q.alive: continue
        for m in q.perms:
            if not matches(g, p, m, f, src): continue
            if m.owner is not p and untargetable(g, m): continue
            srccol = spell.pips if spell is not None else (E.colors_of(src) if src is not None and hasattr(src, 'owner') else '')
            if m.owner is not p and E.protected_from(g, m, srccol): continue
            cands.append(m)
    if harmful:
        opp = [m for m in cands if m.owner is not p]
        if opp: return max(opp, key=lambda m: pval(g, m))
        if f.get('controller') != 'you': return None
        return min(cands, key=lambda m: pval(g, m)) if cands else None     # a cost on your own: the least valuable
    mine = [m for m in cands if m.owner is p]
    return max(mine, key=lambda m: pval(g, m)) if mine else (max(cands, key=lambda m: pval(g, m)) if cands else None)


def select(g, p, sel, harmful, ctx, src=None, spell=None):
    s = sel.get('sel')
    if s == 'self': return [src] if src is not None and hasattr(src, 'owner') and src in src.owner.perms else []
    if s == 'all':
        return [m for q in g.players if q.alive for m in list(q.perms) if matches(g, p, m, sel.get('filter'), src)]
    if s == 'event': return [ctx['event_perm']] if ctx.get('event_perm') is not None else []
    t = choose_target(g, p, sel, harmful, ctx, src, spell)
    return [t] if t is not None else []


def snapshot(g, m, ctx):
    ctx['target_snapshot'] = {'pow': epow(g, m), 'tgh': m.tgh + m.plus, 'mv': m.cd.cmc if m.cd is not None else 0}
    ctx['target_owner'] = m.owner


# ------------------------------------------------------------------ effects
def execute(g, p, effects, src=None, ctx=None, spell=None, depth=0):
    ctx = ctx if ctx is not None else {}
    for e in effects:
        if g.over or not p.alive: return
        try:
            run(g, p, e, src, ctx, spell, depth)
        except (KeyError, TypeError, ValueError, AttributeError, IndexError) as err:   # never crash a game on odd card data
            log(f'      [dsl: skipped {e.get("do")} ({err.__class__.__name__})]', g)
    check_state(g)


def run(g, p, e, src, ctx, spell, depth):
    d = e['do']; opps = g.opps(p)
    kind = 'burn' if spell is not None else 'triggers'
    if d == 'draw':
        for q in players(g, p, e.get('who', 'you'), ctx):
            if e.get('optional') and len(q.library) <= 12: continue       # "you may draw": don't deck yourself
            draw(g, q, num(g, p, e.get('n'), ctx, src))
    elif d == 'damage':
        n = num(g, p, e.get('n'), ctx, src) + (1 if has(p, 'thor') else 0); to = e.get('to', {})
        if to.get('sel') == 'any_target':
            srccol = spell.pips if spell is not None else (E.colors_of(src) if src is not None and hasattr(src, 'owner') else '')
            tg = [m for q in opps for m in q.perms if m.creature and not untargetable(g, m) and etgh(g, m) <= n
                  and not E.protected_from(g, m, srccol)]
            best = max(tg, key=lambda m: pval(g, m)) if tg else None
            lethal = [q for q in opps if q.life <= n]
            if lethal: lose_life(g, lethal[0], n, p, kind=kind)
            elif best is not None and pval(g, best) >= 3: snapshot(g, best, ctx); apply_removal(g, p, best, f'dmg{n}', spell)
            elif opps: lose_life(g, min(opps, key=lambda q: q.life), n, p, kind=kind)
        elif to.get('sel') == 'player':
            for q in players(g, p, to.get('who', 'each_opponent'), ctx):
                if q is not p: lose_life(g, q, n, p, kind=kind)
        else:
            for m in select(g, p, to, True, ctx, src, spell):
                if to.get('sel') == 'target': snapshot(g, m, ctx)
                if etgh(g, m) <= n:
                    if to.get('sel') == 'target': apply_removal(g, p, m, f'dmg{n}', spell)
                    else: die(g, m, 'destroy')
    elif d in ('destroy', 'exile', 'bounce', 'tuck'):
        sel = e.get('what', {})
        ms = select(g, p, sel, True, ctx, src, spell)
        if sel.get('sel') == 'all':
            import ais
            hit = {m.owner for m in ms}
            prot = {q: ais.wipe_response(g, q, {'destroy': 'destroy', 'exile': 'exile', 'bounce': 'evac'}.get(d, 'destroy'), p) for q in hit}
            for m in ms:
                if m not in m.owner.perms or m.phased: continue
                if prot.get(m.owner) == 'indes' and d == 'destroy': continue
                if d == 'destroy': die(g, m, 'destroy')
                elif d == 'exile': exile_perm(g, m)
                elif d == 'bounce':
                    if m.token: leave(g, m)
                    else: bounce(g, m)
                else: tuck(g, m)
        else:
            for m in ms:
                snapshot(g, m, ctx)
                apply_removal(g, p, m, d, spell)
    elif d == 'token':
        n = num(g, p, e.get('n'), ctx, src); kws = e.get('keywords', [])
        made = make_tokens(g, p, n, int(e.get('pow', 1)), int(e.get('tgh', e.get('pow', 1))), fly='flying' in kws,
                           dt='deathtouch' in kws, lifelink='lifelink' in kws, warrior=e.get('warrior', False),
                           attacking=e.get('attacking', False), sick=not ('haste' in kws or e.get('attacking')),
                           types=e.get('types'))
        if e.get('attacking') and 'new_attackers' in ctx: ctx['new_attackers'] += made
    elif d == 'treasure': p.treasures += num(g, p, e.get('n'), ctx, src)
    elif d == 'clue': p.clues += num(g, p, e.get('n'), ctx, src)
    elif d == 'gain_life':
        n = num(g, p, e.get('n'), ctx, src) * int(e.get('mult', 1))
        if e.get('per_opponent'): n *= max(1, len(opps))
        for q in players(g, p, e.get('who', 'you'), ctx): gain(q, n)
    elif d == 'lose_life':
        n = num(g, p, e.get('n'), ctx, src)
        for q in players(g, p, e.get('who', 'each_opponent'), ctx):
            before = q.life; lose_life(g, q, n, p if q is not p else q, kind='drain')
            if q is not p: ctx['lost'] = ctx.get('lost', 0) + (before - q.life)
    elif d == 'counters':
        n = num(g, p, e.get('n'), ctx, src) * counter_mult(g, p)
        for m in select(g, p, e.get('what', {'sel': 'self'}), False, ctx, src, spell):
            m.plus += n
    elif d == 'pump':
        dp, dt = num(g, p, e.get('pow'), ctx, src), num(g, p, e.get('tgh'), ctx, src)
        for m in select(g, p, e.get('what', {}), dp < 0, ctx, src, spell):
            a, b = g.eot_pt.get(id(m), (0, 0)); g.eot_pt[id(m)] = (a + dp, b + dt)
        if dt < 0:
            for q in g.players:
                for m in list(q.perms):
                    if m.creature and etgh(g, m) <= 0: die(g, m, 'destroy')
    elif d == 'set_pt':
        p.pump = max(p.pump, total_mana(g, p))
    elif d == 'grant':
        for m in select(g, p, e.get('what', {}), False, ctx, src, spell):
            g.eot_kw.setdefault(id(m), set()).add(e.get('keyword'))
    elif d == 'search':
        search(g, p, e, ctx)
    elif d == 'reanimate':
        pool = [(c, p) for c in p.gy if card_matches(c, e.get('filter') or {'type': 'creature'})]
        if e.get('from') == 'any':
            pool += [(c, q) for q in g.opps(p) for c in q.gy if card_matches(c, e.get('filter') or {'type': 'creature'})]
        if pool:
            c, q = max(pool, key=lambda x: (x[0].bomb, x[0].pow, x[0].cmc))
            q.gy.remove(c); enter(g, p, c, orig=q)
            log(f'    {c.name} returns to the battlefield', g)
    elif d == 'regrow':
        cs = [c for c in p.gy if card_matches(c, e.get('filter'))]
        if cs:
            c = max(cs, key=lambda c: (c.bomb, c.cmc)); p.gy.remove(c); p.hand.append(c)
    elif d == 'discard':
        for q in players(g, p, e.get('who', 'each_opponent'), ctx):
            n = num(g, p, e.get('n'), ctx, src)
            if q is p and e.get('random'):          # "discard a card at random" (Gamble)
                for _ in range(n):
                    if q.hand: E.discard_index(g, q, g.rng.randrange(len(q.hand)))
            elif q is p: discard_worst(g, q, n)
            else:
                for _ in range(n):
                    if q.hand: E.discard_index(g, q, g.rng.randrange(len(q.hand)))
    elif d == 'mill':
        for q in players(g, p, e.get('who', 'you'), ctx): mill(g, q, num(g, p, e.get('n'), ctx, src))
    elif d == 'sacrifice':
        f = e.get('filter') or {'type': 'creature'}
        for q in players(g, p, e.get('who', 'each_opponent'), ctx):
            for _ in range(num(g, p, e.get('n', 1), ctx, src)):
                cs = [m for m in q.perms if matches(g, q, m, dict(f, controller=None))]
                if cs: die(g, min(cs, key=lambda m: pval(g, m)), 'sac')
    elif d == 'add_mana': p.floatA += num(g, p, e.get('n'), ctx, src)
    elif d == 'extra_combat': p.extra_combats += 1
    elif d == 'proliferate':
        for m in p.perms:
            if m.plus > 0: m.plus += 1
    elif d == 'double_counters':
        for m in p.perms:
            if m.plus > 0: m.plus *= 2
    elif d == 'amass': amass(g, p, num(g, p, e.get('n'), ctx, src))
    elif d == 'untap':
        sel = e.get('what', {}); f = (sel.get('filter') or {})
        if f.get('type') == 'land':
            for L in p.lands: L.tapped = False
        else:
            for m in select(g, p, sel, False, ctx, src, spell): m.tapped = False
    elif d == 'tap':
        for m in select(g, p, e.get('what', {}), True, ctx, src, spell): m.tapped = True
    elif d == 'phase_out':
        for m in select(g, p, e.get('what', {}), False, ctx, src, spell): m.phased = True
    elif d == 'blink':
        sel = e.get('what', {})
        if sel.get('upto') or sel.get('sel') == 'target':
            # only blink a nontoken creature that gets something from entering again
            cands = [m for m in p.perms if m.creature and m.cd is not None and not m.phased and m.cd is not p.cmd
                     and (etb_value(m.cd) > 0) and matches(g, p, m, sel.get('filter'), src)]
            ms = [max(cands, key=lambda m: etb_value(m.cd))] if cands else []
        else:
            ms = select(g, p, sel, False, ctx, src, spell)
        for m in ms:
            if m.cd is not None and m.owner is p and not m.token:
                cd = m.cd; leave(g, m); enter(g, p, cd); log(f'    {cd.name} is blinked', g)
    elif d == 'opp_ramp':
        q = ctx.get('target_owner')
        if q is not None and q.alive: land_ramp(g, q, 1, True)
    elif d == 'opp_token':
        q = ctx.get('target_owner')
        if q is not None and q.alive: make_tokens(g, q, 1, int(e.get('pow', 3)))
    elif d == 'opp_treasure':
        q = ctx.get('target_owner')
        if q is not None and q.alive: q.treasures += num(g, p, e.get('n'), ctx, src)
    elif d == 'exile_graveyard':
        for q in players(g, p, e.get('who', 'each_player'), ctx):
            q.exile.extend(q.gy); q.gy = []
    elif d == 'wheel':
        qs = players(g, p, e.get('who', 'each_player'), ctx)
        n = max([len(q.hand) for q in qs] + [0])
        for q in qs: E.discard_cards(g, q, list(q.hand))
        for q in qs: draw(g, q, n)
    elif d == 'modal':
        modes = e.get('modes', [])
        k = min(int(e.get('choose', 1)), len(modes))
        best = sorted(modes, key=lambda ms: -value_of(g, p, ms, spell))[:k]
        for ms in best: execute(g, p, ms, src, ctx, spell, depth + 1)
    elif d == 'counter_spell':
        pass                                      # counterspells are cast through the engine's response window
    elif d == 'oracle':                          # Thassa's Oracle
        x = sum(m.cd.pips.count('U') for m in p.perms if m.cd is not None and m.cd.perm and not m.phased)
        if x >= len(p.library):
            import ais; ais.win(g, p, 'combo'); return
        top = [p.library.pop() for _ in range(min(x, len(p.library)))]
        if top:
            best = max(top, key=lambda c: E.card_worth(g, p, c)); top.remove(best)
            g.rng.shuffle(top); p.library[:0] = top; p.library.append(best)
    elif d == 'narset_dig':                      # Narset -2: top four, take a noncreature nonland card, rest on the bottom
        top = [p.library.pop() for _ in range(min(4, len(p.library)))]
        ok = [c for c in top if not c.creature and not c.land]
        if ok:
            c = max(ok, key=lambda c: E.card_worth(g, p, c)); top.remove(c); p.hand.append(c); p.seen_names.add(c.name)
        g.rng.shuffle(top); p.library[:0] = top
    elif d == 'put_land':                        # put a land card from your hand onto the battlefield
        ls = [c for c in p.hand if c.land]
        if ls:
            import ais
            c = max(ls, key=lambda c: (len(c.tags.get('c', '')), 'f' in c.tags)); p.hand.remove(c)
            p.lands.append(E.Land(c, ais.land_enters_tapped(p, c))); E.landfall(g, p)
            if E.POOL_RULES and 'f' in c.tags and p.lands and p.lands[-1].cd is c: ais.crack_fetch(g, p, p.lands[-1])
    elif d == 'ring_protection':                  # The One Ring: protection from everything until your next turn
        p.ring_prot = True
        log(f'    {NAME(p)} gains protection from everything until their next turn', g)


def search(g, p, e, ctx):
    f = e.get('filter') or {}; to = e.get('to', 'hand'); n = int(e.get('n', 1))
    for _ in range(n):
        if f.get('type') == 'land' and to == 'battlefield':
            land_ramp(g, p, 1, e.get('tapped', False)); continue
        if f.get('type') == 'land':
            E.land_to_hand(g, p); continue
        cands = [c for c in p.library if card_matches(c, f)]
        if not cands: return
        if not f or f.get('type') in (None, 'any'):
            import ais
            nm = ais.tutor_pick(g, p, 'any')
            c = next((x for x in cands if x.name == nm), None) or max(cands, key=lambda c: card_value(g, p, c))
        else:
            wish = __import__('pool_ai').wish_list(g, p) if p.key not in E.IDENT else []
            want = [c for c in cands if c.name in wish]
            c = min(want, key=lambda c: wish.index(c.name)) if want else max(cands, key=lambda c: (card_value(g, p, c), c.cmc))
        p.library.remove(c)
        a = E.agent_for(g, p)
        if a is not None: E.agent_take(g, a, p, c); continue      # Opposition Agent
        if to == 'battlefield':
            if c.land: p.lands.append(E.Land(c, e.get('tapped', False)))
            else: enter(g, p, c)
        elif to == 'graveyard': p.gy.append(c)
        elif to == 'top':
            g.rng.shuffle(p.library); p.library.append(c)
        else:
            p.hand.append(c); p.seen_names.add(c.name)
        p.stats['tutored'] += 1
        log(f'    {NAME(p)} searches for {c.name}', g)


def resolve_spell(g, p, c, ctx):
    ctx = dict(ctx or {})
    for a in abil(c):
        if a.get('type') in ('spell',):
            execute(g, p, a['effects'], None, ctx, spell=c)


# ------------------------------------------------------------------ triggers
def fire(g, event, **kw):
    if not active(g) or g.over: return
    depth = getattr(g, 'dsl_depth', 0)
    if depth > MAX_DEPTH: return
    g.dsl_depth = depth + 1
    try:
        for q in list(g.players):
            if not q.alive: continue
            srcs = list(q.perms)
            x = kw.get('dying')
            if x is not None and x.owner is q and x not in srcs: srcs.append(x)      # "when ~ dies"
            for src in srcs:
                if src.cd is None or not getattr(src.cd, 'dsl', None) or (src.phased and src is not x): continue
                for a in src.cd.dsl:
                    if a.get('type') != 'triggered' or a.get('event') != event: continue
                    if src is x and src not in q.perms and a.get('source') != 'self': continue
                    reps = trigger_copies(g, q, src, event, kw)
                    for ctx in trigger_matches(g, q, src, a, kw):
                        for _ in range(reps):
                            execute(g, q, a['effects'], src, ctx)
                            if g.over: return
    finally:
        g.dsl_depth = depth


def trigger_copies(g, q, src, event, kw):
    """Veyran: your instant/sorcery casts make your permanents' triggers fire an extra time.
    Harmonic Prodigy: triggers of your Shamans and other Wizards fire an extra time."""
    n = 1
    c = kw.get('spell')
    if event == 'cast' and kw.get('caster') is q and c is not None and (c.instant or c.sorcery) and has(q, 'veyran'):
        n += 1
    t = src.cd.tags if src.cd is not None else {}
    if ('shaman' in t or 'wizard' in t) and 'prodigy' not in t and has(q, 'prodigy'):
        n += 1
    if event == 'dies' and g.hooks: n += E.CI.total(g, 'trigger_copies', q, 'dies', kw.get('perm'))     # Teysa
    return n


def trigger_matches(g, q, src, a, kw):
    """yield one ctx per time the ability triggers"""
    ev = a['event']; s = a.get('source'); w = a.get('who')
    base = {'x': 3}
    if ev in ('etb', 'dies'):
        m = kw.get('perm'); owner = kw.get('owner', m.owner if m is not None else None)
        if m is None: return
        if s == 'self':
            if a.get('if_cast') and not kw.get('was_cast'): return          # "when ~ enters, if you cast it"
            if m is src or (ev == 'dies' and kw.get('card') is src.cd and m is src): yield dict(base, event_perm=m)
            return
        if not m.creature: return
        if a.get('other') and m is src: return
        if 'min_pow' in a and epow(g, m) < a['min_pow']: return
        if s == 'you_creature' and owner is q: yield dict(base, event_perm=m)
        elif s == 'opp_creature' and owner is not q: yield dict(base, event_perm=m, event_player=owner)
        elif s == 'any_creature': yield dict(base, event_perm=m)
    elif ev == 'attack':
        atk = kw.get('attackers', []); pl = kw.get('player')
        ctx = dict(base, event_player=kw.get('defender'), new_attackers=kw.get('new', []))
        if s == 'self' and src in atk: yield ctx
        elif s == 'you_any' and pl is q and atk: yield ctx
        elif s == 'you_each' and pl is q:
            for m in atk:
                if a.get('subtype') == 'warrior' and not m.warrior: continue
                yield dict(ctx, event_perm=m)
        elif s == 'equipped' and any(src.attached is m for m in atk): yield ctx
    elif ev == 'combat_damage':
        m = kw.get('attacker')
        ctx = dict(base, event_player=kw.get('defender'), event_perm=m)
        if s == 'self' and m is src: yield ctx
        elif s == 'you_creature' and m is not None and m.owner is q:
            if once_per_turn(g, q, f'cd{id(src)}{id(a)}{g.round}'): yield ctx
        elif s == 'equipped' and src.attached is m: yield ctx
    elif ev == 'cast':
        caster = kw.get('caster'); c = kw.get('spell')
        if w == 'you' and caster is not q: return
        if w == 'opponent' and caster is q: return
        sp = a.get('spell', 'any')
        ok = {'any': True, 'instant_or_sorcery': c.instant or c.sorcery, 'noncreature': not c.creature,
              'creature': c.creature, 'artifact': 'A' in c.types, 'enchantment': 'E' in c.types,
              'legendary': 'leg' in c.tags}.get(sp, True)
        if not ok: return
        if a.get('nth') and caster.spells_this_turn != a['nth']: return
        yield dict(base, spell=c, event_player=caster)
    elif ev == 'draw':
        pl = kw.get('player')
        if w == 'you' and pl is q: yield dict(base)
        elif w == 'opponent' and pl is not q: yield dict(base, event_player=pl)
    elif ev in ('upkeep', 'end_step'):
        pl = kw.get('player')
        if w == 'you' and pl is q: yield dict(base)
        elif w == 'opponent' and pl is not q: yield dict(base, event_player=pl)
        elif w == 'any': yield dict(base, event_player=pl)
    elif ev == 'landfall':
        if kw.get('player') is q: yield dict(base)


# ------------------------------------------------------------------ static abilities
def statics(g, kind):
    """(controller, source, ability) for every static / replacement ability of this kind on the battlefield.
    The index is rebuilt when the set of permanents changes (a pure cache)."""
    sig = getattr(g, 'bf_ver', 0)                 # bumped whenever a card permanent enters, leaves or changes control
    idx = getattr(g, 'static_idx', None)
    if idx is None or idx[0] != sig:
        by = {}
        for q in g.players:
            for src in q.perms:
                if src.cd is None or not getattr(src.cd, 'dsl', None): continue
                for a in src.cd.dsl:
                    if a.get('type') in ('static', 'replacement'):
                        k = a.get('static') or a.get('replace')
                        by.setdefault(k, []).append((q, src, a))
        idx = g.static_idx = (sig, by)
    for q, src, a in idx[1].get(kind, ()):
        if q.alive and not src.phased and src.owner is q and src in q.perms: yield q, src, a


def pt(g, m):
    dp = dt = 0
    e = getattr(g, 'eot_pt', {}).get(id(m))
    if e: dp += e[0]; dt += e[1]
    if getattr(g, 'auras', None) or getattr(g, 'selfpt', None) or getattr(m.owner, 'elspeth_emblem', False):
        a, b = E.CI.attached_bonus(g, m); dp += a; dt += b
    if not active(g): return dp, dt
    for q, src, a in statics(g, 'anthem'):
        if m.creature and matches(g, q, m, a.get('filter'), src): dp += a.get('pow', 0); dt += a.get('tgh', 0)
    for q, src, a in statics(g, 'equip_bonus'):
        if src.attached is m: dp += a.get('pow', 0); dt += a.get('tgh', 0)
    for q, src, a in statics(g, 'bma_anthem'):                  # Beastmaster Ascension with 7+ quest counters
        if src.plus >= 7 and m.creature and m.owner is q: dp += 5; dt += 5
    if m.cd is not None and getattr(m.cd, 'dsl', None):
        for a in m.cd.dsl:
            if a.get('static') == 'self_scaling':
                k = num(g, m.owner, a.get('per'), {}, m); dp += a.get('pow', 1) * k
                if a.get('base0'): dt += k
    return dp, dt


def has_kw(g, m, kw):
    if kw in getattr(g, 'eot_kw', {}).get(id(m), ()): return True
    if m.cd is not None and kw in m.cd.kws and not m.neutered: return True
    if getattr(g, 'auras', None) and E.CI.attached_kw(g, m, kw): return True
    if g.hooks and E.CI.granted_kw(g, m, kw): return True
    if not active(g): return False
    if m.cd is not None and getattr(m.cd, 'dsl', None):
        for a in m.cd.dsl:
            if a.get('static') == 'unblockable' and kw == 'unblockable': return True
            if a.get('static') == 'cant_block' and kw == 'cant_block': return True
            if a.get('static') == 'self_keyword' and a.get('keyword') == kw: return True   # e.g. an indestructible artifact
    for q, src, a in statics(g, 'keyword'):
        if a.get('keyword') == kw and m.creature and matches(g, q, m, a.get('filter'), src): return True
    for q, src, a in statics(g, 'equip_keyword'):
        if a.get('keyword') == kw and src.attached is m: return True
    return False


ETB_TAGS = ('titan', 'archon', 'gray', 'wurm', 'rsd', 'witness', 'wall', 'atraxa', 'heir', 'suntitan', 'tokbig',
            'bowmasters', 'skate', 'mycoloth', 'recruit', 'prepare', 'draw', 'tok', 'tut')


def etb_value(cd):
    v = sum(2 for k in ETB_TAGS if k in cd.tags) + (3 if cd.tags.get('rem') and 'etb' in cd.tags else 0)
    for a in (cd.dsl or []):
        if a.get('type') == 'triggered' and a.get('event') == 'etb' and a.get('source') == 'self': v += 3
    return v


def protection(g, m):
    """colours m has protection from (from auto-modeled equipment)"""
    if not active(g): return ''
    cols = ''
    for q, src, a in statics(g, 'equip_protection'):
        if src.attached is m: cols += a.get('colors', '')
    return cols


def cost_delta(g, p, c):
    if not active(g): return 0
    d = 0
    for q, src, a in statics(g, 'cost'):
        mine = a.get('who') == 'you' and q is p
        theirs = a.get('who') == 'opponents' and q is not p
        if not (mine or theirs): continue
        sp = a.get('spell', 'any')
        ok = {'any': True, 'instant_or_sorcery': c.instant or c.sorcery, 'noncreature': not c.creature,
              'creature': c.creature, 'artifact': 'A' in c.types, 'enchantment': 'E' in c.types,
              'legendary': 'leg' in c.tags}.get(sp, True)
        if ok: d += a.get('amount', 0)
    return d


def token_mult(g, p):
    if not active(g): return 1
    k = 1
    for q, src, a in statics(g, 'tokens'):
        if q is p: k *= int(a.get('multiplier', 2))
    return k


def counter_mult(g, p):
    k = 1
    for q, src, a in statics(g, 'counters'):
        if q is p: k *= int(a.get('multiplier', 2))
    return k


def no_lifegain(g, p):
    if not active(g): return False
    return any(q is not p for q, src, a in statics(g, 'no_lifegain'))


# ------------------------------------------------------------------ AI: values and ability use
EFF_VALUE = {'narset_dig': 1.3, 'draw': 1.3, 'treasure': 0.8, 'clue': 0.6, 'gain_life': 0.15, 'amass': 0.7, 'proliferate': 1.0,
             'extra_combat': 3.0, 'add_mana': 0.5, 'mill': 0.1}


def value_of(g, p, effects, spell=None):
    v = 0.0
    for e in effects:
        d = e.get('do'); n = e.get('n', 1); n = n if isinstance(n, int) else 2
        if d in ('destroy', 'exile', 'bounce', 'tuck', 'damage'):
            sel = e.get('what') or e.get('to') or {}
            if sel.get('sel') == 'all':
                ms = select(g, p, sel, True, {}, None, spell)
                v += sum(pval(g, m) * (1 if m.owner is not p else -1.2) for m in ms) / 2.0
            elif sel.get('sel') in ('player', 'any_target') and d == 'damage':
                v += 0.6 * n * (len(g.opps(p)) if sel.get('who') == 'each_opponent' else 1)
            else:
                t = choose_target(g, p, sel, True, {}, None, spell)
                if t is None: v -= 1
                elif t.owner is p: v -= pval(g, t) + 0.5          # it would hit your own permanent
                else: v += pval(g, t) - 1.5
        elif d == 'token':
            v += 0.7 * n * (e.get('pow', 1) + 0.5 * len(e.get('keywords', [])))
        elif d in ('counters', 'pump'):
            v += 0.4 * n if d == 'counters' else 0.3
        elif d == 'lose_life':
            v += 0.5 * n * (len(g.opps(p)) if e.get('who') == 'each_opponent' else 1 if e.get('who') != 'you' else -0.3)
        elif d in ('search', 'reanimate', 'regrow'): v += 3.0
        elif d == 'sacrifice': v += 2.0 if e.get('who') != 'you' else -1.0
        elif d == 'discard': v += 0.8 if e.get('who') != 'you' else -0.5
        elif d == 'modal':
            ms = sorted((value_of(g, p, m, spell) for m in e.get('modes', [])), reverse=True)
            v += sum(ms[:int(e.get('choose', 1))])
        else: v += EFF_VALUE.get(d, 0.3) * (n if d in ('draw', 'treasure', 'clue', 'add_mana') else 1)
    return v


def card_value(g, p, c):
    """0-9 estimate of how good casting c is now (used when no hand-written priority exists)"""
    v = 0.0
    if c.creature: v += 1.0 + 0.45 * c.pow + 0.3 * sum(1 for k in ('fly', 'dt', 'lifelink', 'trample', 'haste') if k in c.tags)
    for a in abil(c):
        t = a.get('type')
        if t == 'spell': v += value_of(g, p, a['effects'], c)
        elif t == 'triggered':
            per = value_of(g, p, a['effects'])
            v += per * (1.0 if a.get('event') == 'etb' and a.get('source') == 'self' else 2.2)
        elif t == 'activated': v += 0.8 * value_of(g, p, a['effects'])
        elif t == 'loyalty': v += 0.6 * value_of(g, p, a['effects'])
        elif t in ('static', 'replacement'): v += 2.0 if a.get('static') != 'note' else 0
    return max(0.5, min(9.0, v))


def _pay_ability_cost(g, p, src, cost, dry):
    mana = cost.get('mana', '')
    gen, pips = E.parse_cost(mana) if mana else (0, '')
    if cost.get('tap') and (src.tapped or (src.creature and src.sick)): return False
    if cost.get('life') and p.life <= cost['life'] + 5: return False
    if cost.get('discard') and len(p.hand) < cost['discard']: return False
    fod = None
    if cost.get('sac') and cost['sac'] != 'self':
        f = {'type': cost['sac']} if cost['sac'] in TYPE_OK else {}
        fods = [m for m in p.perms if m is not src and matches(g, p, m, f) and (m.token or pval(g, m) < 3)]
        if not fods: return False
        fod = min(fods, key=lambda m: pval(g, m))
    if cost.get('other') or cost.get('tap_other') or cost.get('energy'): return False
    if not can_pay(g, p, gen, pips): return False
    if dry: return True
    pay(g, p, gen, pips)
    if cost.get('tap'): src.tapped = True
    if cost.get('life'): lose_life(g, p, cost['life'], p)
    if cost.get('discard'): discard_worst(g, p, cost['discard'])
    if fod is not None: die(g, fod, 'sac')
    if cost.get('sac') == 'self': die(g, src, 'sac')
    return True


def ability_options(g, p, sorcery_ok=True):
    """activated/loyalty abilities of DSL permanents as (utility, label, fn) choices"""
    out = []
    if not active(g): return out
    if sorcery_ok: out += equip_options(g, p)
    for src in list(p.perms):
        if src.cd is None or not getattr(src.cd, 'dsl', None) or src.phased: continue
        if E.stopped(g, src.cd.name): continue                    # Disruptor Flute
        for i, a in enumerate(src.cd.dsl):
            t = a.get('type')
            if t == 'activated':
                if a.get('sorcery') and not sorcery_ok: continue
                if E.POOL_RULES and all(e.get('do') == 'add_mana' for e in a['effects']) and a.get('cost', {}).get('sac'):
                    continue                       # sacrificing for mana nobody is waiting to spend: never proactive
                key = f'act{id(src)}{i}'
                uses = p.flag_turn.get(key + 'n', (None, 0))
                stamp = (g.round, p.key)
                cap = 8 if set(a.get('cost', {})) <= {'tap'} else 3          # tap-only abilities (untap engines) can recur
                if uses[0] == stamp and (a.get('once') or uses[1] >= cap): continue
                if not _pay_ability_cost(g, p, src, a.get('cost', {}), True): continue
                cost_v = 0.25 * sum(int(x) if x.isdigit() else 1 for x in a.get('cost', {}).get('mana', ''))
                cost_v += 1.5 if a.get('cost', {}).get('sac') == 'self' else 0
                u = value_of(g, p, a['effects']) - cost_v
                if 'ai_min' in a: u = max(u, a['ai_min'])                   # card data can insist the AI uses it

                def go(src=src, a=a, key=key, stamp=stamp):
                    if src not in p.perms or not _pay_ability_cost(g, p, src, a.get('cost', {}), False): return False
                    u0 = p.flag_turn.get(key + 'n', (None, 0))
                    p.flag_turn[key + 'n'] = (stamp, (u0[1] + 1) if u0[0] == stamp else 1)
                    log(f'  {NAME(p)} activates {src.name}', g)
                    execute(g, p, a['effects'], src, {})
                    return True
                out.append((u, f'{src.name} ability', go))
            elif t == 'loyalty' and sorcery_ok:
                if not hasattr(src, 'loyalty_used') or src.loyalty_used != (g.round, p.key):
                    loy = getattr(src, 'loyalty', None)
                    if loy is None:
                        loy = int(getattr(src.cd, 'start_loyalty', 3) or 3); src.loyalty = loy
                    if loy + a['loyalty'] < 0: continue
                    u = value_of(g, p, a['effects']) + 0.3 * a['loyalty']

                    def lo(src=src, a=a):
                        if getattr(src, 'loyalty_used', None) == (g.round, p.key) or src not in p.perms: return False
                        src.loyalty_used = (g.round, p.key); src.loyalty += a['loyalty']
                        log(f'  {NAME(p)} uses {src.name} ({a["loyalty"]:+d})', g)
                        execute(g, p, a['effects'], src, {})
                        if src.loyalty <= 0 and src in p.perms: leave(g, src); p.gy.append(src.cd)
                        return True
                    out.append((u, f'{src.name} {a["loyalty"]:+d}', lo))
    return out


def equip_options(g, p):
    """attach unattached auto-modeled equipment ("Equip {N}") to the creature that will use it best"""
    out = []
    for src in list(p.perms):
        if src.cd is None or not getattr(src.cd, 'dsl', None) or src.cd.tags.get('prot') == 'boots': continue
        eq = [a for a in src.cd.dsl if a.get('static') == 'equip_cost']
        if not eq: continue
        if src.attached is not None and src.attached in p.perms and not src.attached.phased: continue
        if E.stopped(g, src.cd.name): continue                    # Disruptor Flute stops equip
        n = int(eq[0].get('mana', 2))
        cands = [m for m in p.perms if m.creature and not m.phased and not m.noatk and not untargetable(g, m)]
        if not cands or not can_pay(g, p, n, ''): continue
        def score(m):
            return (3 if m.army else 0) + (2 if m.fly or has_kw(g, m, 'flying') else 0) + 0.3 * epow(g, m) - (1 if m.sick else 0)
        best = max(cands, key=score)
        bonus = sum(a.get('pow', 0) for a in src.cd.dsl if a.get('static') == 'equip_bonus')
        trig = sum(1 for a in src.cd.dsl if a.get('source') == 'equipped')
        u = 2.5 + 0.5 * bonus + 2.0 * trig - 0.3 * n

        def go(src=src, best=best, n=n):
            if not can_pay(g, p, n, '') or best not in p.perms: return False
            pay(g, p, n, ''); src.attached = best
            log(f'  {NAME(p)} equips {src.name} to {best.name}', g); return True
        out.append((u, f'equip {src.name}', go))
    return out


def rigid_abilities(g, p):
    """rigid AI: use every ability that looks worth it, best first"""
    for _ in range(6):
        opts = sorted([o for o in ability_options(g, p) if o[0] > 0.5], key=lambda o: -o[0])
        if not opts or not opts[0][2](): return


# ------------------------------------------------------------------ building cards
SAFE_TAGS = ('pow', 'tgh', 'fly', 'dt', 'vig', 'lifelink', 'haste', 'trample', 'reach', 'flash', 'convoke', 'leg',
             'human', 'warrior', 'shaman', 'wizard', 'c', 't', 'ck', 'f', 'amt', 'rock', 'dork', 'rite', 'bomb', 'fb',
             'perCreature', 'noatk', 'spectacle')


def hints(abilities, types):
    """AI hint tags derived from abilities (what the card is *for*); execution stays in the interpreter"""
    h = {}
    spell = 'I' in types or 'S' in types
    for a in abilities:
        if a.get('static') == 'uncounterable': h['unc'] = True
        if a.get('static') == 'free_counter': h['free'] = True
        if a.get('static') == 'sorcery_flash': h['gandalf'] = True
        if a.get('type') != 'spell': continue
        effs = list(a['effects'])
        for e in list(effs):
            if e.get('do') == 'modal':
                for ms in e.get('modes', []): effs += ms
        for e in effs:
            d = e.get('do')
            sel = e.get('what') or e.get('to') or {}
            if d in ('destroy', 'exile', 'bounce', 'tuck', 'damage') and sel.get('sel') in ('target', 'any_target') and 'rem' not in h:
                f = sel.get('filter') or {}
                h['rem'] = d if d != 'damage' else f'dmg{e.get("n") if isinstance(e.get("n"), int) else 3}'
                t = f.get('type')
                h['tgt'] = {'creature': 'c', 'cp': 'cp', 'acp': 'cap', 'ac': 'cap', 'ce': 'ce', 'nonland': 'nl',
                            'permanent': 'p', 'artifact': 'a', 'nonartifact_creature': 'cna'}.get(t, 'c' if d == 'damage' else 'nl')
                if sel.get('sel') == 'any_target': h['face'] = True
            if d in ('destroy', 'exile', 'bounce') and sel.get('sel') == 'all' and (sel.get('filter') or {}).get('type') in ('creature', 'nonland', 'permanent'):
                h['wipe'] = {'destroy': 'destroy', 'exile': 'exile', 'bounce': 'evac'}[d]
            if d == 'pump' and sel.get('sel') == 'all' and (e.get('tgh') or 0) <= -4: h['wipe'] = 'minus'
            if d == 'counter_spell':
                ft = (e.get('filter') or {}).get('type', 'any')
                h['ctr'] = {'noncreature': 'nc', 'creature': 'cre', 'instant_or_sorcery': 'ise'}.get(ft, 'any')
                if (e.get('filter') or {}).get('min_mv', 0) >= 4: h['ctr'] = 'mv4'
                if e.get('unless'): h['soft'] = e['unless']
            if d == 'draw' and spell: h['draw'] = h.get('draw', 0) + (e['n'] if isinstance(e.get('n'), int) else 2)
            if d == 'search' and e.get('to') in ('hand', 'top'):
                t = (e.get('filter') or {}).get('type')
                if t != 'land': h['tut'] = {'creature': 'cre', 'instant_or_sorcery': 'is', 'artifact': 'art', 'enchantment': 'ench'}.get(t, 'any')
            if d == 'reanimate': h['rean'] = 'animate' if e.get('from') == 'any' else 'evil'
            if d == 'token' and e.get('n') == 'X': h['tokx'] = True
    return h


def build_cd(name, rec_types, rec_cost, autotags, oracle, loyalty=None):
    """CD for a card driven by the interpreter"""
    abilities, unparsed = compile_card(name, oracle, rec_types, loyalty)
    base = {k: v for k, v in autotags.items() if k in SAFE_TAGS}
    base.update(hints(abilities, rec_types))
    tagstr = ' '.join(k if v is True else f'{k}={v}' for k, v in base.items())
    cd = E.CD(name, rec_types, rec_cost, tagstr)
    cd.dsl = abilities or None
    cd.unparsed = unparsed
    cd.start_loyalty = loyalty
    cd.source = 'scryfall+dsl'
    return cd


OVERRIDES_PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'cards_dsl.json')


def overrides():
    if os.path.exists(OVERRIDES_PATH):
        return json.load(open(OVERRIDES_PATH))
    return {}


if __name__ == '__main__':
    import scryfall, autotag
    for n, rec in scryfall.fetch(sys.argv[1:]).items():
        face = rec.get('card_faces', [rec])[0] if rec.get('card_faces') else rec
        r = autotag.autotag(rec)
        ab, bad = compile_card(n, face.get('oracle_text', rec.get('oracle_text', '')), r['types'], face.get('loyalty'))
        print(f'== {n}  ({r["types"]} {r["cost"]})')
        print(json.dumps(ab, indent=1))
        for b in bad: print('   not modeled:', b)


E.DSLMOD = sys.modules[__name__]