"""Utility lands for the opponent pools: creature lands, channel lands, sacrifice-to-draw lands, land destruction,
graveyard lands and the rest. Lands are not permanents in p.perms, so their abilities are offered through the
'land_options' event (brain.hook_options) and upkeep through 'land_upkeep' (cardimpl.turn_start).

Creature lands become a creature (a Perm standing in for the land) until end of turn: the land leaves p.lands while
animated and comes back at the start of the next turn; if the creature died, the land goes to the graveyard.
"""
from engine import *
import engine as E
import cardimpl as CI
from cardimpl import on
from pool_cards import card, note
import impl_common as IC
from impl_common import best_opp_nonland
from impl_fixes import destroy_land, best_opp_land

BASICS = ('Plains', 'Island', 'Swamp', 'Mountain', 'Forest', 'Wastes')


def land_options(g, p, s, post):
    """activated abilities of p's lands (pool games)"""
    o = []
    for L in list(p.lands):
        h = CI.HOOKS.get(L.cd.name)
        if h and 'land_options' in h and CI.live(L.cd.name):
            o += h['land_options'](g, L, p, s, post) or []
    return o


def land_upkeep(g, p):
    for L in list(p.lands):
        h = CI.HOOKS.get(L.cd.name)
        if h and 'land_upkeep' in h and CI.live(L.cd.name): h['land_upkeep'](g, L, p)


def pay_without(g, p, L, gen, pips):
    """pay a land's activation cost with other sources (the land itself stays untapped)"""
    if L not in p.lands: return False
    i = p.lands.index(L); p.lands.remove(L)
    try:
        if not can_pay(g, p, gen, pips): return False
        pay(g, p, gen, pips); return True
    finally:
        p.lands.insert(i, L)


def can_pay_without(g, p, L, gen, pips):
    if L not in p.lands: return False
    i = p.lands.index(L); p.lands.remove(L)
    try: return can_pay(g, p, gen, pips)
    finally: p.lands.insert(i, L)


def sac_land(g, p, L):
    if L in p.lands: p.lands.remove(L); p.gy.append(L.cd)
    if g.hooks: CI.fire(g, 'land_gy', p, L.cd)


# ------------------------------------------------------------------ creature lands
def revert_animated(g):
    """start of a turn: animated lands stop being creatures (or went to the graveyard if the creature died)"""
    for p, m, L in getattr(g, 'animated', []):
        if m in p.perms:
            p.perms.remove(m); L.tapped = m.tapped
            if m.plus and L.cd.name == 'Raging Ravine': L_counters(L, m.plus)
            p.lands.append(L)
        elif m.owner is p and p.alive:
            p.gy.append(L.cd)
    g.animated = []


def L_counters(L, n=None):
    """+1/+1 counters kept on a creature land (Raging Ravine) between animations"""
    if n is not None:
        if L.data is None: L.data = {}
        L.data['counters'] = n
    return (L.data or {}).get('counters', 0)


def animate(g, p, L, pw, tg, kws=(), fly=False):
    p.lands.remove(L)
    m = Perm(p, None, pw=pw, tg=tg, fly=fly, name=L.cd.name)
    m.tapped = L.tapped
    m.sick = p.land_turn == p.turns and L.cd is getattr(p, 'last_land_cd', None)
    m.dt = 'deathtouch' in kws; m.life = 'lifelink' in kws
    m.ttypes = frozenset(('land',))
    m.plus = L_counters(L)
    m.data = {'land': L}
    if kws: g.eot_kw.setdefault(id(m), set()).update(kws)
    p.perms.append(m)
    if not hasattr(g, 'animated'): g.animated = []
    g.animated.append((p, m, L))
    log(f'  {NAME(p)} animates {L.cd.name}', g)
    return m


def open_attack(g, p, power, evasive=False):
    """is there an opponent this creature could hit (no untapped blocker that stops it)?"""
    for q in g.opps(p):
        blockers = [b for b in q.perms if b.creature and not b.tapped and not b.phased]
        if evasive or not blockers or max(E.etgh(g, b) for b in blockers) < power: return True
    return False


def manland(name, gen, pips, pw, tg, kws=(), fly=False, extra=None, status_text=''):
    @on(name, 'land_options')
    def _m(g, L, p, s, post):
        if post is not False or L.tapped or not can_pay_without(g, p, L, gen, pips): return []
        if p.land_turn == p.turns and L is p.lands[-1]: return []           # just played: it would be summoning sick
        spare = total_mana(g, p) - (gen + len(pips)) - 1
        evasive = fly or 'menace' in kws
        if not open_attack(g, p, pw + L_counters(L), evasive): return []
        u = 0.8 + 0.35 * (pw + L_counters(L)) + (0.5 if spare >= 2 else -0.8)

        def go():
            if L not in p.lands or L.tapped or not pay_without(g, p, L, gen, pips): return False
            m = animate(g, p, L, pw, tg, kws, fly)
            if extra: extra(g, p, m)
            return True
        return [(u, f'animate {name}', go)]
    note(name, 'Full', status_text or f'{{{gen}}}{pips}: {pw}/{tg}' + (f' {", ".join(kws)}' if kws else '') +
         ' until end of turn (animated to attack when it has an open attack)')


manland('Mutavault', 1, '', 2, 2, status_text='{1}: 2/2 with all creature types until end of turn (attacks when open)')
manland('Hissing Quagmire', 1, 'BG', 2, 2, kws=('deathtouch',))
manland('Needle Spires', 2, 'RW', 2, 1, kws=('double strike',))
manland('Raging Ravine', 2, 'RG', 3, 3,
        status_text='{2}{R}{G}: 3/3 until end of turn; a +1/+1 counter each time it attacks (kept on the land)')
manland('Hive of the Eye Tyrant', 3, 'B', 3, 3, kws=('menace',),
        status_text='{3}{B}: 3/3 menace until end of turn; exiles a card from the defender\'s graveyard when it attacks')


# ------------------------------------------------------------------ sacrifice-to-draw lands and pain lands
def _pain(name):
    CI.ON_TAP[name] = lambda g, p, L, used: lose_life(g, p, 1, p) if isinstance(L, Land) else None


def cycler_land(name, cost, pain=True):
    """{cost}, {T}, sacrifice: draw a card; mana costs 1 life"""
    if pain: _pain(name)

    @on(name, 'land_options')
    def _c(g, L, p, s, post):
        if L.tapped or not can_pay_without(g, p, L, cost, ''): return []
        lands = len(p.lands) + sum(1 for c in p.hand if c.land)
        if lands < 7 and not (post is None and lands >= 5): return []

        def go():
            if L not in p.lands or L.tapped or not pay_without(g, p, L, cost, ''): return False
            sac_land(g, p, L); draw(g, p, 1)
            log(f'  {NAME(p)} sacrifices {name}: draws a card', g); return True
        return [(1.2 if post is None else 0.6, f'{name} draw', go)]
    note(name, 'Full', ('mana costs 1 life; ' if pain else '') + f'{{{cost}}}, T, sacrifice: draw (used when flooded)')


cycler_land('Horizon Canopy', 1)
cycler_land('Silent Clearing', 1)
cycler_land('Sunbaked Canyon', 1)
cycler_land('Waterlogged Grove', 1)


# ------------------------------------------------------------------ channel lands (from hand)
@on('Boseiju, Who Endures', 'hand_options')
def _boseiju(g, c, p, s, post):
    """channel {1}{G}: destroy an opponent's artifact, enchantment or nonbasic land (they may search for a basic)"""
    if not can_pay(g, p, 1, 'G'): return []
    t = best_opp_nonland(g, p, lambda m: m.cd is not None and not m.creature and ('A' in m.cd.types or 'E' in m.cd.types))
    if t is None or pval(g, t) < 4: return []

    def go():
        if c not in p.hand or t not in t.owner.perms or not can_pay(g, p, 1, 'G'): return False
        p.hand.remove(c); pay(g, p, 1, 'G'); p.gy.append(c)
        log(f'  {NAME(p)} channels Boseiju', g); apply_removal(g, p, t, 'destroy'); return True
    return [(pval(g, t) - 2.0, 'channel Boseiju', go)]
note('Boseiju, Who Endures', 'Full', 'channel {1}{G} from hand: destroys a valuable artifact or enchantment')


@on('Sokenzan, Crucible of Defiance', 'hand_options')
def _sokenzan(g, c, p, s, post):
    """channel {3}{R}: two 1/1 colorless Spirit tokens with haste (cheaper per legendary creature)"""
    leg = sum(1 for m in p.perms if m.creature and m.cd is not None and 'leg' in m.cd.tags)
    cost = max(0, 3 - leg)
    lands = len(p.lands) + sum(1 for x in p.hand if x.land)
    if post is not False or not can_pay(g, p, cost, 'R') or lands < 6: return []

    def go():
        if c not in p.hand or not can_pay(g, p, cost, 'R'): return False
        p.hand.remove(c); pay(g, p, cost, 'R'); p.gy.append(c)
        make_tokens(g, p, 2, 1, sick=False, types=('spirit',))
        log(f'  {NAME(p)} channels Sokenzan', g); return True
    return [(1.5, 'channel Sokenzan', go)]
note('Sokenzan, Crucible of Defiance', 'Full', 'channel {3}{R} (less per legendary creature): two hasty 1/1s, when lands are plentiful')


# ------------------------------------------------------------------ land destruction lands
KEY_LANDS = ("Gaea's Cradle", 'Cabal Coffers', 'Urborg, Tomb of Yawgmoth', 'Ancient Tomb', "Urza's Saga", 'Nykthos, Shrine to Nyx',
             'Serra\'s Sanctum', 'Itlimoc, Cradle of the Sun', 'Tolarian Academy', 'Dark Depths', 'Field of the Dead',
             "Mishra's Workshop", 'Glacial Chasm', 'Maze of Ith', 'Kor Haven', 'Gavony Township')


def best_key_land(g, p, nonbasic=True):
    cands = [(q, L) for q in g.opps(p) for L in q.lands if L.cd.name not in BASICS]
    if not cands: return None
    key = [x for x in cands if x[1].cd.name in KEY_LANDS or int(x[1].cd.tags.get('amt', 1)) >= 2]
    return max(key, key=lambda x: threat(g, p, x[0])) if key else None


def _ld_land(name, cost, cond, status_text):
    @on(name, 'land_options')
    def _l(g, L, p, s, post):
        if L.tapped or not can_pay_without(g, p, L, cost, ''): return []
        x = best_key_land(g, p)
        if x is None or not cond(g, p, x[0]): return []

        def go():
            if L not in p.lands or L.tapped or not pay_without(g, p, L, cost, ''): return False
            q, T_ = x
            if T_ not in q.lands: return False
            sac_land(g, p, L); destroy_land(g, q, T_)
            if name == 'Ghost Quarter': land_ramp(g, q, 1, True)
            return True
        return [(2.5, f'{name} -> {x[1].cd.name}', go)]
    note(name, 'Full', status_text)


_ld_land('Ghost Quarter', 0, lambda g, p, q: True, 'T, sacrifice: destroys a key opposing land (its controller fetches a basic)')
_ld_land('Tectonic Edge', 1, lambda g, p, q: len(q.lands) >= 4,
         '{1}, T, sacrifice: destroys a key nonbasic land of an opponent with four or more lands')


# ------------------------------------------------------------------ mana and value lands
@on('Castle Locthwain', 'land_options')
def _locthwain(g, L, p, s, post):
    """{1}{B}{B}, {T}: draw a card, then lose life equal to the cards in hand (end of an opponent's turn)"""
    if post is not None or L.tapped or not can_pay_without(g, p, L, 1, 'BB'): return []
    if p.life - (len(p.hand) + 1) < 15: return []

    def go():
        if L not in p.lands or L.tapped or not pay_without(g, p, L, 1, 'BB'): return False
        L.tapped = True; draw(g, p, 1); lose_life(g, p, len(p.hand), p)
        log(f'  {NAME(p)} uses Castle Locthwain', g); return True
    return [(1.5, 'Castle Locthwain', go)]
note('Castle Locthwain', 'Full', 'enters tapped without another land early; draws at end of turn when life allows')


@on('Castle Embereth', 'land_options')
def _embereth(g, L, p, s, post):
    """{1}{R}{R}, {T}: creatures you control get +1/+0 until end of turn (before a wide attack)"""
    if post is not False or L.tapped or not can_pay_without(g, p, L, 1, 'RR'): return []
    ready = [m for m in p.perms if m.creature and not m.sick and not m.tapped and not m.noatk]
    if len(ready) < 4: return []

    def go():
        if L not in p.lands or L.tapped or not pay_without(g, p, L, 1, 'RR'): return False
        L.tapped = True
        for m in p.perms:
            if m.creature: CI._eot(g, m, 1, 0)
        log(f'  {NAME(p)} uses Castle Embereth', g); return True
    return [(0.5 + 0.35 * len(ready), 'Castle Embereth', go)]
note('Castle Embereth', 'Full', '{1}{R}{R},{T}: creatures +1/+0 before an attack with four or more')


@on('War Room', 'land_options')
def _war_room(g, L, p, s, post):
    """{3}, {T}, pay life equal to the colors in your commander's identity: draw a card"""
    n = len(set(p.ident) & set('WUBRG'))
    if post is not None or L.tapped or not can_pay_without(g, p, L, 3, '') or p.life - n < 12: return []

    def go():
        if L not in p.lands or L.tapped or not pay_without(g, p, L, 3, ''): return False
        L.tapped = True; lose_life(g, p, n, p); draw(g, p, 1)
        log(f'  {NAME(p)} uses War Room', g); return True
    return [(1.3, 'War Room', go)]
note('War Room', 'Full', '{3},{T}, life = commander colors: draw (at end of an opponent\'s turn)')


@on("Karn's Bastion", 'land_options')
def _bastion(g, L, p, s, post):
    """{4}, {T}: proliferate"""
    if post is not None or L.tapped or not can_pay_without(g, p, L, 4, ''): return []
    worth = sum(1 for m in p.perms if (m.plus > 0 or m.loyalty)) - sum(1 for q in g.opps(p) for m in q.perms if m.plus > 0)
    if worth < 2: return []

    def go():
        if L not in p.lands or L.tapped or not pay_without(g, p, L, 4, ''): return False
        L.tapped = True; IC.proliferate(g, p); return True
    return [(0.8 + 0.3 * worth, "Karn's Bastion", go)]
note("Karn's Bastion", 'Full', '{4},{T}: proliferate (at end of turn, when it helps)')


@on('Academy Ruins', 'land_options')
def _ruins(g, L, p, s, post):
    """{1}{U}, {T}: put an artifact card from your graveyard on top of your library"""
    if post is not None or L.tapped or not can_pay_without(g, p, L, 1, 'U'): return []
    arts = [c for c in p.gy if 'A' in c.types and not c.land]
    if not arts: return []
    c = max(arts, key=lambda c: (card_worth(g, p, c), c.cmc))
    if card_worth(g, p, c) < 40: return []

    def go():
        if L not in p.lands or L.tapped or c not in p.gy or not pay_without(g, p, L, 1, 'U'): return False
        L.tapped = True; p.gy.remove(c); p.library.append(c)
        log(f'  {NAME(p)} puts {c.name} on top with Academy Ruins', g); return True
    return [(1.5, 'Academy Ruins', go)]
note('Academy Ruins', 'Full', '{1}{U},{T}: the best artifact from the graveyard on top of the library')


@on('Kor Haven', 'land_defend')
def _kor_haven(g, L, d, p, atk, assign):
    """{1}{W}, {T}: prevent all combat damage the biggest unblocked attacker would deal"""
    if L.tapped or not can_pay_without(g, d, L, 1, 'W'): return
    unb = [a for a in atk if a not in assign and a in p.perms]
    if not unb: return
    a = max(unb, key=lambda m: epow(g, m))
    if epow(g, a) < 4: return
    if not pay_without(g, d, L, 1, 'W'): return
    L.tapped = True; atk.remove(a)
    log(f'    {NAME(d)} uses Kor Haven on {a.name}', g)
note('Kor Haven', 'Full', '{1}{W},{T}: prevents the combat damage of the biggest unblocked attacker (4+ power)')


@on('Vault of the Archangel', 'land_options')
def _vault(g, L, p, s, post):
    """{2}{W}{B}, {T}: creatures you control gain deathtouch and lifelink until end of turn (before an attack)"""
    if post is not False or L.tapped or not can_pay_without(g, p, L, 2, 'WB'): return []
    ready = [m for m in p.perms if m.creature and not m.sick and not m.tapped and not m.noatk]
    if len(ready) < 3: return []

    def go():
        if L not in p.lands or L.tapped or not pay_without(g, p, L, 2, 'WB'): return False
        L.tapped = True
        for m in p.perms:
            if m.creature: g.eot_kw.setdefault(id(m), set()).update(('deathtouch', 'lifelink'))
        log(f'  {NAME(p)} uses Vault of the Archangel', g); return True
    return [(0.3 * len(ready), 'Vault of the Archangel', go)]
note('Vault of the Archangel', 'Full', 'deathtouch and lifelink for the team before an attack with three or more')


@on('Pendelhaven', 'land_options')
def _pendelhaven(g, L, p, s, post):
    """{T}: target 1/1 creature gets +1/+2 until end of turn"""
    if post is not False or L.tapped: return []
    ones = [m for m in p.perms if m.creature and E.epow(g, m) == 1 and E.etgh(g, m) == 1 and not m.sick and not m.tapped]
    if not ones or total_mana(g, p) >= 6: return []

    def go():
        if L not in p.lands or L.tapped: return False
        L.tapped = True; CI._eot(g, ones[0], 1, 2); return True
    return [(0.4, 'Pendelhaven', go)]
note('Pendelhaven', 'Full', 'taps for {G}; {T}: a 1/1 attacker gets +1/+2 when the mana is not needed')


@on('Oran-Rief, the Vastwood', 'land_options')
def _oran(g, L, p, s, post):
    """{T}: a +1/+1 counter on each green creature that entered this turn (end of your main phase)"""
    if post is not True or L.tapped: return []
    new = [m for m in p.perms if m.creature and m.sick and m.cd is not None and 'G' in m.cd.pips]
    if len(new) < 2: return []

    def go():
        if L not in p.lands or L.tapped: return False
        L.tapped = True
        for m in new: m.plus += 1
        return True
    return [(0.4 * len(new), 'Oran-Rief', go)]
note('Oran-Rief, the Vastwood', 'Full', 'enters tapped; {T}: +1/+1 counters on green creatures that entered this turn')


@on('Yavimaya Hollow', 'regenerate')
def _hollow(g, L, p, m):
    """{G}, {T}: regenerate target creature (saves a valuable creature from destruction)"""
    if L.tapped or pval(g, m) < 4 or not can_pay_without(g, p, L, 0, 'G'): return False
    if not pay_without(g, p, L, 0, 'G'): return False
    L.tapped = True; m.tapped = True
    log(f'    {NAME(p)} regenerates {m.name} with Yavimaya Hollow', g)
    return True
note('Yavimaya Hollow', 'Full', '{G},{T}: regenerates a valuable creature that would be destroyed')


def try_regenerate(g, m):
    """a land that regenerates (Yavimaya Hollow) saves m from destruction"""
    p = m.owner
    for L in p.lands:
        h = CI.HOOKS.get(L.cd.name)
        if h and 'regenerate' in h and CI.live(L.cd.name) and h['regenerate'](g, L, p, m): return True
    return False


# ------------------------------------------------------------------ enter-the-battlefield lands
def _mortuary(g, p, L):
    cs = [c for c in p.gy if c.creature]
    if cs:
        c = max(cs, key=lambda c: (c.bomb, card_worth(g, p, c)))
        p.gy.remove(c); p.library.append(c); log(f'    Mortuary Mire puts {c.name} on top', g)
CI.LAND_ETB['Mortuary Mire'] = _mortuary
note('Mortuary Mire', 'Full', 'enters tapped; the best creature card from the graveyard on top of the library')


def _sanctuary(g, p, L):
    islands = sum(1 for x in p.lands if x is not L and (x.cd.name == 'Island' or 'island' in getattr(x.cd, 'subtypes', ())))
    if islands >= 3:
        L.tapped = False
        cs = [c for c in p.gy if c.instant or c.sorcery]
        if cs:
            c = max(cs, key=lambda c: card_worth(g, p, c))
            p.gy.remove(c); p.library.append(c); log(f'    Mystic Sanctuary puts {c.name} on top', g)
CI.LAND_ETB['Mystic Sanctuary'] = _sanctuary
card('Mystic Sanctuary', 'c=U t', types='L')
note('Mystic Sanctuary', 'Full', 'enters untapped with three other Islands, then puts the best instant or sorcery from '
     'the graveyard on top')


card('Simic Growth Chamber', 'amt=2 c=UG t bounceland', types='L')
note('Simic Growth Chamber', 'Full', 'enters tapped, returns a land to hand, taps for {G}{U}')
card('Reliquary Tower', 'c=C nomax', types='L')
note('Reliquary Tower', 'Full', 'no maximum hand size')


# ------------------------------------------------------------------ hideaway: Mosswort Bridge
def _mosswort(g, p, L):
    top = [p.library.pop() for _ in range(min(4, len(p.library)))]
    if not top: return
    import impl_topdeck
    c = max(top, key=lambda c: (not c.land, c.cmc if c.creature else card_worth(g, p, c) / 20))
    top.remove(c); p.library[:0] = top
    if L.data is None: L.data = {}
    L.data['hidden'] = c
    log(f'    Mosswort Bridge hides a card', g)
CI.LAND_ETB['Mosswort Bridge'] = _mosswort


@on('Mosswort Bridge', 'land_options')
def _mosswort_play(g, L, p, s, post):
    """{G}, {T}: play the hidden card free if creatures you control have total power 10 or more"""
    c = (L.data or {}).get('hidden')
    if c is None or post is None or L.tapped or not can_pay_without(g, p, L, 0, 'G'): return []
    if sum(epow(g, m) for m in p.perms if m.creature) < 10: return []

    def go():
        if L not in p.lands or L.tapped or not pay_without(g, p, L, 0, 'G'): return False
        L.tapped = True; L.data['hidden'] = None
        if c.land: play_land_card_free(g, p, c)
        else: cast_card(g, p, c, 'lib', {})
        return True
    return [(1.0 + c.cmc / 3.0, f'Mosswort Bridge ({c.name})', go)]
note('Mosswort Bridge', 'Full', 'hideaway 4 (keeps the best spell); {G},{T}: casts it free with 10 power on board')


def play_land_card_free(g, p, c):
    p.lands.append(Land(c, True)); log(f'  {NAME(p)} puts {c.name} onto the battlefield', g)


# ------------------------------------------------------------------ Emeria, the Sky Ruin
@on('Emeria, the Sky Ruin', 'land_upkeep')
def _emeria(g, L, p):
    plains = sum(1 for x in p.lands if x.cd.name == 'Plains' or 'plains' in getattr(x.cd, 'subtypes', ()))
    if plains < 7: return
    cs = [c for c in p.gy if c.creature]
    if cs:
        c = max(cs, key=lambda c: (c.bomb, card_worth(g, p, c)))
        p.gy.remove(c); enter(g, p, c); log(f'    Emeria returns {c.name}', g)
note('Emeria, the Sky Ruin', 'Full', 'enters tapped; with seven Plains, each upkeep returns the best creature card')


# ------------------------------------------------------------------ Tolaria West: transmute
@on('Tolaria West', 'hand_options')
def _tolaria(g, c, p, s, post):
    """transmute {1}{U}{U}: discard it to tutor a card with mana value 0 (a combo artifact or a key land)"""
    if post is not False or not can_pay(g, p, 1, 'UU'): return []
    import pool_ai
    wish = pool_ai.wish_list(g, p)
    zero = [x for x in searchable(g, p) if x.cmc == 0]
    if not zero: return []
    t = next((x for x in zero if x.name in wish), None) or max(zero, key=lambda x: (x.name in KEY_LANDS, card_worth(g, p, x)))
    if t.name not in wish and not (t.land and t.name in KEY_LANDS) and card_worth(g, p, t) < 45: return []

    def go():
        if c not in p.hand or t not in p.library or not can_pay(g, p, 1, 'UU'): return False
        p.hand.remove(c); pay(g, p, 1, 'UU'); p.gy.append(c)
        p.library.remove(t); p.hand.append(t); g.rng.shuffle(p.library)
        log(f'  {NAME(p)} transmutes Tolaria West for {t.name}', g); return True
    return [(2.5 if t.name in wish else 1.5, f'transmute Tolaria West ({t.name})', go)]
note('Tolaria West', 'Full', 'enters tapped; transmute {1}{U}{U} for a mana-value-0 combo piece or key land')


# ------------------------------------------------------------------ Dakmor Salvage: dredge 2
def dakmor_dredge(g, p):
    c = next((x for x in p.gy if x.name == 'Dakmor Salvage'), None)
    if c is None or len(p.library) < 25 or sum(1 for x in p.hand if x.land) >= 1: return False
    mill(g, p, 2); p.gy.remove(c); p.hand.append(c)
    log(f'  {NAME(p)} dredges Dakmor Salvage', g)
    return True
note('Dakmor Salvage', 'Full', 'enters tapped; dredge 2 instead of drawing when out of lands in hand')
