// The browser table: the setup screen, the loading screen, the table (table.js), the log, and your decisions.
import { el, card as cardOf, renderTable as drawTable, renderSteps } from './table.js';
const $ = (sel) => document.querySelector(sel);
let lastId = 0, pending = null, source = null;
let liveFrom = 0;           // events up to this id are history replayed on load: no pop-up messages for them
let images = {};            // card name -> image files in /images/ (the table uses them from step 2d)

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
function renderTable(view) {
  lastView = view;
  drawTable($('#table'), view, images);
  renderSteps($('#steps'), view);
}
const card = (name, o = {}) => cardOf(images, name, Object.assign({ size: 'sm' }, o));

// ------------------------------------------------------------------ decisions
async function answer(value) {
  if (!pending) return;
  const id = pending.id;
  pending = null; renderPrompt(null); markChoices(null);
  const r = await api('/api/answer', { id, answer: value });
  if (!r.ok) logLine(r.data.error, 'invalid');
}

const btn = (label, value, cls = '') => el('button', { class: cls, onclick: () => answer(value) }, label);

function you(view) { return view && view.players.find((p) => p.you); }

function renderPrompt(ev) {
  const box = $('#prompt'); box.replaceChildren(); closeMenu();
  $('#pass').hidden = !(ev && ev.request.kind === 'priority');
  $('#table').classList.toggle('can-act', !!(ev && ev.request.kind === 'priority'));
  if (!ev) { box.append(el('div', { class: 'stats' }, 'Waiting for the other players…')); return; }
  const req = ev.request;
  box.append(el('h3', {}, req.prompt));
  if (req.kind === 'priority') {
    const me = you(ev.view);
    if (req.data.stack && req.data.stack.length) box.append(el('div', { class: 'row' }, el('b', {}, 'On the stack'), req.data.stack.map((x) => card(x.name))));
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
    const boxes = req.choices.map((c, i) => el('label', {}, el('input', { type: 'checkbox', value: i }), ' ', c));
    box.append(el('div', { class: 'row' }, boxes));
    box.append(el('div', { class: 'row' },
      el('button', { class: 'primary', onclick: () => answer(boxes.map((b) => b.firstChild).filter((b) => b.checked).map((b) => +b.value)) }, 'Attack'),
      btn('No attack', [])));
  } else if (req.kind === 'mulligan') {
    box.append(el('div', { class: 'row' }, (req.data.hand || []).map((n) => card(n))));
    box.append(el('div', { class: 'row' }, btn('Keep', 'keep', 'primary'), btn('Mulligan', 'mulligan')));
  } else if (req.kind === 'continue') {
    box.append(el('div', { class: 'row' }, btn('Continue', true, 'primary')));
  } else {
    box.append(el('div', { class: 'row' }, req.choices.map((c, i) => btn(c, i))));
  }
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

function closeMenu() { const m = $('#menu'); if (m) m.remove(); }

function menu(e, title, items) {
  closeMenu();
  const m = el('div', { id: 'menu', role: 'menu' }, el('div', { class: 'title' }, title),
    items.map(([label, value]) => el('button', { role: 'menuitem', onclick: () => { closeMenu(); answer(value); } }, label)));
  document.body.append(m);
  const r = m.getBoundingClientRect();
  m.style.left = `${Math.min(e.clientX, window.innerWidth - r.width - 8)}px`;
  m.style.top = `${Math.min(e.clientY, window.innerHeight - r.height - 8)}px`;
  m.querySelector('button').focus();
}

function act(e, title, items, otherwise) {
  if (!items.length) { toast(otherwise); return; }
  if (items.length === 1) { answer(items[0][1]); return; }
  menu(e, title, items);
}

const tapItems = (srcs) => srcs.flatMap((x) => x.colours.length > 1
  ? [...x.colours].map((c) => [`Tap for {${c}}`, { do: 'tap', source: x.id, colour: c }])
  : [[`Tap for {${x.colours}}`, { do: 'tap', source: x.id }]]);

// a choice whose answer is on the table (a target, a card to discard): those cards and players light up and a click
// picks them; the list in the panel still works
function markChoices(ev) {
  for (const x of document.querySelectorAll('.targetable')) { x.classList.remove('targetable'); delete x.dataset.choice; }
  if (!ev || ev.request.kind === 'priority' || !ev.request.data.refs) return;
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
    const items = tapItems(srcs.filter((x) => x.land === i));
    if (L.ability) items.push(['Activate an ability', { do: 'use', land: i }]);
    act(e, name, items, L.tapped ? `${name} is tapped.` : `${name} can't be tapped for mana right now.`);
  } else if (t.dataset.perm !== undefined) {
    const i = +t.dataset.perm;
    const items = tapItems(srcs.filter((x) => x.perm === i));
    items.push(['Activate an ability', { do: 'use', perm: i }]);
    act(e, name, items, '');
  } else if (t.dataset.treasure !== undefined) {
    act(e, 'Treasure', tapItems(srcs.filter((x) => x.treasure).slice(0, 1)), 'No untapped Treasure.');
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
function logLine(text, cls = '') {
  const log = $('#log');
  const atEnd = log.scrollTop + log.clientHeight >= log.scrollHeight - 4;
  log.append(el('div', { class: cls }, text));
  if (atEnd) log.scrollTop = log.scrollHeight;
}

function onEvent(ev) {
  lastId = Math.max(lastId, ev.id);
  if (ev.view) renderTable(ev.view);
  if (ev.kind === 'log') logLine(ev.text, ev.text.startsWith('---') ? 'turn' : '');
  else if (ev.kind === 'invalid') { logLine('Not allowed: ' + ev.text, 'invalid'); if (ev.id > liveFrom) toast(ev.text); }
  else if (ev.kind === 'auto') logLine('(automatic) ' + ev.text, 'auto');
  else if (ev.kind === 'loading') loading(ev.done, ev.total);
  else if (ev.kind === 'images') { images = ev.images; loading(null); if (lastView) renderTable(lastView); }
  else if (ev.kind === 'request') { pending = ev; renderPrompt(ev); markChoices(ev); }
  else if (ev.kind === 'over') { pending = null; renderPrompt(null); $('#prompt').replaceChildren(el('h3', {}, `Game over: ${ev.winner || 'no winner'} (${ev.how})`)); }
  else if (ev.kind === 'error') logLine(ev.text, 'invalid');
}

function connect() {
  if (source) source.close();
  source = new EventSource(`/api/events?since=${lastId}`);
  source.onmessage = (m) => onEvent(JSON.parse(m.data));
}

// ------------------------------------------------------------------ setup
let catalog = null, chosen = { deck: null, tier: 't3' };

function screen(name) {
  $('#setup').hidden = name !== 'setup';
  $('#game').hidden = name !== 'game';
  $('#to-setup').hidden = name !== 'game';
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
}

async function startGame(e) {
  e.preventDefault();
  const f = $('#newgame');
  const body = { deck: chosen.deck, tier: chosen.tier, ai: f.ai.value, profile: f.profile.value,
    tools: { hint: f.hint.checked, undo: f.undo.checked, compare: f.compare.checked } };
  if (f.seed.value) body.seed = +f.seed.value;
  if (f.seat.value) body.seat = +f.seat.value;
  if (f.opp.value === 'pick') {
    body.opponents = [...document.querySelectorAll('#picks input:checked')].map((b) => b.value);
    if (body.opponents.length !== 3) { $('#setup-error').textContent = 'Pick exactly three opponents.'; return; }
  }
  $('#setup-error').textContent = '';
  $('#log').replaceChildren(); pending = null; renderTable(null); renderPrompt(null); loading(0, 0);
  const r = await api('/api/new', body);
  if (!r.ok) { loading(null); $('#setup-error').textContent = r.data.error; return; }
  $('#status').textContent = `Seed ${r.data.seed} · seats: ${r.data.seats.join(', ')}`;
  screen('game');
}

async function init() {
  catalog = (await api('/api/options')).data;
  chosen.deck = catalog.decks[0].key;
  const f = $('#newgame');
  f.addEventListener('submit', startGame);
  for (const r of f.opp) r.addEventListener('change', renderSetup);
  $('#to-setup').addEventListener('click', () => screen('setup'));
  $('#table').addEventListener('click', onTableClick);
  $('#pass').addEventListener('click', () => answer({ do: 'pass' }));
  document.addEventListener('keydown', (e) => { if (e.key === 'Escape') closeMenu(); });
  document.addEventListener('click', (e) => { if (!e.target.closest('#menu') && !e.target.closest('#table')) closeMenu(); });
  renderSetup();
  if (Object.keys(catalog.images).length < catalog.decks.length) {      // the commanders' images arrive shortly after start-up
    setTimeout(async () => { catalog = (await api('/api/options')).data; renderSetup(); }, 6000);
  }
  const st = (await api('/api/state')).data;
  images = st.images || {};
  liveFrom = st.last_event || 0;
  if (st.game) {
    $('#status').textContent = `Seed ${st.game.seed} · seats: ${st.game.seats.join(', ')}`;
    renderTable(st.view); screen('game');
  } else screen('setup');
  connect();
}
init();
