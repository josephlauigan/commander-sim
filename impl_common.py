"""Hand-written implementations shared by many pool decks (see cardimpl.py for the hook events)."""
from engine import *
import engine as E
from cardimpl import on
import cardimpl as CI
from cardimpl import _eot
from pool_cards import card, note


# ======================================================== Auras
AURA = {}          # name -> spec


def aura(name, pow=0, tgh=0, kws=(), bonus=None, umbra=False, prot='', target='own', on_etb=None, host_ok=None,
         back_to_hand=None, status=('Full', ''), tags=''):
    """register an Aura: static bonus / keywords / protection for the enchanted creature, umbra armor, an ETB
    effect, where it goes when it leaves (back_to_hand: 'always' like Rancor, 'host_dies' like Angelic Destiny)"""
    AURA[name] = dict(pow=pow, tgh=tgh, kws=frozenset(kws), bonus=bonus, umbra=umbra, prot=prot, target=target,
                      on_etb=on_etb, host_ok=host_ok, back=back_to_hand)
    CI.HOOKS.setdefault(name, {})['etb'] = _aura_etb
    card(name, tags or 'aura', types='E', dsl=[])
    note(name, *status)


def own_host(g, p, spec, aura_perm=None):
    """the creature an Aura should go on: the commander in a voltron deck, else the best creature"""
    cands = [m for m in p.perms if m.creature and not m.phased and m is not aura_perm and m.cd is not None or
             (m.creature and m.token and not m.phased)]
    if spec['host_ok']: cands = [m for m in cands if spec['host_ok'](g, p, m)]
    if not cands: return None
    import pool_ai
    pref = pool_ai.config(p).get('aura_host')
    for m in cands:
        if pref == 'commander' and m.is_cmd: return m
    return max(cands, key=lambda m: (m.is_cmd * 2 + pval(g, m) + (3 if m.fly else 0)))


def _aura_etb(g, src, p, m):
    if m is not src: return
    spec = AURA[src.cd.name]
    host = getattr(g, 'attach_to', None)
    if host is None or host not in host.owner.perms:
        host = own_host(g, src.owner, spec, src)
    if host is None:
        leave(g, src); src.owner.gy.append(src.cd); return
    src.attached = host
    if not hasattr(g, 'auras') or g.auras is None: g.auras = []
    g.auras.append(src)
    g.dsl_on = True
    log(f'    {src.cd.name} enchants {host.name}', g)
    if spec['on_etb']: spec['on_etb'](g, src.owner, src, host)


def auras_on(g, m):
    return [a for a in g.auras if a.attached is m and a in a.owner.perms and not a.phased]


def attached_bonus(g, m):
    dp = dt = 0
    if getattr(g, 'auras', None):
        for a in auras_on(g, m):
            spec = AURA[a.cd.name]
            dp += spec['pow']; dt += spec['tgh']
            if spec['bonus']:
                x, y = spec['bonus'](g, a.owner, a, m); dp += x; dt += y
    if m.creature and getattr(m.owner, 'elspeth_emblem', False): dp += 2; dt += 2
    if getattr(g, 'selfpt', None) and m.cd is not None and m.cd.name in SELF_PT and m.owner.alive:
        x, y = SELF_PT[m.cd.name](g, m.owner, m); dp += x; dt += y
    return dp, dt


SELF_PT = {}         # creature name -> fn(g, p, m) -> (dp, dt): characteristic-defining bonuses (Kor Spiritdancer ...)


def attached_kw(g, m, kw):
    for a in auras_on(g, m):
        if kw in AURA[a.cd.name]['kws']: return True
    return False


def attached_prot(g, m):
    return ''.join(AURA[a.cd.name]['prot'] for a in auras_on(g, m))


def umbra_save(g, m):
    for a in auras_on(g, m):
        if AURA[a.cd.name]['umbra']:
            log(f'    {a.cd.name} is destroyed instead of {m.name} (umbra armor)', g)
            die(g, a, 'destroy')
            return True
    return False


def aura_fall(g, m):
    """m left the battlefield: Auras attached to it go to the graveyard (or back to hand)"""
    for a in list(g.auras):
        if a is m:
            g.auras.remove(a); continue
        if a.attached is m:
            back = AURA[a.cd.name]['back']
            died = m not in m.owner.perms
            g.auras.remove(a)
            leave(g, a)
            if back == 'always' or (back == 'host_dies' and died): a.owner.hand.append(a.cd)
            else: to_zone_card(g, a, 'gy')


CI.attached_bonus = attached_bonus
CI.attached_kw = attached_kw
CI.attached_prot = attached_prot
CI.umbra_save = umbra_save
CI.aura_fall = aura_fall
CI.auras_on = auras_on


def n_ench(p): return sum(1 for m in p.perms if m.cd is not None and 'E' in m.cd.types and not m.phased)
def n_auras(p): return sum(1 for m in p.perms if m.cd is not None and 'aura' in m.cd.subtypes and m.attached is not None)


def _sheltered_etb(g, p, a, host):
    oring_exile(g, a, p, lambda m: m.cd is not None or m.token)


def _cartouche_etb(g, p, a, host):
    make_tokens(g, p, 1, 1, color='W', types=('warrior',))


def _reverie_etb(g, p, a, host): draw(g, p, n_auras(p))
def _draw1(g, p, a, host): draw(g, p, 1)


aura('All That Glitters', bonus=lambda g, p, a, m: (k := sum(1 for x in p.perms if x.cd is not None and
                                                        ('A' in x.cd.types or 'E' in x.cd.types)), k))
aura('Angelic Destiny', 4, 4, ('flying', 'first strike'), back_to_hand='host_dies')
aura('Battle Mastery', kws=('double strike',))
aura('Cartouche of Solidarity', 1, 1, ('first strike',), on_etb=_cartouche_etb)
aura('Celestial Mantle', 3, 3, status=('Partial', '+3/+3; the life-doubling combat trigger is modeled in a hook'))
aura('Daybreak Coronet', 3, 3, ('first strike', 'vigilance', 'lifelink'),
     host_ok=lambda g, p, m: any(a.attached is m for a in getattr(g, 'auras', None) or []))
aura('Ethereal Armor', kws=('first strike',), bonus=lambda g, p, a, m: (n_ench(p), n_ench(p)))
aura('Felidar Umbra', kws=('lifelink',), umbra=True, status=('Approximate', 'lifelink, umbra armor; moving it is not modeled'))
aura('Flickering Ward', prot='B', status=('Approximate', 'protection from black (the most common removal colour) '
                                                        'and the bounce-and-recast Light-Paws loop'))
aura("Gryff's Boon", 1, 0, ('flying',), status=('Partial', '+1/+0 and flying; graveyard recursion not modeled'))
aura('Hyena Umbra', 1, 1, ('first strike',), umbra=True)
aura('Mammoth Umbra', 3, 3, ('vigilance',), umbra=True)
aura("On Serra's Wings", 1, 1, ('flying', 'vigilance', 'lifelink'))
aura("Sage's Reverie", bonus=lambda g, p, a, m: (n_auras(p), n_auras(p)), on_etb=_reverie_etb)
aura("Sentinel's Eyes", 1, 1, ('vigilance',), status=('Partial', '+1/+1 vigilance; escape not modeled'))
aura('Sheltered by Ghosts', 1, 0, ('lifelink',), on_etb=_sheltered_etb,
     status=('Approximate', 'O-Ring effect on entry; +1/+0 and lifelink; its ward is not modeled'))
aura('Shielded by Faith', kws=('indestructible',), status=('Approximate', 'indestructible; moving it is not modeled'))
aura('Spectra Ward', 2, 2, prot='WUBRG')
aura('Spirit Link', status=('Approximate', 'gains life equal to the enchanted creature\'s combat damage (hook)'))
aura('Spirit Mantle', 1, 1, ('unblockable',), status=('Approximate', 'protection from creatures read as unblockable'))
aura('Timely Ward', kws=('indestructible',), status=('Approximate', 'indestructible; flash on a commander not modeled'))
aura('Unquestioned Authority', kws=('unblockable',), on_etb=_draw1,
     status=('Approximate', 'draw on entry; protection from creatures read as unblockable'))
aura('Ancestral Mask', bonus=lambda g, p, a, m: (2 * (sum(n_ench(q) for q in g.players if q.alive) - 1),) * 2)
aura('Rancor', 2, 0, ('trample',), back_to_hand='always')
aura('Snake Umbra', 1, 1, umbra=True, status=('Full', '+1/+1, draw on damage to an opponent (hook), umbra armor'))
aura('Spider Umbra', 1, 1, ('reach',), umbra=True)
aura('Empyrial Armor', bonus=lambda g, p, a, m: (len(p.hand), len(p.hand)))


def _host_damage(name):
    def deco(fn):
        CI.HOOKS.setdefault(name, {})['combat_damage'] = fn
        return fn
    return deco


@_host_damage('Celestial Mantle')
def _mantle(g, src, p, a, d, dmg):
    if src.attached is a: gain(src.owner, src.owner.life)


@_host_damage('Spirit Link')
def _spirit_link(g, src, p, a, d, dmg):
    if src.attached is a: gain(src.owner, dmg)


@_host_damage('Snake Umbra')
def _snake(g, src, p, a, d, dmg):
    if src.attached is a: draw(g, src.owner, 1)


# hostile Auras: removal on entry
card('Darksteel Mutation', 'rem=exile tgt=c etb', types='E', dsl=[])
note('Darksteel Mutation', 'Approximate', 'the creature is neutralised: modeled as exile')
card('Song of the Dryads', 'rem=exile tgt=nl etb', types='E', dsl=[])
note('Song of the Dryads', 'Approximate', 'the permanent becomes a Forest: modeled as exile')
card("Kenrith's Transformation", 'rem=exile tgt=c etb draw=1', types='E', dsl=[])
note("Kenrith's Transformation", 'Approximate', 'draw a card; the creature becomes a vanilla 3/3: modeled as exile')
# land Auras: extra mana, modeled as a mana rock
for _n, _t in (('Utopia Sprawl', 'rock=1:A'), ('Wild Growth', 'rock=1:G'), ('Fertile Ground', 'rock=1:A'),
               ('Overgrowth', 'rock=2:G')):
    card(_n, _t, types='E', dsl=[])
    note(_n, 'Approximate', 'extra mana modeled as a separate mana source (untaps with the land in practice)')


# ======================================================== exile-until-it-leaves (Oblivion Ring family)
def oring_exile(g, src, p, pred, per_opponent=False, creature_only=False):
    """exile the best permanent(s) opponents control until src leaves the battlefield"""
    targets = []
    for q in (g.opps(p) if per_opponent else [None]):
        cands = [m for o in ([q] if q else g.opps(p)) for m in o.perms
                 if not m.phased and not untargetable(g, m) and pred(m) and (m.creature or not creature_only)
                 and not protected_from(g, m, src.cd.pips)]
        if cands:
            best = max(cands, key=lambda m: pval(g, m))
            if pval(g, best) >= 1: targets.append(best)
    exiled = []
    for m in targets:
        owner = m.owner
        apply_removal(g, p, m, 'exile', src.cd)
        if m not in owner.perms and not m.token:
            cd = m.phys or m.cd
            if cd is not owner.cmd and cd in m.orig.exile: exiled.append((cd, m.orig))
    if src.data is None: src.data = {}
    src.data['oring'] = src.data.get('oring', []) + exiled


def oring_return(g, src):
    for cd, owner in (src.data or {}).get('oring', []):
        if cd in owner.exile and owner.alive:
            owner.exile.remove(cd); enter(g, owner, cd)
            log(f'    {cd.name} returns to the battlefield', g)


def _oring(name, pred, per_opponent=False, creature_only=False, tags='rem=exile tgt=nl', status=('Full', '')):
    def etb(g, src, p, m):
        if m is src: oring_exile(g, src, src.owner, pred, per_opponent, creature_only)
    CI.HOOKS.setdefault(name, {})['etb'] = etb
    CI.HOOKS[name]['leaves'] = oring_return
    card(name, tags, types='E', dsl=[])
    note(name, *status)


_any = lambda m: True
_oring('Oblivion Ring', _any)
_oring('Banishing Light', _any)
_oring('Cast Out', _any, tags='rem=exile tgt=nl flash', status=('Full', 'cycling not modeled'))
_oring('Journey to Nowhere', _any, creature_only=True, tags='rem=exile tgt=c')
_oring('Detention Sphere', _any, status=('Approximate', 'exiles the one target (other copies of the same name stay)'))
_oring('Grasp of Fate', _any, per_opponent=True)


# ======================================================== Esper Sentinel
@CI.on('Esper Sentinel', 'cast')
def _sentinel(g, src, caster, c):
    o = src.owner
    if caster is o or c.creature or c.land: return
    if casts_this_turn(g, caster, lambda x: not x.creature) != 1: return
    x = epow(g, src)
    if x and can_pay(g, caster, x, '') and total_mana(g, caster) >= x + 2:
        pay(g, caster, x, ''); return
    draw(g, o, 1)
card('Esper Sentinel', 'human pow=1')
note('Esper Sentinel', 'Approximate', 'opponents pay X only with 2 mana to spare after it; otherwise you draw')


# ======================================================== cost reducers
def _reducer(name, pred, amount=-1, status=('Full', '')):
    CI.HOOKS.setdefault(name, {})['cost'] = lambda g, src, caster, c: amount if caster is src.owner and pred(c) else 0
    note(name, *status)


_reducer('Danitha Capashen, Paragon', lambda c: 'aura' in c.subtypes or 'equipment' in c.subtypes)
_reducer('Hero of Iroas', lambda c: 'aura' in c.subtypes, status=('Partial', 'Aura discount; heroic not modeled'))
_reducer('Starfield Mystic', lambda c: 'E' in c.types, status=('Partial', 'enchantment discount; +1/+1 counters not modeled'))
_reducer('Pearl Medallion', lambda c: 'W' in c.pips)


# ======================================================== sacrifice outlets and death payoffs (aristocrats)
def _bombard(g, p, src, m):
    opps = g.opps(p)
    if not opps: return
    lethal = [q for q in opps if q.life <= 1]
    lose_life(g, lethal[0] if lethal else min(opps, key=lambda q: q.life), 1, p, kind='triggers')


def _feeder(g, p, src, m): src.plus += 1
def _ashnod(g, p, src, m): p.floatA += 2
def _altar(g, p, src, m): p.floatA += 1


# free sacrifice outlets: name -> effect when used (None: nothing but the death itself)
SAC_OUTLET = {'Viscera Seer': None, 'Carrion Feeder': _feeder, 'Woe Strider': None, 'Cartel Aristocrat': None,
              "Ashnod's Altar": _ashnod, 'Phyrexian Altar': _altar, 'Goblin Bombardment': _bombard,
              'Yahenni, Undying Partisan': None, 'Spawning Pit': None, 'Carrion Feeder ': None}
# death payoffs: name -> value per creature death of yours (drains count once per opponent)
DEATH_DRAIN = {'Blood Artist': 1, 'Zulaport Cutthroat': 1, 'Cruel Celebrant': 1, 'Bastion of Remembrance': 1,
               'Falkenrath Noble': 1, 'Vindictive Vampire': 1, 'Syr Konrad, the Grim': 1, 'Poison-Tip Archer': 1,
               'Elas il-Kor, Sadistic Pilgrim': 1, 'Mayhem Devil': 0.5, 'Goblin Bombardment': 0.3,
               'Mirkwood Bats': 0.3, 'Grave Pact': 1.5, 'Dictate of Erebos': 1.5, 'Butcher of Malakir': 1.5}
DEATH_DRAW = {'Grim Haruspex': 1, 'Midnight Reaper': 1, 'Morbid Opportunist': 0.5, 'Skemfar Avenger': 1,
              'Dark Prophecy': 1, 'Pitiless Plunderer': 0.6, 'Korvold, Fae-Cursed King': 1}


def death_value(g, p, m=None):
    """what one creature death is worth to p right now (drains, cards, Treasures), before copies"""
    names = [x.cd.name for x in p.perms if x.cd is not None and not x.phased]
    tags = sum(1 for x in p.perms if x.cd is not None and ('bartist' in x.cd.tags or 'drain' in x.cd.tags))
    drain = sum(DEATH_DRAIN.get(n, 0) for n in names) + tags
    draw_ = sum(DEATH_DRAW.get(n, 0) for n in names)
    copies = 1 + (CI.total(g, 'trigger_copies', p, 'dies', m) if g.hooks else 0)
    return copies * (drain * len(g.opps(p)) * 0.8 + draw_ * 1.5)


def outlets(p):
    return [x for x in p.perms if x.cd is not None and x.cd.name in SAC_OUTLET and not x.phased and not stopped(E.CUR_G, x.cd.name)]


def sac_through(g, p, src, m):
    """sacrifice m through outlet src"""
    fx = SAC_OUTLET[src.cd.name]
    log(f'  {NAME(p)} sacrifices {m.name} to {src.cd.name}', g)
    die(g, m, 'sac')
    if fx: fx(g, p, src, m)


def aristocrat_options(g, p, s, post):
    """sacrifice fodder when the death triggers are worth more than the creature, or are lethal"""
    outs = outlets(p)
    if not outs: return []
    src = outs[0]
    fod = [m for m in p.perms if m.creature and not m.phased and m is not src and not m.is_cmd
           and m.cd is not None and m.cd.name not in SAC_OUTLET or (m.creature and m.token and not m.phased)]
    if not fod: return []
    per = death_value(g, p)
    drain_per = sum(DEATH_DRAIN.get(x.cd.name, 0) for x in p.perms if x.cd is not None) + \
        sum(1 for x in p.perms if x.cd is not None and ('bartist' in x.cd.tags or 'drain' in x.cd.tags))
    drain_per *= 1 + (CI.total(g, 'trigger_copies', p, 'dies', None) if g.hooks else 0)
    lethal = drain_per and all(q.life <= drain_per * len(fod) for q in g.opps(p))
    m = min(fod, key=lambda x: pval(g, x))
    gain_ = per - 1.5 * pval(g, m) - (0.8 if post is False and not m.sick and not m.tapped else 0)
    if not lethal and gain_ < 1.0: return []

    def go(src=src, m=m):
        if src not in p.perms or m not in p.perms: return False
        sac_through(g, p, src, m); return True
    return [(9.0 if lethal else 1.0 + gain_, f'sacrifice {m.name} ({src.cd.name})', go)]


def sac_in_response(g, p, m, kind):
    """a creature of p's is about to be exiled / bounced / stolen: sacrifice it for value instead"""
    if not m.creature or kind not in ('exile', 'bounce', 'tuck') or m.is_cmd: return False
    outs = [o for o in outlets(p) if o is not m]
    if not outs: return False
    sac_through(g, p, outs[0], m)
    return True


for _n in ('Cartel Aristocrat', 'Woe Strider', 'Viscera Seer', 'Carrion Feeder'):
    note(_n, 'Approximate', 'free sacrifice outlet used by the aristocrat AI (scry / protection side effects ignored)')
for _n in ("Ashnod's Altar", 'Phyrexian Altar'):
    note(_n, 'Approximate', 'sacrifice outlet: mana added as any colour, usable this turn')
note('Goblin Bombardment', 'Full', 'sacrifice a creature: 1 damage to the weakest opponent (or a lethal one)')
card('Goblin Bombardment', '', types='E', dsl=[])
card('Phyrexian Altar', '', types='A', dsl=[])
card('Woe Strider', 'pow=3 tgh=2', dsl=[{'type': 'triggered', 'event': 'etb', 'source': 'self',
                                         'effects': [{'do': 'token', 'n': 1, 'pow': 0, 'tgh': 1, 'keywords': [], 'types': ['goat']}]}])


# ======================================================== simple death-trigger creatures (shared by several decks)
def _dies_other(name, fn, nontoken=False, other=True, yours=True, status=('Full', '')):
    @CI.on(name, 'dies')
    def _h(g, src, m, cause):
        if not m.creature or (other and m is src) or (yours and m.owner is not src.owner) or (nontoken and m.token): return
        fn(g, src.owner, src, m)
    card(name, None, dsl=[])
    note(name, *status)


def _drain1(g, p, src, m):
    for q in g.opps(p): lose_life(g, q, 1, p, kind='drain')
    gain(p, len(g.opps(p)))


_dies_other('Cruel Celebrant', _drain1, other=False)
_dies_other('Grim Haruspex', lambda g, p, s, m: draw(g, p, 1), nontoken=True)
_dies_other('Midnight Reaper', lambda g, p, s, m: (lose_life(g, p, 1, p), draw(g, p, 1)), nontoken=True, other=False)
_dies_other('Sifter of Skulls', lambda g, p, s, m: make_tokens(g, p, 1, 1, color='', types=('eldrazi', 'scion')),
            nontoken=True, status=('Approximate', 'Scion token; its sacrifice-for-mana is not modeled'))
_dies_other('Pawn of Ulamog', lambda g, p, s, m: make_tokens(g, p, 1, 0, 1, color='', types=('eldrazi', 'spawn')),
            nontoken=True, other=False, status=('Approximate', 'Spawn token; its sacrifice-for-mana is not modeled'))
_dies_other('Requiem Angel', lambda g, p, s, m: None if has_type(m, 'spirit') else
            make_tokens(g, p, 1, 1, fly=True, color='W', types=('spirit',)))
_dies_other('Dark Prophecy', lambda g, p, s, m: (draw(g, p, 1), lose_life(g, p, 1, p)), other=False)
_dies_other('Vindictive Vampire', lambda g, p, s, m: (_dmg_each(g, p, 1), gain(p, 1)))
_dies_other('Syr Konrad, the Grim', lambda g, p, s, m: _dmg_each(g, p, 1), yours=False, other=True,
            status=('Approximate', 'any other creature dying deals 1 to each opponent (cards leaving graveyards ignored)'))


def _dmg_each(g, p, n):
    for q in g.opps(p): lose_life(g, q, n, p, kind='triggers')


@CI.on('Morbid Opportunist', 'dies')
def _morbid(g, src, m, cause):
    if m.creature and m is not src and once_per_turn(g, src.owner, f'morbid{id(src)}'): draw(g, src.owner, 1)
card('Morbid Opportunist', 'human pow=1 tgh=3', dsl=[])
note('Morbid Opportunist', 'Full', '')


@CI.on('Carrier Thrall', 'self_dies')
def _thrall(g, m, cause): make_tokens(g, m.owner, 1, 1, color='', types=('eldrazi', 'scion'))
card('Carrier Thrall', 'pow=2 tgh=1', dsl=[])
note('Carrier Thrall', 'Approximate', 'Scion token; its sacrifice-for-mana is not modeled')


@CI.on('Mirkwood Bats', 'sacrifice')
def _bats_sac(g, src, p, what):
    if p is src.owner and (what in ('Treasure', 'Food', 'Clue') or getattr(what, 'token', False)): _drain_only(g, p)


def _drain_only(g, p):
    for q in g.opps(p): lose_life(g, q, 1, p, kind='drain')


card('Mirkwood Bats', 'pow=2 tgh=3 fly', dsl=[])
note('Mirkwood Bats', 'Approximate', 'drains on token sacrifices (Treasure / Food / Clue included); token creation not counted')


@CI.on('Revel in Riches', 'dies')
def _revel(g, src, m, cause):
    if m.creature and m.owner is not src.owner: src.owner.treasures += 1


@CI.on('Revel in Riches', 'upkeep')
def _revel_win(g, src, p):
    if p is src.owner and p.treasures >= 10:
        import ais; ais.win(g, p, 'Revel in Riches')
card('Revel in Riches', '', types='E', dsl=[])
note('Revel in Riches', 'Full', '')


@CI.on('Priest of Forgotten Gods', 'options')
def _priest(g, src, p, s, post):
    if src.tapped or src.sick: return []
    fod = sorted([m for m in p.perms if m.creature and m is not src and not m.is_cmd and (m.token or pval(g, m) < 2.5)],
                 key=lambda m: pval(g, m))
    if len(fod) < 2 or not g.opps(p): return []

    def go():
        f2 = sorted([m for m in p.perms if m.creature and m is not src and not m.is_cmd and (m.token or pval(g, m) < 2.5)],
                    key=lambda m: pval(g, m))[:2]
        if len(f2) < 2 or src.tapped: return False
        src.tapped = True
        for m in f2: die(g, m, 'sac')
        for q in g.opps(p): lose_life(g, q, 2, p, kind='drain'); edict(g, q)
        p.floatA += 2; draw(g, p, 1); check_state(g); return True
    return [(3.5 + 0.5 * len(g.opps(p)), 'Priest of Forgotten Gods', go)]
card('Priest of Forgotten Gods', 'human pow=1 tgh=2', dsl=[])
note('Priest of Forgotten Gods', 'Full', 'sacrifices two spare creatures: each opponent loses 2 and sacrifices, BB, draw')


# ======================================================== staples shared by many decks
def spell(name, prio=None, status=('Full', ''), tags=None, types=None):
    """register a hand-written instant / sorcery: @spell(name)(resolve_fn)"""
    def deco(fn):
        CI.HOOKS.setdefault(name, {})['resolve'] = fn
        if prio is not None: CI.SPELL_PRIO[name] = prio
        if tags is not None or types is not None: card(name, tags or '', types=types, dsl=[])
        note(name, *status)
        return fn
    return deco


@spell('Brainstorm', prio=lambda g, p, c: 40 if len(p.library) > 10 else 0, tags='', types='I',
       status=('Approximate', 'draw three, put back the two least useful cards'))
def _brainstorm(g, p, c, ctx):
    draw(g, p, 3)
    for _ in range(2):
        rest = [x for x in p.hand if x is not c]
        if not rest: break
        lands = sum(1 for x in rest if x.land)
        x = min(rest, key=lambda x: (card_worth(g, p, x) if not x.land else (25 if lands <= 2 and len(p.lands) < 6 else 4)))
        p.hand.remove(x); p.library.append(x)


@spell('Fact or Fiction', prio=45, tags='draw=2', types='I',
       status=('Approximate', 'the opponent\'s split is modeled as: you keep the best two of five'))
def _fof(g, p, c, ctx):
    top = [p.library.pop() for _ in range(min(5, len(p.library)))]
    top.sort(key=lambda x: -card_worth(g, p, x))
    for x in top[:2]: p.hand.append(x); p.seen_names.add(x.name)
    p.gy.extend(top[2:])


@spell("Council's Judgment", tags='rem=exile tgt=nl', types='S',
       status=('Approximate', 'exiles the best opposing nonland permanent (it doesn\'t target: hexproof ignored); '
                              'the vote is not modeled'))
def _judgment(g, p, c, ctx):
    cands = [m for q in g.opps(p) for m in q.perms if not m.phased]
    if cands:
        m = max(cands, key=lambda m: pval(g, m))
        owner = m.owner; log(f'    {m.name} ({NAME(owner)}) is exiled by vote', g)
        owner.lost_names[m.name] += 1; exile_perm(g, m); check_state(g)


@spell('Sign in Blood', prio=45, tags='draw=2 lose=2', types='S', status=('Full', 'you draw two and lose 2'))
def _sib(g, p, c, ctx):
    draw(g, p, 2); lose_life(g, p, 2, p)


# ---- Mind Stone: {1}, {T}, sacrifice: draw
@CI.on('Mind Stone', 'options')
def _mind_stone(g, src, p, s, post):
    if post is not True or src.tapped or len(p.lands) < 6 or not can_pay(g, p, 1, ''): return []

    def go():
        if src.tapped or src not in p.perms: return False
        src.tapped = True
        if not can_pay(g, p, 1, ''): src.tapped = False; return False
        pay(g, p, 1, ''); die(g, src, 'sac'); draw(g, p, 1); return True
    return [(1.0, 'crack Mind Stone', go)]
note('Mind Stone', 'Full', 'taps for {C}; cracked for a card late (six or more lands)')


# ---- Otawara, Soaring City: channel {3}{U} (less per legendary creature), discard: bounce
@CI.on('Otawara, Soaring City', 'hand_options')
def _otawara(g, c, p, s, post):
    n = max(0, 3 - sum(1 for m in p.perms if m.creature and m.cd is not None and 'leg' in m.cd.tags))
    if not can_pay(g, p, n, 'U'): return []
    cands = [m for q in g.opps(p) for m in q.perms if not untargetable(g, m) and not m.phased and
             (m.creature or (m.cd is not None and any(x in m.cd.types for x in 'AEP')))]
    if not cands: return []
    t = max(cands, key=lambda m: pval(g, m))
    if pval(g, t) < 4: return []

    def go():
        if c not in p.hand or t not in t.owner.perms or not can_pay(g, p, n, 'U'): return False
        pay(g, p, n, 'U'); p.hand.remove(c); p.gy.append(c)
        log(f'  {NAME(p)} channels Otawara', g); apply_removal(g, p, t, 'bounce'); return True
    return [(pval(g, t) - 4.0, f'Otawara -> {t.name}', go)]
note('Otawara, Soaring City', 'Full', 'land; channelled from hand to bounce a real threat')


# ---- adventures: cast the spell half, then the creature later
ADVENTURE = {}


def adventure(name, spell_cost, kind, tgt, status):
    """kind/tgt: removal type of the adventure half; the creature half is the card's normal cast"""
    ADVENTURE[name] = (spell_cost, kind, tgt)
    note(name, *status)


adventure('Brazen Borrower', (1, 'U'), 'bounce', 'nl', ('Approximate', 'Petty Theft bounces a threat, then the '
                                                                       'Faerie is castable (as from exile)'))


def adventure_options(g, p, s, post):
    o = []
    for c in p.hand:
        if c.name not in ADVENTURE or id(c) in getattr(p, 'adv_done', set()): continue
        (gen, pips), kind, tgt = ADVENTURE[c.name]
        if not can_pay(g, p, gen, pips) or not castable(g, p, c): continue
        tg = legal_targets(g, p, kind, tgt)
        if not tg: continue
        t = max(tg, key=lambda m: pval(g, m))
        if pval(g, t) < 4: continue

        def go(c=c, t=t, gen=gen, pips=pips, kind=kind):
            if c not in p.hand or t not in t.owner.perms or not can_pay(g, p, gen, pips): return False
            pay(g, p, gen, pips)
            p.spells_this_turn += 1; p.stats['spells_cast'] += 1; on_cast(g, p, c)
            log(f'  {NAME(p)} casts the adventure of {c.name} -> {t.name}', g)
            apply_removal(g, p, t, kind)
            if not hasattr(p, 'adv_done'): p.adv_done = set()
            p.adv_done.add(id(c)); return True
        o.append((pval(g, t) - 3.5, f'{c.name} adventure -> {t.name}', go))
    return o


# ---- Reflector Mage: bounce and the owner can't recast it until your next turn
@CI.on('Reflector Mage', 'etb')
def _reflector(g, src, p, m):
    if m is not src: return
    cands = [x for q in g.opps(src.owner) for x in q.perms if x.creature and not untargetable(g, x)]
    if not cands: return
    t = max(cands, key=lambda x: pval(g, x))
    owner, name = t.owner, t.name
    apply_removal(g, src.owner, t, 'bounce', src.cd)
    if t not in owner.perms: owner.locked_name = (name, src.owner.turns + 1, src.owner)


@CI.on('Reflector Mage', 'can_cast')
def _reflector_lock(g, src, caster, c, zone):
    ln = getattr(caster, 'locked_name', None)
    if ln and ln[0] == c.name and ln[2].turns < ln[1]: return False
    return True
card('Reflector Mage', 'human wizard pow=2 tgh=3', dsl=[])
note('Reflector Mage', 'Full', '')


@CI.on('Spark Double', 'etb')
def _spark(g, src, p, m):
    if m is not src or (src.data or {}).get('copied'): return
    o = src.owner
    cands = [x for x in o.perms if x is not src and x.cd is not None and (x.creature or 'P' in x.cd.types) and not x.token]
    if not cands: return
    best = max(cands, key=lambda x: pval(g, x))
    leave(g, src)
    n = enter(g, o, best.cd); n.data = {'copied': True}; n.phys = src.cd
    if n.creature: n.plus += 1
    if n.loyalty is not None: n.loyalty += 1
    log(f'    Spark Double copies {best.cd.name}', g)
card('Spark Double', 'pow=0 tgh=0', dsl=[])
note('Spark Double', 'Approximate', 'enters as a copy of your best creature or planeswalker (+1 counter); goes to '
     'the graveyard as Spark Double')


@CI.on('Frost Titan', 'etb')
def _frost(g, src, p, m):
    if m is src: _frost_tap(g, src)


@CI.on('Frost Titan', 'attack')
def _frost_atk(g, src, p, atk, d):
    if src in atk: _frost_tap(g, src)


def _frost_tap(g, src):
    cands = [x for q in g.opps(src.owner) for x in q.perms if x.creature and not x.phased and not untargetable(g, x)]
    if cands:
        t = max(cands, key=lambda x: pval(g, x)); t.tapped = True
        if t.cd is not None: t.data = dict(t.data or {}, frozen=t.owner.turns + 1)
card('Frost Titan', 'pow=6 bomb=6', dsl=[], ward=2)
note('Frost Titan', 'Approximate', 'taps the best opposing creature on entry and attack (the no-untap is not '
     'enforced); the targeting tax is ward {2}')


@CI.on('Dream Trawler', 'attack')
def _trawler(g, src, p, atk, d):
    if src in atk: draw(g, p, 1)


@CI.on('Dream Trawler', 'draw')
def _trawler_draw(g, src, p):
    if p is src.owner: _eot(g, src, 1, 0)
card('Dream Trawler', 'pow=3 tgh=5 fly lifelink bomb=5', dsl=[])
note('Dream Trawler', 'Approximate', 'draws on attack, +1/+0 per draw; the discard-for-hexproof is used by the '
     'protection AI')


# ---- rebound (Ephemerate)
def _ephemerate_rebound(g, p, c):
    import impl_t2
    cands = [m for m in p.perms if m.creature and impl_t2.blink_value(g, p, m) > 0]
    if cands: impl_t2.blink(g, p, max(cands, key=lambda m: impl_t2.blink_value(g, p, m)))
    p.exile.remove(c); p.gy.append(c)


CI.HOOKS.setdefault('Ephemerate', {})['rebound'] = _ephemerate_rebound


# ======================================================== graveyard hate
def gy_worth(g, owner, q):
    """how much it's worth to owner to exile q's graveyard (reanimation targets, flashback, escape, recursion)"""
    if q is owner: return -1
    v = 0.0
    for c in q.gy:
        if c.creature: v += max(0, (c.bomb or c.pow) - 3) * 1.2
        if 'fb' in c.tags or c.name in ('Uro, Titan of Nature\'s Wrath', 'Life from the Loam', 'Bloodghast', 'Gravecrawler'): v += 2
    if any(m.cd is not None and m.cd.name in ('Meren of Clan Nel Toth', 'Sheoldred, Whispering One', 'Syr Konrad, the Grim')
           for m in q.perms): v += 3
    if q.key == 'seph': v += 3
    if any('rean' in c.tags for c in q.hand) or any(x.tags.get('rean') for x in q.gy): v += 3
    return v


def exile_gy(g, q, by=None):
    if not q.gy: return
    q.exile.extend(q.gy); q.gy = []
    log(f'    {NAME(q)}\'s graveyard is exiled' + (f' by {by}' if by else ''), g)


def _bog(g, p, L):
    q = max(g.opps(p), key=lambda q: gy_worth(g, p, q), default=None)
    if q is not None and gy_worth(g, p, q) > 0: exile_gy(g, q, 'Bojuka Bog')


CI.LAND_ETB['Bojuka Bog'] = _bog
note('Bojuka Bog', 'Full', 'enters tapped; exiles the most dangerous graveyard')


@CI.on('Rest in Peace', 'etb')
def _rip(g, src, p, m):
    if m is src:
        for q in g.players:
            if q.alive: exile_gy(g, q, 'Rest in Peace')


@CI.on('Rest in Peace', 'sba')
def _rip_sweep(g, src):
    for q in g.players:
        if q.alive and q.gy: q.exile.extend(q.gy); q.gy = []


@CI.on('Rest in Peace', 'no_graveyard')
def _rip_nogy(g, src, p): return 1


card('Rest in Peace', '', types='E', dsl=[])
note('Rest in Peace', 'Approximate', 'graveyards are exiled on entry and then kept empty (cards are exiled as soon as '
     'state-based checks run; dies triggers still happen); undying/persist stop')


@CI.on('Dauthi Voidwalker', 'sba')
def _dauthi_sweep(g, src):
    o = src.owner
    for q in g.opps(o):
        if q.gy:
            src.data = src.data or {}
            src.data.setdefault('void', []).extend((c, q) for c in q.gy)
            q.exile.extend(q.gy); q.gy = []


@CI.on('Dauthi Voidwalker', 'options')
def _dauthi_play(g, src, p, s, post):
    if post is None or src.tapped or src.sick: return []
    void = [(c, q) for c, q in (src.data or {}).get('void', []) if c in q.exile and not c.land]
    if not void: return []
    c, q = max(void, key=lambda x: card_worth(g, p, x[0]) + x[0].cmc * 5)
    if c.cmc < 4: return []

    def go():
        if src not in p.perms or c not in q.exile: return False
        die(g, src, 'sac')
        if c not in q.exile: return True
        q.exile.remove(c)
        log(f'  Dauthi Voidwalker: {NAME(p)} plays {c.name} free', g)
        if c.perm: enter(g, p, c, orig=q)
        else: p.hand.append(c); cast_card(g, p, c, 'hand', {})
        return True
    return [(2.0 + c.cmc * 0.6, f'Dauthi Voidwalker plays {c.name}', go)]
card('Dauthi Voidwalker', 'pow=3 tgh=2', dsl=[], kws={'unblockable_shadow'})
note('Dauthi Voidwalker', 'Approximate', 'shadow read as unblockable and unable to block; opponents\' cards going to '
     'the graveyard are exiled with void counters; sacrifice to play the best one free')


def _gy_hate_card(name, cost, when_used, status):
    """cost: (generic, pips) to activate; exiles the reanimator's graveyard in response"""
    @CI.on(name, 'gy_hate')
    def _h(g, src, reanimator, gy_owner):
        if src.tapped or not can_pay(g, src.owner, *cost): return False
        pay(g, src.owner, *cost)
        if when_used == 'sac': die(g, src, 'sac')
        else: src.tapped = True
        exile_gy(g, gy_owner, name); return True

    @CI.on(name, 'options')
    def _proactive(g, src, p, s, post):
        if src.tapped or not can_pay(g, p, *cost): return []
        q = max(g.opps(p), key=lambda q: gy_worth(g, p, q), default=None)
        if q is None or gy_worth(g, p, q) < 8: return []

        def go():
            if src not in p.perms or not can_pay(g, p, *cost): return False
            pay(g, p, *cost)
            if when_used == 'sac': die(g, src, 'sac')
            else: src.tapped = True
            exile_gy(g, q, name); return True
        return [(1.0 + gy_worth(g, p, q) / 4.0, f'{name} on {NAME(q)}', go)]
    note(name, *status)


_gy_hate_card("Tormod's Crypt", (0, ''), 'sac', ('Full', 'exiles a graveyard in response to reanimation, or '
                                                           'proactively when it holds real threats'))
_gy_hate_card('Soul-Guide Lantern', (0, ''), 'sac', ('Approximate', 'exiles a graveyard (in response or proactively); '
                                                                    'the entry exile and draw modes are not used'))
card("Tormod's Crypt", '', types='A', dsl=[])
card('Soul-Guide Lantern', '', types='A', dsl=[])


# ======================================================== pillowfort: attack taxes and caps
def _tax(name, amount, status=('Full', ''), types=None, tags=None):
    @CI.on(name, 'attack_tax')
    def _t(g, src, attacker, d):
        if d is not src.owner or attacker is src.owner: return 0
        return amount(g, src) if callable(amount) else amount
    if types is not None: card(name, tags or '', types=types, dsl=[])
    note(name, *status)


_tax('Ghostly Prison', 2, types='E')
_tax('Propaganda', 2, types='E')
_tax('Windborn Muse', 2, tags='pow=2 tgh=3 fly', types='C')
_tax('Baird, Steward of Argive', 1, tags='leg human pow=2 tgh=4 vig', types='C')
_tax('Sphere of Safety', lambda g, src: sum(1 for m in src.owner.perms if m.cd is not None and 'E' in m.cd.types), types='E')
_tax("Norn's Annex", 1, types='A', status=('Approximate', '{W/P} per attacker read as {1}'))
_tax('Archangel of Tithes', lambda g, src: 0 if src.tapped else 1, tags='pow=3 tgh=5 fly', types='C',
     status=('Approximate', 'attack tax while untapped; the blocking tax while it attacks is ignored'))
_tax('Elephant Grass', 2, types='E', status=('Approximate', 'attack tax 2 (black creatures pay it too); its '
                                                            'cumulative upkeep is paid while it matters'))


@CI.on('Crawlspace', 'attack_cap')
def _crawl(g, src, attacker, d):
    return 2 if d is src.owner else None
card('Crawlspace', '', types='A', dsl=[])
note('Crawlspace', 'Full', '')


@CI.on('Silent Arbiter', 'attack_cap')
def _arbiter(g, src, attacker, d): return 1
card('Silent Arbiter', 'pow=1 tgh=5', dsl=[])
note('Silent Arbiter', 'Approximate', 'one attacker per combat; the one-blocker limit is not modeled')


@CI.on('Elephant Grass', 'upkeep')
def _grass(g, src, p):
    if p is not src.owner: return
    src.data = src.data or {}; age = src.data.get('age', 0) + 1; src.data['age'] = age
    if age <= 3 and can_pay(g, p, age, ''): pay(g, p, age, '')
    else: die(g, src, 'sac')


# ======================================================== shroud / hexproof grants, uncounterable spells
def _grant(name, pred, kw, status=('Full', '')):
    @CI.on(name, 'grant_kw')
    def _g(g, src, m, k):
        return k == kw and m.owner is src.owner and m is not src and pred(g, src, m)
    note(name, *status)


_grant('Greater Auramancy', lambda g, s, m: m.cd is not None and 'E' in m.cd.types or bool(IC_auras(g, m)), 'shroud')
_grant('Sterling Grove', lambda g, s, m: m.cd is not None and 'E' in m.cd.types, 'shroud')
_grant('Privileged Position', lambda g, s, m: True, 'hexproof')
card('Greater Auramancy', '', types='E', dsl=[])
card('Privileged Position', '', types='E', dsl=[])


def IC_auras(g, m):
    return auras_on(g, m) if getattr(g, 'auras', None) else []


@CI.on('Sterling Grove', 'options')
def _grove(g, src, p, s, post):
    if post is not True or not can_pay(g, p, 1, ''): return []
    if not any(c for c in p.library if 'E' in c.types): return []

    def go():
        if src not in p.perms or not can_pay(g, p, 1, ''): return False
        pay(g, p, 1, ''); die(g, src, 'sac')
        cs = [c for c in p.library if 'E' in c.types]
        c = max(cs, key=lambda c: card_worth(g, p, c)); p.library.remove(c); g.rng.shuffle(p.library); p.library.append(c)
        return True
    return [(0.5 if len(p.hand) > 2 else 2.0, 'Sterling Grove tutor', go)]
card('Sterling Grove', '', types='E', dsl=[])
note('Sterling Grove', 'Full', 'other enchantments have shroud; sacrificed late to put an enchantment on top')


for _n, _pred in (('Destiny Spinner', lambda c: c.creature or 'E' in c.types), ('Allosaurus Shepherd', lambda c: 'G' in c.pips)):
    CI.on(_n, 'uncounterable')(lambda g, src, caster, c, _pred=_pred: 1 if caster is src.owner and _pred(c) else 0)
note('Destiny Spinner', 'Approximate', 'creature and enchantment spells can\'t be countered; the land animation is not used')
card('Destiny Spinner', 'pow=2 tgh=3', dsl=[])


# ======================================================== fog (Spore Frog)
@CI.on('Spore Frog', 'blocks')
def _spore_frog(g, src, p, atk, d, assign):
    if d is not src.owner or getattr(g, 'fog', None) == turn_stamp(g): return
    incoming = sum(epow(g, a) for a in atk if a not in assign)
    if incoming >= max(6, d.life * 0.35):
        die(g, src, 'sac'); g.fog = turn_stamp(g)
        log(f'    Spore Frog prevents all combat damage this turn', g)
card('Spore Frog', 'pow=1', dsl=[])
note('Spore Frog', 'Full', 'sacrificed to fog a big attack')


# ======================================================== planeswalkers
WALKERS = {}


def walker(name, abilities, status=('Approximate', ''), tags='', static=None):
    """abilities: [(loyalty change, label, value(g, p, src) -> utility or None if not usable now, effect(g, p, src))]
    The AI uses one ability per turn at sorcery speed, the most valuable one it can afford."""
    WALKERS[name] = abilities
    card(name, tags or 'leg', dsl=[])
    note(name, *status)

    @CI.on(name, 'options')
    def _opts(g, src, p, s, post):
        if post is None or src.owner is not p: return []
        if src.loyalty is None: src.loyalty = int(src.cd.start_loyalty or 3)
        if src.loyalty_used == (g.round, p.key): return []
        out = []
        for delta, label, val, eff in WALKERS[name]:
            if src.loyalty + delta < 0: continue
            u = val(g, p, src)
            if u is None: continue

            def go(delta=delta, eff=eff, label=label):
                if src not in p.perms or src.loyalty_used == (g.round, p.key) or src.loyalty + delta < 0: return False
                src.loyalty_used = (g.round, p.key)
                src.loyalty += delta * (2 if delta > 0 and _doubler(g, p) else 1)
                log(f'  {NAME(p)} uses {name} ({delta:+d}): {label}', g)
                eff(g, p, src)
                if src in p.perms and src.loyalty <= 0: leave(g, src); to_zone_card(g, src, 'gy')
                return True
            out.append((u + 0.15 * delta, f'{name} {delta:+d}', go))
        return out


def _doubler(g, p):
    return any(m.cd is not None and m.cd.name == 'Doubling Season' for m in p.perms)


def always(x): return lambda g, p, src: x


def best_opp_creature(g, p, pred=lambda m: True):
    cs = [m for q in g.opps(p) for m in q.perms if m.creature and not m.phased and not untargetable(g, m) and pred(m)]
    return max(cs, key=lambda m: pval(g, m)) if cs else None


def best_opp_nonland(g, p, pred=lambda m: True):
    cs = [m for q in g.opps(p) for m in q.perms if not m.phased and not untargetable(g, m) and pred(m)]
    return max(cs, key=lambda m: pval(g, m)) if cs else None


# Elspeth, Sun's Champion
walker("Elspeth, Sun's Champion", [
    (1, 'three Soldiers', always(3.0), lambda g, p, src: make_tokens(g, p, 3, 1, color='W', types=('soldier',))),
    (-3, 'destroy power 4+', lambda g, p, src: (lambda o, m: o - m - 3 if o - m >= 6 else None)(
        sum(pval(g, m) for q in g.opps(p) for m in q.perms if m.creature and epow(g, m) >= 4),
        sum(pval(g, m) for m in p.perms if m.creature and epow(g, m) >= 4)),
     lambda g, p, src: [die(g, m, 'destroy') for q in g.players for m in list(q.perms) if m.creature and epow(g, m) >= 4]),
    (-7, 'emblem', always(9.0), lambda g, p, src: setattr(p, 'elspeth_emblem', True)),
], ('Approximate', 'tokens, the power-4 sweep when it pays, the emblem (+2/+2 flying) as a lasting anthem'))


# Teferi, Hero of Dominaria
def _teferi_minus(g, p, src):
    t = best_opp_nonland(g, p)
    if t is not None: apply_removal(g, p, t, 'tuck')


walker('Teferi, Hero of Dominaria', [
    (1, 'draw, untap two lands', always(3.0), lambda g, p, src: (draw(g, p, 1), [setattr(L, 'tapped', False) for L in p.lands[:2]])),
    (-3, 'tuck a threat', lambda g, p, src: (lambda t: pval(g, t) - 2.5 if t is not None and pval(g, t) >= 5 else None)(best_opp_nonland(g, p)),
     _teferi_minus),
], ('Approximate', '+1 draws and untaps two lands; -3 tucks a threat; the emblem is not modeled'))


# Liliana, Death's Majesty
def _lili_minus(g, p, src):
    cs = [c for c in p.gy if c.creature]
    if cs:
        c = max(cs, key=lambda c: (c.bomb, c.pow, c.cmc)); p.gy.remove(c); n = enter(g, p, c)
        n.ttypes = frozenset(('zombie',))


walker("Liliana, Death's Majesty", [
    (1, 'Zombie, mill two', always(2.5), lambda g, p, src: (make_tokens(g, p, 1, 2, color='B', types=('zombie',)), mill(g, p, 2))),
    (-3, 'reanimate', lambda g, p, src: (max((c.bomb or c.pow) for c in p.gy if c.creature) - 1.0) if any(
        c.creature and (c.bomb or c.pow) >= 4 for c in p.gy) else None, _lili_minus),
    (-7, 'destroy non-Zombies', lambda g, p, src: 8.0 if sum(1 for q in g.opps(p) for m in q.perms if m.creature) >= 4 else None,
     lambda g, p, src: [die(g, m, 'destroy') for q in g.players for m in list(q.perms) if m.creature and not has_type(m, 'zombie')]),
], ('Full', ''))



# ======================================================== artifact tokens (Treasure / Food / Clue) with replacements
def make_artifact_tokens(g, p, kind, n=1):
    """create n Treasure / Food / Clue tokens, applying Academy Manufactor, Chatterfang and token doubling"""
    if n <= 0: return
    mult = E.DSLMOD.token_mult(g, p) if E.DSLMOD is not None else 1
    n *= mult
    kinds = [kind]
    if any(m.cd is not None and m.cd.name == 'Academy Manufactor' for m in p.perms if not m.phased):
        kinds = ['Treasure', 'Food', 'Clue']
    for k in kinds:
        if k == 'Treasure': p.treasures += n
        elif k == 'Food': p.foods = getattr(p, 'foods', 0) + n
        else: p.clues += n
    if g.hooks: CI.fire(g, 'token_created', p, kinds, n * len(kinds))


def sac_food(g, p, n=1):
    if getattr(p, 'foods', 0) < n: return False
    p.foods -= n
    for _ in range(n):
        if g.hooks: CI.fire(g, 'sacrifice', p, 'Food')
    return True


def food_options(g, p, s, post):
    """{2}, {T}, sacrifice a Food: gain 3 life (only when life matters)"""
    if getattr(p, 'foods', 0) < 1 or not can_pay(g, p, 2, '') or p.life > 15 or post is False: return []

    def go():
        if getattr(p, 'foods', 0) < 1 or not can_pay(g, p, 2, ''): return False
        pay(g, p, 2, ''); sac_food(g, p); gain(p, 3); return True
    return [(2.0, 'eat a Food', go)]


# ======================================================== proliferate
def proliferate(g, p, times=1):
    """each permanent / player with counters gets one more of each kind it has (you choose: your +1/+1 and loyalty,
    opponents' -1/-1). Orc Armies are left to their hand tag in the four main decks."""
    times *= 1 + (CI.total(g, 'proliferate_extra', p) if g.hooks else 0)            # Tekuthal
    dbl = 2 if any(m.cd is not None and m.cd.name == 'Doubling Season' for m in p.perms) else 1
    for _ in range(times):
        for m in p.perms:
            if m.army or m.phased: continue
            if m.plus > 0: m.plus += dbl
            if m.loyalty is not None and m.cd is not None and 'P' in m.cd.types: m.loyalty += dbl
            if m.data and m.data.get('counters'):
                for k in m.data['counters']: m.data['counters'][k] += dbl
        for q in g.opps(p):
            for m in list(q.perms):
                if m.creature and m.plus < 0:
                    m.plus -= 1
                    if etgh(g, m) <= 0: die(g, m, 'sba')
        if g.hooks: CI.fire(g, 'proliferated', p)


CI.proliferate = proliferate


# ======================================================== mana: Cradle, Nykthos, Coffers, Crypt Ghast, Circle of Dreams
def devotion(p, col):
    return sum(m.cd.pips.count(col) for m in p.perms if m.cd is not None and m.cd.perm and not m.phased)


CI.DYN_MANA["Gaea's Cradle"] = lambda g, p, L: sum(1 for m in p.perms if m.creature and not m.phased)
CI.DYN_MANA['Nykthos, Shrine to Nyx'] = lambda g, p, L: max(1, max(devotion(p, c) for c in 'WUBRG') - 2)
CI.DYN_MANA['Circle of Dreams Druid'] = lambda g, p, m: sum(1 for x in p.perms if x.creature and not x.phased)
CI.DYN_MANA['Cabal Coffers'] = lambda g, p, L: max(1, sum(1 for x in p.lands if 'swamp' in x.cd.subtypes or x.cd.name == 'Swamp'
                                                        or _urborg(g)) - 2)
CI.DYN_MANA['Marwyn, the Nurturer'] = lambda g, p, m: max(1, epow(g, m))
note("Gaea's Cradle", 'Full', 'G for each creature you control')
note('Nykthos, Shrine to Nyx', 'Approximate', 'taps for devotion minus the {2} activation')
note('Circle of Dreams Druid', 'Full', 'G for each creature you control')
note('Cabal Coffers', 'Approximate', 'B per Swamp minus the {2} activation (Urborg makes every land a Swamp)')


def _urborg(g):
    return any(L.cd.name == 'Urborg, Tomb of Yawgmoth' for q in g.players if q.alive for L in q.lands)


note('Urborg, Tomb of Yawgmoth', 'Approximate', 'every land counts as a Swamp for Cabal Coffers and Crypt Ghast')


@CI.on('Crypt Ghast', 'land_mana')
def _ghast(g, src, p, L):
    return 1 if p is src.owner and ('swamp' in L.cd.subtypes or L.cd.name == 'Swamp' or _urborg(g)) else 0
card('Crypt Ghast', 'pow=2', dsl=[])
note('Crypt Ghast', 'Partial', 'Swamps tap for an extra B; extort not modeled')


@CI.on('Collector Ouphe', 'no_artifact_mana')
def _ouphe(g, src, p): return 1
card('Collector Ouphe', 'pow=2', dsl=[])
note('Collector Ouphe', 'Approximate', 'artifact mana (rocks, Treasures) is off for everyone; other artifact abilities '
     'still work')
