"""The text client: play (for now, watch) a practice game in the terminal."""
import sys


def _line(p):
    tag = ' (you)' if p['you'] else ''
    dead = '' if p['alive'] else '  [out]'
    return (f"  {p['name']}{tag}: {p['life']} life, hand {p['hand_count']}, library {p['library']}, "
            f"graveyard {len(p['graveyard'])}{dead}")


def _grouped(lines):
    """identical lines once, with a count, in first-seen order"""
    seen = {}
    for x in lines: seen[x] = seen.get(x, 0) + 1
    return list(seen.items())


def show_table(view, out=sys.stdout):
    """a compact picture of the table from the human seat"""
    print(f"\n=== Round {view['round']} ===", file=out)
    for p in view['players']:
        print(_line(p), file=out)
        lands = p['lands']
        if lands:
            untapped = sum(1 for L in lands if not L['tapped'])
            print(f"      lands: {len(lands)} ({untapped} untapped)", file=out)
        lines = []
        for m in p['battlefield']:
            bits = [m['name']]
            if 'pt' in m: bits.append(m['pt'])
            if m.get('counters'): bits.append(f"+{m['counters']} counters")
            if m.get('loyalty') is not None: bits.append(f"loyalty {m['loyalty']}")
            if m.get('attached_to'): bits.append(f"on {m['attached_to']}")
            if m['tapped']: bits.append('tapped')
            lines.append(', '.join(bits))
        for text_, n in _grouped(lines):
            print('      ' + text_ + (f'  x{n}' if n > 1 else ''), file=out)
        if p['you'] and 'hand' in p:
            print('      hand: ' + ('; '.join(p['hand']) or '(empty)'), file=out)
            srcs = p.get('mana_sources') or []
            print(f"      mana pool: {p.get('mana_pool', 'empty')}; untapped sources: "
                  + (', '.join(f"{x['name']} ({x['colours']})" for x in srcs) or 'none'), file=out)


HELP = """Commands:
  show              the table again
  tap N [C]         tap mana source N (C: the colour, for sources that make several)
  land N            play card N from your hand as your land drop
  cast N            cast card N from your hand
  cast cmd          cast your commander from the command zone
  use N             an ability of your permanent N (Equip, loyalty abilities, ...)
  pass              pass priority (move on)
When asked to choose (a target, an ability, a blocker), type its number, or cancel / the last choice.
Attacking: type the attackers' numbers (1,3), all, or none.
  quit              end the game"""


def _me(view):
    return next(p for p in view['players'] if p['you'])


def priority_prompt(req, out):
    """what you need in front of you each time you get priority: pool, hand and sources, numbered"""
    me = _me(req.data['view'])
    print(f"\n{req.prompt}. Mana pool: {me['mana_pool']}", file=out)
    for x in req.data.get('stack') or []: print(f"  On the stack: {x['name']}", file=out)
    print('  Hand: ' + ('  '.join(f'{i + 1}) {n}' for i, n in enumerate(me['hand'])) or '(empty)'), file=out)
    srcs = me.get('mana_sources') or []
    print('  Mana sources: ' + ('  '.join(f"{i + 1}) {x['name']} ({x['colours']})" for i, x in enumerate(srcs))
                                or '(none untapped)'), file=out)
    perms = me['battlefield']
    if perms:
        print('  Your permanents: ' + '  '.join(f"{m['i'] + 1}) {m['name']}" + (' (tapped)' if m['tapped'] else '')
                                              for m in perms), file=out)
    if me['commander_in_zone']: print(f"  Commander in the command zone: {me['commander']} (tax {me['tax']})", file=out)


def parse(line, view):
    """a typed command -> an action dict, 'show', 'help', 'quit', or an error string starting with '?'"""
    w = line.strip().split()
    if not w: return '?Type a command (help for the list).'
    cmd = w[0].lower()
    if cmd in ('pass', 'p'): return {'do': 'pass'}
    if cmd in ('show', 's'): return 'show'
    if cmd in ('help', 'h', '?'): return 'help'
    if cmd in ('quit', 'q'): return 'quit'
    if cmd == 'cast' and len(w) > 1 and w[1].lower() in ('cmd', 'commander'): return {'do': 'cast', 'zone': 'cmd'}
    if cmd == 'use':
        if len(w) < 2 or not w[1].isdigit(): return '?Which permanent? e.g. use 1'
        return {'do': 'use', 'perm': int(w[1]) - 1}
    if cmd in ('tap', 'land', 'cast'):
        if len(w) < 2 or not w[1].isdigit(): return f'?Which one? e.g. {cmd} 1'
        n = int(w[1]) - 1
        if cmd == 'tap':
            srcs = _me(view).get('mana_sources') or []
            if not 0 <= n < len(srcs): return '?No such mana source.'
            act = {'do': 'tap', 'source': srcs[n]['id']}
            if len(w) > 2: act['colour'] = w[2].upper()[0]
            return act
        return {'do': cmd, 'card': n}
    return f"?Unknown command {w[0]!r} (help for the list)."


def run(session, out=sys.stdout, inp=input):
    """drive a session from the terminal until the game ends"""
    session.start()
    print(f"Practice game: {session.deck} vs {session.tier}, seed {session.seed}. Seats: {', '.join(session.seats)}",
          file=out)
    while True:
        ev = session.events.get()
        k = ev['kind']
        if k == 'log':
            print(ev['text'], file=out)
        elif k == 'turn':
            if ev['player'] == session.deck: show_table(ev['view'], out)
        elif k in ('invalid', 'auto'):
            print(('  Not allowed: ' if k == 'invalid' else '  (automatic) ') + ev['text'], file=out)
        elif k == 'request':
            req = ev['request']
            if req.kind == 'priority':
                priority_prompt(req, out)
                while True:
                    try:
                        line = inp('> ')
                    except EOFError:
                        line = 'quit'
                    act = parse(line, req.data['view'])
                    if act == 'show': show_table(req.data['view'], out); continue
                    if act == 'help': print(HELP, file=out); continue
                    if act == 'quit': session.close(); break
                    if isinstance(act, str): print('  ' + act[1:], file=out); continue
                    session.answer(act); break
            elif req.kind == 'attack':
                print(f'\n{req.prompt}', file=out)
                for i, ch in enumerate(req.choices): print(f'  {i + 1}) {ch}', file=out)
                while True:
                    try:
                        line = inp('attackers (e.g. 1,3 / all / none)> ').strip().lower()
                    except EOFError:
                        line = 'none'
                    if line in ('none', 'n', ''): session.answer([]); break
                    if line == 'all': session.answer(list(range(len(req.choices)))); break
                    parts = [x for x in line.replace(' ', ',').split(',') if x]
                    if parts and all(x.isdigit() and 1 <= int(x) <= len(req.choices) for x in parts):
                        session.answer([int(x) - 1 for x in parts]); break
                    print('  Type numbers separated by commas, all, or none.', file=out)
            elif req.kind in ('choose', 'target', 'block'):
                print(f'\n{req.prompt}', file=out)
                for i, ch in enumerate(req.choices): print(f'  {i + 1}) {ch}', file=out)
                while True:
                    try:
                        line = inp('> ').strip().lower()
                    except EOFError:
                        line = 'cancel'
                    if line in ('cancel', 'c', 'x'): session.answer('cancel'); break
                    if line.isdigit() and 1 <= int(line) <= len(req.choices):
                        session.answer(int(line) - 1); break
                    print(f'  Type a number from 1 to {len(req.choices)}, or cancel.', file=out)
            elif req.kind == 'continue':
                try:
                    inp(f"[{req.prompt}] Enter to continue, q to quit: ").strip().lower() == 'q' and session.close()
                except EOFError:
                    session.close()
                session.answer(True)
        elif k == 'over':
            if ev['view'] is not None: show_table(ev['view'], out)
            print(f"\nGame over: {ev['winner'] or 'no winner'} ({ev['how']})", file=out)
            return ev
        elif k == 'error':
            print(ev['text'], file=out)
            return ev
