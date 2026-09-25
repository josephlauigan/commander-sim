"""Tier 4 pool decks: Yuriko, Krenko, Chulane, Prosper, Grand Arbiter Augustin IV (and the counterspells they share)."""
from engine import *
import engine as E
import cardimpl as CI
from cardimpl import on, _eot, count_type
from pool_cards import card, note
import impl_common as IC
from impl_common import make_artifact_tokens, best_opp_creature, best_opp_nonland, walker, always
from impl_t2 import best_target_any

# ---- counterspell tags
card('Force of Negation', 'ctr=nc fon', types='I', dsl=[])
note('Force of Negation', 'Full', 'free on opponents\' turns (exile a blue card)')
card('Pact of Negation', 'ctr=any pact', types='I', dsl=[])
note('Pact of Negation', 'Full', 'free; {3}{U}{U} at the next upkeep or lose (only cast with five lands)')
card('Mental Misstep', 'ctr=mv1 misstep', types='I', dsl=[])
note('Mental Misstep', 'Full', 'counters a mana-value-1 spell for 2 life')
card('Stern Scolding', 'ctr=cre2', types='I', dsl=[])
note('Stern Scolding', 'Full', '')
card('Pyroblast', 'ctr=blue rem=destroy tgt=blue', types='I', dsl=[])
note('Pyroblast', 'Full', 'counter a blue spell or destroy a blue permanent')
card('Red Elemental Blast', 'ctr=blue rem=destroy tgt=blue', types='I', dsl=[])
note('Red Elemental Blast', 'Full', '')
card('Deadly Rollick', 'rem=exile tgt=c freecmd', types='I', dsl=[])
note('Deadly Rollick', 'Full', 'free with your commander out')
card('Snuff Out', 'rem=destroy tgt=c snuff', types='I', dsl=[])
note('Snuff Out', 'Approximate', 'free for 4 life with a Swamp; the nonblack restriction is ignored')
card('Fatal Push', 'rem=destroy tgt=c mv4', types='I', dsl=[])
note('Fatal Push', 'Approximate', 'destroys mana value 4 or less (revolt assumed)')
note('Mana Drain', 'Approximate', 'counterspell; the extra mana next turn is not modeled')


# ======================================================== Ninjutsu and Yuriko, the Tiger's Shadow
NINJUTSU = {'Yuriko, the Tiger\'s Shadow': (0, 'UB'), 'Ink-Eyes, Servant of Oni': (3, 'BB'), 'Fallen Shinobi': (2, 'UB'),
            'Higure, the Still Wind': (2, 'UU'), 'Ingenious Infiltrator': (0, 'UB'), 'Mist-Syndicate Naga': (2, 'U'),
            'Mistblade Shinobi': (0, 'U'), 'Moon-Circuit Hacker': (0, 'U'), 'Prosperous Thief': (1, 'U'),
            'Silent-Blade Oni': (4, 'UB'), 'Silver-Fur Master': (0, 'UB'), 'Thousand-Faced Shadow': (2, 'UU')}


def ninjutsu_cost(g, p, name):
    gen, pips = NINJUTSU.get(name, (2, 'UB'))                          # Satoru grants {2}{U}{B}
    disc = sum(1 for m in p.perms if m.cd is not None and m.cd.name == 'Silver-Fur Master' and not m.phased)
    return max(0, gen - disc), pips


def ninjutsu(g, p, atk, d, assign):
    """after blockers: return an unblocked attacker to hand and put a Ninja onto the battlefield tapped and attacking"""
    for _ in range(3):
        unbl = [a for a in atk if a not in assign and a in p.perms and not a.is_cmd]
        if not unbl: return
        satoru = any(m.cd is not None and m.cd.name == 'Satoru Umezawa, Mirror of the Ninja' or
                     (m.cd is not None and m.cd.name == 'Satoru Umezawa') for m in p.perms)
        cands = [c for c in p.hand if c.name in NINJUTSU or (satoru and c.creature)]
        yuri = p.cmd if (p.cmd.name == 'Yuriko, the Tiger\'s Shadow' and p.cmd_in_zone) else None
        if yuri is not None: cands.append(yuri)
        best = None
        for c in cands:
            gen, pips = ninjutsu_cost(g, p, c.name)
            if c is yuri: gen, pips = 0, 'UB'
            if c.name not in NINJUTSU and satoru: gen, pips = 2, 'UB'
            if not can_pay(g, p, gen, pips): continue
            v = (c.bomb or c.pow) + (6 if c is yuri else 0) + (3 if c.name in NINJUTSU else 0)
            if best is None or v > best[0]: best = (v, c, gen, pips)
        if best is None: return
        _, c, gen, pips = best
        a = min(unbl, key=lambda m: (pval(g, m), -epow(g, m)))
        if pval(g, a) > (c.bomb or c.pow) + 3 and c is not yuri: return
        pay(g, p, gen, pips)
        atk.remove(a)
        if a.token: leave(g, a)
        else: bounce(g, a)
        if c is yuri: p.cmd_in_zone = False
        else: p.hand.remove(c)
        m = enter(g, p, c); m.tapped = True; m.sick = False
        if c is p.cmd: m.is_cmd = True
        atk.append(m)
        p.stats['ninjutsu'] += 1
        log(f'    ninjutsu: {c.name} replaces {a.name}', g)
        if g.hooks:
            g.ninja_atk = atk                      # the attack a ninjutsu trigger can add to (Thousand-Faced Shadow)
            try:
                CI.fire(g, 'ninjutsu', p, m)
            finally:
                g.ninja_atk = None


@on('Satoru Umezawa', 'ninjutsu')
def _satoru(g, src, p, m):
    if p is src.owner and once_per_turn(g, p, f'satoru{id(src)}'):
        top = [p.library.pop() for _ in range(min(3, len(p.library)))]
        if top:
            c = max(top, key=lambda c: card_worth(g, p, c)); top.remove(c); p.hand.append(c)
        p.library[:0] = top
card('Satoru Umezawa', 'leg human pow=2 tgh=4', dsl=[])
note('Satoru Umezawa', 'Full', 'every creature card in hand has ninjutsu {2}{U}{B}; ninjutsu digs three deep')


def _ninja(m):
    return has_type(m, 'ninja')


@on('Yuriko, the Tiger\'s Shadow', 'combat_damage')
def _yuriko(g, src, p, a, d, dmg):
    if a.owner is not src.owner or not _ninja(a) or not p.library: return
    c = p.library.pop(); p.hand.append(c); p.seen_names.add(c.name)
    if c.cmc:
        for q in g.opps(p): lose_life(g, q, c.cmc, p, kind='triggers')
    log(f'    Yuriko reveals {c.name}: each opponent loses {c.cmc}', g)
card('Yuriko, the Tiger\'s Shadow', 'leg human pow=1 tgh=3', dsl=[])
note('Yuriko, the Tiger\'s Shadow', 'Full', 'commander ninjutsu; each Ninja hit reveals the top card: to hand, each '
     'opponent loses its mana value')


YURIKO_BIG = ['Draco', 'Enter the Infinite', 'Dig Through Time', 'Treasure Cruise']
EVASIVE_ONE = ('Ornithopter', 'Changeling Outcast', 'Gudul Lurker', 'Slither Blade', 'Faerie Seer', 'Spectral Sailor',
               'Triton Shorestalker', 'Siren Stormtamer', 'Hope of Ghirapur', 'Moon-Circuit Hacker', 'Ingenious Infiltrator')


def yuriko_wish(g, p):
    """tutors: a big card for the reveal only when it ends up on top (Vampiric Tutor, or a way to put it back);
    otherwise card advantage and Yuriko's enablers"""
    yuri = any(m.is_cmd for m in p.perms) or p.cmd_in_zone
    putback = (getattr(p, 'to_top', False) or any(c.name == 'Brainstorm' for c in p.hand)
               or any(m.cd is not None and m.cd.name in ('Scroll Rack', "Sensei's Divining Top") for m in p.perms))
    if yuri and putback: return YURIKO_BIG
    return ['Rhystic Study', 'Scroll Rack', "Sensei's Divining Top", 'Mystic Remora']


def yuriko_prio(g, p, c):
    """evasive one-drops first: they carry Yuriko in on turn two; with a ninjutsu window open, cantrips before
    combat stack the biggest cards for the reveal"""
    if c.name in ('Brainstorm', 'Ponder', 'Preordain') and __import__('pool_ai').combat_reserve(g, p):
        big = any(x.cmc >= 5 for x in p.hand if x is not c) or c.name != 'Brainstorm'
        return 72 if big else 45
    if c.name in EVASIVE_ONE or (c.creature and c.cmc <= 2 and ('fly' in c.tags or c.name in ('Invisible Stalker',))):
        have = sum(1 for m in p.perms if m.creature and not m.phased)
        return 80 if have < 2 else 58
    return None


for _n, (_g, _p) in NINJUTSU.items():
    note(_n, 'Full', f'ninjutsu {_g}{_p}') if _n not in ('Yuriko, the Tiger\'s Shadow',) else None


@on('Ingenious Infiltrator', 'combat_damage')
def _infiltrator(g, src, p, a, d, dmg):
    if a.owner is src.owner and _ninja(a): draw(g, src.owner, 1)


@on('Mistblade Shinobi', 'combat_damage')
def _mistblade(g, src, p, a, d, dmg):
    if a is src:
        t = max([m for m in d.perms if m.creature and not untargetable(g, m)], key=lambda m: pval(g, m), default=None)
        if t is not None: bounce(g, t)


@on('Moon-Circuit Hacker', 'combat_damage')
def _hacker(g, src, p, a, d, dmg):
    if a is src: draw(g, p, 1)


@on('Prosperous Thief', 'combat_damage')
def _thief(g, src, p, a, d, dmg):
    if a.owner is src.owner and (_ninja(a) or has_type(a, 'rogue')) and once_per_turn(g, p, f'thief{id(src)}{id(d)}'):
        make_artifact_tokens(g, p, 'Treasure', 1)


@on('Mist-Syndicate Naga', 'combat_damage')
def _naga(g, src, p, a, d, dmg):
    if a is src: enter_token_copy(g, p, src.cd)


@on('Ink-Eyes, Servant of Oni', 'combat_damage')
def _inkeyes(g, src, p, a, d, dmg):
    if a is src:
        cs = [c for c in d.gy if c.creature]
        if cs:
            c = max(cs, key=lambda c: (c.bomb or c.pow)); d.gy.remove(c); enter(g, p, c, orig=d)


@on('Fallen Shinobi', 'combat_damage')
def _shinobi(g, src, p, a, d, dmg):
    if a is not src: return
    for _ in range(2):
        if not d.library: return
        c = d.library.pop()
        if c.land: d.exile.append(c); continue
        d.exile.append(c)
        if c.perm: enter(g, p, c, orig=d)
        elif c.instant or c.sorcery: d.exile.remove(c); p.hand.append(c); cast_card(g, p, c, 'hand', {})


@on('Silent-Blade Oni', 'combat_damage')
def _oni(g, src, p, a, d, dmg):
    if a is not src: return
    cs = [c for c in d.hand if not c.land and not ('ctr' in c.tags)]
    if cs:
        c = max(cs, key=lambda c: card_worth(g, d, c)); d.hand.remove(c)
        if c.perm: enter(g, p, c, orig=d)
        else: p.hand.append(c); cast_card(g, p, c, 'hand', {})


@on('Higure, the Still Wind', 'combat_damage')
def _higure(g, src, p, a, d, dmg):
    if a is src:
        import impl_t1; impl_t1.tutor_named(g, p, lambda c: 'ninja' in c.subtypes)


@on('Thousand-Faced Shadow', 'ninjutsu')
def _tfs(g, src, p, m):
    """it entered from hand attacking (ninjutsu): a token copy of another attacking creature, tapped and attacking.
    Only the ninjutsu itself triggers it, so a token copy (or a Shadow tapped by Thalia, Heretic Cathar) never does."""
    atk = getattr(g, 'ninja_atk', None)
    if m is not src or atk is None: return
    others = [x for x in atk if x is not src and x.cd is not None and x in x.owner.perms]
    if others:
        t = enter_token_copy(g, src.owner, max(others, key=lambda x: pval(g, x)).cd)
        if t is not None:
            t.tapped = True; t.sick = False; atk.append(t)


card('Silver-Fur Master', 'pow=2', dsl=[{'type': 'static', 'static': 'anthem', 'filter': {'type': 'creature', 'controller': 'you',
                                                                                          'other': True, 'subtype': 'ninja'}, 'pow': 1, 'tgh': 1}])
for _n in ('Mistblade Shinobi', 'Moon-Circuit Hacker', 'Prosperous Thief', 'Mist-Syndicate Naga', 'Ink-Eyes, Servant of Oni',
           'Fallen Shinobi', 'Silent-Blade Oni', 'Higure, the Still Wind', 'Thousand-Faced Shadow', 'Ingenious Infiltrator'):
    card(_n, None, dsl=[])
note('Silver-Fur Master', 'Approximate', 'ninjutsu discount; other Ninjas +1/+1 (Rogues not counted)')
card('Changeling Outcast', 'pow=1 noblock', dsl=[{'type': 'static', 'static': 'unblockable'}])
note('Changeling Outcast', 'Full', 'unblockable, every creature type (a Ninja for Yuriko)')
card('Gudul Lurker', 'pow=1', dsl=[{'type': 'static', 'static': 'unblockable'}])
note('Gudul Lurker', 'Approximate', 'unblockable; megamorph not used')


@on('Tetsuko Umezawa, Fugitive', 'grant_kw')
def _tetsuko(g, src, m, kw):
    return kw == 'unblockable' and m.owner is src.owner and m.creature and (epow(g, m) <= 1 or etgh(g, m) <= 1)
card('Tetsuko Umezawa, Fugitive', 'leg human pow=1 tgh=3', dsl=[])
note('Tetsuko Umezawa, Fugitive', 'Full', '')


def _kaito_minus(g, p, src):
    for t in make_tokens(g, p, 1, 1, color='U', types=('ninja',)): t.data = {'unblockable': True}


walker('Kaito Shizuki', [
    (1, 'draw (discard unless attacked)', always(2.5), lambda g, p, src: (draw(g, p, 1),)),
    (-2, 'unblockable Ninja', always(3.0), _kaito_minus),
], ('Approximate', 'phasing on its first turn and the emblem are not modeled; the Ninja token is unblockable'))


# ======================================================== Krenko, Mob Boss (Goblins)
GOBLIN = ('goblin',)


def goblins(g, p, n):
    return make_tokens(g, p, n, 1, color='R', types=GOBLIN)


@on('Krenko, Mob Boss', 'options')
def _krenko(g, src, p, s, post):
    """{T}: a Goblin per Goblin (instant speed). Without a haste enabler the tokens can't attack the turn they're
    made, so Krenko waits for the end of the turn before yours (the tokens are ready to attack, and sorcery-speed
    removal and wipes had their chance); with haste, activating in your main phase attacks right away."""
    if src.tapped or (src.sick and not E.DSLMOD.has_kw(g, src, 'haste') and not _goblin_haste(p)): return []
    n = count_type(g, p, 'goblin')
    haste = _goblin_haste(p) or any(m.cd is not None and m.cd.name == 'Legion Loyalist' for m in p.perms)
    if post is None: u = 3.5 + 0.4 * n                       # end of the turn before yours
    elif post is False: u = (3.0 + 0.4 * n) if haste else (0.2 if n < 4 else 1.0 + 0.2 * n)
    else: u = 0.2 + 0.1 * n                                   # after combat: only if nothing better will come

    def go():
        if src.tapped: return False
        src.tapped = True; n = count_type(g, p, 'goblin'); goblins(g, p, n)
        log(f'  Krenko makes {n} Goblins', g); return True
    return [(u, f'Krenko: {n} Goblins', go)]


def _goblin_haste(p):
    return any(m.cd is not None and m.cd.name in ('Goblin Warchief', 'Goblin Chieftain') for m in p.perms)


@on('Goblin Warchief', 'grant_kw')
def _warchief_haste(g, src, m, kw):
    return kw == 'haste' and m.owner is src.owner and has_type(m, 'goblin')


@on('Goblin Chieftain', 'grant_kw')
def _chieftain_haste(g, src, m, kw):
    return kw == 'haste' and m.owner is src.owner and has_type(m, 'goblin') and m is not src
card('Krenko, Mob Boss', 'leg warrior pow=3', dsl=[])
note('Krenko, Mob Boss', 'Full', 'tap: a Goblin per Goblin you control (haste from Warchief / Chieftain / Greaves)')
IC._reducer('Goblin Warchief', lambda c: 'goblin' in c.subtypes, status=('Full', 'Goblin spells cost {1} less; Goblins have haste'))
card('Goblin Warchief', 'pow=2 warrior', dsl=[])
card('Goblin Chieftain', 'pow=2 haste', dsl=[{'type': 'static', 'static': 'anthem', 'filter': {'type': 'creature', 'controller': 'you',
                                                                                              'other': True, 'subtype': 'goblin'}, 'pow': 1, 'tgh': 1}])
note('Goblin Chieftain', 'Full', '')
IC._reducer('Ruby Medallion', lambda c: 'R' in c.pips)


def _etb_goblins(name, n, tags, status=('Full', '')):
    @on(name, 'etb')
    def _e(g, src, p, m):
        if m is src: goblins(g, src.owner, n)
    card(name, tags, dsl=[])
    note(name, *status)


_etb_goblins('Beetleback Chief', 2, 'pow=2 warrior')
_etb_goblins('Siege-Gang Commander', 3, 'pow=2', ('Approximate', 'three Goblins; the sacrifice-for-2 ability works as a sacrifice outlet'))
_etb_goblins('Goblin Instigator', 1, 'pow=1')


@on('Mogg War Marshal', 'etb')
def _mogg(g, src, p, m):
    if m is src: goblins(g, src.owner, 1)


@on('Mogg War Marshal', 'self_dies')
def _mogg_dies(g, m, cause): goblins(g, m.owner, 1)
card('Mogg War Marshal', 'pow=1 warrior', dsl=[])
note('Mogg War Marshal', 'Approximate', 'Goblin on entry and death; echo not paid')


@IC.spell('Hordeling Outburst', prio=55, types='S')
def _outburst(g, p, c, ctx): goblins(g, p, 3)


@on('Impact Tremors', 'etb')
def _tremors(g, src, p, m):
    if m.owner is src.owner and m.creature and m is not src:
        for q in g.opps(src.owner): lose_life(g, q, 1, src.owner, kind='triggers')
card('Impact Tremors', '', types='E', dsl=[])
note('Impact Tremors', 'Full', '')


@on('Purphoros, God of the Forge', 'etb')
def _purphoros(g, src, p, m):
    if m.owner is src.owner and m.creature and m is not src:
        for q in g.opps(src.owner): lose_life(g, q, 2, src.owner, kind='triggers')
card('Purphoros, God of the Forge', 'leg', types='E', dsl=[])
note('Purphoros, God of the Forge', 'Approximate', '2 damage per creature entering; never a creature (devotion ignored); pump unused')


IC.SAC_OUTLET['Skirk Prospector'] = lambda g, p, src, m: setattr(p, 'floatA', p.floatA + 1)
note('Skirk Prospector', 'Approximate', 'free sacrifice outlet (Goblins) adding one mana')


@on('Pashalik Mons', 'dies')
def _mons(g, src, m, cause):
    if m.owner is src.owner and (m is src or has_type(m, 'goblin')): best_target_any(g, src.owner, 1)
card('Pashalik Mons', 'leg pow=2 warrior', dsl=[])
note('Pashalik Mons', 'Partial', 'a Goblin dying deals 1; the token-making activation is not used')


@on('Goblin Sharpshooter', 'dies')
def _sharpshooter(g, src, m, cause):
    if m is not src: best_target_any(g, src.owner, 1)
card('Goblin Sharpshooter', 'pow=1', dsl=[])
note('Goblin Sharpshooter', 'Approximate', '1 damage whenever a creature dies (the untap loop is abstracted)')


@on('Goblin Chainwhirler', 'etb')
def _chainwhirler(g, src, p, m):
    if m is src:
        for q in g.opps(src.owner):
            lose_life(g, q, 1, src.owner, kind='triggers')
            for x in list(q.perms):
                if x.creature and etgh(g, x) <= 1: die(g, x, 'destroy')
card('Goblin Chainwhirler', 'pow=3', dsl=[], kws={'first strike'})
note('Goblin Chainwhirler', 'Full', '')


@on('Goblin Lackey', 'combat_damage')
def _lackey(g, src, p, a, d, dmg):
    if a is src:
        gs = [c for c in p.hand if 'goblin' in c.subtypes and c.perm]
        if gs: c = max(gs, key=lambda c: c.cmc); p.hand.remove(c); enter(g, p, c)
card('Goblin Lackey', 'pow=1', dsl=[])
note('Goblin Lackey', 'Full', '')


@on('Goblin Recruiter', 'etb')
def _recruiter(g, src, p, m):
    if m is not src: return
    o = src.owner
    gs = sorted([c for c in o.library if 'goblin' in c.subtypes], key=lambda c: card_worth(g, o, c))[-4:]
    for c in gs: o.library.remove(c)
    g.rng.shuffle(o.library); o.library.extend(gs)
card('Goblin Recruiter', 'pow=1', dsl=[])
note('Goblin Recruiter', 'Approximate', 'stacks the four best Goblins on top')


@on('Goblin Ringleader', 'etb')
def _ringleader(g, src, p, m):
    if m is src:
        import impl_t1; impl_t1.look_take(g, src.owner, 4, lambda c: 'goblin' in c.subtypes, k=4)
card('Goblin Ringleader', 'pow=2 haste', dsl=[])
note('Goblin Ringleader', 'Full', '')


@on('Muxus, Goblin Grandee', 'etb')
def _muxus(g, src, p, m):
    if m is not src: return
    o = src.owner
    top = [o.library.pop() for _ in range(min(6, len(o.library)))]
    for c in top:
        if c.creature and 'goblin' in c.subtypes and c.cmc <= 5: enter(g, o, c)
        else: o.library.insert(0, c)
card('Muxus, Goblin Grandee', 'leg pow=4', dsl=[])
note('Muxus, Goblin Grandee', 'Partial', 'Goblins from the top six onto the battlefield; the attack pump is not used')


@on('Goblin Piledriver', 'attack')
def _piledriver(g, src, p, atk, d):
    if src in atk: _eot(g, src, 2 * sum(1 for m in atk if m is not src and has_type(m, 'goblin')), 0)
card('Goblin Piledriver', 'pow=1 tgh=2 warrior', dsl=[])
note('Goblin Piledriver', 'Full', '')


@on('Battle Cry Goblin', 'attack')
def _bcg(g, src, p, atk, d):
    if src in atk and sum(epow(g, m) for m in atk) >= 6:
        return make_tokens(g, p, 1, 1, color='R', types=GOBLIN, attacking=True, sick=False)
card('Battle Cry Goblin', 'pow=2', dsl=[])
note('Battle Cry Goblin', 'Approximate', 'pack tactics Goblin; the pump is not used')


@on('Krenko, Tin Street Kingpin', 'attack')
def _kingpin(g, src, p, atk, d):
    if src in atk: src.plus += 1; goblins(g, p, epow(g, src))
card('Krenko, Tin Street Kingpin', 'leg pow=1 tgh=2', dsl=[])
note('Krenko, Tin Street Kingpin', 'Full', '')


card('Coat of Arms', '', types='A', dsl=[])
note('Coat of Arms', 'Unmodeled', 'the shared-type anthem is not modeled (too costly to compute per creature)')


@IC.spell('Battle Hymn', prio=0, types='I', status=('Approximate', 'adds R per creature (used only with a big hand)'))
def _hymn(g, p, c, ctx): p.floatA += sum(1 for m in p.perms if m.creature)


card('Conspicuous Snoop', 'pow=2 rogue', dsl=[])
note('Conspicuous Snoop', 'Partial', 'body only')


@on('Rionya, Fire Dancer', 'combat_start')
def _rionya(g, src, p):
    if src.owner is not p: return
    cr = [m for m in p.perms if m.creature and m is not src and m.cd is not None]
    if not cr: return
    t = enter_token_copy(g, p, max(cr, key=lambda m: pval(g, m)).cd)
    if t is not None: t.sick = False; t.temp = True; return [t]
card('Rionya, Fire Dancer', 'leg human wizard pow=3 tgh=4', dsl=[])
note('Rionya, Fire Dancer', 'Approximate', 'one hasty copy of the best creature each combat (instant/sorcery count ignored)')


@on('Thornbite Staff', 'etb')
def _staff(g, src, p, m):
    if m is src:
        k = next((x for x in src.owner.perms if x.cd is not None and x.cd.name == 'Krenko, Mob Boss'), None)
        if k is not None: src.attached = k
card('Thornbite Staff', '', types='A', dsl=[{'type': 'static', 'static': 'equip_cost', 'mana': 4}])
note('Thornbite Staff', 'Approximate', 'attaches to Krenko; the untap loop is a combo')


def _chandra_plus(g, p, src):
    if p.library:
        c = p.library.pop()
        if not c.land and can_pay(g, p, c.generic, c.pips) and not any(k in c.tags for k in ('ctr',)):
            p.hand.append(c); pay(g, p, c.generic, c.pips); cast_card(g, p, c, 'hand', {})
        else:
            p.exile.append(c)
            for q in g.opps(p): lose_life(g, q, 2, p, kind='triggers')


walker('Chandra, Torch of Defiance', [
    (1, 'exile top: cast it or 2 damage each', always(3.0), _chandra_plus),
    (1, 'RR', always(1.0), lambda g, p, src: setattr(p, 'floatR', p.floatR + 2)),
    (-3, '4 damage to a creature', lambda g, p, src: (pval(g, t) - 2.0 if (t := best_opp_creature(g, p, lambda m: etgh(g, m) <= 4)) is not None
                                                        and pval(g, t) >= 4 else None),
     lambda g, p, src: apply_removal(g, p, best_opp_creature(g, p, lambda m: etgh(g, m) <= 4), 'dmg4')),
], ('Approximate', 'the emblem is not used'))


@on('Valakut Exploration', 'landfall')
def _valakut(g, src, p):
    if p is src.owner and p.library:
        c = p.library.pop(); p.hand.append(c); p.impulse.append(c); p.seen_names.add(c.name)
        src.data = src.data or {}; src.data['n'] = src.data.get('n', 0) + 1


@on('Valakut Exploration', 'end_step')
def _valakut_end(g, src, p):
    if p is src.owner and src.data and src.data.get('n'):
        n = src.data['n']; src.data['n'] = 0
        for q in g.opps(p): lose_life(g, q, min(n, sum(1 for c in p.exile) + 1), p, kind='triggers')
card('Valakut Exploration', '', types='E', dsl=[])
note('Valakut Exploration', 'Approximate', 'landfall impulse card; unplayed cards deal damage at end step (approximately)')


# ======================================================== Chulane, Teller of Tales (and Aluren)
@on('Chulane, Teller of Tales', 'cast')
def _chulane(g, src, caster, c):
    if caster is not src.owner or not c.creature: return
    draw(g, caster, 1)
    import dsl
    dsl.run(g, caster, {'do': 'put_land'}, None, {}, None, 0)
card('Chulane, Teller of Tales', 'leg human pow=2 tgh=4 vig', dsl=[])
note('Chulane, Teller of Tales', 'Full', 'creature spells: draw, then a land from hand onto the battlefield (the bounce '
     'activation is only part of the Aluren combo)')


@on('Aluren', 'cost')
def _aluren(g, src, caster, c):
    if c.creature and c.cmc <= 3: return -99
    return 0
card('Aluren', '', types='E', dsl=[])
note('Aluren', 'Approximate', 'creature spells with MV 3 or less are free for everyone (the flash part is ignored); '
     'the Chulane loop is a combo')


def _self_bounce(name, pred_other, tags, status=('Approximate', '')):
    @on(name, 'etb')
    def _b(g, src, p, m):
        if m is not src: return
        o = src.owner
        mine = [x for x in o.perms if x is not src and pred_other(x) and not x.is_cmd]
        cands = [x for x in mine if x.cd is not None and CI.HOOKS.get(x.cd.name, {}).get('etb') is not None or
                 (x.cd is not None and __import__('dsl').etb_value(x.cd) > 0)]
        tgt = max(cands, key=lambda x: __import__('dsl').etb_value(x.cd), default=None) if cands else None
        if tgt is None: tgt = src
        if tgt.token: leave(g, tgt)
        else: bounce(g, tgt)
    card(name, tags, dsl=[])
    note(name, *status)


_self_bounce('Shrieking Drake', lambda x: x.creature, 'pow=1 fly', ('Approximate', 'returns your best ETB creature (or itself)'))
_self_bounce('Whitemane Lion', lambda x: x.creature, 'pow=2 flash', ('Approximate', 'returns your best ETB creature (or itself)'))
_self_bounce('Kor Skyfisher', lambda x: x.cd is not None and not x.cd.land, 'pow=2 tgh=3 fly',
             ('Approximate', 'returns your best ETB permanent (or itself)'))
card("Man-o'-War", 'pow=2 rem=bounce tgt=c etb', dsl=[])
note("Man-o'-War", 'Full', 'bounces the best opposing creature')


@on('Consecrated Sphinx', 'draw')
def _sphinx(g, src, p):
    if p is not src.owner and once_per_turn(g, src.owner, f'sphinx{id(src)}{p.key}{p.draw_n}') and len(src.owner.library) > 12:
        draw(g, src.owner, 2)
card('Consecrated Sphinx', 'pow=4 tgh=6 fly bomb=6', dsl=[])
note('Consecrated Sphinx', 'Approximate', 'draws two per opponent draw (capped by library size)')


@on('Hullbreaker Horror', 'cast')
def _hullbreaker(g, src, caster, c):
    if caster is not src.owner: return
    t = best_opp_nonland(g, caster)
    if t is not None and pval(g, t) >= 3: bounce(g, t)
card('Hullbreaker Horror', 'pow=7 tgh=8 flash unc bomb=7', dsl=[])
note('Hullbreaker Horror', 'Approximate', 'uncounterable; each of your spells bounces the best opposing nonland permanent')


card('Temur Sabertooth', 'pow=4 tgh=3', dsl=[])
note('Temur Sabertooth', 'Partial', 'body only (re-buying ETB creatures is not used)')


# ======================================================== Prosper, Tome-Bound (Treasure, impulse draw)
@on('Prosper, Tome-Bound', 'end_step')
def _prosper(g, src, p):
    if p is src.owner and p.library:
        c = p.library.pop(); p.hand.append(c); p.seen_names.add(c.name)
        p.impulse_long = getattr(p, 'impulse_long', []) + [(c, p.turns + 1)]


@on('Prosper, Tome-Bound', 'cast')
def _prosper_treasure(g, src, caster, c):
    if caster is src.owner and (c in caster.impulse or any(x is c for x, _ in getattr(caster, 'impulse_long', []))):
        make_artifact_tokens(g, caster, 'Treasure', 1)
card('Prosper, Tome-Bound', 'leg pow=1 tgh=4 dt', dsl=[])
note('Prosper, Tome-Bound', 'Full', 'end step: exile the top card, playable until the end of your next turn; a '
     'Treasure whenever you play a card from exile (Prosper, Laelia, Valakut, Siege, Reckless Impulse ...)')


@IC.spell('Reckless Impulse', prio=45, types='S')
def _reckless(g, p, c, ctx):
    for _ in range(2):
        if p.library:
            x = p.library.pop(); p.hand.append(x); p.seen_names.add(x.name)
            p.impulse_long = getattr(p, 'impulse_long', []) + [(x, p.turns + 1)]


@on('Mahadi, Emporium Master', 'end_step')
def _mahadi(g, src, p):
    if p is src.owner: make_artifact_tokens(g, p, 'Treasure', getattr(g, 'deaths_turn', {}).get(turn_stamp(g), 0))


@on('Mahadi, Emporium Master', 'dies')
def _mahadi_count(g, src, m, cause):
    if m.creature:
        g.deaths_turn = getattr(g, 'deaths_turn', {}); st = turn_stamp(g); g.deaths_turn[st] = g.deaths_turn.get(st, 0) + 1
card('Mahadi, Emporium Master', 'leg pow=3', dsl=[])
note('Mahadi, Emporium Master', 'Approximate', 'counts deaths while it is on the battlefield')


@on('Ragavan, Nimble Pilferer', 'combat_damage')
def _ragavan(g, src, p, a, d, dmg):
    if a is src:
        make_artifact_tokens(g, p, 'Treasure', 1)
        if d.library:
            c = d.library.pop(); d.exile.append(c)
            if not c.land and can_pay(g, p, *cost_of(p, c)) and c.perm:
                pay(g, p, *cost_of(p, c)); d.exile.remove(c); enter(g, p, c, orig=d)
card('Ragavan, Nimble Pilferer', 'leg pow=2 tgh=1', dsl=[])
note('Ragavan, Nimble Pilferer', 'Approximate', 'Treasure and a stolen top card on hit (only permanents are cast); dash not used')


@on('Xorn', 'token_created')
def _xorn(g, src, p, kinds, n):
    if p is src.owner and 'Treasure' in kinds: p.treasures += 1
card('Xorn', 'pow=3 tgh=2', dsl=[])
note('Xorn', 'Approximate', 'an extra Treasure for hooked Treasure makers')


@on('Reckless Fireweaver', 'token_created')
def _fireweaver(g, src, p, kinds, n):
    if p is src.owner:
        for q in g.opps(p): lose_life(g, q, n, p, kind='triggers')
card('Reckless Fireweaver', 'human pow=1 tgh=3', dsl=[])
note('Reckless Fireweaver', 'Approximate', 'artifact tokens made by hooks trigger it; cast artifacts do not')


@on('Dark Confidant', 'upkeep')
def _bob(g, src, p):
    if p is src.owner and p.library:
        c = p.library.pop(); p.hand.append(c); lose_life(g, p, c.cmc, p)
card('Dark Confidant', 'human wizard pow=2 tgh=1', dsl=[])
note('Dark Confidant', 'Full', '')


@on('Tavern Scoundrel', 'options')
def _scoundrel(g, src, p, s, post):
    if src.tapped or src.sick or post is None or not can_pay(g, p, 1, '') or not (p.treasures or getattr(p, 'foods', 0) or p.clues): return []

    def go():
        if src.tapped or not can_pay(g, p, 1, ''): return False
        pay(g, p, 1, ''); src.tapped = True
        import impl_t3; impl_t3.sac_worst_permanent(g, p, src)
        if g.rng.random() < 0.5: make_artifact_tokens(g, p, 'Treasure', 2)
        return True
    return [(1.0, 'Tavern Scoundrel flip', go)]
card('Tavern Scoundrel', 'human rogue pow=1 tgh=3', dsl=[])
note('Tavern Scoundrel', 'Full', '')


IC.aura('Sticky Fingers', kws=('menace',), status=('Approximate', 'menace and a Treasure on combat damage; the draw on death is ignored'))


@on('Sticky Fingers', 'combat_damage')
def _sticky(g, src, p, a, d, dmg):
    if src.attached is a: make_artifact_tokens(g, src.owner, 'Treasure', 1)


def _obnix_plus(g, p, src):
    for q in g.opps(p):
        if q.hand and q.life > 12: lose_life(g, q, 2, p, kind='drain')
        elif q.hand: discard_worst(g, q, 1)
        else: lose_life(g, q, 2, p, kind='drain')


walker('Ob Nixilis, the Adversary', [
    (1, 'each opponent loses 2 or discards', always(2.5), _obnix_plus),
    (-2, 'draw', always(2.0), lambda g, p, src: draw(g, p, 2)),
], ('Approximate', 'casualty copy not used'))


# ======================================================== GAA and control staples
def _imprintable(c):
    return c.instant and c.cmc <= 2 and 'ctr' not in c.tags


@on('Isochron Scepter', 'etb')
def _scepter_etb(g, src, p, m):
    """imprint: Dramatic Reversal if in hand, else the best removal / card-draw instant with mana value 2 or less"""
    if m is not src: return
    o = src.owner
    cs = [c for c in o.hand if _imprintable(c)]
    if not cs: return
    c = max(cs, key=lambda c: (c.name == 'Dramatic Reversal', 'rem' in c.tags, card_worth(g, o, c)))
    o.hand.remove(c); o.exile.append(c)
    if src.data is None: src.data = {}
    src.data['imprint'] = c.name
    log(f'    Isochron Scepter imprints {c.name}', g)


@on('Isochron Scepter', 'options')
def _scepter_use(g, src, p, s, post):
    """{2}, {T}: cast a copy of the imprinted card (the Reversal loop itself is in the combo framework)"""
    if p is not src.owner or src.tapped or not src.data or not can_pay(g, p, 2, ''): return []
    name = src.data.get('imprint')
    if name is None or name == 'Dramatic Reversal': return []
    c = DB[name]
    if 'rem' in c.tags:
        tg = legal_targets(g, p, c.tags['rem'], c.tags.get('tgt', 'c'), 'mv4' in c.tags, spell=c)
        if not tg: return []
        t = max(tg, key=lambda m: pval(g, m))
        if pval(g, t) < 3: return []
        u, ctx = 2.0 + pval(g, t), {'target': t}
    else:
        u, ctx = 1.5 + card_worth(g, p, c) / 10.0, {}

    def go():
        if src.tapped or not can_pay(g, p, 2, ''): return False
        pay(g, p, 2, ''); src.tapped = True
        log(f'  {NAME(p)} casts a copy of {name} with Isochron Scepter', g)
        n = len(p.gy)
        cast_card(g, p, c, 'lib', dict(ctx))
        if len(p.gy) > n and p.gy[-1] is c: p.gy.pop()             # the copy ceases to exist
        elif c in p.exile[-1:]: p.exile.pop()
        return True
    return [(u, f'Isochron Scepter ({name})', go)]


def _scepter_prio(g, p, c):
    if any(x.name == 'Dramatic Reversal' for x in p.hand): return 62
    if any(x.name == 'Dramatic Reversal' for x in p.library): return 0          # wait for the Reversal
    return 30 if any(_imprintable(x) and 'rem' in x.tags for x in p.hand) else 0


card('Isochron Scepter', '', types='A', dsl=[])
note('Isochron Scepter', 'Full', 'imprints Dramatic Reversal (combo) or the best removal / draw instant, and casts copies; '
     'held until the Reversal is in hand')
card('Dramatic Reversal', '', types='I', dsl=[])
CI.SPELL_PRIO['Isochron Scepter'] = _scepter_prio
CI.SPELL_PRIO['Dramatic Reversal'] = 0


@on('Sanguine Bond', 'gain_life')
def _bond(g, src, p, n):
    if p is src.owner and g.opps(p) and getattr(g, 'bond_depth', 0) < 3:
        g.bond_depth = getattr(g, 'bond_depth', 0) + 1
        try:
            lose_life(g, max(g.opps(p), key=lambda q: threat(g, p, q)), n, p, kind='drain')
        finally:
            g.bond_depth -= 1


@on('Exquisite Blood', 'lose_life')
def _blood(g, src, p, n):
    if p is not src.owner and getattr(g, 'bond_depth', 0) < 3:
        g.bond_depth = getattr(g, 'bond_depth', 0) + 1
        try:
            gain(src.owner, n)
        finally:
            g.bond_depth -= 1
card('Sanguine Bond', '', types='E', dsl=[])
card('Exquisite Blood', '', types='E', dsl=[])
note('Sanguine Bond', 'Full', 'your life gain drains the most threatening opponent; with Exquisite Blood a combo')
note('Exquisite Blood', 'Full', 'opponents\' life loss gains you life; with Sanguine Bond a combo')


note('Temur Sabertooth', 'Partial', 'body only (re-buying ETB creatures is not used)')


# ======================================================== Heliod, Sun-Crowned (replaces GAA IV in Tier 4)
def heliod_update(src):
    """a creature (5/5) only while its controller's devotion to white is 5 or more"""
    import impl_rules2
    if src.data is None: src.data = {}
    src.pow = src.tgh = 5
    src.data['anim'] = impl_rules2.devotion(src.owner, 'W') >= 5


@on('Heliod, Sun-Crowned', 'etb')
def _heliod_etb(g, src, p, m):
    if m is src or m.owner is src.owner: heliod_update(src)


@on('Heliod, Sun-Crowned', 'sba')
def _heliod_sba(g, src, *a):
    heliod_update(src)


@on('Heliod, Sun-Crowned', 'grant_kw')
def _heliod_ind(g, src, m, kw):
    return kw == 'indestructible' and m is src


@on('Heliod, Sun-Crowned', 'gain_life')
def _heliod_gain(g, src, p, n):
    """whenever you gain life: a +1/+1 counter on your best creature (a Walking Ballista first: one more ping)"""
    if p is not src.owner: return
    cre = [m for m in p.perms if m.creature and not m.phased and m is not src]
    if not cre: return
    t = next((m for m in cre if m.cd is not None and m.cd.name == 'Walking Ballista'), None) or \
        max(cre, key=lambda m: pval(g, m))
    t.plus += 1


card('Heliod, Sun-Crowned', 'leg', types='E', dsl=[])
note('Heliod, Sun-Crowned', 'Approximate', 'indestructible; a creature only with devotion to white 5+ (5/5); life gain '
     'puts a +1/+1 counter on your best creature; the {1}{W} lifelink grant is used for the Walking Ballista combo only')
