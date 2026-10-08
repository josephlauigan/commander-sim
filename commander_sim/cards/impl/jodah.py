"""Jodah, the Unifier (JD's WUBRG Legends deck, key `jodah`, decklists/JD/jodah-wubrg-legends.md).

- Jodah: legendary creatures you control get +X/+X (X = the legendary creatures you control); whenever you cast a
  legendary spell from your hand, exile from the top until a legendary nonland card with lesser mana value, cast it
  free, the rest to the bottom in a random order.
- The AI: the outside decks' generic priorities, with legendary spells first while Jodah is out (the more expensive,
  the more the cascade can find).
"""
import importlib
from commander_sim.engine import *
from commander_sim import engine as E
from commander_sim.cards import cardimpl as CI
from commander_sim.cards.cardimpl import on
from commander_sim.cards.pool_cards import card, note
from commander_sim.cards.impl import common as IC

JODAH = 'Jodah, the Unifier'


def full(name, text): note(name, 'Full', text)


def human(g, p):
    return E.human_choice(g, p)


def legendary(g, m):
    """a legendary permanent: printed, a legendary token (Kaldra, Voja), or the Ring-bearer"""
    if m.cd is None: return bool(m.data and m.data.get('legendary'))
    return importlib.import_module('commander_sim.cards.impl.mine').is_legendary(g, m)


def legend_card(c):
    return 'leg' in c.tags and not c.land


def jodahs(p):
    return sum(1 for m in p.perms if m.cd is not None and m.cd.name == JODAH and not m.phased)


# ------------------------------------------------------------------ Jodah: the anthem
def _jodah_pt(g, m):
    p = m.owner
    k = jodahs(p)
    if not k or not legendary(g, m): return 0, 0
    x = k * sum(1 for y in p.perms if y.creature and not y.phased and legendary(g, y))
    return x, x
IC.CREATURE_PT.append(_jodah_pt)


@on(JODAH, 'etb')
def _jodah_etb(g, src, p, m):
    if m is src: g.selfpt = True                     # the engine reads legends' size from CREATURE_PT from here on


# ------------------------------------------------------------------ Jodah: the legend cascade
@on(JODAH, 'cast')
def _jodah_cast(g, src, caster, c):
    o = src.owner
    if caster is not o or not legend_card(c): return
    cur = getattr(g, 'cur_cast', None)
    if cur is None or cur[0] is not c or cur[2] != 'hand': return          # only spells cast from your hand
    if not trigger_window(g, o, src, f'cascade into a legend with mana value below {c.cmc}', imp=4): return
    jodah_cascade(g, o, c.cmc)


def jodah_cascade(g, o, mv):
    """exile from the top until a legendary nonland card with mana value below mv; cast it free (you may); the rest
    go to the bottom in a random order"""
    seen, hit = [], None
    while o.library:
        x = o.library.pop()
        if legend_card(x) and x.cmc < mv:
            hit = x; break
        seen.append(x)
    cast = False
    if hit is not None:
        hc = human(g, o)
        want = hc.yes_no(g, o, f'Jodah: cast {hit.name} without paying its mana cost?') if hc is not None else True
        if want and castable(g, o, hit, 'lib'):
            g.last_x = 0                                  # cast free: X is 0
            o.milestone.setdefault('jodah', o.turns)
            o.stats['jodah_cascades'] += 1
            log(f'    Jodah reveals {hit.name} ({len(seen)} other cards): cast free', g)
            cast_card(g, o, hit, 'lib', {})
            cast = True
        else:
            log(f'    Jodah reveals {hit.name} ({len(seen)} other cards): not cast', g)
            seen.append(hit)
    else:                                             # nothing cheaper: the whole library is exiled, then goes back
        log(f'    Jodah exiles {len(seen)} cards and finds no legendary nonland card with mana value below {mv}: '
            f'they all go to the bottom in a random order (the library is shuffled)', g)
    g.rng.shuffle(seen)
    o.library[:0] = seen                              # to the bottom, random order
    return cast


full(JODAH, 'legends you control get +X/+X (X = your legendary creatures); casting a legendary spell from hand cascades '
     'into a legendary nonland card with lesser mana value, cast free')


# ================================================================== the AI
def jodah_prio(g, p, c):
    """the generic priorities; with Jodah out, legendary spells first, the dearer the better (more to cascade into)"""
    from commander_sim.ai import pool_ai
    v = pool_ai.generic_prio(g, p, c) or 0
    if c is p.cmd: return max(v, 86)
    if not v and c.dsl and E.DSLMOD is not None: v = int(E.DSLMOD.card_value(g, p, c) * 10)
    if legend_card(c) and jodahs(p):
        v = max(v, 40) + 4 + min(16, 2 * c.cmc)
    elif legend_card(c) and c.creature:
        v = max(v, 35)
    return min(90, v)


CI.jodah_prio = jodah_prio


# ================================================================== the 99: cards that need code
def _attached(g, m, name):
    return [e for e in m.owner.perms if e.cd is not None and e.cd.name == name and e.attached is m and not e.phased]


def _mine_named(p, name):
    return [m for m in p.perms if m.cd is not None and m.cd.name == name and not m.phased]


def _best_legend(g, p, exclude=()):
    cs = [m for m in p.perms if m.creature and not m.phased and legendary(g, m) and m not in exclude]
    return max(cs, key=lambda m: (not m.sick, pval(g, m)), default=None)


# ------------------------------------------------------------------ Kaldra
card('Sword of Kaldra', 'leg', types='A', dsl=[{'type': 'static', 'static': 'equip_bonus', 'pow': 5, 'tgh': 5},
                                              {'type': 'static', 'static': 'equip_cost', 'mana': 4}])
card('Shield of Kaldra', 'leg', types='A', dsl=[{'type': 'static', 'static': 'equip_keyword', 'keyword': 'indestructible'},
                                               {'type': 'static', 'static': 'equip_cost', 'mana': 4}])
card('Helm of Kaldra', 'leg', types='A', dsl=[{'type': 'static', 'static': 'equip_keyword', 'keyword': 'first strike'},
                                             {'type': 'static', 'static': 'equip_keyword', 'keyword': 'trample'},
                                             {'type': 'static', 'static': 'equip_keyword', 'keyword': 'haste'},
                                             {'type': 'static', 'static': 'equip_cost', 'mana': 2}])
KALDRA = ('Sword of Kaldra', 'Shield of Kaldra', 'Helm of Kaldra')


def kaldra_exile(g, src, victim):
    """Sword of Kaldra: the equipped creature dealt damage to victim: exile it. True if it was exiled"""
    if src.cd is None and not (src.data and src.data.get('kaldra')) and not _attached(g, src, 'Sword of Kaldra'): return False
    if not _attached(g, src, 'Sword of Kaldra'): return False
    if victim not in victim.owner.perms or protected_from(g, victim, ''): return False
    log(f'    Sword of Kaldra exiles {victim.name}', g)
    victim.owner.lost_names[victim.name] += 1
    exile_perm(g, victim)
    return True
CI.kaldra_exile = kaldra_exile


def _kaldra_indestructible(g, m):
    """Shield of Kaldra: the Kaldra equipment are indestructible"""
    return m.cd is not None and m.cd.name in KALDRA and bool(_mine_named(m.owner, 'Shield of Kaldra'))
for _n in KALDRA: CI.SELF_REGEN[_n] = lambda g, m: _kaldra_indestructible(g, m)


@on('Helm of Kaldra', 'options')
def _kaldra_assemble(g, src, p, s, post):
    """{1}: with Helm, Sword and Shield of Kaldra all out, create Kaldra (a legendary 4/4 Avatar) wearing all three"""
    if p is not src.owner or post is None or not can_pay(g, p, 1, ''): return []
    pieces = [next(iter(_mine_named(p, n)), None) for n in KALDRA]
    if None in pieces or any(m.data and m.data.get('kaldra') for m in p.perms if m.cd is None): return []

    def go():
        if not can_pay(g, p, 1, '') or any(x not in p.perms for x in pieces): return False
        pay(g, p, 1, '')
        if not ability_window(g, p, src, 'create Kaldra'): return True
        k = make_tokens(g, p, 1, 4, 4, color='', types=('avatar',), data={'legendary': True, 'kaldra': True})
        if k:
            for e in pieces: e.attached = k[0]
            k[0].data['indestr'] = True
            log(f'  {NAME(p)} creates Kaldra wearing Sword, Shield and Helm', g)
        return True
    return [(9.0, 'Helm of Kaldra: create Kaldra', go)]


for _n, _t in (('Sword of Kaldra', 'equipped creature +5/+5; exiles any creature it deals damage to; equip {4}'),
               ('Shield of Kaldra', 'equipped creature and the Kaldra equipment are indestructible; equip {4}'),
               ('Helm of Kaldra', 'first strike, trample and haste; equip {2}; {1}: with all three Kaldra pieces, '
                                  'create Kaldra (legendary 4/4) wearing them')):
    full(_n, _t)


# ------------------------------------------------------------------ Blackblade Reforged
def _blackblade_pt(g, m):
    k = len(_attached(g, m, 'Blackblade Reforged'))
    return (k * len(m.owner.lands),) * 2 if k else (0, 0)
IC.CREATURE_PT.append(_blackblade_pt)
card('Blackblade Reforged', 'leg', types='A', dsl=[])


@on('Blackblade Reforged', 'etb')
def _blackblade_etb(g, src, p, m):
    if m is src: g.selfpt = True


@on('Blackblade Reforged', 'options')
def _blackblade_equip(g, src, p, s, post):
    """equip a legendary creature {3}, any other {7}"""
    if p is not src.owner or post is None or g.active is not p: return []
    if src.attached is not None and src.attached in p.perms: return []
    t = _best_legend(g, p)
    n = 3
    if t is None:
        cs = [m for m in p.perms if m.creature and not m.phased and not m.noatk]
        t = max(cs, key=lambda m: pval(g, m), default=None); n = 7
    if t is None or not can_pay(g, p, n, ''): return []

    def go():
        if not can_pay(g, p, n, '') or t not in p.perms: return False
        return equip_to(g, p, src, t, n)
    return [(1.5 + 0.25 * len(p.lands) - 0.2 * n, f'equip Blackblade Reforged to {t.name}', go)]
full('Blackblade Reforged', 'equipped creature +1/+1 per land you control; equip a legend {3}, anything else {7}')


# ------------------------------------------------------------------ planeswalkers
def _surveil(g, p, n):
    importlib.import_module('commander_sim.cards.impl.topdeck').scry(g, p, n, 'gy')


def _dakkon_exile(g, p, src):
    t = IC.best_opp_creature(g, p)
    if t is not None: apply_removal(g, p, t, 'exile')


def _dakkon_ult(g, p, src):
    arts = [c for c in p.hand + p.gy if 'A' in c.types and not c.land]
    if not arts: return
    c = max(arts, key=lambda c: (c.cmc, card_worth(g, p, c)))
    (p.hand if c in p.hand else p.gy).remove(c)
    enter(g, p, c)


IC.walker('Dakkon, Shadow Slayer', [
    (1, 'surveil 2', IC.always(1.2), lambda g, p, src: _surveil(g, p, 2)),
    (-3, 'exile a creature', lambda g, p, src: (lambda t: pval(g, t) - 1 if t is not None and pval(g, t) >= 3 else None)(
        IC.best_opp_creature(g, p)), _dakkon_exile),
    (-6, 'an artifact onto the battlefield', lambda g, p, src: 6.0 if any('A' in c.types and c.cmc >= 5 for c in p.hand + p.gy) else None,
     _dakkon_ult),
], status=('Full', 'enters with loyalty equal to your lands; +1 surveil 2, -3 exile the best opposing creature, -6 your '
           'best artifact from hand or graveyard onto the battlefield'))
CI.AS_ENTERS['Dakkon, Shadow Slayer'] = lambda g, p, m: setattr(m, 'loyalty', len(p.lands))


def _mord_draw(g, p, src):
    draw(g, p, 2)
    if p.hand:
        x = min(p.hand, key=lambda c: card_worth(g, p, c))
        p.hand.remove(x); p.library.insert(0, x)


def _mord_dog(g, p, src):
    t = make_tokens(g, p, 1, 0, 0, color='U', types=('dog', 'illusion'), data={'mord_dog': True})


IC.TOKEN_PT.append(lambda g, m: ((2 * len(m.owner.hand),) * 2) if m.data.get('mord_dog') else (0, 0))


def _mord_ult(g, p, src):
    p.hand, p.library = p.library, p.hand
    g.rng.shuffle(p.library)
    p.no_max_hand = True


IC.walker('Mordenkainen', [
    (2, 'draw two, one to the bottom', IC.always(2.5), _mord_draw),
    (-2, 'a Dog Illusion (twice your hand)', lambda g, p, src: 0.6 * len(p.hand) if len(p.hand) >= 3 else None, _mord_dog),
    (-10, 'swap hand and library', lambda g, p, src: 9.0, _mord_ult),
], status=('Full', '+2 draw two then put one on the bottom, -2 a Dog Illusion with power and toughness twice your hand '
           'size, -10 exchange hand and library (no maximum hand size)'))


def _w6_land(g, p, src):
    ls = [c for c in p.gy if c.land]
    if ls: x = ls[0]; p.gy.remove(x); p.hand.append(x)


def _w6_ping(g, p, src):
    t = IC.best_opp_creature(g, p, lambda m: etgh(g, m) <= 1)
    if t is not None: apply_removal(g, p, t, 'dmg1')
    elif g.opps(p): lose_life(g, min(g.opps(p), key=lambda q: q.life), 1, p, kind='burn', damage=True)


IC.walker('Wrenn and Six', [
    (1, 'a land back to hand', lambda g, p, src: 1.5 if any(c.land for c in p.gy) else 0.8, _w6_land),
    (-1, '1 damage', lambda g, p, src: (lambda t: pval(g, t) if t is not None and pval(g, t) >= 1.5 else None)(
        IC.best_opp_creature(g, p, lambda m: etgh(g, m) <= 1)), _w6_ping),
    (-7, 'emblem: retrace', lambda g, p, src: 6.0, lambda g, p, src: setattr(p, 'w6_retrace', True)),
], status=('Approximate', '+1 a land from graveyard to hand, -1 one damage (an X/1 or a player), -7 emblem (retrace '
           'recorded, not used)'))


def _spark_attach(g, p, src):
    t = _best_legend(g, p) or max([m for m in p.perms if m.creature and not m.phased], key=lambda m: epow(g, m), default=None)
    if t is not None: src.attached = t; t.plus += 1


IC.walker('The Aetherspark', [
    (1, 'attach, +1/+1 counter', lambda g, p, src: 2.0 if any(m.creature for m in p.perms) else 0.5, _spark_attach),
    (-5, 'draw two', lambda g, p, src: 3.5, lambda g, p, src: draw(g, p, 2)),
    (-10, 'add ten mana', lambda g, p, src: 5.0 if any(c.cmc >= 8 for c in p.hand) else None,
     lambda g, p, src: setattr(p, 'floatA', p.floatA + 10)),
], status=('Full', '+1 attach to your best creature with a +1/+1 counter, -5 draw two, -10 add ten mana; combat damage '
           'by the equipped creature on your turn adds that much loyalty'))


@on('The Aetherspark', 'combat_damage')
def _spark_loyalty(g, src, p, a, d, dmg):
    if p is src.owner and src.attached is a and src.loyalty is not None: src.loyalty += dmg


def _carth_extra(g, src, p):
    return 1 if p is src.owner else 0
CI.HOOKS.setdefault('Carth the Lion', {})['loyalty_extra'] = _carth_extra


def _carth_look(g, o):
    top = o.library[-7:]
    pw = [c for c in top if 'P' in c.types]
    rest = [c for c in top]
    del o.library[-len(top):]
    if pw:
        c = max(pw, key=lambda c: card_worth(g, o, c)); rest.remove(c); o.hand.append(c)
        log(f'    Carth finds {c.name}', g)
    g.rng.shuffle(rest); o.library[:0] = rest


@on('Carth the Lion', 'etb')
def _carth_etb(g, src, p, m):
    if m is src and trigger_window(g, src.owner, src, 'look at seven for a planeswalker'): _carth_look(g, src.owner)


@on('Carth the Lion', 'dies')
def _carth_pw_dies(g, src, m, cause):
    if m.owner is src.owner and m.cd is not None and 'P' in m.cd.types and trigger_window(g, src.owner, src, 'look at seven'):
        _carth_look(g, src.owner)
card('Carth the Lion', 'leg pow=3 tgh=5', dsl=[])
full('Carth the Lion', 'on entry and when your planeswalker dies, a planeswalker from the top seven to hand; your '
     'loyalty abilities cost an extra [+1]')


# ------------------------------------------------------------------ creatures
def discover(g, o, n, src_name):
    """exile from the top until a nonland card with mana value n or less: cast it free (or to hand); rest to the bottom"""
    seen, hit = [], None
    while o.library:
        x = o.library.pop()
        if not x.land and x.cmc <= n: hit = x; break
        seen.append(x)
    if hit is not None:
        if castable(g, o, hit, 'lib') and not ('ctr' in hit.tags and not g.stack):
            g.last_x = 0
            log(f'    {src_name}: discover {n} casts {hit.name}', g)
            cast_card(g, o, hit, 'lib', {})
        else:
            o.hand.append(hit)
    g.rng.shuffle(seen); o.library[:0] = seen


@on('Caparocti Sunborn', 'attack')
def _caparocti(g, src, p, atk, d):
    if p is not src.owner or src not in atk: return None
    cand = sorted([m for m in p.perms if not m.tapped and not m.phased and m not in atk and m is not src
                   and (m.creature or (m.cd is not None and 'A' in m.cd.types))], key=lambda m: pval(g, m))
    if len(cand) < 2 or not trigger_window(g, p, src, 'tap two: discover 3'): return None
    for m in cand[:2]: m.tapped = True
    discover(g, p, 3, 'Caparocti Sunborn')
    return None
card('Caparocti Sunborn', 'leg pow=4 tgh=4', dsl=[])
full('Caparocti Sunborn', 'attacking: tap your two least valuable untapped artifacts or creatures to discover 3')


def _regen(cost_g, cost_p):
    def fn(g, m):
        p = m.owner
        if E.human_choice(g, p) is None and not can_pay(g, p, cost_g, cost_p): return False
        if not can_pay(g, p, cost_g, cost_p): return False
        pay(g, p, cost_g, cost_p); m.tapped = True
        log(f'    {m.name} regenerates', g)
        return True
    return fn
CI.SELF_REGEN['Cromat'] = _regen(0, 'BG')
CI.SELF_REGEN['Korlash, Heir to Blackblade'] = _regen(1, 'B')


@on('Cromat', 'crew')
def _cromat_combat(g, src, p):
    """before attacking: {U}{R} for flying when the defenders have no fliers to block, {R}{W} pumps with spare mana"""
    if p is not src.owner or src.tapped or src.sick: return
    opp_fly = any(m.fly for q in g.opps(p) for m in q.perms if m.creature and not m.tapped)
    if not src.fly and not opp_fly and can_pay(g, p, 0, 'UR') and total_mana(g, p) >= 4:
        pay(g, p, 0, 'UR'); g.eot_kw.setdefault(id(src), set()).add('flying'); g.dsl_on = True
        log('  Cromat gains flying', g)
    while total_mana(g, p) >= 4 and can_pay(g, p, 0, 'RW'):
        pay(g, p, 0, 'RW'); a, b = g.eot_pt.get(id(src), (0, 0)); g.eot_pt[id(src)] = (a + 1, b + 1); g.dsl_on = True
card('Cromat', 'leg pow=5 tgh=5', dsl=[])
note('Cromat', 'Approximate', '{B}{G} regenerate, {U}{R} flying before an attack the defenders can\'t block in the air, '
     '{R}{W} +1/+1 with spare mana; the blocker-destroy and top-of-library abilities are not used')
card('Korlash, Heir to Blackblade', 'leg pow=0 tgh=0', dsl=[])
IC.SELF_PT['Korlash, Heir to Blackblade'] = lambda g, p, m: ((sum(1 for L in p.lands if 'swamp' in
    importlib.import_module('commander_sim.ais').land_types(L.cd.name)[0]),) * 2)


CI.AS_ENTERS['Korlash, Heir to Blackblade'] = lambda g, p, m: setattr(g, 'selfpt', True)   # before its 0/0 is checked
full('Korlash, Heir to Blackblade', 'power and toughness equal to your Swamps; {1}{B} regenerate; grandeur needs a '
     'second copy (never in a singleton deck)')


@on('Dragonlord Dromoka', 'can_cast')
def _dromoka(g, src, caster, c, zone):
    return not (caster is not src.owner and g.active is src.owner)
card('Dragonlord Dromoka', 'leg pow=5 tgh=7 fly lifelink unc', dsl=[])
full('Dragonlord Dromoka', "can't be countered; flying, lifelink; opponents can't cast spells during your turn")


def _hydra_prio(g, p, c):
    x = total_mana(g, p) - 2
    return 40 + min(30, 5 * x) if x >= 3 else 0
CI.SPELL_PRIO['Genesis Hydra'] = _hydra_prio
card('Genesis Hydra', 'xtutor', types='C', dsl=[])


@on('Genesis Hydra', 'etb')
def _hydra_etb(g, src, p, m):
    """X (the spare mana it was cast with; 0 when cast free): the top X cards, a nonland permanent with mana value X or
    less onto the battlefield, the rest shuffled in; the Hydra gets X +1/+1 counters. The reveal happens as it
    resolves rather than as it is cast (only a counterspell tells the two apart)"""
    if m is not src: return
    o = src.owner
    x, g.last_x = int(getattr(g, 'last_x', 0) or 0), 0
    if x > 0:
        top = o.library[-x:]; del o.library[-x:]
        ok = [y for y in top if not y.land and y.cmc <= x and (y.perm or y.creature)]
        if ok:
            y = max(ok, key=lambda y: (y.cmc, card_worth(g, o, y))); top.remove(y)
            log(f'    Genesis Hydra puts {y.name} onto the battlefield', g)
            enter(g, o, y)
        o.library.extend(top); g.rng.shuffle(o.library)
    if src in o.perms:
        src.plus += x
        if etgh(g, src) <= 0: die(g, src, 'sba')
full('Genesis Hydra', 'X = spare mana: a nonland permanent with mana value X or less from the top X onto the '
     'battlefield, and X +1/+1 counters')


def _lagrella_etb(g, src, p, m):
    if m is not src or src not in src.owner.perms: return
    o = src.owner
    if not trigger_window(g, o, src, 'exile a creature of each opponent', imp=5) or src not in o.perms: return
    IC.oring_exile(g, src, o, lambda x: x.creature, per_opponent=True, creature_only=True)
CI.HOOKS.setdefault('Lagrella, the Magpie', {})['etb'] = _lagrella_etb
CI.HOOKS['Lagrella, the Magpie']['leaves'] = lambda g, m: IC.oring_return(g, m)
card('Lagrella, the Magpie', 'leg pow=2 tgh=3 rem=exile tgt=c', dsl=[])
note('Lagrella, the Magpie', 'Full', "exiles the best creature of each opponent until it leaves (yours aren't exiled, "
     'so the +1/+1 counters clause never applies)')


@on('Mirri, Weatherlight Duelist', 'attack_cap')
def _mirri_cap(g, src, attacker, d):
    return 1 if d is src.owner and src.tapped else None


@on('Mirri, Weatherlight Duelist', 'blocks')
def _mirri_blocks(g, src, p, atk, d, assign):
    """Mirri attacks: each opponent blocks with at most one creature"""
    if p is not src.owner or src not in atk: return
    used = {}
    for a in sorted(list(assign), key=lambda a: -epow(g, a)):
        b = assign.get(a)
        if b is None or b.owner is not d: continue
        if d in used and used[d] is not b: assign[a] = None
        else: used[d] = b
card('Mirri, Weatherlight Duelist', 'leg pow=3 tgh=2 fs', dsl=[{'type': 'static', 'static': 'self_keyword', 'keyword': 'first strike'}])
full('Mirri, Weatherlight Duelist', "first strike; attacking: each opponent blocks with at most one creature; while "
     "tapped, no more than one creature can attack you")


@on('Razia, Boros Archangel', 'options')
def _razia(g, src, p, s, post):
    return []
card('Razia, Boros Archangel', 'leg pow=6 tgh=3 fly vig haste', dsl=[])
note('Razia, Boros Archangel', 'Approximate', 'flying, vigilance, haste; the redirect-3-damage ability is not used')


@on("Sol'kanar the Swamp King", 'cast')
def _solkanar(g, src, caster, c):
    if 'B' in c.pips: gain(src.owner, 1)
card("Sol'kanar the Swamp King", 'leg pow=5 tgh=5 swampwalk', dsl=[])
full("Sol'kanar the Swamp King", 'swampwalk; you gain 1 life whenever any player casts a black spell')

card('Szadek, Lord of Secrets', 'leg pow=5 tgh=5 fly', dsl=[])
full('Szadek, Lord of Secrets', 'flying; combat damage to a player becomes +1/+1 counters and that player mills as many')


@on('Tolsimir Wolfblood', 'options')
def _tolsimir(g, src, p, s, post):
    if p is not src.owner or src.tapped or src.sick or post is None: return []
    if any(m.data and m.data.get('voja') for m in p.perms if m.cd is None): return []

    def go():
        if src.tapped: return False
        src.tapped = True
        if not ability_window(g, p, src, 'create Voja'): return True
        make_tokens(g, p, 1, 2, 2, color='GW', types=('wolf',), data={'legendary': True, 'voja': True})
        return True
    return [(2.5, 'Tolsimir: create Voja', go)]
card('Tolsimir Wolfblood', 'leg pow=3 tgh=4', dsl=[
    {'type': 'static', 'static': 'anthem', 'pow': 1, 'tgh': 1, 'filter': {'type': 'creature', 'controller': 'you', 'other': True, 'color': 'G'}},
    {'type': 'static', 'static': 'anthem', 'pow': 1, 'tgh': 1, 'filter': {'type': 'creature', 'controller': 'you', 'other': True, 'color': 'W'}}])
full('Tolsimir Wolfblood', 'your other green creatures +1/+1 and other white creatures +1/+1; {T}: Voja, a legendary '
     '2/2 Wolf (one at a time)')


@on('Urza, Powerstone Prodigy', 'options')
def _urza_loot(g, src, p, s, post):
    if p is not src.owner or src.tapped or src.sick or post is None or not can_pay(g, p, 1, '') or not p.hand: return []

    def go():
        if src.tapped or not can_pay(g, p, 1, ''): return False
        pay(g, p, 1, ''); src.tapped = True
        if not ability_window(g, p, src, 'draw, then discard'): return True
        draw(g, p, 1)
        if p.hand:
            arts = [c for c in p.hand if 'A' in c.types and card_worth(g, p, c) < 30]
            x = min(arts or p.hand, key=lambda c: card_worth(g, p, c))
            discard_cards(g, p, [x])
            if 'A' in x.types and once_per_turn(g, p, 'urza_stone'): IC.make_artifact_tokens(g, p, 'Powerstone')
        return True
    return [(1.0 if post else 0.4, 'Urza: loot', go)]
card('Urza, Powerstone Prodigy', 'leg pow=1 tgh=3 vig', dsl=[])
full('Urza, Powerstone Prodigy', 'vigilance; {1}, {T}: draw then discard; discarding an artifact makes a Powerstone '
     '(once a turn)')


@on('King Darien XLVIII', 'options')
def _darien(g, src, p, s, post):
    if p is not src.owner or post is None or g.active is not p or not can_pay(g, p, 3, 'GW'): return []

    def go():
        if not can_pay(g, p, 3, 'GW'): return False
        pay(g, p, 3, 'GW')
        if not ability_window(g, p, src, 'a counter and a Soldier'): return True
        src.plus += 1; make_tokens(g, p, 1, 1, 1, color='W', types=('soldier',))
        return True
    return [(1.2 if post else 0.3, 'King Darien: counter and a Soldier', go)]
card('King Darien XLVIII', 'leg pow=2 tgh=3', dsl=[{'type': 'static', 'static': 'anthem', 'pow': 1, 'tgh': 1,
                                                  'filter': {'type': 'creature', 'controller': 'you', 'other': True}}])
note('King Darien XLVIII', 'Approximate', 'other creatures +1/+1; {3}{G}{W} a +1/+1 counter and a 1/1 Soldier with spare '
     'mana; the sacrifice-for-protection ability is not used')


@on('Sisters of Stone Death', 'blocks')
def _sisters(g, src, p, atk, d, assign):
    """{B}{G}: exile a creature blocking the Sisters"""
    if p is not src.owner or src not in atk: return
    b = assign.get(src)
    if b is not None and b in d.perms and can_pay(g, p, 0, 'BG') and not untargetable(g, b):
        pay(g, p, 0, 'BG'); log(f'  Sisters of Stone Death exile {b.name}', g)
        d.lost_names[b.name] += 1; exile_perm(g, b); assign[src] = None
        src.data = dict(src.data or {}, sisters=(src.data or {}).get('sisters', []) + [(b.cd, d)] if b.cd is not None else [])
card('Sisters of Stone Death', 'leg pow=7 tgh=5', dsl=[])
note('Sisters of Stone Death', 'Approximate', '{B}{G} exiles a creature that blocks them; the lure and the {2}{B} '
     'reanimation of exiled creatures are not used')



# ------------------------------------------------------------------ spells
@IC.spell("Cartographer's Survey", prio=lambda g, p, c: 46 if len(p.lands) <= 6 else 20, tags='', types='S',
          status=('Full', 'up to two lands from the top seven onto the battlefield tapped; the rest to the bottom'))
def _survey(g, p, c, ctx):
    top = p.library[-7:]; del p.library[-7:]
    lands = sorted([x for x in top if x.land], key=lambda x: -len(x.tags.get('c', '')))[:2]
    for x in lands:
        top.remove(x); p.lands.append(Land(x, True)); landfall(g, p)
    g.rng.shuffle(top); p.library[:0] = top


@IC.spell('Experimental Augury', prio=36, tags='', types='I',
          status=('Full', 'the best of the top three to hand, the rest to the bottom; proliferate'))
def _augury(g, p, c, ctx):
    top = p.library[-3:]; del p.library[-3:]
    if top:
        x = max(top, key=lambda y: card_worth(g, p, y)); top.remove(x); p.hand.append(x)
    p.library[:0] = top
    importlib.import_module('commander_sim.cards.impl.mine').proliferate_all(g, p)


def _despair_one(g, p, q, kind):
    pool = [m for m in q.perms if not m.phased and (m.creature if kind == 'C' else (m.cd is not None and kind in m.cd.types))]
    if pool:
        die(g, min(pool, key=lambda m: pval(g, m)), 'sac')
    else:
        lose_life(g, q, 2, p); draw(g, p, 1)


@IC.spell('Invoke Despair', prio=lambda g, p, c: 50 if g.opps(p) else 0, tags='', types='S',
          status=('Full', 'the most threatening opponent sacrifices a creature, an enchantment and a planeswalker '
                          '(their least valuable); for each they can\'t, they lose 2 and you draw'))
def _invoke(g, p, c, ctx):
    q = ctx.get('target') or (max(g.opps(p), key=lambda o: threat(g, p, o)) if g.opps(p) else None)
    if q is None or not q.alive: return
    for k in ('C', 'E', 'P'): _despair_one(g, p, q, k)
    check_state(g)


@on("Kaervek's Purge", 'hand_options')
def _purge(g, c, p, s, post):
    """{X}{B}{R}: destroy target creature with mana value X; it deals its power to its controller"""
    if post is None or g.active is not p or c not in p.hand: return []
    tg = [m for m in legal_targets(g, p, 'destroy', 'c', spell=c) if m.cd is not None and can_pay(g, p, m.cd.cmc, 'BR')]
    if not tg: return []
    t = max(tg, key=lambda m: pval(g, m) + 0.2 * epow(g, m))
    if pval(g, t) < 3: return []

    def go():
        x = t.cd.cmc
        if c not in p.hand or not can_pay(g, p, x, 'BR') or t not in t.owner.perms: return False
        pay(g, p, x, 'BR'); p.hand.remove(c); p.spells_this_turn += 1; on_cast(g, p, c)
        ok = counter_window(g, p, c, 4, {})
        p.gy.append(c)
        if ok and t in t.owner.perms and not untargetable(g, t):
            q, power = t.owner, epow(g, t)
            apply_removal(g, p, t, 'destroy', c)
            if t not in q.perms and power: lose_life(g, q, power, p, kind='burn', damage=True)
        return True
    return [(pval(g, t) * 0.5 - 0.8, f"Kaervek's Purge on {t.name}", go)]
card("Kaervek's Purge", '', types='S', dsl=[])
full("Kaervek's Purge", 'X = the target\'s mana value: destroys it and deals its power to its controller')


@on('Profane Tutor', 'hand_options')
def _profane(g, c, p, s, post):
    """suspend 2 for {1}{B}: two upkeeps later, a free Demonic Tutor"""
    if post is None or g.active is not p or c not in p.hand or not can_pay(g, p, 1, 'B'): return []

    def go():
        if c not in p.hand or not can_pay(g, p, 1, 'B'): return False
        pay(g, p, 1, 'B'); p.hand.remove(c); p.exile.append(c)
        p.suspended = getattr(p, 'suspended', []) + [[c, 2]]
        log(f'  {NAME(p)} suspends Profane Tutor', g)
        return True
    return [(2.2, 'suspend Profane Tutor', go)]


def suspend_upkeep(g, p):
    """the beginning of p's upkeep: a time counter off each suspended card; at zero it's cast free"""
    for e in list(getattr(p, 'suspended', []) or []):
        c = e[0]
        if c not in p.exile: p.suspended.remove(e); continue
        e[1] -= 1
        if e[1] > 0: continue
        p.suspended.remove(e); p.exile.remove(c)
        log(f'  {NAME(p)} casts {c.name} from suspend', g)
        cast_card(g, p, c, 'lib', {})
CI.suspend_upkeep = suspend_upkeep


@IC.spell('Profane Tutor', tags='tut=any', types='S',
          status=('Full', 'suspend 2 for {1}{B}, then cast free: search for any card'))
def _profane_resolve(g, p, c, ctx):
    tutor(g, p, 'any')
CI.SPELL_PRIO['Profane Tutor'] = 0                    # never hard-cast (no mana cost): suspended from hand

card('Dissipate', 'ctr=any ctrexile', types='I', dsl=[])
full('Dissipate', 'counter target spell; it is exiled instead of going to the graveyard')
card('Desertion', 'ctr=any', types='I', dsl=[])
full('Desertion', 'counter target spell; a countered artifact or creature spell enters under your control')


# ------------------------------------------------------------------ Mirari, Memory Jar, Court of Ardenvale
@on('Mirari', 'cast')
def _mirari(g, src, caster, c):
    """whenever you cast an instant or sorcery: pay {3} to copy it (the AI copies spells worth a second resolution)"""
    o = src.owner
    if caster is not o or not (c.instant or c.sorcery) or not can_pay(g, o, 3, ''): return
    t = c.tags
    worth = any(k in t for k in ('draw', 'rem', 'tut', 'drawcre')) or c.name in (
        'Fact or Fiction', 'Invoke Despair', 'Demonic Tutor', 'Cartographer\'s Survey', 'Experimental Augury')
    if not worth or 'ctr' in t: return
    hc = human(g, o)
    if hc is not None and not hc.yes_no(g, o, f'Mirari: pay {{3}} to copy {c.name}?'): return
    if not trigger_window(g, o, src, f'pay {{3}}: copy {c.name}'): return
    pay(g, o, 3, '')
    copy_spell(g, o, c)
card('Mirari', 'leg', types='A', dsl=[])
CI.SPELL_PRIO['Mirari'] = lambda g, p, c: 44 if sum(1 for x in p.hand if x.instant or x.sorcery) >= 2 else 26
full('Mirari', 'whenever you cast an instant or sorcery, pay {3} to copy it (draw, removal and tutors)')


@on('Memory Jar', 'options')
def _jar(g, src, p, s, post):
    """{T}, sacrifice: everyone sets their hand aside and draws seven; at the end step they discard and get the old
    hands back. Used with a small hand and mana left to spend the seven"""
    if p is not src.owner or post is not False or g.active is not p or len(p.hand) > 2 or total_mana(g, p) < 3: return []

    def go():
        if src not in p.perms: return False
        leave(g, src); p.gy.append(src.cd)
        if not ability_window(g, p, src.cd, 'wheel for seven'): return True
        due = []
        for q in g.players:
            if not q.alive: continue
            held = list(q.hand); q.hand.clear(); q.exile.extend(held)
            due.append((q, held))
            draw(g, q, 7)
        g.jar_due = (getattr(g, 'jar_due', None) or []) + due
        log(f'  {NAME(p)} cracks Memory Jar', g)
        return True
    return [(3.0 + 0.4 * (7 - len(p.hand)), 'Memory Jar', go)]


def jar_end(g):
    due, g.jar_due = g.jar_due, []
    for q, held in due:
        if not q.alive: continue
        discard_cards(g, q, list(q.hand))
        for c in held:
            if c in q.exile: q.exile.remove(c); q.hand.append(c)
CI.jar_end = jar_end
card('Memory Jar', '', types='A', dsl=[])
CI.SPELL_PRIO['Memory Jar'] = 38
full('Memory Jar', '{T}, sacrifice: every player sets their hand aside and draws seven; at the end step they discard '
     'and take the old hands back (cracked with a small hand and mana to use the seven)')


@on('Court of Ardenvale', 'etb')
def _court_etb(g, src, p, m):
    if m is src and trigger_window(g, src.owner, src, 'become the monarch'): CI.become_monarch(g, src.owner)


@on('Court of Ardenvale', 'upkeep')
def _court(g, src, p):
    o = src.owner
    if p is not o: return
    cs = [c for c in o.gy if (c.perm or c.creature) and not c.land and c.cmc <= 3]
    if not cs or not trigger_window(g, o, src, 'return a permanent card'): return
    c = max(cs, key=lambda c: card_worth(g, o, c))
    if c not in o.gy: return
    o.gy.remove(c)
    if getattr(g, 'monarch', None) is o:
        enter(g, o, c); log(f'    Court of Ardenvale returns {c.name} to the battlefield', g)
    else:
        o.hand.append(c); log(f'    Court of Ardenvale returns {c.name} to hand', g)
card('Court of Ardenvale', '', types='E', dsl=[])
CI.SPELL_PRIO['Court of Ardenvale'] = 50
full('Court of Ardenvale', 'you become the monarch; each upkeep, a permanent card with mana value 3 or less from your '
     'graveyard to hand, or onto the battlefield while you are the monarch')


# ------------------------------------------------------------------ mana
card('Fyndhorn Elder', 'pow=1 tgh=1 dork=G', dsl=[])
CI.DYN_MANA['Fyndhorn Elder'] = lambda g, p, m: 2
full('Fyndhorn Elder', '{T}: {G}{G}')
card("Sisay's Ring", 'rock=2:C', types='A', dsl=[])
full("Sisay's Ring", '{T}: {C}{C}')
