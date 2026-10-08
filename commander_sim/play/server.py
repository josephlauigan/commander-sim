"""The browser table's server: standard library only (http.server), one game at a time, on 127.0.0.1 (with --lan, on
the local network too, for a second player on another computer).

Seats: each browser has a cookie (practice_seat) naming it; a game maps cookies to the seats people play. The
computer running the server is the host: it starts, loads, saves and ends games, and a page on it without a seat
plays the first seat. With two players the host opens the table ('two': true), which waits for the second person to
join with its code from their own computer; each person then sees the table from their own seat, and Undo or Try it
asks the other person first.

Routes:
  GET  /                    the page (static/index.html); GET /static/<file> for its files
  GET  /api/options         your decks and the tiers (with each tier's decks), for the setup screen
  GET  /api/state           the game now: its settings, your seat's latest view, the decision waiting for you, and
                            a two-player game waiting for its second person (the lobby)
  GET  /api/events          server-sent events: every event of the game in order, each with an id (a reconnect
                            with Last-Event-ID, or ?since=N, resumes after it)
  GET  /images/<file>       a card image from the cache (data/images/)
  POST /api/new             start a game: {deck, tier, seed?, seat?, opponents?, profile?, ai?, tools?, images?, two?}:
                            tools are the practice switches {hint, undo, compare}; the card images load first
                            ('loading' events with done/total, then 'images': name -> files). two: a two-player
                            game (two opponents): returns the join code and waits for the second person
  POST /api/join            the second person: {code, deck} (a deck other than the host's)
  POST /api/answer          answer the waiting decision: {id, answer}; a stale id is refused (409)
  POST /api/undo            take back your last answer ({n}: the last n): the game replays from its seed (two
                            players: {asked: name} and the other person is asked first)
  POST /api/approve         the other person's answer to an Undo or Try it: {id, yes}
  GET  /api/review          after the game: a summary, and each decision with what the AI would have done and both scores
  POST /api/save            save the game (data/saves/<time>-<deck>-<tier>-<seed>.json); GET /api/saves lists them
  POST /api/load            {name}: replay a saved game to where it was saved
  POST /api/tryit           {n}: back to decision n with the AI's choice there (the review's "Try it")
  POST /api/hint            what the AI would do at the decision waiting for you: {text, detail, choice}
  POST /api/quit            end the game

The engine runs on the session's worker thread. A pump thread moves the session's events into the hub's history
(JSON-ready, numbered) and wakes the event streams. Views are built on the engine thread, never here. The history
keeps each event's routing (a seat's own requests and secrets, each seat's view); a stream sends a seat only its own
(`for_seat`), and another person's request as {'kind': 'waiting', 'who': name}.
"""
import json
import os
import queue
import secrets
import socket
import threading
import time
import webbrowser
from http.server import ThreadingHTTPServer, BaseHTTPRequestHandler
from urllib.parse import urlparse, parse_qs

from commander_sim import DATA
from commander_sim.play.controller import Request
from commander_sim.play.session import Session, MY_DECKS, TIERS

STATIC = os.path.join(os.path.dirname(__file__), 'static')
IMAGES = os.path.join(DATA, 'images')
SAVES = os.path.join(DATA, 'saves')
TYPES = {'.html': 'text/html; charset=utf-8', '.js': 'text/javascript; charset=utf-8', '.css': 'text/css; charset=utf-8',
         '.gif': 'image/gif', '.png': 'image/png', '.jpg': 'image/jpeg', '.svg': 'image/svg+xml', '.json': 'application/json'}


def jsonable(x):
    """an event or request as plain JSON data (anything unexpected becomes its text)"""
    return json.loads(json.dumps(x, default=lambda o: o.__dict__ if isinstance(o, Request) else str(o)))


class Hub:
    """the current game and its event history, shared by the request handlers"""
    PROPOSAL_WAIT = 60.0             # an Undo or Try it the other person hasn't answered: a new one may replace it

    def __init__(s, lan=False):
        s.lan = lan                  # listening on the local network (a second player can join)
        s.cond = threading.Condition()
        s.session = None
        s.events = []                # [{'id': n, ...}] for the current game, with their routing (see for_seat)
        s.pending = {}               # seat -> the request waiting for its answer: its event
        s.views = {}                 # seat -> the latest view of the table from that seat
        s.images = {}                # card name -> image files, for this game
        s.tools = {}
        s.tokens = {}                # browser cookie -> the seat it plays, in this game
        s.lobby = None               # a two-player game waiting for its second person (see open_lobby)
        s.proposal = None            # an Undo or Try it waiting for the other person's OK
        s.next_proposal = 1
        s.next_id = 1
        s.game_no = 0

    # ------------------------------------------------------------------ games
    def new_game(s, opts, sess=None, tokens=None):
        s.quit()
        if sess is None:
            sess = Session(opts['deck'], opts['tier'], seed=opts.get('seed'), seat=opts.get('seat'),
                           opponents=opts.get('opponents'), profile=opts.get('profile', 'loose'),
                           ai=opts.get('ai', 'lookahead'), views=True,
                           compare=bool((opts.get('tools') or {}).get('compare', True)), seats=opts.get('seats'),
                           partner=opts.get('partner'), autopass=opts.get('autopass') or 'respond')
        with s.cond:
            s.game_no += 1
            s.session, s.events, s.pending, s.views, s.images = sess, [], {}, {}, {}
            s.tools = {k: bool((opts.get('tools') or {}).get(k, True)) for k in ('hint', 'undo', 'compare')}
            s.tokens = dict(tokens or {})
            s.proposal = None
            no = s.game_no
        threading.Thread(target=s._pump, args=(sess, no), name='practice-pump', daemon=True).start()
        threading.Thread(target=s._load, args=(sess, no, opts.get('images', True)), name='practice-images',
                         daemon=True).start()
        return sess

    def _load(s, sess, no, want_images):
        """the loading screen: card images for this game's decks, then the game starts"""
        found = {}
        if want_images:
            from commander_sim import poolmode
            from commander_sim.play import images
            names = images.game_names([poolmode.seat_spec(k) for k in sess.seats])
            try:
                found = images.prepare(names, lambda d, t: sess.events.put({'kind': 'loading', 'done': d, 'total': t}))
            except Exception as e:                      # images are a nicety: the game starts without them
                found = images.cached(names)            # offline: the ones already on disk
                sess.events.put({'kind': 'log', 'text': f'(card images unavailable: {e})'})
        sess.events.put({'kind': 'images', 'images': found})
        with s.cond:
            if s.game_no != no: return                  # replaced or quit while loading
        sess.start()

    def quit(s):
        with s.cond:
            sess, s.session = s.session, None
            s.pending, s.lobby, s.proposal = {}, None, None
            s.cond.notify_all()
        if sess is not None:
            sess.close()
            sess.join(30)          # its engine thread may be mid-playout: one game at a time touches the engine

    # ------------------------------------------------------------------ two players
    def open_lobby(s, opts, host, sess=None):
        """a two-player game waiting for the second person: opts as for new_game (or a loaded game, sess), host the
        host's cookie. Returns the join code"""
        if sess is None:              # the settings are checked now, not when the second person arrives
            Session(opts['deck'], opts['tier'], seed=opts.get('seed') or 1, opponents=opts.get('opponents'),
                    profile=opts.get('profile', 'loose'), ai=opts.get('ai', 'lookahead'),
                    partner=next(k for k in MY_DECKS if k != opts['deck']))
        s.quit()
        code = f'{secrets.randbelow(10 ** 6):06d}'
        with s.cond:
            s.lobby = {'opts': opts, 'code': code, 'host': host, 'session': sess,
                       'deck': sess.deck if sess else opts['deck'], 'partner': sess.partner if sess else None}
            s.cond.notify_all()
        return code

    def join(s, token, code, deck):
        """the second person joins the waiting game with its code; None, or why not"""
        with s.cond:
            lb = s.lobby
            if lb is None: return 'There is no game waiting for a second player.'
            if str(code or '').strip() != lb['code']: return 'That code is not right: the host\'s screen shows it.'
            if lb['partner'] and deck != lb['partner']: return f'This saved game goes on with the {lb["partner"]} deck.'
            if deck not in MY_DECKS: return f'deck: one of {", ".join(MY_DECKS)}'
            if deck == lb['deck']: return 'The host plays that deck: pick another.'
            if token == lb['host']: return 'This browser is the host\'s: join from the other computer.'
            s.lobby = None
        tokens = {lb['host']: lb['deck'], token: deck}
        try:
            if lb['session'] is None: s.new_game(dict(lb['opts'], partner=deck), tokens=tokens)
            else: s.new_game({'tools': lb['opts'].get('tools'), 'images': lb['opts'].get('images', True)},
                             lb['session'], tokens=tokens)
        except (ValueError, TypeError) as e:
            return str(e)
        return None

    def lobby_info(s, local):
        """the waiting game as a page sees it: the host also gets the code and the address to give"""
        lb = s.lobby
        if lb is None: return None
        o = lb['opts']
        d = {'deck': lb['deck'], 'partner': lb['partner'], 'tier': o.get('tier') or (lb['session'] and lb['session'].tier),
             'loaded': lb['session'] is not None}
        if local: d.update(code=lb['code'], addresses=lan_addresses() if s.lan else [])
        return d

    def seat_of(s, token, local):
        """the seat a browser plays in the current game (a page on the host computer without one plays the first
        seat), or None. A game the host has ended keeps its seats until the next starts (its last events still
        reach both people)"""
        with s.cond:
            if token in s.tokens: return s.tokens[token]
            return s.session.deck if local and s.session is not None else None

    def _other(s, sess, seat):
        return next((k for k in sess.humans if k != seat), None)

    def request_rewind(s, seat, what, n):
        """Undo (what 'undo': seat's last n answers) or Try it ('tryit': seat's decision n). One person: done now.
        Two: the other person is asked first. Returns (why not, None), or (None, the name asked) or (None, None)"""
        with s.cond:
            sess = s.session
            if sess is None: return 'There is no game running.', None
            if what == 'undo' and not s.tools.get('undo', True): return 'Undo is switched off for this game.', None
            other = s._other(sess, seat)
            if other is not None:
                if what == 'undo':
                    to = sess.undo_point(n, seat)
                    if to is None: return 'Nothing to undo yet.', None
                    theirs = sess.answer_seats[to:].count(other)
                    what_text = 'to take back their last action' if n == 1 else f'to take back their last {n} actions'
                    also = f' That also takes back your last {theirs} answer{"s" if theirs != 1 else ""}.' if theirs else ''
                else:
                    if sess.try_entry(n, seat) is None: return 'That decision has no AI choice to try.', None
                    what_text = "to try the AI's choice at one of their decisions"
                    also = ' The game goes back to that decision for both of you.'
                p = s.proposal
                if p is not None and time.time() - p['at'] < s.PROPOSAL_WAIT:
                    return f'Still waiting for {sess.names.get(p["to"], p["to"])} to answer the last request.', None
                if p is not None: s._push({'kind': 'proposal_done', 'id': p['id']})
                name = sess.names.get(seat, seat)
                s.proposal = {'id': s.next_proposal, 'by': seat, 'to': other, 'what': what, 'n': n, 'at': time.time(),
                              'text': f'{name} asks {what_text}.{also}'}
                s.next_proposal += 1
                s._push({'kind': 'proposal', 'seat': other, 'proposal': {k: s.proposal[k] for k in ('id', 'text')}})
                return None, sess.names.get(other, other)
        return (sess.undo(n, seat) if what == 'undo' else sess.try_it(n, seat)), None

    def approve(s, seat, pid, yes):
        """the other person's answer to an Undo or Try it; None, or why not"""
        with s.cond:
            p, sess = s.proposal, s.session
            if sess is None or p is None or p['id'] != pid or p['to'] != seat: return 'That request is no longer open.'
            s.proposal = None
            s._push({'kind': 'proposal_done', 'id': pid})
            if not yes:
                s._push({'kind': 'declined', 'seat': p['by'],
                         'text': f'{sess.names.get(seat, seat)} said no to the {"Undo" if p["what"] == "undo" else "Try it"}.'})
                return None
        why = sess.undo(p['n'], p['by']) if p['what'] == 'undo' else sess.try_it(p['n'], p['by'])
        if why:
            with s.cond: s._push({'kind': 'declined', 'seat': p['by'], 'text': why})
        return why

    # ------------------------------------------------------------------ events
    def _pump(s, sess, no):
        """the session's events into the history, until a newer game replaces it (a game that ended can go on:
        the review's "Try it" restarts it)"""
        while True:
            try:
                ev = sess.events.get(timeout=1.0)
            except queue.Empty:
                if s.game_no != no: return
                continue
            with s.cond:
                if s.game_no != no: return                     # a newer game replaced this one
                s._add(ev, sess)

    def _add(s, ev, sess):
        """(holding the lock) record one session event"""
        if ev['kind'] == 'reset':                       # Undo: the game restarts; what came before is replayed
            s.events, s.pending, s.views, s.proposal = [], {}, {}, None
        views = ev.get('views')
        if views is None and ev.get('view') is not None: views = {sess.deck: ev['view']}
        if ev['kind'] == 'request':
            req = ev['request']
            seat = ev.get('seat') or sess.deck
            out = {'kind': 'request', 'request': {'kind': req.kind, 'prompt': req.prompt, 'choices': req.choices,
                                                  'data': {k: v for k, v in req.data.items() if k != 'view'}},
                   'seat': seat, 'who': sess.names.get(seat, seat)}
            if 'view' in req.data: views = {seat: req.data['view']}
        else:
            out = {k: v for k, v in ev.items() if k not in ('view', 'views')}
            if ev['kind'] == 'images': s.images = ev['images']
        if views:
            s.views.update({k: v for k, v in views.items() if v is not None})
            out['views'] = {k: s.views.get(k) for k in views}
        out = jsonable(out)
        s._push(out)
        if out['kind'] == 'request': s.pending[out['seat']] = out

    def _push(s, out):
        """(holding the lock) one event into the history, numbered"""
        out['id'] = s.next_id; s.next_id += 1
        s.events.append(out)
        s.cond.notify_all()

    def answer(s, seat, rid, value):
        """None, or why the answer can't be taken"""
        with s.cond:
            if s.session is None: return 'There is no game running.'
            p = s.pending.get(seat)
            if p is None or p['id'] != rid: return 'That decision has already been answered.'
            del s.pending[seat]
            sess = s.session
            s._push({'kind': 'decided', 'by': seat, 'who': sess.names.get(seat, seat)})   # answered: no longer waiting
        sess.answer(value, seat)
        return None

    def undo(s, n=1, seat=None):
        """None, or why it can't (one person; see request_rewind)"""
        with s.cond:
            sess = s.session
        if sess is None: return 'There is no game running.'
        return s.request_rewind(seat or sess.deck, 'undo', n)[0]

    # ------------------------------------------------------------------ saved games (data/saves/)
    def save(s):
        """None and the file's name, or why not"""
        with s.cond:
            sess = s.session
        if sess is None: return 'There is no game to save.', None
        if sess.current is None and not sess.finished:
            return 'Save when the game is waiting on a decision (not while the AI players play).', None
        with sess.hint_lock:
            data = sess.saved()
        data['tools'] = s.tools
        data['saved_at'] = time.strftime('%Y-%m-%d %H:%M')
        os.makedirs(SAVES, exist_ok=True)
        name = f"{time.strftime('%Y%m%d-%H%M%S')}-{sess.deck}{'+' + sess.partner if sess.partner else ''}-{sess.tier}-{sess.seed}.json"
        with open(os.path.join(SAVES, name), 'w', encoding='utf-8') as f: json.dump(data, f)
        return None, name

    def saves(s):
        out = []
        if os.path.isdir(SAVES):
            for name in sorted(os.listdir(SAVES), reverse=True):
                if not name.endswith('.json'): continue
                try:
                    with open(os.path.join(SAVES, name), encoding='utf-8') as f: d = json.load(f)
                except (OSError, ValueError):
                    continue
                out.append({'name': name, 'deck': d.get('deck'), 'partner': d.get('partner'), 'tier': d.get('tier'),
                            'seed': d.get('seed'), 'round': d.get('round'), 'finished': d.get('finished'),
                            'saved_at': d.get('saved_at')})
        return out

    def load(s, name, images=True, host=None):
        """(None, None), (None, join code) for a two-player game (it waits for the second person), or (why not, None):
        the saved game replays to where it was saved"""
        path = os.path.normpath(os.path.join(SAVES, str(name)))
        if not path.startswith(SAVES + os.sep) or not os.path.isfile(path): return 'No such saved game.', None
        try:
            with open(path, encoding='utf-8') as f: data = json.load(f)
            tools = data.get('tools') or {}
            sess = Session.load(data, views=True, compare=bool(tools.get('compare', True)))
        except (OSError, ValueError, KeyError) as e:
            return f"That saved game can't be loaded: {e}", None
        if sess.partner:
            if not s.lan: return 'A two-player game: start the server with --lan to play it with your friend.', None
            return None, s.open_lobby({'tools': tools, 'images': images, 'tier': sess.tier}, host, sess)
        s.new_game({'tools': tools, 'images': images}, sess)
        return None, None

    def review(s, seat):
        with s.cond:
            sess = s.session
        if sess is None: return 'There is no game.'
        if seat is None: return 'You have no seat in this game.'
        return sess.review(seat)

    def set_autopass(s, seat, mode):
        """when this seat's person gets priority, from their next decision on (play.human.autopass)"""
        with s.cond:
            sess = s.session
            if sess is None: return 'There is no game running.'
            if seat is None: return 'You have no seat in this game.'
            return sess.set_autopass(seat, mode)

    def hint(s, seat):
        with s.cond:
            sess = s.session
            if sess is None: return 'There is no game running.'
            if seat is None: return 'You have no seat in this game.'
            if not s.tools.get('hint', True): return 'Hint is switched off for this game.'
        return sess.hint(seat)

    def state(s, seat=None, local=True):
        with s.cond:
            sess = s.session
            game = None
            if sess is not None and seat is not None:
                game = {'deck': sess.deck, 'tier': sess.tier, 'seed': sess.seed, 'seats': sess.seats,
                        'profile': sess.profile, 'ai': sess.ai, 'tools': s.tools, 'you': seat, 'humans': sess.humans,
                        'names': dict(sess.names), 'autopass': sess.autopass_of(seat, at=10 ** 9)}
            p = s.proposal
            return {'game': game, 'busy': sess is not None and seat is None, 'host': local, 'lan': s.lan,
                    'lobby': s.lobby_info(local), 'view': s.views.get(seat), 'pending': s.pending.get(seat),
                    'images': s.images, 'last_event': s.events[-1]['id'] if s.events else 0,
                    'proposal': {k: p[k] for k in ('id', 'text')} if p and p['to'] == seat else None}

    def events_after(s, since, timeout=15.0):
        """events with id > since (waits up to timeout for one); [] on timeout"""
        with s.cond:
            if not any(e['id'] > since for e in s.events[-1:]):
                s.cond.wait(timeout)
            return [e for e in s.events if e['id'] > since]


def for_seat(ev, seat):
    """a history event as `seat` sees it, or None: another seat's request is only who is deciding, another seat's
    messages and secrets are left out, and each event carries this seat's view of the table"""
    if 'seat' in ev and ev['seat'] != seat:
        return {'id': ev['id'], 'kind': 'waiting', 'who': ev['who']} if ev['kind'] == 'request' else None
    if ev.get('hide') is not None and ev['hide'] == seat: return None
    out = {k: v for k, v in ev.items() if k not in ('seat', 'hide', 'views', 'who')}
    if 'views' in ev and ev['views'].get(seat) is not None: out['view'] = ev['views'][seat]
    return out


def lan_addresses():
    """this computer's addresses on the local network (a best guess: the one with the route out first)"""
    out = []
    try:
        with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as u:
            u.connect(('10.255.255.255', 1))                # no packet is sent: this only picks the interface
            out.append(u.getsockname()[0])
    except OSError:
        pass
    try:
        for ip in socket.gethostbyname_ex(socket.gethostname())[2]:
            if not ip.startswith('127.') and ip not in out: out.append(ip)
    except OSError:
        pass
    return [ip for ip in out if not ip.startswith('127.')]


_CATALOG = []


def catch_up(evs, pending):
    """a page catching up on many events at once (a reload, a reconnect): every action carries the table it left,
    but only the last table matters, so the others are left out (a long game's history is megabytes otherwise)"""
    last = max((i for i, e in enumerate(evs) if 'view' in e), default=None)
    keep = {last} | {i for i, e in enumerate(evs) if pending is not None and e['id'] == pending['id']}
    return [e if i in keep or 'view' not in e else {k: v for k, v in e.items() if k != 'view'} for i, e in enumerate(evs)]


def options():
    """the setup screen's data: your decks, the tiers, and the commanders' images already cached"""
    from commander_sim import poolmode
    from commander_sim.play import catalog, images
    if not _CATALOG:
        poolmode._setup('loose', 'adaptive', 1.0)
        _CATALOG.append(catalog.catalog())
    c = _CATALOG[0]
    names = [d['commander'] for d in c['decks']] + [x['commander'] for t in c['tiers'] for x in t['decks']]
    return dict(c, images=images.cached(names))


def fetch_commanders():
    """at start-up, in the background: the commanders' images for the setup screen (once; cached after)"""
    from commander_sim.play import catalog, images
    try:
        images.prepare(catalog.commanders())
    except Exception:
        pass


class Handler(BaseHTTPRequestHandler):
    hub = None                                         # set by make_server
    protocol_version = 'HTTP/1.1'
    COOKIE = 'practice_seat'
    HOST_ONLY = ('/api/new', '/api/load', '/api/quit', '/api/save', '/api/saves')

    def log_message(s, fmt, *args):                    # quiet: the terminal is for the game
        pass

    # ------------------------------------------------------------------ helpers
    def local(s):
        """the request comes from the computer running the server (the host)"""
        return s.client_address[0] in ('127.0.0.1', '::1', '::ffff:127.0.0.1') or s.client_address[0].startswith('127.')

    def token(s):
        """this browser's cookie (a new one is set with the response if it has none)"""
        if getattr(s, '_token', None) is None:
            for part in (s.headers.get('Cookie') or '').split(';'):
                k, _, v = part.strip().partition('=')
                if k == s.COOKIE and v: s._token = v
            if getattr(s, '_token', None) is None:
                s._token, s._new_token = secrets.token_urlsafe(16), True
        return s._token

    def seat(s):
        return s.hub.seat_of(s.token(), s.local())

    def _send(s, code, body, ctype='application/json'):
        data = body if isinstance(body, bytes) else (json.dumps(body) if ctype == 'application/json' else body).encode()
        s.send_response(code)
        s.send_header('Content-Type', ctype)
        s.send_header('Content-Length', str(len(data)))
        s.send_header('Cache-Control', 'no-store')
        if getattr(s, '_new_token', False):
            s.send_header('Set-Cookie', f'{s.COOKIE}={s._token}; Path=/; HttpOnly; SameSite=Strict; Max-Age=31536000')
            s._new_token = False
        s.end_headers()
        s.wfile.write(data)

    def _reply(s, why, ok=None):
        """409 with why, or 200 with ok"""
        return s._send(409, {'error': why}) if why else s._send(200, dict({'ok': True}, **(ok or {})))

    def _body(s):
        n = int(s.headers.get('Content-Length') or 0)
        if not n: return {}
        try:
            return json.loads(s.rfile.read(n))
        except ValueError:
            return None

    # ------------------------------------------------------------------ routes
    def do_GET(s):
        s._token = s._new_token = None
        url = urlparse(s.path)
        if url.path in ('/', '/index.html'): s.token(); return s._static('index.html')
        if url.path.startswith('/static/'): return s._static(url.path[len('/static/'):])
        if url.path.startswith('/images/'): return s._static(url.path[len('/images/'):], root=IMAGES)
        if url.path in s.HOST_ONLY and not s.local(): return s._send(403, {'error': 'Only the host computer can do that.'})
        if url.path == '/api/state': return s._send(200, s.hub.state(s.seat(), s.local()))
        if url.path == '/api/options': return s._send(200, dict(options(), lan=s.hub.lan, host=s.local()))
        if url.path == '/api/saves': return s._send(200, {'saves': s.hub.saves()})
        if url.path == '/api/review':
            res = s.hub.review(s.seat())
            return s._send(409, {'error': res}) if isinstance(res, str) else s._send(200, res)
        if url.path == '/api/events':
            q = parse_qs(url.query)
            since = s.headers.get('Last-Event-ID') or (q.get('since') or ['0'])[0]
            return s._stream(int(since) if str(since).isdigit() else 0)
        s._send(404, {'error': 'not found'})

    def do_POST(s):
        s._token = s._new_token = None
        url = urlparse(s.path)
        body = s._body()
        if body is None: return s._send(400, {'error': 'the request body is not JSON'})
        if url.path in s.HOST_ONLY and not s.local(): return s._send(403, {'error': 'Only the host computer can do that.'})
        if url.path == '/api/new':
            if body.get('deck') not in MY_DECKS: return s._send(400, {'error': f'deck: one of {", ".join(MY_DECKS)}'})
            if body.get('tier') not in TIERS: return s._send(400, {'error': f'tier: one of {", ".join(TIERS)}'})
            if body.get('opponents') is not None and not (isinstance(body['opponents'], list)
                                                          and all(isinstance(x, str) for x in body['opponents'])):
                return s._send(400, {'error': 'opponents: a list of deck keys'})
            if body.get('seats') is not None and not (isinstance(body['seats'], list) and all(isinstance(x, str) for x in body['seats'])):
                return s._send(400, {'error': 'seats: four deck keys in turn order'})
            if body.get('seat') is not None and not isinstance(body['seat'], int):
                return s._send(400, {'error': 'seat: 1 to 4'})
            from commander_sim.play.human import AUTOPASS
            if body.get('autopass') is not None and body['autopass'] not in AUTOPASS:
                return s._send(400, {'error': f'autopass: one of {", ".join(AUTOPASS)}'})
            if body.get('two'):
                if not s.hub.lan:
                    return s._send(409, {'error': 'To play with a friend, start the server with --lan: '
                                                  'python3 -m commander_sim.play --lan'})
                try:
                    code = s.hub.open_lobby({k: v for k, v in body.items() if k not in ('two', 'seat', 'seats')}, s.token())
                except (ValueError, TypeError) as e:
                    return s._send(400, {'error': str(e)})
                return s._send(200, {'ok': True, 'code': code, 'addresses': lan_addresses()})
            try:
                sess = s.hub.new_game(body, tokens={s.token(): body['deck']})
            except (ValueError, TypeError) as e:
                return s._send(400, {'error': str(e)})
            return s._send(200, {'ok': True, 'seed': sess.seed, 'seats': sess.seats})
        if url.path == '/api/join':
            return s._reply(s.hub.join(s.token(), body.get('code'), body.get('deck')))
        if url.path == '/api/answer':
            return s._reply(s.hub.answer(s.seat(), body.get('id'), body.get('answer')))
        if url.path in ('/api/undo', '/api/tryit'):
            seat = s.seat()
            if seat is None: return s._reply('You have no seat in this game.')
            if url.path == '/api/undo': why, asked = s.hub.request_rewind(seat, 'undo', int(body.get('n', 1)) if str(body.get('n', 1)).isdigit() else 1)
            else: why, asked = s.hub.request_rewind(seat, 'tryit', body.get('n'))
            return s._reply(why, {'asked': asked} if asked else None)
        if url.path == '/api/approve':
            return s._reply(s.hub.approve(s.seat(), body.get('id'), bool(body.get('yes'))))
        if url.path == '/api/save':
            why, name = s.hub.save()
            return s._reply(why, {'name': name})
        if url.path == '/api/load':
            why, code = s.hub.load(body.get('name'), body.get('images', True) is not False, s.token())
            return s._reply(why, {'code': code, 'addresses': lan_addresses()} if code else None)
        if url.path == '/api/autopass':
            return s._reply(s.hub.set_autopass(s.seat(), body.get('mode')))
        if url.path == '/api/hint':
            res = s.hub.hint(s.seat())
            return s._send(409, {'error': res}) if isinstance(res, str) else s._send(200, res)
        if url.path == '/api/quit':
            s.hub.quit(); return s._send(200, {'ok': True})
        s._send(404, {'error': 'not found'})

    def _static(s, name, root=STATIC):
        path = os.path.normpath(os.path.join(root, name))
        if not path.startswith(root + os.sep) or not os.path.isfile(path): return s._send(404, {'error': 'not found'})
        with open(path, 'rb') as f: data = f.read()
        s._send(200, data, TYPES.get(os.path.splitext(path)[1], 'application/octet-stream'))

    def _stream(s, since):
        token, local = s.token(), s.local()
        s.send_response(200)
        s.send_header('Content-Type', 'text/event-stream')
        s.send_header('Cache-Control', 'no-store')
        s.send_header('Connection', 'close')
        if getattr(s, '_new_token', False):
            s.send_header('Set-Cookie', f'{s.COOKIE}={s._token}; Path=/; HttpOnly; SameSite=Strict; Max-Age=31536000')
        s.end_headers()
        s.close_connection = True
        try:
            while True:
                evs = s.hub.events_after(since)
                if not evs:
                    s.wfile.write(b': keep-alive\n\n'); s.wfile.flush(); continue
                since = evs[-1]['id']
                seat = s.hub.seat_of(token, local)            # each time: the page may join a game meanwhile
                if seat is None: continue
                evs = [x for x in (for_seat(e, seat) for e in evs) if x is not None]
                if len(evs) > 20: evs = catch_up(evs, s.hub.pending.get(seat))
                for e in evs:
                    s.wfile.write(f"id: {e['id']}\ndata: {json.dumps(e)}\n\n".encode())
                s.wfile.flush()
        except (BrokenPipeError, ConnectionResetError, ConnectionAbortedError):
            pass


def make_server(port=8765, host='127.0.0.1'):
    hub = Hub(lan=host not in ('127.0.0.1', 'localhost'))
    handler = type('PracticeHandler', (Handler,), {'hub': hub})
    srv = ThreadingHTTPServer((host, port), handler)
    srv.daemon_threads = True
    srv.hub = hub
    return srv


def start(port=0, fetch_images=True):
    """the server on this device only, running on a background thread (the iPad app): returns (server, url). Port 0
    takes any free port"""
    srv = make_server(port, '127.0.0.1')
    if fetch_images: threading.Thread(target=fetch_commanders, name='practice-commanders', daemon=True).start()
    threading.Thread(target=srv.serve_forever, name='practice-server', daemon=True).start()
    return srv, f'http://127.0.0.1:{srv.server_address[1]}/'


def serve(port=8765, open_browser=True, lan=False):
    srv = make_server(port, '0.0.0.0' if lan else '127.0.0.1')
    threading.Thread(target=fetch_commanders, name='practice-commanders', daemon=True).start()
    url = f'http://127.0.0.1:{srv.server_address[1]}/'
    print(f'Practice mode: {url}  (Ctrl+C to stop)')
    if lan:
        addrs = lan_addresses()
        print('Listening on the local network too. Your friend opens '
              + (' or '.join(f'http://{a}:{srv.server_address[1]}/' for a in addrs) or f'this computer\'s address, port {srv.server_address[1]}')
              + ' (the join code shows when you open a two-player table).')
        print('If their page does not load, allow the port through this computer\'s firewall '
              f'(e.g. sudo ufw allow {srv.server_address[1]}/tcp). Anyone on the network can open the page; '
              'only someone with the code can take the second seat.')
    if open_browser: threading.Timer(0.5, lambda: webbrowser.open(url)).start()
    try:
        srv.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        srv.hub.quit(); srv.server_close()
