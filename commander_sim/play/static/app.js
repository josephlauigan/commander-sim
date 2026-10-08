// The browser table: the setup screen, the loading screen, the table (table.js), the log, and your decisions.
import { el, card as cardOf, renderTable as drawTable, renderSteps, fitBoards } from './table.js';
const $ = (sel) => document.querySelector(sel);
let lastId = 0, pending = null, source = null;
let liveFrom = 0;           // events up to this id are history replayed on load: no pop-up messages for them
let images = {};            // card name -> image files in /images/ (the table uses them from step 2d)
// who this page is: its seat, whether it's on the host computer, and (two players) the people at the table
let me = { seat: null, host: true, humans: [], names: {} };
let waitingOn = null;       // two players: the other person, while the game waits on their decision
const two = () => me.humans.length > 1;
const otherName = () => { const k = me.humans.find((x) => x !== me.seat); return (k && me.names[k]) || 'your friend'; };

function loading(done, total) {
  const box = $('#loading');
  if (done === null) { box.hidden = true; return; }
  box.hidden = false;
  const pct = total ? Math.round((100 * done) / total) : 0;
  box.querySelector('.bar > div').style.width = pct + '%';
  box.querySelector('.bar').setAttribute('aria-valuenow', pct);
}

async function api(path, body) {
  const r = await fetch(path, body === undefined ? {} : { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) });
  return { ok: r.ok, data: await r.json() };
}

// ------------------------------------------------------------------ the table
let lastView = null;
let passBtn = null;          // the Pass priority button: moved into the table's pass row after each drawing
function renderTable(view) {
  lastView = view;
  passBtn = passBtn || document.getElementById('pass');
  drawTable($('#table'), view, images);
  const row = document.querySelector('#table .passbar');
  if (row && passBtn) row.append(passBtn);
  if (!$('#game').hidden) fitBoards($('#table'));
  renderSteps($('#steps'), view);
}
const card = (name, o = {}) => cardOf(images, name, Object.assign({ size: 'sm' }, o));

// ------------------------------------------------------------------ decisions
// taps you made since your last other action, most recent last: {land} / {perm} / {treasure}. A tapped land among
// them can be untapped (Undo takes back that tap and any made after it)
let recentTaps = [];

async function answer(value, where) {
  if (!pending) return;
  if (value && value.do === 'tap') recentTaps.push(where || {});
  else if (pending.request.kind === 'priority') recentTaps = [];
  const id = pending.id;
  pending = null; renderPrompt(null); markChoices(null);
  const r = await api('/api/answer', { id, answer: value });
  if (!r.ok) logLine(r.data.error, 'invalid');
}

const btn = (label, value, cls = '') => el('button', { class: cls, onclick: () => answer(value) }, label);

function you(view) { return view && view.players.find((p) => p.you); }

function renderPrompt(ev) {
  const box = $('#prompt'); box.replaceChildren(); closeMenu();
  requestAnimationFrame(() => fitBoards($('#table')));     // the stack row comes and goes: the areas change size
  if (ev && tryBanner) {
    box.append(el('div', { class: 'tryit' }, tryBanner.applied ? `Trying the AI's choice: ${tryBanner.ai}. Carry on from here.`
      : `The AI would have: ${tryBanner.ai}. Make that play, then carry on.`));
    tryBanner = null;
  }
  $('#hintbox').hidden = true;                // a hint is for the decision it was asked at
  for (const x of document.querySelectorAll('.stackbar')) x.remove();
  (passBtn || $('#pass')).hidden = !(ev && ev.request.kind === 'priority');
  $('#table').classList.toggle('can-act', !!(ev && ev.request.kind === 'priority'));
  if (!ev) { box.append(el('div', { class: 'stats' }, waitingOn ? `Waiting for ${waitingOn}…` : 'Waiting for the other players…')); return; }
  const req = ev.request;
  box.append(el('h3', {}, req.prompt));
  if (req.kind === 'priority') {
    const me = you(ev.view);
    if (req.data.stack && req.data.stack.length) showStack(req);
    box.append(el('p', { class: 'help' }, 'Click a land or mana rock to tap it, a card in your hand to cast or play it, a permanent to use its abilities, your commander in the command zone or a card in your graveyard to cast it.'));
    const row = (title, items) => items.length && list.append(el('div', { class: 'row' }, el('b', {}, title), items));
    const list = el('details', { class: 'actions' }, el('summary', {}, 'The same as a list'));
    row('Tap', (me.mana_sources || []).flatMap((x) => tapButtons(x)));
    row('Hand', me.hand.flatMap((n, i) => me.hand_land[i]
      ? [btn(`Play ${n}`, { do: 'land', card: i })].concat(me.hand_special[i] ? [btn(`Cast ${n.split(' // ')[0]}`, { do: 'cast', card: i })] : [])
      : [btn(`Cast ${n}`, { do: 'cast', card: i })]));
    if (me.commander_in_zone) row('Command zone', [btn(`${me.commander} (tax ${me.tax})`, { do: 'cast', zone: 'cmd' })]);
    row('Activate', me.battlefield.map((m) => btn(m.name, { do: 'use', perm: m.i })));
    row('Land abilities', me.lands.filter((L) => L.ability).map((L) => btn(L.name, { do: 'use', land: L.i })));
    row('Graveyard', (me.graveyard_playable || []).map((x) => btn(`${x.name} (${x.how})`, { do: x.how === 'land' ? 'land' : 'cast', zone: 'gy', card: x.i })));
    box.append(list);
  } else if (req.kind === 'attack') {
    box.append(el('p', { class: 'help' }, 'Click your creatures to attack with them (click again to take one back).'));
    const boxes = req.choices.map((c, i) => el('label', {}, el('input', Object.assign({ type: 'checkbox', value: i,
      onchange: (e) => { if (e.target.checked) attackSel.add(i); else attackSel.delete(i); syncAttack(); } },
      attackSel.has(i) ? { checked: '' } : {})), ' ', c));
    box.append(el('details', { class: 'actions' }, el('summary', {}, 'The same as a list'), el('div', { class: 'row' }, boxes)));
    box.append(el('div', { class: 'row' },
      el('button', { id: 'attack-go', class: 'primary', onclick: () => answer([...attackSel].sort((x, y) => x - y)) }, attackLabel()),
      btn('Attack with everything', req.choices.map((_, i) => i)), btn('No attack', [])));
  } else if (req.kind === 'mulligan') {
    box.append(el('div', { class: 'row' }, (req.data.names || req.data.hand || []).map((n) => card(n, { size: 'md' }))));
    box.append(el('div', { class: 'row' }, btn('Keep', 'keep', 'primary'), btn('Mulligan', 'mulligan')));
  } else if (req.kind === 'continue') {
    box.append(el('div', { class: 'row' }, btn('Continue', true, 'primary')));
  } else {
    box.append(el('div', { class: 'row' }, req.choices.map((c, i) => btn(c, i))));
  }
}

// the stack, between the opponents and your battlefield, with who gets priority on it in turn order
function showStack(req) {
  const opps = document.querySelector('#table .opps');
  if (!opps) return;
  const me = (req.data.order || []).findIndex((x) => you(pending.view) && x.key === you(pending.view).key);
  const order = (req.data.order || []).map((x, i) => el('span', { class: i < me ? 'passed' : i === me ? 'you' : '' },
    i < me ? `${x.name} · passed` : i === me ? 'You' : x.name));
  const top = req.data.stack[0];
  const what = (x) => (x.text ? x.text.split(': ').slice(1).join(': ') || x.text : x.name);
  const head = top.kind && top.kind !== 'spell' ? `${top.controller}: ${what(top)}`
    : req.data.caster ? `${req.data.caster} casts ${top.name}` : 'On the stack';
  opps.after(el('section', { class: 'stackbar', 'aria-label': 'The stack' },
    el('div', { class: 'cards' }, req.data.stack.map((x) => el('figure', { class: 'item' + (x.kind && x.kind !== 'spell' ? ' ability' : '') },
      cardOf(images, x.name, { size: 'sm' }),
      x.controller ? el('figcaption', {}, el('b', {}, x.controller),
        x.kind && x.kind !== 'spell' ? ` · ${x.kind === 'trigger' ? 'trigger' : 'ability'}: ${what(x)}` : '',
        x.target ? ` → ${x.target}` : '') : null))),
    el('div', {}, el('h3', {}, head),
      order.length ? el('div', { class: 'order' }, el('b', {}, 'Priority: '), order.flatMap((o, i) => (i ? [' → ', o] : [o]))) : null)));
}

// ------------------------------------------------------------------ acting on the table (your priority)
function tapButtons(x) {
  return x.colours.length > 1 ? [...x.colours].map((c) => btn(`${x.name} {${c}}`, { do: 'tap', source: x.id, colour: c }))
    : [btn(`${x.name} {${x.colours}}`, { do: 'tap', source: x.id })];
}

function toast(text) {
  const t = $('#toast');
  t.textContent = text; t.hidden = false;
  clearTimeout(toast.timer); toast.timer = setTimeout(() => { t.hidden = true; }, 6000);
}
const note = (text) => toast(text);

// two players: the other person asks to Undo or Try it, and you say yes or no
function showProposal(p) {
  const box = $('#proposal');
  if (!p) { box.hidden = true; box.replaceChildren(); return; }
  const reply = async (yes) => {
    box.hidden = true;
    const r = await api('/api/approve', { id: p.id, yes });
    if (!r.ok) toast(r.data.error);
  };
  box.replaceChildren(el('p', {}, p.text), el('div', { class: 'row' },
    el('button', { type: 'button', onclick: () => reply(false) }, 'No'),
    el('button', { type: 'button', class: 'primary', onclick: () => reply(true) }, 'Yes, go back')));
  box.dataset.id = p.id; box.hidden = false;
}

function closeMenu() { const m = $('#menu'); if (m) m.remove(); }

function menu(e, title, items) {
  closeMenu();
  const m = el('div', { id: 'menu', role: 'menu' }, el('div', { class: 'title' }, title),
    items.map(([label, value, where]) => el('button', { role: 'menuitem', onclick: () => { closeMenu(); typeof value === 'function' ? value() : answer(value, where); } }, label)));
  document.body.append(m);
  const r = m.getBoundingClientRect();
  m.style.left = `${Math.min(e.clientX, window.innerWidth - r.width - 8)}px`;
  m.style.top = `${Math.min(e.clientY, window.innerHeight - r.height - 8)}px`;
  m.querySelector('button').focus();
}

function act(e, title, items, otherwise) {
  if (!items.length) { toast(otherwise); return; }
  if (items.length === 1) { typeof items[0][1] === 'function' ? items[0][1]() : answer(items[0][1], items[0][2]); return; }
  menu(e, title, items);
}

const tapItems = (srcs, where) => srcs.flatMap((x) => x.colours.length > 1
  ? [...x.colours].map((c) => [`Tap for {${c}}`, { do: 'tap', source: x.id, colour: c }, where])
  : [[`Tap for {${x.colours}}`, { do: 'tap', source: x.id }, where]]);

function untapItem(where) {
  const k = recentTaps.map((w) => JSON.stringify(w)).lastIndexOf(JSON.stringify(where));
  if (k < 0) return [];
  const n = recentTaps.length - k;
  return [[n === 1 ? 'Untap (take back that tap)' : `Untap (takes back your last ${n} taps)`, async () => {
    const r = await api('/api/undo', { n });
    if (!r.ok) toast(r.data.error);
    else if (r.data.asked) { recentTaps = []; note(`Asked ${r.data.asked} to agree to the untap…`); }
    else recentTaps = recentTaps.slice(0, k);
  }]];
}

// declaring attackers: the creatures picked so far (indices into the request's choices)
let attackSel = new Set();
const attackLabel = () => (attackSel.size ? `Attack with ${attackSel.size}` : 'Attack with none');
function syncAttack() {
  for (const x of document.querySelectorAll('[data-choice]')) x.classList.toggle('attacking', attackSel.has(+x.dataset.choice));
  const go = document.getElementById('attack-go'); if (go) go.textContent = attackLabel();
  for (const b of document.querySelectorAll('#prompt input[type=checkbox]')) b.checked = attackSel.has(+b.value);
}

// a choice whose answer is on the table (a target, a card to discard): those cards and players light up and a click
// picks them; the list in the panel still works
function markChoices(ev) {
  for (const x of document.querySelectorAll('.targetable, .attacking, .attacker-now')) {
    x.classList.remove('targetable', 'attacking', 'attacker-now'); delete x.dataset.choice;
  }
  if (!ev || ev.request.kind === 'priority') return;
  const at = (r) => r && document.querySelector(r.player ? `[data-player="${CSS.escape(r.player)}"]`
    : `[data-seat="${CSS.escape(r.seat)}"][data-i="${r.perm}"]`);
  for (const r of ev.request.data.attackers || []) { const x = at(r); if (x) x.classList.add('attacking'); }
  { const x = at(ev.request.data.attacker); if (x) x.classList.add('attacker-now'); }
  if (!ev.request.data.refs) return;
  ev.request.data.refs.forEach((r, k) => {
    if (!r) return;
    const sel = r.player ? `[data-player="${CSS.escape(r.player)}"]` : r.hand !== undefined ? `[data-hand="${r.hand}"]`
      : `[data-seat="${CSS.escape(r.seat)}"][data-i="${r.perm}"]`;
    const x = document.querySelector(sel);
    if (x && x.dataset.choice === undefined) { x.classList.add('targetable'); x.dataset.choice = k; }
  });
}

function onTableClick(e) {
  const pick = e.target.closest('[data-choice]');
  if (pick && pending && pending.request.kind === 'attack') {
    e.preventDefault(); const i = +pick.dataset.choice;
    if (attackSel.has(i)) attackSel.delete(i); else attackSel.add(i);
    syncAttack(); return;
  }
  if (pick && pending && pending.request.kind !== 'priority') {
    e.preventDefault(); e.stopPropagation(); answer(+pick.dataset.choice); return;
  }
  const t = e.target.closest('[data-land],[data-perm],[data-hand],[data-cmd],[data-gy],[data-treasure]');
  if (!t) return;
  if (!pending || pending.request.kind !== 'priority') { toast("You don't have priority right now."); return; }
  e.preventDefault(); hidePreviewSoon();
  const me = you(pending.view); const srcs = me.mana_sources || [];
  const name = t.dataset.name || '';
  if (t.dataset.land !== undefined) {
    const i = +t.dataset.land, L = me.lands[i];
    const items = tapItems(srcs.filter((x) => x.land === i), { land: i });
    if (L.tapped && tools.undo) items.push(...untapItem({ land: i }));
    if (L.ability) items.push(['Activate an ability', { do: 'use', land: i }]);
    act(e, name, items, L.tapped ? `${name} is tapped.` : `${name} can't be tapped for mana right now.`);
  } else if (t.dataset.perm !== undefined) {
    const i = +t.dataset.perm;
    const items = tapItems(srcs.filter((x) => x.perm === i), { perm: i });
    if (me.battlefield.find((m) => m.i === i)?.tapped && tools.undo) items.push(...untapItem({ perm: i }));
    items.push(['Activate an ability', { do: 'use', perm: i }]);
    act(e, name, items, '');
  } else if (t.dataset.treasure !== undefined) {
    act(e, 'Treasure', tapItems(srcs.filter((x) => x.treasure).slice(0, 1), { treasure: 1 }), 'No untapped Treasure.');
  } else if (t.dataset.hand !== undefined) {
    const i = +t.dataset.hand;
    if (!me.hand_land[i]) answer({ do: 'cast', card: i });
    else if (me.hand_special[i]) menu(e, name, [['Play it as your land', { do: 'land', card: i }], [`Cast ${name.split(' // ')[0]}`, { do: 'cast', card: i }]]);
    else answer({ do: 'land', card: i });
  } else if (t.dataset.cmd !== undefined) {
    answer({ do: 'cast', zone: 'cmd' });
  } else if (t.dataset.gy !== undefined) {
    const i = +t.dataset.gy;
    const playable = (me.graveyard_playable || []).find((x) => x.i === i);
    answer({ do: playable && playable.how === 'land' ? 'land' : 'cast', zone: 'gy', card: i });
  }
}

function hidePreviewSoon() { const p = document.getElementById('preview'); if (p) p.hidden = true; }

// ------------------------------------------------------------------ the event stream
let logQueue = [];           // log lines waiting to be added (a burst of events adds them in one go)
function logLine(text, cls = '') {
  logQueue.push(el('div', { class: cls }, text));
  if (logQueue.length === 1) setTimeout(flushLog, 0);
}

function flushLog() {
  if (!logQueue.length) return;
  const log = $('#log');
  const atEnd = log.scrollTop + log.clientHeight >= log.scrollHeight - 4;
  log.append(...logQueue); logQueue = [];
  if (atEnd) log.scrollTop = log.scrollHeight;
}

let stale = null;            // a view not drawn yet (history catching up draws only the last of a run of views)
let drawTimer = null;
function scheduleDraw() {
  if (drawTimer) return;
  drawTimer = setTimeout(() => { drawTimer = null; if (stale) { renderTable(stale); stale = null; markChoices(pending); } }, 40);
}
function onEvent(ev, draw = true) {
  if (ev.view) { if (draw || ev.kind === 'request') { renderTable(ev.view); stale = null; } else stale = ev.view; }
  else if (stale && (ev.kind === 'request' || ev.kind === 'over')) { renderTable(stale); stale = null; }
  if (ev.kind === 'log') logLine(ev.text, ev.text.startsWith('---') ? 'turn' : '');
  else if (ev.kind === 'invalid') { logLine('Not allowed: ' + ev.text, 'invalid'); if (ev.id > liveFrom && !ev.replay) toast(ev.text); }
  else if (ev.kind === 'reset') {           // Undo or Try it: the game is replayed from its seed up to a decision
    const mine = !ev.by || ev.by === me.seat || !two();
    const who = me.names[ev.by] || ev.by;
    if (!ev.undone || !mine) recentTaps = [];
    $('#log').replaceChildren(); logQueue = []; pending = null; waitingOn = null; renderPrompt(null); markChoices(null);
    showProposal(null);
    if (ev.tryit) {
      if (mine) tryBanner = ev.tryit;
      logLine(mine ? `(Try it: the game is replayed to that decision, with the AI's choice: ${ev.tryit.ai})`
        : `(Try it: ${who} goes back to one of their decisions to try the AI's choice; the game is replayed to there)`, 'auto');
      screen('game');
    } else logLine(mine ? `(Undo: ${ev.undone} action${ev.undone > 1 ? 's' : ''} taken back; the game so far is replayed)`
      : `(Undo: ${who} took back ${ev.undone} action${ev.undone > 1 ? 's' : ''}; the game so far is replayed)`, 'auto');
  }
  else if (ev.kind === 'waiting') { waitingOn = ev.who; if (!pending) renderPrompt(null); }
  else if (ev.kind === 'decided') {
    if (ev.by === me.seat) {                  // your answer (history caught up on a reload: that decision is done)
      if (pending && pending.id < ev.id) { pending = null; renderPrompt(null); markChoices(null); }
    } else if (ev.who === waitingOn) { waitingOn = null; if (!pending) renderPrompt(null); }
  }
  else if (ev.kind === 'proposal') { if (ev.id > liveFrom) showProposal(ev.proposal); }
  else if (ev.kind === 'proposal_done') { if (+$('#proposal').dataset.id === ev.id) showProposal(null); }
  else if (ev.kind === 'declined') { if (ev.id > liveFrom) toast(ev.text); }
  else if (ev.kind === 'reviewing') {
    $('#prompt').replaceChildren(el('h3', {}, 'Game over'), el('p', {}, `Working out the AI comparison: ${ev.done} of ${ev.total} decisions…`));
  }
  else if (ev.kind === 'auto') logLine('(automatic) ' + ev.text, 'auto');
  else if (ev.kind === 'loading') loading(ev.done, ev.total);
  else if (ev.kind === 'images') { images = ev.images; loading(null); if (lastView) renderTable(lastView); }
  else if (ev.kind === 'request') {
    if (!ev.view) ev.view = lastView;          // caught-up history carries only the latest table
    pending = ev; waitingOn = null; attackSel = new Set(); renderPrompt(ev); markChoices(ev);
  }
  else if (ev.kind === 'over') {
    pending = null; waitingOn = null; renderPrompt(null); tryBanner = null; showProposal(null);
    if (ev.how === 'closed' && !me.host) {
      $('#prompt').replaceChildren(el('h3', {}, 'The host ended the game.'));
      return;
    }
    const winner = ev.winner && ev.winner === me.seat ? 'you won' : ((me.names && me.names[ev.winner]) || ev.winner || 'no winner');
    $('#prompt').replaceChildren(el('h3', {}, `Game over: ${winner}${ev.how ? ` (${ev.how})` : ''}`),
      el('div', { class: 'row' }, tools.compare ? el('button', { class: 'primary', onclick: showReview }, 'Review the game') : null,
        me.host && !two() ? el('button', { onclick: sameSeed }, 'Play this seed again') : null,
        me.host ? el('button', { onclick: saveGame }, 'Save game') : null,
        me.host ? el('button', { onclick: () => screen('setup') }, 'New game') : null));
    if ((location.hash === '#review' || new URLSearchParams(location.search).has('review')) && tools.compare) showReview();   // a direct link
  }
  else if (ev.kind === 'error') logLine(ev.text, 'invalid');
}

// ------------------------------------------------------------------ playback of opponents' turns
// Events queue up here. An action during another player's turn (a log line with its view of the table) is shown,
// then the next one waits BASE / speed ms; paused, only "Next action" moves on. Everything else (your own turn,
// history replayed when the page loads) shows at once. The engine is waiting on your next decision meanwhile, so
// a decision is reached only after the actions before it have played.
const BASE = 1100;
let inbox = [], paused = false, speed = 1, stepOnce = false, skipping = false, pumping = false;
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const theirTurn = (view) => view && view.players.some((p) => p.key === view.active && !p.you);
const paced = (ev) => ev.id > liveFrom && !ev.replay && ev.kind === 'log' && ev.view && theirTurn(ev.view);

function receive(ev) { inbox.push(ev); pump(); }

async function pump() {
  if (pumping) return;
  pumping = true;
  try {
    while (inbox.length) {
      const ev = inbox[0];
      if (paced(ev) && !skipping) {
        if (paused && !stepOnce) break;
        stepOnce = false;
        inbox.shift(); onEvent(ev); showControls();
        if (!paused) await sleep(BASE / speed);
      } else {
        inbox.shift();
        onEvent(ev, false);                  // drawn once the burst is over (scheduleDraw), not once per event
        if (ev.kind === 'request' || ev.kind === 'over') skipping = false;
      }
    }
  } finally {
    pumping = false; showControls(); scheduleDraw();
  }
}

function showControls() {
  const bar = $('#playback');
  const waiting = inbox.some(paced);
  bar.hidden = $('#game').hidden || !(theirTurn(lastView) || waiting);
  $('#pb-pause').textContent = paused ? '▶ Play' : '❚❚ Pause';
  $('#pb-next').disabled = !paused || !waiting;
  for (const b of bar.querySelectorAll('[data-speed]')) b.classList.toggle('on', +b.dataset.speed === speed);
}

function setupPlayback() {
  $('#pb-pause').addEventListener('click', () => { paused = !paused; showControls(); pump(); });
  $('#pb-next').addEventListener('click', () => { stepOnce = true; pump(); });
  $('#pb-skip').addEventListener('click', () => { skipping = true; pump(); });
  for (const b of $('#playback').querySelectorAll('[data-speed]')) b.addEventListener('click', () => { speed = +b.dataset.speed; showControls(); });
}

function connect() {
  if (source) source.close();
  source = new EventSource(`/api/events?since=${lastId}`);
  source.onmessage = (m) => { const ev = JSON.parse(m.data); lastId = Math.max(lastId, ev.id); receive(ev); };
}

// ------------------------------------------------------------------ the review (after the game)
let tryBanner = null, reviewData = null, diffsOnly = true;

async function showReview() {
  const r = await api('/api/review');
  if (!r.ok) { toast(r.data.error); return; }
  reviewData = r.data; renderReview(); screen('review');
}

const fmt = (x) => (x == null ? '–' : (x > 0 ? '+' : '') + x.toFixed(1));

function renderReview() {
  const { summary: m, decisions } = reviewData;
  const rows = decisions.filter((d) => !diffsOnly || d.differs);
  $('#review').replaceChildren(
    el('h2', {}, m.won ? `You won in round ${m.rounds}` : m.winner ? `${m.winner} won in round ${m.rounds}${m.how ? ` (${m.how})` : ''}` : `No winner after ${m.rounds} rounds`),
    el('p', { class: 'summary' }, `${m.answers} answers in all; ${m.decisions} decisions the AI weighs, ${m.compared} compared: `,
      el('b', {}, `${m.matched} matched the AI`), `, `, el('b', { class: 'worse' }, `${m.worse} clearly worse`),
      ` (the AI's choice scored ${m.worse_margin} or more higher).`),
    el('p', { class: 'help' }, 'Scores: where the game stood at the end of your next turn, averaged over six playouts, from −100 (lost) to +100 (won).'),
    el('div', { class: 'row' }, el('label', {}, el('input', Object.assign({ type: 'radio', name: 'rv', onchange: () => { diffsOnly = true; renderReview(); } }, diffsOnly ? { checked: '' } : {})), ' Differences only'),
      el('label', {}, el('input', Object.assign({ type: 'radio', name: 'rv', onchange: () => { diffsOnly = false; renderReview(); } }, diffsOnly ? {} : { checked: '' })), ' All decisions')),
    rows.length ? el('table', { class: 'review' },
      el('thead', {}, el('tr', {}, ['When', 'Situation', 'You', 'The AI', 'Your score', "AI's score", ''].map((h) => el('th', {}, h)))),
      el('tbody', {}, rows.map((d) => el('tr', { class: d.scored && d.score_ai - d.score_yours >= m.worse_margin ? 'worse' : '' },
        el('td', { class: 'when' }, `Round ${d.round} · ${d.step_text}`), el('td', {}, d.situation), el('td', {}, d.yours),
        el('td', {}, d.ai_text || (d.note ? el('em', {}, d.note) : '–')),
        el('td', { class: 'num' }, fmt(d.score_yours)), el('td', { class: 'num' }, fmt(d.score_ai)),
        el('td', {}, d.can_try && d.differs ? el('button', { class: 'try', onclick: () => tryIt(d.n) }, 'Try it') : null)))))
      : el('p', {}, diffsOnly ? 'You and the AI made the same choices everywhere they were compared.' : 'No decisions recorded.'),
    el('div', { class: 'row' }, el('button', { onclick: () => screen('game') }, 'Back to the table'),
      me.host && !two() ? el('button', { onclick: sameSeed }, 'Play this seed again') : null,
      me.host ? el('button', { class: 'primary', onclick: () => screen('setup') }, 'New game') : null));
}

async function tryIt(n) {
  const r = await api('/api/tryit', { n });
  if (!r.ok) toast(r.data.error);
  else if (r.data.asked) note(`Asked ${r.data.asked} to agree to going back…`);
}

async function saveGame() {
  const r = await api('/api/save', {});
  if (!r.ok) { toast(r.data.error); return; }
  const t = $('#toast'); t.textContent = `Saved: ${r.data.name}`; t.hidden = false;
  clearTimeout(toast.timer); toast.timer = setTimeout(() => { t.hidden = true; }, 4000);
  listSaves();
}

async function listSaves() {
  const r = await api('/api/saves');
  if (!r.ok) return;
  showResume(r.data.autosave);
  if (!r.data.saves.length) return;
  $('#saves').replaceChildren(el('table', { class: 'review' },
    el('thead', {}, el('tr', {}, ['Saved', 'Deck', 'Tier', 'Seed', 'Where', ''].map((h) => el('th', {}, h)))),
    el('tbody', {}, r.data.saves.map((x) => el('tr', {}, el('td', {}, x.saved_at || ''), el('td', {}, x.partner ? `${x.deck} + ${x.partner} (two players)` : x.deck),
      el('td', {}, (x.tier || '').toUpperCase()), el('td', {}, x.seed),
      el('td', {}, x.finished ? `finished, round ${x.round}` : `round ${x.round}`),
      el('td', {}, el('button', { onclick: () => loadGame(x.name) }, x.finished ? 'Open (review)' : 'Continue')))))));
}

// the game in progress when the page or the app was closed (the server saves it at each of your decisions)
function showResume(x) {
  const box = $('#resume');
  box.hidden = !x || x.finished;
  if (box.hidden) return;
  const deck = catalog.decks.find((d) => d.key === x.deck);
  box.replaceChildren(el('span', {}, el('strong', {}, 'Your last game: '),
    `${deck ? deck.name : x.deck} at ${(x.tier || '').toUpperCase()}, round ${x.round}`, el('small', {}, ` (seed ${x.seed}, ${x.saved_at || ''})`)),
  el('button', { class: 'primary big', onclick: () => loadGame('autosave.json') }, 'Continue'));
}

async function loadGame(name) {
  $('#log').replaceChildren(); logQueue = []; pending = null; inbox = []; skipping = false;
  renderTable(null); renderPrompt(null); loading(0, 0);
  const r = await api('/api/load', { name });
  if (!r.ok) { loading(null); toast(r.data.error); return; }
  if (r.data.code) { loading(null); showLobby(); return; }       // a two-player game: waits for your friend
  await enterGame();
}

// the game has started (or this page joined one): its settings, then the table
async function enterGame() {
  const st = (await api('/api/state')).data;
  if (!st.game) { route(st); return; }
  setMe(st);
  setStatus(st.game.seed, st.game.seats); setTools(st.game.tools); setAutopass(st.game.autopass); screen('game');
  if (st.proposal) showProposal(st.proposal);
}

// when the game stops to give you priority (play/human.py autopass); a change applies from your next decision
function setAutopass(mode) { $('#autopass').value = mode || 'respond'; }
async function changeAutopass() {
  const r = await api('/api/autopass', { mode: $('#autopass').value });
  if (!r.ok) toast(r.data.error);
  else note('From your next decision on.');
}

function setMe(st) {
  me = { seat: st.game ? st.game.you : null, host: st.host, humans: st.game ? st.game.humans : [], names: st.game ? st.game.names : {} };
  $('#save').dataset.host = st.host ? '1' : '';
}

// ------------------------------------------------------------------ two players: waiting for your friend, joining
let pollTimer = null;
function poll(fn, ms) { clearTimeout(pollTimer); pollTimer = setTimeout(fn, ms); }

async function showLobby() {
  const st = (await api('/api/state')).data;
  if (st.game) { await enterGame(); return; }
  const lb = st.lobby;
  if (!lb) { screen('setup'); return; }
  const port = location.port ? `:${location.port}` : '';
  const links = (lb.addresses || []).map((a) => `http://${a}${port}/?join=${lb.code}`);
  const deck = (catalog.decks.find((d) => d.key === lb.deck) || {}).name || lb.deck;
  $('#lobby').replaceChildren(
    el('h2', {}, 'Waiting for your friend'),
    el('p', {}, `You play ${deck}${lb.partner ? ` and your friend plays ${(catalog.decks.find((d) => d.key === lb.partner) || {}).name || lb.partner} (a saved game)` : ''}. On their computer, they open:`),
    links.length ? el('div', {}, links.map((u) => el('div', { class: 'link' }, u)))
      : el('p', { class: 'help' }, `this computer's address on your network, port ${location.port || 80} (no address found: see your network settings)`),
    el('p', {}, 'and pick their deck. The page asks for this code (the link above fills it in):'),
    el('div', { class: 'code' }, lb.code),
    el('p', { class: 'help' }, "Both computers need to be on the same network. If their page doesn't load, this computer's firewall may be blocking the port."),
    el('div', { class: 'row' }, el('button', { type: 'button', onclick: async () => { await api('/api/quit', {}); clearTimeout(pollTimer); screen('setup'); } }, 'Cancel')));
  screen('lobby');
  poll(showLobby, 1000);
}

let joinDeck = null;
async function showJoin(st) {
  const lb = st.lobby;
  const box = $('#join');
  if (!lb) {
    box.replaceChildren(el('h2', {}, st.busy ? 'A game is being played on this table' : 'Waiting for the host'),
      el('p', {}, st.busy ? 'This computer has no seat in it. When the host opens a new table for two, you can join it here.'
        : 'The host opens a table for two on their computer (Players: me and a friend); this page then lets you join.'));
    screen('join'); poll(route, 2000); return;
  }
  const code = new URLSearchParams(location.search).get('join') || '';
  const hostDeck = catalog.decks.find((d) => d.key === lb.deck);
  const allowed = (d) => d.key !== lb.deck && (!lb.partner || d.key === lb.partner);
  if (!joinDeck || !allowed(catalog.decks.find((d) => d.key === joinDeck) || {})) joinDeck = (catalog.decks.find(allowed) || {}).key;
  const prev = box.querySelector('input[name=code]');
  const typed = prev ? prev.value : code;
  const err = el('span', { class: 'error' });
  box.replaceChildren(
    el('h2', {}, 'Join the table'),
    el('p', {}, `The host plays ${hostDeck ? hostDeck.name : lb.deck}, at ${(lb.tier || '').toUpperCase()}, with two AI opponents.`
      + (lb.partner ? ' It is a saved game: you go on with the same deck.' : ' Pick your deck:')),
    el('div', { class: 'decks', role: 'radiogroup', 'aria-label': 'Your deck' }, catalog.decks.map((d) => el('button', Object.assign({
      type: 'button', class: 'deck', role: 'radio', 'aria-checked': String(d.key === joinDeck),
      onclick: () => { joinDeck = d.key; showJoin(st); },
    }, allowed(d) ? {} : { disabled: '' }), cardImage(d.commander), el('span', {}, el('b', {}, d.name),
      el('small', {}, d.key === lb.deck ? "The host's deck" : `Bracket ${d.bracket}`))))),
    el('div', { class: 'row' }, el('label', {}, 'Code ', el('input', { name: 'code', inputmode: 'numeric', autocomplete: 'off', value: typed })),
      el('button', { type: 'button', class: 'primary big', onclick: async () => {
        const r = await api('/api/join', { code: box.querySelector('input[name=code]').value, deck: joinDeck });
        if (!r.ok) { err.textContent = r.data.error; return; }
        clearTimeout(pollTimer);
        $('#log').replaceChildren(); logQueue = []; pending = null; inbox = []; skipping = false; renderTable(null); renderPrompt(null); loading(0, 0);
        await enterGame();
      } }, 'Join'), err));
  screen('join');
  const check = async () => { const s2 = (await api('/api/state')).data; if (s2.game || !s2.lobby) route(s2); else poll(check, 2000); };
  poll(check, 2000);
}

// where a page belongs: the game it has a seat in, waiting for a friend (host), joining (not the host), or setup
async function route(st) {
  st = st || (await api('/api/state')).data;
  if (st.game) { await enterGame(); return; }
  if (!st.host) { await showJoin(st); return; }
  if (st.lobby) { await showLobby(); return; }
  screen('setup');
}

async function sameSeed() {
  const st = (await api('/api/state')).data;
  if (!st.game) { screen('setup'); return; }
  const g = st.game;
  const body = { deck: g.deck, tier: g.tier, seed: g.seed, ai: g.ai, profile: g.profile, tools: g.tools,
    seats: g.seats, autopass: g.autopass };
  $('#log').replaceChildren(); logQueue = []; pending = null; inbox = []; skipping = false; renderTable(null); renderPrompt(null); loading(0, 0);
  const r = await api('/api/new', body);
  if (!r.ok) { loading(null); toast(r.data.error); return; }
  await enterGame();
}

// ------------------------------------------------------------------ setup
let tools = { hint: true, undo: true, compare: true };
function setTools(t) {
  tools = Object.assign({ hint: true, undo: true, compare: true }, t || {});
  $('#undo').hidden = !tools.undo;
  $('#hint').hidden = !tools.hint;
}

async function hint() {
  if (!pending) { toast('There is no decision waiting for you.'); return; }
  const box = $('#hintbox');
  box.hidden = false; box.replaceChildren(el('h3', {}, 'Hint'), el('p', {}, 'The AI is thinking…'));
  const r = await api('/api/hint', {});
  if (!r.ok) { box.replaceChildren(el('h3', {}, 'Hint'), el('p', {}, r.data.error)); return; }
  box.replaceChildren(el('h3', {}, 'The AI would: ' + r.data.text),
    r.data.detail.length ? el('ul', {}, r.data.detail.map((d, i) => (i === 0 ? el('li', { class: 'lead' }, d) : el('li', {}, d)))) : null,
    el('button', { type: 'button', onclick: () => { box.hidden = true; } }, 'Hide'));
}

async function undo() {
  const r = await api('/api/undo', { n: 1 });
  if (!r.ok) toast(r.data.error);
  else if (r.data.asked) note(`Asked ${r.data.asked} to agree to the Undo…`);
}

function setStatus(seed, seats) {
  const st = $('#status');
  st.textContent = `Seed ${seed}` + (two() ? ` · with ${otherName()}` : '');
  st.title = `Seats in turn order: ${seats.join(', ')}`;
}

let catalog = null, chosen = { deck: null, tier: 't3' };

window.addEventListener('resize', () => fitBoards(document.getElementById('table')));

function screen(name) {
  document.body.classList.toggle('playing', name === 'game');     // the game screen fits the window, no scrolling
  $('#setup').hidden = name !== 'setup';
  $('#game').hidden = name !== 'game';
  $('#review').hidden = name !== 'review';
  $('#lobby').hidden = name !== 'lobby';
  $('#join').hidden = name !== 'join';
  if (name !== 'lobby' && name !== 'join') clearTimeout(pollTimer);
  if (name !== 'game') $('#playback').hidden = true;
  $('#to-setup').hidden = name !== 'game' || !me.host;
  $('#undo').hidden = name !== 'game' || !tools.undo;
  $('#save').hidden = name === 'setup' || name === 'lobby' || name === 'join' || !me.host;
  if (name === 'setup') listSaves();
  if (name === 'game') requestAnimationFrame(() => fitBoards($('#table')));    // hidden areas measure as zero
  $('#hint').hidden = name !== 'game' || !tools.hint;
  $('#autopass-box').hidden = name !== 'game';
}

function cardImage(name, cls = '') {
  const files = (catalog && catalog.images[name]) || images[name];
  return files ? el('img', { src: `/images/${files[0]}`, alt: name, class: cls, loading: 'lazy' }) : el('span', { class: 'noimg ' + cls });
}

function pct(x) { return x == null ? '–' : `${x.toFixed(1)}%`; }

function renderSetup() {
  const deck = catalog.decks.find((d) => d.key === chosen.deck);
  $('#decks').replaceChildren(...catalog.decks.map((d) => el('button', {
    type: 'button', class: 'deck', role: 'radio', 'aria-checked': String(d.key === chosen.deck),
    onclick: () => { chosen.deck = d.key; renderSetup(); },
  }, cardImage(d.commander), el('span', {}, el('b', {}, d.name),
    el('small', {}, `Bracket ${d.bracket} · ${d.game_changers.length} Game Changers`),
    el('small', {}, d.average == null ? 'No simulation results yet' : `Simulations: ${pct(d.average)} average`)))));
  $('#tiers').replaceChildren(...catalog.tiers.map((t) => el('button', {
    type: 'button', class: 'tier', role: 'radio', 'aria-checked': String(t.key === chosen.tier),
    onclick: () => { chosen.tier = t.key; renderSetup(); },
  }, el('span', {}, el('b', {}, t.label)),
    el('span', { class: 'names' }, t.decks.map((x) => el('span', { title: x.commander }, cardImage(x.commander), x.name))),
    el('span', { class: 'rate' }, deck && deck.win_rates ? pct(deck.win_rates[t.key]) : '', el('small', {}, deck && deck.win_rates ? 'your win rate in sims' : '')))));
  const tier = catalog.tiers.find((t) => t.key === chosen.tier);
  const prev = new Set([...document.querySelectorAll('#picks input:checked')].map((b) => b.value));
  $('#picks').replaceChildren(...tier.decks.map((x) => el('label', {},
    el('input', Object.assign({ type: 'checkbox', value: x.key }, prev.has(x.key) ? { checked: '' } : {})), ' ', x.name)));
  $('#picks').hidden = $('#newgame').opp.value !== 'pick';
  const f = $('#newgame');
  if (catalog.app) { f.players.value = 'one'; $('#players-pick').hidden = true; }   // the iPad app: one player only
  const pair = f.players.value === 'two';
  for (const x of document.querySelectorAll('.opp-n')) x.textContent = pair ? 'Two' : 'Three';
  for (const x of document.querySelectorAll('.opp-n-lc')) x.textContent = pair ? 'two' : 'three';
  document.querySelector('.seat-pick').hidden = pair;
  const lanNote = $('#lan-note');
  lanNote.hidden = !pair;
  lanNote.textContent = catalog.lan
    ? 'You pick your deck here; your friend picks theirs (a different one) when they join. Seats are drawn at random. Undo and Try it ask the other person first.'
    : 'To play with a friend, stop the server and start it again with --lan:  python3 -m commander_sim.play --lan';
  f.querySelector('button[type=submit]').textContent = pair ? 'Open the table for your friend' : 'Start game';
  f.querySelector('button[type=submit]').disabled = pair && !catalog.lan;
}

async function startGame(e) {
  e.preventDefault();
  const f = $('#newgame');
  const body = { deck: chosen.deck, tier: chosen.tier, ai: f.ai.value, profile: f.profile.value,
    tools: { hint: f.hint.checked, undo: f.undo.checked, compare: f.compare.checked }, autopass: f.autopass.value };
  const pair = f.players.value === 'two';
  if (f.seed.value) body.seed = +f.seed.value;
  if (f.seat.value && !pair) body.seat = +f.seat.value;
  if (f.opp.value === 'pick') {
    body.opponents = [...document.querySelectorAll('#picks input:checked')].map((b) => b.value);
    const want = pair ? 2 : 3;
    if (body.opponents.length !== want) { $('#setup-error').textContent = `Pick exactly ${want === 2 ? 'two' : 'three'} opponents.`; return; }
  }
  $('#setup-error').textContent = '';
  if (pair) {
    body.two = true;
    const r = await api('/api/new', body);
    if (!r.ok) { $('#setup-error').textContent = r.data.error; return; }
    $('#log').replaceChildren(); logQueue = []; pending = null; inbox = []; skipping = false; renderTable(null); renderPrompt(null);
    await showLobby();
    return;
  }
  $('#log').replaceChildren(); logQueue = []; pending = null; inbox = []; skipping = false; renderTable(null); renderPrompt(null); loading(0, 0);
  const r = await api('/api/new', body);
  if (!r.ok) { loading(null); $('#setup-error').textContent = r.data.error; return; }
  await enterGame();
}

// card size: A- / A+ in the header, remembered in this browser
let zoom = 1;
function setZoom(z) {
  zoom = Math.min(2.2, Math.max(0.6, Math.round(z * 100) / 100));
  document.documentElement.style.setProperty('--zoom', zoom);
  fitBoards(document.getElementById('table'));
  try { localStorage.setItem('cardZoom', String(zoom)); } catch (e) { /* private window: not remembered */ }
}

async function init() {
  try { setZoom(parseFloat(localStorage.getItem('cardZoom')) || 1); } catch (e) { setZoom(1); }
  for (const b of document.querySelectorAll('#zoom button')) b.addEventListener('click', () => setZoom(zoom + 0.15 * +b.dataset.zoom));
  catalog = (await api('/api/options')).data;
  chosen.deck = catalog.decks[0].key;
  const f = $('#newgame');
  f.addEventListener('submit', startGame);
  for (const r of f.opp) r.addEventListener('change', renderSetup);
  for (const r of f.players) r.addEventListener('change', renderSetup);
  $('#to-setup').addEventListener('click', () => screen('setup'));
  $('#table').addEventListener('click', onTableClick);
  $('#undo').addEventListener('click', undo);
  $('#save').addEventListener('click', saveGame);
  listSaves();
  $('#hint').addEventListener('click', hint);
  $('#autopass').addEventListener('change', changeAutopass);
  setupPlayback();
  (passBtn = passBtn || $('#pass')).addEventListener('click', () => answer({ do: 'pass' }));
  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') closeMenu();
    // Space passes priority (when the button is showing and you aren't typing or using a menu)
    if (e.key === ' ' && pending && pending.request.kind === 'priority' && !$('#game').hidden
        && !(e.target instanceof Element && e.target.closest('input, select, textarea, button, #menu'))) {
      e.preventDefault(); answer({ do: 'pass' });
    }
  });
  document.addEventListener('click', (e) => { if (!e.target.closest('#menu') && !e.target.closest('#table')) closeMenu(); });
  renderSetup();
  if (Object.keys(catalog.images).length < catalog.decks.length) {      // the commanders' images arrive shortly after start-up
    setTimeout(async () => { catalog = (await api('/api/options')).data; renderSetup(); }, 6000);
  }
  const st = (await api('/api/state')).data;
  images = st.images || {};
  liveFrom = st.last_event || 0;
  me.host = st.host;
  if (st.game) {
    setMe(st);
    setStatus(st.game.seed, st.game.seats); setTools(st.game.tools); setAutopass(st.game.autopass);
    renderTable(st.view); screen('game');
    if (st.proposal) showProposal(st.proposal);
  } else await route(st);
  connect();
}
init();
