"""Tier 2 pool decks: Kaalia, Meren, Sythis, Brago, Lord Windgrace (and the staples they share)."""
from engine import *
import engine as E
import cardimpl as CI
from cardimpl import on, _eot, count_type
from pool_cards import card, note
import impl_common as IC


def best_target_any(g, p, n):
    """'deals n damage to any target': kill the best creature it kills, else the most threatening opponent's face"""
    opps = g.opps(p)
    if not opps: return
    lethal = [q for q in opps if q.life <= n]
    if lethal: lose_life(g, lethal[0], n, p, kind='triggers'); return
    tg = [m for q in opps for m in q.perms if m.creature and not untargetable(g, m) and etgh(g, m) <= n]
    best = max(tg, key=lambda m: pval(g, m)) if tg else None
    if best is not None and pval(g, best) >= 3: apply_removal(g, p, best, f'dmg{n}')
    else: lose_life(g, max(opps, key=lambda q: threat(g, p, q)), n, p, kind='triggers')


def blink(g, p, m):
    """exile m and return it under its owner's control (ETBs again, untapped, summoning sick)"""
    if m.token or m.cd is None or m not in m.owner.perms: return None
    if getattr(g, 'blink_depth', 0) >= 3: return None                      # blink chains are combos, not loops here
    g.blink_depth = getattr(g, 'blink_depth', 0) + 1
    try:
        return _blink(g, p, m)
    finally:
        g.blink_depth -= 1


def _blink(g, p, m):
    cd, owner, cmd = m.cd, m.orig, m.is_cmd
    leave(g, m)
    n = enter(g, owner, cd, orig=owner)
    n.is_cmd = cmd
    if g.hooks: CI.fire(g, 'exiled_from_bf', m)
    log(f'    {cd.name} is blinked', g)
    return n


def blink_value(g, p, m):
    """how much re-entering is worth for p's permanent m"""
    if m.token or m.cd is None or m.is_cmd: return 0
    import dsl
    v = dsl.etb_value(m.cd)
    if m.cd.name in CI.HOOKS and 'etb' in CI.HOOKS[m.cd.name]: v += 3
    if m.cd.name in ('Oblivion Ring', 'Banishing Light', 'Detention Sphere', 'Cast Out', 'Journey to Nowhere'): v = 0
    return v


# ======================================================== monarch, Palace Jailer
@on('Palace Jailer', 'etb')
def _jailer(g, src, p, m):
    if m is not src: return
    CI.become_monarch(g, src.owner)
    IC.oring_exile(g, src, src.owner, lambda x: x.creature)


@on('Palace Jailer', 'monarch')
def _jailer_lose(g, src, p):
    if p is not src.owner: IC.oring_return(g, src)
    if src.data: src.data['oring'] = []


card('Palace Jailer', 'human pow=2', dsl=[])
note('Palace Jailer', 'Full', 'monarch (end-step draw, taken by combat damage); exiles a creature until an opponent '
     'becomes the monarch')


# ======================================================== Brago and blink
@on('Brago, King Eternal', 'combat_damage')
def _brago(g, src, p, a, d, dmg):
    if a is not src: return
    o = src.owner
    for m in [m for m in o.perms if m is not src and not m.token and m.cd is not None and not m.cd.land
              and (blink_value(g, o, m) > 0 or m.tapped and ('rock' in m.cd.tags))]:
        blink(g, o, m)
card('Brago, King Eternal', 'leg pow=2 tgh=4 fly', dsl=[])
note('Brago, King Eternal', 'Full', 'combat damage: blink every nonland permanent with an ETB worth repeating '
     '(and tapped mana rocks)')


@on('Soulherder', 'end_step')
def _soulherder(g, src, p):
    if p is not src.owner: return
    cands = [m for m in p.perms if m is not src and m.creature and blink_value(g, p, m) > 0]
    if cands: blink(g, p, max(cands, key=lambda m: blink_value(g, p, m)))


@on('Soulherder', 'exiled_from_bf')
def _soulherder_grow(g, src, m):
    if m.creature: src.plus += 1
card('Soulherder', 'pow=1', dsl=[])
note('Soulherder', 'Full', 'end step: blink the best ETB creature; grows when creatures are exiled')


@on("Conjurer's Closet", 'end_step')
def _closet(g, src, p):
    if p is not src.owner: return
    cands = [m for m in p.perms if m.creature and blink_value(g, p, m) > 0]
    if cands: blink(g, p, max(cands, key=lambda m: blink_value(g, p, m)))
card("Conjurer's Closet", '', types='A', dsl=[])
note("Conjurer's Closet", 'Full', '')


def _blink_option(name, cost_g, cost_p, other=True, label=None, needs_pair=False):
    @on(name, 'options')
    def _opt(g, src, p, s, post):
        if post is None and not E.can_pay(g, p, cost_g, cost_p): return []
        if not can_pay(g, p, cost_g, cost_p): return []
        cands = [m for m in p.perms if m.creature and (m is not src or not other) and blink_value(g, p, m) >= 3]
        if not cands: return []
        t = max(cands, key=lambda m: blink_value(g, p, m))

        def go():
            if t not in p.perms or not can_pay(g, p, cost_g, cost_p): return False
            pay(g, p, cost_g, cost_p); blink(g, p, t); return True
        return [(0.5 + 0.5 * blink_value(g, p, t), label or f'{name}: blink {t.name}', go)]


_blink_option('Eldrazi Displacer', 3, '')
_blink_option('Deadeye Navigator', 1, 'U', other=False)
card('Eldrazi Displacer', 'pow=3', dsl=[])
note('Eldrazi Displacer', 'Approximate', 'blinks your best ETB creature ({C} read as {1}); flickering opponents\' '
     'creatures not used')
card('Deadeye Navigator', 'pow=5 bomb=5', dsl=[])
note('Deadeye Navigator', 'Approximate', 'soulbond read as: pay {1}{U} to blink any of your ETB creatures')

card('Ephemerate', 'prot=blink ephemerate', types='I')
note('Ephemerate', 'Approximate', 'protection blink (outside decks), rebound blink next upkeep; not cast proactively')


# ======================================================== Kaalia of the Vast
ADD = ('angel', 'demon', 'dragon')


@on('Kaalia of the Vast', 'attack')
def _kaalia(g, src, p, atk, d):
    if src not in atk or src.owner is not p: return
    cands = [c for c in p.hand if c.creature and set(ADD) & set(c.subtypes)]
    if not cands: return
    c = max(cands, key=lambda c: (c.bomb, c.pow, c.cmc))
    p.hand.remove(c)
    m = enter(g, p, c)
    m.tapped = True; m.sick = False
    log(f'    Kaalia puts {c.name} onto the battlefield attacking', g)
    p.stats['kaalia_cheats'] += 1
    return [m]
card('Kaalia of the Vast', 'leg human pow=2 fly')
note('Kaalia of the Vast', 'Full', 'attacks: best Angel/Demon/Dragon from hand onto the battlefield attacking (ETBs fire)')


def kaalia_prio(g, p, c):
    """hold Angels, Demons and Dragons for Kaalia when she's ready to attack"""
    kaalia = any(m.is_cmd for m in p.perms)
    if c.creature and set(ADD) & set(c.subtypes) and c.cmc >= 5 and (kaalia or p.cmd_in_zone and p.tax <= 2):
        return 0
    return None


@on('Terror of the Peaks', 'etb')
def _terror(g, src, p, m):
    if m is not src and m.owner is src.owner and m.creature and epow(g, m) > 0: best_target_any(g, src.owner, epow(g, m))
card('Terror of the Peaks', 'pow=5 tgh=4 fly bomb=6', dsl=[])
note('Terror of the Peaks', 'Approximate', 'damage equal to power when another creature enters; the targeting life tax '
     'is ignored')


@on('Scourge of Valkas', 'etb')
def _valkas(g, src, p, m):
    if m.owner is src.owner and (m is src or has_type(m, 'dragon')):
        best_target_any(g, src.owner, count_type(g, src.owner, 'dragon'))
card('Scourge of Valkas', 'pow=4 fly', dsl=[])
note('Scourge of Valkas', 'Approximate', 'Dragon ETB damage; firebreathing not used')


@on('Dragon Tempest', 'etb')
def _tempest(g, src, p, m):
    if m.owner is not src.owner or not m.creature: return
    if m.fly or E.DSLMOD.has_kw(g, m, 'flying'): m.sick = False
    if has_type(m, 'dragon'): best_target_any(g, src.owner, count_type(g, src.owner, 'dragon'))
card('Dragon Tempest', '', types='E', dsl=[])
note('Dragon Tempest', 'Full', '')


@on('Drakuseth, Maw of Flames', 'attack')
def _drakuseth(g, src, p, atk, d):
    if src in atk:
        best_target_any(g, p, 4); best_target_any(g, p, 3); best_target_any(g, p, 3)
card('Drakuseth, Maw of Flames', 'leg pow=7 fly bomb=7', dsl=[])
note('Drakuseth, Maw of Flames', 'Approximate', '4 + 3 + 3 damage split by the any-target heuristic')


@on('Balefire Dragon', 'combat_damage')
def _balefire(g, src, p, a, d, dmg):
    if a is src:
        for m in list(d.perms):
            if m.creature and etgh(g, m) <= dmg: die(g, m, 'destroy')
card('Balefire Dragon', 'pow=6 fly bomb=6', dsl=[])
note('Balefire Dragon', 'Full', '')


@on('Master of Cruelties', 'blocks')
def _moc(g, src, p, atk, d, assign):
    if src in atk and len(atk) == 1 and src not in assign and d.life > 1 and not prevents_damage(g, d, p):
        lose_life(g, d, d.life - 1, p, kind='triggers')
        _eot(g, src, -epow(g, src), 0)
card('Master of Cruelties', 'pow=1 tgh=4 dt', dsl=[])
note('Master of Cruelties', 'Approximate', 'attacking alone unblocked sets life to 1; the attack-alone rule is not '
     'forced on the AI')


@on('Admonition Angel', 'landfall')
def _admonition(g, src, p):
    if p is src.owner: IC.oring_exile(g, src, p, lambda m: m is not src)


@on('Admonition Angel', 'leaves')
def _admonition_leave(g, src): IC.oring_return(g, src)
card('Admonition Angel', 'pow=6 fly bomb=6', dsl=[])
note('Admonition Angel', 'Full', 'landfall: exile a nonland permanent until it leaves')


@on('Angel of Serenity', 'etb')
def _serenity(g, src, p, m):
    if m is not src: return
    o = src.owner
    for _ in range(3):
        cands = [x for q in g.opps(o) for x in q.perms if x.creature and not untargetable(g, x)]
        if not cands: break
        x = max(cands, key=lambda x: pval(g, x))
        if pval(g, x) < 2: break
        owner = x.owner; apply_removal(g, o, x, 'exile', src.cd)
        if x not in owner.perms and not x.token and x.cd in x.orig.exile:
            src.data = src.data or {}; src.data.setdefault('serenity', []).append((x.cd, x.orig))


@on('Angel of Serenity', 'leaves')
def _serenity_leave(g, src):
    for cd, owner in (src.data or {}).get('serenity', []):
        if cd in owner.exile: owner.exile.remove(cd); owner.hand.append(cd)
card('Angel of Serenity', 'pow=5 tgh=6 fly bomb=6', dsl=[])
note('Angel of Serenity', 'Approximate', 'exiles up to three opposing creatures (graveyard targets not used); they '
     'return to hand when it leaves')


@on('Archangel Avacyn // Avacyn, the Purifier', 'etb')
def _avacyn(g, src, p, m):
    if m is src:
        for x in src.owner.perms:
            if x.creature: g.eot_kw.setdefault(id(x), set()).add('indestructible')
card('Archangel Avacyn // Avacyn, the Purifier', 'leg pow=4 fly vig flash', dsl=[])
note('Archangel Avacyn // Avacyn, the Purifier', 'Partial', 'flash, ETB indestructible; the transform is not modeled')


@on('Archfiend of Despair', 'no_lifegain')
def _despair_nolife(g, src, p):
    return 1 if p is not src.owner else 0


@on('Archfiend of Despair', 'end_step')
def _despair(g, src, p):
    st = turn_stamp(g)
    for q in g.opps(src.owner):
        lt = getattr(q, 'lost_turn', None)
        if lt and lt[0] == st and lt[1] > 0: lose_life(g, q, lt[1], src.owner, kind='drain')
card('Archfiend of Despair', 'pow=6 fly bomb=6', dsl=[])
note('Archfiend of Despair', 'Full', '')


@on('Liesa, Shroud of Dusk', 'cast')
def _liesa(g, src, caster, c):
    lose_life(g, caster, 2, src.owner if caster is not src.owner else caster, kind='drain')
card('Liesa, Shroud of Dusk', 'leg pow=5 fly lifelink', dsl=[])
note('Liesa, Shroud of Dusk', 'Approximate', 'every spell costs its caster 2 life; the recast-for-life clause is ignored')


@on('Glorybringer', 'attack')
def _glorybringer(g, src, p, atk, d):
    if src not in atk or (src.data or {}).get('exerted') == p.turns - 1: return
    cands = [m for q in g.opps(p) for m in q.perms if m.creature and not has_type(m, 'dragon') and not untargetable(g, m)
             and etgh(g, m) <= 4]
    if cands:
        best = max(cands, key=lambda m: pval(g, m))
        if pval(g, best) >= 2:
            apply_removal(g, p, best, 'dmg4', src.cd); src.data = {'exerted': p.turns}
card('Glorybringer', 'pow=4 fly haste', dsl=[])
note('Glorybringer', 'Approximate', 'exerts for 4 damage to a creature every other attack')


@on('Thundermaw Hellkite', 'etb')
def _thundermaw(g, src, p, m):
    if m is not src: return
    for q in g.opps(src.owner):
        for x in list(q.perms):
            if x.creature and (x.fly or E.DSLMOD.has_kw(g, x, 'flying')):
                x.tapped = True
                if etgh(g, x) <= 1: die(g, x, 'destroy')
card('Thundermaw Hellkite', 'pow=5 fly haste bomb=5', dsl=[])
note('Thundermaw Hellkite', 'Full', '')


@on('Aurelia, Exemplar of Justice', 'upkeep')
def _aurelia_ex(g, src, p):
    if p is not src.owner: return
    cr = [m for m in p.perms if m.creature and not m.noatk]
    if cr:
        t = max(cr, key=lambda m: (m.is_cmd, epow(g, m))); _eot(g, t, 2, 0)
        g.eot_kw.setdefault(id(t), set()).update(('trample', 'vigilance'))
card('Aurelia, Exemplar of Justice', 'leg pow=2 tgh=5 fly', dsl=[], kws={'flying', 'mentor'})
note('Aurelia, Exemplar of Justice', 'Full', 'mentor; +2/+0, trample and vigilance on the best attacker each combat')


@on('Bloodgift Demon', 'upkeep')
def _bloodgift(g, src, p):
    if p is src.owner and p.life > 10: draw(g, p, 1); lose_life(g, p, 1, p)
card('Bloodgift Demon', 'pow=5 tgh=4 fly bomb=5', dsl=[])
note('Bloodgift Demon', 'Full', '')


@on('Demonlord Belzenlok', 'etb')
def _belz(g, src, p, m):
    if m is not src: return
    o = src.owner
    for _ in range(4):
        nl = next((c for c in reversed(o.library) if not c.land), None)
        if nl is None: break
        o.library.remove(nl); o.hand.append(nl); lose_life(g, o, 1, o)
        if nl.cmc < 4: break
card('Demonlord Belzenlok', 'leg pow=6 fly trample bomb=6', dsl=[])
note('Demonlord Belzenlok', 'Approximate', 'nonland cards to hand until one with MV < 4 (the exiled lands are skipped)')


@on('Resplendent Angel', 'end_step')
def _resplendent(g, src, p):
    gt = getattr(src.owner, 'gained_turn', None)
    if gt and gt[0] == turn_stamp(g) and gt[1] >= 5:
        make_tokens(g, src.owner, 1, 4, fly=True, color='W', types=('angel',))
card('Resplendent Angel', 'pow=3 fly', dsl=[])
note('Resplendent Angel', 'Approximate', '4/4 Angel on 5+ life gained in a turn; the pump is not used')
card('Baneslayer Angel', 'pow=5 fly lifelink bomb=5', dsl=[])
note('Baneslayer Angel', 'Approximate', 'first strike, lifelink, flying; protection from Demons and Dragons ignored')
card('Archfiend of Sorrows', 'pow=4 tgh=5 fly', dsl=[{'type': 'triggered', 'event': 'etb', 'source': 'self', 'effects': [
    {'do': 'pump', 'pow': -2, 'tgh': -2, 'what': {'sel': 'all', 'filter': {'type': 'creature', 'controller': 'opp'}}}]}])
note('Archfiend of Sorrows', 'Partial', 'ETB -2/-2 to opposing creatures; unearth not modeled')
card('Sign in Blood', 'draw=2 lose=2', types='S', dsl=[])
note('Sign in Blood', 'Full', 'you draw two and lose 2')


# ======================================================== Lord Windgrace (lands)
def _wg_plus(g, p, src):
    if not p.hand: draw(g, p, 1); return
    lands = [c for c in p.hand if c.land]
    c = lands[0] if lands and (len(lands) >= 2 or any(True for _ in p.gy if _.land) or len(p.lands) >= 5) else \
        min(p.hand, key=lambda c: card_worth(g, p, c))
    discard_cards(g, p, [c]); draw(g, p, 2 if c.land else 1)
    if g.hooks and c.land: CI.fire(g, 'land_gy', p, c)


def _wg_minus(g, p, src):
    ls = sorted([c for c in p.gy if c.land], key=lambda c: (-len(c.tags.get('c', '')), 'f' not in c.tags))[:2]
    for c in ls:
        p.gy.remove(c); p.lands.append(Land(c, False)); landfall(g, p)
        import ais
        if E.POOL_RULES and 'f' in c.tags and p.lands and p.lands[-1].cd is c: ais.crack_fetch(g, p, p.lands[-1])


def _wg_ult(g, p, src):
    for _ in range(6):
        t = IC.best_opp_nonland(g, p)
        if t is None: break
        apply_removal(g, p, t, 'destroy')
    make_tokens(g, p, 6, 2, color='G', types=('cat', 'warrior'))


IC.walker('Lord Windgrace', [
    (2, 'discard, draw', lambda g, p, src: 2.5, _wg_plus),
    (-3, 'two lands back', lambda g, p, src: 1.5 + 1.5 * min(2, sum(1 for c in p.gy if c.land)) if sum(1 for c in p.gy if c.land) >= 1 else None, _wg_minus),
    (-11, 'ultimate', lambda g, p, src: 12.0, _wg_ult),
], ('Full', 'loyalty abilities and ultimate (forestwalk on the Cats ignored)'))


@on('Moraug, Fury of Akoum', 'landfall')
def _moraug(g, src, p):
    if src.owner is p and g.active is p and getattr(p, 'combat_no', 0) < 1 and p.extra_combats < 3: p.extra_combats += 1
card('Moraug, Fury of Akoum', 'leg pow=6 bomb=6 warrior', dsl=[])
note('Moraug, Fury of Akoum', 'Approximate', 'main-phase landfall: an additional combat (the per-attack +1/+0 is not modeled)')


@on('Omnath, Locus of Rage', 'landfall')
def _omnath(g, src, p):
    if src.owner is p: make_tokens(g, p, 1, 5, color='RG', types=('elemental',))


@on('Omnath, Locus of Rage', 'dies')
def _omnath_dies(g, src, m, cause):
    if m.owner is src.owner and m.creature and has_type(m, 'elemental') or m is src: best_target_any(g, src.owner, 3)
card('Omnath, Locus of Rage', 'leg pow=5 bomb=6', dsl=[])
note('Omnath, Locus of Rage', 'Full', '')


IC.SELF_PT['Multani, Yavimaya\'s Avatar'] = lambda g, p, m: ((k := len(p.lands) + sum(1 for c in p.gy if c.land)), k)


@on('Multani, Yavimaya\'s Avatar', 'etb')
def _multani(g, src, p, m):
    if m is src: g.selfpt = True
card('Multani, Yavimaya\'s Avatar', 'leg pow=0 tgh=0 reach trample bomb=6', dsl=[])
note('Multani, Yavimaya\'s Avatar', 'Partial', '+1/+1 per land on the battlefield and in the graveyard; the '
     'graveyard recursion is not used')


@on('The Gitrog Monster', 'upkeep')
def _gitrog_up(g, src, p):
    if p is not src.owner: return
    if len(p.lands) >= 4:
        L = min(p.lands, key=lambda L: (not L.tapped, len(L.cd.tags.get('c', ''))))
        p.lands.remove(L); p.gy.append(L.cd); CI.fire(g, 'land_gy', p, L.cd)
    else: die(g, src, 'sac')


@on('The Gitrog Monster', 'land_gy')
def _gitrog_draw(g, src, p, cd):
    if p is src.owner and len(p.library) > 8: draw(g, p, 1)


@on('The Gitrog Monster', 'extra_lands')
def _gitrog_extra(g, src, p): return 1 if p is src.owner else 0


@on('The Gitrog Monster', 'discard')
def _gitrog_discard(g, src, q, c):
    if q is src.owner and c.land and len(q.library) > 8: draw(g, q, 1)
card('The Gitrog Monster', 'leg pow=6 dt bomb=6', dsl=[])
note('The Gitrog Monster', 'Approximate', 'upkeep land sacrifice, extra land drop, draw when lands hit the graveyard '
     '(sacrifices, fetch lands, discards)')


@on('Courser of Kruphix', 'lands_from_top')
def _courser_top(g, src, p): return 1 if p is src.owner else 0


@on('Courser of Kruphix', 'landfall')
def _courser_life(g, src, p):
    if p is src.owner: gain(p, 1)
card('Courser of Kruphix', 'pow=2 tgh=4', dsl=[])
note('Courser of Kruphix', 'Full', '')


@on('Conduit of Worlds', 'lands_from_gy')
def _conduit(g, src, p): return 1 if p is src.owner else 0
card('Conduit of Worlds', '', types='A', dsl=[])
note('Conduit of Worlds', 'Partial', 'land drops from the graveyard; casting permanents from the graveyard not used')


@IC.spell('Mulch', prio=40, tags='', types='S', status=('Full', ''))
def _mulch(g, p, c, ctx):
    top = [p.library.pop() for _ in range(min(4, len(p.library)))]
    for x in top:
        (p.hand if x.land else p.gy).append(x)
        if x.land and g.hooks: pass


@IC.spell('Grow from the Ashes', prio=lambda g, p, c: 60 if p.turns <= 6 else 30, tags='', types='S',
          status=('Full', 'kicked when {2} more is available'))
def _grow(g, p, c, ctx):
    n = 2 if can_pay(g, p, 2, '') else 1
    if n == 2: pay(g, p, 2, '')
    land_ramp(g, p, n, False)


@IC.spell('Hour of Promise', prio=lambda g, p, c: 55 if p.turns <= 8 else 30, tags='', types='S',
          status=('Approximate', 'two basic-typed lands tapped; the Desert Zombies are not modeled'))
def _hour(g, p, c, ctx):
    land_ramp(g, p, 2, True)


card('Selesnya Sanctuary', 'c=GW t bounceland amt=2', types='L', dsl=[])
note('Selesnya Sanctuary', 'Full', 'bounce land')


# ======================================================== Meren of Clan Nel Toth (recursion)
@on('Meren of Clan Nel Toth', 'dies')
def _meren_xp(g, src, m, cause):
    if m.owner is src.owner and m is not src and m.creature: src.owner.experience = getattr(src.owner, 'experience', 0) + 1


@on('Meren of Clan Nel Toth', 'end_step')
def _meren_end(g, src, p):
    if p is not src.owner: return
    cs = [c for c in p.gy if c.creature]
    if not cs: return
    xp = getattr(p, 'experience', 0)
    import dsl
    ok = [c for c in cs if c.cmc <= xp]
    if ok:
        c = max(ok, key=lambda c: (dsl.etb_value(c) + (c.bomb or c.pow) + (3 if c.name in CI.HOOKS else 0), c.cmc))
        p.gy.remove(c); enter(g, p, c); log(f'    Meren returns {c.name} to the battlefield', g)
    else:
        c = max(cs, key=lambda c: card_worth(g, p, c)); p.gy.remove(c); p.hand.append(c)
card('Meren of Clan Nel Toth', 'leg human shaman pow=3 tgh=4', dsl=[])
note('Meren of Clan Nel Toth', 'Full', 'experience per creature death; end step: best creature back (battlefield '
     'if its MV fits the experience, else hand)')


EVOKE = {'Shriekmaw': (1, 'B'), 'Mulldrifter': (2, 'U')}


def evoke_options(g, p, s, post):
    o = []
    if post is None: return o
    for c in p.hand:
        if c.name not in EVOKE or not castable(g, p, c): continue
        gen, pips = EVOKE[c.name]
        if can_pay(g, p, *cost_of(p, c)) or not can_pay(g, p, gen, pips): continue
        if c.name == 'Shriekmaw':
            t = IC.best_opp_creature(g, p, lambda m: not (m.cd is not None and ('A' in m.cd.types or 'B' in m.cd.pips)))
            if t is None or pval(g, t) < 3: continue
            u = pval(g, t) - 2.5
        else:
            u = 2.5

        def go(c=c, gen=gen, pips=pips):
            if c not in p.hand or not can_pay(g, p, gen, pips): return False
            p.hand.remove(c); pay(g, p, gen, pips); p.spells_this_turn += 1; on_cast(g, p, c)
            log(f'  {NAME(p)} evokes {c.name}', g)
            m = enter(g, p, c, was_cast=True)
            if m in p.perms: die(g, m, 'sac')
            return True
        o.append((u, f'evoke {c.name}', go))
    return o


card('Shriekmaw', 'pow=3 tgh=2 rem=destroy tgt=cna etb', dsl=[])
note('Shriekmaw', 'Approximate', 'ETB destroys a nonartifact creature (the nonblack clause is checked on evoke only); '
     'evoke used when the full cost is out of reach; fear not modeled')
card('Nekrataal', 'human pow=2 tgh=1 rem=destroy tgt=cna etb', dsl=[], kws={'first strike'})
note('Nekrataal', 'Approximate', 'ETB destroy; the nonblack restriction is ignored')


@on('Survival of the Fittest', 'options')
def _survival(g, src, p, s, post):
    if post is None or not can_pay(g, p, 0, 'G'): return []
    disc = [c for c in p.hand if c.creature]
    lib = [c for c in searchable(g, p) if c.creature]
    if not disc or not lib: return []
    d = min(disc, key=lambda c: card_worth(g, p, c))
    reanim = any(m.cd is not None and m.cd.name in ('Meren of Clan Nel Toth',) for m in p.perms)
    if reanim: d = max(disc, key=lambda c: c.cmc if c.cmc <= getattr(p, 'experience', 0) else -1)
    want = max(lib, key=lambda c: card_worth(g, p, c))

    def go():
        if d not in p.hand or not can_pay(g, p, 0, 'G'): return False
        pay(g, p, 0, 'G'); discard_cards(g, p, [d])
        if want in p.library: p.library.remove(want); p.hand.append(want); g.rng.shuffle(p.library); p.stats['tutored'] += 1
        return True
    return [(1.5 + card_worth(g, p, want) / 30.0, 'Survival of the Fittest', go)]
card('Survival of the Fittest', '', types='E', dsl=[])
note('Survival of the Fittest', 'Approximate', 'discards the worst creature (or one Meren can return) for the best one')


@on('Birthing Pod', 'options')
def _pod(g, src, p, s, post):
    if post is None or src.tapped or src.sick or not can_pay(g, p, 1, 'G'): return []
    fod = [m for m in p.perms if m.creature and not m.token and m.cd is not None and not m.is_cmd]
    best = None
    for m in fod:
        up = [c for c in searchable(g, p) if c.creature and c.cmc == m.cd.cmc + 1]
        if up:
            c = max(up, key=lambda c: card_worth(g, p, c))
            gain_ = card_worth(g, p, c) / 10.0 - pval(g, m) + 1.0
            if best is None or gain_ > best[0]: best = (gain_, m, c)
    if best is None or best[0] < 0.5: return []
    _, m, c = best

    def go():
        if m not in p.perms or c not in p.library or src.tapped or not can_pay(g, p, 1, 'G'): return False
        pay(g, p, 1, 'G'); src.tapped = True
        p.library.remove(c); g.rng.shuffle(p.library)     # take it before the sacrifice's triggers draw
        die(g, m, 'sac'); enter(g, p, c); return True
    return [(1.5 + best[0], f'Birthing Pod {m.name} -> {c.name}', go)]
card('Birthing Pod', '', types='A', dsl=[])
note('Birthing Pod', 'Full', 'sacrifice a creature for one with MV one higher, when it upgrades')


@on('Evolutionary Leap', 'options')
def _leap(g, src, p, s, post):
    if post is None or not can_pay(g, p, 0, 'G'): return []
    fod = [m for m in p.perms if m.creature and (m.token or pval(g, m) < 2) and not m.is_cmd]
    if not fod or not any(c.creature for c in p.library): return []
    m = min(fod, key=lambda x: pval(g, x))

    def go():
        if m not in p.perms or not can_pay(g, p, 0, 'G'): return False
        pay(g, p, 0, 'G'); die(g, m, 'sac')
        for i in range(len(p.library) - 1, -1, -1):
            if p.library[i].creature:
                c = p.library.pop(i); p.hand.append(c); break
        return True
    return [(2.0 + IC.death_value(g, p) / 2.0, 'Evolutionary Leap', go)]
card('Evolutionary Leap', '', types='E', dsl=[])
note('Evolutionary Leap', 'Full', 'sacrifice spare creatures for the next creature card')


@IC.spell('Victimize', prio=lambda g, p, c: 58 if sum(1 for x in p.gy if x.creature and (x.bomb or x.pow) >= 4) >= 1 and
          any(m.creature and (m.token or pval(g, m) < 2.5) for m in p.perms) else 0, tags='', types='S',
          status=('Full', 'sacrifice a creature: the two best creature cards return tapped'))
def _victimize(g, p, c, ctx):
    fod = [m for m in p.perms if m.creature and not m.is_cmd]
    if not fod: return
    die(g, min(fod, key=lambda m: pval(g, m)), 'sac')
    cs = sorted([x for x in p.gy if x.creature], key=lambda x: -((x.bomb or x.pow) + x.cmc * 0.1))[:2]
    for x in cs:
        p.gy.remove(x); m = enter(g, p, x); m.tapped = True


note('Grisly Salvage', 'Approximate', 'hand-tagged (Sephiroth\'s list): outside decks cast it for a land (or with Meren out); the rest go to the graveyard')


# ======================================================== Sythis, Harvest's Hand (enchantress)
def _constellation(name, fn, status=('Full', '')):
    @on(name, 'etb')
    def _c(g, src, p, m):
        if m.owner is src.owner and m.cd is not None and 'E' in m.cd.types: fn(g, src.owner, src, m)
    note(name, *status)


_constellation('Setessan Champion', lambda g, p, src, m: (setattr(src, 'plus', src.plus + 1), draw(g, p, 1)))
_constellation('Eidolon of Blossoms', lambda g, p, src, m: draw(g, p, 1))
card('Setessan Champion', 'human warrior pow=1 tgh=3', dsl=[])
card('Eidolon of Blossoms', 'pow=2', dsl=[])


@on('Hallowed Haunting', 'cast')
def _haunting(g, src, caster, c):
    if caster is src.owner and 'E' in c.types:
        make_tokens(g, caster, 1, 1, color='W', types=('spirit', 'cleric'))
        g.selfpt = True


@on('Hallowed Haunting', 'grant_kw')
def _haunting_kw(g, src, m, kw):
    o = src.owner
    return kw in ('flying', 'vigilance') and m.owner is o and m.creature and IC.n_ench(o) >= 7
card('Hallowed Haunting', '', types='E', dsl=[])
note('Hallowed Haunting', 'Approximate', 'Spirit Cleric per enchantment cast (P/T = Spirits you control); flying and '
     'vigilance at seven enchantments')


@on('Starfield of Nyx', 'upkeep')
def _starfield(g, src, p):
    if p is not src.owner: return
    es = [c for c in p.gy if 'E' in c.types and not ('aura' in c.subtypes)]
    if es:
        c = max(es, key=lambda c: card_worth(g, p, c)); p.gy.remove(c); enter(g, p, c)
card('Starfield of Nyx', '', types='E', dsl=[])
note('Starfield of Nyx', 'Partial', 'returns an enchantment each upkeep; animating enchantments at five is not modeled')


@on('Carpet of Flowers', 'upkeep')
def _carpet(g, src, p):
    if p is not src.owner: return
    x = max((sum(1 for L in q.lands if 'island' in L.cd.subtypes or L.cd.name == 'Island') for q in g.opps(p)), default=0)
    p.floatA += x
card('Carpet of Flowers', '', types='E', dsl=[])
note('Carpet of Flowers', 'Approximate', 'mana equal to an opponent\'s Islands once each turn (precombat)')


@on('Charming Prince', 'etb')
def _prince(g, src, p, m):
    if m is not src: return
    o = src.owner
    cands = [x for x in o.perms if x is not src and x.creature and blink_value(g, o, x) >= 3]
    if cands: blink(g, o, max(cands, key=lambda x: blink_value(g, o, x)))
    else: gain(o, 3)
card('Charming Prince', 'human pow=2', dsl=[])
note('Charming Prince', 'Approximate', 'blinks your best ETB creature (immediately, not at end step), else 3 life')


# ======================================================== value blinks: Ephemerate, Restoration Angel (pool decks)
def _value_blink(name, gen, pips):
    """cast at the end of an opponent's turn for value: blink your best enters-the-battlefield creature (Ephemerate
    rebounds; Restoration Angel also arrives as a 3/4 flash flier). Kept for protection when nothing is worth it."""
    @on(name, 'hand_options')
    def _opt(g, c, p, s, post):
        if post is not None or g.active is p or c not in p.hand or p.key in STYLE_KEYS or not can_pay(g, p, gen, pips): return []
        cands = [m for m in p.perms if m.creature and not m.token and blink_value(g, p, m) >= 3
                 and not (name == 'Restoration Angel' and has_type(m, 'angel'))]
        t = max(cands, key=lambda m: blink_value(g, p, m)) if cands else None
        if t is None and name != 'Restoration Angel': return []
        v = (blink_value(g, p, t) if t is not None else 0) + (2.5 if name == 'Restoration Angel' else 0)
        if v < 3: return []

        def go():
            if c not in p.hand or not can_pay(g, p, gen, pips): return False
            p.hand.remove(c); pay(g, p, gen, pips)
            p.spells_this_turn += 1; p.cast_names.add(c.name); on_cast(g, p, c)
            if name == 'Ephemerate': p.exile.append(c); p.rebound = getattr(p, 'rebound', []) + [c]
            else: enter(g, p, c)
            if t is not None and t in p.perms: blink(g, p, t)
            log(f'  {NAME(p)} casts {name} at end of turn' + (f': blinks {t.name}' if t is not None else ''), g)
            return True
        return [(v - 1.0, f'{name} (end of turn)', go)]


STYLE_KEYS = ('seph', 'veyran', 'sauron', 'najeela')
_value_blink('Ephemerate', 0, 'W')
_value_blink('Restoration Angel', 3, 'W')
note('Ephemerate', 'Full', 'blinks your best enters-the-battlefield creature at the end of an opponent\'s turn (or in '
     'answer to removal), rebounding next upkeep')
note('Restoration Angel', 'Full', 'flash 3/4 flier: cast at the end of an opponent\'s turn, blinking your best ETB '
     'non-Angel creature, or in answer to removal')
