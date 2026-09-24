"""Hand-written implementations shared by many pool decks (see cardimpl.py for the hook events)."""
from engine import *
import engine as E
from cardimpl import on
import cardimpl as CI
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
