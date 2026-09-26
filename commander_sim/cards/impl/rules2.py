"""The last approximated clauses of pool cards, implemented from their rules text: forced attacks,
crew, flash in combat, evoke / dash / casualty / overload / level up / adapt / harmonize / bargain / improvise,
blocking restrictions and taxes, ward from Auras, graveyard triggers, and the remaining activated abilities.

It also records which Approximate notes were AI decisions about cards whose rules were already complete (those
become Full with the decision described), and which cards are pieces of combos resolved as their end result.
"""
import importlib
from commander_sim.engine import *
from commander_sim import engine as E
from commander_sim.cards import cardimpl as CI
from commander_sim.cards.cardimpl import on, _eot, count_type
from commander_sim.cards.pool_cards import card, note
from commander_sim.cards.impl import common as IC
from commander_sim.cards.impl.common import best_opp_creature, best_opp_nonland, make_artifact_tokens, WALKERS
from commander_sim.cards.impl import rules as R


def full(name, text): note(name, 'Full', text)


def turn_now(g): return turn_stamp(g)


def ready_attackers(g, p):
    return [m for m in p.perms if m.creature and not m.tapped and not m.phased and not m.noatk
            and (not m.sick or (E.DSLMOD is not None and E.DSLMOD.has_kw(g, m, 'haste')))]


# ================================================================== forced attacks
def forced_attackers(g, p, atk, cands):
    """Goblin Rabblemaster: other Goblins you control attack each combat if able; Legion Warboss / Rabblemaster
    tokens made this turn attack if able"""
    rabble = any(m.cd is not None and m.cd.name == 'Goblin Rabblemaster' and not m.phased for m in p.perms)
    out = list(atk)
    for m in cands:
        if m in out or m.tapped or not m.creature: continue
        if (rabble and E.has_type(m, 'goblin') and not (m.cd is not None and m.cd.name == 'Goblin Rabblemaster')) or \
                (m.data and m.data.get('must_attack') == turn_now(g)):
            out.append(m)
    return out


@on('Goblin Rabblemaster', 'upkeep')
def _rabble_tok(g, src, p):
    if p is src.owner and p is g.active:
        for m in make_tokens(g, p, 1, 1, color='R', types=('goblin',), sick=False):
            if m.data is None: m.data = {}
            m.data['must_attack'] = turn_now(g)


@on('Legion Warboss', 'upkeep')
def _warboss_tok(g, src, p):
    if p is src.owner and p is g.active:
        for m in make_tokens(g, p, 1, 1, color='R', types=('goblin',), sick=False):
            if m.data is None: m.data = {}
            m.data['must_attack'] = turn_now(g)
full('Goblin Rabblemaster', 'a Goblin with haste each combat that must attack; other Goblins must attack; +1/+0 per '
     'other attacking Goblin')
full('Legion Warboss', 'a hasty Goblin each combat that must attack; mentor')


# ================================================================== crew: Esika's Chariot
@on("Esika's Chariot", 'crew')
def _crew(g, src, p):
    if p is not src.owner or src.tapped: return
    crew = sorted([m for m in p.perms if m.creature and not m.tapped and not m.phased and m is not src],
                  key=lambda m: (pval(g, m), -epow(g, m)))
    power, pick = 0, []
    for m in crew:
        if power >= 4: break
        if m.sick or epow(g, m) < 4 or m.token: pick.append(m); power += epow(g, m)
    if power < 4 or not R_open(g, p): return
    for m in pick: m.tapped = True
    if src.data is None: src.data = {}
    src.data['anim'] = True; src.pow = src.tgh = 4; src.data['crewed'] = turn_now(g)
    log(f"  {NAME(p)} crews Esika's Chariot", g)


def R_open(g, p):
    from commander_sim.cards.impl import lands as impl_lands
    return impl_lands.open_attack(g, p, 4)


@on("Esika's Chariot", 'attack')
def _chariot_copy(g, src, p, atk, d):
    if src in atk:
        toks = [m for m in p.perms if m.token and m.creature and not m.phased]
        if toks:
            t = max(toks, key=lambda m: epow(g, m))
            make_tokens(g, p, 1, t.pow, t.tgh, fly=t.fly, types=tuple(t.ttypes), color=t.colors)


@on("Esika's Chariot", 'etb')
def _chariot_cats(g, src, p, m):
    if m is src: make_tokens(g, src.owner, 2, 2, color='G', types=('cat',))


card("Esika's Chariot", 'leg', types='A', dsl=[])
full("Esika's Chariot", 'two 2/2 Cats on entry; crew 4 (tapped creatures with power 4 or more); attacking copies a token')


def uncrew(g, p):
    for m in p.perms:
        if m.data and m.data.get('crewed') and m.data.get('crewed') != turn_now(g):
            m.data['anim'] = False; m.data['crewed'] = None


# ================================================================== flash in combat: Embercleave, The Wandering Emperor
E.SELF_COST['Embercleave'] = lambda g, p, c: -sum(1 for m in p.perms if m.creature and m.tapped and getattr(p, 'attacking', None)
                                                  == turn_now(g) and m in getattr(p, 'attackers', ()))


@on('Embercleave', 'hand_attack')
def _cleave_flash(g, c, p, atk, d):
    """flash, costs {1} less per attacking creature, attaches on entry: cast after attackers are declared"""
    p.attacking = turn_now(g); p.attackers = list(atk)
    if c not in p.hand or not castable(g, p, c) or not can_pay(g, p, *cost_of(p, c)): return
    if len(atk) < 2 and sum(epow(g, m) for m in atk) < 5: return
    p.hand.remove(c); pay(g, p, *cost_of(p, c))
    log(f'  {NAME(p)} flashes in Embercleave', g)
    on_cast(g, p, c)
    if not counter_window(g, p, c, 5, {}): p.gy.append(c); return
    m = enter(g, p, c, was_cast=True)
    host = max(atk, key=lambda a: epow(g, a)) if atk else None
    if host is not None: m.attached = host


CI.SPELL_PRIO['Embercleave'] = 0
full('Embercleave', 'flash during combat, {1} less per attacker; attaches to the best attacker: +1/+1, double strike, trample')


@on('The Wandering Emperor', 'hand_defend')
def _emperor_flash(g, c, d, p, atk, assign):
    """flashed in during an opponent's attack: -2 exiles the biggest tapped attacker"""
    if c not in d.hand or not castable(g, d, c) or not can_pay(g, d, *cost_of(d, c)): return
    t = max([a for a in atk if a in p.perms and a.tapped], key=lambda a: pval(g, a), default=None)
    if t is None or pval(g, t) < 4: return
    pay(g, d, *cost_of(d, c)); d.hand.remove(c)
    log(f'  {NAME(d)} flashes in The Wandering Emperor', g)
    on_cast(g, d, c)
    if not counter_window(g, d, c, 5, {}): d.gy.append(c); return
    m = enter(g, d, c, was_cast=True)
    m.loyalty = (m.loyalty or 3) - 2; m.loyalty_used = (g.round, d.key, 1)
    atk.remove(t); apply_removal(g, d, t, 'exile'); gain(d, 2)
full('The Wandering Emperor', 'flash: cast during an opponent\'s attack to exile a tapped attacker (-2); otherwise '
     '+1 counter and first strike, -1 2/2 Samurai')


# ================================================================== evoke: Solitude
@on('Solitude', 'hand_options')
def _solitude_evoke(g, c, p, s, post):
    """evoke by exiling a white card from hand (free, instant speed): exile the best opposing creature"""
    if can_pay(g, p, *cost_of(p, c)) and post is not None: return []
    whites = [x for x in p.hand if x is not c and 'W' in x.pips]
    t = best_opp_creature(g, p)
    if not whites or t is None or pval(g, t) < 6: return []

    def go():
        if c not in p.hand or t not in t.owner.perms: return False
        x = min([x for x in p.hand if x is not c and 'W' in x.pips], key=lambda x: card_worth(g, p, x), default=None)
        if x is None: return False
        p.hand.remove(x); p.exile.append(x); p.hand.remove(c)
        log(f'  {NAME(p)} evokes Solitude (exiling {x.name})', g); on_cast(g, p, c)
        m = enter(g, p, c, was_cast=True)
        if m in p.perms: die(g, m, 'sac')
        return True
    return [(pval(g, t) - 4.0, 'evoke Solitude', go)]
full('Solitude', 'flash, lifelink; exiles the best opposing creature on entry; evoke by exiling a white card (free)')
full('Mulldrifter', 'flying; draw two on entry; evoke {2}{U} when the full cost is out of reach')


# ================================================================== dash: Ragavan
@on('Ragavan, Nimble Pilferer', 'hand_options')
def _ragavan_dash(g, c, p, s, post):
    if post is not False or not can_pay(g, p, 1, 'R') or not castable(g, p, c): return []
    from commander_sim.cards.impl import lands as impl_lands
    if not impl_lands.open_attack(g, p, 2): return []

    def go():
        if c not in p.hand or not can_pay(g, p, 1, 'R'): return False
        p.hand.remove(c); pay(g, p, 1, 'R')
        log(f'  {NAME(p)} dashes Ragavan', g); on_cast(g, p, c)
        if not counter_window(g, p, c, 3, {}): p.gy.append(c); return True
        m = enter(g, p, c, was_cast=True); m.sick = False
        if m.data is None: m.data = {}
        m.data['dash'] = True
        return True
    return [(2.5, 'dash Ragavan', go)]


@on('Ragavan, Nimble Pilferer', 'end_step')
def _ragavan_back(g, src, p):
    if p is src.owner and src.data and src.data.get('dash') and src in p.perms:
        leave(g, src); p.hand.append(src.cd)
full('Ragavan, Nimble Pilferer', 'combat damage: a Treasure and the defender\'s top card (castable this turn); dash {1}{R}')


# ================================================================== casualty: Ob Nixilis, the Adversary
@on('Ob Nixilis, the Adversary', 'etb')
def _obnix_casualty(g, src, p, m):
    if m is not src or not g.last_cast_etb or (src.data or {}).get('copy'): return
    o = src.owner
    fod = [x for x in o.perms if x.creature and not x.is_cmd and x is not src and (x.token or pval(g, x) < 3) and epow(g, x) >= 2]
    if not fod: return
    x = max(fod, key=lambda x: epow(g, x)); n = epow(g, x)
    die(g, x, 'sac')
    cp = enter_token_copy(g, o, src.cd)
    if cp is not None:
        cp.loyalty = n
        if cp.data is None: cp.data = {}
        cp.data['copy'] = True
        log(f'    casualty {n}: a token copy of Ob Nixilis with loyalty {n}', g)
full('Ob Nixilis, the Adversary', 'casualty X (a spare creature: a token copy with loyalty X); +1 drain 2 or discard, '
     '-2 Devil, -7 draw seven and lose 7')


# ================================================================== overload: Winds of Abandon
@IC.spell('Winds of Abandon', prio=lambda g, p, c: 62 if total_mana(g, p) >= 6 and sum(
        pval(g, m) for q in g.opps(p) for m in q.perms if m.creature) >= 12 else (40 if best_opp_creature(g, p) is not None else 0),
          types='S', status=('Full', 'exiles an opposing creature (its controller searches a basic), or overloaded for '
                                     '{4}{W}{W} every opposing creature'))
def _winds(g, p, c, ctx):
    overload = getattr(p, 'winds_overload', False) or total_mana(g, p) >= 4
    if overload and can_pay(g, p, 4, ''):
        pay(g, p, 4, '')
        for q in g.opps(p):
            n = 0
            for m in list(q.perms):
                if m.creature and not m.phased: exile_perm(g, m); n += 1
            land_ramp(g, q, n, True)
        return
    t = ctx.get('target') or best_opp_creature(g, p)
    if t is not None: q = t.owner; apply_removal(g, p, t, 'exile', c); land_ramp(g, q, 1, True)


# ================================================================== level up / adapt / mana tokens
@on('Joraga Treespeaker', 'options')
def _joraga(g, src, p, s, post):
    if p is not src.owner or post is not False: return []
    lvl = (src.data or {}).get('level', 0)
    target = 1 if lvl < 1 else 5
    if lvl >= 5 or not can_pay(g, p, 1, 'G'): return []
    if lvl >= 1 and total_mana(g, p) < 6: return []

    def go():
        if not can_pay(g, p, 1, 'G'): return False
        pay(g, p, 1, 'G')
        if src.data is None: src.data = {}
        src.data['level'] = src.data.get('level', 0) + 1
        return True
    return [(2.5 if lvl < 1 else 0.6 + 0.3 * count_type(g, p, 'elf'), f'level up Joraga ({lvl + 1})', go)]


CI.DYN_MANA['Joraga Treespeaker'] = lambda g, p, m: 2 if (m.data or {}).get('level', 0) >= 1 else 0
card('Joraga Treespeaker', 'pow=1 tgh=1 dork=G', dsl=[])


@on('Joraga Treespeaker', 'extra_mana')
def _joraga_elves(g, src, p, U):
    """level 5: Elves you control have {T}: add {G}{G}"""
    if src.owner is not p or (src.data or {}).get('level', 0) < 5: return []
    have = {id(u[0]) for u in U}
    return [[m, 'G', 2] for m in p.perms if m.creature and m is not src and E.has_type(m, 'elf') and not m.tapped
            and not m.sick and id(m) not in have]
full('Joraga Treespeaker', 'level up {1}{G}: level 1 taps for GG; level 5 every Elf taps for GG')


@on('Incubation Druid', 'options')
def _adapt(g, src, p, s, post):
    if p is not src.owner or post is None or src.plus > 0 or not can_pay(g, p, 3, 'GG'): return []

    def go():
        if src.plus > 0 or not can_pay(g, p, 3, 'GG'): return False
        pay(g, p, 3, 'GG'); src.plus += 3; log(f'  {NAME(p)} adapts Incubation Druid', g); return True
    return [(1.5, 'adapt Incubation Druid', go)]
full('Incubation Druid', 'taps for one mana of a colour a land could make; three once it has a counter; adapt 3 for {3}{G}{G}')


def mana_token(kind):
    """tokens that tap for mana (Freyalise's Elf Druids)"""
    def mk(g, p, n):
        for m in make_tokens(g, p, n, 1, color='G', types=('elf', 'druid')):
            if m.data is None: m.data = {}
            m.data['manatok'] = kind
    return mk


_elfdruid = mana_token('G')


def _freyalise_plus(g, p, src): _elfdruid(g, p, 2)


if "Freyalise, Llanowar's Fury" in WALKERS:
    WALKERS["Freyalise, Llanowar's Fury"][0] = (2, 'two Elf Druids', lambda g, p, src: 3.0, _freyalise_plus)
full("Freyalise, Llanowar's Fury", '+2 two 1/1 Elf Druids that tap for {G}; -2 destroy an artifact or enchantment; '
     '-6 draw per green creature')


# ================================================================== harmonize, bargain, improvise, transmute, cycling
@on("Nature's Rhythm", 'gy_options')
def _harmonize(g, c, p, s, post):
    """harmonize {X}{G}{G}{G}{G} (tapping a creature reduces it by its power): cast from the graveyard, then exile it"""
    if post is not False: return []
    red = max((epow(g, m) for m in p.perms if m.creature and not m.tapped), default=0)
    x = total_mana(g, p) + red - 4
    if x < 3 or not can_pay(g, p, max(0, 0), 'GGGG'): return []

    def go():
        if c not in p.gy: return False
        tapper = max([m for m in p.perms if m.creature and not m.tapped], key=lambda m: epow(g, m), default=None)
        r = epow(g, tapper) if tapper is not None else 0
        if tapper is not None: tapper.tapped = True
        xx = total_mana(g, p) - 4 + r
        if not can_pay(g, p, max(0, xx - r), 'GGGG'): return False
        p.gy.remove(c); pay(g, p, max(0, xx - r), 'GGGG'); p.exile.append(c)
        log(f"  {NAME(p)} harmonizes Nature's Rhythm (X={xx})", g)
        from commander_sim.cards.impl import t3 as impl_t3
        impl_t3._put_creature(g, p, lambda y: y.cmc <= xx)
        return True
    return [(1.0 + x / 2.0, "harmonize Nature's Rhythm", go)]
full("Nature's Rhythm", 'X = spare mana: a creature with MV X or less onto the battlefield; harmonize from the graveyard')


@IC.spell('Beseech the Mirror', prio=50, types='S', status=('Full', 'tutor to hand; bargained (a token or spare '
          'artifact/enchantment sacrificed), a found card with MV 4 or less is cast free'))
def _beseech(g, p, c, ctx):
    fod = [m for m in p.perms if (m.token or (m.cd is not None and ('A' in m.cd.types or 'E' in m.cd.types) and pval(g, m) < 2))]
    before = set(map(id, p.hand))
    tutor(g, p, 'any')
    new = [x for x in p.hand if id(x) not in before]
    if fod and new and new[0].cmc <= 4 and not new[0].land and castable(g, p, new[0]):
        m = min(fod, key=lambda m: pval(g, m))
        if m.token: leave(g, m)
        else: die(g, m, 'sac')
        x = new[0]; p.hand.remove(x)
        log(f'    bargained: {x.name} is cast free', g)
        cast_card(g, p, x, 'lib', {})


@IC.spell('Whir of Invention', prio=lambda g, p, c: 55 if total_mana(g, p) >= 5 else 0, types='I',
          status=('Full', 'X = spare mana (improvise: untapped artifacts help pay); the best artifact with MV X or less'))
def _whir(g, p, c, ctx):
    x = ctx.get('x', 0)
    arts = [m for m in p.perms if not m.tapped and ((m.cd is not None and 'A' in m.cd.types and 'rock' not in m.cd.tags)
                                                    or (m.token and 'artifact' in m.ttypes))]
    for m in arts: m.tapped = True
    x += len(arts)
    cs = [y for y in searchable(g, p) if 'A' in y.types and y.cmc <= x]
    if cs:
        from commander_sim.cards.impl import t5 as impl_t5
        y = max(cs, key=lambda y: (y.name in impl_t5.IC_COMBO_ART, card_worth(g, p, y), y.cmc))
        p.library.remove(y); g.rng.shuffle(p.library); enter(g, p, y)


@IC.spell('Transmute Artifact', prio=lambda g, p, c: 50 if any(m.cd is not None and 'A' in m.cd.types and pval(g, m) < 3
                                                               for m in p.perms) else 0, types='S',
          status=('Full', 'sacrifice an artifact, search an artifact: onto the battlefield if you pay the difference '
                          'in mana value'))
def _transmute(g, p, c, ctx):
    arts = [m for m in p.perms if m.cd is not None and 'A' in m.cd.types and not m.phased]
    if not arts: return
    s = min(arts, key=lambda m: pval(g, m)); mv = s.cd.cmc; die(g, s, 'sac')
    from commander_sim.ai import pool_ai
    cs = [y for y in searchable(g, p) if 'A' in y.types]
    if not cs: return
    wish = pool_ai.wish_list(g, p)
    best = max(cs, key=lambda y: (y.name in wish and can_pay(g, p, max(0, y.cmc - mv), ''), card_worth(g, p, y)))
    p.library.remove(best); g.rng.shuffle(p.library)
    diff = max(0, best.cmc - mv)
    if can_pay(g, p, diff, ''): pay(g, p, diff, ''); enter(g, p, best)
    else: p.gy.append(best)


@on('Unearth', 'hand_options')
def _unearth_cycle(g, c, p, s, post):
    """cycling {2} when there is nothing worth returning"""
    if any(x.creature and x.cmc <= 3 for x in p.gy) or not can_pay(g, p, 2, ''): return []

    def go():
        if c not in p.hand or not can_pay(g, p, 2, ''): return False
        p.hand.remove(c); pay(g, p, 2, ''); p.gy.append(c); draw(g, p, 1); return True
    return [(0.8, 'cycle Unearth', go)]
full('Unearth', 'returns the best creature card with MV 3 or less; cycling {2} otherwise')


# ================================================================== Wishclaw Talisman: the opponent uses it back
@on('Wishclaw Talisman', 'options')
def _wishclaw(g, src, p, s, post):
    if p is not src.owner or src.tapped or not can_pay(g, p, 1, '') or (src.data or {}).get('wishes', 3) <= 0: return []
    if g.active is not p and post is not None: return []

    def go():
        if src.tapped or not can_pay(g, p, 1, ''): return False
        pay(g, p, 1, ''); src.tapped = True
        if src.data is None: src.data = {}
        src.data['wishes'] = src.data.get('wishes', 3) - 1
        tutor(g, p, 'any')
        q = max(g.opps(p), key=lambda q: threat(g, p, q)) if g.opps(p) else None
        if q is not None and src in p.perms and src.data['wishes'] > 0:
            p.perms.remove(src); src.owner = q; q.perms.append(src); src.tapped = False
            g.bf_ver = getattr(g, 'bf_ver', 0) + 1; g.hook_cache = None
            log(f'    {NAME(q)} gains control of Wishclaw Talisman', g)
        return True
    u = 3.5 if importlib.import_module('commander_sim.ai.pool_ai').wish_list(g, p) else 1.5
    return [(u, 'Wishclaw Talisman', go)]
full('Wishclaw Talisman', 'three wishes: {1},{T} tutor (on your turn), then the most threatening opponent gains control '
     'and uses it on theirs')


# ================================================================== Aura of Silence, Soul-Guide Lantern, Shadowspear
@on('Aura of Silence', 'options')
def _aura_silence(g, src, p, s, post):
    if p is not src.owner: return []
    t = best_opp_nonland(g, p, lambda m: m.cd is not None and ('A' in m.cd.types or 'E' in m.cd.types))
    if t is None or pval(g, t) < 5: return []

    def go():
        if src not in p.perms or t not in t.owner.perms: return False
        die(g, src, 'sac'); apply_removal(g, p, t, 'destroy'); return True
    return [(pval(g, t) - 3.0, f'sacrifice Aura of Silence -> {t.name}', go)]
full('Aura of Silence', 'opponents\' artifacts and enchantments cost {2} more; sacrifice: destroy a valuable one')


@on('Soul-Guide Lantern', 'etb')
def _lantern_etb(g, src, p, m):
    if m is not src: return
    best = max([(q, x) for q in g.opps(src.owner) for x in q.gy], key=lambda t: (t[1].creature, t[1].bomb, t[1].cmc), default=None)
    if best is not None: best[0].gy.remove(best[1]); best[0].exile.append(best[1])


@on('Soul-Guide Lantern', 'options')
def _lantern_draw(g, src, p, s, post):
    if p is not src.owner or post is None or not can_pay(g, p, 1, ''): return []
    if any(x.creature and x.bomb >= 5 for q in g.opps(p) for x in q.gy): return []
    if total_mana(g, p) < 4: return []

    def go():
        if src not in p.perms or not can_pay(g, p, 1, ''): return False
        pay(g, p, 1, ''); die(g, src, 'sac'); draw(g, p, 1); return True
    return [(0.7, 'Soul-Guide Lantern draw', go)]
full('Soul-Guide Lantern', 'exiles a card from a graveyard on entry; sacrificed to exile a graveyard (reanimation) or '
     'for a card')


@on('Shadowspear', 'options')
def _shadowspear(g, src, p, s, post):
    """{1}: opponents' permanents lose hexproof and indestructible until end of turn (before removal or combat)"""
    if p is not src.owner or post is None or not can_pay(g, p, 1, '') or getattr(g, 'spear', None) == turn_now(g): return []
    prot = [m for q in g.opps(p) for m in q.perms if not m.phased and E.DSLMOD is not None and
            (E.DSLMOD.has_kw(g, m, 'hexproof') or E.DSLMOD.has_kw(g, m, 'indestructible')) and pval(g, m) >= 5]
    removal = [c for c in p.hand if 'rem' in c.tags and can_pay(g, p, c.generic + 1, c.pips)]
    if not prot or not removal: return []

    def go():
        if not can_pay(g, p, 1, ''): return False
        pay(g, p, 1, ''); g.spear = turn_now(g)
        log(f'  {NAME(p)} activates Shadowspear', g); return True
    return [(1.5 + pval(g, max(prot, key=lambda m: pval(g, m))) / 3.0, 'Shadowspear', go)]


def spear_active(g, m, p=None):
    return getattr(g, 'spear', None) == turn_now(g)
full('Shadowspear', 'equipped creature +1/+1, trample, lifelink; {1}: opponents lose hexproof and indestructible this turn')


# ================================================================== blocking rules
@on('Silent Arbiter', 'blocks')
def _arbiter_block(g, src, p, atk, d, assign):
    """no more than one creature can block each combat"""
    if len(assign) > 1:
        keep = max(assign.items(), key=lambda kv: pval(g, kv[0]))
        assign.clear(); assign[keep[0]] = keep[1]
full('Silent Arbiter', 'no more than one creature can attack or block each combat')


@on('Archangel of Tithes', 'blocks')
def _tithes_block(g, src, p, atk, d, assign):
    """while it attacks, creatures can't block unless their controller pays {1} each"""
    if p is not src.owner or src not in atk: return
    for a, b in list(assign.items()):
        if can_pay(g, d, 1, ''): pay(g, d, 1, '')
        else: del assign[a]
full('Archangel of Tithes', 'flying; attack tax {1} while untapped; blocking tax {1} while it attacks')


def annex_life(g, p, d, atk):
    """Norn's Annex: {W/P} per attacker: pay W, or 2 life"""
    if not any(m.cd is not None and m.cd.name == "Norn's Annex" and not m.phased for m in d.perms): return atk
    out = []
    for m in atk:
        if can_pay(g, p, 0, 'W'): pay(g, p, 0, 'W'); out.append(m)
        elif p.life > 12: lose_life(g, p, 2, p); out.append(m)
    return out


@on("Norn's Annex", 'attack_tax')
def _annex_tax(g, src, attacker, d): return 0
full("Norn's Annex", 'each attacker at you costs {W/P} ({W} or 2 life)')


# ================================================================== ward from an Aura; Liesa's commander tax in life
def aura_ward(g, m):
    if not getattr(g, 'auras', None): return 0
    return sum(2 for a in IC.auras_on(g, m) if a.cd.name == 'Sheltered by Ghosts')
full('Sheltered by Ghosts', 'exiles an opposing nonland permanent until it leaves; +1/+0, lifelink and ward {2}')


def liesa_tax(p, c):
    return c is p.cmd and c.name == 'Liesa, Shroud of Dusk'
full('Liesa, Shroud of Dusk', 'can\'t be countered; flying, lifelink; commander tax paid in life; each spell costs its '
     'caster 2 life')


# ================================================================== graveyard triggers: Syr Konrad
@on('Syr Konrad, the Grim', 'cards_to_gy')
def _konrad_mill(g, src, p, cards):
    n = sum(1 for c in cards if c.creature)
    if n:
        for q in g.opps(src.owner): lose_life(g, q, n, src.owner, kind='triggers')
full('Syr Konrad, the Grim', 'another creature dying, or a creature card milled or discarded, deals 1 to each opponent')


# ================================================================== Glimpse of Nature
@IC.spell('Glimpse of Nature', prio=lambda g, p, c: 55 if sum(1 for x in p.hand if x.creature and x.cmc <= 2) >= 2 else 0,
          types='S', status=('Full', 'this turn, each creature spell you cast draws a card'))
def _glimpse(g, p, c, ctx):
    p.glimpse = turn_now(g)


def glimpse_draw(g, p, c):
    if c.creature and getattr(p, 'glimpse', None) == turn_now(g): draw(g, p, 1)


# ================================================================== Chatterfang
def chatterfang_squirrels(g, p, n):
    if n > 0 and any(m.cd is not None and m.cd.name == 'Chatterfang, Squirrel General' and not m.phased for m in p.perms):
        g.no_fang = True
        try: make_tokens(g, p, n, 1, color='G', types=('squirrel',))
        finally: g.no_fang = False


@on('Chatterfang, Squirrel General', 'options')
def _fang(g, src, p, s, post):
    """{B}, sacrifice X Squirrels: target creature gets +X/-X"""
    if p is not src.owner or not can_pay(g, p, 0, 'B'): return []
    sq = [m for m in p.perms if m.token and 'squirrel' in m.ttypes]
    t = best_opp_creature(g, p, lambda m: etgh(g, m) <= len(sq))
    if t is None or pval(g, t) < 4: return []
    x = etgh(g, t)

    def go():
        if t not in t.owner.perms or not can_pay(g, p, 0, 'B'): return False
        pay(g, p, 0, 'B')
        for m in [m for m in p.perms if m.token and 'squirrel' in m.ttypes][:x]: die(g, m, 'sac')
        if not untargetable(g, t): _eot(g, t, x, -x); die(g, t, 'sba') if etgh(g, t) <= 0 else None
        return True
    return [(pval(g, t) - 0.5 * x, f'Chatterfang -> {t.name}', go)]
full('Chatterfang, Squirrel General', 'forestwalk; every token creation adds that many 1/1 Squirrels; {B}, sacrifice X '
     'Squirrels: +X/-X')


# ================================================================== Grist: repeat on Insects
def _grist_plus(g, p, src):
    while True:
        make_tokens(g, p, 1, 1, color='G', types=('insect',))
        if not p.library: return
        c = p.library.pop(); p.gy.append(c)
        if 'insect' not in c.subtypes: return
        src.loyalty += 1


if 'Grist, the Hunger Tide' in WALKERS:
    WALKERS['Grist, the Hunger Tide'][0] = (1, 'Insect and mill', lambda g, p, src: 2.5, _grist_plus)
full('Grist, the Hunger Tide', '+1 Insect and mill (repeats on a milled Insect), -2 sacrifice to destroy, -5 drain '
     'per creature card in the graveyard')


# ================================================================== Detention Sphere, Eldrazi Displacer, Charming Prince
@on('Detention Sphere', 'etb')
def _dsphere(g, src, p, m):
    if m is not src: return
    o = src.owner
    t = best_opp_nonland(g, o, lambda x: not (x.cd is not None and x.cd.land))
    if t is None: return
    same = [x for q in g.opps(o) for x in q.perms if x.name == t.name and not x.phased]
    src.data = src.data or {}; src.data['held'] = []
    for x in same:
        if x.token: leave(g, x)
        else: src.data['held'].append((x.cd, x.owner)); exile_perm(g, x)
    log(f'    Detention Sphere exiles every {t.name}', g)


@on('Detention Sphere', 'leaves')
def _dsphere_back(g, m):
    for cd, q in (m.data or {}).get('held', []):
        if q.alive and cd in q.exile: q.exile.remove(cd); enter(g, q, cd)
full('Detention Sphere', 'exiles a nonland permanent and every other one with its name (tokens are gone for good); '
     'they return when it leaves')


@on('Eldrazi Displacer', 'options')
def _displacer_opp(g, src, p, s, post):
    """{2}{C}: blink an opponent's token (it's gone) or reset a threatening creature (it returns tapped)"""
    if p is not src.owner or post is None or not can_pay(g, p, 3, ''): return []
    toks = [m for q in g.opps(p) for m in q.perms if m.token and m.creature and not untargetable(g, m)]
    if not toks: return []
    t = max(toks, key=lambda m: epow(g, m))
    if epow(g, t) < 3: return []

    def go():
        if t not in t.owner.perms or not can_pay(g, p, 3, ''): return False
        pay(g, p, 3, ''); leave(g, t); log(f'  {NAME(p)} blinks {t.name} with Eldrazi Displacer (a token: gone)', g); return True
    return [(1.0 + 0.4 * epow(g, t), 'Eldrazi Displacer on a token', go)]
full('Eldrazi Displacer', 'blinks your ETB creatures, or an opponent\'s token (gone) with {2}{C}')


@on('Charming Prince', 'etb')
def _prince(g, src, p, m):
    if m is not src: return
    o = src.owner
    cs = [x for x in o.perms if x is not src and x.creature and not x.token and x.cd is not None and R_etb(x) > 0]
    if cs:
        t = max(cs, key=R_etb); leave(g, t); o.oath_return = getattr(o, 'oath_return', []) + [t.cd]
        log(f'    Charming Prince exiles {t.name} until the end step', g)
    else: gain(o, 3)


@on('Charming Prince', 'end_step')
def _prince_back(g, src, p):
    o = src.owner
    for cd in getattr(o, 'oath_return', []): enter(g, o, cd)
    o.oath_return = []


def R_etb(m):
    return importlib.import_module('commander_sim.cards.impl.partials').etb_val(m)
full('Charming Prince', 'blinks your best ETB creature until the end step, else scry 2 / 3 life')


# ================================================================== pumps: Battle Cry Goblin, Purphoros, Scourge, Resplendent, Rionya
@on('Battle Cry Goblin', 'options')
def _bcg_pump(g, src, p, s, post):
    if p is not src.owner or post is not False or not can_pay(g, p, 1, 'R'): return []
    gobs = [m for m in ready_attackers(g, p) if E.has_type(m, 'goblin')]
    if len(gobs) < 4 or getattr(p, 'bcg_turn', None) == turn_now(g): return []

    def go():
        if not can_pay(g, p, 1, 'R'): return False
        pay(g, p, 1, 'R'); p.bcg_turn = turn_now(g)
        for m in p.perms:
            if m.creature and E.has_type(m, 'goblin'): _eot(g, m, 1, 0)
        return True
    return [(0.3 * len(gobs), 'Battle Cry Goblin pump', go)]
full('Battle Cry Goblin', '{1}{R}: Goblins +1/+0 before a wide attack; pack tactics Goblin token')


def devotion(p, col):
    return sum(m.cd.pips.count(col) for m in p.perms if m.cd is not None and not m.phased)


@on('Purphoros, God of the Forge', 'etb')
def _purph_on(g, src, p, m):
    if m is src:
        if src.data is None: src.data = {}
        src.pow, src.tgh = 6, 5
    if m.owner is src.owner and m.creature and m is not src:
        for q in g.opps(src.owner): lose_life(g, q, 2, src.owner, kind='triggers', damage=True)
    purph_update(src)


def purph_update(src):
    on_ = devotion(src.owner, 'R') >= 5
    if src.data is None: src.data = {}
    src.data['anim'] = on_


@on('Purphoros, God of the Forge', 'options')
def _purph_pump(g, src, p, s, post):
    purph_update(src)
    if p is not src.owner or post is not False or not can_pay(g, p, 2, 'R'): return []
    ready = ready_attackers(g, p)
    if len(ready) < 4 or getattr(p, 'purph_turn', None) == turn_now(g): return []

    def go():
        if not can_pay(g, p, 2, 'R'): return False
        pay(g, p, 2, 'R'); p.purph_turn = turn_now(g)
        for m in p.perms:
            if m.creature: _eot(g, m, 1, 0)
        return True
    return [(0.3 * len(ready), 'Purphoros pump', go)]


@on('Purphoros, God of the Forge', 'grant_kw')
def _purph_ind(g, src, m, kw):
    return kw == 'indestructible' and m is src
full('Purphoros, God of the Forge', 'indestructible; a creature only with devotion to red 5+ (6/5); 2 damage to each '
     'opponent per creature entering; {2}{R}: creatures +1/+0')


@on('Scourge of Valkas', 'options')
def _scourge_fire(g, src, p, s, post):
    if p is not src.owner or post is not False or src.tapped or src.sick or total_mana(g, p) < 4: return []
    from commander_sim.cards.impl import lands as impl_lands
    if not impl_lands.open_attack(g, p, epow(g, src), True): return []
    n = total_mana(g, p) - 2

    def go():
        if not can_pay(g, p, n - 1, 'R'): return False
        pay(g, p, n - 1, 'R'); _eot(g, src, n, 0); return True
    return [(0.4 * n, 'Scourge of Valkas firebreathing', go)]
full('Scourge of Valkas', 'flying; Dragons entering deal damage equal to their power; {R}: +1/+0 before an open attack')


@on('Resplendent Angel', 'options')
def _resplendent_pump(g, src, p, s, post):
    if p is not src.owner or post is not False or src.tapped or not can_pay(g, p, 3, 'WWW'): return []

    def go():
        if not can_pay(g, p, 3, 'WWW'): return False
        pay(g, p, 3, 'WWW'); _eot(g, src, 2, 0); g.eot_kw.setdefault(id(src), set()).add('lifelink'); return True
    return [(1.2, 'Resplendent Angel pump', go)]
full('Resplendent Angel', 'flying; a 4/4 Angel at end step after gaining 5+ life; {3}{W}{W}{W}: +2/+0 and lifelink')


@on('Rionya, Fire Dancer', 'combat_start')
def _rionya_x(g, src, p):
    if p is not src.owner: return
    cs = [m for m in p.perms if m.creature and m is not src and m.cd is not None and not m.phased]
    if not cs: return
    t = max(cs, key=lambda m: (epow(g, m) + R_etb(m)))
    x = 1 + R.casts_isc(g, p)
    out = []
    for _ in range(x):
        tok = enter_token_copy(g, p, t.cd)
        if tok is not None:
            tok.sick = False
            if tok.data is None: tok.data = {}
            tok.data['rionya'] = True; out.append(tok)
    log(f'    Rionya makes {len(out)} hasty copies of {t.name}', g)
    return out


@on('Rionya, Fire Dancer', 'end_step')
def _rionya_exile(g, src, p):
    if p is src.owner:
        for m in [m for m in p.perms if m.data and m.data.get('rionya')]: leave(g, m)
full('Rionya, Fire Dancer', 'each combat: 1 + (instants and sorceries cast this turn) hasty token copies of a creature, '
     'exiled at end step')


def casts_isc(g, p):
    log_ = getattr(p, 'turn_casts', None)
    if not log_ or log_[0] != turn_stamp(g): return 0
    return sum(1 for c in log_[1] if c.instant or c.sorcery)


R.casts_isc = casts_isc


# ================================================================== Stoneforge Mystic, Gilded Goose, Legion's Landing, Destiny Spinner
@on('Stoneforge Mystic', 'options')
def _sfm_put(g, src, p, s, post):
    """{1}{W}, {T}: put an Equipment card from your hand onto the battlefield"""
    if p is not src.owner or src.tapped or src.sick or not can_pay(g, p, 1, 'W'): return []
    eq = [c for c in p.hand if 'equipment' in c.subtypes]
    if not eq: return []
    c = max(eq, key=lambda c: c.cmc)
    if c.cmc <= 2: return []

    def go():
        if c not in p.hand or src.tapped or not can_pay(g, p, 1, 'W'): return False
        pay(g, p, 1, 'W'); src.tapped = True; p.hand.remove(c); enter(g, p, c)
        log(f'  {NAME(p)} puts {c.name} onto the battlefield with Stoneforge Mystic', g); return True
    return [(1.0 + c.cmc / 2.0, f'Stoneforge Mystic ({c.name})', go)]
full('Stoneforge Mystic', 'fetches an Equipment; {1}{W},{T}: puts an expensive Equipment from hand onto the battlefield')


@on('Gilded Goose', 'options')
def _goose_food(g, src, p, s, post):
    if p is not src.owner or src.tapped or not can_pay(g, p, 1, 'G') or getattr(p, 'foods', 0) > 0: return []
    if post is not None: return []

    def go():
        if src.tapped or not can_pay(g, p, 1, 'G'): return False
        pay(g, p, 1, 'G'); src.tapped = True; make_artifact_tokens(g, p, 'Food', 1); return True
    return [(0.9, 'Gilded Goose: Food', go)]
full('Gilded Goose', 'flying; a Food on entry; {1}{G},{T}: Food (at end of turn); {T}, sacrifice a Food: any colour')


@on("Legion's Landing // Adanto, the First Fort", 'land_options')
def _adanto(g, L, p, s, post):
    """the flipped land: {2}{W}, {T}: a 1/1 lifelink Vampire (at end of turn)"""
    from commander_sim.cards.impl import lands as impl_lands
    if not getattr(p, 'adanto', False) or post is not None or L.tapped or not impl_lands.can_pay_without(g, p, L, 2, 'W'): return []

    def go():
        if L.tapped or not impl_lands.pay_without(g, p, L, 2, 'W'): return False
        L.tapped = True; make_tokens(g, p, 1, 1, lifelink=True, color='W', types=('vampire',)); return True
    return [(1.2, 'Adanto token', go)]
full("Legion's Landing // Adanto, the First Fort", 'a 1/1 lifelink token; flips with three attackers into a land that '
     'taps for W or makes more tokens')


@on('Destiny Spinner', 'options')
def _spinner(g, src, p, s, post):
    """{3}{G}: a land becomes an X/X Elemental with trample and haste (X = enchantments you control)"""
    if p is not src.owner or post is not False or not can_pay(g, p, 3, 'G'): return []
    x = sum(1 for m in p.perms if m.cd is not None and 'E' in m.cd.types and not m.phased)
    if x < 4 or not p.lands: return []
    from commander_sim.cards.impl import lands as impl_lands
    if not impl_lands.open_attack(g, p, x): return []

    def go():
        if not can_pay(g, p, 3, 'G') or len(p.lands) < 2: return False
        pay(g, p, 3, 'G')
        L = next((L for L in p.lands if L.tapped), p.lands[0])
        impl_lands.animate(g, p, L, x, x, kws=('trample', 'haste'))
        return True
    return [(0.5 * x, 'Destiny Spinner animates a land', go)]
full('Destiny Spinner', 'your creature and enchantment spells can\'t be countered; {3}{G}: a land becomes an X/X '
     'trample haste Elemental')


# ================================================================== Hullbreaker Horror: bounce a spell
def hullbreaker_counter(g, q, c):
    """q controls Hullbreaker Horror and holds a cheap instant: casting it bounces c (the spell returns to its hand)"""
    if not any(m.cd is not None and m.cd.name == 'Hullbreaker Horror' and not m.phased for m in q.perms): return False
    inst = [x for x in q.hand if x.instant and 'ctr' not in x.tags and can_pay(g, q, *cost_of(q, x)) and castable(g, q, x)]
    if not inst: return False
    x = min(inst, key=lambda x: (x.cmc, card_worth(g, q, x)))
    pay(g, q, *cost_of(q, x)); q.hand.remove(x); q.gy.append(x); on_cast(g, q, x)
    log(f'    {NAME(q)} casts {x.name}: Hullbreaker Horror returns {c.name} to its owner\'s hand', g)
    return True
full('Hullbreaker Horror', 'flash, can\'t be countered; each of your spells bounces an opposing spell (as a counter) '
     'or nonland permanent')


# ================================================================== Valakut Exploration, Light Up the Stage, Nissa, Finale
@on('Valakut Exploration', 'end_step')
def _valakut_end2(g, src, p):
    """at your end step, each card exiled with it that you didn't play goes to the graveyard: 1 damage each"""
    if p is not src.owner: return
    held = getattr(p, 'valakut_cards', [])
    left = []
    for c in held:
        if c in p.hand: p.hand.remove(c); p.gy.append(c); left.append(c)
    if left:
        for q in g.opps(p): lose_life(g, q, len(left), p, kind='burn', damage=True)
    p.valakut_cards = []


@on('Valakut Exploration', 'landfall')
def _valakut_land2(g, src, p):
    if p is not src.owner or not p.library: return
    c = p.library.pop(); p.hand.append(c); p.seen_names.add(c.name)
    p.valakut_cards = getattr(p, 'valakut_cards', []) + [c]
full('Valakut Exploration', 'landfall: exile the top card, playable until end of turn; unplayed ones go to the '
     'graveyard at end step, 1 damage each to each opponent')

full('Light Up the Stage', 'spectacle {R}; exile the top two, playable until the end of your next turn')
full('Nissa, Resurgent Animist', 'landfall mana; the second landfall each turn reveals until an Elf or Elemental '
     '(the rest to the bottom)')


@IC.spell('Finale of Devastation', prio=lambda g, p, c: 60 if total_mana(g, p) >= 5 else 0, types='S',
          status=('Full', 'X = spare mana: a creature with MV X or less from library or graveyard onto the battlefield; '
                          'X >= 10: creatures +X/+X and haste'))
def _finale(g, p, c, ctx):
    x = ctx.get('x', 0)
    gy = [y for y in p.gy if y.creature and y.cmc <= x]
    from commander_sim.cards.impl import t3 as impl_t3
    got = impl_t3._put_creature(g, p, lambda y: y.cmc <= x)
    if got is None and gy:
        y = max(gy, key=lambda y: (card_worth(g, p, y), y.cmc)); p.gy.remove(y); enter(g, p, y)
    if x >= 10:
        for m in p.perms:
            if m.creature: _eot(g, m, x, x); m.sick = False
card('Finale of Devastation', 'xtutor', types='S', dsl=[])


# ================================================================== Sakura-Tribe Elder: chump, then sacrifice
@on('Sakura-Tribe Elder', 'blocks')
def _ste_block(g, src, p, atk, d, assign):
    if src.owner is d and src in assign.values() and src in d.perms:
        die(g, src, 'sac'); land_ramp(g, d, 1, True)
        log(f'    {NAME(d)} sacrifices Sakura-Tribe Elder after blocking', g)


@on('Sakura-Tribe Elder', 'options')
def _ste_eot(g, src, p, s, post):
    if p is not src.owner or post is not None: return []

    def go():
        if src not in p.perms: return False
        die(g, src, 'sac'); land_ramp(g, p, 1, True); return True
    return [(1.5, 'sacrifice Sakura-Tribe Elder', go)]
card('Sakura-Tribe Elder', 'pow=1 tgh=1', dsl=[])
full('Sakura-Tribe Elder', 'blocks, then is sacrificed for a basic land (or at the end of an opponent\'s turn)')


# ================================================================== Farseek, Springbloom Druid, Knight of the White Orchid, Spire of Industry
@IC.spell('Farseek', prio=lambda g, p, c: 80 if p.turns <= 4 else 40, types='S',
          status=('Full', 'a Plains, Island, Swamp or Mountain card (duals included) onto the battlefield tapped'))
def _farseek(g, p, c, ctx):
    types = ('plains', 'island', 'swamp', 'mountain')
    cs = [x for x in searchable(g, p) if x.land and (x.name.lower() in types or set(x.subtypes) & set(types))]
    if cs:
        x = max(cs, key=lambda x: len(x.tags.get('c', ''))); p.library.remove(x); g.rng.shuffle(p.library)
        p.lands.append(Land(x, True)); log(f'    Farseek finds {x.name}', g)


@on('Springbloom Druid', 'etb')
def _springbloom(g, src, p, m):
    if m is not src or len(src.owner.lands) < 3: return
    o = src.owner
    L = min(o.lands, key=lambda L: (L.cd.name not in ('Forest', 'Island', 'Plains', 'Swamp', 'Mountain'), not L.tapped))
    o.lands.remove(L); o.gy.append(L.cd)
    land_ramp(g, o, 2, True)
card('Springbloom Druid', 'pow=1 tgh=1', dsl=[])
full('Springbloom Druid', 'on entry: sacrifice a land for two basics tapped')


@on('Knight of the White Orchid', 'etb')
def _kotwo(g, src, p, m):
    if m is not src: return
    o = src.owner
    if any(len(q.lands) > len(o.lands) for q in g.opps(o)):
        cs = [x for x in searchable(g, o) if x.land and (x.name == 'Plains' or 'plains' in x.subtypes)]
        if cs: x = cs[0]; o.library.remove(x); g.rng.shuffle(o.library); o.lands.append(Land(x, False))
card('Knight of the White Orchid', 'pow=2 tgh=2', dsl=[], kws={'first strike'})
full('Knight of the White Orchid', 'first strike; a Plains onto the battlefield if an opponent has more lands')

full('Spire of Industry', '{C}; any colour for 1 life while you control an artifact')


# ================================================================== the rest: AI decisions on complete cards
AI_DECISIONS = {
    'Lightning Greaves': 'equip {0}: shroud and haste (moved onto the commander / best creature)',
    'Swiftfoot Boots': 'equip {1}: hexproof and haste (on the commander / best creature)',
    'Carrion Feeder': 'free sacrifice outlet: +1/+1 counter each time; can\'t block',
    'Cartel Aristocrat': 'free sacrifice outlet: protection from a colour (used against removal)',
    'Viscera Seer': 'free sacrifice outlet: scry 1 each time',
    'Woe Strider': 'Goat on entry; free sacrifice outlet with scry 1; escape',
    'Toxic Deluge': 'X = the biggest opposing toughness worth killing (max 10)',
    'Skullclamp': 'equip {1} onto 1-toughness creatures to draw two, up to twice a turn',
    'Heroic Intervention': 'hexproof and indestructible for your permanents, cast in response to removal or wipes',
    'Esper Sentinel': 'opponents pay X for their first noncreature spell only when they can spare it; otherwise you draw',
    'Selfless Spirit': 'sacrificed to make your creatures indestructible against destroy effects',
    'Deflecting Swat': 'free with your commander: redirects targeted removal',
    'Fertile Ground': 'the enchanted land taps for an extra mana of any colour',
    'Overgrowth': 'the enchanted land taps for an extra GG',
    'Utopia Sprawl': 'the enchanted Forest taps for an extra mana of the chosen colour',
    'Wild Growth': 'the enchanted land taps for an extra G',
    'Orcish Bowmasters': '1 damage on entry and per extra draw of an opponent, amassing an Orc Army',
    'Flawless Maneuver': 'free with your commander: your creatures indestructible (against destroy removal and wipes)',
    'Deadly Dispute': 'sacrifice an artifact or creature: draw two and a Treasure',
    "Teferi's Protection": 'phases out everything you control (against removal and wipes)',
    'Giver of Runes': 'protection for another creature in response to targeted removal',
    'Mother of Runes': 'protection from a colour in response to targeted removal',
    'Tireless Provisioner': 'landfall: a Treasure (preferred to a Food)',
    'Brazen Borrower': 'Petty Theft bounces a threat, then the Faerie is cast from exile; flash, flying',
    'Survival of the Fittest': 'discard a creature (the worst, or one Meren can return) for the best creature',
    'Dauthi Voidwalker': 'shadow; opponents\' cards going to the graveyard are exiled with void counters; sacrifice to '
                         'play one free',
    'Walking Ballista': 'X = spare mana; pings X/1 creatures or lethal players',
    'Black Market Connections': 'each precombat main: Treasure, plus a card or a 3/2 depending on life',
    'Laelia, the Blade Reforged': 'haste; attacking exiles the top card, castable this turn; grows on exile',
    'Rest in Peace': 'graveyards exiled on entry; cards that would go to a graveyard are exiled (so nothing dies)',
    'Outpost Siege': 'Khans: an impulse card each upkeep',
    'Fable of the Mirror-Breaker // Reflection of Kiki-Jiki': 'I Goblin Shaman (Treasure on attack), II rummage two, III '
                                                              'Reflection copies a nonlegendary creature each turn',
    'Cabal Ritual': 'BBB (BBBBB with threshold) when mana is short',
    'Return of the Wildspeaker': 'draw per non-Human power, or Humans +3/+3, whichever is bigger',
    'Timberwatch Elf': 'taps to pump an attacker by the number of Elves',
    'Alseid of Life\'s Bounty': 'sacrificed for protection from a colour in response to removal',
    'Benevolent Bodyguard': 'sacrificed for protection from a colour in response to removal',
    'Life from the Loam': 'returns up to three lands; dredge 3 when short on lands',
    'Dream Trawler': 'flying, lifelink; draws on attack (+1/+0 per draw); discard for hexproof in response',
    "Elspeth, Sun's Champion": '+1 three Soldiers, -3 destroys power 4+, -7 emblem (+2/+2, flying)',
    'Teferi, Time Raveler': 'opponents cast only at sorcery speed; -3 bounce and draw (+1 has no use here)',
    'Academy Manufactor': 'Treasure / Food / Clue creation makes one of each instead',
    'Consecrated Sphinx': 'flying; may draw two whenever an opponent draws',
    'Shrieking Drake': 'flying; returns your best ETB creature (or itself) on entry',
    'Whitemane Lion': 'flash; returns your best ETB creature (or itself) on entry',
    'Kor Skyfisher': 'flying; returns your best ETB permanent (or itself) on entry',
    'Thalia, Heretic Cathar': 'first strike; opponents\' creatures and nonbasic lands enter tapped',
    'Birgi, God of Storytelling // Harnfel, Horn of Bounty': 'R per spell cast (the Harnfel face is never cast)',
    'Zealous Conscripts': 'haste; steals the best opposing permanent until end of turn (untapped)',
    "Urza's Saga": 'taps for C; II makes Constructs; III tutors a 0/1-cost artifact, then it is sacrificed',
    "Caesar, Legion's Emperor": 'attacks: sacrifice spare fodder for two tokens, damage or cards',
    "Outlaws' Merriment": 'a random hasty token each upkeep',
    "Tilonalli's Summoner": 'attacks: X attacking Elementals (exiled at end of combat unless you have the city\'s blessing)',
    'Tendershoot Dryad': 'a Saproling each upkeep; Saprolings +2/+2 with the city\'s blessing',
    'Felidar Umbra': 'lifelink, umbra armor',
    'Flickering Ward': 'protection from a colour; returns to hand to reuse',
    'Gods Willing': 'protection from a colour in response to targeted removal',
    'Shielded by Faith': 'indestructible',
    'Spirit Link': 'you gain life equal to the enchanted creature\'s damage',
    'Timely Ward': 'indestructible (flash on a commander)',
    'Valorous Stance': 'indestructible in response, or destroy toughness 4+',
    'Uro, Titan of Nature\'s Wrath': 'enters: 3 life, draw, a land (sacrificed unless escaped); escape as a 6/6',
    'Deadeye Navigator': 'soulbond: {1}{U} blinks the paired ETB creature',
    'Ephemerate': 'blinks a creature (protection or value), rebound',
    'Spark Double': 'enters as a copy of your best creature or planeswalker with a +1 counter',
    'Angel of Serenity': 'exiles up to three opposing creatures until it leaves',
    'Demonlord Belzenlok': 'flying, trample; reveals until a nonland card (again for MV 4+)',
    'Drakuseth, Maw of Flames': 'flying; attacks: 4 damage and 3 and 3',
    'Glorybringer': 'flying, haste; exert for 4 damage to a creature',
    'Whispersilk Cloak': 'equipped creature can\'t be blocked and has shroud',
    'Hour of Promise': 'two lands of a basic type onto the battlefield tapped',
    'The Gitrog Monster': 'upkeep land sacrifice, an extra land each turn, draws when lands go to the graveyard',
    'Grisly Salvage': 'reveal five: a land (or creature) to hand, the rest to the graveyard',
    'Sheoldred, Whispering One': 'swampwalk; your upkeep reanimation, opponents\' upkeep edict',
    "Ajani's Chosen": 'a 2/2 Cat per enchantment entering',
    'Elephant Grass': 'attack tax {2} (not for black creatures); cumulative upkeep paid while it matters',
    'Hallowed Haunting': 'a Spirit per enchantment cast; flying and vigilance at seven enchantments',
    'Atraxa, Praetors\' Voice': 'flying, vigilance, deathtouch, lifelink; proliferate at your end step',
    'Ichormoon Gauntlet': 'planeswalkers have a proliferate ability; proliferate on noncreature spells',
    'Port Razer': 'combat damage: untap all, an additional combat (once per player hit)',
    'Savage Beating': 'an additional combat (and double strike when entwined)',
    'Kogla, the Titan Ape': 'fights on entry; destroys an artifact or enchantment on attack',
    'Braids, Arisen Nightmare': 'end step: sacrifice a spare creature; each opponent sacrifices a creature or loses 2 (you draw)',
    'Tinybones, Trinket Thief': 'end step after an opponent discards: plays the best cheap permanent from their graveyard',
    'Aluren': 'creature spells with MV 3 or less are free (everyone; cast at sorcery timing)',
    'Archon of Emeria': 'one spell per turn; opponents\' nonbasic lands enter tapped',
    'Lavinia, Azorius Renegade': 'opponents can\'t cast noncreature spells above their land count; free spells are countered',
    'Battle Hymn': 'R per creature you control',
    'Goblin Recruiter': 'stacks the best Goblins on top',
    'Goblin Sharpshooter': 'untaps whenever a creature dies: 1 damage per death',
    'Siege-Gang Commander': 'three Goblins; {1}{R}, sacrifice a Goblin: 2 damage',
    'Skirk Prospector': 'sacrifice a Goblin: R',
    'Tempt with Vengeance': 'X hasty Elementals (opponents decline the offer)',
    'Mahadi, Emporium Master': 'end step: a Treasure per creature that died this turn',
    'Storm-Kiln Artist': 'a Treasure per instant/sorcery you cast or copy',
    'Xorn': 'Treasure creation makes one more',
    'Gudul Lurker': 'can\'t be blocked',
    'Karn, the Great Creator': 'opponents\' artifact abilities off; -2 wishes an artifact from exile (no sideboard)',
    'Kappa Cannoneer': 'ward {4}; +1/+1 and unblockable this turn when an artifact enters',
    'Signal Pest': 'battle cry; only creatures with flying or reach can block it',
    'Solitary Confinement': 'skip your draw step, prevent all damage to you, discard each upkeep (or sacrifice)',
}
for _n, _t in AI_DECISIONS.items():
    full(_n, _t)

# combos whose loop is resolved as its end result (assembly, casting and every response window are simulated)
COMBO_PIECES = ('Basalt Monolith', 'Grim Monolith', 'Power Artifact', 'Rings of Brighthearth', 'Dramatic Reversal',
                "Thassa's Oracle", 'Restoration Angel', 'Butcher Ghoul', "Geralf's Messenger", 'Mikaeus, the Unhallowed',
                'Freed from the Real', "Pemmin's Aura", 'Demonic Consultation', 'Tainted Pact', 'Mycosynth Lattice',
                'Thornbite Staff')
from commander_sim.cards.impl import combos as impl_combos
for _n in sorted(set(COMBO_PIECES) | {n for n in impl_combos.PIECES
                                      if 'executed abstractly' in importlib.import_module('commander_sim.cards.pool_cards').NOTES.get(n, ('', ''))[1]}):
    from commander_sim.cards import pool_cards
    st, txt = pool_cards.NOTES.get(_n, ('Full', ''))
    note(_n, 'Full', (txt + '; ' if txt and 'loop is executed abstractly' not in txt else '') +
         'combo piece: assembly, casting and responses are simulated; the loop itself resolves as its result')


# ================================================================== sacrifice outlets' own effects
def _seer(g, p, src, m):
    from commander_sim.cards.impl import topdeck as impl_topdeck; impl_topdeck.scry(g, p, 1)


IC.SAC_OUTLET['Viscera Seer'] = _seer
IC.SAC_OUTLET['Woe Strider'] = _seer
card('Carrion Feeder', 'pow=1 tgh=1 noblock', dsl=[])


@on('Woe Strider', 'gy_options')
def _woe_escape(g, c, p, s, post):
    """escape {3}{B}{B}, exile four other cards: returns with two +1/+1 counters"""
    if post is not False or not can_pay(g, p, 3, 'BB') or len([x for x in p.gy if x is not c]) < 4: return []

    def go():
        if c not in p.gy or not can_pay(g, p, 3, 'BB'): return False
        pay(g, p, 3, 'BB')
        for x in sorted([x for x in p.gy if x is not c], key=lambda x: card_worth(g, p, x, True))[:4]:
            p.gy.remove(x); p.exile.append(x)
        p.gy.remove(c); m = enter(g, p, c, plus=2); log(f'  {NAME(p)} escapes Woe Strider', g); return True
    return [(1.6, 'escape Woe Strider', go)]


from commander_sim.ai import pool_ai
pool_ai.PROTECTORS['Cartel Aristocrat'] = ('bf_sac_other', None, 'color', 'self', False)


@on('Ichormoon Gauntlet', 'cast')
def _ichor(g, src, caster, c):
    if caster is src.owner and not c.creature and not c.land: IC.proliferate(g, caster)
full('Ichormoon Gauntlet', 'planeswalkers have 0: proliferate; each noncreature spell you cast proliferates')
def prot_vs(g, m, other):
    """m has protection from creature `other` (Spirit Mantle / Unquestioned Authority: all creatures; Baneslayer:
    Demons and Dragons): no damage from it"""
    if getattr(g, 'auras', None) and any(a.cd.name in ('Spirit Mantle', 'Unquestioned Authority') for a in IC.auras_on(g, m)):
        return True
    if R.dovin_blocked(m) or R.dovin_blocked(other): return True        # Dovin: no damage to or from it
    return m.cd is not None and m.cd.name == 'Baneslayer Angel' and (E.has_type(other, 'demon') or E.has_type(other, 'dragon'))


def prot_unblockable(g, b, a):
    return (a.cd is not None and a.cd.name == 'Baneslayer Angel' and (E.has_type(b, 'demon') or E.has_type(b, 'dragon')))


full('Spirit Mantle', '+1/+1 and protection from creatures (unblockable; no damage from creatures)')
full('Unquestioned Authority', 'draw on entry; protection from creatures (unblockable; no damage from creatures)')
full('Baneslayer Angel', 'flying, first strike, lifelink; protection from Demons and Dragons')
