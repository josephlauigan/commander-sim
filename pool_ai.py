"""AI for outside decks (opponent pools): a generic cast priority built from card tags, plus per-deck
configuration (play style, tutor wish lists, key cards) registered by pools.register().

The four main decks never reach this module; they keep their hand-written priorities in ais.py.
"""
from engine import has, total_mana, pval
import engine as E

DEFAULT_STYLE = {'temp': 1.0, 'aggression': 0.6, 'caution': 0.5}
CONFIG = {}          # deck key -> {'style': {...}, 'wish': [card names], 'key_cards': {name: priority}, ...}


def config(p):
    return CONFIG.get(p.key, {})


def style(p):
    return config(p).get('style', DEFAULT_STYLE)


def generic_prio(g, p, c):
    """0-90 cast priority from what the card is tagged to do. 0 = not cast proactively (held interaction,
    or no tags: the adaptive AI then falls back to the ability interpreter's value estimate)."""
    t = c.tags
    cfg = config(p)
    kc = cfg.get('key_cards', {})
    if c.name in kc: return kc[c.name]
    if c is p.cmd: return cfg.get('cmd_prio', 75) if p.turns >= cfg.get('cmd_turn', 2) else 0
    if 'rock' in t or 'dork' in t or 'lr' in t or 'fastmana' in t: return 85 if p.turns <= 5 else 38
    if 'chromemox' in t:
        spare = [x for x in p.hand if not x.land and 'A' not in x.types and set(x.pips) & set(p.ident)]
        return (82 if p.turns <= 5 else 30) if len(spare) >= 2 else 0
    if 'moxd' in t:
        n = sum(1 for x in p.hand if x.land)
        return (82 if p.turns <= 5 else 30) if n >= 2 or (n >= 1 and p.land_turn == p.turns) else 0
    if 'ctr' in t or 'rem' in t or 'wipe' in t or t.get('prot') or 'tide' in t: return 0   # held / cast by the response logic
    if any(k in t for k in ('rean', 'fill', 'yawg', 'avarice', 'mastery', 'crackle')) and not c.dsl: return 0
    if 'tokx' in t: return 50 if total_mana(E.CUR_G, p) >= 5 else 0
    if 'rhystic' in t or 'tithe' in t or 'eng' in t or 'necro' in t: return 64
    if 'seal' in t: return 56 if p.life >= 15 else 20            # Vampiric Tutor / Imperial Seal
    if 'jeska' in t: return 52
    if 'intuition' in t or 'gifts' in t: return 50
    if 'adnaus' in t: return 58 if p.life >= 30 else 0
    if 'citadel' in t: return 58 if p.life >= 25 else 20
    if 'agent' in t: return 50
    if 'braids' in t or 'birgi' in t: return 45
    if 'rabble' in t: return 55
    if 'stampede' in t: return 55 if sum(1 for m in p.perms if m.creature) >= 6 else 0
    if 'drawcre' in t: return 55 if sum(1 for m in p.perms if m.creature) >= 4 else 20
    if 'clamp' in t: return 55
    if 'crusade' in t or 'anth' in t or 'warleader' in t: return 60
    if any(k in t for k in ('tokup', 'tokatk', 'tok', 'spelltok', 'ping', 'kiln', 'spelldraw', 'drain', 'bartist')): return 62
    if t.get('tut'): return 58
    if 'draw' in t and (c.instant or c.sorcery): return 46
    if 'draw' in t: return 52
    if 'treas' in t or 'mktok' in t or 'drainetb' in t or 'edictetb' in t: return 50
    if c.creature: return 42 + min(16, 2 * c.pow) + (4 if 'fly' in t else 0)
    if t.get('prot') == 'boots' or 'sac' in t: return 40
    if c.dsl: return 0                       # interpreter value decides
    if 'pumpall' in t: return 0              # combat trick: no proactive value
    return 0


def tutor_pick(g, p, kind, ok):
    """a deck's wish list first (combo pieces it is missing), else None (caller falls back to priority)"""
    wish = config(p).get('wish')
    if not wish: return None
    names = {c.name for c in p.library if ok(c)}
    have = {m.cd.name for m in p.perms if m.cd is not None} | {c.name for c in p.hand}
    for w in (wish(g, p) if callable(wish) else wish):
        if w in names and w not in have: return w
    return None


EQUIP_COST = {'Lightning Greaves': 0, 'Swiftfoot Boots': 1, 'Whispersilk Cloak': 2}


def special_options(g, p, s, post):
    """generic activated plays for outside decks: equip Boots / Greaves / Cloak, Skullclamp, special spells"""
    o = spell_options(g, p, s, post)
    if post is None: return o                                    # end-of-turn window: nothing here is instant speed
    for e in p.perms:
        if e.cd is None or e.phased or e.cd.name not in EQUIP_COST: continue
        if e.attached is not None and e.attached in p.perms and not e.attached.phased: continue
        n = EQUIP_COST[e.cd.name]
        if not E.can_pay(g, p, n, ''): continue
        cr = [m for m in p.perms if m.creature and not m.phased and not m.noatk]
        if not cr: continue
        best = max(cr, key=lambda m: (m.is_cmd * 5 + pval(g, m) + (2 if m.sick else 0)))
        u = 2.0 + 0.4 * pval(g, best) + (1.5 if best.sick and not post else 0) - 0.5 * n

        def go(e=e, best=best, n=n):
            if best not in p.perms or not E.can_pay(g, p, n, ''): return False
            E.pay(g, p, n, ''); e.attached = best
            E.log(f'  {E.NAME(p)} equips {e.cd.name} to {best.name}', g); return True
        o.append((u, f'equip {e.cd.name}', go))
    for e in E.find(p, 'clamp'):                              # Skullclamp: equip {1} to an X/1, it dies, draw two
        if E.stopped(g, e.cd.name) or not E.can_pay(g, p, 1, ''): continue
        fod = [m for m in p.perms if m.creature and not m.phased and E.etgh(g, m) == 1 and (m.token or pval(g, m) < 3)]
        if not fod: continue
        m = min(fod, key=lambda x: pval(g, x))

        def clamp(e=e, m=m):
            if m not in p.perms or not E.can_pay(g, p, 1, ''): return False
            E.pay(g, p, 1, ''); e.attached = m
            E.log(f'  {E.NAME(p)} Skullclamps {m.name}', g)
            E.die(g, m, 'sba'); E.draw(g, p, 2); p.stats['clamp_draws'] += 2
            return True
        o.append((4.0 - 0.3 * pval(g, m), 'Skullclamp', clamp))
    return o


# ------------------------------------------------------------------ protection responses (outside decks)
DESTROYISH = ('destroy',)                                    # plus every 'dmgN'
WIPE_INDES = ('destroy', 'dmg13', 'austere', 'austere2', 'nib', 'vandal')


def _destroyish(kind):
    return kind in DESTROYISH or kind.startswith('dmg')


# name: (where, cost (generic, pips) or None, what it saves)
#   saves: 'all_targeted'  any targeted removal of that permanent       'indes' destroy / damage only
#          'color'         targeted removal from a coloured source        'blink' any targeted removal (creature, nontoken)
#   scope: 'one' (the threatened permanent) / 'creatures' / 'perms'   free_cmd: free while you control your commander
PROTECTORS = {
    'Heroic Intervention':     ('hand', (1, 'G'), 'hexproof_indes', 'perms', False),
    "Teferi's Protection":     ('hand', (2, 'W'), 'phase', 'perms', False),
    'Flawless Maneuver':       ('hand', (2, 'W'), 'indes', 'creatures', True),
    'Boros Charm':             ('hand', (0, 'RW'), 'indes', 'perms', False),
    'Deflecting Swat':         ('hand', (2, 'R'), 'all_targeted', 'one', True),
    'Gods Willing':            ('hand', (0, 'W'), 'color', 'one', False),
    'Valorous Stance':         ('hand', (1, 'W'), 'indes', 'one', False),
    'Ephemerate':              ('hand', (0, 'W'), 'blink', 'one', False),
    'Restoration Angel':       ('hand', (3, 'W'), 'blink', 'one', False),
    'Selfless Spirit':         ('bf_sac', None, 'indes', 'creatures', False),
    'Mother of Runes':         ('bf_tap', None, 'color', 'one', False),
    'Giver of Runes':          ('bf_tap', None, 'all_targeted', 'one', False),      # colorless or a colour
    'Benevolent Bodyguard':    ('bf_sac', None, 'color', 'one', False),
    "Alseid of Life's Bounty": ('bf_sac', (1, ''), 'color', 'one', False),
}


def worth_protecting(g, p, m):
    kc = config(p).get('key_cards', {})
    return pval(g, m) >= 4 or m.is_cmd or (m.cd is not None and m.cd.name in kc)


def _saves(how, kind, m, spell, targeted):
    if how in ('phase',): return True
    if how == 'hexproof_indes': return targeted or kind in WIPE_INDES
    if how == 'indes': return _destroyish(kind) or (not targeted and kind in WIPE_INDES)
    if not targeted: return False
    if how == 'all_targeted': return True
    if how == 'blink': return m.creature and not m.token
    if how == 'color': return spell is not None and bool(set(spell.pips) & set('WUBRG'))
    return False


def _use(g, p, name, where, cost, src, target_m):
    """pay for and use protector `name`; True if it happened"""
    if where == 'hand':
        c = next((x for x in p.hand if x.name == name), None)
        if c is None or not E.castable(g, p, c): return False
        free = PROTECTORS[name][4] and E.commander_out(p)
        if not free:
            gen, pips = cost
            if not E.can_pay(g, p, gen, pips): return False
            E.pay(g, p, gen, pips)
        p.hand.remove(c); p.gy.append(c); p.spells_this_turn += 1; p.stats['spells_cast'] += 1
        p.cast_names.add(c.name); E.on_cast(g, p, c)
        if name == 'Restoration Angel':
            p.gy.remove(c); E.enter(g, p, c)
    else:
        if src is None or src not in p.perms: return False
        if where == 'bf_tap':
            if src.tapped or src.sick: return False
            src.tapped = True
        elif where == 'bf_sac':
            if cost and not E.can_pay(g, p, *cost): return False
            if cost: E.pay(g, p, *cost)
            E.die(g, src, 'sac')
    p.stats['protection_used'] += 1
    E.log(f'    {E.NAME(p)} protects with {name}', g)
    return True


def _options(p, m):
    """(name, where, cost, src) protectors p could use now"""
    out = []
    for c in p.hand:
        if c.name in PROTECTORS and PROTECTORS[c.name][0] == 'hand': out.append((c.name, 'hand', PROTECTORS[c.name][1], None))
    for x in p.perms:
        if x.cd is not None and x.cd.name in PROTECTORS and PROTECTORS[x.cd.name][0] != 'hand' and not x.phased:
            if x.cd.name == 'Giver of Runes' and x is m: continue          # "another target creature"
            out.append((x.cd.name, PROTECTORS[x.cd.name][0], PROTECTORS[x.cd.name][1], x))
    return out


def protect(g, owner, m, kind, actor, spell=None):
    """targeted removal is aimed at owner's permanent m: respond if it's worth it"""
    if not worth_protecting(g, owner, m): return False
    cands = []
    for name, where, cost, src in _options(owner, m):
        how, scope = PROTECTORS[name][2], PROTECTORS[name][3]
        if scope in ('one', 'creatures') and not m.creature and how not in ('hexproof_indes', 'phase', 'all_targeted'): continue
        if not _saves(how, kind, m, spell, True): continue
        # cheapest first: permanents that tap, then one-shot cards; save the board-wide ones for wipes
        rank = {'bf_tap': 0, 'bf_sac': 2, 'hand': 1}[where] + (3 if scope != 'one' else 0)
        cands.append((rank, name, where, cost, src))
    for _, name, where, cost, src in sorted(cands, key=lambda x: x[0]):
        if _use(g, owner, name, where, cost, src, m):
            how = PROTECTORS[name][2]
            if how == 'phase':
                for x in owner.perms: x.phased = True
            elif how == 'blink' and m in owner.perms:
                cd = m.cd; E.leave(g, m); n = E.enter(g, owner, cd, orig=m.orig); n.is_cmd = m.is_cmd
            return True
    return False


def wipe_response(g, q, kind, caster):
    """a board wipe is about to hit q: 'all' (everything saved), 'indes' (indestructible), or None"""
    loss = sum(pval(g, m) for m in q.perms if m.creature or kind in ('rift', 'rebuke'))
    if loss < 6: return None
    for name, where, cost, src in sorted(_options(q, None), key=lambda o: o[1] != 'bf_sac'):
        how, scope = PROTECTORS[name][2], PROTECTORS[name][3]
        if scope == 'one' or not _saves(how, kind, None, None, False): continue
        if _use(g, q, name, where, cost, src, None):
            if how == 'phase':
                for m in q.perms: m.phased = True
                return 'all'
            return 'indes'
    return None


def spell_options(g, p, s, post):
    """hand-tagged spells whose casting logic lives in Sephiroth's AI, made usable by outside decks:
    Deadly Dispute (sacrifice an artifact or creature) and reanimation (Animate Dead, Reanimate)"""
    import ais as A
    o = []
    if post is None: return o
    for c in p.hand:
        t = c.tags
        if t.get('fill') == 'dispute' and E.castable(g, p, c) and E.can_pay(g, p, c.generic, c.pips):
            fod = E.sac_fodder(g, p, 'artifact or creature')
            if fod is None or (fod != 'Treasure' and pval(g, fod) >= 3): continue

            def dispute(c=c):
                fod = E.sac_fodder(g, p, 'artifact or creature')
                if c not in p.hand or fod is None or not E.can_pay(g, p, c.generic, c.pips): return False
                E.pay(g, p, c.generic, c.pips)
                if fod == 'Treasure':
                    p.treasures -= 1
                    if g.hooks: E.CI.fire(g, 'sacrifice', p, 'Treasure')
                else: E.die(g, fod, 'sac')
                E.cast_card(g, p, c, 'hand', {}); return True
            o.append((3.5, c.name, dispute))
        if t.get('rean') in ('animate', 'reanimate', 'evil') and E.castable(g, p, c) and E.can_pay(g, p, c.generic, c.pips):
            tg = A.rean_targets(g, p, t['rean'])
            if t['rean'] == 'reanimate': tg = [x for x in tg if p.life - x[1].cmc >= 12]
            if not tg: continue
            v, cd, src = tg[0]

            def rean(c=c, cd=cd, src=src):
                if c not in p.hand or cd not in src.gy or not E.can_pay(g, p, c.generic, c.pips): return False
                E.pay(g, p, c.generic, c.pips)
                E.cast_card(g, p, c, 'hand', {'rean_target': cd, 'rean_src': src, 'rean_value': v}); return True
            o.append((v * 0.9 * (1 - 0.35 * s.ctr_risk), f'{c.name} -> {cd.name}', rean))
    return o
