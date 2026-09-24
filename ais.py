from engine import *
import engine as E


# ======================================================== responses
def tide_response(g, actor, what, value, victim=None):
    """Tishana's Tidebinder (Sephiroth) counters an activated/triggered ability."""
    for q in g.opps(actor):
        if q.key != 'seph' or silenced(g, q): continue
        if victim is not None and victim is not q and value < 9: continue
        if value < 6: continue
        tb = [c for c in q.hand if 'tide' in c.tags]
        if not tb or not can_pay(g, q, 2, 'U'): continue
        pay(g, q, 2, 'U'); c = tb[0]; q.hand.remove(c)
        q.spells_this_turn += 1; on_cast(g, q, c)
        enter(g, q, c)
        q.stats['tide_' + what] += 1
        return True
    return False


def free_sac(p):
    g = E.CUR_G
    return any(m.cd is not None and 'sac' in m.cd.tags and not m.phased and not stopped(g, m.cd.name) for m in p.perms)


def blocked(g, p, name):
    """an activated ability of `name` is shut off by Disruptor Flute (counts it for the reports)"""
    if stopped(g, name):
        p.stats['flute_blocked'] += 1
        return True
    return False


ACTIVATED_ENGINES = ('Erebos, God of the Dead', "Vraska, Betrayal's Sting", 'Ral Zarek, Guest Lecturer',
                     'Nicol Bolas, Dragon-God', 'Rhys the Redeemed')


def flute_pick(g, p):
    """choose the card name that hurts opponents most: commanders with activated engines, combo pieces,
    equipment, sac outlets, planeswalkers ... (public information only: commanders and the battlefield)"""
    taken = {nm for f, nm in g.flutes if f in f.owner.perms}
    cands = {}

    def add(name, v):
        if name and name not in taken and v > cands.get(name, 0): cands[name] = v
    for q in g.opps(p):
        c = q.cmd
        if q.key == 'najeela': add(c.name, 9 + threat(g, p, q) * 0.05)     # WUBRG extra combats + recast tax
        else: add(c.name, (4 if q.cmd_in_zone else 1.5) + (1 if q.tax >= 2 else 0))
        for m in q.perms:
            if m.cd is None or m.phased: continue
            t = m.cd.tags; v = 0
            if 'assault' in t: v = 4 + (7 if has(q, 'sword') else 0)
            if 'sword' in t: v = 4 + (7 if has(q, 'assault') else 0)
            if 'cloak' in t or 'flail' in t or 'animist' in t: v = max(v, 3)
            if t.get('prot') == 'boots': v = max(v, 2.5)
            if 'sac' in t: v = max(v, 3.5 if q.key == 'seph' else 1)
            if 'clamp' in t or 'archivist' in t or 'rhys' in t or 'lidless' in t or t.get('fill') == 'tortured': v = max(v, 4)
            if 'aether' in t: v = max(v, 7)
            if 'P' in m.cd.types or m.cd.name in ACTIVATED_ENGINES: v = max(v, 4)
            if m.cd.dsl and any(x.get('type') in ('activated', 'loyalty') or x.get('static') == 'equip_cost' for x in m.cd.dsl):
                v = max(v, 3)
            add(m.cd.name, v)
        for L in q.lands:
            if L.cd.tags.get('passage') and army_of(q) is not None: add(L.cd.name, 3)
            if L.cd.tags.get('desert') and p.key == 'seph': add(L.cd.name, 5)
    if not cands: return None, 0
    name = max(cands, key=cands.get)
    return name, cands[name]


def pay_card(g, p, c):
    if not pay(g, p, c.generic, c.pips): return False
    p.hand.remove(c); p.gy.append(c); p.spells_this_turn += 1
    p.cast_names.add(c.name)
    on_cast(g, p, c)
    return True


def protect_response(g, owner, m, kind, actor):
    v = pval(g, m)
    if silenced(g, owner):                      # Conqueror's Flail: only non-spell responses
        if owner.key == 'seph' and kind in ('exile', 'bounce', 'tuck') and m.creature and free_sac(owner) and v >= 6:
            owner.stats['sac_saves'] += 1; die(g, m, 'sac'); return True
        return False
    if owner.key == 'seph':
        if v >= 6 and m.creature:
            hi = [c for c in owner.hand if c.tags.get('prot') == 'hi']
            if hi and kind != 'edict' and can_pay(g, owner, 1, 'G'):
                pay_card(g, owner, hi[0]); owner.stats['hi_used'] += 1; return True
            if kind in ('exile', 'bounce', 'tuck') and free_sac(owner) and not m.token:
                owner.stats['sac_saves'] += 1; die(g, m, 'sac'); return True
    elif owner.key == 'sauron':
        if m.army or v >= 5:
            sl = [c for c in owner.hand if c.tags.get('prot') == 'phase']
            if sl and can_pay(g, owner, 0, 'U'):
                pay_card(g, owner, sl[0]); m.phased = True; return True
            if m.army and epow(g, m) >= 7:
                nw = [c for c in owner.hand if c.tags.get('prot') == 'notw']
                if nw:
                    owner.hand.remove(nw[0]); owner.gy.append(nw[0]); return True
    elif owner.key == 'najeela':
        if v >= 5 and not m.token and m.creature:
            for c in owner.hand:
                if c.tags.get('prot') == 'blink' and can_pay(g, owner, c.generic, c.pips):
                    pay(g, owner, c.generic, c.pips); owner.hand.remove(c)
                    if c.creature: enter(g, owner, c)
                    else: owner.gy.append(c)
                    cd = m.cd; leave(g, m)
                    n = enter(g, owner, cd, orig=m.orig)
                    n.is_cmd = cd is owner.cmd
                    return True
    return False


def wipe_response(g, q, kind, caster):
    if q is caster or silenced(g, q): return None
    loss = sum(pval(g, m) for m in q.perms if m.creature or kind in ('rift', 'rebuke'))
    if loss < 6: return None
    if q.key == 'seph':
        if kind in ('destroy', 'dmg13', 'austere', 'nib'):
            hi = [c for c in q.hand if c.tags.get('prot') == 'hi']
            if hi and can_pay(g, q, 1, 'G'):
                pay_card(g, q, hi[0]); q.stats['hi_used'] += 1; return 'indes'
        if kind in ('exile', 'rift', 'rebuke') and free_sac(q):
            for m in list(q.perms):
                if m.creature and not m.token and m.cd.bomb and m.cd is not q.cmd:
                    q.stats['sac_saves'] += 1; die(g, m, 'sac')
    elif q.key == 'najeela':
        for c in list(q.hand):
            if c.tags.get('prot') == 'phase' and can_pay(g, q, c.generic, c.pips, 'convoke' in c.tags):
                pay_card(g, q, c)
                for m in q.perms: m.phased = True
                return 'all'
        if kind in ('destroy', 'dmg13', 'austere', 'nib'):
            for c in list(q.hand):
                if c.tags.get('prot') == 'indes' and can_pay(g, q, c.generic, c.pips):
                    pay_card(g, q, c); return 'indes'
    elif q.key == 'sauron':
        a = army_of(q)
        if a:
            sl = [c for c in q.hand if c.tags.get('prot') == 'phase']
            if sl and can_pay(g, q, 0, 'U'):
                pay_card(g, q, sl[0]); a.phased = True
    return None


def sauron_grounds_response(g, seph, value):
    """Sauron exiles all graveyards in response to a reanimation spell."""
    for q in g.opps(seph):
        if q.key != 'sauron' or value < 6: continue
        gl = [L for L in q.lands if L.cd.tags.get('desert') and not L.tapped and not stopped(g, L.cd.name)]
        if not gl: continue
        L = gl[0]; L.tapped = True
        if not can_pay(g, q, 2, '') or g.rng.random() > 0.9:
            L.tapped = False; continue
        pay(g, q, 2, ''); q.lands.remove(L); q.gy.append(L.cd)
        q.stats['grounds_used'] += 1
        if tide_response(g, q, 'grounds', 9, victim=seph):
            return False
        for p in g.players:
            if p.alive:
                p.exile.extend(p.gy); p.gy = []
        seph.stats['grounds_hit'] += 1
        return True
    return False


# ======================================================== sephiroth
def seph_bval(g, p, cd):
    v = cd.bomb or (cd.pow if cd.creature else 0)
    t = cd.tags
    if 'normgc' in t and any(q.key == 'najeela' for q in g.opps(p)): v += 2
    if 'mother' in t and any(q.key == 'sauron' for q in g.opps(p)): v += 1
    return v


def bomb_on_bf(p):
    return any(m.cd is not None and m.cd.bomb >= 6 for m in p.perms if not m.phased)


def rean_options(g, p):
    opts = []
    for c in p.hand:
        if 'rean' in c.tags: opts.append((c, 'hand', c.generic, c.pips))
    for c in p.gy:
        r = c.tags.get('rean')
        if not r: continue
        if r == 'rites': opts.append((c, 'fb', 3, 'W'))
        elif r == 'dread':
            fod = [m for m in p.perms if m.creature and (m.token or not m.cd.bomb)]
            if len(fod) >= 3: opts.append((c, 'fbsac', 0, ''))
        elif p.yawg: opts.append((c, 'yawg', c.generic, c.pips))
    return opts


def rean_targets(g, p, kind):
    res = []
    for c in p.gy:
        if c.creature and seph_bval(g, p, c) >= 4:
            if kind == 'persist' and 'leg' in c.tags: continue
            res.append((seph_bval(g, p, c), c, p))
    if kind in ('reanimate', 'animate', 'necro'):
        for q in g.opps(p):
            for c in q.gy:
                if c.creature and c.pow >= 5:
                    res.append((c.pow - 0.5, c, q))
    res.sort(key=lambda x: -x[0])
    return res


def seph_reanimate(g, p):
    opts = rean_options(g, p)
    if not opts: return False
    opts.sort(key=lambda o: o[2] + len(o[3]))
    for c, zone, cg, cp in opts:
        kind = c.tags['rean']
        tg = rean_targets(g, p, kind)
        if kind == 'reanimate':
            tg = [x for x in tg if p.life - x[1].cmc >= 12]
        if not tg: continue
        val, cd, src = tg[0]
        if not can_pay(g, p, cg, cp): continue
        pay(g, p, cg, cp)
        if zone == 'hand': p.hand.remove(c)
        else: p.gy.remove(c)
        if zone == 'fbsac':
            fod = sorted([m for m in p.perms if m.creature and (m.token or not m.cd.bomb)], key=lambda x: pval(g, x))[:3]
            for m in fod: die(g, m, 'sac')
        p.spells_this_turn += 1
        on_cast(g, p, c)
        if g.over or not p.alive: return True
        p.stats['rean_cast'] += 1; p.stats['spells_cast'] += 1
        p.cast_names.add(c.name)
        log(f'  Sephiroth casts {c.name} targeting {cd.name}' + (f" from {NAME(src)}'s graveyard" if src is not p else ''), g)
        dest = p.gy if zone == 'hand' else p.exile
        if not counter_window(g, p, c, val, {}):
            dest.append(c); return True
        if sauron_grounds_response(g, p, val):
            dest.append(c); return True
        seph_rean_resolve(g, p, c, {'rean_target': cd, 'rean_src': src})
        dest.append(c)
        check_state(g)
        return True
    return False


def seph_rean_resolve(g, p, c, ctx):
    if 'rean_target' not in ctx:                  # cast by a generic path: choose the best legal target now
        tg = rean_targets(g, p, c.tags.get('rean', 'animate'))
        if not tg: return
        ctx = dict(ctx, rean_target=tg[0][1], rean_src=tg[0][2])
    cd, src = ctx['rean_target'], ctx['rean_src']
    if cd not in src.gy: return
    src.gy.remove(cd)
    if c.tags.get('rean') == 'reanimate': lose_life(g, p, cd.cmc, p)
    was_removed = cd.name in p.removed_bombs
    m = enter(g, p, cd, orig=src)
    if c.tags.get('rean') == 'persist': m.plus = -1
    if c.tags.get('rean') == 'evil': m.plus += 2
    p.stats['rean_resolved'] += 1
    note_bomb(p, cd, was_removed)


def note_bomb(p, cd, was_removed=False):
    if cd.bomb >= 6 or cd.pow >= 6:
        log(f'    {cd.name} hits the battlefield' + (' (recovered after removal)' if was_removed else ''))
        p.stats['bombs_landed'] += 1
        if p.first_bomb is None: p.first_bomb = p.turns
        if was_removed: p.stats['bomb_recovered'] += 1


def seph_fill_resolve(g, p, kind, ctx):
    lib_bombs = sorted([c for c in p.library if c.creature and c.bomb >= 5], key=lambda c: -seph_bval(g, p, c))
    if kind == 'entomb' and lib_bombs:
        p.library.remove(lib_bombs[0]); p.gy.append(lib_bombs[0])
    elif kind == 'buried':
        for c in lib_bombs[:3]: p.library.remove(c); p.gy.append(c)
    elif kind == 'unmarked':
        nl = [c for c in lib_bombs if 'leg' not in c.tags]
        if nl: p.library.remove(nl[0]); p.gy.append(nl[0])
    elif kind == 'grisly':
        top = [p.library.pop() for _ in range(min(5, len(p.library)))]
        lands = [c for c in top if c.land]
        if lands and len(p.lands) < 7: top.remove(lands[0]); p.hand.append(lands[0])
        for c in top:
            p.gy.append(c)
            if any(k in c.tags for k in E.KEYSPELL): p.stats['key_milled'] += 1
    elif kind == 'dispute':
        p.treasures += 1
    if kind in ('entomb', 'buried', 'unmarked'): g.rng.shuffle(p.library)


def has_rean_access(g, p):
    return bool(rean_options(g, p))


def own_bomb_in_gy(g, p):
    return any(c.creature and seph_bval(g, p, c) >= 6 for c in p.gy)


def seph_fill(g, p):
    if own_bomb_in_gy(g, p): return False
    tut = any(c.tags.get('tut') for c in p.hand)
    if not (has_rean_access(g, p) or tut): return False
    if not any(c.creature and c.bomb >= 5 for c in p.library): return False
    for name in ('Entomb', 'Buried Alive', 'Unmarked Grave', 'Grisly Salvage'):
        for c in p.hand:
            if c.name == name and can_pay(g, p, c.generic, c.pips):
                pay(g, p, c.generic, c.pips); cast_card(g, p, c, 'hand', {})
                return True
    return False


def seph_tortured(g, p):
    if not any(m.cd is not None and m.cd.tags.get('fill') == 'tortured' for m in p.perms): return False
    if blocked(g, p, 'Tortured Existence'): return False
    if own_bomb_in_gy(g, p) or getattr(p, 'te_used', -1) == p.turns: return False
    bombs = [c for c in p.hand if c.creature and c.bomb >= 6]
    if not bombs or not can_pay(g, p, 0, 'B') or not has_rean_access(g, p): return False
    pay(g, p, 0, 'B'); b = max(bombs, key=lambda c: seph_bval(g, p, c))
    p.hand.remove(b); p.gy.append(b); p.te_used = p.turns
    small = [c for c in p.gy if c.creature and not c.bomb]
    if small: p.gy.remove(small[0]); p.hand.append(small[0])
    return True


def seph_tutor_target(g, p):
    lib = {c.name for c in p.library}
    rean = has_rean_access(g, p)
    tgt = own_bomb_in_gy(g, p)

    def first(names):
        for n in names:
            if n in lib: return n
        return None
    if not bomb_on_bf(p):
        if rean and not tgt: return first(['Entomb', 'Buried Alive', 'Unmarked Grave'])
        if tgt and not rean:
            order = (['Reanimate'] if p.life > 22 else []) + ['Animate Dead', 'Necromancy', 'Dread Return',
                                                               'Persist', 'Unburial Rites', 'Evil Reawakened']
            return first(order)
        if not rean and not tgt: return first(['Entomb', 'Buried Alive', 'Animate Dead'])
    return first(['Heroic Intervention', 'Counterspell', 'Swan Song', 'Swiftfoot Boots', "Dovin's Veto",
                  'Phyrexian Arena', 'Animate Dead'])


def seph_tutor(g, p):
    tuts = [c for c in p.hand if c.tags.get('tut') == 'any']
    if not tuts: return False
    if bomb_on_bf(p) and p.turns < 6: return False
    if seph_tutor_target(g, p) is None: return False
    for c in sorted(tuts, key=lambda c: c.cmc):
        if not can_pay(g, p, c.generic, c.pips): continue
        if 'needsac' in c.tags:
            fod = [m for m in p.perms if m.creature and (m.token or (not m.cd.bomb and m.cd is not p.cmd))]
            if not fod: continue
            pay(g, p, c.generic, c.pips); die(g, min(fod, key=lambda x: pval(g, x)), 'sac')
        else:
            pay(g, p, c.generic, c.pips)
        cast_card(g, p, c, 'hand', {})
        return True
    return False



def seph_avarice(g, p):
    """Insatiable Avarice (spree): +{2} tutor to top, +{B}{B} target player draws 3 and loses 3.
    Both modes (5 mana) = tutor then draw it with two extra cards."""
    av = [c for c in p.hand if 'avarice' in c.tags]
    if not av: return False
    c = av[0]
    want = seph_tutor_target(g, p)
    if bomb_on_bf(p) and total_mana(g, p) < 7: want = None
    if want and p.life > 12 and can_pay(g, p, 2, 'BBB'): mode = 'both'
    elif want and can_pay(g, p, 2, 'B'): mode = 'tutor'
    elif not want and p.life > 15 and p.turns >= 4 and can_pay(g, p, 0, 'BBB'): mode = 'draw'
    else: return False
    pay(g, p, *{'both': (2, 'BBB'), 'tutor': (2, 'B'), 'draw': (0, 'BBB')}[mode])
    p.hand.remove(c); p.spells_this_turn += 1
    on_cast(g, p, c)
    if g.over or not p.alive: return True
    if mode in ('both', 'tutor'):
        hit = next((x for x in p.library if x.name == want), None)
        if hit:
            p.library.remove(hit); g.rng.shuffle(p.library); p.library.append(hit); p.stats['tutored'] += 1
    if mode in ('both', 'draw'):
        draw(g, p, 3); lose_life(g, p, 3, p)
    p.gy.append(c); p.stats['avarice_' + mode] += 1
    check_state(g)
    return True


def seph_hardcast(g, p):
    if p.cmd_in_zone:
        cg, cp = cost_of(p, p.cmd)
        if can_pay(g, p, cg, cp):
            pay(g, p, cg, cp)
            if cast_card(g, p, p.cmd, 'cmd', {}): note_bomb(p, p.cmd)
            return True
    bombs = sorted([c for c in p.hand if c.creature and c.bomb >= 4], key=lambda c: -seph_bval(g, p, c))
    for c in bombs:
        if can_pay(g, p, c.generic, c.pips):
            pay(g, p, c.generic, c.pips)
            if cast_card(g, p, c, 'hand', {}): note_bomb(p, c)
            return True
    return False


def seph_yawg(g, p):
    ys = [c for c in p.hand if 'yawg' in c.tags]
    if not ys: return False
    reans = [c for c in p.gy if 'rean' in c.tags]
    if not reans or not own_bomb_in_gy(g, p): return False
    if any('rean' in c.tags for c in p.hand): return False
    ch = min(reans, key=lambda c: c.cmc)
    if not can_pay(g, p, 2 + ch.generic, 'B' + ch.pips): return False
    pay(g, p, 2, 'B'); cast_card(g, p, ys[0], 'hand', {})
    return True


def seph_trophy_grounds(g, p):
    if not (own_bomb_in_gy(g, p) or has_rean_access(g, p)): return False
    for q in g.opps(p):
        gl = [L for L in q.lands if L.cd.tags.get('desert')]
        if not gl: continue
        for c in p.hand:
            if c.tags.get('tgt') == 'p' and c.tags.get('rem') == 'destroy' and can_pay(g, p, c.generic, c.pips):
                pay_card(g, p, c)
                q.lands.remove(gl[0]); q.gy.append(gl[0].cd); land_ramp(g, q, 1, True)
                p.stats['trophy_grounds'] += 1
                return True
    return False


def seph_clamp(g, p):
    if not has(p, 'clamp') or blocked(g, p, 'Skullclamp'): return False
    if getattr(p, 'clamp_t', -1) != p.turns: p.clamp_t = p.turns; p.clamp_n = 0
    if p.clamp_n >= 2: return False
    fod = [m for m in p.perms if m.creature and not m.phased and etgh(g, m) == 1 and
           (m.token or m.cd.tags.get('fill') in ('stitcher', 'wayfinder') or (p.turns >= 7 and 'dork' in m.cd.tags))]
    if not fod or not can_pay(g, p, 1, ''): return False
    pay(g, p, 1, ''); p.clamp_n += 1
    die(g, fod[0], 'sac'); draw(g, p, 2)
    return True


def seph_dispute(g, p):
    ds = [c for c in p.hand if c.tags.get('fill') == 'dispute']
    if not ds: return False
    fod = [m for m in p.perms if m.creature and not m.phased and (m.token or (not m.cd.bomb and m.cd.tags.get('fill') == 'stitcher'))]
    if not fod and not p.treasures: return False
    c = ds[0]
    if not can_pay(g, p, c.generic, c.pips): return False
    pay(g, p, c.generic, c.pips)
    if fod: die(g, fod[0], 'sac')
    elif p.treasures: p.treasures -= 1
    cast_card(g, p, c, 'hand', {})
    return True


def seph_boots(g, p):
    for e in find(p, 'prot'):
        if e.cd.tags.get('prot') != 'boots' or e.attached is not None: continue
        if blocked(g, p, e.cd.name): continue
        bombs = [m for m in p.perms if m.creature and m.cd is not None and m.cd.bomb >= 6 and not untargetable(g, m)]
        if bombs and can_pay(g, p, 1, ''):
            pay(g, p, 1, ''); e.attached = max(bombs, key=lambda x: pval(g, x)); return True
    return False


def seph_prio(g, p, c):
    t = c.tags
    if 'shards' in t: return 68
    if c is p.cmd: return 0
    if 'rock' in t or 'dork' in t or 'lr' in t: return 80 if p.turns <= 5 else 30
    if 'tithe' in t: return 72
    if t.get('fill') == 'stitcher': return 75
    if t.get('fill') == 'tortured': return 65
    if t.get('fill') == 'wayfinder': return 45
    if 'eng' in t: return 60
    if 'rite' in t: return 45
    if 'clamp' in t: return 50
    if t.get('prot') == 'boots': return 70 if bomb_on_bf(p) else 25
    if 'sac' in t: return 50
    if 'bartist' in t or 'drain' in t: return 45
    if 'clone' in t: return 45 if any(x.creature and x.cd is not None and x.cd.bomb for q in g.players for x in q.perms) else 0
    if 'witness' in t: return 50 if any(any(k in x.tags for k in E.KEYSPELL) for x in p.gy) else 20
    if 'wall' in t: return 45 if any((x.instant or x.sorcery) and any(k in x.tags for k in E.KEYSPELL) for x in p.gy) else 8
    if 'draw' in t and c.sorcery and 'fill' not in t: return 35
    if 'tide' in t: return 5 if p.turns >= 12 else 0
    if c.creature and not c.bomb: return 30
    return 0


def interaction_reserve(g, p, pred):
    opts = [c for c in p.hand if pred(c) and 'free' not in c.tags]
    if not opts: return (0, '')
    c = min(opts, key=lambda c: c.cmc)
    return (c.generic, c.pips)


def seph_reserve(g, p):
    if not bomb_on_bf(p) and p.turns < 5: return (0, '')
    return interaction_reserve(g, p, lambda c: 'ctr' in c.tags or c.tags.get('prot') == 'hi' or 'tide' in c.tags)


def seph_main(g, p, post):
    for _ in range(14):
        if g.over or not p.alive: return
        if seph_trophy_grounds(g, p): continue
        if seph_reanimate(g, p): continue
        if seph_yawg(g, p): continue
        if seph_tortured(g, p): continue
        if seph_fill(g, p): continue
        if seph_tutor(g, p): continue
        if seph_avarice(g, p): continue
        if seph_hardcast(g, p): continue
        if consider_wipe(g, p): continue
        if use_removal(g, p, 6): continue
        if seph_dispute(g, p): continue
        if seph_clamp(g, p): continue
        if seph_boots(g, p): continue
        if generic_cast(g, p, seph_prio, seph_reserve(g, p)): continue
        break


def seph_dredge(g, p):
    imps = [c for c in p.gy if 'dredge' in c.tags]
    if not imps or len(p.library) < 6: return False
    if own_bomb_in_gy(g, p) or bomb_on_bf(p) or not has_rean_access(g, p): return False
    p.gy.remove(imps[0]); p.hand.append(imps[0])
    E.mill(g, p, 5); p.stats['dredged'] += 1
    return True


def regrow(g, p, only_is):
    order = ['Reanimate', 'Animate Dead', 'Demonic Tutor', 'Necromancy', 'Entomb', 'Buried Alive', 'Dread Return',
             'Diabolic Tutor', 'Unburial Rites', 'Persist', 'Counterspell', 'Swan Song', "Dovin's Veto",
             'Heroic Intervention', 'Toxic Deluge', 'Swords to Plowshares', "Assassin's Trophy"]
    for n in order:
        for c in p.gy:
            if c.name == n and (not only_is or c.instant or c.sorcery):
                p.gy.remove(c); p.hand.append(c)
                if any(k in c.tags for k in E.KEYSPELL): p.stats['key_regrown'] += 1
                return


# ======================================================== generic AI pieces
def generic_cast(g, p, prio, reserve=(0, '')):
    cands = []
    for c in p.hand:
        if c.land: continue
        pr = prio(g, p, c)
        if pr <= 0 and c.dsl and E.DSLMOD is not None: pr = E.DSLMOD.card_value(g, p, c) * 10 - 5
        if pr > 0: cands.append((pr, g.rng.random(), c))
    if p.cmd_in_zone:
        pr = prio(g, p, p.cmd)
        if pr > 0: cands.append((pr, 0.5, p.cmd))
    cands.sort(key=lambda x: (-x[0], x[1]))
    for pr, _, c in cands:
        cg, cp = cost_of(p, c)
        rg, rp = reserve if pr < 90 else (0, '')
        if can_pay(g, p, cg + rg, cp + rp):
            pay(g, p, cg, cp)
            zone = 'cmd' if (c is p.cmd and c not in p.hand) else 'hand'
            cast_card(g, p, c, zone, {})
            return True
    return False


def use_removal(g, p, threshold=5, instants_extra=None):
    if instants_extra is None: instants_extra = E.INSTANT_EXTRA
    for c in list(p.hand):
        t = c.tags
        if 'rem' not in t or c.creature: continue
        if 'needart3' in t and sum(1 for m in p.perms if m.cd is not None and 'A' in m.cd.types) < 3: continue
        tg = legal_targets(g, p, t['rem'], t.get('tgt', 'c'), 'mv4' in t, spell=c)
        if not tg: continue
        best = max(tg, key=lambda x: pval(g, x))
        th = threshold + (instants_extra if c.instant else 0)
        if pval(g, best) < th: continue
        fod = None
        extra = 0
        if 'needsac' in t or 'sacor4' in t:
            fods = [m for m in p.perms if m.creature and (m.token or not m.cd.bomb)]
            if fods: fod = min(fods, key=lambda x: pval(g, x))
            elif 'sacor4' in t: extra = 4                       # Lash of the Balrog: or pay {4}
            else: continue
        if not can_pay(g, p, c.generic + extra, c.pips): continue
        pay(g, p, c.generic + extra, c.pips)
        if fod: die(g, fod, 'sac')
        cast_card(g, p, c, 'hand', {'target': best})
        p.stats['removal_cast'] += 1
        return True
    return False


def wipe_eval(g, p, kind):
    if kind in ('farewell', 'austere2'):
        modes = wipe_modes(g, p, kind)
        ol = ml = 0.0
        for q in g.players:
            if not q.alive: continue
            for m in q.perms:
                if m.phased: continue
                ty = m.cd.types if m.cd is not None else 'C'
                hit = (('art' in modes and 'A' in ty) or ('ench' in modes and 'E' in ty) or ('cre' in modes and m.creature)
                       or ('le3' in modes and m.creature and (m.cd is None or m.cd.cmc <= 3))
                       or ('ge4' in modes and m.creature and m.cd is not None and m.cd.cmc >= 4))
                if hit:
                    if q is p: ml += pval(g, m)
                    else: ol += pval(g, m)
        return ol, ml, None
    if kind == 'vandal':
        return sum(pval(g, m) for q in g.opps(p) for m in q.perms if m.cd is not None and 'A' in m.cd.types), 0, None
    if kind in ('rift', 'rebuke'):
        vals = {q: sum(pval(g, m) for m in q.perms) for q in g.opps(p)}
        if not vals: return 0, 0, None
        if kind == 'rift': return sum(vals.values()), 0, None
        v = max(vals, key=vals.get); return vals[v], 0, v
    biggest = None; X = 0
    if kind == 'nib':
        cr = [m for m in p.perms if m.creature]
        if not cr: return 0, 0, None
        biggest = max(cr, key=lambda x: epow(g, x)); X = epow(g, biggest)
    rean = p.key == 'seph' and has_rean_access(g, p)
    ol = ml = 0
    for q in g.players:
        if not q.alive: continue
        for m in q.perms:
            if not m.creature or m.phased or m is biggest: continue
            hit = {'destroy': True, 'minus': True, 'exile': True, 'evac': True, 'dmg13': etgh(g, m) <= 13,
                   'austere2': m.cd is not None and m.cd.cmc >= 4,
                   'austere': m.cd is not None and m.cd.cmc >= 4, 'nib': etgh(g, m) <= X}[kind]
            if not hit: continue
            v = pval(g, m)
            if kind == 'evac' and not m.token: v *= 0.5          # bounced cards can be recast
            if q is p:
                if rean and m.cd is not None and m.cd.bomb and kind != 'exile': v *= 0.4
                ml += v
            else: ol += v
    return ol, ml, None


def wipe_cost(p, c):
    kind = c.tags.get('wipe')
    if kind == 'rift': return 6, 'U'
    if kind == 'vandal': return 4, 'R'
    return cost_of(p, c)


def wipe_modes(g, p, kind):
    """pick modes for Farewell (any number) or Austere Command (exactly two) by net value"""
    def val(pred):
        v = 0.0
        for q in g.players:
            if not q.alive: continue
            s = -1.2 if q is p else 1.0
            for m in q.perms:
                if not m.phased and pred(m): v += s * pval(g, m)
        return v
    ty = lambda m: m.cd.types if m.cd is not None else 'C'
    opts = {'art': val(lambda m: 'A' in ty(m)), 'ench': val(lambda m: 'E' in ty(m))}
    if kind == 'farewell':
        opts['cre'] = val(lambda m: m.creature)
        ch = {k for k, v in opts.items() if v > 0}
        return ch or {'cre'}
    opts['le3'] = val(lambda m: m.creature and (m.cd is None or m.cd.cmc <= 3))
    opts['ge4'] = val(lambda m: m.creature and m.cd is not None and m.cd.cmc >= 4)
    return set(sorted(opts, key=lambda k: -opts[k])[:2])


def consider_wipe(g, p):
    for c in list(p.hand):
        kind = c.tags.get('wipe')
        if not kind: continue
        cg, cp = wipe_cost(p, c)
        if not can_pay(g, p, cg, cp): continue
        ol, ml, victim = wipe_eval(g, p, kind)
        if ol - 1.2 * ml >= 11 and ol >= 11:
            pay(g, p, cg, cp)
            cast_card(g, p, c, 'hand', {'victim': victim})
            p.stats['wipes_cast'] += 1
            return True
    return False


# ======================================================== veyran
def veyran_prio(g, p, c):
    t = c.tags
    if c is p.cmd: return 70
    if 'rock' in t: return 80 if p.turns <= 5 else 40
    if 'vkitten' in t or 'vfire' in t: return 74
    if 'recruit' in t: return 73
    if 'kiln' in t: return 70
    if 'birgi' in t: return 69
    if t.get('prot') == 'boots': return 50
    if 'ping' in t: return 72
    if 'spelltok' in t: return 71
    if 'aether' in t: return 66
    if 'dragoncaller' in t: return 60
    if 'spelldraw' in t or 'mystic' in t: return 58
    if 'thor' in t: return 50
    if t.get('tut') == 'art': return 55
    if t.get('tut') == 'is': return 30
    if 'draw' in t and (c.instant or c.sorcery): return 48
    if 'draw' in t: return 42
    if 'spider' in t: return 46
    if 'flute' in t: return 45 if flute_pick(g, p)[1] >= 5 else 0
    if 'fbgrant' in t: return 38 if any((x.instant or x.sorcery) and 'draw' in x.tags for x in p.gy) else 0
    if c.creature: return 40
    if 'burn' in t and any(q.life <= 15 for q in g.opps(p)): return 35
    return 0


def payoff(p):
    """payoffs that turn the infinite loop into a win"""
    return has(p, 'ping') or has(p, 'aether') or has(p, 'dragoncaller')


def engine_payoff(p):
    """anything that gets value from each spell (for the 'engine online' milestone)"""
    return payoff(p) or has(p, 'mystic') or has(p, 'spelltok') or has(p, 'spelldraw') or has(p, 'kiln')


def combo_interrupted(g, p, which, key_perms):
    uncounterable = False
    if which == 'veyran':                       # Mistrise Village: {U},{T}: next spell can't be countered
        ml = [L for L in p.lands if L.cd.tags.get('mistrise') and not L.tapped]
        if ml:
            ml[0].tapped = True
            if can_pay(g, p, 0, 'U'):
                pay(g, p, 0, 'U'); uncounterable = True
                log("    Mistrise Village: the loop spell can't be countered", g)
            else:
                ml[0].tapped = False
    if tide_response(g, p, 'combo_' + which, 10):
        if which == 'veyran':
            for m in find(p, 'vkitten'): m.neutered = True
        return True
    for q in g.after(p):
        if not q.alive or q is p or g.over or silenced(g, q): continue
        if g.rng.random() > 0.95: continue
        for c in list(q.hand):
            t = c.tags
            if 'rem' not in t or not c.instant: continue
            tg = [m for m in legal_targets(g, q, t['rem'], t.get('tgt', 'c'), 'mv4' in t, spell=c) if m in key_perms]
            if not tg or not can_pay(g, q, c.generic, c.pips): continue
            pay(g, q, c.generic, c.pips)
            cast_card(g, q, c, 'hand', {'target': tg[0]})
            if tg[0] not in p.perms or tg[0].phased:
                q.stats['combo_stops_removal'] += 1
                return True
            break
        if which == 'veyran' and not uncounterable:
            for ctr in [c for c in q.hand if c.tags.get('ctr') in ('any', 'nc', 'ise')]:
                if 'free' not in ctr.tags and not can_pay(g, q, ctr.generic, ctr.pips): continue
                if not cast_counter(g, q, ctr): continue
                back = pick_counter(g, p, ctr)
                if back is not None and cast_counter(g, p, back): break
                q.stats['combo_stops_counter'] += 1
                return True
    return False


def win(g, p, how):
    log(f'*** {NAME(p)} wins by {how}', g)
    for q in g.opps(p):
        q.last_src = p; q.last_kind = how; eliminate(g, q)
    g.over = True; g.winner = p; g.wintype = how


def veyran_try_combo(g, p):
    if p.combo_tried: return False
    k = [m for m in find(p, 'vkitten') if not m.neutered]
    f = find(p, 'vfire')
    if not k or not f or not payoff(p) or not can_pay(g, p, 2, 'R'): return False
    p.combo_tried = True
    pay(g, p, 2, 'R')
    p.milestone.setdefault('combo', p.turns)
    if g.goldfish: return True
    p.stats['combo_attempt'] += 1
    log('  Veyran starts the Kitten + Firesinger loop', g)
    if combo_interrupted(g, p, 'veyran', k + f):
        p.stats['combo_stopped'] += 1; log('    ...the loop is stopped', g); return False
    win(g, p, 'combo'); return True


def veyran_fair(g, p):
    """Inventors' Fair: {4},{T}, sacrifice: search for an artifact (needs 3+ artifacts). Goes for Aetherflux."""
    fairs = [L for L in p.lands if L.cd.tags.get('fair') and not L.tapped and not blocked(g, p, L.cd.name)]
    if not fairs or has(p, 'aether') or any('aether' in c.tags for c in p.hand): return False
    if not any('aether' in c.tags for c in p.library): return False
    if sum(1 for m in p.perms if m.cd is not None and 'A' in m.cd.types) < 3: return False
    L = fairs[0]; L.tapped = True
    if not can_pay(g, p, 4, ''):
        L.tapped = False; return False
    pay(g, p, 4, '')
    p.lands.remove(L); p.gy.append(L.cd)
    log("  Veyran sacrifices Inventors' Fair", g)
    tutor(g, p, 'art')
    return True


def veyran_boots(g, p):
    """Equip Swiftfoot Boots / Lightning Greaves: Kitten during a combo setup, else Veyran, else the best engine."""
    eq = [e for e in find(p, 'prot') if e.cd.tags.get('prot') == 'boots' and (e.attached is None or e.attached not in p.perms)
          and not blocked(g, p, e.cd.name)]
    if not eq or not can_pay(g, p, 1, ''): return False
    cands = [m for m in p.perms if m.creature and m.cd is not None and not untargetable(g, m)]
    if not cands: return False

    def rank(m):
        t = m.cd.tags
        if 'vkitten' in t and has(p, 'vfire'): return 10
        if 'veyran' in t: return 9
        if 'vkitten' in t or 'vfire' in t: return 7
        return pval(g, m)
    pay(g, p, 1, ''); eq[0].attached = max(cands, key=rank)
    return True


def aether_check(g, p):
    if not has(p, 'aether') or p.life < 51 or blocked(g, p, 'Aetherflux Reservoir'): return False
    opps = g.opps(p)
    if not opps: return False
    lose_life(g, p, 50, p)
    if tide_response(g, p, 'aether', 9): return True
    tgt = max(opps, key=lambda o: threat(g, p, o))
    p.stats['aether_shots'] += 1; p.milestone.setdefault('aether', p.turns)
    log(f'  Veyran fires Aetherflux Reservoir at {NAME(tgt)}', g)
    lose_life(g, tgt, 50, p, kind='aether'); check_state(g)
    return True


def veyran_main(g, p, post):
    for _ in range(16):
        if g.over or not p.alive: return
        if veyran_try_combo(g, p) and (g.over or g.goldfish): return
        if aether_check(g, p): continue
        if veyran_fair(g, p): continue
        if veyran_boots(g, p): continue
        if use_removal(g, p, 6): continue
        if consider_wipe(g, p): continue
        res = interaction_reserve(g, p, lambda c: 'ctr' in c.tags) if p.turns >= 3 else (0, '')
        if generic_cast(g, p, veyran_prio, res): continue
        break


# ======================================================== sauron
def sauron_prio(g, p, c):
    t = c.tags
    if c is p.cmd: return 85
    if 'rock' in t: return 80 if p.turns <= 5 else 40
    if 'rhystic' in t: return 78
    if 'mauhur' in t: return 63
    if 'bowmasters' in t: return 62
    if 'sword' in t: return 60
    if 'assault' in t: return 58 if (has(p, 'sword') or any('sword' in x.tags for x in p.hand)) else 30
    if 'cloak' in t: return 57
    if 'prolif' in t: return 55
    if 'skate' in t:
        a = army_of(p); return 56 if a and a.plus >= 6 else 0
    if 'kaervek' in t: return 54
    if 'witchking' in t: return 52
    if 'eng' in t: return 50
    if 'archivist' in t: return 50 if has(p, 'bowmasters') else 20
    if 'warmachine' in t: return 45
    if t.get('tut'): return 60
    if 'draw' in t and (c.instant or c.sorcery): return 40
    if 'flail' in t: return 56
    if 'ozolith' in t: return 45
    if 'animist' in t: return 44
    if 'unearth' in t: return 35 if any(x.creature and x.cmc <= 3 for x in p.gy) else 0
    if 'pwdiscard' in t: return 38
    if c.creature: return 42
    if c.perm and 'prot' not in t: return 10
    return 0


def sauron_equip(g, p):
    a = army_of(p)
    if not a or a.phased: return False
    for tag in ('sword', 'flail', 'animist', 'cloak'):
        if equipped(a, tag): continue
        eq = [e for e in find(p, tag) if e.attached is not a and not blocked(g, p, e.cd.name)]
        if not eq: continue
        if tag == 'cloak' and any(find(p, x) and not equipped(a, x) for x in ('sword', 'flail', 'animist')): continue
        if tag == 'cloak' and any(e.cd is not None and e.cd.dsl and (e.attached is None or e.attached not in p.perms)
                                  and any(x.get('static') == 'equip_cost' for x in e.cd.dsl) and not stopped(g, e.cd.name)
                                  for e in p.perms):
            continue                                   # e.g. Sword of Hearth and Home goes on first
        if equipped(a, 'cloak'): return False
        if can_pay(g, p, 2, ''):
            pay(g, p, 2, ''); eq[0].attached = a; return True
    return False


def sauron_archivist(g, p):
    if getattr(p, 'arch_t', -1) == p.turns: return False
    ar = [m for m in find(p, 'archivist') if not m.sick and not m.tapped and not blocked(g, p, m.cd.name)]
    if not ar or not has(p, 'bowmasters') or not can_pay(g, p, 0, 'U'): return False
    pay(g, p, 0, 'U'); ar[0].tapped = True; p.arch_t = p.turns
    n = max(len(q.hand) for q in g.players if q.alive)
    for q in g.players:
        if q.alive: q.gy.extend(q.hand); q.hand = []
    for q in g.players:
        if q.alive: draw(g, q, n)
    check_state(g)
    return True


def sauron_main(g, p, post):
    for _ in range(16):
        if g.over or not p.alive: return
        if sauron_equip(g, p): continue
        if sauron_archivist(g, p): continue
        if use_removal(g, p, 6): continue
        if consider_wipe(g, p): continue
        res = interaction_reserve(g, p, lambda c: 'ctr' in c.tags) if p.turns >= 4 else (0, '')
        if generic_cast(g, p, sauron_prio, res): continue
        break


def sauron_combo_ready(g, p):
    a = army_of(p)
    if not a or a.phased or not has(p, 'assault') or not equipped(a, 'sword'): return False
    evasive = equipped(a, 'cloak') or any(L.cd.tags.get('passage') for L in p.lands)
    return evasive and len(p.lands) >= 5


# ======================================================== najeela
def najeela_prio(g, p, c):
    t = c.tags
    if c is p.cmd: return 83 if p.turns >= 2 else 0
    if 'rock' in t or 'dork' in t or 'lr' in t: return 85 if p.turns <= 5 else 40
    if 'crusade' in t: return 78
    if 'tokup' in t or 'tokatk' in t or 'tok' in t or 'tokbig' in t or 'rabble' in t or 'landfall2' in t: return 70
    if 'jinnie' in t: return 67
    if 'uprising' in t or 'mycoloth' in t: return 62
    if 'wisp' in t or 'bolas' in t or 'lidless' in t: return 52
    if 'stampede' in t: return 55 if len([m for m in p.perms if m.creature]) >= 6 else 0
    if 'drawcre' in t: return 55 if len([m for m in p.perms if m.creature]) >= 4 else 20
    if 'warleader' in t: return 66
    if 'remora' in t: return 65 if p.turns <= 3 else 0
    if 'mirror' in t: return 60
    if t.get('tut'): return 62
    if 'eng' in t: return 50
    if 'overrun' in t: return 55 if len([m for m in p.perms if m.creature]) >= 6 else 0
    if 'draw' in t and (c.instant or c.sorcery): return 45
    if c.creature: return 48
    return 0


def najeela_main(g, p, post):
    for _ in range(16):
        if g.over or not p.alive: return
        if use_removal(g, p, 6): continue
        if consider_wipe(g, p): continue
        crs = [m for m in p.perms if m.creature]
        if not post and has(p, 'najeela') and len(crs) >= 3:
            res = (0, 'WUBRG')
        elif p.turns >= 5 and sum(pval(g, m) for m in crs) >= 10:
            res = interaction_reserve(g, p, lambda c: c.tags.get('prot') in ('indes', 'phase'))
        else:
            res = (0, '')
        if post:
            xs = [c for c in p.hand if 'tokx' in c.tags]
            if xs and total_mana(g, p) >= len(xs[0].pips) + 3:
                c = xs[0]; pay(g, p, c.generic, c.pips); x = total_mana(g, p); pay(g, p, x, '')
                cast_card(g, p, c, 'hand', {'x': x}); continue
        if generic_cast(g, p, najeela_prio, res): continue
        break


def najeela_ready(g, p):
    return (any(m.cd is p.cmd and not m.sick for m in p.perms) and
            len([m for m in p.perms if m.creature and not m.sick and not m.noatk]) >= 3)


# ======================================================== tutor choice
def _tutor_pick_named(g, p, kind):
    def ok(c):
        if kind == 'any': return True
        if kind == 'is': return c.instant or c.sorcery
        if kind == 'art': return 'A' in c.types
        if kind == 'perm': return c.perm
        if kind == 'ubr': return not c.land and any(x in c.pips for x in 'UBR')
        if kind == 'cre2': return c.creature and c.pow <= 2
        if kind == 'cre': return c.creature
        if kind == 'ench': return 'E' in c.types
        return True
    okn = {c.name for c in p.library if ok(c)}

    def first(ns):
        for n in ns:
            if n in okn: return n
        return None
    if p.key == 'seph':
        n = seph_tutor_target(g, p)
        return n if n in okn else None
    if p.key == 'veyran':
        if kind == 'art': return first(['Aetherflux Reservoir', 'Sol Ring', 'Fellwar Stone'])
        if kind == 'cre2':
            kitten = has(p, 'vkitten') or any('vkitten' in c.tags for c in p.hand)
            fire = has(p, 'vfire') or any('vfire' in c.tags for c in p.hand)
            if kitten and not fire: return first(['Blazing Firesinger // Seething Song'])
            if fire and not kitten: return first(['Displacer Kitten'])
            return first(['Displacer Kitten', 'Blazing Firesinger // Seething Song', 'Guttersnipe', 'Archmage Emeritus'])
        if not any('ctr' in c.tags for c in p.hand):
            return first(['Counterspell', 'Force of Will', 'Mystic Confluence', 'Stock Up'])
        return first(['Stock Up', 'Flow State', 'Quick Study', 'Crackle with Power'])
    if p.key == 'sauron':
        sw = has(p, 'sword') or any('sword' in c.tags for c in p.hand)
        asl = has(p, 'assault') or any('assault' in c.tags for c in p.hand)
        order = []
        if sw and not asl: order.append('Aggravated Assault')
        if asl and not sw: order.append('Sword of Feast and Famine')
        if sw and asl: order.append('Whispersilk Cloak')
        order += ['Rhystic Study', 'Sword of Feast and Famine', 'Aggravated Assault', 'Deepglow Skate', 'Counterspell']
        return first(order)
    if p.key == 'najeela':
        order = [] if has(p, 'crusade') else ["Cathars' Crusade"]
        order += ['Chromatic Lantern', 'Mirror Entity', "Warleader's Call"]
        return first(order)
    return None



def tutor_pick(g, p, kind):
    """deck-specific wish lists first; otherwise the highest-priority legal card (works for any card pool)"""
    n = _tutor_pick_named(g, p, kind)
    if n: return n
    ok = {'any': lambda c: True, 'is': lambda c: c.instant or c.sorcery, 'art': lambda c: 'A' in c.types,
          'perm': lambda c: c.perm, 'ubr': lambda c: not c.land and any(x in c.pips for x in 'UBR'),
          'cre2': lambda c: c.creature and c.pow <= 2, 'cre': lambda c: c.creature,
          'ench': lambda c: 'E' in c.types}.get(kind, lambda c: True)
    prio = {'seph': seph_prio, 'veyran': veyran_prio, 'sauron': sauron_prio, 'najeela': najeela_prio}[p.key]
    cands = [c for c in p.library if ok(c) and not c.land]
    if not cands: return None
    best = max(cands, key=lambda c: (prio(g, p, c), c.bomb, c.cmc))
    return best.name


# ======================================================== combat
def choose_defender(g, p):
    opps = g.opps(p)
    my = sum(epow(g, m) for m in p.perms if m.creature and not m.tapped and not m.noatk and not m.sick)
    lethal = [q for q in opps if q.life <= my * 0.6]
    if lethal: return min(lethal, key=lambda q: q.life)
    return max(opps, key=lambda q: threat(g, p, q) + g.rng.random() * 3)


def can_block(g, b, a):
    if b.cd is not None and 'noblock' in b.cd.tags: return False
    if E.DSLMOD is not None and g.dsl_on:
        D = E.DSLMOD
        if D.has_kw(g, b, 'cant_block') or D.has_kw(g, a, 'unblockable'): return False
        if D.has_kw(g, a, 'flying') and not (b.fly or D.has_kw(g, b, 'flying') or D.has_kw(g, b, 'reach')
                                             or (b.cd is not None and 'reach' in b.cd.tags)): return False
    if a.cd is not None and 'swampwalk' in a.cd.tags and any(
            L.cd.name in ('Swamp', 'Watery Grave', 'Blood Crypt', 'Overgrown Tomb') for L in b.owner.lands):
        return False                                 # Sheoldred, Whispering One: swampwalk
    if a.fly and not b.fly and not (b.cd is not None and 'reach' in b.cd.tags): return False
    if protected_from(g, a, colors_of(b)): return False          # protection: can't be blocked by that colour
    return True


def double_strike(p, m):
    return m.army and has(p, 'warmachine')


def attack_triggers(g, p, atk, d):
    new = []
    for m in list(atk):
        if m.cd is None: continue
        t = m.cd.tags
        if 'witchking' in t: edict(g, d)
        if 'archon' in t and d.alive: archon_attack(g, p, d)
        if 'titan' in t: make_tokens(g, p, 2, 2)
        if 'tokatk' in t:
            att = m.cd.name.startswith('Adeline')
            n = len(g.opps(p)) if att else int(t['tokatk'])
            new += make_tokens(g, p, n, 1, attacking=att, sick=not att, lifelink=not att)
        if 'suntitan' in t: sun_titan(g, p)
        if 'necromancer' in t:                    # attacking 3/3 Wraith copy of a graveyard creature
            if any(c.creature for c in p.gy):
                w = make_tokens(g, p, 1, 3, attacking=True, sick=False)
                for x in w: x.temp = True; x.name = 'Wraith'
                new += w
        if 'kylox' in t:                          # sac tokens, cast instants/sorceries among top X for free
            toks = [x for x in p.perms if x.token and x.creature and x not in atk]
            X = sum(epow(g, x) for x in toks)
            for x in toks: die(g, x, 'sac')
            top = [p.library.pop() for _ in range(min(X, len(p.library)))]
            for c in top:
                if c.instant or c.sorcery:
                    cast_copy(g, p, (lambda c=c: draw(g, p, int(c.tags['draw']))) if 'draw' in c.tags else None)
                p.exile.append(c)
        if equipped(m, 'animist'): land_ramp(g, p, 1, True)        # Sword of the Animist
    rab = len(find(p, 'rabble'))
    if rab: make_tokens(g, p, rab * len(atk), 1)                       # Rabble Rousing: one Citizen per attacker
    if has(p, 'najeela'):
        w = [m for m in atk if m.warrior and m in p.perms]
        if w: new += make_tokens(g, p, len(w), 1, warrior=True, attacking=True, sick=False)
    if E.DSLMOD is not None and g.dsl_on:
        E.DSLMOD.fire(g, 'attack', attackers=list(atk), defender=d, player=p, new=new)
    check_state(g)
    return [x for x in new if x.tapped and x in p.perms]


def archon_attack(g, p, d):
    edict(g, d)
    if d.hand: d.gy.append(d.hand.pop(g.rng.randrange(len(d.hand))))
    lose_life(g, d, 3, p, kind='drain'); gain(p, 3); draw(g, p, 1)


def resolve_combat(g, p, atk, d, unbl):
    tot_dmg = [0]
    try:
        return _resolve_combat(g, p, atk, d, unbl, tot_dmg)
    finally:
        log(f'  {NAME(p)} attacks {NAME(d)} with {len(atk)} creature(s): {tot_dmg[0]} damage (life now {d.life})', g)


def _resolve_combat(g, p, atk, d, unbl, tot_dmg):
    blockers = [m for m in d.perms if m.creature and not m.tapped and not m.phased]
    incoming = sum(epow(g, m) * (2 if double_strike(p, m) else 1) for m in atk)
    assign = {}; used = set()
    for a in sorted(atk, key=lambda m: -epow(g, m)):
        if a in unbl or a not in p.perms: continue
        cands = [b for b in blockers if b not in used and can_block(g, b, a)]
        if not cands: continue
        ap, at = epow(g, a), etgh(g, a)
        good = [b for b in cands if (epow(g, b) >= at or b.dt) and not (ap >= etgh(g, b) or a.dt)]
        if good: b = min(good, key=lambda x: pval(g, x))
        else:
            trade = [b for b in cands if epow(g, b) >= at or b.dt]
            if trade and pval(g, a) >= min(pval(g, x) for x in trade):
                b = min(trade, key=lambda x: pval(g, x))
            elif (E.AI_MODE == 'adaptive' and ap >= 2 and g.rng.random() < __import__('brain').chump_prob(g, d, incoming)) \
                    or (E.AI_MODE != 'adaptive' and incoming >= d.life * 0.4 and ap >= 2):
                b = min(cands, key=lambda x: pval(g, x))
            else:
                continue
        assign[a] = b; used.add(b); incoming -= ap
    conn = set()
    for a in atk:
        if a not in p.perms or not d.alive: continue
        ap = epow(g, a) * (2 if double_strike(p, a) else 1)
        b = assign.get(a)
        if b is None or b not in d.perms:
            dmg = ap
        else:
            bt = etgh(g, b)
            a_dies = (epow(g, b) >= etgh(g, a) or b.dt) and not protected_from(g, a, colors_of(b))
            b_dies = (ap >= bt or a.dt) and not protected_from(g, b, colors_of(a))
            tr = p.trample or has(p, 'uprising') or (a.cd is not None and 'trample' in a.cd.tags)
            dmg = max(0, ap - bt) if tr else 0
            if b_dies: die(g, b, 'destroy')
            if a_dies: die(g, a, 'destroy')
        if dmg > 0:
            lose_life(g, d, dmg, p, kind='combat'); conn.add(a); tot_dmg[0] += dmg
            if E.DSLMOD is not None and g.dsl_on: E.DSLMOD.fire(g, 'combat_damage', attacker=a, defender=d)
            if a.cd is not None and 'hellkite' in a.cd.tags:          # Hellkite Tyrant steals their artifacts
                for x in [x for x in d.perms if x.cd is not None and 'A' in x.cd.types and not x.creature]:
                    d.perms.remove(x); x.owner = p; x.attached = None; p.perms.append(x)
            if a.life or p.najeela_boost: gain(p, dmg)
            if a.is_cmd: d.cmd_dmg[p.key] += dmg
            if a.army and len(p.hand) <= 3:
                p.gy.extend(p.hand); p.hand = []; draw(g, p, 4)
            if equipped(a, 'sword'):
                if d.hand: d.gy.append(d.hand.pop(g.rng.randrange(len(d.hand))))
                for L in p.lands: L.tapped = False
    if conn and has(p, 'facebreaker'): p.treasures += len(find(p, 'facebreaker'))   # Professional Face-Breaker
    check_state(g)
    return conn


def ozolith_move(g, p):
    """The Ozolith: at the beginning of combat on your turn, move its counters onto a creature (the Army first)"""
    k = getattr(p, 'ozolith_counters', 0)
    if not k or not has(p, 'ozolith'): return
    cr = [m for m in p.perms if m.creature and not m.phased]
    if not cr: return
    tgt = army_of(p) or max(cr, key=lambda m: (not m.noatk, epow(g, m)))
    tgt.plus += k; p.ozolith_counters = 0
    p.stats['ozolith_moves'] += 1
    log(f'  The Ozolith moves {k} counters onto {tgt.name}', g)


def combat(g, p):
    ozolith_move(g, p)
    if g.goldfish:
        goldfish_combat(g, p); return
    p.najeela_boost = False; p.haste_all = False
    ncomb = 0
    while ncomb < 4 and not g.over and p.alive:
        ncomb += 1
        if not g.opps(p): return
        adaptive = E.AI_MODE == 'adaptive'
        if adaptive: import brain
        atk = [m for m in p.perms if m.creature and not m.tapped and not m.phased and (not m.sick or p.haste_all)
               and not m.noatk and epow(g, m) > 0]
        if not atk: break
        d = brain.choose_defender(g, p) if adaptive else choose_defender(g, p)
        if adaptive and ncomb == 1: atk = brain.filter_attackers(g, p, atk)
        if not atk: break
        unbl = set()
        a = army_of(p)
        for m in atk:
            if m.army and equipped(m, 'cloak'): unbl.add(m)
        if a in atk and a not in unbl and epow(g, a) >= 5:
            ps = [L for L in p.lands if L.cd.tags.get('passage') and not L.tapped and not blocked(g, p, L.cd.name)]
            if ps:
                ps[0].tapped = True
                if can_pay(g, p, 4, ''): pay(g, p, 4, ''); unbl.add(a)
                else: ps[0].tapped = False
        for m in atk:
            if not m.vig: m.tapped = True
        atk += attack_triggers(g, p, atk, d)
        if g.over or not d.alive: continue
        conn = resolve_combat(g, p, atk, d, unbl)
        if g.over or not p.alive: return
        if (p.key == 'najeela' and has(p, 'najeela') and ncomb <= 2 and can_pay(g, p, 0, 'WUBRG')
                and not blocked(g, p, 'Najeela, the Blade-Blossom')):
            pay(g, p, 0, 'WUBRG'); p.stats['najeela_act'] += 1
            p.milestone.setdefault('act', p.turns); log('  Najeela activates WUBRG for an extra combat', g)
            if tide_response(g, p, 'najeela', 8): break
            for m in p.perms:
                if m.creature: m.tapped = False
            p.haste_all = True; p.trample = True; p.najeela_boost = True
            continue
        if (p.key == 'sauron' and has(p, 'assault') and a is not None and a in p.perms
                and not blocked(g, p, 'Aggravated Assault')):
            if a in conn and equipped(a, 'sword') and a in unbl and can_pay(g, p, 3, 'RR'):
                pay(g, p, 3, 'RR'); p.stats['combo_attempt'] += 1
                p.milestone.setdefault('combo', p.turns)
                log('  Sauron goes for Sword + Aggravated Assault infinite combats', g)
                if combo_interrupted(g, p, 'sauron', [a]):
                    p.stats['combo_stopped'] += 1; log('    ...the combo is stopped', g); break
                win(g, p, 'combo'); return
            if ncomb == 1 and can_pay(g, p, 3, 'RR'):
                pay(g, p, 3, 'RR')
                if tide_response(g, p, 'assault', 7): break
                for m in p.perms:
                    if m.creature: m.tapped = False
                continue
        if p.extra_combats > 0 and ncomb < 4:          # extra combat phases granted by card abilities
            p.extra_combats -= 1
            for m in p.perms:
                if m.creature: m.tapped = False
            continue
        break
    p.najeela_boost = False; p.haste_all = False; p.trample = False


def goldfish_combat(g, p):
    if p.key == 'najeela' and 'act' not in p.milestone and najeela_ready(g, p) and can_pay(g, p, 0, 'WUBRG'):
        p.milestone['act'] = p.turns
    if p.key == 'sauron' and 'combo' not in p.milestone and sauron_combo_ready(g, p):
        saved = [L.tapped for L in p.lands]
        for L in p.lands: L.tapped = False
        if can_pay(g, p, 3, 'RR'): p.milestone['combo'] = p.turns
        for L, s in zip(p.lands, saved): L.tapped = s
    if p.key == 'najeela' and has(p, 'najeela'):
        w = [m for m in p.perms if m.warrior and not m.sick]
        make_tokens(g, p, len(w), 1, warrior=True, sick=False)


# ======================================================== turn structure
def land_enters_tapped(p, cd):
    t = cd.tags
    if 't' in t or 'f' in t: return True
    if 'ck' in t: return len(p.lands) < 2
    return False


def play_land(g, p):
    lands = [c for c in p.hand if c.land]
    if not lands: return
    have = set(''.join(land_cols(p, L, False) for L in p.lands))

    def score(c):
        cols = p.ident if c.tags.get('c') == 'A' else c.tags.get('c', '')
        s = len(set(cols) - have) * 2 + (0 if land_enters_tapped(p, c) else 3)
        return s + g.rng.random() * 0.1
    c = max(lands, key=score)
    p.hand.remove(c); p.lands.append(Land(c, land_enters_tapped(p, c)))
    log(f'  {NAME(p)} plays {c.name}', g)
    if 'bounceland' in c.tags:                  # Izzet Boilerworks returns another land
        others = [L for L in p.lands if L.cd is not c and 'bounceland' not in L.cd.tags]
        if others:
            L = min(others, key=lambda L: len(land_cols(p, L, False))); p.lands.remove(L); p.hand.append(L.cd)
    landfall(g, p)


def upkeep(g, p):
    if E.DSLMOD is not None and g.dsl_on: E.DSLMOD.fire(g, 'upkeep', player=p)
    for m in list(p.perms):
        if m.cd is None or m.phased or m not in p.perms: continue
        t = m.cd.tags
        if m.cd.name in ACTIVATED_ENGINES and blocked(g, p, m.cd.name): continue    # Disruptor Flute
        if 'eng' in t:
            if 'remora' in t and m.age > 4:
                leave(g, m); p.gy.append(m.cd); continue
            n = m.cd.name
            if n == 'Sylvan Library' and p.life <= 20: continue
            draw(g, p, 1)
            if n == 'Phyrexian Arena': lose_life(g, p, 1, p)
            if n == 'Sylvan Library': lose_life(g, p, 4, p)
            if n == 'Call of the Ring': lose_life(g, p, 2, p)
            if 'erebos' in t: lose_life(g, p, 2, p)
            if 'upkprolif' in t:
                army = army_of(p)
                if army: army.plus += 1
                lose_life(g, p, 1, p)
        if 'spider' in t:                          # saga chapters III and IV draw, then it's sacrificed
            if m.age in (2, 3): draw(g, p, 1)
            if m.age >= 3: leave(g, m); p.gy.append(m.cd); continue
        if 'bolas' in t and m.age <= 5:            # +1 each turn: draw; each opponent loses a card
            draw(g, p, 1)
            for q in g.opps(p):
                if q.hand: q.gy.append(q.hand.pop(g.rng.randrange(len(q.hand))))
        if 'pwdiscard' in t and m.age <= 3:        # Ral Zarek -1: each opponent discards
            for q in g.opps(p):
                if q.hand: q.gy.append(q.hand.pop(g.rng.randrange(len(q.hand))))
        if 'mycoloth' in t and m.plus > 0: make_tokens(g, p, m.plus, 1, color='G')
        if 'tokup' in t: make_tokens(g, p, int(t['tokup']), 1, warrior='warrior' in t)
        if 'sheoW' in t:
            cr = [c for c in p.gy if c.creature]
            if cr:
                b = max(cr, key=lambda c: seph_bval(g, p, c)); p.gy.remove(b)
                was = b.name in p.removed_bombs
                enter(g, p, b); note_bomb(p, b, was)
            for q in g.opps(p): edict(g, q)
    if g.goldfish and p.key == 'sauron' and has(p, 'sauron'):
        amass(g, p, g.rng.randint(2, 6))
    check_state(g)


def end_step(g, p):
    if E.DSLMOD is not None and g.dsl_on: E.DSLMOD.fire(g, 'end_step', player=p)
    for m in [m for m in p.perms if m.temp]: leave(g, m)
    if has(p, 'pvprolif'):                        # Atraxa, Praetors' Voice: proliferate
        for m in p.perms:
            if m.plus > 0: m.plus += 1
    while len(p.hand) > 7 and not has(p, 'nomax'):
        if p.key == 'seph':
            bombs = [c for c in p.hand if c.creature and c.bomb >= 6]
            if bombs:
                c = max(bombs, key=lambda c: c.bomb); p.hand.remove(c); p.gy.append(c); continue
        lands = [c for c in p.hand if c.land]
        c = lands[0] if len(lands) >= 2 else max(p.hand, key=lambda c: c.cmc)
        p.hand.remove(c); p.gy.append(c)
    if p.key == 'seph':
        if bomb_on_bf(p): p.milestone.setdefault('bomb_by', p.turns)
        held = [c for c in p.hand if 'ctr' in c.tags or c.tags.get('prot') == 'hi' or 'tide' in c.tags]
        up = any(can_pay(g, p, c.generic, c.pips) for c in held)
        p.milestone.setdefault('int_up', {})[p.turns] = up
        p.milestone.setdefault('int_held', {})[p.turns] = bool(held)


MAIN = {'seph': seph_main, 'veyran': veyran_main, 'sauron': sauron_main, 'najeela': najeela_main}


def main_fn(p):
    if E.AI_MODE == 'adaptive':
        import brain
        return brain.main
    return MAIN[p.key]


def take_turn(g, p):
    g.active = p
    g.eot_pt = {}; g.eot_kw = {}
    for q in g.players: q.floatA = 0
    p.extra_combats = 0
    p.turns += 1
    for q in g.players: q.floatR = 0          # floating mana empties between turns
    for L in p.lands: L.tapped = False
    for m in list(p.perms):
        m.tapped = False; m.sick = False; m.phased = False; m.age += 1
    p.spells_this_turn = 0; p.yawg = False
    p.pump = 0; p.pumpadd = 0; p.trample = False; p.combo_tried = False
    p.haste_all = False; p.najeela_boost = False
    log(f'--- {NAME(p)} turn {p.turns}: life {p.life}, {len(p.hand)} cards in hand, {len(p.lands)} lands', g)
    upkeep(g, p)
    if g.over or not p.alive: return
    if not (p.key == 'seph' and seph_dredge(g, p)):
        draw(g, p, 1, step=True)
    check_state(g)
    if g.over or not p.alive: return
    nl = len(p.lands)
    play_land(g, p)
    if len(p.lands) == nl and p.turns <= 5: p.stats['land_miss'] += 1
    if p.turns in (4, 6): p.stats[f'mana_T{p.turns}'] = total_mana(g, p); p.stats[f'had_T{p.turns}'] = 1
    main_fn(p)(g, p, False)
    if g.over or not p.alive: return
    if E.AI_MODE != 'adaptive' and E.DSLMOD is not None and g.dsl_on: E.DSLMOD.rigid_abilities(g, p)
    if p.key == 'najeela' and has(p, 'mirror') and not g.goldfish:
        x = total_mana(g, p) - 5
        if x >= 3: p.pump = x
    combat(g, p)
    p.pump = 0
    if g.over or not p.alive: return
    main_fn(p)(g, p, True)
    if p.key == 'veyran' and has(p, 'veyran') and engine_payoff(p): p.milestone.setdefault('engine', p.turns)
    if g.over or not p.alive: return
    end_step(g, p)
    check_state(g)


def mulligan(g, p):
    def draw7():
        p.library.extend(p.hand); p.hand = []; g.rng.shuffle(p.library)
        for _ in range(7): p.hand.append(p.library.pop())

    def ok():
        return 2 <= sum(1 for c in p.hand if c.land) <= 5
    draw7()
    if ok(): return
    draw7()
    if ok(): return
    for bottom in (1, 2):
        draw7()
        if ok() or bottom == 2:
            for _ in range(bottom):
                lands = [c for c in p.hand if c.land]
                spells = [c for c in p.hand if not c.land]
                c = (lands[0] if len(lands) > 4 or not spells else max(spells, key=lambda c: c.cmc))
                p.hand.remove(c); p.library.insert(0, c)
            p.stats['mulls'] = bottom + 1
            return


CMDS = {'seph': 'Atraxa, Grand Unifier', 'veyran': 'Veyran, Voice of Duality',
        'sauron': 'Sauron, the Dark Lord', 'najeela': 'Najeela, the Blade-Blossom'}


def play_game(seed, decks, max_rounds=20, trace=False):
    rng = random.Random(seed)
    players = [Player(k, decks[k], CMDS[k]) for k in ('seph', 'veyran', 'sauron', 'najeela')]
    rng.shuffle(players)
    g = Game(players, rng)
    E.CUR_G = g
    if trace:
        g.log = ['Seat order: ' + ', '.join(NAME(p) for p in players)]
    for p in players:
        rng.shuffle(p.library); mulligan(g, p)
        p.seen_names.update(c.name for c in p.hand)
    for r in range(1, max_rounds + 1):
        g.round = r
        for p in players:
            if p.alive and not g.over:
                if E.AI_MODE == 'adaptive' and r > 1:
                    import brain
                    brain.end_of_turn_window(g, p)     # spend mana held through the other turns
                if p.alive and not g.over:
                    take_turn(g, p)
        if g.over: break
    if not g.over:
        alive = [p for p in players if p.alive]
        g.winner = max(alive, key=lambda p: p.life + board_power(g, p) * 2) if alive else None
        g.wintype = 'timeout'
    return g


def goldfish(seed, key, cards, turns=10):
    rng = random.Random(seed)
    p = Player(key, cards, CMDS[key])
    g = Game([p], rng, goldfish=True)
    rng.shuffle(p.library); mulligan(g, p)
    for t in range(turns):
        g.round = t + 1
        take_turn(g, p)
        if not p.alive: break
    return g, p