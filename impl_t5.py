"""Tier 5 pool decks: Urza, Kinnan, Winota, Zur, Yawgmoth (combos themselves live in impl_combos)."""
from engine import *
import engine as E
import cardimpl as CI
from cardimpl import on, _eot, count_type
from pool_cards import card, note
import impl_common as IC
from impl_common import make_artifact_tokens, best_opp_creature, best_opp_nonland, walker, always
from impl_t2 import best_target_any, blink, blink_value


def is_artifact(m):
    return (m.cd is not None and 'A' in m.cd.types) or bool(m.data and m.data.get('artifact'))


# ======================================================== Urza, Lord High Artificer
@on('Urza, Lord High Artificer', 'etb')
def _urza(g, src, p, m):
    if m is src:
        for t in make_tokens(g, src.owner, 1, 0, 0, color='', types=('construct',)):
            t.data = {'construct': True, 'artifact': True}
        g.selfpt = True


@on('Urza, Lord High Artificer', 'extra_mana')
def _urza_mana(g, src, p, U):
    used = {id(u[0]) for u in U}
    return [[m, 'U', 1] for m in p.perms if is_artifact(m) and not m.tapped and not m.phased and id(m) not in used]


@on('Urza, Lord High Artificer', 'options')
def _urza_five(g, src, p, s, post):
    if post is None or not can_pay(g, p, 5, '') or len(p.library) < 5: return []

    def go():
        if not can_pay(g, p, 5, ''): return False
        pay(g, p, 5, ''); g.rng.shuffle(p.library)
        c = p.library.pop(); log(f'  Urza: {c.name} off the top, free', g)
        if c.land:
            if getattr(p, 'lands_played', 1) < 1: import ais; ais.play_land_card(g, p, c)
            else: p.exile.append(c)
        elif castable(g, p, c) and 'ctr' not in c.tags:
            p.hand.append(c); cast_card(g, p, c, 'hand', {})
        else: p.exile.append(c)
        return True
    return [(1.5 if post else 0.8, 'Urza: free card', go)]
card('Urza, Lord High Artificer', 'leg human pow=1 tgh=4', dsl=[])
note('Urza, Lord High Artificer', 'Full', 'Construct token (+1/+1 per artifact); artifacts tap for U; {5}: a free card '
     'off the top')


def _construct_bonus(g, m):
    if m.data and m.data.get('construct'):
        n = sum(1 for x in m.owner.perms if is_artifact(x) and not x.phased)
        return n, n
    return 0, 0


IC.TOKEN_PT = getattr(IC, 'TOKEN_PT', [])
IC.TOKEN_PT.append(_construct_bonus)


@on('Karn, the Great Creator', 'no_artifact_mana')
def _karn_static(g, src, p): return 1 if p is not src.owner else 0


@on('Karn, the Great Creator', 'mana_lock')
def _karn_lattice(g, src, p):
    return 1 if p is not src.owner and getattr(g, 'lattice_lock', None) is src.owner and \
        any(m.cd is not None and m.cd.name == 'Mycosynth Lattice' for q in g.players for m in q.perms) else 0


walker('Karn, the Great Creator', [
    (1, 'animate an artifact', always(1.0), lambda g, p, src: None),
    (-2, 'artifact from exile', lambda g, p, src: 2.0 if any('A' in c.types for c in p.exile) else None,
     lambda g, p, src: (lambda cs: cs and (p.exile.remove(cs[0]) or p.hand.append(cs[0])))([c for c in p.exile if 'A' in c.types])),
], ('Approximate', 'opponents\' artifacts can\'t activate (mana rocks, Treasures); the wish from outside the game is '
                   'modeled as from exile; the Lattice lock is a combo'))


def _tezz_minus(g, p, src):
    x = src.loyalty
    cs = [c for c in searchable(g, p) if 'A' in c.types and c.cmc <= x]
    if cs:
        c = max(cs, key=lambda c: (c.name in ('Isochron Scepter', 'Power Artifact', 'Basalt Monolith', 'Grim Monolith',
                                              'Mycosynth Lattice'), card_worth(g, p, c), c.cmc))
        p.library.remove(c); g.rng.shuffle(p.library); enter(g, p, c)


walker('Tezzeret the Seeker', [
    (1, 'untap two artifacts', always(2.0), lambda g, p, src: [setattr(m, 'tapped', False) for m in [m for m in p.perms if is_artifact(m) and m.tapped][:2]]),
    (-2, 'artifact onto the battlefield', lambda g, p, src: 3.0 if any('A' in c.types and c.cmc <= src.loyalty for c in p.library) else None, _tezz_minus),
], ('Approximate', '-X uses all its loyalty as X; the ultimate is not used'))


@on('Kappa Cannoneer', 'etb')
def _kappa(g, src, p, m):
    if m.owner is src.owner and is_artifact(m) and m is not src: src.plus += 1
card('Kappa Cannoneer', 'pow=4 tgh=4', dsl=[{'type': 'static', 'static': 'unblockable'}], ward=4)
note('Kappa Cannoneer', 'Approximate', 'ward 4, +1/+1 per artifact entering, unblockable (always, not only that turn); '
     'improvise not used')


@on('Sai, Master Thopterist', 'cast')
def _sai(g, src, caster, c):
    if caster is src.owner and 'A' in c.types:
        for t in make_tokens(g, caster, 1, 1, fly=True, color='', types=('thopter',)): t.data = {'artifact': True}
card('Sai, Master Thopterist', 'leg human pow=1 tgh=4', dsl=[])
note('Sai, Master Thopterist', 'Partial', 'Thopter per artifact spell; the sacrifice-for-cards ability is not used')
for _n in ('Foundry Inspector', 'Etherium Sculptor'):
    IC._reducer(_n, lambda c: 'A' in c.types)
IC._reducer('Jet Medallion', lambda c: 'B' in c.pips)
IC._tutor_etb('Trophy Mage', lambda c: 'A' in c.types and c.cmc == 3, 'human pow=2')
IC._tutor_etb('Tribute Mage', lambda c: 'A' in c.types and c.cmc == 2, 'human pow=2 tgh=2')


@on('Cursed Totem', 'no_creature_mana')
def _totem(g, src, p): return 1
card('Cursed Totem', '', types='A', dsl=[])
note('Cursed Totem', 'Approximate', 'creatures\' mana abilities are off for everyone; other creature activations '
     '(Krenko, Kinnan, Yawgmoth) are not blocked')


@IC.spell('Whir of Invention', prio=lambda g, p, c: 55 if total_mana(g, p) >= 5 else 0, types='I',
          status=('Approximate', 'X = spare mana; the best artifact onto the battlefield (improvise not used)'))
def _whir(g, p, c, ctx):
    x = ctx.get('x', 0)
    cs = [y for y in searchable(g, p) if 'A' in y.types and y.cmc <= x]
    if cs:
        y = max(cs, key=lambda y: (y.name in IC_COMBO_ART, card_worth(g, p, y), y.cmc))
        p.library.remove(y); g.rng.shuffle(p.library); enter(g, p, y)


IC_COMBO_ART = ('Isochron Scepter', 'Power Artifact', 'Basalt Monolith', 'Grim Monolith', 'Mycosynth Lattice', 'Rings of Brighthearth')
card('Whir of Invention', 'xtutor', types='I', dsl=[])


@IC.spell('Windfall', prio=lambda g, p, c: 50 if len(p.hand) <= 2 else 0, types='S', status=('Full', ''))
def _windfall(g, p, c, ctx):
    qs = [q for q in g.players if q.alive]
    n = max(len(q.hand) - (1 if q is p else 0) for q in qs)
    for q in qs: discard_cards(g, q, [x for x in q.hand if x is not c])
    for q in qs: draw(g, q, n)


card('Everflowing Chalice', 'rock=1:C', types='A', dsl=[])
note('Everflowing Chalice', 'Approximate', 'always kicked once')


# ======================================================== Kinnan, Bonder Prodigy
@on('Kinnan, Bonder Prodigy', 'nonland_mana_bonus')
def _kinnan(g, src, p): return 1 if p is src.owner else 0


@on('Kinnan, Bonder Prodigy', 'options')
def _kinnan_act(g, src, p, s, post):
    if post is None or not can_pay(g, p, 5, 'GU'): return []

    def go():
        if not can_pay(g, p, 5, 'GU'): return False
        pay(g, p, 5, 'GU')
        top = [p.library.pop() for _ in range(min(5, len(p.library)))]
        hits = [c for c in top if c.creature and 'human' not in c.subtypes]
        if hits:
            c = max(hits, key=lambda c: (c.bomb or c.pow, c.cmc)); top.remove(c); enter(g, p, c)
        g.rng.shuffle(top); p.library[:0] = top
        return True
    return [(3.0 + (2.0 if post else 0), 'Kinnan: dig five', go)]
card('Kinnan, Bonder Prodigy', 'leg human pow=2', dsl=[])
note('Kinnan, Bonder Prodigy', 'Full', 'nonland mana sources make one extra; {5}{G}{U}: a non-Human creature from the '
     'top five onto the battlefield')


CI.DYN_MANA['Bloom Tender'] = lambda g, p, m: max(1, len(set(''.join(x.cd.pips for x in p.perms if x.cd is not None)) & set('WUBRG')))
card('Bloom Tender', 'pow=1 dork=A noatk', dsl=[])
note('Bloom Tender', 'Full', 'one mana per colour among your permanents')
CI.DYN_MANA['Incubation Druid'] = lambda g, p, m: 3 if m.plus > 0 else 1
card('Incubation Druid', 'pow=0 tgh=2 dork=A noatk', dsl=[])
note('Incubation Druid', 'Approximate', 'three mana once it has a counter (adapt not activated)')


@on('Seedborn Muse', 'upkeep')
def _seedborn(g, src, p):
    o = src.owner
    if p is o: return
    for L in o.lands: L.tapped = False
    for m in o.perms:
        if not (m.cd is not None and 'nountap' in m.cd.tags): m.tapped = False
card('Seedborn Muse', 'pow=2 tgh=4', dsl=[])
note('Seedborn Muse', 'Full', 'untaps your permanents in each other player\'s untap step')


@IC.spell('Worldly Tutor', prio=40, types='I')
def _worldly(g, p, c, ctx):
    cs = [x for x in searchable(g, p) if x.creature]
    if cs:
        x = max(cs, key=lambda x: card_worth(g, p, x)); p.library.remove(x); g.rng.shuffle(p.library); p.library.append(x)


@IC.spell("Nature's Rhythm", prio=lambda g, p, c: 55 if total_mana(g, p) >= 5 else 0, types='S',
          status=('Approximate', 'X = spare mana, creature onto the battlefield; harmonize not used'))
def _rhythm(g, p, c, ctx):
    import impl_t3; impl_t3._put_creature(g, p, lambda y: y.cmc <= ctx.get('x', 0))


card("Nature's Rhythm", 'xtutor', types='S', dsl=[])
card('Elvish Spirit Guide', 'pow=2', dsl=[])
note('Elvish Spirit Guide', 'Partial', 'cast as a 2/2; the exile-for-G is not used')


# ======================================================== Winota, Joiner of Forces
@on('Winota, Joiner of Forces', 'attack')
def _winota(g, src, p, atk, d):
    if src.owner is not p: return
    new = []
    for a in [m for m in atk if not has_type(m, 'human')]:
        top = [p.library.pop() for _ in range(min(6, len(p.library)))]
        hum = [c for c in top if c.creature and 'human' in c.subtypes]
        if hum:
            c = max(hum, key=lambda c: (c.bomb or c.pow, c.cmc)); top.remove(c)
            m = enter(g, p, c); m.tapped = True; m.sick = False
            g.eot_kw.setdefault(id(m), set()).add('indestructible'); new.append(m)
            log(f'    Winota puts {c.name} onto the battlefield attacking', g)
        g.rng.shuffle(top); p.library[:0] = top
        if g.over: break
    return new
card('Winota, Joiner of Forces', 'leg human warrior pow=4', dsl=[])
note('Winota, Joiner of Forces', 'Full', 'each non-Human attacker: a Human from the top six onto the battlefield '
     'attacking and indestructible (ETBs fire)')


@on('Kiki-Jiki, Mirror Breaker', 'options')
def _kiki(g, src, p, s, post):
    if src.tapped or post is not False: return []
    cands = [m for m in p.perms if m.creature and m.cd is not None and 'leg' not in m.cd.tags and m is not src]
    if not cands: return []
    t = max(cands, key=lambda m: blink_value(g, p, m) + epow(g, m) * 0.5)

    def go():
        if src.tapped or t not in p.perms: return False
        src.tapped = True
        c = enter_token_copy(g, p, t.cd)
        if c is not None: c.sick = False; c.temp = True
        return True
    return [(1.5 + 0.5 * blink_value(g, p, t) + 0.3 * epow(g, t), f'Kiki-Jiki copies {t.name}', go)]
card('Kiki-Jiki, Mirror Breaker', 'leg shaman pow=2 haste', dsl=[])
note('Kiki-Jiki, Mirror Breaker', 'Full', 'hasty copy of your best nonlegendary creature each turn (sacrificed at end); '
     'the infinite loops are combos')


@on('Felidar Guardian', 'etb')
def _felidar(g, src, p, m):
    if m is not src: return
    cands = [x for x in src.owner.perms if x is not src and not x.token and x.cd is not None and blink_value(g, src.owner, x) > 0
             and x.cd.name not in ('Felidar Guardian', 'Restoration Angel')]      # the loop is a combo, not a value blink
    if cands: blink(g, src.owner, max(cands, key=lambda x: blink_value(g, src.owner, x)))
card('Felidar Guardian', 'pow=1 tgh=4', dsl=[])
note('Felidar Guardian', 'Full', 'blinks your best ETB permanent (Kiki loop is a combo)')


@on('Zealous Conscripts', 'etb')
def _conscripts(g, src, p, m):
    if m is not src: return
    o = src.owner
    t = best_opp_nonland(g, o, lambda x: x.creature)
    if t is not None and pval(g, t) >= 3 and not t.is_cmd:
        q = t.owner; q.perms.remove(t); t.owner = o; t.tapped = False; t.sick = False; o.perms.append(t)
        g.bf_ver = getattr(g, 'bf_ver', 0) + 1
        o.borrowed = getattr(o, 'borrowed', []) + [t]
card('Zealous Conscripts', 'human warrior pow=3 haste', dsl=[])
note('Zealous Conscripts', 'Approximate', 'steals the best opposing creature until end of turn (untapped, hasty)')


@on('Goblin Guide', 'attack')
def _guide(g, src, p, atk, d):
    if src in atk and d.library and d.library[-1].land: d.hand.append(d.library.pop())
card('Goblin Guide', 'pow=2 haste', dsl=[])
note('Goblin Guide', 'Full', '')
card('Signal Pest', 'pow=0 tgh=1', dsl=[], kws={'battle cry'})
note('Signal Pest', 'Approximate', 'battle cry; its evasion is ignored')


@on('Loran of the Third Path', 'etb')
def _loran(g, src, p, m):
    if m is src:
        t = best_opp_nonland(g, src.owner, lambda x: x.cd is not None and ('A' in x.cd.types or 'E' in x.cd.types))
        if t is not None and pval(g, t) >= 2: apply_removal(g, src.owner, t, 'destroy')
card('Loran of the Third Path', 'leg human pow=2 tgh=1 vig', dsl=[])
note('Loran of the Third Path', 'Partial', 'destroys an artifact/enchantment on entry; the draw ability is not used')


# ======================================================== Zur the Enchanter
ZUR_PREF = ('Necropotence', 'Ethereal Armor', 'Empyrial Armor', 'Ghostly Prison', 'Propaganda', 'Solitary Confinement',
            'Rest in Peace', 'Mystic Remora', 'Oblivion Ring', 'Aura of Silence', 'Copy Enchantment')


@on('Zur the Enchanter', 'attack')
def _zur(g, src, p, atk, d):
    if src not in atk: return
    have = {m.cd.name for m in p.perms if m.cd is not None}
    cs = [c for c in searchable(g, p) if 'E' in c.types and c.cmc <= 3 and c.name not in have]
    if not cs: return
    c = min(cs, key=lambda c: (ZUR_PREF.index(c.name) if c.name in ZUR_PREF else 99, -card_worth(g, p, c)))
    p.library.remove(c); g.rng.shuffle(p.library)
    g.attach_to = src if 'aura' in c.subtypes else None
    try:
        enter(g, p, c)
    finally:
        g.attach_to = None
    log(f'    Zur fetches {c.name}', g)
card('Zur the Enchanter', 'leg human wizard pow=1 tgh=4 fly', dsl=[])
note('Zur the Enchanter', 'Full', 'attacks: an enchantment with MV 3 or less onto the battlefield (Necropotence and '
     'armor Auras on Zur first)')


@on('Solitary Confinement', 'prevent_damage')
def _conf_dmg(g, src, p): return 1 if p is src.owner else 0


@on('Solitary Confinement', 'skip_draw')
def _conf_draw(g, src, p): return 1 if p is src.owner else 0


@on('Solitary Confinement', 'upkeep')
def _conf_up(g, src, p):
    if p is not src.owner: return
    if p.hand: discard_worst(g, p, 1)
    else: die(g, src, 'sac')
card('Solitary Confinement', '', types='E', dsl=[])
note('Solitary Confinement', 'Approximate', 'skip your draw step, prevent all damage to you, discard each upkeep (or '
     'sacrifice); player shroud not modeled')


@on('Notion Thief', 'steal_draw')
def _thief(g, src, p): return p is not src.owner
card('Notion Thief', 'human rogue pow=3 tgh=1 flash', dsl=[])
note('Notion Thief', 'Full', 'opponents\' extra draws become yours')


@on('Copy Enchantment', 'etb')
def _copy_ench(g, src, p, m):
    if m is not src: return
    cs = [x for q in g.players for x in q.perms if x.cd is not None and 'E' in x.cd.types and x is not src and 'aura' not in x.cd.subtypes]
    if cs:
        best = max(cs, key=lambda x: pval(g, x) + (3 if x.cd.name == 'Necropotence' else 0))
        leave(g, src); n = enter(g, src.owner, best.cd); n.phys = src.cd
card('Copy Enchantment', '', types='E', dsl=[])
note('Copy Enchantment', 'Full', 'copies the best enchantment on the battlefield')


walker('Jace, Wielder of Mysteries', [
    (1, 'mill two, draw', always(2.5), lambda g, p, src: (mill(g, max(g.opps(p), key=lambda q: len(q.library)) if g.opps(p) else p, 2), draw(g, p, 1))),
], ('Approximate', 'drawing from an empty library wins (with Consultation / Pact that is a combo); the -8 is not used'))


# ======================================================== Yawgmoth, Thran Physician
@on('Yawgmoth, Thran Physician', 'options')
def _yawg(g, src, p, s, post):
    if post is None: return []
    out = []
    fod = [m for m in p.perms if m.creature and m is not src and not m.is_cmd and (m.token or pval(g, m) < 2 or
                                                                                   (m.cd is not None and 'undying' in m.cd.kws and m.plus <= 0))]
    tgt = [m for q in g.opps(p) for m in q.perms if m.creature and etgh(g, m) <= 1 and pval(g, m) >= 2 and not untargetable(g, m)]
    if fod and p.life > 8 and (tgt or IC.death_value(g, p) >= 2 or len(p.hand) <= 2):
        m = min(fod, key=lambda x: (not (x.cd is not None and 'undying' in x.cd.kws), pval(g, x)))

        def go(m=m):
            if m not in p.perms or p.life <= 8: return False
            lose_life(g, p, 1, p); die(g, m, 'sac')
            t = [x for q in g.opps(p) for x in q.perms if x.creature and etgh(g, x) <= 1 and not untargetable(g, x)]
            if t:
                x = max(t, key=lambda x: pval(g, x)); x.plus -= 1
                if etgh(g, x) <= 0: die(g, x, 'sba')
            draw(g, p, 1); return True
        out.append((1.5 + IC.death_value(g, p) / 2.0 + (1.5 if tgt else 0), f'Yawgmoth: sacrifice {m.name}', go))
    if can_pay(g, p, 0, 'BB') and p.hand and any(m.plus > 0 or m.loyalty for m in p.perms) and post is True:
        def prol():
            if not can_pay(g, p, 0, 'BB') or not p.hand: return False
            pay(g, p, 0, 'BB'); discard_worst(g, p, 1); IC.proliferate(g, p); return True
        out.append((1.0, 'Yawgmoth: proliferate', prol))
    return out
card('Yawgmoth, Thran Physician', 'leg human pow=2 tgh=4', dsl=[])
note('Yawgmoth, Thran Physician', 'Full', 'pay 1 life, sacrifice: -1/-1 counter on an X/1 and draw; BB discard: '
     'proliferate (the undying loop is a combo); protection from Humans not modeled')


@on('Nether Traitor', 'gy_dies')
def _traitor(g, c, p, m):
    if c in p.gy and can_pay(g, p, 0, 'B'):
        pay(g, p, 0, 'B'); p.gy.remove(c); enter(g, p, c)
card('Nether Traitor', 'pow=1 haste', dsl=[], kws={'unblockable_shadow', 'haste'})
note('Nether Traitor', 'Full', 'shadow; returns for B when another creature of yours dies')


@IC.spell('Beseech the Mirror', prio=50, types='S', status=('Approximate', 'tutor to hand (bargain free-cast not used)'))
def _beseech(g, p, c, ctx):
    tutor(g, p, 'any')


@on('Wishclaw Talisman', 'options')
def _wishclaw(g, src, p, s, post):
    if post is None or src.tapped or not can_pay(g, p, 1, '') or g.active is not p: return []

    def go():
        if src.tapped or not can_pay(g, p, 1, ''): return False
        pay(g, p, 1, ''); src.tapped = True; tutor(g, p, 'any')
        opp = max(g.opps(p), key=lambda q: threat(g, p, q))
        p.perms.remove(src); src.owner = opp; opp.perms.append(src); g.bf_ver = getattr(g, 'bf_ver', 0) + 1
        return True
    return [(3.5, 'Wishclaw Talisman', go)]
card('Wishclaw Talisman', '', types='A', dsl=[])
note('Wishclaw Talisman', 'Approximate', 'tutor, then the most threatening opponent gets it (they don\'t use it back)')


@IC.spell('Cabal Ritual', prio=0, types='I', status=('Approximate', 'adds BBB (BBBBB with threshold) when mana is short'))
def _cabal(g, p, c, ctx): p.floatA += 5 if len(p.gy) >= 7 else 3
