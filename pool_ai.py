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
    fn = cfg.get('prio_fn')
    if fn is not None:
        r = fn(g, p, c)
        if r is not None: return r
    if E.CI is not None and c.name in E.CI.SPELL_PRIO:
        v = E.CI.SPELL_PRIO[c.name]
        return v(g, p, c) if callable(v) else v
    if c is p.cmd: return cfg.get('cmd_prio', 75) if p.turns >= cfg.get('cmd_turn', 2) else 0
    if 'rock' in t or 'dork' in t or 'lr' in t or 'fastmana' in t: return 85 if p.turns <= 5 else 38
    if 'chromemox' in t:
        spare = [x for x in p.hand if not x.land and 'A' not in x.types and set(x.pips) & set(p.ident)]
        return (82 if p.turns <= 5 else 30) if len(spare) >= 2 else 0
    if 'moxd' in t:
        n = sum(1 for x in p.hand if x.land)
        return (82 if p.turns <= 5 else 30) if n >= 2 or (n >= 1 and p.land_turn == p.turns) else 0
    if t.get('prot') == 'boots' or 'cloak' in t: return 45 if any(m.creature for m in p.perms) else 25
    if 'rem' in t and 'etb' in t and c.perm:                     # creature / enchantment with an ETB removal
        tg = E.legal_targets(E.CUR_G, p, t['rem'], t.get('tgt', 'c'), 'mv4' in t, spell=c)
        best = max((pval(E.CUR_G, m) for m in tg), default=0)
        return 45 + min(30, 6 * best) if best >= 2 else (35 if c.creature else 0)
    if 'ctr' in t or 'rem' in t or 'wipe' in t or t.get('prot') or 'tide' in t: return 0   # held / cast by the response logic
    if any(k in t for k in ('rean', 'fill', 'yawg', 'avarice', 'mastery', 'crackle')) and not c.dsl and not c.creature: return 0
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
    if 'xtutor' in t: return 0                                     # SPELL_PRIO decides (X spells)
    if 'aura' in t:                                            # an Aura needs a creature to go on
        if not any(m.creature and not m.phased for m in p.perms): return 0
        return cfg.get('aura_prio', 55)
    if 'threedreams' in t: return 60
    if 'explore' in t: return 50 if any(x.land for x in p.hand if x is not c) else 30
    if 'loam' in t: return 45 if sum(1 for x in p.gy if x.land) >= 2 else 0
    if 'rishkar' in t: return 55 if max((E.epow(g, m) for m in p.perms if m.creature), default=0) >= 4 else 0
    if 'krasis' in t: return 0                                     # cast by special_options with X
    if 'combatspell' in t: return 0                                # SPELL_PRIO decides
    if 'fable' in t: return 58
    if c.creature: return 42 + min(16, 2 * c.pow) + (4 if 'fly' in t else 0)
    if c.perm and E.CI is not None and c.name in E.CI.HOOKS: return cfg.get('hooked_prio', 55)
    if t.get('prot') == 'boots' or 'sac' in t: return 40
    if c.dsl: return 0                       # interpreter value decides
    if 'pumpall' in t: return 0              # combat trick: no proactive value
    return 0


def tutor_pick(g, p, kind, ok):
    """a deck's wish list first (combo pieces it is missing), else None (caller falls back to priority)"""
    wish = config(p).get('wish')
    if not wish:
        import impl_combos
        wish = impl_combos.missing_pieces
    names = {c.name for c in p.library if ok(c)}
    have = {m.cd.name for m in p.perms if m.cd is not None} | {c.name for c in p.hand}
    for w in (wish(g, p) if callable(wish) else wish):
        if w in names and w not in have: return w
    return None


EQUIP_COST = {'Lightning Greaves': 0, 'Swiftfoot Boots': 1, 'Whispersilk Cloak': 2}


def special_options(g, p, s, post):
    """generic activated plays for outside decks: equip Boots / Greaves / Cloak, Skullclamp, special spells"""
    o = spell_options(g, p, s, post)
    import impl_common
    o += impl_common.adventure_options(g, p, s, post)
    import impl_t2
    o += impl_t2.evoke_options(g, p, s, post)
    o += impl_common.aristocrat_options(g, p, s, post)
    o += impl_common.food_options(g, p, s, post)
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
    'Dream Trawler':           ('bf_self_discard', None, 'all_targeted', 'self', False),
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
        p.hand.remove(c); p.spells_this_turn += 1; p.stats['spells_cast'] += 1
        if name == 'Ephemerate':
            p.exile.append(c); p.rebound = getattr(p, 'rebound', []) + [c]
        elif name != 'Restoration Angel': p.gy.append(c)
        p.cast_names.add(c.name); E.on_cast(g, p, c)
        if name == 'Restoration Angel': E.enter(g, p, c)
    else:
        if src is None or src not in p.perms: return False
        if where == 'bf_tap':
            if src.tapped or src.sick: return False
            src.tapped = True
        elif where == 'bf_self_discard':
            if not p.hand: return False
            E.discard_worst(g, p, 1); src.tapped = True
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
            if PROTECTORS[x.cd.name][3] == 'self' and x is not m: continue
            out.append((x.cd.name, PROTECTORS[x.cd.name][0], PROTECTORS[x.cd.name][1], x))
    return out


def protect(g, owner, m, kind, actor, spell=None):
    """targeted removal is aimed at owner's permanent m: respond if it's worth it"""
    if not worth_protecting(g, owner, m):
        import impl_common
        return impl_common.sac_in_response(g, owner, m, kind)
    cands = []
    for name, where, cost, src in _options(owner, m):
        how, scope = PROTECTORS[name][2], PROTECTORS[name][3]
        if scope in ('one', 'creatures') and not m.creature and how not in ('hexproof_indes', 'phase', 'all_targeted'): continue
        if not _saves(how, kind, m, spell, True): continue
        # cheapest first: permanents that tap, then one-shot cards; save the board-wide ones for wipes
        rank = {'bf_tap': 0, 'bf_sac': 2, 'hand': 1, 'bf_self_discard': 0}[where] + (3 if scope not in ('one', 'self') else 0)
        cands.append((rank, name, where, cost, src))
    for _, name, where, cost, src in sorted(cands, key=lambda x: x[0]):
        if _use(g, owner, name, where, cost, src, m):
            how = PROTECTORS[name][2]
            if how == 'phase':
                for x in owner.perms: x.phased = True
            elif how == 'blink' and m in owner.perms:
                cd = m.cd; E.leave(g, m); n = E.enter(g, owner, cd, orig=m.orig); n.is_cmd = m.is_cmd
            return True
    import impl_common
    return impl_common.sac_in_response(g, owner, m, kind)


def wipe_response(g, q, kind, caster):
    """a board wipe is about to hit q: 'all' (everything saved), 'indes' (indestructible), or None"""
    loss = sum(pval(g, m) for m in q.perms if m.creature or kind in ('rift', 'rebuke'))
    if loss < 6: return None
    for name, where, cost, src in sorted(_options(q, None), key=lambda o: o[1] != 'bf_sac'):
        how, scope = PROTECTORS[name][2], PROTECTORS[name][3]
        if scope in ('one', 'self') or not _saves(how, kind, None, None, False): continue
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
        if t.get('fill') == 'grisly' and E.castable(g, p, c) and E.can_pay(g, p, c.generic, c.pips) and \
                (len(p.lands) < 6 or any(m.cd is not None and m.cd.name == 'Meren of Clan Nel Toth' for m in p.perms)):
            def grisly(c=c):
                if c not in p.hand or not E.can_pay(g, p, c.generic, c.pips): return False
                E.pay(g, p, c.generic, c.pips); E.cast_card(g, p, c, 'hand', {}); return True
            o.append((2.5, c.name, grisly))
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
                if g.hooks and E.CI.gy_response(g, p, v, src):
                    p.hand.remove(c); p.gy.append(c); return True
                E.cast_card(g, p, c, 'hand', {'rean_target': cd, 'rean_src': src, 'rean_value': v}); return True
            o.append((v * 0.9 * (1 - 0.35 * s.ctr_risk), f'{c.name} -> {cd.name}', rean))
        if 'krasis' in t and E.castable(g, p, c) and post is not None:                    # Hydroid Krasis: X = spare mana
            x = E.total_mana(g, p) - 2
            if x >= 4:
                def krasis(c=c):
                    x = E.total_mana(g, p) - 2
                    while x >= 1 and not E.can_pay(g, p, x, 'GU'): x -= 1
                    if c not in p.hand or x < 1: return False
                    E.pay(g, p, x, 'GU')
                    p.hand.remove(c); p.cast_names.add(c.name); p.spells_this_turn += 1; E.on_cast(g, p, c)
                    E.gain(p, x // 2); E.draw(g, p, x // 2)
                    m = E.enter(g, p, c, was_cast=True); m.plus += x; return True
                o.append((1.0 + 0.6 * x, f'Hydroid Krasis X={x}', krasis))
        if 'primalmight' in t and E.castable(g, p, c) and post is not None:                # Primal Might: pump + fight
            mine = [m for m in p.perms if m.creature and not m.phased]
            x = E.total_mana(g, p) - 1
            if mine and x >= 1:
                me = max(mine, key=lambda m: E.epow(g, m))
                tg = [m for q in g.opps(p) for m in q.perms if m.creature and not E.untargetable(g, m) and
                      E.etgh(g, m) <= E.epow(g, me) + x]
                if tg:
                    foe = max(tg, key=lambda m: pval(g, m))
                    if pval(g, foe) >= 3:
                        def might(c=c, me=me, foe=foe):
                            x = E.total_mana(g, p) - 1
                            if c not in p.hand or x < 0 or not E.can_pay(g, p, x, 'G'): return False
                            E.pay(g, p, x, 'G'); p.hand.remove(c); p.gy.append(c); E.on_cast(g, p, c)
                            import cardimpl; cardimpl._eot(g, me, x, x)
                            if foe in foe.owner.perms and me in p.perms: E.apply_removal(g, p, foe, f'dmg{E.epow(g, me)}', c)
                            return True
                        o.append((pval(g, foe) - 2.5, f'Primal Might -> {foe.name}', might))
        if 'rotw' in t and E.castable(g, p, c) and E.can_pay(g, p, c.generic, c.pips):      # Return of the Wildspeaker
            nh = [m for m in p.perms if m.creature and not m.phased and not E.has_type(m, 'human')]
            if not nh: continue
            draw_n = max(E.epow(g, m) for m in nh)
            atk = [m for m in nh if not m.sick and not m.tapped and not m.noatk]
            if post is False and len(atk) >= 3:
                def pump(c=c):
                    if c not in p.hand or not E.can_pay(g, p, c.generic, c.pips): return False
                    E.pay(g, p, c.generic, c.pips); p.hand.remove(c); p.gy.append(c); E.on_cast(g, p, c)
                    for m in p.perms:
                        if m.creature and not E.has_type(m, 'human'):
                            a0, b0 = g.eot_pt.get(id(m), (0, 0)); g.eot_pt[id(m)] = (a0 + 3, b0 + 3)
                    return True
                o.append((1.0 + 1.0 * len(atk), f'{c.name} (+3/+3)', pump))
            elif draw_n >= 3:
                def rdraw(c=c, n=draw_n):
                    if c not in p.hand or not E.can_pay(g, p, c.generic, c.pips): return False
                    E.pay(g, p, c.generic, c.pips); p.hand.remove(c); p.gy.append(c); E.on_cast(g, p, c)
                    E.draw(g, p, n); return True
                o.append((1.0 + 0.8 * draw_n, f'{c.name} (draw {draw_n})', rdraw))
    return o
