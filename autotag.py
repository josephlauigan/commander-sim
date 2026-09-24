"""Turn a Scryfall card record into simulator tags.

Exact:   types, mana cost, power/toughness, legendary, keywords (flying, deathtouch, ...),
         mana abilities of lands, rocks and dorks.
Parsed:  common templated Oracle effects -- draw, tutor, removal, sweepers, counterspells,
         reanimation, tokens, spell-cast triggers (magecraft), drains, anthems, protection,
         flashback, cost reductions.
Flagged: every Oracle line the parser didn't understand is returned in `unparsed`, so you
         can see exactly what the simulation is ignoring for that card.

Hand-written tags in carddb.py always take priority over these automatic ones.
    python3 autotag.py "Talrand, Sky Summoner" "Grim Tutor"      # preview tags (fetches from Scryfall)
"""
import re, sys

NUM = {'a': 1, 'an': 1, 'one': 1, 'two': 2, 'three': 3, 'four': 4, 'five': 5, 'six': 6, 'seven': 7,
       'eight': 8, 'x': 3, 'that many': 2}
CARD_TYPES = ('Land', 'Creature', 'Artifact', 'Enchantment', 'Instant', 'Sorcery', 'Planeswalker', 'Battle')
TYPE_LETTER = {'Land': 'L', 'Creature': 'C', 'Artifact': 'A', 'Enchantment': 'E', 'Instant': 'I',
               'Sorcery': 'S', 'Planeswalker': 'P'}
KEYWORDS = {'flying': 'fly', 'deathtouch': 'dt', 'vigilance': 'vig', 'lifelink': 'lifelink', 'haste': 'haste',
            'trample': 'trample', 'flash': 'flash', 'reach': 'reach', 'convoke': 'convoke'}
IGNORABLE_KEYWORDS = ('menace', 'first strike', 'double strike', 'hexproof', 'indestructible', 'prowess', 'ward',
                      'defender', 'shroud', 'intimidate', 'changeling', 'cycling', 'kicker', 'equip', 'protection')
IGNORABLE = (r'\bscry\b', r'\bsurveil\b', r'you gain \d+ life', r'enters tapped', r'mill ', r'investigate',
             r'^equip \{', r'crew \d', r'this spell can\'t be countered', r'as .* enters, choose', r'you may pay \{\d\}\.?$')


def num(s):
    s = s.strip().lower()
    return int(s) if s.isdigit() else NUM.get(s, 1)


def conv_cost(mc):
    """'{2}{U}{U/P}{X}' -> '2UU' (X dropped, hybrid/phyrexian take the first colour)"""
    if not mc: return '-'
    gen = 0; pips = ''
    for sym in re.findall(r'\{([^}]+)\}', mc.split(' // ')[0]):
        if sym.isdigit(): gen += int(sym)
        elif sym in ('X', 'Y'): continue
        elif sym == 'C': gen += 1
        else:
            c = sym.split('/')[0]
            if c in 'WUBRG': pips += c
            elif c.isdigit(): gen += int(c)
    return (str(gen) if gen else '') + pips if (gen or pips) else '0'


def front(rec):
    f = rec.get('card_faces')
    if f and rec.get('layout') not in ('split',):
        face = dict(f[0])
        for k in ('mana_cost', 'type_line', 'oracle_text', 'power', 'toughness'):
            if not face.get(k) and rec.get(k): face[k] = rec[k]
        return face, [x.get('oracle_text', '') for x in f[1:]]
    return rec, []


def target_kind(phrase):
    p = phrase.lower()
    if 'nonland permanent' in p: return 'nl'
    if 'artifact, creature, or planeswalker' in p or 'artifact or creature' in p: return 'cap'
    if 'creature or planeswalker' in p: return 'cp'
    if 'creature or enchantment' in p: return 'ce'
    if 'nonartifact creature' in p: return 'cna'
    if 'creature' in p: return 'c'
    if 'permanent' in p: return 'p'
    if 'artifact' in p: return 'a'
    if 'enchantment' in p or 'planeswalker' in p: return 'nl'
    return None


def autotag(rec):
    face, back = front(rec)
    name = rec['name'].split(' // ')[0]
    tl = face.get('type_line', rec.get('type_line', ''))
    main_types = tl.split('—')[0]
    types = ''.join(TYPE_LETTER[t] for t in CARD_TYPES if t in main_types and t in TYPE_LETTER) or 'S'
    if 'C' in types and 'A' in types: types = 'AC'
    cost = conv_cost(face.get('mana_cost') or rec.get('mana_cost', ''))
    tags = {}; unparsed = []; notes = []
    creature = 'C' in types; land = 'L' in types; spell = ('I' in types or 'S' in types)
    if 'Legendary' in tl: tags['leg'] = True
    sub = tl.split('—')[1] if '—' in tl else ''
    if 'Human' in sub: tags['human'] = True
    if 'Warrior' in sub: tags['warrior'] = True
    if 'Shaman' in sub: tags['shaman'] = True
    if 'Wizard' in sub: tags['wizard'] = True
    if creature:
        pw, th = face.get('power', '1'), face.get('toughness', '1')
        pw = int(pw) if str(pw).isdigit() else 1
        th = int(th) if str(th).isdigit() else max(1, pw)
        tags['pow'] = pw
        if th != max(1, pw): tags['tgh'] = th
        if pw >= 5: tags['bomb'] = min(9, pw)
    for kw in rec.get('keywords', []) or []:
        k = KEYWORDS.get(kw.lower())
        if k: tags[k] = True
    text = face.get('oracle_text', rec.get('oracle_text', '')) or ''
    text = re.sub(r'\([^)]*\)', '', text)                       # drop reminder text
    short = name.split(',')[0]
    for nm in (name, short):
        if len(nm) > 3: text = text.replace(nm, '~')
    text = text.replace('this creature', '~').replace('this spell', '~').replace('this artifact', '~') \
               .replace('this enchantment', '~').replace('this land', '~')

    mana_cols = {}

    def add(k, v=True):
        if k not in tags: tags[k] = v

    low = text.lower()
    whole_hit = set()
    if 'destroy all artifacts' in low and 'destroy all enchantments' in low and 'mana value' in low:
        tags['wipe'] = 'austere2'; whole_hit.add('destroy all')
    if 'exile all artifacts' in low and 'exile all creatures' in low:
        tags['wipe'] = 'farewell'; whole_hit.add('exile all')
    for raw in [l.strip() for l in text.split('\n') if l.strip()]:
        L = raw.lower().rstrip('.')
        hit = any(L.lstrip('• ').startswith(w) for w in whole_hit) or L.startswith('choose ')
        # keyword-only lines ("Flying, vigilance")
        parts = [x.strip() for x in L.split(',')]
        if all(x in KEYWORDS or any(x == k or x.startswith(k + ' ') for k in IGNORABLE_KEYWORDS) for x in parts):
            for x in parts:
                k = KEYWORDS.get(x)
                if k: add(k)
            continue
        etb = bool(re.match(r'(when|whenever) ~ enters', L))
        activated = bool(re.match(r'^[^"]*?\{[^}]*\}[^"]*?:', L)) and not spell
        eff = spell or etb
        upkeep = L.startswith('at the beginning of your upkeep') or L.startswith('at the beginning of your draw step')
        castrig = bool(re.match(r'(magecraft — )?whenever you cast (or copy )?(an instant or sorcery|a noncreature|an? (instant|sorcery)) spell', L))
        attack = bool(re.match(r'whenever ~ attacks', L))
        # --- mana abilities
        if re.search(r'creatures you control have "\{t\}: add', L): add('rite'); hit = True
        m = re.search(r'\{t\}(?:, [^:]*)?: add ([^.]*)', L)
        if m and not hit:
            add_s = m.group(1)
            if ('any color' in add_s or 'any type' in add_s) and 'spend this mana only' not in L and 'as long as' not in L: cols = 'A'
            elif 'any color' in add_s or 'as long as' in L: cols = 'C'
            else: cols = ''.join(c for c in 'wubrg' if '{' + c + '}' in add_s).upper() or 'C'
            amt = len(re.findall(r'\{[wubrgc]\}', add_s)) if ' or ' not in add_s else 1
            if land:
                prev = mana_cols.get('c', '')
                mana_cols['c'] = 'A' if 'A' in (prev, cols) else ''.join(sorted(set(prev + cols) - {'C'}, key='WUBRG'.index)) or 'C'
                if amt >= 2: add('amt', amt)
            elif creature: add('dork', cols); add('noatk')
            elif not spell: add('rock', f'{max(1, amt)}:{cols}')
            hit = True
        # --- draw
        m = re.search(r'(?:^|[,.:—] |then |and |you )draw (a|one|two|three|four|five|x|\d+) cards?', L)
        if m and re.search(r'whenever (you|an opponent|a player|equipped)[^,]* draws?|opponent draws|player draws', L): m = None
        if m and not castrig:
            if upkeep: add('eng', 1); hit = True
            elif eff: add('draw', num(m.group(1))); hit = True
            elif activated and not land and 'sacrifice' not in L.split(':')[0]: add('eng', 1); hit = True
        m = re.search(r'draw (a|one|two|three) additional cards?', L)
        if m and upkeep: add('eng', 1); hit = True
        m = re.search(r'put (one|two|three) of them into your hand|you may play (those|that) cards?|draw a card for each', L)
        if m and eff and 'draw' not in tags:
            add('draw', 2 if (m.group(1) in ('two', 'three') or m.group(2) == 'those') else 1); hit = True
        # --- tutors / ramp
        m = re.search(r'search your library for (an?|up to (?:one|two|three))\s(.*?)\s?cards?\b', L)
        if m and land and re.search(r'sacrifice ~', L) and re.search(r'land|forest|plains|island|swamp|mountain', m.group(2)):
            tags['f'] = True; mana_cols['c'] = 'A'; hit = True; m = None
        if m and 'the other into your hand' in L:
            add('lr', 1); add('lrt'); add('lh', 1); hit = True; m = None
        if m:
            what = m.group(2)
            if 'land' in what or 'forest' in what or 'plains' in what or 'island' in what or 'swamp' in what or 'mountain' in what:
                n = 2 if 'up to two' in m.group(1) else 1
                if 'onto the battlefield' in L:
                    add('lr', n)
                    if 'tapped' in L: add('lrt')
                    if 'into your hand' in L: add('lh', 1)
                else: add('lh', 1)
            elif 'into your graveyard' in L: add('fill', 'entomb')
            else:
                kind = ('is' if 'instant or sorcery' in what else 'art' if 'artifact' in what else
                        'ench' if 'enchantment' in what else 'cre' if 'creature' in what else 'any')
                add('tut', kind)
            hit = True
        # --- removal
        m = re.search(r'(destroy|exile) (up to one )?(another )?target ([^.]*?)(\.|$| and | with | you don\'t| an opponent controls| your opponents control)', L)
        if m and eff and 'from a graveyard' not in L and 'card' not in m.group(4) and 'you control' not in m.group(4):
            k = target_kind(m.group(4))
            if k:
                add('rem', 'destroy' if m.group(1) == 'destroy' else 'exile'); add('tgt', k)
                if etb and not spell: add('etb')
                hit = True
        m = re.search(r'return (up to one )?target (spell or permanent|nonland permanent|creature|permanent|artifact or creature)[^.]*? to (its|their) owner\'s hand', L)
        if m and eff and 'overload' not in text.lower() and 'you control' not in m.group(0).replace("you don't control", ''):
            add('rem', 'bounce'); add('tgt', target_kind(m.group(2)) or 'nl')
            if etb and not spell: add('etb')
            hit = True
        m = re.search(r'deals (\d+|x) damage to (any target|target creature or planeswalker|target creature|any other target)', L)
        if m and not castrig and eff:
            d = m.group(1); d = 3 if d == 'x' else int(d)
            add('rem', f'dmg{d}'); add('tgt', 'c')
            if 'any' in m.group(2): add('face')
            if etb and not spell: add('etb')
            hit = True
        if re.search(r'(destroy|exile) target artifact', L) and 'rem' in tags and 'alsoart' not in tags and tags.get('tgt') == 'c':
            add('alsoart'); hit = True
        if re.search(r'owner of target permanent shuffles it into their library', L) and eff:
            add('rem', 'tuck'); add('tgt', 'p'); hit = True
        if re.search(r'return target nonland permanent you don\'t control to its owner\'s hand', L) and 'overload' in text.lower():
            add('wipe', 'rift'); add('rem', 'bounce'); add('tgt', 'nl'); hit = True
        if L.startswith('overload'): hit = True
        if re.search(r'destroy target artifact you don\'t control', L) and 'overload' in low:
            add('wipe', 'vandal'); add('rem', 'destroy'); add('tgt', 'a'); hit = True
        if re.search(r'deals damage equal to its power to each other creature', L): add('wipe', 'nib'); hit = True
        if re.search(r'metalcraft .* exile that creature', L): add('rem', 'exile'); add('tgt', 'c'); add('needart3'); hit = True
        if re.search(r'^tap target creature', L): hit = True
        # --- sweepers
        if re.search(r'destroy all creatures', L): add('wipe', 'destroy'); hit = True
        elif re.search(r'exile all creatures', L): add('wipe', 'exile'); hit = True
        elif re.search(r'all creatures get -(\d+|x)/-(\d+|x)', L): add('wipe', 'minus'); hit = True
        elif re.search(r'return all creatures to their owners\' hands', L): add('wipe', 'evac'); hit = True
        elif re.search(r'return all nonland permanents (you don\'t control|target player controls|your opponents control)', L):
            add('wipe', 'rebuke' if 'target player' in L else 'rift'); hit = True
        m = re.search(r'deals (\d+) damage to each creature', L)
        if m:
            if int(m.group(1)) >= 6: add('wipe', 'dmg13')
            else: notes.append(f'small damage sweep ({m.group(1)}) not modeled')
            hit = True
        if re.search(r'costs \{1\} less to cast for each creature on the battlefield', L): add('perCreature'); hit = True
        # --- counterspells
        m = re.search(r'counter target ([a-z, ]*?)\s?(spell or ability|spell|ability)', L)
        if m and 'that targets a permanent you control' in L:
            add('prot', 'notw'); hit = True; m = None
        if m:
            what = m.group(1)
            if m.group(2) == 'ability' or 'ability' in what: notes.append('ability counter not modeled')
            else:
                scope = ('nc' if 'noncreature' in what else 'cre' if what.strip() == 'creature' else
                         'mv4' if 'mana value 4' in L else 'ise' if what else 'any')
                add('ctr', scope)
                s = re.search(r'unless its controller pays \{(\d+)\}', L)
                if s: add('soft', int(s.group(1)))
            hit = True
        # --- reanimation
        m = re.search(r'(return|put) target (?:nonlegendary )?creature card from (a|your) graveyard (to|onto) the battlefield', L)
        if m and (spell or 'aura' in tl.lower() or 'Enchantment' in types):
            kind = 'animate' if m.group(2) == 'a' else 'evil'
            if 'lose life equal to its mana value' in L: kind = 'reanimate'
            add('rean', kind); hit = True
        if re.search(r'enchant creature card in a graveyard', L): add('rean', 'animate'); hit = True
        if re.search(r'return enchanted creature card to the battlefield', L): hit = True
        # --- tokens
        m = re.search(r'create (a|an|one|two|three|four|five|x|that many|\d+) (\d+|x)/(\d+|x) ([^.]*?)creature tokens?', L)
        if m:
            n = num(m.group(1)); p_ = 1 if m.group(2) == 'x' else int(m.group(2))
            fly = 'flying' in L; dt = 'deathtouch' in L
            if m.group(1) == 'x' and spell: add('tokx')
            elif castrig: add('spelltok', p_); fly and add('spelltokfly')
            elif upkeep: add('tokup', n)
            elif attack: add('tokatk', n)
            elif spell: add('mktok', f'{n}:{p_}:{int(fly)}')
            else: add('tok', n); add('tokp', p_); fly and add('tokfly'); dt and add('tokdt')
            hit = True
        m = re.search(r'create (a|two|three|x) treasure tokens?', L)
        if m and not castrig:
            add('treas', num(m.group(1))); hit = True
        # --- spell-cast triggers (magecraft / "whenever you cast a noncreature spell")
        if castrig:
            m = re.search(r'deals? (\d+) damage to (each opponent|target opponent|any target|each of up to)', L)
            if m:
                add('ping', int(m.group(1)))
                if 'each opponent' not in m.group(2): add('ral')
            if re.search(r'draw a card', L): add('spelldraw')
            if re.search(r'create a treasure', L): add('kiln')
            if re.search(r'create (a|an) (\d+)/(\d+)', L) and 'spelltok' not in tags:
                mm = re.search(r'create (a|an) (\d+)/(\d+)', L); add('spelltok', int(mm.group(2)))
            if re.search(r'gets? \+\d/\+\d until end of turn', L): pass
            if creature and any(k in tags for k in ('ping', 'spelldraw', 'spelltok', 'kiln')): add('noatk')
            hit = True
        # --- drains, death triggers
        m = re.search(r'each opponent loses (\d+|x) life', L)
        if m and (etb or spell):
            add('drainetb', num(m.group(1))); hit = True
        if re.search(r'whenever (~ or )?another creature (you control )?dies, each opponent loses 1 life', L): add('drain'); hit = True
        if re.search(r'whenever (~ or )?another creature dies, target (player|opponent) loses 1 life', L): add('bartist'); hit = True
        if re.search(r'each opponent sacrifices a creature', L) and (etb or spell): add('edictetb'); hit = True
        # --- anthems and pumps
        m = re.search(r'(other )?creatures you control get \+(\d+)/\+(\d+)(?! until end of turn)', L)
        if m and 'until end of turn' not in L: add('anth', int(m.group(2))); hit = True
        elif m: add('pumpall', int(m.group(2))); hit = True
        # --- protection
        if re.search(r'gain hexproof and indestructible until end of turn', L): add('prot', 'hi'); hit = True
        elif re.search(r'creatures you control gain indestructible until end of turn', L): add('prot', 'indes'); hit = True
        elif re.search(r'phases? out', L) and spell: add('prot', 'phase'); hit = True
        if re.search(r'equipped creature has (hexproof|shroud)', L): add('prot', 'boots'); hit = True
        if re.search(r'^sacrifice (a|another) creature:', L): add('sac'); hit = True
        m = re.search(r'flashback (\{[^ ]+\})', L)
        if m: add('fb', conv_cost(m.group(1).upper())); hit = True
        if not hit and any(re.search(p, L) for p in IGNORABLE): hit = True
        if not hit and re.match(r'(equipped|enchanted) creature gets \+\d/\+\d', L): hit = True
        if not hit: unparsed.append(raw)
    if land:
        basic = ''.join(c for t_, c in (('Plains', 'W'), ('Island', 'U'), ('Swamp', 'B'), ('Mountain', 'R'), ('Forest', 'G')) if t_ in sub)
        pm = ''.join(c for c in 'WUBRG' if c in (rec.get('produced_mana') or []))
        cols = mana_cols.get('c', '')
        if cols != 'A':
            cols = ''.join(c for c in 'WUBRG' if c in cols + basic + pm) or ('C' if cols in ('', 'C') else cols)
        tags['c'] = 'A' if cols == 'A' or len(cols) >= 5 else cols
        tl_low = text.lower()
        if 'enters tapped unless' in tl_low or 'unless you control' in tl_low: tags['ck'] = True
        elif 'enters tapped' in tl_low: tags['t'] = True
        if re.search(r'sacrifice ~: search your library for a basic land', tl_low): tags['f'] = True; tags['c'] = 'A'
        unparsed = [u for u in unparsed if not re.search(r'enters tapped|\{t\}: add|pay 2 life', u.lower())]
    for b in back:
        if b: unparsed.append('[back face] ' + b.replace('\n', ' / '))
    tagstr = ' '.join(k if v is True else f'{k}={v}' for k, v in tags.items())
    return {'types': types, 'cost': cost, 'tags': tagstr, 'unparsed': unparsed, 'notes': notes,
            'game_changer': bool(rec.get('game_changer'))}


if __name__ == '__main__':
    import scryfall
    for n, rec in scryfall.fetch(sys.argv[1:]).items():
        r = autotag(rec)
        print(f"{n}|{r['types']}|{r['cost']}|{r['tags']}")
        for u in r['unparsed']: print('    not modeled:', u)
        for u in r['notes']: print('    note:', u)
