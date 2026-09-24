"""Tier 3 pool decks: Korvold, Marwyn, Atraxa, Aurelia, Tergrid."""
from engine import *
import engine as E
import cardimpl as CI
from cardimpl import on, _eot, count_type
from pool_cards import card, note
import impl_common as IC
from impl_common import make_artifact_tokens, sac_food, proliferate, walker, always, best_opp_creature, best_opp_nonland
from impl_t2 import best_target_any


# ======================================================== Korvold, Fae-Cursed King (sacrifice value)
def sac_worst_permanent(g, p, exclude=None):
    """sacrifice the least valuable permanent (Food / Clue / Treasure first); returns what was sacrificed"""
    if getattr(p, 'foods', 0): sac_food(g, p); return 'Food'
    if p.clues:
        p.clues -= 1
        if g.hooks: CI.fire(g, 'sacrifice', p, 'Clue')
        return 'Clue'
    if p.treasures:
        p.treasures -= 1
        if g.hooks: CI.fire(g, 'sacrifice', p, 'Treasure')
        return 'Treasure'
    cands = [m for m in p.perms if m is not exclude and not m.is_cmd and not m.phased]
    if cands:
        m = min(cands, key=lambda m: pval(g, m)); die(g, m, 'sac'); return m
    if p.lands:
        L = min(p.lands, key=lambda L: (not L.tapped, len(L.cd.tags.get('c', ''))))
        p.lands.remove(L); p.gy.append(L.cd)
        if g.hooks: CI.fire(g, 'sacrifice', p, L.cd); CI.fire(g, 'land_gy', p, L.cd)
        return L.cd
    return None


@on('Korvold, Fae-Cursed King', 'etb')
def _korvold_etb(g, src, p, m):
    if m is src: sac_worst_permanent(g, src.owner, exclude=src)


@on('Korvold, Fae-Cursed King', 'attack')
def _korvold_atk(g, src, p, atk, d):
    if src in atk: sac_worst_permanent(g, p, exclude=src)


@on('Korvold, Fae-Cursed King', 'sacrifice')
def _korvold_sac(g, src, p, what):
    if p is src.owner and what is not src:
        src.plus += 1
        if len(p.library) > 10: draw(g, p, 1)
card('Korvold, Fae-Cursed King', 'leg pow=4 fly', dsl=[])
note('Korvold, Fae-Cursed King', 'Full', 'enters/attacks: sacrifice the least valuable permanent; every sacrifice '
     '(Treasure / Food / Clue spending included): +1/+1 counter and a card')


@on('Mayhem Devil', 'sacrifice')
def _mayhem(g, src, p, what):
    best_target_any(g, src.owner, 1)
card('Mayhem Devil', 'pow=3', dsl=[])
note('Mayhem Devil', 'Full', 'any player\'s sacrifice (Treasures included): 1 damage')


card('Academy Manufactor', 'pow=1 tgh=3', types='AC', dsl=[])
note('Academy Manufactor', 'Approximate', 'hooked token makers create one of each; engine-tagged Treasure makers are '
     'not multiplied')


@on('Chatterfang, Squirrel General', 'token_created')
def _chatter_art(g, src, p, kinds, n):
    if p is src.owner: make_tokens(g, p, n, 1, color='G', types=('squirrel',))
card('Chatterfang, Squirrel General', 'leg pow=3 warrior', dsl=[])
note('Chatterfang, Squirrel General', 'Approximate', 'extra Squirrels for artifact tokens made by hooks; creature '
     'token copies and its activated ability not modeled')


@on('Gilded Goose', 'etb')
def _goose(g, src, p, m):
    if m is src: make_artifact_tokens(g, src.owner, 'Food', 1)


CI.DYN_MANA['Gilded Goose'] = lambda g, p, m: 1 if getattr(p, 'foods', 0) else 0
CI.ON_TAP['Gilded Goose'] = lambda g, p, m, used: sac_food(g, p)
card('Gilded Goose', 'pow=0 tgh=2 fly dork=A noatk', dsl=[])
note('Gilded Goose', 'Approximate', 'Food on entry; taps and sacrifices a Food for mana (the make-a-Food ability unused)')


@on('Trail of Crumbs', 'etb')
def _trail(g, src, p, m):
    if m is src: make_artifact_tokens(g, src.owner, 'Food', 1)


@on('Trail of Crumbs', 'sacrifice')
def _trail_sac(g, src, p, what):
    if p is src.owner and what == 'Food' and can_pay(g, p, 1, '') and len(p.library) > 5:
        pay(g, p, 1, '')
        top = [p.library.pop() for _ in range(min(2, len(p.library)))]
        perm = [c for c in top if c.perm or c.land]
        if perm:
            c = max(perm, key=lambda c: card_worth(g, p, c)); top.remove(c); p.hand.append(c)
        p.library[:0] = top
card('Trail of Crumbs', '', types='E', dsl=[])
note('Trail of Crumbs', 'Full', '')


@on("Witch's Oven", 'options')
def _oven(g, src, p, s, post):
    if src.tapped or post is None: return []
    fod = [m for m in p.perms if m.creature and not m.is_cmd and (m.token or pval(g, m) < 2)]
    if not fod: return []
    m = min(fod, key=lambda x: pval(g, x))
    v = IC.death_value(g, p) + 1.0 - 1.2 * pval(g, m)
    if v < 1.0: return []

    def go():
        if src.tapped or m not in p.perms: return False
        src.tapped = True; n = 2 if etgh(g, m) >= 4 else 1
        die(g, m, 'sac'); make_artifact_tokens(g, p, 'Food', n); return True
    return [(v, f"Witch's Oven ({m.name})", go)]
card("Witch's Oven", '', types='A', dsl=[])
note("Witch's Oven", 'Full', 'sacrifice spare creatures for Food when the death is worth it')


@on('Savvy Hunter', 'attack')
def _savvy(g, src, p, atk, d):
    if src in atk: make_artifact_tokens(g, p, 'Food', 1)


@on('Savvy Hunter', 'options')
def _savvy_draw(g, src, p, s, post):
    if getattr(p, 'foods', 0) < 2 or post is None: return []

    def go():
        if getattr(p, 'foods', 0) < 2: return False
        sac_food(g, p, 2); draw(g, p, 1); return True
    return [(2.0, 'Savvy Hunter: two Foods for a card', go)]
card('Savvy Hunter', 'human warrior pow=3', dsl=[])
note('Savvy Hunter', 'Full', '')


@on('Grim Hireling', 'combat_damage')
def _hireling(g, src, p, a, d, dmg):
    if a.owner is src.owner and once_per_turn(g, src.owner, f'hire{id(src)}{id(d)}'):
        make_artifact_tokens(g, src.owner, 'Treasure', 2)
card('Grim Hireling', 'pow=3 tgh=2', dsl=[])
note('Grim Hireling', 'Partial', 'two Treasures per player hit; the -X/-X ability is not used')


@on('Old Gnawbone', 'combat_damage')
def _gnawbone(g, src, p, a, d, dmg):
    if a.owner is src.owner: make_artifact_tokens(g, src.owner, 'Treasure', dmg)
card('Old Gnawbone', 'leg pow=7 fly bomb=7', dsl=[])
note('Old Gnawbone', 'Full', '')


@on('Awakening Zone', 'upkeep')
def _azone(g, src, p):
    if p is src.owner: make_tokens(g, p, 1, 0, 1, color='', types=('eldrazi', 'spawn'))
card('Awakening Zone', '', types='E', dsl=[])
note('Awakening Zone', 'Approximate', '0/1 Spawn each upkeep (sacrifice fodder; the mana ability is not used)')


@on('Bloodghast', 'gy_landfall')
def _bloodghast(g, c, p):
    if c in p.gy: p.gy.remove(c); enter(g, p, c)
card('Bloodghast', 'pow=2 tgh=1 noblock', dsl=[])
note('Bloodghast', 'Approximate', 'returns on landfall; the haste clause is ignored')


@on('Reassembling Skeleton', 'gy_options')
def _skeleton(g, c, p, s, post):
    if post is None or not can_pay(g, p, 1, 'B') or not IC.outlets(p): return []

    def go():
        if c not in p.gy or not can_pay(g, p, 1, 'B'): return False
        pay(g, p, 1, 'B'); p.gy.remove(c); m = enter(g, p, c); m.tapped = True; return True
    return [(0.5 + IC.death_value(g, p) / 2.0, 'return Reassembling Skeleton', go)]
card('Reassembling Skeleton', 'pow=1 warrior', dsl=[])
note('Reassembling Skeleton', 'Full', 'recurs itself when there is a sacrifice outlet')


@on('Gravecrawler', 'gy_options')
def _gravecrawler(g, c, p, s, post):
    if post is None or not can_pay(g, p, 0, 'B') or not any(has_type(m, 'zombie') for m in p.perms): return []
    if not IC.outlets(p) and not any(m.cd is not None and m.cd.name == 'Phyrexian Altar' for m in p.perms): return []

    def go():
        if c not in p.gy or not can_pay(g, p, 0, 'B'): return False
        pay(g, p, 0, 'B'); p.gy.remove(c); p.spells_this_turn += 1; on_cast(g, p, c); enter(g, p, c); return True
    return [(0.5 + IC.death_value(g, p) / 2.0, 'cast Gravecrawler from the graveyard', go)]
card('Gravecrawler', 'pow=2 tgh=1 noblock', dsl=[])
note('Gravecrawler', 'Full', 'castable from the graveyard with a Zombie; the Phyrexian Altar loop is a combo (see combos)')


def _grist_plus(g, p, src):
    make_tokens(g, p, 1, 1, color='BG', types=('insect',)); mill(g, p, 1)


def _grist_minus(g, p, src):
    fod = [m for m in p.perms if m.creature and (m.token or pval(g, m) < 2)]
    t = best_opp_creature(g, p)
    if fod and t is not None:
        die(g, min(fod, key=lambda m: pval(g, m)), 'sac'); apply_removal(g, p, t, 'destroy')


walker('Grist, the Hunger Tide', [
    (1, 'Insect, mill', always(2.5), _grist_plus),
    (-2, 'sacrifice: destroy', lambda g, p, src: (pval(g, t) - 2.0 if (t := best_opp_creature(g, p)) is not None and
                                                  pval(g, t) >= 4 and any(m.creature and (m.token or pval(g, m) < 2) for m in p.perms)
                                                  else None), _grist_minus),
    (-5, 'drain creature cards', lambda g, p, src: 1.5 * sum(1 for c in p.gy if c.creature) - 3,
     lambda g, p, src: [lose_life(g, q, sum(1 for c in p.gy if c.creature), p, kind='drain') for q in g.opps(p)]),
], ('Approximate', 'insect-mill repeat on milled Insects ignored'))


card('Fable of the Mirror-Breaker // Reflection of Kiki-Jiki', 'fable', types='E', dsl=[])
note('Fable of the Mirror-Breaker // Reflection of Kiki-Jiki', 'Approximate', 'chapter I Goblin (Treasure on attack), '
     'II rummage two, III flips into Reflection (copy a nonlegendary creature each turn; Kiki combos in combos)')


@on('Fable of the Mirror-Breaker // Reflection of Kiki-Jiki', 'etb')
def _fable(g, src, p, m):
    if m is src:
        src.data = {'lore': 1}
        for t in make_tokens(g, src.owner, 1, 2, color='R', types=('goblin', 'shaman')): t.data = {'fable_goblin': True}


@on('Fable of the Mirror-Breaker // Reflection of Kiki-Jiki', 'upkeep')
def _fable_lore(g, src, p):
    if p is not src.owner or not src.data or 'lore' not in src.data: return
    src.data['lore'] += 1
    if src.data['lore'] == 2:
        k = min(2, len(p.hand))
        worst = sorted(p.hand, key=lambda c: card_worth(g, p, c))[:k]
        if worst: discard_cards(g, p, worst); draw(g, p, len(worst))
    elif src.data['lore'] >= 3:
        src.data = {'reflection': True}; src.data['flipped'] = p.turns
        log('    Fable flips into Reflection of Kiki-Jiki', g)


@on('Fable of the Mirror-Breaker // Reflection of Kiki-Jiki', 'attack')
def _fable_goblin(g, src, p, atk, d):
    if src.owner is p:
        for m in atk:
            if m.data and m.data.get('fable_goblin'): make_artifact_tokens(g, p, 'Treasure', 1)


# ======================================================== Marwyn, the Nurturer (Elves, Craterhoof)
@on('Marwyn, the Nurturer', 'etb')
def _marwyn(g, src, p, m):
    if m is not src and m.owner is src.owner and has_type(m, 'elf'): src.plus += 1
card('Marwyn, the Nurturer', 'leg pow=1 dork=G', dsl=[])
note('Marwyn, the Nurturer', 'Full', '+1/+1 counter per Elf; taps for G equal to its power')


def hoof_bonus(g, p):
    return sum(1 for m in p.perms if m.creature and not m.phased)


@on('Craterhoof Behemoth', 'etb')
def _hoof(g, src, p, m):
    if m is not src: return
    o = src.owner; x = hoof_bonus(g, o)
    for c in o.perms:
        if c.creature: _eot(g, c, x, x); g.eot_kw.setdefault(id(c), set()).add('trample')
    o.trample = True
    log(f'    Craterhoof: creatures get +{x}/+{x} and trample', g)
card('Craterhoof Behemoth', 'pow=5 haste bomb=8', dsl=[])
note('Craterhoof Behemoth', 'Full', 'haste; creatures +X/+X and trample (X = creatures you control)')


def hoof_prio(g, p, c):
    """cast Craterhoof (or tutor into it) only when the board makes it lethal-ish; before combat"""
    n = sum(1 for m in p.perms if m.creature and not m.phased)
    return 88 if n >= 6 else (30 if n >= 4 else 0)


CI.SPELL_PRIO['Craterhoof Behemoth'] = hoof_prio


def _put_creature(g, p, pred, prefer_hoof=True):
    cs = [c for c in p.library if c.creature and pred(c)]
    if not cs: return None
    import pool_ai
    wish = [c for c in cs if c.name in pool_ai.wish_list(g, p)]
    if wish:
        c = wish[0]; p.library.remove(c); g.rng.shuffle(p.library); p.stats['tutored'] += 1; enter(g, p, c); return c
    hoof = next((c for c in cs if c.name == 'Craterhoof Behemoth'), None)
    n = sum(1 for m in p.perms if m.creature and not m.phased)
    c = hoof if (hoof is not None and prefer_hoof and n >= 5) else max(cs, key=lambda c: (card_worth(g, p, c), c.cmc))
    p.library.remove(c); g.rng.shuffle(p.library); p.stats['tutored'] += 1
    enter(g, p, c)
    return c


@IC.spell('Natural Order', prio=lambda g, p, c: 80 if sum(1 for m in p.perms if m.creature) >= 5 and any(
    x.name == 'Craterhoof Behemoth' for x in p.library) else (40 if any(m.creature and (m.token or pval(g, m) < 2) for m in p.perms) else 0),
          status=('Full', 'sacrifices a spare green creature for the best green creature (Craterhoof on a wide board)'))
def _natural_order(g, p, c, ctx):
    _put_creature(g, p, lambda x: 'G' in x.pips)


def _x_tutor(name, cost_pips, extra=0, pred=lambda c: True, status=('Full', '')):
    """{X}+pips: put a creature with MV <= X (+extra) onto the battlefield: X = all spare mana"""
    @IC.spell(name, prio=lambda g, p, c: 60 if total_mana(g, p) >= 3 + len(cost_pips) else 0, status=status)
    def _r(g, p, c, ctx):
        x = ctx.get('x', 0) + extra
        _put_creature(g, p, lambda y: y.cmc <= x and pred(y))
        if name == 'Finale of Devastation' and ctx.get('x', 0) >= 10:
            for m in p.perms:
                if m.creature: _eot(g, m, ctx['x'], ctx['x']); m.sick = False
    card(name, 'xtutor', types='S', dsl=[])


_x_tutor("Green Sun's Zenith", 'G', pred=lambda c: 'G' in c.pips, status=('Full', 'X = spare mana; green creature onto the battlefield'))
_x_tutor('Finale of Devastation', 'GG', status=('Approximate', 'X = spare mana; library only (the graveyard option is not used)'))
note('Chord of Calling', 'Full-auto', 'compiled (convoke)')


@IC.spell('Eldritch Evolution', prio=lambda g, p, c: 55 if any(m.creature and not m.is_cmd for m in p.perms) else 0,
          status=('Full', 'sacrifice the lowest creature, fetch one with MV up to two more onto the battlefield'))
def _eldritch(g, p, c, ctx):
    snap = getattr(c, 'sac_snapshot', None) or {'mv': 1}
    _put_creature(g, p, lambda x: x.cmc <= snap['mv'] + 2, prefer_hoof=False)
    c.sac_snapshot = None
    return 'exile'


card('Eldritch Evolution', '', types='S',
     dsl=[{'type': 'additional_cost', 'text': 'as an additional cost to cast ~, sacrifice a creature'}])


CI.DYN_MANA['Joraga Treespeaker'] = lambda g, p, m: 2 if (m.data or {}).get('level') else 1


@on('Joraga Treespeaker', 'options')
def _joraga(g, src, p, s, post):
    if post is not False or (src.data or {}).get('level') or not can_pay(g, p, 1, 'G'): return []

    def go():
        if not can_pay(g, p, 1, 'G'): return False
        pay(g, p, 1, 'G'); src.data = {'level': 1}; return True
    return [(3.0, 'level up Joraga Treespeaker', go)]
card('Joraga Treespeaker', 'pow=1 dork=G noatk', dsl=[])
note('Joraga Treespeaker', 'Approximate', 'levels once to tap for GG (level 5 not modeled)')
card('Arbor Elf', 'pow=1 dork=G noatk', dsl=[])
note('Arbor Elf', 'Approximate', 'untapping a Forest read as tapping for G')


@on('Kogla, the Titan Ape', 'etb')
def _kogla(g, src, p, m):
    if m is src:
        t = best_opp_creature(g, src.owner, lambda x: etgh(g, x) <= 7)
        if t is not None: apply_removal(g, src.owner, t, 'dmg7')


@on('Kogla, the Titan Ape', 'attack')
def _kogla_atk(g, src, p, atk, d):
    if src in atk:
        ts = [m for m in d.perms if m.cd is not None and ('A' in m.cd.types or 'E' in m.cd.types) and not untargetable(g, m)]
        if ts: apply_removal(g, p, max(ts, key=lambda m: pval(g, m)), 'destroy')
card('Kogla, the Titan Ape', 'leg pow=7 tgh=6 bomb=7', dsl=[])
note('Kogla, the Titan Ape', 'Approximate', 'fights on entry (as 7 damage), destroys an artifact/enchantment on attack')


@IC.spell('Primal Might', prio=0, tags='primalmight', types='S', status=('Full', 'X pump and fight: used as removal'))
def _primal(g, p, c, ctx): pass


@on('Esika\'s Chariot', 'etb')
def _chariot(g, src, p, m):
    if m is src: make_tokens(g, src.owner, 2, 2, color='G', types=('cat',))


@on('Esika\'s Chariot', 'attack')
def _chariot_atk(g, src, p, atk, d):
    if src in atk:
        toks = [m for m in p.perms if m.token and m.creature]
        if toks:
            t = max(toks, key=lambda m: epow(g, m))
            make_tokens(g, p, 1, t.pow, t.tgh, fly=t.fly, color=t.colors, types=t.ttypes)
card('Esika\'s Chariot', 'leg pow=4', types='AC', dsl=[])
note('Esika\'s Chariot', 'Approximate', 'two Cats on entry; attacks as a 4/4 (crew not required) copying a token')


def _freyalise_minus(g, p, src):
    t = best_opp_nonland(g, p, lambda m: m.cd is not None and ('A' in m.cd.types or 'E' in m.cd.types))
    if t is not None: apply_removal(g, p, t, 'destroy')


walker("Freyalise, Llanowar's Fury", [
    (2, 'Elf Druid mana token', always(3.0),
     lambda g, p, src: [setattr(t, 'data', {'dork': True}) for t in make_tokens(g, p, 1, 1, color='G', types=('elf', 'druid'))]),
    (-2, 'destroy artifact/enchantment', lambda g, p, src: (pval(g, t) - 2 if (t := best_opp_nonland(g, p, lambda m: m.cd is not None and (
        'A' in m.cd.types or 'E' in m.cd.types))) is not None and pval(g, t) >= 4 else None), _freyalise_minus),
    (-6, 'draw per green creature', lambda g, p, src: 0.8 * sum(1 for m in p.perms if m.creature),
     lambda g, p, src: draw(g, p, sum(1 for m in p.perms if m.creature))),
], ('Approximate', 'the Elf Druid tokens are not mana sources'))


@on('Nissa, Who Shakes the World', 'land_mana')
def _nissa_wstw(g, src, p, L):
    return 1 if p is src.owner and ('forest' in L.cd.subtypes or L.cd.name == 'Forest') else 0


walker('Nissa, Who Shakes the World', [
    (1, 'land becomes a 3/3 haste', always(3.0),
     lambda g, p, src: make_tokens(g, p, 1, 3, 3, sick=False, color='G', types=('elemental',))),
], ('Approximate', 'Forests tap for an extra G; +1 makes a 3/3 haste (as a token, not a land); the ultimate is not used'))




# ======================================================== Aurelia (extra combats)
@on('Combat Celebrant', 'attack')
def _celebrant(g, src, p, atk, d):
    if src not in atk or (src.data or {}).get('exerted') in (p.turns, p.turns - 1): return
    src.data = dict(src.data or {}, exerted=p.turns)
    for m in p.perms:
        if m.creature and m is not src: m.tapped = False
    p.extra_combats += 1
    log('    Combat Celebrant exerts: untap, additional combat', g)
card('Combat Celebrant', 'human warrior pow=4 tgh=1', dsl=[])
note('Combat Celebrant', 'Full', 'exerts (not two turns running): untap the others, additional combat')


@on('Hellkite Charger', 'attack')
def _charger(g, src, p, atk, d):
    if src in atk and can_pay(g, p, 5, 'RR') and sum(epow(g, m) for m in atk) >= 8 and once_per_turn(g, p, f'charger{id(src)}{p.combat_no}'):
        pay(g, p, 5, 'RR')
        for m in atk: m.tapped = False
        p.extra_combats += 1
card('Hellkite Charger', 'pow=5 fly haste bomb=5', dsl=[])
note('Hellkite Charger', 'Full', 'pays {5}{R}{R} for another combat when the attack is big enough')


@on('Port Razer', 'combat_damage')
def _razer(g, src, p, a, d, dmg):
    if a is src and once_per_turn(g, p, f'razer{id(src)}{id(d)}'):
        for m in p.perms:
            if m.creature: m.tapped = False
        p.extra_combats += 1
card('Port Razer', 'pow=4', dsl=[])
note('Port Razer', 'Approximate', 'combat damage: untap all, additional combat (once per player hit)')


def _combat_spell(name, cost_note, entwine=False, rebound=False, fb=None, status=('Full', '')):
    @IC.spell(name, prio=lambda g, p, c: 70 if g.active is p and getattr(p, 'combat_no', 0) == 0 and
              sum(1 for m in p.perms if m.creature and not m.sick and not m.noatk) >= 2 else 0, status=status)
    def _r(g, p, c, ctx):
        p.extra_combats += 1
        if name == 'Savage Beating' and can_pay(g, p, 1, 'R'):
            pay(g, p, 1, 'R')
            for m in p.perms:
                if m.creature: g.eot_kw.setdefault(id(m), set()).add('double strike')
        if rebound:
            p.rebound = getattr(p, 'rebound', []) + [c]; return 'exile'
    card(name, 'combatspell' + (f' fb={fb}' if fb else ''), types='S' if name != 'Savage Beating' else 'I', dsl=[])


_combat_spell('Relentless Assault', '')
_combat_spell('Seize the Day', '', fb='2RR', status=('Full', 'additional combat; flashback'))
_combat_spell('World at War', '', rebound=True, status=('Full', 'additional combat; rebound next turn'))
_combat_spell('Savage Beating', '', entwine=True, status=('Approximate', 'cast before combat: additional combat, '
                                                                         'double strike when entwined'))


def _war_rebound(g, p, c):
    if c in p.exile: p.exile.remove(c); p.gy.append(c); p.extra_combats += 1


CI.HOOKS.setdefault('World at War', {})['rebound'] = _war_rebound


@on('Embercleave', 'etb')
def _embercleave(g, src, p, m):
    if m is not src: return
    cr = [x for x in src.owner.perms if x.creature and not x.noatk]
    if cr: src.attached = max(cr, key=lambda x: (not x.sick, epow(g, x)))


CI.SPELL_PRIO['Embercleave'] = lambda g, p, c: 70 if any(m.creature and not m.sick for m in p.perms) else 0
card('Embercleave', 'leg flash', types='A', dsl=[{'type': 'static', 'static': 'equip_bonus', 'pow': 1, 'tgh': 1},
                                                  {'type': 'static', 'static': 'equip_keyword', 'keyword': 'double strike'},
                                                  {'type': 'static', 'static': 'equip_keyword', 'keyword': 'trample'},
                                                  {'type': 'static', 'static': 'equip_cost', 'mana': 3}])
note('Embercleave', 'Approximate', 'cast in the main phase onto the best attacker (the attacker discount is not applied)')


@on('Helm of the Host', 'combat_start')
def _helm(g, src, p):
    if src.owner is not p or src.attached is None or src.attached not in p.perms or src.attached.cd is None: return
    t = enter_token_copy(g, p, src.attached.cd)
    if t is None: return
    t.sick = False
    log(f'    Helm of the Host copies {src.attached.name}', g)
    return [t]
card('Helm of the Host', 'leg', types='A', dsl=[{'type': 'static', 'static': 'equip_cost', 'mana': 5}])
note('Helm of the Host', 'Full', 'a hasty token copy of the equipped creature at the start of each combat')


@on('Legion Loyalist', 'attack')
def _loyalist(g, src, p, atk, d):
    if src in atk and len(atk) >= 3:
        for m in atk: g.eot_kw.setdefault(id(m), set()).update(('first strike', 'trample'))
card('Legion Loyalist', 'pow=1 haste', dsl=[])
note('Legion Loyalist', 'Approximate', 'battalion: first strike and trample (the token-blocking clause is ignored)')


@on('Stoneforge Mystic', 'etb')
def _sfm(g, src, p, m):
    if m is src:
        import impl_t1; impl_t1.tutor_named(g, src.owner, lambda c: 'equipment' in c.subtypes)
card('Stoneforge Mystic', 'pow=1 tgh=2', dsl=[])
note('Stoneforge Mystic', 'Approximate', 'fetches an Equipment; the put-onto-battlefield ability is not used')
card('Winds of Abandon', 'rem=exile tgt=c rland', types='S', dsl=[])
note('Winds of Abandon', 'Approximate', 'single-target mode (overload not used)')


@on('Outpost Siege', 'upkeep')
def _siege(g, src, p):
    if p is src.owner and p.library:
        c = p.library.pop(); p.hand.append(c); p.impulse.append(c); p.seen_names.add(c.name)
card('Outpost Siege', '', types='E', dsl=[])
note('Outpost Siege', 'Approximate', 'Khans: an impulse card each upkeep')


# ======================================================== Atraxa (superfriends, proliferate)
@on("Atraxa, Praetors' Voice", 'end_step')
def _atraxa(g, src, p):
    if p is src.owner: IC.proliferate(g, p)
note("Atraxa, Praetors' Voice", 'Approximate', 'proliferates at your end step (+1/+1, loyalty, opponents\' -1/-1); '
     'in the four main decks\' own games it keeps its hand tag')


@on('Evolution Sage', 'landfall')
def _evosage(g, src, p):
    if p is src.owner: IC.proliferate(g, p)
card('Evolution Sage', 'pow=3 tgh=2', dsl=[])
note('Evolution Sage', 'Full', 'landfall: proliferate')


@on('Flux Channeler', 'cast')
def _flux(g, src, caster, c):
    if caster is src.owner and not c.creature: IC.proliferate(g, caster)
note('Flux Channeler', 'Full', 'noncreature spell: proliferate (pool games)')


@on('Inexorable Tide', 'cast')
def _tide(g, src, caster, c):
    if caster is src.owner: IC.proliferate(g, caster)
note('Inexorable Tide', 'Full', 'every spell: proliferate (pool games)')


@on('Thrummingbird', 'combat_damage')
def _thrum(g, src, p, a, d, dmg):
    if a is src: IC.proliferate(g, p)
card('Thrummingbird', 'pow=1 fly', dsl=[])
note('Thrummingbird', 'Full', '')


@on('Tekuthal, Inquiry Dominus', 'proliferate_extra')
def _tekuthal(g, src, p): return 1 if p is src.owner else 0
card('Tekuthal, Inquiry Dominus', 'leg pow=3 tgh=5 fly', dsl=[])
note('Tekuthal, Inquiry Dominus', 'Partial', 'proliferate twice; its indestructible ability is not used')


@on('Doubling Season', 'etb')
def _ds(g, src, p, m):
    if m is not src and m.owner is src.owner and m.loyalty is not None and m.cd is not None and 'P' in m.cd.types:
        m.loyalty *= 2
card('Doubling Season', '', types='E', dsl=[{'type': 'replacement', 'replace': 'tokens', 'multiplier': 2},
                                            {'type': 'replacement', 'replace': 'counters', 'multiplier': 2}])
note('Doubling Season', 'Full', 'tokens and counters doubled; planeswalkers enter with double loyalty; + abilities '
     'and proliferate add double')


@IC.spell('Contentious Plan', prio=35, types='S', status=('Full', ''))
def _plan(g, p, c, ctx):
    IC.proliferate(g, p); draw(g, p, 1)


def _jace_minus(g, p, src):
    t = best_opp_creature(g, p)
    if t is not None: apply_removal(g, p, t, 'bounce')


walker('Jace, the Mind Sculptor', [
    (0, 'Brainstorm', always(3.5), lambda g, p, src: CI.HOOKS['Brainstorm']['resolve'](g, p, None, {})),
    (2, 'fateseal', always(2.0), lambda g, p, src: None),
    (-1, 'bounce a creature', lambda g, p, src: (pval(g, t) - 3.0 if (t := best_opp_creature(g, p)) is not None and pval(g, t) >= 5 else None), _jace_minus),
], ('Approximate', 'Brainstorm, fateseal as loyalty only, bounce; the ultimate is not used'))


def _karn_minus(g, p, src):
    t = best_opp_nonland(g, p)
    if t is not None: apply_removal(g, p, t, 'exile')


walker('Karn Liberated', [
    (4, 'opponent exiles a card from hand', always(2.5),
     lambda g, p, src: (lambda q: q.hand and (q.hand.remove(min(q.hand, key=lambda c: card_worth(g, q, c))) or True))(
         max(g.opps(p), key=lambda q: len(q.hand)))),
    (-3, 'exile a permanent', lambda g, p, src: (pval(g, t) - 2.0 if (t := best_opp_nonland(g, p)) is not None and pval(g, t) >= 4 else None), _karn_minus),
], ('Approximate', 'the restart ultimate is not used; +4 takes the opponent\'s worst card'))


def _oko_elk(g, p, src):
    t = best_opp_creature(g, p)
    if t is not None and t.cd is not None:
        t.neutered = True; t.data = dict(t.data or {}, elk=True)
        a, b = g.eot_pt.get(id(t), (0, 0))
        t.pow, t.tgh, t.plus = 3, 3, 0


walker('Oko, Thief of Crowns', [
    (2, 'Food', always(2.0), lambda g, p, src: make_artifact_tokens(g, p, 'Food', 1)),
    (1, 'Elk an opposing creature', lambda g, p, src: (pval(g, t) - 2.0 if (t := best_opp_creature(g, p)) is not None and pval(g, t) >= 4 else None), _oko_elk),
], ('Approximate', '+1 turns the best opposing creature into a vanilla 3/3; the exchange ability is not used'))


def _ttr_minus(g, p, src):
    t = best_opp_nonland(g, p, lambda m: m.creature or (m.cd is not None and ('A' in m.cd.types or 'E' in m.cd.types)))
    if t is not None: apply_removal(g, p, t, 'bounce')
    draw(g, p, 1)


walker('Teferi, Time Raveler', [
    (1, 'sorceries at instant speed', always(1.5), lambda g, p, src: None),
    (-3, 'bounce and draw', lambda g, p, src: (pval(g, t) - 1.5 if (t := best_opp_nonland(g, p, lambda m: m.creature or (
        m.cd is not None and ('A' in m.cd.types or 'E' in m.cd.types)))) is not None and pval(g, t) >= 3 else None), _ttr_minus),
], ('Approximate', 'opponents cast spells only at sorcery speed (their counterspells and instant removal are off '
                   'on other turns); +1 has no effect here'))


@on('Teferi, Time Raveler', 'can_cast')
def _ttr_lock(g, src, caster, c, zone):
    if caster is src.owner or g.active is caster: return True
    return False


def _garruk_overrun(g, p, src):
    for m in p.perms:
        if m.creature: _eot(g, m, 3, 3)
    p.trample = True


walker('Garruk Wildspeaker', [
    (1, 'untap two lands', always(1.5), lambda g, p, src: [setattr(L, 'tapped', False) for L in p.lands[:2]]),
    (-1, '3/3 Beast', always(2.8), lambda g, p, src: make_tokens(g, p, 1, 3, color='G', types=('beast',))),
    (-4, 'overrun', lambda g, p, src: 1.5 * sum(1 for m in p.perms if m.creature and not m.sick) - 2 if g.active is p else None,
     _garruk_overrun),
], ('Full', ''))


def _vraska_plus(g, p, src):
    fod = [m for m in p.perms if m is not src and (m.token or pval(g, m) < 1.5)]
    if fod: die(g, min(fod, key=lambda m: pval(g, m)), 'sac'); gain(p, 1); draw(g, p, 1)
    elif getattr(p, 'foods', 0) or p.treasures or p.clues:
        import impl_t3; impl_t3.sac_worst_permanent(g, p, src); gain(p, 1); draw(g, p, 1)


def _vraska_minus(g, p, src):
    t = best_opp_nonland(g, p, lambda m: m.cd is not None and m.cd.cmc <= 3)
    if t is not None: apply_removal(g, p, t, 'destroy')


walker('Vraska, Golgari Queen', [
    (2, 'sacrifice: gain 1, draw', lambda g, p, src: 2.5 if any(m is not src and (m.token or pval(g, m) < 1.5) for m in p.perms)
     or getattr(p, 'foods', 0) or p.treasures or p.clues else None, _vraska_plus),
    (-3, 'destroy MV 3 or less', lambda g, p, src: (pval(g, t) - 2.0 if (t := best_opp_nonland(g, p, lambda m: m.cd is not None and m.cd.cmc <= 3)) is not None
                                                     and pval(g, t) >= 3.5 else None), _vraska_minus),
], ('Approximate', 'the emblem is not used'))


def _ugin_minus(g, p, src):
    x = src.loyalty
    for q in g.players:
        for m in list(q.perms):
            if m.cd is not None and set(m.cd.pips) & set('WUBRG') and m.cd.cmc <= 4 and not m.cd.land:
                exile_perm(g, m)


walker('Ugin, the Spirit Dragon', [
    (2, '3 damage', always(3.0), lambda g, p, src: best_target_any(g, p, 3)),
    (-4, 'exile coloured permanents MV 4 or less', lambda g, p, src: (lambda o, m: (o - m) / 2 - 1 if o - m >= 8 else None)(
        sum(pval(g, m) for q in g.opps(p) for m in q.perms if m.cd is not None and set(m.cd.pips) & set('WUBRG') and m.cd.cmc <= 4),
        sum(pval(g, m) for m in p.perms if m.cd is not None and set(m.cd.pips) & set('WUBRG') and m.cd.cmc <= 4)), _ugin_minus),
], ('Approximate', '-X fixed at 4; the ultimate is not used'))


def _emperor_exile(g, p, src):
    t = best_opp_creature(g, p, lambda m: m.tapped)
    if t is not None: apply_removal(g, p, t, 'exile'); gain(p, 2)


walker('The Wandering Emperor', [
    (1, '+1/+1 counter, first strike', always(1.5), lambda g, p, src: None),
    (-1, '2/2 Samurai', always(2.8), lambda g, p, src: make_tokens(g, p, 1, 2, color='W', types=('samurai',))),
    (-2, 'exile a tapped creature', lambda g, p, src: (pval(g, t) - 1.5 if (t := best_opp_creature(g, p, lambda m: m.tapped)) is not None
                                                        and pval(g, t) >= 3 else None), _emperor_exile),
], ('Approximate', 'used at sorcery speed (flash timing not modeled)'))


walker('Tamiyo, Field Researcher', [
    (1, 'draw on combat damage', always(2.0), lambda g, p, src: draw(g, p, 1) if any(m.creature and not m.sick for m in p.perms) else None),
    (-2, 'tap two threats', lambda g, p, src: 1.5 if any(True for q in g.opps(p) for m in q.perms if m.creature) else None,
     lambda g, p, src: [setattr(m, 'tapped', True) for m in sorted([m for q in g.opps(p) for m in q.perms if m.creature],
                                                                   key=lambda m: -pval(g, m))[:2]]),
], ('Approximate', '+1 approximated as a card; the free-spells ultimate is not used'))


@IC.spell('Hour of Revelation', prio=lambda g, p, c: 70 if IC_wipe_gain(g, p) >= 10 else 0, types='S',
          status=('Full', 'destroys all nonland permanents when that swings the board'))
def _hour_rev(g, p, c, ctx):
    for q in g.players:
        for m in list(q.perms):
            if not m.phased: die(g, m, 'destroy')


def IC_wipe_gain(g, p):
    return sum(pval(g, m) for q in g.opps(p) for m in q.perms) - 1.3 * sum(pval(g, m) for m in p.perms)


@on('Ichormoon Gauntlet', 'options')
def _ichormoon(g, src, p, s, post):
    ws = [m for m in p.perms if m.cd is not None and 'P' in m.cd.types and m.loyalty_used != (g.round, p.key)]
    if post is None or not ws: return []

    def go():
        w = ws[0]
        if w.loyalty_used == (g.round, p.key): return False
        w.loyalty_used = (g.round, p.key); IC.proliferate(g, p); return True
    return [(1.0, 'Ichormoon Gauntlet: proliferate', go)]
card('Ichormoon Gauntlet', '', types='A', dsl=[])
note('Ichormoon Gauntlet', 'Approximate', 'a planeswalker may proliferate instead of its own ability')


# ======================================================== Tergrid (discard, edicts)
def _on_opp_discard(name, fn, status=('Full', '')):
    @on(name, 'discard')
    def _d(g, src, q, c):
        if q is not src.owner: fn(g, src.owner, src, q, c)
    note(name, *status)


_on_opp_discard("Liliana's Caress", lambda g, p, s, q, c: lose_life(g, q, 2, p, kind='drain'))
_on_opp_discard('Megrim', lambda g, p, s, q, c: lose_life(g, q, 2, p, kind='triggers'))
_on_opp_discard("Geth's Grimoire", lambda g, p, s, q, c: draw(g, p, 1) if len(p.library) > 10 else None)
_on_opp_discard('Waste Not', lambda g, p, s, q, c: (make_tokens(g, p, 1, 2, color='B', types=('zombie',)) if c.creature else
                                                  setattr(p, 'floatA', p.floatA + 2) if c.land else draw(g, p, 1)))
for _n in ("Liliana's Caress", 'Megrim', "Geth's Grimoire", 'Waste Not'): card(_n, '', types='E' if _n != "Geth's Grimoire" else 'A', dsl=[])


def _discard_random(g, q, n):
    for _ in range(n):
        if q.hand: discard_index(g, q, g.rng.randrange(len(q.hand)))


@IC.spell('Hymn to Tourach', prio=50, types='S', status=('Full', 'two random discards from the fullest hand'))
def _hymn(g, p, c, ctx):
    q = max(g.opps(p), key=lambda q: (len(q.hand), threat(g, p, q)), default=None)
    if q: _discard_random(g, q, 2)


@IC.spell('Mind Twist', prio=lambda g, p, c: 50 if total_mana(g, p) >= 4 else 0, types='S',
          status=('Full', 'X = spare mana, random discards'))
def _twist(g, p, c, ctx):
    q = max(g.opps(p), key=lambda q: (len(q.hand), threat(g, p, q)), default=None)
    if q: _discard_random(g, q, ctx.get('x', 3))


card('Mind Twist', 'xtutor', types='S', dsl=[])


@IC.spell('Painful Quandary', prio=45, types='E', status=('Approximate', 'opponents pay 5 life per spell (they never discard)'))
def _quandary_never(g, p, c, ctx): pass


@on('Painful Quandary', 'cast')
def _quandary(g, src, caster, c):
    if caster is not src.owner:
        if caster.hand and caster.life <= 15: discard_worst(g, caster, 1)
        else: lose_life(g, caster, 5, src.owner, kind='drain')
card('Painful Quandary', '', types='E', dsl=[])


def _lotv_plus(g, p, src):
    for q in g.players:
        if q.alive and q.hand:
            if q is p: discard_worst(g, q, 1)
            else: discard_index(g, q, g.rng.randrange(len(q.hand)))


walker('Liliana of the Veil', [
    (1, 'each player discards', lambda g, p, src: 2.0 if sum(len(q.hand) for q in g.opps(p)) >= 2 else 0.5, _lotv_plus),
    (-2, 'edict', lambda g, p, src: 3.0 if any(m.creature for q in g.opps(p) for m in q.perms) else None,
     lambda g, p, src: edict(g, max(g.opps(p), key=lambda q: threat(g, p, q)))),
], ('Approximate', 'the ultimate is not used'))


walker('Liliana, Dreadhorde General', [
    (1, '2/2 Zombie', always(2.8), lambda g, p, src: make_tokens(g, p, 1, 2, color='B', types=('zombie',))),
    (-4, 'each player sacrifices two', lambda g, p, src: 4.0 if sum(1 for q in g.opps(p) for m in q.perms if m.creature) >= 3 else None,
     lambda g, p, src: [edict(g, q) for q in g.players if q.alive for _ in range(2)]),
], ('Approximate', 'creatures dying draw (static); the ultimate is not used'))


@on('Liliana, Dreadhorde General', 'dies')
def _ldg(g, src, m, cause):
    if m.owner is src.owner and m.creature: draw(g, src.owner, 1)


def _each_opp_discard_etb(name, n, tags, status=('Full', '')):
    @on(name, 'etb')
    def _e(g, src, p, m):
        if m is src:
            for q in g.opps(src.owner): discard_worst_for(g, q, n)
    card(name, tags, dsl=[])
    note(name, *status)


def discard_worst_for(g, q, n):
    """an opponent chooses what to discard: their least useful card"""
    for _ in range(n):
        if q.hand: discard_cards(g, q, [min(q.hand, key=lambda c: card_worth(g, q, c))])


_each_opp_discard_etb('Burglar Rat', 1, 'pow=1')
_each_opp_discard_etb('Elderfang Disciple', 1, 'pow=1')


@on('Chittering Rats', 'etb')
def _rats(g, src, p, m):
    if m is src and g.opps(src.owner):
        q = max(g.opps(src.owner), key=lambda q: threat(g, src.owner, q))
        if q.hand:
            c = min(q.hand, key=lambda c: card_worth(g, q, c)); q.hand.remove(c); q.library.append(c)
card('Chittering Rats', 'pow=2', dsl=[])
note('Chittering Rats', 'Full', '')


def _each_player_sacrifice(g, p, nontoken=False, spare_self=True):
    for q in g.players:
        if not q.alive: continue
        cr = [m for m in q.perms if m.creature and not m.phased and not (nontoken and m.token)]
        if not cr: continue
        die(g, min(cr, key=lambda m: pval(g, m)), 'sac')


@on('Accursed Marauder', 'etb')
def _marauder(g, src, p, m):
    if m is src: _each_player_sacrifice(g, src.owner, nontoken=True)
card('Accursed Marauder', 'pow=3 tgh=1', dsl=[])
note('Accursed Marauder', 'Full', 'each player sacrifices a nontoken creature')


@on('Merciless Executioner', 'etb')
def _executioner(g, src, p, m):
    if m is src: _each_player_sacrifice(g, src.owner)
card('Merciless Executioner', 'pow=3 tgh=1', dsl=[])
note('Merciless Executioner', 'Full', '')


@IC.spell('Innocent Blood', prio=lambda g, p, c: 45 if any(m.creature for q in g.opps(p) for m in q.perms) else 0, types='S')
def _innocent(g, p, c, ctx): _each_player_sacrifice(g, p)


@on('Archfiend of Depravity', 'end_step')
def _depravity(g, src, p):
    if p is src.owner: return
    cr = sorted([m for m in p.perms if m.creature and not m.phased], key=lambda m: -pval(g, m))
    for m in cr[2:]: die(g, m, 'sac')
card('Archfiend of Depravity', 'pow=5 tgh=4 fly bomb=6', dsl=[])
note('Archfiend of Depravity', 'Full', 'opponents keep only two creatures at their end step')


@on('Braids, Arisen Nightmare', 'end_step')
def _braids_an(g, src, p):
    if p is not src.owner: return
    fod = [m for m in p.perms if m.creature and m is not src and (m.token or pval(g, m) < 2)]
    if not fod: return
    die(g, min(fod, key=lambda m: pval(g, m)), 'sac')
    for q in g.opps(p):
        cr = [m for m in q.perms if m.creature]
        if cr: die(g, min(cr, key=lambda m: pval(g, m)), 'sac')
        else: lose_life(g, q, 2, p, kind='drain'); draw(g, p, 1)
card('Braids, Arisen Nightmare', 'leg pow=3 tgh=3', dsl=[])
note('Braids, Arisen Nightmare', 'Approximate', 'sacrifices a spare creature each end step (creature type only)')


@on('Phyrexian Obliterator', 'blocks')
def _obliterator(g, src, p, atk, d, assign):
    if d is src.owner:
        for a, b in assign.items():
            if b is src:
                for _ in range(epow(g, a)):
                    ps = [x for x in p.perms if not x.phased]
                    if not ps: break
                    die(g, min(ps, key=lambda x: pval(g, x)), 'sac')
card('Phyrexian Obliterator', 'pow=5 tgh=5 trample bomb=6', dsl=[])
note('Phyrexian Obliterator', 'Approximate', 'blocking it costs the attacker that many permanents; damage from spells '
     'not counted')


@on('Tinybones, Trinket Thief', 'end_step')
def _tinybones(g, src, p):
    if p is not src.owner: return
    if any(getattr(q, 'discarded_turn', None) == turn_stamp(g) for q in g.opps(p)):
        cs = [c for q in g.opps(p) for c in q.gy if not c.land and c.cmc <= 4]
        if cs:
            c = max(cs, key=lambda c: card_worth(g, p, c))
            for q in g.opps(p):
                if c in q.gy: q.gy.remove(c)
            if c.perm: enter(g, p, c)
card('Tinybones, Trinket Thief', 'leg pow=1 tgh=2', dsl=[])
note('Tinybones, Trinket Thief', 'Approximate', 'end step after a discard: plays the best cheap permanent from an '
     'opponent\'s graveyard')


@on('Rotting Regisaur', 'upkeep')
def _regisaur(g, src, p):
    if p is src.owner and p.hand: discard_worst(g, p, 1)
card('Rotting Regisaur', 'pow=7 tgh=6 bomb=6', dsl=[])
note('Rotting Regisaur', 'Full', '')


@on('Gix, Yawgmoth Praetor', 'combat_damage')
def _gix(g, src, p, a, d, dmg):
    if a.owner is src.owner and d is not src.owner and p.life > 10 and len(p.library) > 10:
        lose_life(g, p, 1, p); draw(g, p, 1)
card('Gix, Yawgmoth Praetor', 'leg pow=3 tgh=3', dsl=[])
note('Gix, Yawgmoth Praetor', 'Partial', 'draw for combat damage; the discard-to-play ability is not used')


note('Tergrid, God of Fright // Tergrid\'s Lantern', 'Full', 'steals permanents opponents sacrifice or discard '
     '(hand-coded); the Lantern back face is not used')


CI.SPELL_PRIO['Academy Manufactor'] = 50
note("Hunter's Insight", 'Unmodeled', 'the combat-damage draw is not modeled; never cast')
