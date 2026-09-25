"""Card rules for cards in the main decks (decklists/mine/) that need more than their hand tags: equipment, lands with
abilities, and cards whose full text the tag model left out. Hooks here are live in every game (pool rules).
"""
from engine import *
import engine as E
import cardimpl as CI
from cardimpl import on
from pool_cards import card, note


def full(name, text): note(name, 'Full', text)


# ================================================================== Sephiroth
# ------------------------------------------------------------------ Displacer Kitten
ETB_VALUE = {'atraxa': 8.0, 'rsd': 4.0, 'witness': 2.5, 'wall': 2.0, 'wurm': 4.0, 'bowmasters': 2.5, 'skate': 3.0}


def etb_value(g, p, m):
    """what re-entering the battlefield is worth for permanent m (its enters-the-battlefield effects)"""
    if m.cd is None: return 0.0
    t = m.cd.tags
    v = sum(x for k, x in ETB_VALUE.items() if k in t)
    if 'draw' in t and m.cd.perm: v += 1.5 * int(t['draw'])
    if CI.live(m.cd.name) and 'etb' in CI.HOOKS[m.cd.name]: v = max(v, 2.0)
    if m.cd.start_loyalty and m.loyalty is not None: v = max(v, 0.6 * (int(m.cd.start_loyalty) - m.loyalty))
    v -= 0.5 * max(0, m.plus)                                  # counters are lost
    if getattr(g, 'auras', None) and m.creature: v -= 2.0 * len(CI.auras_on(g, m))
    return v


@on('Displacer Kitten', 'cast')
def _kitten(g, src, caster, c):
    """whenever you cast a noncreature spell: exile up to one other nonland permanent you control, return it"""
    p = src.owner
    if caster is not p or c.creature or c.land or src not in p.perms or src.phased: return
    cands = [m for m in p.perms if m is not src and not m.token and not m.phased and m.cd is not None and m.orig is p]
    if not cands: return
    m = max(cands, key=lambda x: etb_value(g, p, x))
    if etb_value(g, p, m) < 2.0: return
    cd, was_cmd = m.cd, m.is_cmd
    log(f'    Displacer Kitten flickers {m.name}', g)
    leave(g, m)
    n = enter(g, p, cd, orig=p)
    n.is_cmd = was_cmd
card('Displacer Kitten', 'pow=2 tgh=2 noatk', types='C', dsl=[])
full('Displacer Kitten', 'each noncreature spell you cast flickers your permanent with the best enters-the-battlefield '
     'value (Atraxa, Grand Unifier, Eternal Witness ...); stolen permanents are left alone')


# ------------------------------------------------------------------ Nim Deathmantle
@on('Nim Deathmantle', 'creature_to_gy')
def _nim_return(g, src, m):
    """whenever a nontoken creature is put into your graveyard from the battlefield, you may pay {4}: return it and
    attach this Equipment to it"""
    p = src.owner
    if src not in p.perms or src.phased or m.token or m.orig is not p or m.cd not in p.gy: return
    if m.cd is p.cmd or not can_pay(g, p, 4, ''): return
    if pval(g, m) < 3 and etb_value(g, p, m) < 2.5: return
    pay(g, p, 4, '')
    p.gy.remove(m.cd)
    n = enter(g, p, m.cd, orig=p)
    src.attached = n
    log(f'    {NAME(p)} pays 4: Nim Deathmantle returns {n.name}', g)


@on('Nim Deathmantle', 'options')
def _nim_equip(g, src, p, s, post):
    """equip {4}: onto your best creature (a bomb first)"""
    if p is not src.owner or post is None or not can_pay(g, p, 4, ''): return []
    if src.attached is not None and src.attached in p.perms and not src.attached.phased: return []
    cre = [m for m in p.perms if m.creature and not m.phased and m.cd is not None]
    if not cre: return []
    t = max(cre, key=lambda m: pval(g, m))

    def go():
        if not can_pay(g, p, 4, '') or t not in p.perms: return False
        pay(g, p, 4, ''); src.attached = t
        log(f'  {NAME(p)} equips Nim Deathmantle to {t.name}', g); return True
    return [(1.0 + 0.2 * pval(g, t), 'equip Nim Deathmantle', go)]
card('Nim Deathmantle', 'nim', types='A', dsl=[])
full('Nim Deathmantle', 'equipped creature +2/+2, intimidate, black Zombie; returns your nontoken creatures that die '
     'for {4} (worth-it check) and re-attaches; equip {4}')


# ------------------------------------------------------------------ Triskelion
@on('Triskelion', 'etb')
def _trisk_etb(g, src, p, m):
    if m is src: src.plus += 3


@on('Triskelion', 'options')
def _trisk_ping(g, src, p, s, post):
    """remove a +1/+1 counter: 1 damage to any target (lethal players, X/1 creatures, or its own last counters when
    Nim Deathmantle can bring it back)"""
    if p is not src.owner or src.plus <= 0 or post is None: return []
    opps = g.opps(p)
    lethal = [q for q in opps if q.life <= src.plus]
    tg = [m for q in opps for m in q.perms if m.creature and etgh(g, m) <= 1 and pval(g, m) >= 2.5 and not untargetable(g, m)]
    if not lethal and not tg: return []

    def go():
        if src.plus <= 0 or src not in p.perms: return False
        src.plus -= 1
        if lethal and lethal[0].alive: lose_life(g, lethal[0], 1, p, kind='triggers')
        elif tg and tg[0] in tg[0].owner.perms: apply_removal(g, p, tg[0], 'dmg1')
        if etgh(g, src) <= 0: die(g, src, 'sba')
        return True
    return [(8.0 if lethal else 2.0, 'Triskelion ping', go)]
card('Triskelion', 'pow=1 tgh=1', types='AC', dsl=[])
full('Triskelion', 'enters with three +1/+1 counters; removes counters to ping lethal players or X/1 creatures')


# ------------------------------------------------------------------ Strip Mine
@on('Strip Mine', 'land_options')
def _strip(g, L, p, s, post):
    """{T}, sacrifice: destroy target land (an opponent's key land: Cradle, Coffers, Urborg, Tomb ...)"""
    if L.tapped: return []
    import impl_lands, impl_fixes
    tg = [(q, x) for q in g.opps(p) for x in q.lands if x.cd.name in impl_lands.KEY_LANDS]
    if not tg: return []
    q, x = max(tg, key=lambda t: (t[1].cd.name in ("Gaea's Cradle", 'Cabal Coffers', 'Serra\'s Sanctum', 'Tolarian Academy',
                                                   'Itlimoc, Cradle of the Sun', 'Nykthos, Shrine to Nyx'), threat(g, p, t[0])))

    def go():
        if L not in p.lands or x not in q.lands: return False
        p.lands.remove(L); p.gy.append(L.cd)
        impl_fixes.destroy_land(g, q, x)
        log(f'  {NAME(p)} sacrifices Strip Mine: destroys {x.cd.name} ({NAME(q)})', g); return True
    return [(3.0, f'Strip Mine -> {x.cd.name}', go)]
full('Strip Mine', '{T}: {C}; {T}, sacrifice: destroy an opponent\'s key land (Cradle, Coffers, Urborg, Ancient Tomb ...)')


# ------------------------------------------------------------------ Galadriel's Dismissal
card("Galadriel's Dismissal", 'prot=phase kicker', types='I', dsl=[])
full("Galadriel's Dismissal", '{W}: your threatened bomb phases out; kicked ({2}{W} more): all your creatures phase out '
     'in answer to a sweeper')


# ------------------------------------------------------------------ Necromancy: the flash mode
def card_etb_value(c):
    """enters-the-battlefield value of a creature card (for a flashed-in Necromancy, which keeps nothing else)"""
    t = c.tags
    v = sum(x for k, x in ETB_VALUE.items() if k in t)
    if 'draw' in t: v += 1.5 * int(t['draw'])
    return v


@on('Necromancy', 'hand_options')
def _necro_flash(g, c, p, s, post):
    """cast at instant speed at the end of an opponent's turn for an enters-the-battlefield creature (Atraxa, Grand
    Unifier): the creature is sacrificed at that turn's cleanup and goes back to the graveyard"""
    if post is not None or p.key != 'seph' or g.active is p or c not in p.hand or not can_pay(g, p, 2, 'B'): return []
    import ais as A
    tg = [(card_etb_value(x), x, q) for q in g.players if q.alive for x in q.gy if x.creature]
    tg = [t for t in tg if t[0] >= 4.0]
    if not tg: return []
    val, cd, src = max(tg, key=lambda t: t[0])

    def go():
        if c not in p.hand or cd not in src.gy or not can_pay(g, p, 2, 'B'): return False
        p.hand.remove(c); pay(g, p, 2, 'B')
        p.spells_this_turn += 1; p.cast_names.add(c.name)
        on_cast(g, p, c)
        log(f'  {NAME(p)} flashes in Necromancy for {cd.name}', g)
        if not counter_window(g, p, c, 6, {}) or (g.hooks and CI.gy_response(g, p, 6, src)):
            p.gy.append(c); return True
        n0 = len(p.perms)
        A.seph_rean_resolve(g, p, c, {'rean_target': cd, 'rean_src': src})
        new = [m for m in p.perms[n0:] if m.cd is cd]
        for m in new: die(g, m, 'sac')                      # cast when a sorcery couldn't be: sacrificed at cleanup
        p.gy.append(c); check_state(g)
        return True
    return [(1.5 + 0.5 * val, f'Necromancy (flash) -> {cd.name}', go)]
note('Necromancy', 'Full', 'reanimates at sorcery speed (the aura is abstracted: the creature stays); flashed in at '
     'the end of an opponent\'s turn for an enters-the-battlefield creature, which is sacrificed at cleanup')


note('Stinkweed Imp', 'Full', 'flying; combat damage to a creature destroys it (the same as deathtouch for a 1-power '
     'creature with no other damage); dredge 5 replaces the draw-step draw when a reanimation spell is available')
note('Atraxa, Grand Unifier', 'Full', 'flying, vigilance, deathtouch, lifelink; enters: reveal ten, one card of each '
     'card type to hand (best first; a multi-type card fills its scarcest type), the rest on the bottom')
note('Farewell', 'Full', 'modes by net value: artifacts, creatures, enchantments, and graveyards (theirs against '
     'yours, since your reanimation targets are there)')
note("Yawgmoth's Will", 'Full', 'until end of turn: cast spells and play lands from the graveyard as it was when it '
     'resolved (reanimation spells included); cards that reach your graveyard this turn are exiled')
note('Altar of Dementia', 'Full', 'free sacrifice outlet; each sacrifice mills you for the creature\'s power while the '
     'library can spare it (reanimation targets)')
note('Tortured Existence', 'Full', '{B}, discard a creature: return a creature from your graveyard (bins a bomb for '
     'the reanimation spells and returns the best creature that isn\'t a reanimation target), once per turn')


# ================================================================== Veyran
def _post_veyran():
    """keyword data for hand-tagged cards (their tags carry no keywords)"""
    E.DB['Emeritus of Conflict // Lightning Bolt'].kws = frozenset({'first strike'})
    E.DB['Emeritus of Ideation // Ancestral Recall'].ward = 2
    E.DB['Ral, Storm Conduit'].start_loyalty = '4'
import pool_cards as _PC
_PC.POST.append(_post_veyran)


def spell_copy_value(g, p, c):
    """what one more copy of spell c is worth (Ral's -2, Return the Favor)"""
    t = c.tags
    v = 1.5 * int(t.get('draw', 0)) + (2.0 if t.get('tut') else 0) + (4.0 if 'burn' in t else 0)
    if 'rem' in t:
        tg = [m for m in legal_targets(g, p, t['rem'], t.get('tgt', 'c'), 'mv4' in t, spell=c) if m.owner is not p]
        v += max((pval(g, m) for m in tg), default=0)
    if 'mastery' in t: v += 5.0
    if 'jeska' in t: v += 4.0
    return v + 1.0 * sum(1 for m in p.perms if m.cd is not None and any(k in m.cd.tags for k in ('ping', 'spelltok', 'spelldraw', 'kiln')))


# ------------------------------------------------------------------ Thousand-Year Storm
def _storm_n(g, p):
    """copies: one per instant/sorcery cast before this one this turn; Veyran doubles the trigger"""
    return max(0, is_casts(g, p) - 1) * (2 if has(p, 'veyran') else 1)


@on('Thousand-Year Storm', 'cast')
def _storm(g, src, caster, c):
    p = src.owner
    if caster is not p or not (c.instant or c.sorcery) or src.phased or src not in p.perms: return
    n = _storm_n(g, p)
    if not n: return
    cc = getattr(g, 'cur_cast', None)
    ctx = cc[1] if cc is not None and cc[0] is c else None           # X is copied (Crackle with Power)
    log(f'    Thousand-Year Storm copies {c.name} {n} time(s)', g)
    for _ in range(n):
        copy_spell(g, p, c, ctx)
        if g.over: return


@on('Thousand-Year Storm', 'copycast')
def _storm_copycast(g, src, p, effect):
    """a cast copy of a back-face spell (prepared, Lightning Bolt) is an instant/sorcery cast too"""
    if p is not src.owner or src.phased: return
    for _ in range(_storm_n(g, p)):
        magecraft(g, p, copy=True); effect()
        if g.over: return
card('Thousand-Year Storm', 'tys', types='E', dsl=[])
full('Thousand-Year Storm', 'each instant/sorcery you cast (or cast as a copy) is copied once per instant/sorcery cast '
     'before it this turn, twice with Veyran; copies pick new targets and keep X')


# ------------------------------------------------------------------ Ral, Storm Conduit: loyalty abilities
@on('Ral, Storm Conduit', 'options')
def _ral(g, src, p, s, post):
    """+2: scry 1. -2: copy the next instant or sorcery you cast this turn (when a spell worth copying is in hand)"""
    if p is not src.owner or post is None or src.phased or src.loyalty is None: return []
    if src.data and src.data.get('act') == turn_stamp(g): return []
    best = max((spell_copy_value(g, p, c) for c in p.hand if (c.instant or c.sorcery) and can_pay(g, p, *cost_of(p, c))),
               default=0)

    def act(minus):
        def go():
            if src not in p.perms or (src.data and src.data.get('act') == turn_stamp(g)): return False
            src.data = dict(src.data or {}, act=turn_stamp(g))
            if minus:
                src.loyalty -= 2; p.ral_copy = turn_stamp(g)
                log(f'  {NAME(p)} uses Ral, Storm Conduit -2: the next instant or sorcery is copied', g)
                if src.loyalty <= 0: leave(g, src); to_zone_card(g, src, 'gy')
            else:
                src.loyalty += 2
                import impl_topdeck; impl_topdeck.scry(g, p, 1)
            return True
        return go
    out = [(1.0, 'Ral, Storm Conduit +2 (scry 1)', act(False))]
    if src.loyalty >= 2 and best >= 3: out.append((1.5 + 0.5 * best, 'Ral, Storm Conduit -2 (copy)', act(True)))
    return out
note('Ral, Storm Conduit', 'Full', 'magecraft ping on each cast or copied instant/sorcery (doubled by Veyran); +2 scry 1; '
     '-2 copies your next instant/sorcery (used with a spell worth copying in hand)')


# ------------------------------------------------------------------ prepared back faces: cast a copy, paying its cost
PREPARED = {'Blazing Firesinger // Seething Song': (2, 'R', True, 'Seething Song'),
            'Emeritus of Ideation // Ancestral Recall': (0, 'U', True, 'Ancestral Recall'),
            'Sanar, Unfinished Genius // Wild Idea': (3, 'UR', False, 'Wild Idea'),
            'Emeritus of Conflict // Lightning Bolt': (0, 'R', True, 'Lightning Bolt')}
PAYOFF_TAGS = ('ping', 'spelltok', 'spelldraw', 'kiln', 'dragoncaller', 'mystic', 'aether')


def _prepared_effect(g, p, name):
    if name.startswith('Blazing'): return lambda: setattr(p, 'floatR', p.floatR + 5)
    if name.startswith('Emeritus of Ideation'): return lambda: draw(g, p, 3)
    if name.startswith('Sanar'): return lambda: tutor(g, p, 'is')
    return lambda: bolt_something(g, p, 3)


def _prepared_value(g, p, name):
    base = 1.2 * sum(1 for m in p.perms if m.cd is not None and any(k in m.cd.tags for k in PAYOFF_TAGS))
    if name.startswith('Blazing'):                 # Seething Song: {2}{R} for five red, worth it to reach a spell
        tot = total_mana(g, p)
        reach = [c for c in p.hand if not c.land and tot < sum(cost_of(p, c)[0:1]) + len(cost_of(p, c)[1]) <= tot + 2]
        return base + (3.0 if reach else 0.0) - 0.5
    if name.startswith('Emeritus of Ideation'): return base + 5.0
    if name.startswith('Sanar'): return base + 3.0
    opps = g.opps(p)
    if any(q.life <= 3 for q in opps): return 10.0
    tg = [m for q in opps for m in q.perms if m.creature and etgh(g, m) <= 3 and not untargetable(g, m)]
    return base + 0.8 * max((pval(g, m) for m in tg), default=0)


def _prepared_opt(g, src, p, s, post):
    if p is not src.owner or src.phased or not (src.data and src.data.get('prepared')): return []
    gen, pips, instant, spell = PREPARED[src.cd.name]
    if post is None and not instant and not has(p, 'gandalf'): return []      # Wild Idea is a sorcery
    if not can_pay(g, p, gen, pips): return []
    v = _prepared_value(g, p, src.cd.name)
    if v < 1.0: return []

    def go():
        if not (src.data and src.data.get('prepared')) or not can_pay(g, p, gen, pips): return False
        pay(g, p, gen, pips); src.data['prepared'] = False
        log(f'  {NAME(p)} casts a copy of {spell} ({src.cd.name.split(" //")[0]})', g)
        cast_copy(g, p, _prepared_effect(g, p, src.cd.name))
        return True
    return [(v, f'{spell} (prepared copy)', go)]


for _n in PREPARED: on(_n, 'options')(_prepared_opt)


@on('Emeritus of Ideation // Ancestral Recall', 'attack')
def _ideation_attack(g, src, p, atk, d):
    """whenever it attacks: exile eight cards from your graveyard to prepare it again"""
    if src.owner is not p or src not in atk or (src.data and src.data.get('prepared')) or len(p.gy) < 8: return
    ex = sorted(p.gy, key=lambda c: (c.instant or c.sorcery, card_worth(g, p, c, in_gy=True)))[:8]
    for c in ex: p.gy.remove(c); p.exile.append(c)
    src.data = dict(src.data or {}, prepared=True)
    log(f'    {NAME(p)} exiles eight cards: Emeritus of Ideation is prepared', g)


full('Blazing Firesinger // Seething Song', 'enters prepared: a Seething Song copy ({2}{R}: five red) is cast when the '
     'mana reaches a spell or with magecraft payoffs out')
full('Emeritus of Ideation // Ancestral Recall', '5/5 flying, ward {2}; enters prepared ({U}: draw three); attacking '
     'exiles eight graveyard cards to prepare it again')
full('Sanar, Unfinished Genius // Wild Idea', 'enters prepared (a Wild Idea copy for {3}{U}{R} finds an instant or '
     'sorcery); taps for a Treasure once you have cast an instant or sorcery that turn')
full('Emeritus of Conflict // Lightning Bolt', 'first strike; prepared by your third spell each turn: a Lightning Bolt '
     'copy for {R} (lethal, or the best X/3 creature)')


# ------------------------------------------------------------------ card selection
def _top(p, n):
    return [p.library.pop() for _ in range(min(n, len(p.library)))]


def _desire(g, p, c):
    import impl_topdeck
    return impl_topdeck.desire(g, p, c)


@on('Expressive Iteration', 'resolve')
def _iteration(g, p, c, ctx):
    """top three: one to hand, one to the bottom, one exiled (playable this turn)"""
    top = sorted(_top(p, 3), key=lambda x: -_desire(g, p, x))
    if not top: return
    p.hand.append(top[0])
    if len(top) >= 2: p.hand.append(top[1]); p.impulse.append(top[1])     # exiled, may play it this turn
    if len(top) >= 3: p.library.insert(0, top[2])


@on('Flow State', 'resolve')
def _flow(g, p, c, ctx):
    """top three: one to hand (two with an instant and a sorcery in the graveyard), the rest on the bottom"""
    k = 2 if (any(x.instant for x in p.gy) and any(x.sorcery for x in p.gy)) else 1
    top = sorted(_top(p, 3), key=lambda x: -_desire(g, p, x))
    p.hand.extend(top[:k]); p.library[:0] = top[k:]


@on('Stock Up', 'resolve')
def _stock(g, p, c, ctx):
    """top five: two to hand, the rest on the bottom"""
    top = sorted(_top(p, 5), key=lambda x: -_desire(g, p, x))
    p.hand.extend(top[:2]); p.library[:0] = top[2:]


@on('Visions of Beyond', 'resolve')
def _visions(g, p, c, ctx):
    draw(g, p, 3 if any(len(q.gy) >= 20 for q in g.players) else 1)
card('Visions of Beyond', 'draw=1', types='I', dsl=[])


@on('Banishing Betrayal', 'resolve')
def _betrayal(g, p, c, ctx):
    t = ctx.get('target')
    if t is not None and t in t.owner.perms: apply_removal(g, p, t, 'bounce', c)
    import impl_topdeck; impl_topdeck.scry(g, p, 1, to='gy')


@on('Prismari Charm', 'resolve')
def _charm(g, p, c, ctx):
    """one mode: 1 damage to each of one or two targets (X/1 creatures, a player at 1), bounce an opposing commander or
    big token, or surveil 2 then draw"""
    opps = g.opps(p)
    x1 = sorted([m for q in opps for m in q.perms if m.creature and etgh(g, m) <= 1 and pval(g, m) >= 2
                 and not untargetable(g, m)], key=lambda m: -pval(g, m))[:2]
    faces = [q for q in opps if q.life <= 1]
    ping_v = sum(pval(g, m) for m in x1) + 10.0 * len(faces)
    bn = [m for q in opps for m in q.perms if not untargetable(g, m) and not m.phased and (m.is_cmd or m.token)]
    b = max(bn, key=lambda m: pval(g, m) * (1.5 if m.is_cmd else 1.0)) if bn else None
    bounce_v = pval(g, b) * (1.5 if b.is_cmd else 1.0) if b is not None else 0
    if ping_v >= max(4.0, bounce_v):
        for q in faces[:2]: lose_life(g, q, 1, p, kind='burn')
        for m in x1[:2 - min(2, len(faces))]:
            if m in m.owner.perms: apply_removal(g, p, m, 'dmg1', c)
    elif bounce_v >= 6.0:
        apply_removal(g, p, b, 'bounce', c)
    else:
        import impl_topdeck; impl_topdeck.scry(g, p, 2, to='gy'); draw(g, p, 1)


full('Expressive Iteration', 'top three: the best to hand, the next exiled and playable this turn, the last to the bottom')
full('Flow State', 'top three: one to hand (two with an instant and a sorcery in the graveyard), the rest to the bottom')
full('Stock Up', 'top five: the best two to hand, the rest to the bottom')
full('Visions of Beyond', 'draws three when any graveyard has twenty or more cards, else one')
full('Banishing Betrayal', 'bounces a nonland permanent, then surveil 1')
full('Prismari Charm', 'the best of its three modes: ping one or two X/1s (or a player at 1), bounce an opposing '
     'commander or token, or surveil 2 and draw')


# ------------------------------------------------------------------ Reenact the Crime
def gy_this_turn(g):
    """(card, owner) for every card put into a graveyard this turn (after each turn's start snapshot)"""
    import collections
    out = []
    for q in g.players:
        left = collections.Counter(getattr(q, 'gy_start', None) or {})
        for x in q.gy:
            if left[id(x)] > 0: left[id(x)] -= 1
            else: out.append((x, q))
    return out


def reenact_target(g, p, exclude=None):
    cands = [(x, q) for x, q in gy_this_turn(g) if not x.land and x is not exclude and 'crackle' not in x.tags]
    return max(cands, key=lambda z: card_worth(g, p, z[0])) if cands else None


@on('Reenact the Crime', 'resolve')
def _reenact(g, p, c, ctx):
    """exile a nonland card put into a graveyard this turn; cast a copy of it for free"""
    t = reenact_target(g, p, exclude=c)
    if t is None: return                                     # no legal target: the spell does nothing
    x, q = t
    q.gy.remove(x); q.exile.append(x)
    log(f'    Reenact the Crime copies {x.name}', g)
    copy_spell(g, p, x, cast=True)
full('Reenact the Crime', 'exiles the best nonland card put into a graveyard this turn and casts a copy for free '
     '(cast only when there is one)')


# ------------------------------------------------------------------ Return the Favor (spree)
@on('Return the Favor', 'hand_cast')
def _rtf_copy(g, c, p, spell):
    """+{1}: copy target instant or sorcery spell: your own big spell"""
    if getattr(g, 'in_rtf', False) or c not in p.hand or not (spell.instant or spell.sorcery) or spell is c: return
    if spell_copy_value(g, p, spell) < 5.0 or not can_pay(g, p, 1, 'RR'): return
    g.in_rtf = True
    try:
        p.hand.remove(c); pay(g, p, 1, 'RR')
        p.spells_this_turn += 1; p.cast_names.add(c.name)
        on_cast(g, p, c)
        log(f'  {NAME(p)} casts Return the Favor: copy {spell.name}', g)
        if counter_window(g, p, c, 5, {}):
            cc = getattr(g, 'cur_cast', None)
            copy_spell(g, p, spell, cc[1] if cc is not None and cc[0] is spell else None)
        p.gy.append(c)
    finally:
        g.in_rtf = False


def rtf_redirect(g, owner, m, kind, actor, spell):
    """+{1}: change the target of a removal spell aimed at your permanent to one of the caster's"""
    if spell is None or actor is None or actor is owner or kind in ('edict', 'wipe'): return False
    rtf = [c for c in owner.hand if c.name == 'Return the Favor']
    if not rtf or not can_pay(g, owner, 1, 'RR') or pval(g, m) < 4: return False
    alt = [x for x in actor.perms if not x.phased and not untargetable(g, x) and (x.creature or kind != 'dmg')]
    if not alt: return False
    t = max(alt, key=lambda x: pval(g, x))
    c = rtf[0]
    owner.hand.remove(c); pay(g, owner, 1, 'RR')
    owner.spells_this_turn += 1; owner.cast_names.add(c.name)
    on_cast(g, owner, c)
    owner.gy.append(c)
    log(f'    {NAME(owner)} casts Return the Favor: {spell.name} now targets {t.name}', g)
    apply_removal(g, actor, t, kind, spell)
    return True
full('Return the Favor', 'spree: copies your own big instant/sorcery ({1}{R}{R}), or turns a removal spell aimed at '
     'your permanent onto one of the caster\'s ({1}{R}{R})')


# ------------------------------------------------------------------ Old Fat Spider Can't See Me: chapter II
def damage_prevented(g, m):
    """chapter II: all damage a creature would deal is prevented while the Saga remains"""
    for q in g.players:
        for x in q.perms:
            if x.cd is not None and 'spider' in x.cd.tags and x.data and x.data.get('prevent') is m: return True
    return False


def spider_chapter2(g, p, saga):
    cr = [m for q in g.opps(p) for m in q.perms if m.creature and not m.phased and not untargetable(g, m)]
    if cr:
        t = max(cr, key=lambda m: epow(g, m))
        saga.data = dict(saga.data or {}, prevent=t)
        log(f'    Old Fat Spider: all damage {t.name} would deal is prevented', g)
note("Old Fat Spider Can't See Me", 'Full', 'I: your best creature has hexproof while it remains; II: all damage the '
     'biggest opposing creature would deal is prevented while it remains; III-IV: draw')


# ------------------------------------------------------------------ the rest of Veyran's list
note('Crackle with Power', 'Full', 'X from spare mana (5X to each of X targets): lethal players first, then big '
     'creatures, then the lowest life')
note('Burst Lightning', 'Full', '2 damage, or 4 kicked ({4} more) when that kills a better target or a player')
note('Eris, Roar of the Storm', 'Full', 'costs {2} less per mana value among your graveyard instants/sorceries; flying, '
     'prowess; second spell each turn makes a 4/4 flying prowess Dragon (two with Veyran)')
note('Harmonic Prodigy', 'Full', 'prowess; Shaman and Wizard triggers trigger an additional time')
note('Veyran, Voice of Duality', 'Full', 'magecraft +1/+1 (doubled by itself); instant/sorcery triggers of your '
     'permanents trigger an additional time')
note('Gandalf, Friend of the Shire', 'Full', 'flash; sorceries at instant speed; the Ring trigger never happens (nothing '
     'in this list tempts)')
note('Jin-Gitaxias, Progress Tyrant', 'Full', 'copies your first artifact/instant/sorcery each turn (a whole copy with '
     'new targets; artifacts as tokens); counters an opponent\'s first artifact/instant/sorcery each turn')
note("Mizzix's Mastery", 'Full', 'one instant/sorcery copied and cast free ({3}{R}), or overloaded ({5}{R}{R}{R}) '
     'for every instant/sorcery in the graveyard; the cards are exiled')
note('Hydro-Channeler', 'Full', 'taps for {U} for instant and sorcery spells only (its {1}: any colour filter adds '
     'no mana and this list makes both colours)')


# ------------------------------------------------------------------ Veyran's utility lands (end of an opponent's turn)
@on('Desolate Lighthouse', 'land_options')
def _lighthouse(g, L, p, s, post):
    """{1}{U}{R}, {T}: draw a card, then discard a card (with mana left over at the end of an opponent's turn)"""
    import impl_lands as IL
    if post is not None or L.tapped or g.active is p or not IL.can_pay_without(g, p, L, 1, 'UR'): return []

    def go():
        if L not in p.lands or L.tapped or not IL.pay_without(g, p, L, 1, 'UR'): return False
        L.tapped = True; draw(g, p, 1); discard_worst(g, p, 1)
        log(f'  {NAME(p)} loots with Desolate Lighthouse', g); return True
    return [(1.2, 'Desolate Lighthouse loot', go)]


@on('Spectacle Summit', 'land_options')
def _summit(g, L, p, s, post):
    """{2}{U}{R}, {T}: surveil 1 (with mana left over at the end of an opponent's turn)"""
    import impl_lands as IL
    if post is not None or L.tapped or g.active is p or not IL.can_pay_without(g, p, L, 2, 'UR'): return []

    def go():
        if L not in p.lands or L.tapped or not IL.pay_without(g, p, L, 2, 'UR'): return False
        L.tapped = True
        import impl_topdeck; impl_topdeck.scry(g, p, 1, to='gy'); return True
    return [(0.6, 'Spectacle Summit surveil', go)]


full('Desolate Lighthouse', '{T}: {C}; {1}{U}{R}, {T}: loot at the end of an opponent\'s turn with mana to spare')
full('Spectacle Summit', 'enters tapped; {T}: {U} or {R}; {2}{U}{R}, {T}: surveil 1 at the end of an opponent\'s turn')
