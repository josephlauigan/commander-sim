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
        elif k == 'request':
            req = ev['request']
            if req.kind == 'continue':
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
