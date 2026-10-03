"""Card rules for cards in the main decks (decklists/mine/) that need more than their hand tags: equipment, lands with
abilities, and cards whose full text the tag model left out. Hooks here are live in every game (pool rules).
"""
from commander_sim.engine import *
from commander_sim import engine as E
from commander_sim.cards import cardimpl as CI
from commander_sim.cards.cardimpl import on
from commander_sim.cards.pool_cards import card, note


def full(name, text): note(name, 'Full', text)


# ================================================================== Sephiroth
# ------------------------------------------------------------------ Displacer Kitten
ETB_VALUE = {'atraxa': 8.0, 'archon': 6.0, 'rsd': 4.0, 'titan': 3.0, 'witness': 2.5, 'wall': 2.0, 'wurm': 4.0,
             'bowmasters': 2.5, 'skate': 3.0}


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
    if E.human_choice(g, p) is None and etb_value(g, p, max(cands, key=lambda x: etb_value(g, p, x))) < 2.0: return
    if not trigger_window(g, p, src, 'flicker a permanent'): return
    cands = [m for m in cands if m in p.perms]
    if not cands: return
    hc = E.human_choice(g, p)
    if hc is not None:                                       # practice mode: up to one, your pick
        k = hc.choose(g, p, 'target', 'Displacer Kitten: flicker one of your nonland permanents?',
                      [hc.legal.describe_target(g, p, x) for x in cands], cancel='none')
        if k is None: return
        m = cands[k]; cd, was_cmd = m.cd, m.is_cmd
        log(f'    Displacer Kitten flickers {m.name}', g)
        leave(g, m); n = enter(g, p, cd, orig=p); n.is_cmd = was_cmd
        return
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
    if m.cd is p.cmd: return
    if E.human_choice(g, p) is None and (not can_pay(g, p, 4, '') or (pval(g, m) < 3 and etb_value(g, p, m) < 2.5)
                                         or any(cd is m.cd for _, cd, _ in getattr(g, 'marchesa_due', None) or ())): return
    if not trigger_window(g, p, src, f'pay {{4}}: return {m.name}'): return
    hc = E.human_choice(g, p)
    if hc is not None:                                       # practice mode: you may pay {4}
        if m.cd is p.cmd or not hc.pay_tax(g, p, 4, f'Nim Deathmantle: return {m.name} and attach it'): return
        if m.cd not in p.gy: return
        p.gy.remove(m.cd); n = enter(g, p, m.cd, orig=p); src.attached = n
        log(f'    {NAME(p)} pays 4: Nim Deathmantle returns {n.name}', g); return
    if m.cd is p.cmd or not can_pay(g, p, 4, ''): return
    if any(cd is m.cd for _, cd, _ in getattr(g, 'marchesa_due', None) or ()): return   # Marchesa returns it free
    if pval(g, m) < 3 and etb_value(g, p, m) < 2.5: return
    pay(g, p, 4, '')
    if m.cd not in p.gy:              # left the graveyard while paying (a Treasure's sacrifice set off a sweep): no return
        log(f'    {NAME(p)} pays 4, but {m.name} has left the graveyard', g)
        return
    p.gy.remove(m.cd)
    n = enter(g, p, m.cd, orig=p)
    src.attached = n
    log(f'    {NAME(p)} pays 4: Nim Deathmantle returns {n.name}', g)


@on('Nim Deathmantle', 'options')
def _nim_equip(g, src, p, s, post):
    """equip {4}: onto your best creature (a bomb first)"""
    if p is not src.owner or post is None or not can_pay(g, p, 4, ''): return []
    if src.attached is not None and src.attached in p.perms and not src.attached.phased: return []
    cre = [m for m in p.perms if m.creature and not m.phased and m.cd is not None
           and not (m.cd.name == 'Triskelion' and has(p, 'mikaeus'))]      # +2/+2 would stop its loop needing no outlet
    if not cre: return []
    t = max(cre, key=lambda m: pval(g, m))

    def go():
        if not can_pay(g, p, 4, '') or t not in p.perms: return False
        log(f'  {NAME(p)} equips Nim Deathmantle to {t.name}', g)
        return equip_to(g, p, src, t, 4)
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
        if not ability_window(g, p, src, '1 damage', imp=8 if lethal else 3): return True
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
    from commander_sim.cards.impl import lands as impl_lands, fixes as impl_fixes
    tg = [(q, x) for q in g.opps(p) for x in q.lands if x.cd.name in impl_lands.KEY_LANDS]
    if not tg: return []
    q, x = max(tg, key=lambda t: (t[1].cd.name in ("Gaea's Cradle", 'Cabal Coffers', 'Serra\'s Sanctum', 'Tolarian Academy',
                                                   'Itlimoc, Cradle of the Sun', 'Nykthos, Shrine to Nyx'), threat(g, p, t[0])))

    def go():
        if L not in p.lands or x not in q.lands: return False
        p.lands.remove(L); p.gy.append(L.cd)
        log(f'  {NAME(p)} sacrifices Strip Mine: destroy {x.cd.name} ({NAME(q)})', g)
        if ability_window(g, p, L.cd, f'destroy {x.cd.name}') and x in q.lands: impl_fixes.destroy_land(g, q, x)
        return True
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
    from commander_sim import ais as A
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
from commander_sim.cards import pool_cards as _PC
_PC.POST.append(_post_veyran)


def _you(g, p):
    """practice mode: the human seat's versions of these cards' choices (play/cards.py), or None for the AI"""
    return importlib.import_module('commander_sim.play.cards') if E.human_choice(g, p) is not None else None


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


# ------------------------------------------------------------------ Muldrotha, the Gravetide
MULD_TYPES = (('L', 'land'), ('C', 'creature'), ('A', 'artifact'), ('E', 'enchantment'), ('P', 'planeswalker'), ('B', 'battle'))


def muldrotha_on(g, p):
    """during each of your turns, a land and a permanent spell of each permanent type from your graveyard"""
    return g.active is p and any(m.cd is not None and m.cd.name == 'Muldrotha, the Gravetide' and not m.phased
                                 and not m.neutered for m in p.perms)


def muld_used(g, p):
    st = turn_stamp(g)
    v = getattr(p, 'muld_used', None)
    if not v or v[0] != st: p.muld_used = v = (st, set())
    return v[1]


def muld_types(g, p, c):
    """the permanent types of graveyard card c that Muldrotha still lets p play this turn"""
    if not muldrotha_on(g, p) or c not in p.gy: return []
    used = muld_used(g, p)
    if c.land: return [] if 'L' in used else ['L']
    if not c.perm: return []
    return [t for t, _ in MULD_TYPES if t != 'L' and t in c.types and t not in used]


def muld_mark(g, p, t):
    muld_used(g, p).add(t)


@on('Muldrotha, the Gravetide', 'options')
def _muldrotha(g, src, p, s, post):
    """the AI: cast the best permanent card from the graveyard whose type is still unused this turn"""
    if p is not src.owner or post is None or not muldrotha_on(g, p): return []
    from commander_sim import ais
    out = []
    for c in list(p.gy):
        ts = muld_types(g, p, c)
        if not ts or c.land or not castable(g, p, c, 'gy'): continue
        gen, pips = cost_of(p, c)
        if not can_pay(g, p, gen, pips): continue
        v = ais.deck_prio(g, p, c) / 10.0 + 0.3 * card_worth(g, p, c, in_gy=True) / 10.0
        if v <= 0.5: continue

        def go(c=c, t=ts[0], gen=gen, pips=pips):
            if c not in p.gy or t not in muld_types(g, p, c) or not can_pay(g, p, gen, pips): return False
            pay(g, p, gen, pips); muld_mark(g, p, t)
            log(f'  {NAME(p)} casts {c.name} from the graveyard (Muldrotha)', g)
            cast_card(g, p, c, 'mgy', {})
            return True
        out.append((v, f'{c.name} from the graveyard (Muldrotha)', go))
    return out
CI.muldrotha_on, CI.muld_types, CI.muld_mark, CI.muld_used, CI.MULD_TYPES = muldrotha_on, muld_types, muld_mark, muld_used, MULD_TYPES
card('Muldrotha, the Gravetide', 'pow=6 tgh=6 leg', dsl=[])
full('Muldrotha, the Gravetide', 'during each of your turns, play a land and cast a permanent spell of each permanent type '
     '(artifact, creature, enchantment, planeswalker, battle) from your graveyard; a countered one goes back to the graveyard')


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
    if not trigger_window(g, p, src, f'copy {c.name} {n} time(s)', imp=6): return
    log(f'    Thousand-Year Storm copies {c.name} {n} time(s)', g)
    for _ in range(n):
        copy_spell(g, p, c, ctx)
        if g.over: return


@on('Thousand-Year Storm', 'copycast')
def _storm_copycast(g, src, p, effect):
    """a cast copy of a back-face spell (prepared, Lightning Bolt) is an instant/sorcery cast too"""
    if p is not src.owner or src.phased or not _storm_n(g, p): return
    if not trigger_window(g, p, src, 'copy the spell', imp=6): return
    for _ in range(_storm_n(g, p)):
        magecraft(g, p, copy=True); effect()
        if g.over: return
card('Thousand-Year Storm', 'tys', types='E', dsl=[])
full('Thousand-Year Storm', 'each instant/sorcery you cast (or cast as a copy) is copied once per instant/sorcery cast '
     'before it this turn, twice with Veyran; copies pick new targets and keep X')


# ------------------------------------------------------------------ Alania, Divergent Storm
def alania_triggers(p):
    """Alania triggers once, +1 with Veyran (a cast instant or sorcery), +1 more with Harmonic Prodigy (a Wizard)"""
    return 1 + (1 if has(p, 'veyran') else 0) + (1 if has(p, 'prodigy') else 0)


def alania_pick(g, p, c, opps):
    """the AI: which opponent draws the card for a copy of c, or None to decline. A copied counterspell does nothing
    here, so it isn't worth a card; otherwise the least threatening opponent gets it"""
    if 'ctr' in c.tags: return None
    return min(opps, key=lambda q: threat(g, p, q))


@on('Alania, Divergent Storm', 'cast')
def _alania(g, src, caster, c):
    """your first instant and your first sorcery each turn (any turn): for each trigger, you may have target opponent
    draw a card; if you do, copy the spell (new targets, X kept). The 'first this turn' check counts spells cast before
    Alania was out too"""
    p = src.owner
    if caster is not p or not (c.instant or c.sorcery) or src.phased or src not in p.perms: return
    if casts_this_turn(g, p, (lambda x: x.instant) if c.instant else (lambda x: x.sorcery)) != 1: return
    cc = getattr(g, 'cur_cast', None)
    ctx = cc[1] if cc is not None and cc[0] is c else None           # X is copied (Crackle with Power)
    hc = E.human_choice(g, p)
    for _ in range(alania_triggers(p)):
        opps = g.opps(p)
        if not opps or g.over: return
        if hc is None and alania_pick(g, p, c, opps) is None: return
        if not trigger_window(g, p, src, f'copy {c.name}'): continue
        opps = g.opps(p)
        if not opps: return
        q = hc.alania(g, p, c, opps) if hc is not None else alania_pick(g, p, c, opps)
        if q is None:
            if hc is None: return                                     # the AI's answer is the same for each trigger
            continue
        draw(g, q, 1)
        log(f'    Alania: {NAME(q)} draws a card, {NAME(p)} copies {c.name}', g)
        copy_spell(g, p, c, ctx)


card('Alania, Divergent Storm', 'pow=3 tgh=5 leg noatk wizard alania', types='C', dsl=[])
full('Alania, Divergent Storm', 'your first instant and first sorcery each turn: for each trigger (Veyran and Harmonic '
     'Prodigy add one each) you may have the least threatening opponent draw a card to copy the spell (not a '
     'counterspell); copies pick new targets and keep X. The Otter clause is not modeled (no Otters in your decks)')


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
                src.loyalty -= 2
                log(f'  {NAME(p)} uses Ral, Storm Conduit -2: the next instant or sorcery is copied', g)
                if src.loyalty <= 0: leave(g, src); to_zone_card(g, src, 'gy')
                if ability_window(g, p, src.cd, '-2'): p.ral_copy = turn_stamp(g)
            else:
                src.loyalty += 2
                if ability_window(g, p, src, '+2'):
                    from commander_sim.cards.impl import topdeck as impl_topdeck; impl_topdeck.scry(g, p, 1)
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
        cast_copy(g, p, _prepared_effect(g, p, src.cd.name), name=spell, instant=instant)
        return True
    return [(v, f'{spell} (prepared copy)', go)]


for _n in PREPARED: on(_n, 'options')(_prepared_opt)


@on('Emeritus of Ideation // Ancestral Recall', 'attack')
def _ideation_attack(g, src, p, atk, d):
    """whenever it attacks: exile eight cards from your graveyard to prepare it again"""
    if src.owner is not p or src not in atk or (src.data and src.data.get('prepared')) or len(p.gy) < 8: return
    if not trigger_window(g, p, src, 'exile eight cards to prepare it'): return
    if len(p.gy) < 8: return
    ex = sorted(p.gy, key=lambda c: (c.instant or c.sorcery, card_worth(g, p, c, in_gy=True)))[:8]
    if _you(g, p):
        ex = _you(g, p).pick_exile(g, p, p.gy, 8, 'Emeritus of Ideation attacks: exile eight cards from your graveyard '
                                   'to prepare it?', default=ex)
        if not ex: return
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
    from commander_sim.cards.impl import topdeck as impl_topdeck
    return impl_topdeck.desire(g, p, c)


@on('Expressive Iteration', 'resolve')
def _iteration(g, p, c, ctx):
    """top three: one to hand, one to the bottom, one exiled (playable this turn)"""
    if _you(g, p): return _you(g, p).expressive_iteration(g, p, c, ctx)
    top = sorted(_top(p, 3), key=lambda x: -_desire(g, p, x))
    if not top: return
    p.hand.append(top[0])
    if len(top) >= 2: p.hand.append(top[1]); p.impulse.append(top[1])     # exiled, may play it this turn
    if len(top) >= 3: p.library.insert(0, top[2])


@on('Flow State', 'resolve')
def _flow(g, p, c, ctx):
    """top three: one to hand (two with an instant and a sorcery in the graveyard), the rest on the bottom"""
    k = 2 if (any(x.instant for x in p.gy) and any(x.sorcery for x in p.gy)) else 1
    if _you(g, p): return _you(g, p).look_and_take(g, p, 3, k, 'Flow State')
    top = sorted(_top(p, 3), key=lambda x: -_desire(g, p, x))
    p.hand.extend(top[:k]); p.library[:0] = top[k:]


@on('Stock Up', 'resolve')
def _stock(g, p, c, ctx):
    """top five: two to hand, the rest on the bottom"""
    if _you(g, p): return _you(g, p).look_and_take(g, p, 5, 2, 'Stock Up')
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
    from commander_sim.cards.impl import topdeck as impl_topdeck; impl_topdeck.scry(g, p, 1, to='gy')


@on('Prismari Charm', 'resolve')
def _charm(g, p, c, ctx):
    """one mode: 1 damage to each of one or two targets (X/1 creatures, a player at 1), bounce an opposing commander or
    big token, or surveil 2 then draw"""
    if _you(g, p): return _you(g, p).prismari_charm(g, p, c, ctx)
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
        from commander_sim.cards.impl import topdeck as impl_topdeck; impl_topdeck.scry(g, p, 2, to='gy'); draw(g, p, 1)


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
    from commander_sim.cards.impl import lands as IL
    if post is not None or L.tapped or g.active is p or not IL.can_pay_without(g, p, L, 1, 'UR'): return []

    def go():
        if L not in p.lands or L.tapped or not IL.pay_without(g, p, L, 1, 'UR'): return False
        L.tapped = True
        log(f'  {NAME(p)} loots with Desolate Lighthouse', g)
        if ability_window(g, p, L.cd, 'draw, then discard'): draw(g, p, 1); discard_worst(g, p, 1)
        return True
    return [(1.2, 'Desolate Lighthouse loot', go)]


@on('Spectacle Summit', 'land_options')
def _summit(g, L, p, s, post):
    """{2}{U}{R}, {T}: surveil 1 (with mana left over at the end of an opponent's turn)"""
    from commander_sim.cards.impl import lands as IL
    if post is not None or L.tapped or g.active is p or not IL.can_pay_without(g, p, L, 2, 'UR'): return []

    def go():
        if L not in p.lands or L.tapped or not IL.pay_without(g, p, L, 2, 'UR'): return False
        L.tapped = True
        if ability_window(g, p, L.cd, 'surveil 1'):
            from commander_sim.cards.impl import topdeck as impl_topdeck; impl_topdeck.scry(g, p, 1, to='gy')
        return True
    return [(0.6, 'Spectacle Summit surveil', go)]


full('Desolate Lighthouse', '{T}: {C}; {1}{U}{R}, {T}: loot at the end of an opponent\'s turn with mana to spare')
full('Spectacle Summit', 'enters tapped; {T}: {U} or {R}; {2}{U}{R}, {T}: surveil 1 at the end of an opponent\'s turn')


# ================================================================== Sauron
def _post_sauron():
    E.DB['Mauhúr, Uruk-hai Captain'].kws = frozenset({'menace'})
    E.DB['Ral Zarek, Guest Lecturer'].start_loyalty = '3'
    E.DB["Vraska, Betrayal's Sting"].start_loyalty = '6'
_PC.POST.append(_post_sauron)


# ------------------------------------------------------------------ the Ring (emblem)
def ring_bearer(g, p):
    m = getattr(p, 'ring_bearer', None)
    return m if m is not None and m in p.perms and not m.phased else None


def ring_level(p):
    return getattr(p, 'ring_level', 0)


def is_legendary(g, m):
    if m.cd is not None and 'leg' in m.cd.tags: return True
    return ring_level(m.owner) >= 1 and ring_bearer(g, m.owner) is m          # the Ring: your Ring-bearer is legendary


def ring_tempt(g, p):
    """the Ring tempts you: the emblem gains its next ability, you choose a Ring-bearer (the Army first: it is the
    threat, and legendary it turns Champion's Helm on), then the 'tempts you' / 'choose a Ring-bearer' triggers"""
    p.ring_level = min(4, ring_level(p) + 1)
    cr = [m for m in p.perms if m.creature and not m.phased]
    hc = E.human_choice(g, p)                             # practice mode: the person makes the Ring's choices
    if cr:
        p.ring_bearer = hc.ring_bearer(g, p, cr) if hc is not None else \
            max(cr, key=lambda m: (m.army, epow(g, m) + 2 * etgh(g, m) / 5))
        log(f'    The Ring tempts {NAME(p)} (level {p.ring_level}): Ring-bearer {p.ring_bearer.name}', g)
        for m in find(p, 'callring'):                     # Call of the Ring: you may pay 2 life to draw a card
            if (hc.yes_no(g, p, 'Call of the Ring: pay 2 life to draw a card?') if hc is not None else p.life > 10):
                lose_life(g, p, 2, p); draw(g, p, 1)
    for m in find(p, 'sauron'):                           # Sauron, the Dark Lord: you may discard your hand, draw four
        if hc is not None:
            if hc.yes_no(g, p, f'Sauron: discard your hand ({len(p.hand)} cards) and draw four?'):
                discard_cards(g, p, list(p.hand)); draw(g, p, 4)
                log(f'    {NAME(p)} discards the hand and draws four (Sauron)', g)
            break
        if len(p.hand) <= 3:
            discard_cards(g, p, list(p.hand)); draw(g, p, 4)
            log(f'    {NAME(p)} discards the hand and draws four (Sauron)', g)
            break


def ring_attack(g, p, atk):
    """level 2: whenever your Ring-bearer attacks, draw a card, then discard a card"""
    b = ring_bearer(g, p)
    if b is not None and b in atk and ring_level(p) >= 2:
        draw(g, p, 1); discard_worst(g, p, 1)


def ring_unblockable(g, b, a):
    """level 1: your Ring-bearer can't be blocked by creatures with greater power"""
    return ring_level(a.owner) >= 1 and ring_bearer(g, a.owner) is a and epow(g, b) > epow(g, a)


def ring_blocked(g, p, a, b):
    """level 3: a creature blocking your Ring-bearer is sacrificed at end of combat"""
    return ring_level(p) >= 3 and ring_bearer(g, p) is a


def ring_damage(g, p, a, d):
    """level 4: your Ring-bearer's combat damage to a player makes each opponent lose 3 life"""
    if ring_level(p) >= 4 and ring_bearer(g, p) is a:
        for q in g.opps(p): lose_life(g, q, 3, p, kind='drain')


note('Call of the Ring', 'Full', 'upkeep: the Ring tempts you (the four Ring abilities in order); choosing a Ring-bearer '
     'pays 2 life for a card')
note('Sauron, the Dark Lord', 'Full', 'ward (sacrifice a legendary artifact or creature); amass Orcs 1 per opponent '
     'spell; Army combat damage tempts you; each tempt may discard the hand for four cards (with three or fewer)')
note('Ringsight', 'Full', 'the Ring tempts you, then a black, blue or red card to hand')


# ------------------------------------------------------------------ Champion's Helm
def helm_hexproof(g, m):
    return equipped(m, 'helm') and is_legendary(g, m)
full("Champion's Helm", 'equipped creature +2/+2, and hexproof while legendary (the Ring makes your Ring-bearer '
     'legendary); equip {1}, onto the Army')


# ------------------------------------------------------------------ Sauron, the Necromancer
def necromancer_attack(g, p, src):
    """exile a creature card from your graveyard: a tapped, attacking token copy that is a 3/3 black Wraith with menace;
    exiled at the next end step unless Sauron is your Ring-bearer"""
    cs = [c for c in p.gy if c.creature]
    if not cs: return []
    c = max(cs, key=lambda c: card_worth(g, p, c, in_gy=True) + c.pow)
    p.gy.remove(c); p.exile.append(c)
    t = enter_token_copy(g, p, c)
    if t is None: return []
    t.pow, t.tgh, t.plus, t.tapped, t.sick = 3, 3, 0, True, False
    t.name = f'Wraith ({c.name})'
    g.eot_kw.setdefault(id(t), set()).add('menace')
    t.temp = ring_bearer(g, p) is not src
    return [t]
full('Sauron, the Necromancer', 'menace; attacking exiles your best graveyard creature for a tapped attacking token '
     'copy (a 3/3 black Wraith with menace, its abilities kept), exiled at end step unless Sauron is Ring-bearer')


# ------------------------------------------------------------------ counters: Mauhúr, Metallic Mimic
def orcish(m):
    return m.army or (m.cd is not None and any(has_type(m, t) for t in ('orc', 'goblin')))


def add_counters(g, m, n):
    """+1/+1 counters on m; Mauhúr: one more on an Army, Goblin or Orc you control"""
    if n > 0 and orcish(m) and any(x.cd is not None and 'mauhur' in x.cd.tags and not x.phased for x in m.owner.perms): n += 1
    m.plus += n


@on('Metallic Mimic', 'etb')
def _mimic(g, src, p, m):
    """(Orc chosen) each other Orc you control enters with an additional +1/+1 counter"""
    if m is not src and m.owner is src.owner and m.creature and orcish(m): add_counters(g, m, 1)
full('Metallic Mimic', 'names Orc: each other Orc (and every new Orc Army) enters with an additional +1/+1 counter')
full('Mauhúr, Uruk-hai Captain', 'menace; every +1/+1 counter placement on an Army, Goblin or Orc you control gets one more')


# ------------------------------------------------------------------ Kindred Discovery, Reconnaissance Mission
def kindred_match(src, m):
    """is m of the type Kindred Discovery named (Orc: Orcs and Orc Armies)"""
    t = (src.data or {}).get('ctype', 'orc')
    return orcish(m) if t == 'orc' else has_type(m, t)


def kindred_enter(g, p, m):
    for src in find(p, 'kindred'):
        if kindred_match(src, m): draw(g, p, 1)


@on('Kindred Discovery', 'etb')
def _kindred_etb(g, src, p, m):
    if m is not src and m.owner is src.owner and m.creature and kindred_match(src, m) and trigger_window(g, src.owner, src, 'draw a card'):
        draw(g, src.owner, 1)


@on('Kindred Discovery', 'attack')
def _kindred_attack(g, src, p, atk, d):
    if p is src.owner:
        n = sum(1 for m in atk if kindred_match(src, m))
        if n and trigger_window(g, p, src, f'draw {n}'): draw(g, p, n)


@on('Reconnaissance Mission', 'combat_damage')
def _recon(g, src, p, a, d, dmg):
    if p is src.owner and trigger_window(g, p, src, 'draw a card'): draw(g, p, 1)


@on('Reconnaissance Mission', 'hand_options')
def _recon_cycle(g, c, p, s, post):
    """cycling {2} when few creatures would connect"""
    if post is not None or c not in p.hand or not can_pay(g, p, 2, ''): return []
    if sum(1 for m in p.perms if m.creature) >= 2: return []

    def go():
        if c not in p.hand or not can_pay(g, p, 2, ''): return False
        p.hand.remove(c); pay(g, p, 2, ''); p.gy.append(c); draw(g, p, 1); return True
    return [(1.0, 'cycle Reconnaissance Mission', go)]
full('Kindred Discovery', 'names a creature type as it enters (Orc for Sauron: Orc Armies count): draws whenever a '
     'creature of that type you control enters or attacks')
full('Reconnaissance Mission', 'draws for each creature of yours that deals combat damage to a player; cycling {2} '
     'with fewer than two creatures')


# ------------------------------------------------------------------ attack triggers: Iron Man, War Machine
def modified(g, m):
    return m.plus != 0 or any(e.attached is m for e in m.owner.perms) or bool(getattr(g, 'auras', None) and CI.auras_on(g, m))


@on('Iron Man, Armored Avenger', 'attack')
def _ironman(g, src, p, atk, d):
    if src.owner is p and src in atk and any(m is not src and modified(g, m) for m in atk) \
            and trigger_window(g, p, src, 'modified attackers gain flying'):
        for m in atk:
            if m is not src and modified(g, m): g.eot_kw.setdefault(id(m), set()).add('flying')


@on('War Machine, Avenging Arsenal', 'attack')
def _warmachine(g, src, p, atk, d):
    if src.owner is p and src in atk and any(modified(g, m) for m in atk) \
            and trigger_window(g, p, src, 'modified attackers gain double strike'):
        for m in atk:
            if modified(g, m): g.eot_kw.setdefault(id(m), set()).add('double strike')
full('Iron Man, Armored Avenger', 'flying; a +1/+1 counter on the Army per card you draw; attacking gives your other '
     'attacking modified creatures flying')
full('War Machine, Avenging Arsenal', 'flying; attacking gives your attacking modified creatures double strike')


# ------------------------------------------------------------------ Kaervek the Merciless
def kaervek(g, k, caster, c):
    """an opponent casts a spell: damage equal to its mana value to any target (a creature it kills, else a face)"""
    n = c.cmc
    if n <= 0: return
    hc = E.human_choice(g, k)
    if hc is not None: return hc.deal_damage(g, k, n, 'Kaervek the Merciless')
    cr = [m for q in g.opps(k) for m in q.perms if m.creature and not untargetable(g, m) and etgh(g, m) <= n]
    best = max(cr, key=lambda m: pval(g, m)) if cr else None
    if best is not None and pval(g, best) >= 4: apply_removal(g, k, best, f'dmg{n}')
    else:
        opps = g.opps(k)
        if opps: lose_life(g, min(opps, key=lambda q: q.life - n), n, k, kind='triggers')
full('Kaervek the Merciless', 'each opponent spell: damage equal to its mana value to any target (the best creature it '
     'kills, else the lowest life total)')


# ------------------------------------------------------------------ Vision, Synthezoid Avenger
@on('Vision, Synthezoid Avenger', 'cast')
def _vision(g, src, caster, c):
    """a spell cast outside its caster's turn: phase out if the spell threatens Vision, else a +1/+1 counter"""
    if g.active is caster or src.phased or src not in src.owner.perms: return
    if not trigger_window(g, src.owner, src, 'phase out, or a +1/+1 counter'): return
    if src not in src.owner.perms: return
    cc = getattr(g, 'cur_cast', None)
    ctx = cc[1] if cc is not None and cc[0] is c else {}
    if caster is not src.owner and (ctx.get('target') is src or 'wipe' in c.tags): src.phased = True
    else: src.plus += 1
full('Vision, Synthezoid Avenger', 'flying; each spell cast outside its caster\'s turn: phases out if it threatens '
     'Vision, else a +1/+1 counter')


# ------------------------------------------------------------------ Scarlet Witch
@on('Scarlet Witch, Chaotic Avenger', 'combat_damage')
def _witch(g, src, p, a, d, dmg):
    """combat damage to a player: exile the top two face down, then cast a Hero or noncreature spell from among the
    cards exiled with her, free"""
    if a is not src: return
    if not trigger_window(g, p, src, 'exile the top two, cast one free'): return
    top = [p.library.pop() for _ in range(min(2, len(p.library)))]
    src.data = dict(src.data or {}); src.data.setdefault('witch', []).extend(top)
    p.exile.extend(top)
    ok = [c for c in src.data['witch'] if c in p.exile and not c.land and (not c.creature or 'hero' in c.subtypes)]
    if not ok: return
    c = max(ok, key=lambda c: card_worth(g, p, c))
    if _you(g, p):
        k = _you(g, p).choose(g, p, 'choose', 'Scarlet Witch: cast a spell from among the cards exiled with her, free?',
                              [x.name for x in ok], cancel='cast nothing')
        if k is None: return
        c = ok[k]
    p.exile.remove(c); src.data['witch'].remove(c)
    log(f'    Scarlet Witch: {NAME(p)} casts {c.name} free', g)
    cast_card(g, p, c, 'lib', spell_targets(g, p, c))
full('Scarlet Witch, Chaotic Avenger', 'flying; combat damage to a player exiles the top two, then casts the best Hero '
     'or noncreature card exiled with her for free')


# ------------------------------------------------------------------ planeswalkers: Vraska, Ral Zarek
def _pw_once(g, src):
    return not (src.data and src.data.get('act') == turn_stamp(g))


def _pw_use(g, src, cost):
    src.data = dict(src.data or {}, act=turn_stamp(g))
    src.loyalty += cost
    if src.loyalty <= 0: leave(g, src); to_zone_card(g, src, 'gy'); return False
    return True


@on("Vraska, Betrayal's Sting", 'options')
def _vraska(g, src, p, s, post):
    """0: draw, lose 1 life, proliferate. -2: a creature becomes a Treasure. -9: a player's poison goes to nine"""
    if p is not src.owner or post is None or src.phased or src.loyalty is None or not _pw_once(g, src): return []
    out = []

    def zero():
        if not _pw_once(g, src): return False
        _pw_use(g, src, 0)
        if ability_window(g, p, src, '0'): draw(g, p, 1); lose_life(g, p, 1, p); proliferate_all(g, p)
        return True
    out.append((2.0 if p.life > 10 else 0.5, "Vraska 0 (draw, proliferate)", zero))
    cr = [m for q in g.opps(p) for m in q.perms if m.creature and not untargetable(g, m)]
    if cr and src.loyalty >= 2:
        t = max(cr, key=lambda m: pval(g, m))

        def minus2():
            if not _pw_once(g, src) or t not in t.owner.perms: return False
            _pw_use(g, src, -2)
            log(f'  {NAME(p)} uses Vraska -2: {t.name} becomes a Treasure', g)
            if ability_window(g, p, src.cd, '-2', target=t) and t in t.owner.perms:
                q = t.owner; leave(g, t); q.treasures += 1
            return True
        out.append((pval(g, t) - 2.0, f'Vraska -2 -> {t.name}', minus2))
    if src.loyalty >= 9:
        q = max(g.opps(p), key=lambda o: threat(g, p, o)) if g.opps(p) else None
        if q is not None:
            def minus9():
                if not _pw_once(g, src): return False
                _pw_use(g, src, -9)
                if not ability_window(g, p, src.cd, '-9', imp=10): return True
                if not melira(q): q.poison = max(getattr(q, 'poison', 0), 9)
                log(f'  {NAME(p)} uses Vraska -9: {NAME(q)} has nine poison counters', g); return True
            out.append((12.0, f'Vraska -9 -> {NAME(q)}', minus9))
    return out


def proliferate_all(g, p):
    """proliferate: your Army and other creatures' +1/+1 counters, your loyalty, opponents' poison and -1/-1"""
    for m in p.perms:
        if m.phased: continue
        if m.plus > 0: add_counters(g, m, 1)
        if m.loyalty is not None and m.cd is not None and 'P' in m.cd.types: m.loyalty += 1
    for q in g.opps(p):
        if getattr(q, 'poison', 0) > 0 and not melira(q): q.poison += 1
        for m in list(q.perms):
            if m.creature and m.plus < 0:
                m.plus -= 1
                if etgh(g, m) <= 0: die(g, m, 'sba')
    check_state(g)


@on('Ral Zarek, Guest Lecturer', 'options')
def _ralz(g, src, p, s, post):
    """+1: surveil 2. -1: each opponent discards a card. -2: a creature card (mana value 3 or less) from your graveyard
    to the battlefield. -7: flip five coins, an opponent skips that many turns"""
    if p is not src.owner or post is None or src.phased or src.loyalty is None or not _pw_once(g, src): return []
    out = []

    def plus1():
        if not _pw_once(g, src): return False
        _pw_use(g, src, 1)
        if ability_window(g, p, src, '+1'):
            from commander_sim.cards.impl import topdeck as impl_topdeck; impl_topdeck.scry(g, p, 2, to='gy')
        return True
    out.append((1.0, 'Ral Zarek +1 (surveil 2)', plus1))
    hands = sum(1 for q in g.opps(p) if q.hand)
    if hands and src.loyalty >= 2:
        def minus1():
            if not _pw_once(g, src): return False
            _pw_use(g, src, -1)
            if ability_window(g, p, src.cd, '-1'):
                for q in g.opps(p):
                    if q.hand: discard_index(g, q, g.rng.randrange(len(q.hand)))
            return True
        out.append((0.8 + 0.5 * hands, 'Ral Zarek -1 (each opponent discards)', minus1))
    rc = [c for c in p.gy if c.creature and c.cmc <= 3]
    if rc and src.loyalty >= 2:
        c = max(rc, key=lambda c: card_worth(g, p, c, in_gy=True))

        def minus2():
            if not _pw_once(g, src) or c not in p.gy: return False
            _pw_use(g, src, -2)
            if ability_window(g, p, src.cd, '-2') and c in p.gy: p.gy.remove(c); enter(g, p, c)
            return True
        out.append((0.8 * card_worth(g, p, c, in_gy=True) / 2.0, f'Ral Zarek -2 -> {c.name}', minus2))
    if src.loyalty >= 7 and g.opps(p):
        q = max(g.opps(p), key=lambda o: threat(g, p, o))

        def minus7():
            if not _pw_once(g, src): return False
            _pw_use(g, src, -7)
            if not ability_window(g, p, src.cd, '-7', imp=10): return True
            n = sum(1 for _ in range(5) if g.rng.random() < 0.5)
            q.skip_turns = getattr(q, 'skip_turns', 0) + n
            log(f'  {NAME(p)} uses Ral Zarek -7: {NAME(q)} skips {n} turn(s)', g); return True
        out.append((8.0, f'Ral Zarek -7 -> {NAME(q)}', minus7))
    return out
full("Vraska, Betrayal's Sting", 'loyalty 6: 0 draws, costs 1 life and proliferates (opponents\' poison included); -2 '
     'turns the best opposing creature into a Treasure; -9 sets a player to nine poison')
full('Ral Zarek, Guest Lecturer', 'loyalty 3: +1 surveil 2; -1 each opponent discards; -2 returns a creature (mana '
     'value 3 or less); -7 an opponent skips 0-5 turns (five coins)')


# ------------------------------------------------------------------ Jace's Archivist
def archivist_worth(g, p):
    """the wheel: with Orcish Bowmasters out (every opponent draw pings), or when your hand is much smaller"""
    most = max((len(q.hand) for q in g.players if q.alive), default=0)
    return has(p, 'bowmasters') or (len(p.hand) <= 1 and most >= 4)
full("Jace's Archivist", '{U}, {T}: everyone discards and draws the largest hand size (used with Orcish Bowmasters, or '
     'when your hand is much smaller)')


# ------------------------------------------------------------------ lands: Barad-dûr, Plaza of Heroes, Unclaimed Territory, Talisman
@on('Barad-dûr', 'land_options')
def _baraddur(g, L, p, s, post):
    """{X}{X}{B}, {T}: amass Orcs X, only if a creature died this turn"""
    if L.tapped or post is None or getattr(g, 'died_turn', None) != turn_stamp(g): return []
    from commander_sim.cards.impl import lands as IL
    x = 0
    while IL.can_pay_without(g, p, L, 2 * (x + 1), 'B'): x += 1
    if x < 1: return []

    def go():
        if L.tapped or not IL.pay_without(g, p, L, 2 * x, 'B'): return False
        L.tapped = True; log(f'  {NAME(p)} uses Barad-dûr: amass Orcs {x}', g)
        if ability_window(g, p, L.cd, f'amass Orcs {x}'): amass(g, p, x)
        return True
    return [(1.0 + x, f'Barad-dûr (amass {x})', go)]


def plaza_colors(g, p):
    if PAY_FOR is not None and 'leg' in PAY_FOR.tags: return p.ident                 # a legendary spell: any colour
    return ''.join(sorted({x for m in p.perms if m.cd is not None and is_legendary(g, m) for x in m.cd.pips if x in 'WUBRG'}))


def territory_colors(g, p):
    return p.ident if PAY_FOR is not None and PAY_FOR.creature and 'orc' in PAY_FOR.subtypes else ''


full('Barad-dûr', 'enters tapped without a legendary creature; {T}: {B}; {X}{X}{B}, {T}: amass Orcs X after a creature died')
full('Plaza of Heroes', '{C}; any colour for legendary spells; colours among your legendary permanents; the exile '
     'protection is not used')
full('Unclaimed Territory', 'names Orc: {C}, or any colour for Orc creature spells')
full('Talisman of Creativity', '{T}: {C}, or {U}/{R} for 1 damage to you')


# ------------------------------------------------------------------ Bloodsoaked Insight // Sanguine Morass
def insight_cost(g, p):
    lost = sum(v for q in g.opps(p) for st, v in [getattr(q, 'lost_turn', (None, 0))] if st == turn_stamp(g))
    return max(0, 5 - lost)


@on('Bloodsoaked Insight // Sanguine Morass', 'hand_options')
def _insight(g, c, p, s, post):
    """cast the front face ({5}{B/R}{B/R}, {1} less per life opponents lost this turn): the top three of an opponent's
    library, playable until the end of your next turn; otherwise it is a land (Sanguine Morass)"""
    if post is None or c not in p.hand or g.active is not p: return []
    gen = insight_cost(g, p)
    if gen > 2 or not can_pay(g, p, gen, 'B') or not can_pay(g, p, gen + 1, 'B'): return []

    def go():
        if c not in p.hand or not can_pay(g, p, gen + 1, 'B'): return False
        p.hand.remove(c); pay(g, p, gen + 1, 'B')
        p.spells_this_turn += 1; on_cast(g, p, c)
        opps = g.opps(p)
        if opps:
            q = max(opps, key=lambda o: len(o.library))
            top = [q.library.pop() for _ in range(min(3, len(q.library)))]
            p.hand.extend(top)
            p.impulse_long = (getattr(p, 'impulse_long', None) or []) + [(x, p.turns + 1) for x in top]
            log(f'  {NAME(p)} casts Bloodsoaked Insight: {", ".join(x.name for x in top)} from {NAME(q)}', g)
        p.gy.append(c); return True
    return [(3.5 - gen, 'Bloodsoaked Insight', go)]
full('Bloodsoaked Insight // Sanguine Morass', 'a tapped B/R land, or (when opponents lost enough life this turn) the '
     'top three of an opponent\'s library, playable until the end of your next turn')


# the engine reaches these through the card-rules module
CI.add_counters = add_counters
CI.kindred_enter = kindred_enter
CI.kaervek = kaervek
CI.plaza_colors = plaza_colors
CI.territory_colors = territory_colors
CI.ring_tempt = ring_tempt


# ======================================================== Urabrask, Heretic Praetor
@on('Urabrask, Heretic Praetor', 'upkeep')
def _urabrask(g, src, p):
    """your upkeep: exile the top card, you may play it this turn; each opponent's upkeep: their next draw this turn is
    exiled instead, playable this turn (engine.draw)"""
    if p is src.owner:
        if p.library and trigger_window(g, p, src, 'exile the top card, playable this turn') and p.library:
            c = p.library.pop(); p.hand.append(c); p.impulse.append(c); p.seen_names.add(c.name)
            log(f'    Urabrask exiles {c.name} (playable this turn)', g)
    elif p.alive and trigger_window(g, src.owner, src, f"{NAME(p)}'s next draw is exiled"):
        p.urabrask = turn_stamp(g)


card('Urabrask, Heretic Praetor', 'leg pow=4 tgh=4 haste', dsl=[])
note('Urabrask, Heretic Praetor', 'Full', 'your upkeep: top card exiled, playable this turn; opponents: the next draw '
     'each upkeep is exiled instead and playable that turn (so it is not a draw); unplayed cards stay exiled')
note('Slaughter Pact', 'Full', 'destroy target nonblack creature for {0}; pay {2}{B} at your next upkeep or lose; the '
     'AI casts it only when it can pay that')
note("Champion's Helm", 'Full', 'equip {1}: +2/+2, hexproof while legendary; the AI equips the most valuable legendary '
     'creature (the Army once it is the Ring-bearer), else the Army')


# ======================================================== Kefka, Court Mage // Kefka, Ruler of Ruin
KEFKA = 'Kefka, Court Mage // Kefka, Ruler of Ruin'


def _kefka_wheel(g, src, p):
    """each player discards a card, then you draw a card for each card type among the discarded cards"""
    discarded = []
    for q in [x for x in g.players if x.alive]:
        if not q.hand: continue
        hc = E.human_choice(g, q)
        c = hc.pick_cards(g, q, q.hand, 1, 'Kefka: discard a card')[0] if hc is not None else \
            min(q.hand, key=lambda x: E.card_worth(g, q, x))          # each player gives up their least useful card
        E.discard_cards(g, q, [c]); discarded.append(c)
    kinds = {t for c in discarded for t in c.types if t in 'LCISAEP'}
    if kinds: draw(g, p, len(kinds))
    log(f"    Kefka: {', '.join(c.name for c in discarded) or 'nothing'} discarded, {NAME(p)} draws {len(kinds)}", g)


@on(KEFKA, 'etb')
def _kefka_etb(g, src, p, m):
    if m is src and trigger_window(g, src.owner, src, 'each opponent discards'): _kefka_wheel(g, src, p)


@on(KEFKA, 'attack')
def _kefka_attack(g, src, p, atk, d):
    if src in atk and not (src.data or {}).get('ruin') and trigger_window(g, p, src, 'each opponent discards'): _kefka_wheel(g, src, p)


@on(KEFKA, 'options')
def _kefka_ruin(g, src, p, s, post):
    """{8}, sorcery speed: each opponent sacrifices a permanent of their choice, then Kefka transforms"""
    if p is not src.owner or post is None or g.active is not p or (src.data or {}).get('ruin') or not can_pay(g, p, 8, ''):
        return []

    def go():
        if src not in p.perms or not can_pay(g, p, 8, ''): return False
        pay(g, p, 8, '')
        if not ability_window(g, p, src, '{8}: each opponent sacrifices a permanent', imp=7): return True
        for q in g.opps(p):
            toks = [m for m in q.perms if m.token and not m.phased]
            if toks: die(g, min(toks, key=lambda m: pval(g, m)), 'sac')          # their choice: the cheapest thing
            elif len(q.lands) > 4:
                L = q.lands.pop(); q.gy.append(L.cd)
            elif q.perms: die(g, min(q.perms, key=lambda m: pval(g, m)), 'sac')
            elif q.lands:
                L = q.lands.pop(); q.gy.append(L.cd)
        src.data = dict(src.data or {}, ruin=True)
        src.pow, src.tgh, src.fly = 5, 7, True
        log(f'  {NAME(p)} activates Kefka ({{8}}): each opponent sacrifices a permanent; Kefka transforms', g)
        return True
    return [(3.0 + 0.5 * len(g.opps(p)), 'Kefka: {8}, transform', go)]


@on(KEFKA, 'lose_life')
def _kefka_ruin_draw(g, src, q, n):
    """Kefka, Ruler of Ruin: whenever an opponent loses life during your turn, you draw that many cards"""
    p = src.owner
    if (src.data or {}).get('ruin') and q is not p and g.active is p and n > 0 and p.alive \
            and trigger_window(g, p, src, f'draw {n}'):
        draw(g, p, n)                                             # not optional, even if it decks you


card(KEFKA, 'leg human wizard pow=4 tgh=5 kefka', dsl=[])
note(KEFKA, 'Full', 'enters or attacks: each player discards their least useful card, you draw one per card type '
     'discarded; {8} at sorcery speed: each opponent sacrifices their cheapest permanent (a token, else a land when '
     'they have lands to spare), then it transforms: 5/7 flying, and draws as many cards as an opponent loses life '
     'on your turn (stopping five short of an empty library)')
note('Brush Off', 'Full', 'counter target spell; {1}{U} against an instant or sorcery')
note("Avacyn's Pilgrim", 'Full', 'mana dork: {T}: add {W}')


# ======================================================== Sephiroth's loops
# Each loop is run as its end result, after one window for opponents to answer a key piece (ais.combo_interrupted).
OUTLETS = ('Viscera Seer', "Ashnod's Altar", 'Altar of Dementia')     # free sacrifice outlets
DRAINS = ('Blood Artist', 'Zulaport Cutthroat')
LOOPS = (   # (name, piece groups (one card of each), 'kills' alone or needs a 'payoff')
    ('Mikaeus + Triskelion', (('Mikaeus, the Unhallowed',), ('Triskelion',)), 'kills'),      # outlet: trisk_groups
    ('Mikaeus + Kitchen Finks', (('Mikaeus, the Unhallowed',), ('Kitchen Finks',), OUTLETS), 'payoff'),
    ('Melira + Kitchen Finks', (('Melira, Sylvok Outcast',), ('Kitchen Finks',), OUTLETS), 'payoff'),
    ("Nim Deathmantle + Ashnod's Altar + Grave Titan", (('Nim Deathmantle',), ("Ashnod's Altar",), ('Grave Titan',)), 'payoff'),
)
LOOP_CARDS = {n for _, groups, _ in LOOPS for grp in groups for n in grp}


def trisk_needs_outlet(g, p, names):
    """Mikaeus + Triskelion needs no outlet: returned by undying with four counters, Triskelion pings itself twice and
    opponents twice, and dies as a 2/2 with 2 damage. Only a counterless toughness of 4 or more (Elesh Norn, Grand
    Cenobite, or Nim Deathmantle on it) makes it ping itself four times; then it takes a free sacrifice outlet"""
    t = next((m for m in p.perms if m.cd is not None and m.cd.name == 'Triskelion' and not m.phased), None)
    if t is not None: return etgh(g, t) - t.plus >= 4
    return 'Elesh Norn, Grand Cenobite' in names


def loop_groups(g, p, name, groups, names):
    """a loop's piece groups as they stand: Mikaeus + Triskelion adds an outlet only when it needs one"""
    return groups + (OUTLETS,) if name == LOOPS[0][0] and trisk_needs_outlet(g, p, names) else groups


def _names_bf(g, p):
    return {m.cd.name for m in p.perms if m.cd is not None and not m.phased and not m.neutered and not stopped(g, m.cd.name)}


def loop_kills(name, names):
    """does this loop win with what is on the battlefield (names)?"""
    kind = next(k for n, _, k in LOOPS if n == name)
    if kind == 'kills': return True
    if any(d in names for d in DRAINS) or 'Altar of Dementia' in names: return True      # drain, or mill everyone
    return name.startswith('Nim') and 'Triskelion' in names      # infinite mana: Deathmantle keeps returning Triskelion


def loops_blocked(g, p):
    """graveyard hate or Hushbringer: nothing that dies comes back, or its triggers don't happen"""
    from commander_sim.cards.impl import partials as IP
    return bool(g.hooks and CI.total(g, 'no_graveyard', p)) or IP.hushed(g)


def seph_loops(g, p):
    """the loops p can run now: [(name, key permanents, kills)]"""
    if p.key != 'seph' or loops_blocked(g, p): return []
    names = _names_bf(g, p)
    out = []
    for name, groups, _ in LOOPS:
        groups = loop_groups(g, p, name, groups, names)
        if not all(any(n in names for n in grp) for grp in groups): continue
        keys = []
        for grp in groups:                                   # a piece is key only if nothing else fills its role
            have = [m for m in p.perms if m.cd is not None and m.cd.name in grp and not m.phased]
            if len(have) == 1 and not (grp == groups[0] and name.endswith('Kitchen Finks')
                                       and {'Mikaeus, the Unhallowed', 'Melira, Sylvok Outcast'} <= names):
                keys.append(have[0])
        out.append((name, keys, loop_kills(name, names)))
    return out


def run_loop(g, p, name, keys, kills):
    from commander_sim import ais
    p.stats['combo_attempt'] += 1
    p.milestone.setdefault('combo', p.turns)
    p.loop_turn = p.turns
    log(f'  {NAME(p)} goes for the {name} loop', g)
    if ais.combo_interrupted(g, p, 'seph', keys):
        p.stats['combo_stopped'] += 1; log('    ...the loop is stopped', g); return True
    if any(m.cd is not None and 'shards' in m.cd.tags and not m.phased for m in p.perms) and name != LOOPS[0][0]:
        for q in g.opps(p):                                  # Aura Shards: every creature entering destroys one
            for m in list(q.perms):
                if m.cd is not None and ('A' in m.cd.types or 'E' in m.cd.types) and not indestructible(g, m) \
                        and not untargetable(g, m):
                    die(g, m, 'destroy')
        log("    Aura Shards clears the opponents' artifacts and enchantments", g)
    if kills:
        names = _names_bf(g, p)
        by_mill = name != LOOPS[0][0] and not any(d in names for d in DRAINS) and 'Triskelion' not in names
        ais.win(g, p, 'combo', through_life=not by_mill); return True
    if 'Finks' in name:
        gain(p, 1000); log(f'    {NAME(p)} gains 1000 life (as good as infinite)', g)
    else:                                                    # infinite colourless mana, a Zombie kept each loop
        make_tokens(g, p, 20, 2, color='B'); p.floatC = getattr(p, 'floatC', 0) + 20
        log(f'    {NAME(p)} keeps 20 Zombies and floats 20 colourless mana', g)
    return True


def loop_options(g, p, s):
    """the AI's main-phase options for Sephiroth's loops"""
    from commander_sim.ai import brain
    if getattr(p, 'loop_turn', None) == p.turns: return []
    o = []
    for name, keys, kills in seph_loops(g, p):
        if not kills and 'Finks' in name and p.life >= 500: continue       # infinite life already
        u = 14.0 - 6.0 * brain.removal_risk(g, p) if kills else (4.0 if 'Finks' in name else 3.0)
        o.append((u, f'loop: {name}' + ('' if kills else ' (no payoff)'),
                  lambda name=name, keys=keys, kills=kills: run_loop(g, p, name, keys, kills)))
    return o


def loop_need(g, p, extra=()):
    """card names that would complete a loop (with what is on the battlefield plus `extra` names), killing loops first"""
    if loops_blocked(g, p): return []
    names = _names_bf(g, p) | set(extra)
    want = []
    for name, groups, _ in sorted(LOOPS, key=lambda l: l[2] != 'kills'):
        groups = loop_groups(g, p, name, groups, names)
        missing = [grp for grp in groups if not any(n in names for n in grp)]
        if len(missing) == 1 and (loop_kills(name, names) or name.endswith('Finks')):
            want += [n for n in missing[0] if n not in want]
    return want


def loop_prio(g, p, c):
    """Sephiroth's cast priority for a loop piece in hand: it completes a loop / it is one of two pieces down"""
    if c.name not in LOOP_CARDS or loops_blocked(g, p): return None
    if c.name in loop_need(g, p): return 85
    names = _names_bf(g, p)
    for name, groups, _ in LOOPS:
        groups = loop_groups(g, p, name, groups, names)
        if any(c.name in grp for grp in groups) and sum(any(n in names for n in grp) for grp in groups) >= 1:
            return 55
    return None


note('Melira, Sylvok Outcast', 'Full', "you can't get poison counters; your creatures can't get -1/-1 counters "
     '(persist returns them without one); no creature in the pools has infect, so the last clause never applies')
note('Kitchen Finks', 'Full', 'enters: gain 2 life; persist. Loops with Melira or Mikaeus and a free sacrifice outlet')


# ------------------------------------------------------------------ Ephemerate: your decks' AI
def blink_worth(g, p, m):
    """what blinking p's creature m is worth: its enter effect (hand-tagged or compiled)"""
    from commander_sim.cards.impl import t2 as impl_t2
    return max(impl_t2.blink_value(g, p, m), etb_value(g, p, m))


def ephemerate_cast(g, p, m, why):
    """cast Ephemerate from hand on your creature m: exile it and return it (it enters again, untapped and new, so a
    removal spell aimed at it fizzles); the card is exiled and cast again for free at your next upkeep (rebound).
    True if m got away (False if Ephemerate couldn't be cast or was countered)"""
    c = next((x for x in p.hand if x.name == 'Ephemerate'), None)
    if c is None or m not in p.perms or m.token or not castable(g, p, c) or not can_pay(g, p, 0, 'W'): return False
    p.hand.remove(c); pay(g, p, 0, 'W')
    p.spells_this_turn += 1; p.stats['spells_cast'] += 1; p.cast_names.add(c.name)
    on_cast(g, p, c)
    log(f'  {NAME(p)} casts Ephemerate on {m.name} ({why})', g)
    if g.over or not counter_window(g, p, c, 6 if why == 'protection' else 3, {}):
        p.gy.append(c); return False
    p.exile.append(c); p.rebound = getattr(p, 'rebound', []) + [c]
    p.stats['ephemerate_' + why] += 1
    if m in p.perms:
        from commander_sim.cards.impl import t2 as impl_t2
        impl_t2.blink(g, p, m)
    return True


@on('Ephemerate', 'hand_options')
def _ephemerate_value(g, c, p, s, post):
    """your decks: blink the creature with the best enters-the-battlefield effect (Archon of Cruelty, Grave Titan);
    rebound blinks again next upkeep. Kept in hand while it is the only protection for a bomb"""
    from commander_sim import ais
    from commander_sim.cards.impl import t2 as impl_t2
    if p.key not in ais.MAIN or c not in p.hand or not can_pay(g, p, 0, 'W'): return []
    cands = [(blink_worth(g, p, m), m) for m in p.perms if m.creature and not m.phased and not m.token]
    cands = [x for x in cands if x[0] >= 3]
    if not cands: return []
    bv, m = max(cands, key=lambda x: x[0])
    keep = 2.5 if any(x.cd is not None and x.cd.bomb >= 6 for x in p.perms if x.creature) else 0.0
    return [(bv - 2.0 - keep, f'Ephemerate (blink {m.name})',       # two blinks with rebound
             lambda m=m: ephemerate_cast(g, p, m, 'value'))]
note('Ephemerate', 'Full', 'instant {W}: blinks your creature, in response to targeted removal (the spell fizzles) '
     'or for its enter effect; rebound: cast again free at your next upkeep on the best enter-effect creature')
card('Sword of Truth and Justice', '', types='A', dsl=[      # the parser dropped the counter before the proliferate
    {'type': 'static', 'static': 'equip_bonus', 'pow': 2, 'tgh': 2},
    {'type': 'static', 'static': 'equip_protection', 'colors': 'WU'},
    {'type': 'triggered', 'event': 'combat_damage', 'source': 'equipped',
     'effects': [{'do': 'counters', 'n': 1, 'what': {'sel': 'target', 'filter': {'type': 'creature', 'controller': 'you'}}},
                 {'do': 'proliferate'}]},
    {'type': 'static', 'static': 'equip_cost', 'mana': 2}])
note('Sword of Truth and Justice', 'Full', 'equipped creature +2/+2, protection from white and blue; on combat damage to '
     'a player: a +1/+1 counter on your best creature (the Army), then proliferate; equip {2}')
note('Sword of Fire and Ice', 'Full', 'equipped creature +2/+2, protection from red and blue; on combat damage to a '
     'player: 2 damage to any target (a creature it kills, else a player) and draw a card; equip {2}')


# ======================================================== Twinflame
MAGECRAFT = ('ping', 'mystic', 'spelldraw', 'spelltok', 'kiln', 'dragoncaller')


def twinflame_value(g, p, m, post, spells=None):
    """what a hasty token copy of creature m (until the end step) is worth this turn; spells: the instants and
    sorceries still castable after Twinflame (each triggers a magecraft copy again)"""
    if m.cd is None or not m.creature or m.phased or is_legendary(g, m) or m.owner is not p: return 0.0
    t = m.cd.tags
    v = max(0.0, etb_value(g, p, m) + 0.5 * max(0, m.plus))          # the copy enters fresh: its ETB, no counters
    if 'rem' in t and 'etb' in t: v += 3.0                          # Venser: another bounce
    if any(k in t for k in MAGECRAFT):
        if spells is None: spells = sum(1 for c in p.hand if (c.instant or c.sorcery) and c.name != 'Twinflame')
        per = int(t['ping']) * len(g.opps(p)) if 'ping' in t else 1.5
        v += 0.5 * per * spells * (2 if has(p, 'veyran') else 1)
    if not post and not m.cd.tags.get('noatk') and epow(g, m) > 0:
        v += 0.25 * epow(g, m)                                         # haste: attacks this turn
    return v


@on('Twinflame', 'hand_options')
def _twinflame(g, c, p, s, post):
    """Strive: {1}{R}, plus {2}{R} for each target beyond the first. Hasty token copies of any number of your creatures,
    exiled at the next end step. The AI copies the creatures worth copying that it can pay for, in its main phases."""
    if post is None or g.active is not p or c not in p.hand or not can_pay(g, p, 1, 'R'): return []
    mana = total_mana(g, p)
    rest = sorted(x.cmc for x in p.hand if (x.instant or x.sorcery) and x is not c)
    best = None
    for n in range(1, 6):                                  # n targets: {1}{R} + {2}{R} per extra one
        cost = 2 + 3 * (n - 1)
        if not can_pay(g, p, 1 + 2 * (n - 1), 'R' * n): break
        left, spells = mana - cost, 0
        for x in rest:                                     # the cheapest spells still castable afterwards
            if x <= left: left -= x; spells += 1
        ranked = sorted(((twinflame_value(g, p, m, post, spells), m) for m in p.perms), key=lambda x: -x[0])[:n]
        if len(ranked) < n or ranked[-1][0] < 1.5: break
        total = sum(v for v, _ in ranked)
        if best is None or total > best[0]: best = (total, ranked)
    if best is None: return []
    total, tg = best
    k = len(tg) - 1

    def go():
        targets = [m for _, m in tg if m in p.perms and not m.phased]
        if c not in p.hand or not targets: return False
        extra = len(targets) - 1
        if not can_pay(g, p, 1 + 2 * extra, 'R' * (extra + 1)): return False
        p.hand.remove(c); pay(g, p, 1 + 2 * extra, 'R' * (extra + 1))
        p.spells_this_turn += 1; p.stats['spells_cast'] += 1; p.cast_names.add(c.name)
        log(f"  {NAME(p)} casts Twinflame copying {', '.join(m.name for m in targets)}", g)
        on_cast(g, p, c)
        if g.over or not p.alive: return True
        if not counter_window(g, p, c, 3, {}):
            p.gy.append(c); return True
        for m in targets:
            if m not in p.perms: continue
            cp = E.enter_token_copy(g, p, m.cd)
            if cp is not None: cp.sick = False; cp.temp = True        # haste; exiled at the end step
        p.gy.append(c); check_state(g)
        return True
    return [(1.0 + 0.6 * total, f"Twinflame ({len(tg)} target{'s' if k else ''})", go)]


card('Twinflame', 'twinflame', types='S', cost='1R', dsl=[])
note('Twinflame', 'Full', 'strive ({2}{R} per extra target): hasty token copies of your nonlegendary creatures, exiled at '
     'the end step; the AI copies magecraft creatures when spells are left to cast, ETB creatures (Venser) and attackers')


# ======================================================== Sauron: Underworld Breach + Brain Freeze / Grapeshot
# Storm: a copy per spell cast before it this turn (every player's). With Breach out, each one is recast from the
# graveyard for its mana cost plus three other graveyard cards; Brain Freeze on yourself supplies those cards.
STORM_PIECES = ('Brain Freeze', 'Grapeshot')


def storm_count(g):
    """spells cast this turn before the one resolving now"""
    return max(0, sum(casts_this_turn(g, q) for q in g.players) - 1)


def _opp_by(g, p, key):
    opps = [q for q in g.opps(p) if q.alive and not q.life_locked]
    return min(opps, key=key) if opps else None


@on('Brain Freeze', 'resolve')
def _brain_freeze(g, p, c, ctx):
    """storm; target player mills 3 per copy. The Breach line passes its targets in ctx['mill'] (one per copy)"""
    n = ctx.get('storm', storm_count(g)) + 1
    plan = list(ctx.get('mill') or [])
    for i in range(n):
        q = plan[i] if i < len(plan) else _opp_by(g, p, lambda x: len(x.library))
        if q is not None and q.alive: mill(g, q, 3)
    log(f'    Brain Freeze: {n} cop{"y" if n == 1 else "ies"}, 3 cards each', g)


@on('Grapeshot', 'resolve')
def _grapeshot(g, p, c, ctx):
    """storm; 1 damage to any target per copy: the lowest-life opponents first, so the copies kill someone"""
    n = ctx.get('storm', storm_count(g)) + 1
    left = n
    while left > 0:
        q = _opp_by(g, p, lambda x: x.life)
        if q is None: break
        k = min(left, max(1, q.life))
        lose_life(g, q, k, p, kind='burn'); left -= k
        check_state(g)
    log(f'    Grapeshot: {n} damage', g)


card('Brain Freeze', 'storm stormmill', types='I', cost='1U', dsl=[])
card('Grapeshot', 'storm stormburn', types='S', cost='1R', dsl=[])
full('Brain Freeze', 'storm (spells cast before it this turn, by anyone); each copy mills a player 3. Held for the '
     'Underworld Breach line, where it mills you for fuel, then the table')
full('Grapeshot', 'storm; each copy deals 1 damage to any target (lowest-life opponent first). Held for the Breach line')


RITUALS = {'Dark Ritual': (0, 3, 3), 'Cabal Ritual': (1, 3, 5)}    # {generic} besides {B}, mana made, with threshold
PETAL = 'Lotus Petal'                                             # {0}: sacrifice it for one mana of any colour
LINE_CARDS = STORM_PIECES + tuple(RITUALS) + (PETAL,)


def _where(p, name):
    if any(c.name == name for c in p.hand): return 'hand'
    if any(c.name == name for c in p.gy): return 'gy'
    return None


def _line_state(g, p):
    """the numbers the Breach line decides on. Ritual mana floats as any colour in the engine, but a real ritual
    makes only {B}: it pays generic costs and other rituals, never Brain Freeze's {U} or Grapeshot's {R}"""
    real = [u for u in E.mana_units(g, p) if u[0] != 'G']
    opps = [q for q in g.opps(p) if q.alive]
    return {'land': sum(u[2] for u in real), 'float': p.floatA,
            **{c: sum(u[2] for u in real if c in u[1]) for c in 'UBR'},
            'fuel': sum(1 for c in p.gy if c.name not in LINE_CARDS), 'gy': len(p.gy),
            'storm': sum(casts_this_turn(g, q) for q in g.players),
            'where': {n: _where(p, n) for n in LINE_CARDS},
            'breach': has(p, 'breach'), 'breach_hand': any('breach' in c.tags for c in p.hand),
            'lib': {q: len(q.library) for q in opps}, 'life': {q: q.life for q in opps}, 'mylib': len(p.library)}


def _can(st, gen, col):
    """can the line pay {gen} plus one {col}? ({B} can come from ritual mana)"""
    ok = (st['float'] > 0 or st['B'] > 0) if col == 'B' else (st[col] > 0 and st['land'] > 0)
    return ok and st['float'] + st['land'] >= gen + 1


def _ready(st, name):
    w = st['where'][name]
    return w == 'hand' or (w == 'gy' and st['fuel'] >= 3)


def _mill_plan(p, st, copies):
    """who each Brain Freeze copy mills (st: after paying for it). The table if the copies finish it. Otherwise
    yourself first, enough fuel to pay for the next Brain Freeze (Lotus Petal escapes for {U}, rituals for generic),
    escape it, and recast each ritual twice more for storm; then the opponent closest to an empty library. If nothing
    can pay the next Brain Freeze's {U}, this is the last one: every copy goes to the table"""
    lib = dict(st['lib'])
    need = sum(-(-n // 3) for q, n in lib.items() if n > 0 and st['life'][q] > 0)
    w = st['where']
    need_u = 0 if st['U'] > 0 and st['land'] > 0 else 1
    need_gen = max(0, 2 - st['land'] - st['float'] - need_u)
    petal = w[PETAL] is not None
    rit = [n for n in RITUALS if w[n]]
    last = need_u and not petal
    esc = need_u + (0 if rit else need_gen) if petal else 0
    esc += -(-need_gen // 2) if rit else 0
    keep = 3 + 3 * esc + 6 * len(rit)
    mine = 0 if copies >= need or last else min(copies, -(-max(0, keep - st['fuel']) // 3), st['mylib'] // 3)
    plan = [p] * mine
    for _ in range(copies - mine):
        live = [q for q, n in lib.items() if n > 0 and st['life'][q] > 0]
        if not live: plan.append(p if st['mylib'] >= 3 * (mine + 1) else live and live[0] or p); mine += 1; continue
        q = min(live, key=lambda x: lib[x]); lib[q] -= 3; plan.append(q)
    return plan


def _line_step(st):
    """the next cast of the Breach line, or None. Fuel (graveyard cards) goes first to what the next Brain Freeze needs:
    its own escape, and a Lotus Petal escape when blue mana runs out; spare fuel becomes ritual escapes, which add
    storm (and pay generic costs)"""
    if not st['breach']:
        return 'breach' if st['breach_hand'] and _can(st, 1, 'R') else None
    alive = [q for q, n in st['lib'].items() if n > 0 and st['life'][q] > 0]
    if not alive: return None
    w = st['where']
    bf_res = 3 if w['Brain Freeze'] == 'gy' else 0
    petal_ready = w[PETAL] == 'hand' or (w[PETAL] == 'gy' and st['fuel'] >= 3 + bf_res)
    petal_res = 3 if w[PETAL] and st['U'] <= 1 else 0          # the Brain Freeze after this one will want a Petal

    def ritual(res):
        for n, (gen, made, thr) in RITUALS.items():
            got = thr if st['gy'] >= 7 else made
            if (w[n] == 'hand' or (w[n] == 'gy' and st['fuel'] >= 3 + res)) and got > gen + 1 and _can(st, gen, 'B'):
                return n
        return None
    copies = st['storm'] + 1
    gs = w['Grapeshot'] and _ready(st, 'Grapeshot') and _can(st, 1, 'R')
    if gs and copies >= min(st['life'][q] for q in alive): return 'Grapeshot'
    bf = w['Brain Freeze'] and _ready(st, 'Brain Freeze')
    if st['U'] == 0 or st['land'] == 0:                             # no blue left: only a Petal can make it
        if petal_ready and bf: return PETAL
        return 'Grapeshot' if gs else None
    if st['land'] + st['float'] < 2:                                # blue, but short of the generic {1}
        r = ritual(bf_res + petal_res)
        if r: return r
        if petal_ready and bf: return PETAL
        return 'Grapeshot' if gs else None
    r = ritual(bf_res + petal_res)                                  # spare fuel first becomes storm
    if r: return r
    if bf: return 'Brain Freeze'
    return 'Grapeshot' if gs else None


def _pay_line(g, p, gen, col):
    """pay for a spell of the line: the pip from a real source ({B} may use ritual mana), generic from ritual mana first"""
    if col == 'B' and p.floatA > 0: p.floatA -= 1
    elif not pay(g, p, 0, col): return False
    k = min(p.floatA, gen); p.floatA -= k
    return gen - k == 0 or pay(g, p, gen - k, '')


def _dry_run(g, p):
    """would the line finish every opponent if nobody interacts? Played for real on a copy of the game with the
    opponents' hands removed (so it neither cheats nor fears their counters); cached for the position"""
    from commander_sim.ai import search
    key = (turn_stamp(g), total_mana(g, p), p.floatA, len(p.gy), len(p.hand), len(p.library),
           sum(casts_this_turn(g, q) for q in g.players), tuple(len(q.library) for q in g.players))
    cached = getattr(g, 'breach_dry', None)
    if cached is not None and cached[0] == key: return cached[1]
    g2 = search.clone(g); p2 = g2.players[g.players.index(p)]
    for q in g2.opps(p2): q.hand = []
    breach_line(g2, p2, execute=True)
    ok = all(not q.alive or not q.library for q in g2.opps(p2))
    g.breach_dry = (key, ok)
    return ok


def breach_line(g, p, execute=False):
    """Sauron's Breach line. Dry run (execute=False): does it finish every opponent (burned to 0 or milled out) if
    nobody interacts? Execute: cast it for real, one spell at a time (each can be countered)."""
    if p.key != 'sauron' or not g.opps(p): return False
    st = _line_state(g, p)
    if not st['breach'] and not st['breach_hand']: return False
    if not any(st['where'][n] for n in STORM_PIECES): return False
    if not execute: return _dry_run(g, p)
    p.stats['breach_line'] += 1
    p.milestone.setdefault('combo', p.turns)
    log(f'  {NAME(p)} goes for the Underworld Breach line', g)
    for _ in range(300):
        if g.over or not p.alive: break
        st = _line_state(g, p)
        act = _line_step(st)
        if act is None: break
        if act == 'breach':
            c = next(x for x in p.hand if 'breach' in x.tags)
            if not _pay_line(g, p, 1, 'R'): break
            if not cast_card(g, p, c, 'hand', {}) or not has(p, 'breach'):
                log('    ...Underworld Breach is stopped', g); break
            continue
        zone = st['where'][act]
        c = next(x for x in (p.hand if zone == 'hand' else p.gy) if x.name == act)
        ctx = {'storm': st['storm']}
        gen, col = (RITUALS[act][0], 'B') if act in RITUALS else (None, None) if act == PETAL else \
            (1, 'U' if act == 'Brain Freeze' else 'R')
        if zone == 'gy':
            fuel = sorted([x for x in p.gy if x.name not in LINE_CARDS], key=lambda x: card_worth(g, p, x, True))
            if len(fuel) < 3: break
            for x in fuel[:3]: p.gy.remove(x); p.exile.append(x)
            p.stats['breach_escapes'] += 1
        if gen is not None and not _pay_line(g, p, gen, col): break
        if act == 'Brain Freeze': ctx['mill'] = _mill_plan(p, _line_state(g, p), st['storm'] + 1)   # as paid for
        cast_card(g, p, c, 'escape' if zone == 'gy' else 'hand', ctx)
        if act == PETAL:                                      # sacrifice it at once, so it can be escaped again
            m = next((x for x in p.perms if x.cd is not None and x.cd.name == PETAL), None)
            if m is not None:
                leave(g, m); to_zone_card(g, m, 'gy')
                if st['U'] == 0: p.floatU = getattr(p, 'floatU', 0) + 1
                elif st['R'] == 0: p.floatR = getattr(p, 'floatR', 0) + 1
                else: p.floatA += 1
    check_state(g)
    return True


def breach_options(g, p, post):
    """the AI's main-phase option: go for the Breach line when the dry run finishes the table"""
    if post is None or g.active is not p or not breach_line(g, p): return []
    return [(15.0, 'Underworld Breach line', lambda: breach_line(g, p, execute=True))]
