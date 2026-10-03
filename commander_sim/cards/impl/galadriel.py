"""Galadriel, Light of Valinor (your Bant Rebels deck, key `galadriel`, decklists/mine/galadriel-bant-rebels.md).

- Galadriel's Alliance trigger: each other creature entering picks a mode not yet chosen this turn ({G}{G}{G}, a
  +1/+1 counter on each creature you control, or scry 2 and draw).
- The Rebel chain: Ramosian Sergeant, Lieutenant, Captain and Commander, Defiant Vanguard and Lin Sivvi search for a
  Rebel permanent card with a mana value cap and put it onto the battlefield; Ramosian Revivalist returns one from the
  graveyard. Maskwood Nexus makes every creature card you own a Rebel (and every other type).
- Creature types: Kindred Discovery, Door of Destinies, Patchwork Banner, Vanquisher's Banner and Secluded Courtyard
  name a type as they enter.
- The other cards that need code: Panharmonicon, Abduction, Bribery, Crackdown, Mangara, Tocasia's Welcome, Voice of
  Resurgence, Eerie Interlude, Flicker, Planar Genesis, Return to Dust, Unbreakable Formation, Lawbringer,
  Lightbringer, Ballista Squad, Errant Doomsayers, Whipcorder, Knight of the Holy Nimbus, Cho-Manno, Amrou Seekers,
  Elvish Archdruid, Springleaf Drum, Grand Coliseum.
- The AI: cast priorities, which Rebel to fetch, protection and wipe responses, answers when attacked.
"""
import collections
import importlib
from commander_sim.engine import *
from commander_sim import engine as E
from commander_sim.cards import cardimpl as CI
from commander_sim.cards.cardimpl import on
from commander_sim.cards.pool_cards import card, note
from commander_sim.cards.impl import common as IC


def full(name, text): note(name, 'Full', text)


def human(g, p):
    return E.human_choice(g, p)


def _legal():
    return importlib.import_module('commander_sim.play.legal')


def named(p, name):
    return next((m for m in p.perms if m.cd is not None and m.cd.name == name and not m.phased), None)


# ================================================================== creature types
def maskwood(p):
    """Maskwood Nexus: your creatures, creature spells and creature cards are every creature type"""
    return named(p, 'Maskwood Nexus') is not None


def card_is(p, c, t):
    """is card c (in p's library, hand or graveyard) of creature type t"""
    return t in c.subtypes or 'changeling' in c.kws or (c.creature and maskwood(p))


def type_counts(p):
    n = collections.Counter()
    zones = list(p.hand) + list(p.library) + list(p.gy) + [m.cd for m in p.perms if m.cd is not None and m.creature]
    if p.cmd is not None: zones.append(p.cmd)
    for c in zones:
        if c.creature:
            for t in c.subtypes: n[t] += 1
    return n


def choose_type(g, p, what):
    """the creature type a permanent names as it enters: you choose; the AI names its deck's main type (Orc for
    Sauron's Kindred Discovery)"""
    counts = type_counts(p)
    hc = human(g, p)
    if hc is not None:
        ts = [t for t, _ in counts.most_common(12)] or ['human']
        if p.key == 'sauron' and 'orc' not in ts: ts.insert(0, 'orc')
        k = hc.choose(g, p, 'choose', f'{what}: choose a creature type',
                      [f'{t.title()} ({counts[t]} of your creature cards)' for t in ts], cancel=None)
        return ts[k]
    if p.key == 'sauron': return 'orc'
    on_bf = collections.Counter(t for m in p.perms if m.creature for t in subtypes(m))
    return max(counts, key=lambda t: counts[t] + 2 * on_bf[t], default='human')


def ctype(m):
    return (m.data or {}).get('ctype')


def _name_type(g, p, m):
    m.data = dict(m.data or {}, ctype=choose_type(g, p, m.cd.name))
    log(f'    {m.cd.name} names {m.data["ctype"].title()}', g)
    g.selfpt = True                                          # the Banners and the Door change creatures' size


for _n in ('Kindred Discovery', 'Door of Destinies', 'Patchwork Banner', "Vanquisher's Banner"):
    CI.AS_ENTERS[_n] = _name_type


def typed_pt(g, m):
    """the Banners (+1/+1) and Door of Destinies (+1/+1 per charge counter) for creatures of the named type"""
    b = 0
    for x in m.owner.perms:
        if x.cd is None or x.phased or x.cd.name not in ('Patchwork Banner', "Vanquisher's Banner", 'Door of Destinies'):
            continue
        t = ctype(x)
        if t is None or not has_type(m, t): continue
        b += (x.data or {}).get('charge', 0) if x.cd.name == 'Door of Destinies' else 1
    return b, b


IC.CREATURE_PT.append(typed_pt)


@on('Door of Destinies', 'cast')
def _door(g, src, caster, c):
    t = ctype(src)
    if caster is not src.owner or t is None or not card_is(caster, c, t): return
    if not trigger_window(g, caster, src, 'a charge counter'): return
    src.data = dict(src.data or {}, charge=(src.data or {}).get('charge', 0) + 1)
card('Door of Destinies', '', types='A', dsl=[])
full('Door of Destinies', 'names a creature type as it enters; a charge counter for each spell of that type you cast; '
     'creatures of that type get +1/+1 per counter')


@on('Patchwork Banner', 'etb')
def _patchwork_live(g, src, p, m): pass                   # registers the card (its +1/+1 lives in typed_pt)
card('Patchwork Banner', 'rock=1:A', types='A', dsl=[])
full('Patchwork Banner', 'names a creature type as it enters: those creatures get +1/+1; {T}: one mana of any colour')


@on("Vanquisher's Banner", 'cast')
def _vanquisher(g, src, caster, c):
    t = ctype(src)
    if caster is not src.owner or t is None or not c.creature or not card_is(caster, c, t): return
    if trigger_window(g, caster, src, 'draw a card'): draw(g, caster, 1)
card("Vanquisher's Banner", '', types='A', dsl=[])
full("Vanquisher's Banner", 'names a creature type as it enters: those creatures get +1/+1, and you draw a card for '
     'each creature spell of that type you cast')


# Secluded Courtyard: any colour only for creature spells of the named type (and abilities of those creatures)
def _courtyard_cols(g, p, L):
    t = (L.data or {}).get('ctype')
    spell = E.PAY_FOR
    if t is not None and spell is not None and spell.creature and card_is(p, spell, t): return p.ident
    return ''


def _courtyard_etb(g, p, L):
    if L.data is None: L.data = {}
    L.data['ctype'] = choose_type(g, p, 'Secluded Courtyard')
    log(f'    Secluded Courtyard names {L.data["ctype"].title()}', g)


CI.LAND_COLS['Secluded Courtyard'] = _courtyard_cols
CI.LAND_ETB['Secluded Courtyard'] = _courtyard_etb


@on('Secluded Courtyard', 'etb')
def _courtyard_live(g, src, p, m): pass
full('Secluded Courtyard', 'names a creature type as it enters; {T}: {C}, or one mana of any colour spent on a '
     'creature spell of that type')

def shapeshifter(g, p):
    """Maskwood Nexus: a 2/2 blue Shapeshifter with changeling (every creature type)"""
    for x in make_tokens(g, p, 1, 2, 2, color='U', types=('shapeshifter',)):
        x.data = dict(x.data or {}, kws=('changeling',))
        x.ttypes = frozenset(x.ttypes) | E.ALL_TYPES | {'rebel', 'shapeshifter'}


@on('Maskwood Nexus', 'options')
def _maskwood_ai(g, src, p, s, post):
    """{3}, {T}: a Shapeshifter, with spare mana at an opponent's end step"""
    if p is not src.owner or src.tapped or src.phased or post is not None or g.active is p or not can_pay(g, p, 3, ''): return []

    def go():
        if src.tapped or not can_pay(g, p, 3, ''): return False
        pay(g, p, 3, ''); src.tapped = True
        log(f'  {NAME(p)} activates Maskwood Nexus', g)
        if ability_window(g, p, src, 'a 2/2 Shapeshifter'): shapeshifter(g, p)
        return True
    return [(1.6 + fetch_bonus(g, p), 'Maskwood Nexus: a Shapeshifter', go)]
card('Maskwood Nexus', '', types='A', dsl=[])
full('Maskwood Nexus', 'your creatures, creature spells and creature cards are every creature type (the Rebel '
     'searches find any creature, the Banners and the Door count them all); {3}, {T}: a 2/2 blue Shapeshifter with '
     'changeling')


# ================================================================== Galadriel, Light of Valinor
MODES = ('mana', 'counters', 'draw')
MODE_TEXT = {'mana': 'add {G}{G}{G}', 'counters': 'a +1/+1 counter on each creature you control',
             'draw': 'scry 2, then draw a card'}


def _chosen(g, src):
    a = (src.data or {}).get('alliance')
    return a[1] if a and a[0] == turn_stamp(g) else ()


def alliance_value(g, p, mode):
    cre = [m for m in p.perms if m.creature and not m.phased]
    if mode == 'counters':
        return 0.9 * len(cre) + (0.8 if g.active is p and g.step in ('start', 'main1') else 0.0)
    if mode == 'draw': return 2.6
    if g.active is not p or g.step not in ('main1', 'main2'): return 0.2
    have = total_mana(g, p)
    want = [c for c in p.hand if not c.land and c.cmc > have and c.cmc <= have + 3 and 'G' not in c.pips.replace('G', '', 3)]
    return 3.2 if want else 0.6


@on('Galadriel, Light of Valinor', 'etb')
def _alliance(g, src, p, m):
    """Alliance: whenever another creature you control enters, choose one that hasn't been chosen this turn"""
    o = src.owner
    if m is src or m.owner is not o or not m.creature or src not in o.perms or src.phased: return
    if len(_chosen(g, src)) >= len(MODES): return
    if not trigger_window(g, o, src, 'Alliance'): return
    left = [k for k in MODES if k not in _chosen(g, src)]
    if not left: return
    hc = human(g, o)
    if hc is not None:
        k = hc.choose(g, o, 'choose', f'Galadriel (Alliance, {m.name} entered): choose one',
                      [MODE_TEXT[x] for x in left], cancel=None)
        mode = left[k]
    else:
        mode = max(left, key=lambda x: alliance_value(g, o, x))
    src.data = dict(src.data or {}, alliance=(turn_stamp(g), tuple(_chosen(g, src)) + (mode,)))
    o.milestone.setdefault('alliance', o.turns)
    o.stats['alliance_' + mode] += 1
    log(f'    Galadriel (Alliance): {MODE_TEXT[mode]}', g)
    if mode == 'mana':
        if hc is not None: importlib.import_module('commander_sim.play.mana').pool_of(o).add('G', 3)
        else: o.floatG = getattr(o, 'floatG', 0) + 3
    elif mode == 'counters':
        for x in o.perms:
            if x.creature and not x.phased: CI.add_counters(g, x, 1)
    else:
        importlib.import_module('commander_sim.cards.impl.topdeck').scry(g, o, 2)
        draw(g, o, 1)
card('Galadriel, Light of Valinor', 'leg', dsl=[])
full('Galadriel, Light of Valinor', 'Alliance: each other creature entering under your control picks a mode not '
     'chosen this turn: {G}{G}{G}, a +1/+1 counter on each creature you control, or scry 2 and draw (you choose; '
     'the AI weighs them)')


# ================================================================== Panharmonicon
@on('Panharmonicon', 'trigger_copies')
def _panharmonicon(g, src, p, event, arg):
    if event != 'etb' or p is not src.owner or src.phased or arg is None: return 0
    return 1 if (arg.creature or (arg.cd is not None and 'A' in arg.cd.types)) else 0
card('Panharmonicon', '', types='A', dsl=[])
full("Cathars' Crusade", 'a +1/+1 counter on each creature you control whenever a creature enters under your control')
full('Panharmonicon', 'a triggered ability of your permanents that an artifact or creature entering sets off '
     'triggers an additional time (card code, the ability language, the engine\'s enters tags, Cathars\' Crusade)')


# ================================================================== the Rebel searchers
SEARCHERS = {'Ramosian Sergeant': (3, 2), 'Ramosian Lieutenant': (4, 3), 'Ramosian Captain': (5, 4),
             'Defiant Vanguard': (5, 4), 'Ramosian Commander': (6, 5)}
CHAIN = ('Lin Sivvi, Defiant Hero', 'Ramosian Commander', 'Ramosian Captain', 'Defiant Vanguard',
         'Ramosian Lieutenant', 'Ramosian Sergeant')


def rebel_cards(p, cards, cap):
    return [c for c in cards if c.perm and not c.land and c.cmc <= cap and card_is(p, c, 'rebel')]


def rebel_value(g, p, c):
    """how much the AI wants Rebel c on the battlefield now"""
    n = c.name
    on_bf = {m.cd.name for m in p.perms if m.cd is not None}
    searchers = sum(1 for m in p.perms if m.cd is not None and (m.cd.name in SEARCHERS or m.cd.name == CHAIN[0]))
    opp_cols = collections.Counter(x for q in g.opps(p) for m in q.perms if m.creature and pval(g, m) >= 3
                                   for x in colors_of(m))
    if n in on_bf and 'leg' in c.tags: return 0.0
    if n == 'Lin Sivvi, Defiant Hero': return 9.0
    if n in SEARCHERS:
        base = {'Ramosian Commander': 7.0, 'Ramosian Captain': 6.0, 'Defiant Vanguard': 5.5, 'Ramosian Lieutenant': 5.0,
                'Ramosian Sergeant': 3.5}[n]
        return base - 1.2 * max(0, searchers - 1) - (1.5 if n in on_bf else 0)
    if n == 'Lawbringer': return 6.5 if opp_cols['R'] else 1.5
    if n == 'Lightbringer': return 6.5 if opp_cols['B'] else 1.5
    if n == 'Nightwind Glider': return 3.5 + (1.0 if opp_cols['B'] else 0)
    if n == 'Thermal Glider': return 3.5 + (1.0 if opp_cols['R'] else 0)
    if n == 'Ramosian Revivalist':
        return 3.5 + (2.0 if rebel_cards(p, p.gy, 5) else 0)
    return {'Jhovall Queen': 5.0, 'Ballista Squad': 4.5, 'Cho-Manno, Revolutionary': 4.5, 'Whipcorder': 4.0,
            'Knight of the Holy Nimbus': 3.5, 'Amrou Seekers': 3.0, 'Errant Doomsayers': 3.0,
            'Mirror Entity': 4.5}.get(n, card_worth(g, p, c) / 12.0)


ENTER_PAYOFFS = ('Galadriel, Light of Valinor', "Cathars' Crusade", 'Welcoming Vampire', "Tocasia's Welcome",
                 'Mentor of the Meek', 'Kindred Discovery', 'Panharmonicon', "Vanquisher's Banner", 'Patchwork Banner',
                 'Door of Destinies', 'Elvish Archdruid')


def fetch_bonus(g, p):
    """what a creature entering is worth beyond itself here: each payoff on the battlefield"""
    return 0.6 * sum(1 for m in p.perms if m.cd is not None and m.cd.name in ENTER_PAYOFFS and not m.phased)


def fetch_u(g, p, v):
    """the AI's utility for putting a card worth v onto the battlefield for free: like casting it (a card more),
    plus the enters payoffs"""
    return 3.5 + 0.6 * v + fetch_bonus(g, p)


def best_rebel(g, p, cap, zone=None):
    cs = rebel_cards(p, searchable(g, p) if zone is None else zone, cap)
    return max(cs, key=lambda c: (rebel_value(g, p, c), c.cmc), default=None)


def put_rebel(g, p, src, cap, zone='library'):
    """search (or, from the graveyard, return) a Rebel permanent card with mana value cap or less onto the
    battlefield: you pick; the AI takes the most useful one"""
    pool = searchable(g, p) if zone == 'library' else p.gy
    cs = rebel_cards(p, pool, cap)
    hc = human(g, p)
    if hc is not None:
        if not cs:
            log(f'    {src}: no Rebel to find', g)
            if zone == 'library': g.rng.shuffle(p.library)
            return None
        k = hc.choose(g, p, 'search', f'{src}: put which Rebel (mana value {cap} or less) onto the battlefield?',
                      [f'{c.name} (mana value {c.cmc})' for c in cs], cancel='find nothing')
        c = cs[k] if k is not None else None
    else:
        c = max(cs, key=lambda c: (rebel_value(g, p, c), c.cmc), default=None)
    if c is None:
        if zone == 'library': g.rng.shuffle(p.library)
        return None
    (p.library if zone == 'library' else p.gy).remove(c)
    if zone == 'library': g.rng.shuffle(p.library)
    log(f'    {src} puts {c.name} onto the battlefield', g)
    p.stats['rebels_fetched'] += 1
    m = enter(g, p, c)
    return m


def _searcher_ok(g, p, src):
    return (p is src.owner and src in p.perms and not src.phased and not src.tapped and not src.sick
            and not ability_locked(g, src))


def ability_locked(g, m):
    return getattr(g, 'auras', None) is not None and g.auras and CI.locked(g, m, 'noact') or m.neutered


def _when_ok(g, p, post):
    """the AI's timing: your main phases, or an opponent's end step (post None)"""
    if post is None and g.active is p: return False
    return True


def _searcher(cost, cap):
    def opts(g, src, p, s, post):
        if not _searcher_ok(g, p, src) or not _when_ok(g, p, post) or not can_pay(g, p, cost, ''): return []
        c = best_rebel(g, p, cap)
        if c is None: return []
        v = rebel_value(g, p, c)

        def go():
            if not _searcher_ok(g, p, src) or not can_pay(g, p, cost, ''): return False
            pay(g, p, cost, ''); src.tapped = True
            log(f'  {NAME(p)} activates {src.cd.name}', g)
            if ability_window(g, p, src, f'search for a Rebel (mana value {cap} or less)'): put_rebel(g, p, src.cd.name, cap)
            return True
        return [(fetch_u(g, p, v), f'{src.cd.name}: fetch {c.name}', go)]
    return opts


for _n, (_cost, _cap) in SEARCHERS.items():
    CI.on(_n, 'options')(_searcher(_cost, _cap))
    card(_n, None, dsl=[])
    full(_n, f'{{{_cost}}}, {{T}}: search for a Rebel permanent card with mana value {_cap} or less and put it onto '
             f'the battlefield (the AI at an opponent\'s end step or in its second main phase)')


@on('Lin Sivvi, Defiant Hero', 'options')
def _lin(g, src, p, s, post):
    """{X}, {T}: a Rebel permanent card with mana value X or less onto the battlefield; {3}: a Rebel card from your
    graveyard to the bottom of your library"""
    out = []
    if _searcher_ok(g, p, src) and _when_ok(g, p, post):
        have = total_mana(g, p)
        c = best_rebel(g, p, have)
        if c is not None:
            x = c.cmc
            v = rebel_value(g, p, c)

            def go():
                if not _searcher_ok(g, p, src) or not can_pay(g, p, x, ''): return False
                pay(g, p, x, ''); src.tapped = True
                log(f'  {NAME(p)} activates Lin Sivvi, X = {x}', g)
                if ability_window(g, p, src, f'search for a Rebel (mana value {x} or less)'): put_rebel(g, p, 'Lin Sivvi', x)
                return True
            out.append((fetch_u(g, p, v), f'Lin Sivvi (X = {x}): fetch {c.name}', go))
    if p is src.owner and post is True and not ability_locked(g, src) and can_pay(g, p, 3, ''):
        gy = rebel_cards(p, p.gy, 99)
        if gy and not rebel_cards(p, p.library, 99):
            c2 = max(gy, key=lambda c: rebel_value(g, p, c))

            def back():
                if c2 not in p.gy or not can_pay(g, p, 3, ''): return False
                pay(g, p, 3, '')
                log(f'  {NAME(p)} activates Lin Sivvi: {c2.name} to the bottom of the library', g)
                if ability_window(g, p, src, f'put {c2.name} on the bottom of the library') and c2 in p.gy:
                    p.gy.remove(c2); p.library.insert(0, c2)
                return True
            out.append((0.4, f'Lin Sivvi: {c2.name} back to the library', back))
    return out
card('Lin Sivvi, Defiant Hero', None, dsl=[])
full('Lin Sivvi, Defiant Hero', '{X}, {T}: a Rebel permanent card with mana value X or less onto the battlefield; '
     '{3}: a Rebel card from your graveyard to the bottom of your library')


@on('Ramosian Revivalist', 'options')
def _revivalist(g, src, p, s, post):
    if not _searcher_ok(g, p, src) or not _when_ok(g, p, post) or not can_pay(g, p, 6, ''): return []
    c = best_rebel(g, p, 5, p.gy)
    if c is None: return []

    def go():
        if not _searcher_ok(g, p, src) or not can_pay(g, p, 6, ''): return False
        pay(g, p, 6, ''); src.tapped = True
        log(f'  {NAME(p)} activates Ramosian Revivalist', g)
        if ability_window(g, p, src, 'return a Rebel from your graveyard'): put_rebel(g, p, 'Ramosian Revivalist', 5, 'gy')
        return True
    return [(fetch_u(g, p, rebel_value(g, p, c)), f'Ramosian Revivalist: return {c.name}', go)]
card('Ramosian Revivalist', None, dsl=[])
full('Ramosian Revivalist', '{6}, {T}: return a Rebel permanent card with mana value 5 or less from your graveyard '
     'to the battlefield')


# the creatures whose tap abilities are worth more than their attack stay home (unless they grow to 3 power)
NOATK = tuple(SEARCHERS) + ('Lin Sivvi, Defiant Hero', 'Ramosian Revivalist', 'Ballista Squad', 'Errant Doomsayers',
                            'Whipcorder', 'Lawbringer', 'Lightbringer')


def _noatk():
    for n in NOATK:
        cd = E.DB.get(n)
        if cd is not None: cd.tags['noatk'] = True


importlib.import_module('commander_sim.cards.pool_cards').POST.append(_noatk)


# ================================================================== creatures with abilities
def _opp_creatures(g, p, pred=lambda m: True):
    return [m for q in g.opps(p) for m in q.perms if m.creature and not m.phased and not untargetable(g, m) and pred(m)]


def _bringer(colour):
    def opts(g, src, p, s, post):
        if not _searcher_ok(g, p, src) or post is False: return []
        tg = _opp_creatures(g, p, lambda m: colour in colors_of(m) and not protected_from(g, m, 'W'))
        if not tg: return []
        t = max(tg, key=lambda m: pval(g, m))
        if pval(g, t) < 3: return []

        def go():
            return bringer_use(g, p, src, t)
        return [(0.8 + 0.5 * pval(g, t), f'{src.cd.name}: exile {t.name}', go)]
    return opts


def bringer_use(g, p, src, t):
    if src not in p.perms or src.tapped or t not in t.owner.perms: return False
    src.tapped = True
    log(f'  {NAME(p)} sacrifices {src.cd.name}: exile {t.name}', g)
    die(g, src, 'sac')
    if ability_window(g, p, src.cd, f'exile {t.name}', imp=6, target=t) and t in t.owner.perms and not untargetable(g, t):
        apply_removal(g, p, t, 'exile')
    return True


CI.on('Lawbringer', 'options')(_bringer('R'))
CI.on('Lightbringer', 'options')(_bringer('B'))
card('Lawbringer', None, dsl=[]); card('Lightbringer', None, dsl=[])
full('Lawbringer', '{T}, sacrifice it: exile target red creature (the AI on a red threat, or an attacker)')
full('Lightbringer', '{T}, sacrifice it: exile target black creature (the AI on a black threat, or an attacker)')


def tapper_target(g, p, src):
    """an opposing creature this tapper can tap that would block your best attacker"""
    tough = 2 if src.cd.name == 'Errant Doomsayers' else 99
    t = IZ().guildmage_target(g, p)
    if t is not None and etgh(g, t) <= tough: return t
    return None


def IZ():
    return importlib.import_module('commander_sim.cards.impl.zur')


def _tapper(gen, pips):
    def opts(g, src, p, s, post):
        if not _searcher_ok(g, p, src) or post is not False or g.active is not p or not can_pay(g, p, gen, pips): return []
        t = tapper_target(g, p, src)
        if t is None: return []

        def go():
            return tapper_use(g, p, src, t, gen, pips)
        return [(1.0 + 0.5 * pval(g, t), f'{src.cd.name}: tap {t.name}', go)]
    return opts


def tapper_use(g, p, src, t, gen, pips):
    if not _searcher_ok(g, p, src) or t not in t.owner.perms or t.tapped or not can_pay(g, p, gen, pips): return False
    pay(g, p, gen, pips); src.tapped = True
    log(f'  {NAME(p)} activates {src.cd.name}: tap {t.name}', g)
    if ability_window(g, p, src, f'tap {t.name}', target=t) and t in t.owner.perms: t.tapped = True
    return True


TAPPERS = {'Errant Doomsayers': (0, ''), 'Whipcorder': (0, 'W')}
for _n, (_g, _pp) in TAPPERS.items():
    CI.on(_n, 'options')(_tapper(_g, _pp))
    card(_n, None, dsl=[])
full('Errant Doomsayers', '{T}: tap target creature with toughness 2 or less (your blockers\' way out of an attack; '
     'the attacker an opponent would use)')
full('Whipcorder', '{W}, {T}: tap target creature; morph {W} (practice mode: cast it face down for {3}, turn it '
     'face up for {W})')


def precombat(g, q):
    """q (the AI) at the beginning of another player's combat: tap their best attacker with a tapper"""
    a = g.active
    for src in [m for m in q.perms if m.cd is not None and m.cd.name in TAPPERS]:
        gen, pips = TAPPERS[src.cd.name]
        if not _searcher_ok(g, q, src) or not can_pay(g, q, gen, pips): continue
        tough = 2 if src.cd.name == 'Errant Doomsayers' else 99
        cands = [m for m in a.perms if m.creature and not m.tapped and not m.phased and not untargetable(g, m)
                 and etgh(g, m) <= tough and epow(g, m) >= 3 and (not m.sick or importlib.import_module('commander_sim.ais').has_haste(g, m))]
        if not cands: continue
        t = max(cands, key=lambda m: epow(g, m))
        tapper_use(g, q, src, t, gen, pips)


def attack_answers(g, d, p, atk):
    """d (the AI) is attacked: its permanents' answers [(value, fn)]: Ballista Squad, Lawbringer, Lightbringer"""
    out = []
    for src in list(d.perms):
        if src.cd is None or not _searcher_ok(g, d, src): continue
        n = src.cd.name
        if n == 'Ballista Squad':
            have = total_mana(g, d) - 1
            for a in atk:
                if a not in p.perms or untargetable(g, a) or protected_from(g, a, 'W') or no_damage(g, a): continue
                x = etgh(g, a)
                if x > have or indestructible(g, a): continue
                out.append((pval(g, a) + 0.3 * epow(g, a) - 0.4 * x, lambda a=a, x=x, src=src: ballista_use(g, d, src, a, x)))
        elif n in ('Lawbringer', 'Lightbringer'):
            col = 'R' if n == 'Lawbringer' else 'B'
            for a in atk:
                if a in p.perms and col in colors_of(a) and not untargetable(g, a) and not protected_from(g, a, 'W'):
                    out.append((pval(g, a) + 0.3 * epow(g, a) - 1.0, lambda a=a, src=src: bringer_use(g, d, src, a)))
    return out


def ballista_use(g, p, src, t, x):
    if not _searcher_ok(g, p, src) or t not in t.owner.perms or not can_pay(g, p, x, 'W'): return False
    pay(g, p, x, 'W'); src.tapped = True
    log(f'  {NAME(p)} activates Ballista Squad: {x} damage to {t.name}', g)
    if ability_window(g, p, src, f'{x} damage to {t.name}', imp=5, target=t) and t in t.owner.perms \
            and not no_damage(g, t) and etgh(g, t) <= x:
        apply_removal(g, p, t, f'dmg{x}')
    return True
card('Ballista Squad', None, dsl=[])
full('Ballista Squad', '{X}{W}, {T}: X damage to target attacking or blocking creature (the AI kills an attacker)')


# Defiant Vanguard: when it blocks, at end of combat, destroy it and the creature it blocked
def vanguard_blocks(g, assign):
    """after combat damage: each Defiant Vanguard that blocked destroys itself and the attacker"""
    for a, b in list(assign.items()):
        if b is None or b.cd is None or b.cd.name != 'Defiant Vanguard': continue
        if not trigger_window(g, b.owner, b, f'destroy it and {a.name}', imp=5): continue
        if a in a.owner.perms: die(g, a, 'destroy')
        if b in b.owner.perms: die(g, b, 'destroy')


CI.on('Defiant Vanguard', 'options')(_searcher(5, 4))
full('Defiant Vanguard', 'when it blocks, at end of combat, it and the creature it blocked are destroyed (the AI '
     'blocks big attackers with it); {5}, {T}: a Rebel permanent card with mana value 4 or less onto the battlefield')


# Knight of the Holy Nimbus: flanking; regenerates unless an opponent pays {2}
def nimbus_regen(g, m):
    """Knight of the Holy Nimbus would be destroyed: it regenerates unless an opponent paid {2} this turn"""
    p = m.owner
    if (m.data or {}).get('noregen') == turn_stamp(g): return False
    for q in [g.active] + g.after(g.active) if g.active is not None else g.opps(p):
        if q is p or not q.alive: continue
        hc = human(g, q)
        if hc is not None:
            if hc.pay_tax(g, q, 2, "Knight of the Holy Nimbus (it can't regenerate this turn)"):
                m.data = dict(m.data or {}, noregen=turn_stamp(g)); return False
        elif can_pay(g, q, 2, '') and pval(g, m) >= 1.5:
            pay(g, q, 2, '')
            log(f"    {NAME(q)} pays {{2}}: Knight of the Holy Nimbus can't regenerate", g)
            m.data = dict(m.data or {}, noregen=turn_stamp(g)); return False
    m.tapped = True
    g.eot_kw.setdefault(id(m), set()).add('regenerated')
    log('    Knight of the Holy Nimbus regenerates', g)
    return True


CI.SELF_REGEN['Knight of the Holy Nimbus'] = nimbus_regen
full('Knight of the Holy Nimbus', 'flanking (a blocker without flanking gets -1/-1); if it would be destroyed it '
     'regenerates, unless an opponent pays {2} (the AI pays when it can)')

card('Cho-Manno, Revolutionary', 'leg human nodmg pow=2 tgh=2', dsl=[])
full('Cho-Manno, Revolutionary', 'all damage dealt to it is prevented (combat, burn, damage wipes)')
full('Amrou Seekers', "can't be blocked except by artifact and/or white creatures")
full('Nightwind Glider', 'flying, protection from black')
full('Thermal Glider', 'flying, protection from red')
full('Jhovall Queen', 'vigilance')


# ================================================================== Mangara, Tocasia's Welcome, Voice of Resurgence
@on('Mangara, the Diplomat', 'attack')
def _mangara_attack(g, src, p, atk, d):
    o = src.owner
    if p is o or d is not o or len(atk) < 2: return
    if trigger_window(g, o, src, 'draw a card'): draw(g, o, 1)


@on('Mangara, the Diplomat', 'cast')
def _mangara_cast(g, src, caster, c):
    o = src.owner
    if caster is o or caster.spells_this_turn != 2: return
    if trigger_window(g, o, src, 'draw a card'): draw(g, o, 1)
card('Mangara, the Diplomat', None, dsl=[])
full('Mangara, the Diplomat', 'lifelink; draws when an opponent attacks you with two or more creatures, and when an '
     'opponent casts their second spell each turn')


@on("Tocasia's Welcome", 'etb')
def _tocasia(g, src, p, m):
    o = src.owner
    if m.owner is not o or not m.creature or m.cd is None or m.cd.cmc > 3: return
    k = f'tocasia{id(src)}'
    if o.flag_turn.get(k) == (g.round, g.active and g.active.key): return
    if not trigger_window(g, o, src, 'draw a card'): return
    if once_per_turn(g, o, k): draw(g, o, 1)
card("Tocasia's Welcome", '', types='E', dsl=[])
full("Tocasia's Welcome", 'once each turn, when one or more creatures with mana value 3 or less enter under your '
     'control, draw a card')


def _voice_token(g, p):
    g.selfpt = True
    for x in make_tokens(g, p, 1, 1, 1, color='GW', types=('elemental',)):
        x.data = dict(x.data or {}, voice=True)
    log(f'    {NAME(p)} creates an Elemental (its size is the number of creatures they control)', g)


def voice_pt(g, m):
    if not (m.data and m.data.get('voice')): return 0, 0
    n = sum(1 for x in m.owner.perms if x.creature and not x.phased)
    return n - 1, n - 1


IC.TOKEN_PT.append(voice_pt)


@on('Voice of Resurgence', 'cast')
def _voice_cast(g, src, caster, c):
    o = src.owner
    if caster is o or g.active is not o: return
    if trigger_window(g, o, src, 'an Elemental token'): _voice_token(g, o)


@on('Voice of Resurgence', 'self_dies')
def _voice_dies(g, m, cause):
    if trigger_window(g, m.owner, m, 'an Elemental token'): _voice_token(g, m.owner)
card('Voice of Resurgence', None, dsl=[])
full('Voice of Resurgence', 'an Elemental token (its power and toughness are the number of creatures you control) '
     'when an opponent casts a spell during your turn and when it dies')


# ================================================================== Crackdown
def crackdown_on(g):
    return any(m.cd is not None and m.cd.name == 'Crackdown' and not m.phased for q in g.players if q.alive for m in q.perms)


def crackdown_holds(g, m):
    """Crackdown: a nonwhite creature with power 3 or greater doesn't untap"""
    return m.creature and 'W' not in colors_of(m) and epow(g, m) >= 3


CI.crackdown_on = crackdown_on
CI.crackdown_holds = crackdown_holds


@on('Crackdown', 'etb')
def _crackdown_live(g, src, p, m): pass
card('Crackdown', '', types='E', dsl=[])
full('Crackdown', "nonwhite creatures with power 3 or greater don't untap during their controllers' untap steps")


# ================================================================== Abduction
@on('Abduction', 'etb')
def _abduction(g, src, p, m):
    """enchant creature: you control it; it untaps as the Aura enters; when it dies, it returns to its owner"""
    if m is not src: return
    IM = importlib.import_module('commander_sim.cards.impl.marchesa')
    hc = human(g, p)
    t = importlib.import_module('commander_sim.play.cards').pick_creature(
        g, p, 'Abduction: enchant (and gain control of) which creature?') if hc is not None else IM.best_steal(g, p, src.cd)
    if t is None or t.owner is p:
        if t is not None and t.owner is p:
            src.attached = t; _abduction_untap(g, p, src, t); return
        leave(g, src); to_zone_card(g, src, 'gy'); return
    from commander_sim import ais
    if ais.protect_response(g, t.owner, t, 'steal', p, src.cd) or t not in t.owner.perms:
        leave(g, src); to_zone_card(g, src, 'gy'); return
    IM.steal(g, p, t, until_eot=False)
    src.attached = t
    _abduction_untap(g, p, src, t)


def _abduction_untap(g, p, src, t):
    src.data = dict(src.data or {}, abducted=t)          # remembered: the creature is detached as it dies
    if trigger_window(g, p, src, f'untap {t.name}') and t in p.perms: t.tapped = False


@on('Abduction', 'dies')
def _abduction_dies(g, src, m, cause):
    """the enchanted creature died: that card returns to the battlefield under its owner's control"""
    if (src.data or {}).get('abducted') is not m: return
    owner = m.orig if m.orig is not None else m.owner
    if m.token or m.cd is None or not owner.alive: return
    if not trigger_window(g, src.owner, src, f'{m.name} returns to {NAME(owner)}'): return
    if m.cd in owner.gy:
        owner.gy.remove(m.cd); enter(g, owner, m.cd)
        log(f'    {m.name} returns to the battlefield under {NAME(owner)}\'s control (Abduction)', g)


@on('Abduction', 'leaves')
def _abduction_leaves(g, src):
    h = src.attached
    if h is not None and h in src.owner.perms and h.orig is not None and h.orig is not src.owner and h.orig.alive:
        src.owner.perms.remove(h); h.owner = h.orig; h.orig.perms.append(h); g.bf_ver = getattr(g, 'bf_ver', 0) + 1


@on('Abduction', 'sba')
def _abduction_sba(g, src):
    if src in src.owner.perms and (src.attached is None or src.attached not in src.attached.owner.perms):
        src.attached = None; leave(g, src); to_zone_card(g, src, 'gy')
card('Abduction', 'aura', types='E', dsl=[])
full('Abduction', 'steals the best opposing creature (untapped) while it stays attached; when it dies it returns to '
     'the battlefield under its owner\'s control; control returns if Abduction leaves')


# ================================================================== spells
@on('Bribery', 'resolve')
def _bribery(g, p, c, ctx):
    opps = [q for q in g.opps(p) if q.alive]
    if not opps: return 'gy'
    hc = human(g, p)
    if hc is not None:
        k = hc.choose(g, p, 'target', 'Bribery: search which opponent\'s library?', [NAME(q) for q in opps], cancel=None)
        q = opps[k]
        cs = sorted([x for x in q.library if x.creature], key=lambda x: (-x.cmc, x.name))
        pick = None
        if cs:
            j = hc.choose(g, p, 'search', f"Bribery: put which creature from {NAME(q)}'s library onto the battlefield under your control?",
                          [f'{x.name} (mana value {x.cmc})' for x in cs], cancel='find nothing')
            pick = cs[j] if j is not None else None
    else:
        q, pick = bribery_pick(g, p)
    if pick is not None:
        q.library.remove(pick)
        log(f'    Bribery: {NAME(p)} takes {pick.name} from {NAME(q)}\'s library', g)
        m = enter(g, p, pick, orig=q)
    g.rng.shuffle(q.library)
    return 'gy'


def bribery_pick(g, p):
    best = (None, None, -1.0)
    for q in g.opps(p):
        for x in q.library:
            if x.creature:
                v = x.bomb * 2 + x.cmc + (3 if 'fly' in x.tags else 0)
                if v > best[2]: best = (q, x, v)
    return (best[0] or (g.opps(p)[0] if g.opps(p) else None)), best[1]
card('Bribery', '', types='S', dsl=[])
full('Bribery', "the best creature in an opponent's library onto the battlefield under your control (you choose the "
     "opponent and the card)")


def eerie_return(g, p, cards):
    """Eerie Interlude: the cards come back at the beginning of the next end step"""
    if not hasattr(g, 'eot_returns') or g.eot_returns is None: g.eot_returns = []
    g.eot_returns += [(p, cd) for cd in cards]


def eot_returns(g):
    """the beginning of the end step: creatures exiled by Eerie Interlude return under their owner's control"""
    rs = getattr(g, 'eot_returns', None)
    if not rs: return
    g.eot_returns = []
    for p, cd in rs:
        if not p.alive or cd not in p.exile: continue
        p.exile.remove(cd)
        n = enter(g, p, cd)
        n.is_cmd = cd is p.cmd
        log(f'    {cd.name} returns to the battlefield (Eerie Interlude)', g)


CI.eot_returns = eot_returns


def interlude(g, p, ms):
    """exile your creatures ms; they return at the beginning of the next end step (tokens are gone for good)"""
    back = []
    for m in ms:
        if m not in p.perms: continue
        cd, tok = m.cd, m.token
        leave(g, m)
        if not tok and cd is not None:
            p.exile.append(cd); back.append(cd)
    eerie_return(g, p, back)
    log(f'    Eerie Interlude exiles {", ".join(c.name for c in back) or "nothing"} until the end step', g)


@on('Eerie Interlude', 'resolve')
def _interlude(g, p, c, ctx):
    ms = ctx.get('targets')
    if ms is None:
        hc = human(g, p)
        mine = [m for m in p.perms if m.creature and not m.phased]
        if hc is not None:
            ms = []
            while True:
                left = [m for m in mine if m not in ms]
                if not left: break
                k = hc.choose(g, p, 'target', f'Eerie Interlude: exile which of your creatures? ({len(ms)} so far)',
                              [hc.legal.describe_target(g, p, m) for m in left], cancel='done')
                if k is None: break
                ms.append(left[k])
        else:
            ms = [m for m in mine if not m.token and etb_worth(g, p, m) > 0]
    interlude(g, p, [m for m in ms if m in p.perms])
    return 'gy'
card('Eerie Interlude', '', types='I', dsl=[])
full('Eerie Interlude', 'exiles any of your creatures; they return at the beginning of the next end step (the AI '
     'saves its board from a wipe or a creature from removal)')


def etb_worth(g, p, m):
    """what blinking creature m gains: its enters effect, and Galadriel's Alliance"""
    v = 0.0
    if m.cd is not None and CI is not None: v += CI.card_etb_value(g, p, m.cd)
    if named(p, 'Galadriel, Light of Valinor') is not None and m.cd is not None and m.cd.name != 'Galadriel, Light of Valinor':
        v += 1.5
    return v


@on('Flicker', 'resolve')
def _flicker(g, p, c, ctx):
    t = ctx.get('target')
    hc = human(g, p)
    if t is None:
        cands = [m for q in g.players if q.alive for m in q.perms if not m.token and not m.phased and m.cd is not None
                 and not (m.owner is not p and untargetable(g, m))]
        if hc is not None:
            if not cands: return 'gy'
            k = hc.choose(g, p, 'target', 'Flicker: exile and return which nontoken permanent?',
                          [hc.legal.describe_target(g, p, m) for m in cands], cancel=None)
            t = cands[k]
        else:
            t = flicker_target(g, p)
    if t is None or t not in t.owner.perms: return 'gy'
    q, cd = t.owner, t.cd
    owner = t.orig if t.orig is not None else q
    leave(g, t)
    n = enter(g, owner, cd)
    n.is_cmd = cd is owner.cmd
    log(f'    Flicker: {cd.name} leaves and returns', g)
    return 'gy'


def flicker_target(g, p):
    mine = [m for m in p.perms if not m.token and m.creature and not m.phased and m.cd is not None]
    return max(mine, key=lambda m: etb_worth(g, p, m), default=None)
card('Flicker', '', types='S', dsl=[])
full('Flicker', 'exiles a nontoken permanent and returns it at once under its owner\'s control (the AI blinks its '
     'best enters-the-battlefield creature)')


@on('Planar Genesis', 'resolve')
def _genesis(g, p, c, ctx):
    top = [p.library.pop() for _ in range(min(4, len(p.library)))]
    hc = human(g, p)
    land = take = None
    lands = [x for x in top if x.land]
    if hc is not None:
        if lands:
            k = hc.choose(g, p, 'choose', f'Planar Genesis: put a land onto the battlefield tapped? (top four: {", ".join(x.name for x in top)})',
                          [x.name for x in lands], cancel='no land')
            land = lands[k] if k is not None else None
        if land is None and top:
            k = hc.choose(g, p, 'choose', 'Planar Genesis: put which card into your hand?', [x.name for x in top], cancel=None)
            take = top[k]
    else:
        if lands and len(p.lands) < 8: land = lands[0]
        else: take = max(top, key=lambda x: card_worth(g, p, x), default=None)
    rest = [x for x in top if x is not land and x is not take]
    g.rng.shuffle(rest)
    for x in rest: p.library.insert(0, x)
    if land is not None:                               # not the land drop: onto the battlefield tapped
        p.lands.append(E.Land(land, True))
        if land.name in CI.LAND_ETB and CI.live(land.name): CI.LAND_ETB[land.name](g, p, p.lands[-1])
        log(f'    Planar Genesis: {NAME(p)} puts {land.name} onto the battlefield tapped', g)
        landfall(g, p)
    if take is not None:
        p.hand.append(take); p.seen_names.add(take.name)
        log(f'    Planar Genesis: {NAME(p)} takes a card', g)
    return 'gy'


@on('Planar Genesis', 'hand_options')
def _genesis_eot(g, c, p, s, post):
    """at an opponent's end step"""
    if post is not None or g.active is p or c not in p.hand or not can_pay(g, p, 0, 'GU'): return []

    def go():
        if c not in p.hand or not can_pay(g, p, 0, 'GU'): return False
        pay(g, p, 0, 'GU'); p.hand.remove(c); p.spells_this_turn += 1
        cast_card(g, p, c, 'hand', {})
        return True
    return [(2.2, 'Planar Genesis', go)]
card('Planar Genesis', '', types='I', dsl=[])
full('Planar Genesis', 'top four: a land onto the battlefield tapped, or else a card into your hand; the rest to the '
     'bottom in a random order (the AI at an opponent\'s end step)')


@on('Return to Dust', 'resolve')
def _dust(g, p, c, ctx):
    t = ctx.get('target')
    if t is not None and t in t.owner.perms and not untargetable(g, t): apply_removal(g, p, t, 'exile', c)
    if g.active is not p or g.step not in ('main1', 'main2'): return 'gy'
    cands = [m for q in g.players if q.alive for m in q.perms if m is not t and m.cd is not None and not m.phased
             and ('A' in m.cd.types or 'E' in m.cd.types) and not m.creature and not (m.owner is not p and untargetable(g, m))]
    hc = human(g, p)
    if hc is not None:
        if not cands: return 'gy'
        k = hc.choose(g, p, 'target', 'Return to Dust (main phase): exile up to one other artifact or enchantment?',
                      [hc.legal.describe_target(g, p, m) for m in cands], cancel='no second target')
        t2 = cands[k] if k is not None else None
    else:
        theirs = [m for m in cands if m.owner is not p]
        t2 = max(theirs, key=lambda m: pval(g, m), default=None)
        if t2 is not None and pval(g, t2) < 1: t2 = None
    if t2 is not None: apply_removal(g, p, t2, 'exile', c)
    return 'gy'
card('Return to Dust', 'rem=exile tgt=ae', types='I', dsl=[])
full('Return to Dust', 'exile target artifact or enchantment; cast in your main phase, a second one too')


@on('Unbreakable Formation', 'resolve')
def _formation(g, p, c, ctx):
    formation(g, p, g.active is p and g.step in ('main1', 'main2'))
    return 'gy'


def formation(g, p, addendum):
    for m in p.perms:
        if m.creature and not m.phased:
            g.eot_kw.setdefault(id(m), set()).add('indestructible')
            if addendum:
                CI.add_counters(g, m, 1); g.eot_kw[id(m)].add('vigilance')
    log(f'    {NAME(p)}\'s creatures gain indestructible' + (', a +1/+1 counter and vigilance' if addendum else ''), g)
full('Unbreakable Formation', 'your creatures gain indestructible; cast in your main phase, also a +1/+1 counter and '
     'vigilance')


def make_a_stand(g, p):
    for m in p.perms:
        if m.creature and not m.phased: g.eot_kw.setdefault(id(m), set()).add('indestructible')
        if m.creature: g.eot_pt[id(m)] = tuple(a + b for a, b in zip(g.eot_pt.get(id(m), (0, 0)), (1, 0)))
    log(f"    {NAME(p)}'s creatures get +1/+0 and indestructible", g)


# ================================================================== mana
def _elves(p):
    return sum(1 for m in p.perms if m.creature and not m.phased and has_type(m, 'elf'))


@on('Elvish Archdruid', 'etb')
def _archdruid_live(g, src, p, m): pass
CI.DYN_MANA['Elvish Archdruid'] = lambda g, p, m: _elves(p)
full('Elvish Archdruid', 'other Elves you control get +1/+1; {T}: {G} for each Elf you control')


def _drum_creatures(p, drum):
    return [m for m in p.perms if m.creature and not m.tapped and not m.phased]


@on('Springleaf Drum', 'etb')
def _drum_live(g, src, p, m): pass
CI.DYN_MANA['Springleaf Drum'] = lambda g, p, m: 1 if _drum_creatures(p, m) else 0


def _drum_tap(g, p, m, used):
    """tap an untapped creature you control as part of the cost: you choose; the AI taps one that won't attack"""
    cs = _drum_creatures(p, m)
    if not cs: return
    hc = human(g, p)
    if hc is not None and len(cs) > 1:
        k = hc.choose(g, p, 'choose', 'Springleaf Drum: tap which creature?', [hc.legal.describe_target(g, p, x) for x in cs], cancel=None)
        x = cs[k]
    else:
        x = min(cs, key=lambda x: (not x.sick, not x.noatk, epow(g, x)))
    x.tapped = True
CI.ON_TAP['Springleaf Drum'] = _drum_tap
full('Springleaf Drum', '{T}, tap an untapped creature you control: one mana of any colour')

importlib.import_module('commander_sim.cards.impl.rules').set_tags('Grand Coliseum', add=('pain',))
full('Grand Coliseum', 'enters tapped; {T}: {C}, or one mana of any colour for 1 damage')


# ================================================================== the AI: cast priorities
def galadriel_prio(g, p, c):
    t = c.tags; n = c.name
    gal = named(p, 'Galadriel, Light of Valinor')
    cre = sum(1 for m in p.perms if m.creature and not m.phased)
    if c is p.cmd: return 84 if p.turns >= 3 else 66
    if 'ctr' in t or n in INDES or n in ('Eerie Interlude', 'Planar Genesis'): return 0     # held for their window
    if ('rem' in t or 'wipe' in t) and not c.perm: return 0       # removal and wipes: through their own decisions
    if 'rock' in t or 'dork' in t or n == 'Springleaf Drum': return 82 if p.turns <= 5 else 30
    if 'lr' in t or n in ('Farhaven Elf', 'Shared Roots'): return 78 if p.turns <= 5 else 25
    if n == 'Elvish Archdruid': return 76 if p.turns <= 5 else 45
    if n == 'Lin Sivvi, Defiant Hero': return 74
    if n in SEARCHERS:                                             # the engine: first, while none is out
        out = any(m.cd is not None and (m.cd.name in SEARCHERS or m.cd.name == 'Lin Sivvi, Defiant Hero') for m in p.perms)
        return {'Ramosian Commander': 66, 'Ramosian Captain': 64, 'Defiant Vanguard': 60,
                'Ramosian Lieutenant': 62, 'Ramosian Sergeant': 63}[n] + (0 if out else 10)
    if n in ("Cathars' Crusade", 'Panharmonicon'): return 72 if cre >= 2 or gal is not None else 50
    if n in ('Kindred Discovery', "Vanquisher's Banner", "Tocasia's Welcome", 'Beast Whisperer',
             'Welcoming Vampire', 'Mentor of the Meek'): return 70
    if n in ('Door of Destinies', 'Patchwork Banner', 'Maskwood Nexus'): return 58
    if n == 'Mangara, the Diplomat': return 56
    if n == 'Adeline, Resplendent Cathar': return 73
    if n == 'Recruiter of the Guard': return 65
    if n == 'Abduction':
        IM = importlib.import_module('commander_sim.cards.impl.marchesa')
        b = IM.best_steal(g, p, c)
        return min(80, int(40 + 6 * pval(g, b))) if b is not None and pval(g, b) >= 3 else 0
    if n == 'Bribery': return 68
    if n == 'Crackdown':
        theirs = sum(1 for q in g.opps(p) for m in q.perms if m.creature and crackdown_holds(g, m))
        mine = sum(1 for m in p.perms if m.creature and crackdown_holds(g, m))
        return 55 if theirs >= mine + 2 else 0
    if n == 'Flicker':
        b = flicker_target(g, p)
        return 45 if b is not None and etb_worth(g, p, b) >= 3 else 0
    if n == 'Shamanic Revelation': return 60 if cre >= 4 else 15
    if n == 'Elspeth, Sun\'s Champion': return 66
    if n == 'Reya Dawnbringer': return 50
    if 'tokatk' in t: return 70
    if c.creature: return 48
    if 'draw' in t and (c.instant or c.sorcery): return 46
    return 40


def galadriel_abilities(g, p):
    return False


# ================================================================== the AI: protection and wipes
def _cast_protect(g, owner, c):
    from commander_sim import ais
    if c.name == 'Unbreakable Formation':
        if not ais.pay_card(g, owner, c): return False
        formation(g, owner, False); return True
    if c.name == 'Make a Stand':
        if not ais.pay_card(g, owner, c): return False
        make_a_stand(g, owner); return True
    if c.name == 'Rootborn Defenses':
        if not ais.pay_card(g, owner, c): return False
        IZ().rootborn(g, owner); return True
    return False


INDES = ('Unbreakable Formation', 'Make a Stand', 'Rootborn Defenses')


def galadriel_protect(g, owner, m, kind, actor, spell):
    """removal at a key creature: indestructible (Unbreakable Formation, Make a Stand, Rootborn Defenses) against
    destroy and damage, or Eerie Interlude (it returns at the end step) against anything"""
    from commander_sim import ais
    if not m.creature or pval(g, m) < 4: return False
    destroyish = kind == 'destroy' or kind.startswith('dmg')
    for c in list(owner.hand):
        if c.name in INDES and destroyish and can_pay(g, owner, c.generic, c.pips):
            return _cast_protect(g, owner, c)
    for c in list(owner.hand):
        if c.name == 'Eerie Interlude' and not m.token and can_pay(g, owner, c.generic, c.pips):
            if not ais.pay_card(g, owner, c): return False
            interlude(g, owner, [m])
            return True
    return False


def galadriel_wipe_response(g, q, kind):
    from commander_sim import ais
    loss = sum(pval(g, m) for m in q.perms if m.creature or kind in ('rift', 'rebuke'))
    if loss < 6: return None
    if kind in ('destroy', 'dmg13', 'austere', 'nib'):
        for c in list(q.hand):
            if c.name in INDES and can_pay(g, q, c.generic, c.pips):
                return 'indes' if _cast_protect(g, q, c) else None
    for c in list(q.hand):
        if c.name == 'Eerie Interlude' and can_pay(g, q, c.generic, c.pips):
            if not ais.pay_card(g, q, c): return None
            interlude(g, q, [m for m in q.perms if m.creature and not m.token])
            return None
    return None


CI.galadriel_prio = galadriel_prio
CI.galadriel_abilities = galadriel_abilities
CI.galadriel_protect = galadriel_protect
CI.galadriel_wipe_response = galadriel_wipe_response
CI.galadriel_precombat = precombat
CI.galadriel_attack_answers = attack_answers
CI.vanguard_blocks = vanguard_blocks


# ================================================================== Mirror Entity, Whipcorder's morph, Elspeth's emblem
def mirror(g, p, x):
    """Mirror Entity: until end of turn your creatures have base power and toughness X/X and every creature type"""
    for m in p.perms:
        if not m.creature or m.phased: continue
        a, b = g.eot_pt.get(id(m), (0, 0))
        g.eot_pt[id(m)] = (a + x - (m.pow or 0), b + x - (m.tgh or 0))
        g.eot_kw.setdefault(id(m), set()).add('all types')
        if etgh(g, m) <= 0: die(g, m, 'sba')
    log(f"    {NAME(p)}'s creatures are {x}/{x} until end of turn", g)


def enter_face_down(g, p, c):
    """morph: a face-down 2/2 creature with no name and no abilities (Whipcorder); turned face up for its morph cost"""
    m = enter(g, p, c, was_cast=True)
    m.data = dict(m.data or {}, facedown=(m.pow, m.tgh))
    m.pow, m.tgh, m.neutered = 2, 2, True
    log(f'    {NAME(p)} has a face-down 2/2 creature', g)
    return m


def turn_face_up(g, p, m):
    pw, tg = (m.data or {}).get('facedown') or (m.pow, m.tgh)
    m.pow, m.tgh, m.neutered = pw, tg, False
    m.data = {k: v for k, v in (m.data or {}).items() if k != 'facedown'}
    log(f'  {NAME(p)} turns {m.cd.name} face up', g)


def elspeth_emblem(g, p):
    p.elspeth_emblem = True
    g.dsl_on = True                                  # flying is read through the keyword checks
    log(f'    {NAME(p)} gets an emblem: creatures they control get +2/+2 and have flying', g)


CI.elspeth_emblem = elspeth_emblem
full('Mirror Entity', 'changeling; {X}: until end of turn your creatures have base power and toughness X/X and every '
     'creature type (the AI pumps with its spare mana before combat)')
