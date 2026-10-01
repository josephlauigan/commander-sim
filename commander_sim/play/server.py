"""The browser table's server: standard library only (http.server), one game at a time, on 127.0.0.1.

Routes:
  GET  /                    the page (static/index.html); GET /static/<file> for its files
  GET  /api/options         your decks and the tiers (with each tier's decks), for the setup screen
  GET  /api/state           the game now: its settings, your seat's latest view, the decision waiting for you
  GET  /api/events          server-sent events: every event of the game in order, each with an id (a reconnect
                            with Last-Event-ID, or ?since=N, resumes after it)
  GET  /images/<file>       a card image from the cache (data/images/)
  POST /api/new             start a game: {deck, tier, seed?, seat?, opponents?, profile?, ai?, tools?, images?}:
                            tools are the practice switches {hint, undo, compare} (used from Phase 3); the card
                            images load first ('loading' events with done/total, then 'images': name -> files)
  POST /api/answer          answer the waiting decision: {id, answer}; a stale id is refused (409)
  POST /api/undo            take back your last answer ({n}: the last n): the game replays from its seed
  GET  /api/review          after the game: a summary, and each decision with what the AI would have done and both scores
  POST /api/tryit           {n}: back to decision n with the AI's choice there (the review's "Try it")
  POST /api/hint            what the AI would do at the decision waiting for you: {text, detail, choice}
  POST /api/quit            end the game

The engine runs on the session's worker thread. A pump thread moves the session's events into the hub's history
(JSON-ready, numbered) and wakes the event streams. Views are built on the engine thread, never here.
"""
import json
import os
import queue
import threading
import webbrowser
from http.server import ThreadingHTTPServer, BaseHTTPRequestHandler
from urllib.parse import urlparse, parse_qs

from commander_sim import DATA
from commander_sim.play.controller import Request
from commander_sim.play.session import Session, MY_DECKS, TIERS

STATIC = os.path.join(os.path.dirname(__file__), 'static')
IMAGES = os.path.join(DATA, 'images')
TYPES = {'.html': 'text/html; charset=utf-8', '.js': 'text/javascript; charset=utf-8', '.css': 'text/css; charset=utf-8',
         '.gif': 'image/gif', '.png': 'image/png', '.jpg': 'image/jpeg', '.svg': 'image/svg+xml', '.json': 'application/json'}


def jsonable(x):
    """an event or request as plain JSON data (anything unexpected becomes its text)"""
    return json.loads(json.dumps(x, default=lambda o: o.__dict__ if isinstance(o, Request) else str(o)))


class Hub:
    """the current game and its event history, shared by the request handlers"""
    def __init__(s):
        s.cond = threading.Condition()
        s.session = None
        s.events = []                # [{'id': n, ...}] for the current game
        s.pending = None             # the request waiting for an answer: its event
        s.view = None                # the latest view of the table from your seat
        s.images = {}                # card name -> image files, for this game
        s.tools = {}
        s.next_id = 1
        s.game_no = 0

    # ------------------------------------------------------------------ games
    def new_game(s, opts):
        s.quit()
        sess = Session(opts['deck'], opts['tier'], seed=opts.get('seed'), seat=opts.get('seat'),
                       opponents=opts.get('opponents'), profile=opts.get('profile', 'loose'),
                       ai=opts.get('ai', 'lookahead'), views=True,
                       compare=bool((opts.get('tools') or {}).get('compare', True)), seats=opts.get('seats'))
        with s.cond:
            s.game_no += 1
            s.session, s.events, s.pending, s.view, s.images = sess, [], None, None, {}
            s.tools = {k: bool((opts.get('tools') or {}).get(k, True)) for k in ('hint', 'undo', 'compare')}
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
                sess.events.put({'kind': 'log', 'text': f'(card images unavailable: {e})'})
        sess.events.put({'kind': 'images', 'images': found})
        with s.cond:
            if s.game_no != no: return                  # replaced or quit while loading
        sess.start()

    def quit(s):
        with s.cond:
            sess, s.session = s.session, None
            s.pending = None
            s.cond.notify_all()
        if sess is not None: sess.close()

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
                s._add(ev)

    def _add(s, ev):
        """(holding the lock) record one session event"""
        if ev['kind'] == 'reset':                       # Undo: the game restarts; what came before is replayed
            s.events, s.pending, s.view = [], None, None
        if ev['kind'] == 'request':
            req = ev['request']
            out = {'kind': 'request', 'request': {'kind': req.kind, 'prompt': req.prompt, 'choices': req.choices,
                                                  'data': {k: v for k, v in req.data.items() if k != 'view'}}}
            if 'view' in req.data: s.view = req.data['view']
        else:
            out = {k: v for k, v in ev.items() if k != 'view'}
            if ev.get('view') is not None: s.view = ev['view']
            if ev['kind'] == 'images': s.images = ev['images']
        if ev.get('view') is not None or (ev['kind'] == 'request' and 'view' in ev['request'].data):
            out['view'] = s.view
        out['id'] = s.next_id; s.next_id += 1
        out = jsonable(out)
        if out['kind'] == 'request': s.pending = out
        s.events.append(out)
        s.cond.notify_all()

    def answer(s, rid, value):
        """None, or why the answer can't be taken"""
        with s.cond:
            if s.session is None: return 'There is no game running.'
            if s.pending is None or s.pending['id'] != rid: return 'That decision has already been answered.'
            s.pending = None
            sess = s.session
        sess.answer(value)
        return None

    def undo(s, n=1):
        """None, or why it can't"""
        with s.cond:
            sess = s.session
            if sess is None: return 'There is no game running.'
            if not s.tools.get('undo', True): return 'Undo is switched off for this game.'
        return sess.undo(n)

    def review(s):
        with s.cond:
            sess = s.session
        if sess is None: return 'There is no game.'
        return sess.review()

    def hint(s):
        with s.cond:
            sess = s.session
            if sess is None: return 'There is no game running.'
            if not s.tools.get('hint', True): return 'Hint is switched off for this game.'
        return sess.hint()

    def state(s):
        with s.cond:
            sess = s.session
            game = None if sess is None else {'deck': sess.deck, 'tier': sess.tier, 'seed': sess.seed, 'seats': sess.seats,
                                              'profile': sess.profile, 'ai': sess.ai, 'tools': s.tools}
            return {'game': game, 'view': s.view, 'pending': s.pending, 'images': s.images,
                    'last_event': s.events[-1]['id'] if s.events else 0}

    def events_after(s, since, timeout=15.0):
        """events with id > since (waits up to timeout for one); [] on timeout"""
        with s.cond:
            if not any(e['id'] > since for e in s.events[-1:]):
                s.cond.wait(timeout)
            return [e for e in s.events if e['id'] > since]


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

    def log_message(s, fmt, *args):                    # quiet: the terminal is for the game
        pass

    # ------------------------------------------------------------------ helpers
    def _send(s, code, body, ctype='application/json'):
        data = body if isinstance(body, bytes) else (json.dumps(body) if ctype == 'application/json' else body).encode()
        s.send_response(code)
        s.send_header('Content-Type', ctype)
        s.send_header('Content-Length', str(len(data)))
        s.send_header('Cache-Control', 'no-store')
        s.end_headers()
        s.wfile.write(data)

    def _body(s):
        n = int(s.headers.get('Content-Length') or 0)
        if not n: return {}
        try:
            return json.loads(s.rfile.read(n))
        except ValueError:
            return None

    # ------------------------------------------------------------------ routes
    def do_GET(s):
        url = urlparse(s.path)
        if url.path in ('/', '/index.html'): return s._static('index.html')
        if url.path.startswith('/static/'): return s._static(url.path[len('/static/'):])
        if url.path.startswith('/images/'): return s._static(url.path[len('/images/'):], root=IMAGES)
        if url.path == '/api/state': return s._send(200, s.hub.state())
        if url.path == '/api/options': return s._send(200, options())
        if url.path == '/api/review':
            res = s.hub.review()
            return s._send(409, {'error': res}) if isinstance(res, str) else s._send(200, res)
        if url.path == '/api/events':
            q = parse_qs(url.query)
            since = s.headers.get('Last-Event-ID') or (q.get('since') or ['0'])[0]
            return s._stream(int(since) if str(since).isdigit() else 0)
        s._send(404, {'error': 'not found'})

    def do_POST(s):
        url = urlparse(s.path)
        body = s._body()
        if body is None: return s._send(400, {'error': 'the request body is not JSON'})
        if url.path == '/api/new':
            if body.get('deck') not in MY_DECKS: return s._send(400, {'error': f'deck: one of {", ".join(MY_DECKS)}'})
            if body.get('tier') not in TIERS: return s._send(400, {'error': f'tier: one of {", ".join(TIERS)}'})
            if body.get('opponents') is not None and not (isinstance(body['opponents'], list)
                                                          and all(isinstance(x, str) for x in body['opponents'])):
                return s._send(400, {'error': 'opponents: a list of three deck keys'})
            if body.get('seats') is not None and not (isinstance(body['seats'], list) and all(isinstance(x, str) for x in body['seats'])):
                return s._send(400, {'error': 'seats: four deck keys in turn order'})
            if body.get('seat') is not None and not isinstance(body['seat'], int):
                return s._send(400, {'error': 'seat: 1 to 4'})
            try:
                sess = s.hub.new_game(body)
            except (ValueError, TypeError) as e:
                return s._send(400, {'error': str(e)})
            return s._send(200, {'ok': True, 'seed': sess.seed, 'seats': sess.seats})
        if url.path == '/api/answer':
            why = s.hub.answer(body.get('id'), body.get('answer'))
            return s._send(409 if why else 200, {'error': why} if why else {'ok': True})
        if url.path == '/api/undo':
            why = s.hub.undo(int(body.get('n', 1)) if str(body.get('n', 1)).isdigit() else 1)
            return s._send(409 if why else 200, {'error': why} if why else {'ok': True})
        if url.path == '/api/tryit':
            with s.hub.cond:
                sess = s.hub.session
            why = 'There is no game.' if sess is None else sess.try_it(body.get('n'))
            return s._send(409 if why else 200, {'error': why} if why else {'ok': True})
        if url.path == '/api/hint':
            res = s.hub.hint()
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
        s.send_response(200)
        s.send_header('Content-Type', 'text/event-stream')
        s.send_header('Cache-Control', 'no-store')
        s.send_header('Connection', 'close')
        s.end_headers()
        s.close_connection = True
        try:
            while True:
                evs = s.hub.events_after(since)
                if not evs:
                    s.wfile.write(b': keep-alive\n\n'); s.wfile.flush(); continue
                if len(evs) > 20: evs = catch_up(evs, s.hub.pending)
                for e in evs:
                    s.wfile.write(f"id: {e['id']}\ndata: {json.dumps(e)}\n\n".encode())
                    since = e['id']
                s.wfile.flush()
        except (BrokenPipeError, ConnectionResetError, ConnectionAbortedError):
            pass


def make_server(port=8765, host='127.0.0.1'):
    hub = Hub()
    handler = type('PracticeHandler', (Handler,), {'hub': hub})
    srv = ThreadingHTTPServer((host, port), handler)
    srv.daemon_threads = True
    srv.hub = hub
    return srv


def serve(port=8765, open_browser=True):
    srv = make_server(port)
    threading.Thread(target=fetch_commanders, name='practice-commanders', daemon=True).start()
    url = f'http://127.0.0.1:{srv.server_address[1]}/'
    print(f'Practice mode: {url}  (Ctrl+C to stop)')
    if open_browser: threading.Timer(0.5, lambda: webbrowser.open(url)).start()
    try:
        srv.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        srv.hub.quit(); srv.server_close()
