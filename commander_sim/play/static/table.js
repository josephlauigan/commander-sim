// The table (step 2d): opponents across the top, your battlefield in the middle (creatures and other permanents,
// then lands), and your hand, library, graveyard, exile and command zone along the bottom. Cards show their
// Scryfall image when it's cached (drawn from the name otherwise); tapped cards turn sideways; counters, power and
// toughness and loyalty sit on the card. Hovering a card shows it large.

export function el(tag, attrs = {}, ...kids) {
  const e = document.createElement(tag);
  for (const [k, v] of Object.entries(attrs)) {
    if (v === false || v == null) continue;
    if (k === 'class') e.className = v; else if (k.startsWith('on')) e.addEventListener(k.slice(2), v); else e.setAttribute(k, v);
  }
  for (const k of kids.flat()) if (k != null && k !== false) e.append(k.nodeType ? k : String(k));
  return e;
}

export const STEPS = [['start', 'Beginning'], ['main1', 'Main 1'], ['combat', 'Combat'], ['main2', 'Main 2'], ['end', 'End']];

function filesFor(images, name) {
  if (!images) return null;
  name = name.replace(/( \{[^}]*\})+$/, '');                  // a label with its mana cost ("Sol Ring {1}")
  return images[name] || images[name.replace(/ token$/, '')] || images[name.split(' // ')[0]] || null;
}
function imageFor(images, name, face = 0) {                  // face 1: a two-faced card's back (Sanguine Morass)
  const files = filesFor(images, name);
  return files ? `/images/${files[Math.min(face, files.length - 1)]}` : null;
}

// ------------------------------------------------------------------ hover to enlarge
let preview = null;
function showPreview(src, name, e, more = []) {
  if (!preview) { preview = el('div', { id: 'preview', 'aria-hidden': 'true' }); document.body.append(preview); }
  const srcs = src ? [src, ...more.filter((s) => s !== src)] : [];  // a two-faced card: both faces, side by side
  preview.classList.toggle('two', srcs.length > 1);
  preview.replaceChildren(...(srcs.length ? srcs.map((s) => el('img', { src: s, alt: '' })) : [el('div', { class: 'drawn big' }, name)]));
  const right = e.clientX < window.innerWidth / 2, w = srcs.length > 1 ? 608 : 300;
  preview.style.left = right ? `${Math.min(e.clientX + 24, window.innerWidth - w - 20)}px` : `${Math.max(8, e.clientX - w - 24)}px`;
  preview.style.top = `${Math.max(8, Math.min(e.clientY - 200, window.innerHeight - 440))}px`;
  preview.hidden = false;
}
function hidePreview() { if (preview) preview.hidden = true; }

// ------------------------------------------------------------------ images kept between redraws
// The table is redrawn after every action. New <img> elements would blank out until the browser decoded them again
// (cards blinking in and out during playback), so the loaded images of the last drawing are reused.
let pool = new Map();
function takeImg(src, name) {
  const free = pool.get(src);
  if (free && free.length) { const img = free.pop(); img.alt = name; return img; }
  return el('img', { src, alt: name, draggable: 'false', decoding: 'sync' });
}
function collect(root) {
  pool = new Map();
  for (const img of root.querySelectorAll('img')) {
    const src = img.getAttribute('src');
    if (!pool.has(src)) pool.set(src, []);
    pool.get(src).push(img);
  }
}

// ------------------------------------------------------------------ one card
export function card(images, name, o = {}) {
  const src = imageFor(images, name, o.face || 0);
  const all = (filesFor(images, name) || []).map((f) => `/images/${f}`);
  const face = src ? takeImg(src, name) : el('div', { class: 'drawn' }, name);
  const badges = [];
  if (o.counters) badges.push(el('span', { class: 'badge counters', title: `${o.counters} +1/+1 counters` }, `+${o.counters}`));
  if (o.loyalty != null) badges.push(el('span', { class: 'badge loyalty', title: 'loyalty' }, o.loyalty));
  if (o.pt) badges.push(el('span', { class: 'badge pt' }, o.pt));
  if (o.count > 1) badges.push(el('span', { class: 'badge count' }, `×${o.count}`));
  const cls = ['card', o.size || 'md', o.tapped ? 'tapped' : '', o.sick ? 'sick' : '', o.commander ? 'cmdr' : '', o.cls || ''].join(' ');
  const title = [name, o.pt, o.tapped ? 'tapped' : '', o.sick ? 'summoning sick' : '', o.attached_to ? `attached to ${o.attached_to}` : '']
    .filter(Boolean).join(' · ');
  const c = el('div', { class: cls, title, 'data-name': name, ...(o.attrs || {}) }, el('div', { class: 'face' }, face, badges));
  c.addEventListener('mouseenter', (e) => showPreview(src, name, e, all));
  c.addEventListener('mouseleave', hidePreview);
  return c;
}

// lands of one name together (Island ×4), tapped ones apart from untapped
function groupLands(lands) {
  const groups = new Map();
  for (const L of lands) {
    const k = `${L.name}|${L.tapped}`;
    if (!groups.has(k)) groups.set(k, { ...L, count: 0, all: [] });
    const g = groups.get(k); g.count += 1; g.all.push(L);
  }
  return [...groups.values()];
}

function zoneList(title, names, images, attrs = () => ({})) {
  return el('details', { class: 'zonelist' }, el('summary', {}, `${title} (${names.length})`),
    names.length ? el('div', { class: 'cards' }, names.map((n, i) => card(images, n, { size: 'sm', attrs: attrs(i) }))) : el('em', {}, 'empty'));
}

function stats(p, mine) {
  return el('div', { class: 'stats' },
    el('span', { class: 'life', title: 'life' }, `♥ ${p.life}`),
    p.poison ? el('span', { title: 'poison counters' }, `☠ ${p.poison}`) : null,
    el('span', { title: 'cards in hand' }, `✋ ${p.hand_count}`),
    el('span', { title: 'cards in library' }, `📚 ${p.library}`),
    p.treasures ? el('span', { title: 'Treasures', class: mine ? 'clickable' : '', 'data-treasure': mine ? '1' : null },
      `◆ ${p.treasures} Treasure`) : null,
    Object.entries(p.commander_damage || {}).map(([who, n]) => el('span', { title: `commander damage from ${who}` }, `⚔ ${who} ${n}`)));
}

function battlefield(p, images, size, mine) {
  const perms = p.battlefield.filter((m) => !m.phased_out);
  const creatures = perms.filter((m) => m.pt), others = perms.filter((m) => !m.pt);
  const opts = (m) => ({ size, tapped: m.tapped, sick: m.sick, counters: m.counters, loyalty: m.loyalty, pt: m.pt,
    commander: m.commander, attached_to: m.attached_to, attrs: Object.assign({ 'data-seat': p.key, 'data-i': m.i }, mine ? { 'data-perm': m.i } : {}) });
  const treasure = p.treasures ? card(images, 'Treasure', { size, count: p.treasures, cls: 'token',
    attrs: mine ? { 'data-treasure': '1' } : {} }) : null;              // Treasures, one card with a count
  return [
    el('div', { class: 'row perms' }, creatures.map((m) => card(images, m.name, opts(m))),
      creatures.length && (others.length || treasure) ? el('span', { class: 'gap' }) : null,
      others.map((m) => card(images, m.name, opts(m))), treasure),
    el('div', { class: 'row lands' }, mine
      ? p.lands.map((L) => card(images, L.name, { size: 'sm', tapped: L.tapped, face: L.face, cls: 'land', attrs: { 'data-land': L.i } }))
      : groupLands(p.lands).map((g) => card(images, g.name, { size: 'sm', tapped: g.tapped, face: g.face, count: g.count, cls: 'land' }))),
  ];
}

function opponent(p, view, images) {
  return el('section', { class: 'opp' + (p.key === view.active ? ' active' : '') + (p.alive ? '' : ' out'), 'aria-label': p.name, 'data-player': p.key },
    el('header', {}, card(images, p.commander, { size: 'sm', cls: 'portrait' }), el('div', {}, el('h2', {}, p.name), stats(p))),
    el('div', { class: 'opp-board', 'data-fit': '1' }, battlefield(p, images, 'md', false)),
    el('footer', {}, zoneList('Graveyard', p.graveyard, images), p.exile.length ? zoneList('Exile', p.exile, images) : null,
      p.commander_in_zone ? el('span', { class: 'chip' }, `Commander in zone (tax ${p.tax})`) : null));
}

export function renderTable(root, view, images) {
  collect(root);
  root.replaceChildren();
  if (!view) { pool = new Map(); return; }
  const me = view.players.find((p) => p.you);
  const opps = view.players.filter((p) => !p.you);
  root.append(el('div', { class: 'opps' }, opps.map((p) => opponent(p, view, images))));
  root.append(el('section', { class: 'mine' + (me.key === view.active ? ' active' : ''), 'aria-label': 'Your battlefield', 'data-player': me.key },
    el('header', {}, el('h2', {}, `${me.name} (you)`), stats(me, true),
      el('span', { class: 'pool', title: 'your mana pool' }, `Mana pool: ${me.mana_pool}`)),
    el('div', { class: 'mine-board', 'data-fit': '1' }, battlefield(me, images, 'md', true))));
  root.append(el('div', { class: 'passbar' }));          // the Pass priority button goes here (app.js)
  root.append(el('section', { class: 'bottom', 'aria-label': 'Your hand and zones' },
    el('div', { class: 'hand' }, el('h3', {}, `Hand (${me.hand.length})`),
      el('div', { class: 'cards', 'data-fit': '1' }, me.hand.map((n, i) => card(images, n, { size: 'lg', attrs: { 'data-hand': i } })))),
    el('div', { class: 'zones' },
      me.commander_in_zone ? el('div', { class: 'cz' }, el('h3', {}, `Command zone (tax ${me.tax})`),
        card(images, me.commander, { size: 'sm', commander: true, attrs: { 'data-cmd': '1' } })) : null,
      el('div', { class: 'chip' }, `Library ${me.library}`),
      zoneList('Graveyard', me.graveyard, images, (i) => ({ 'data-gy': i })), zoneList('Exile', me.exile, images))));
  pool = new Map();
}

export function renderSteps(root, view) {
  root.replaceChildren();
  if (!view) return;
  const active = view.players.find((p) => p.key === view.active);
  root.append(el('span', { class: 'whose' }, active ? (active.you ? 'Your turn' : `${active.name}'s turn`) : ''),
    ...STEPS.map(([k, label]) => el('span', { class: 'step' + (k === view.step ? ' now' : '') }, label)),
    el('span', { class: 'round' }, `Round ${view.round}`));
}

// Board areas and card sizes. Each area's cards shrink until they fit its height (an area never grows; its cards get
// smaller). On the window-sized game screen the opponents' row, your battlefield and your hand also share the
// height: the largest card scale at which all three fit together is found, each area gets the height its cards need
// at that scale, and then each area enlarges its own cards as far as its height allows.
function contentHeight(box) {
  const top = box.getBoundingClientRect().top;
  let bottom = top;
  for (const c of box.children) bottom = Math.max(bottom, c.getBoundingClientRect().bottom);
  return bottom - top + 4;
}

function shareHeight(root) {
  const areas = ['.opps', '.mine', '.bottom'].map((sel) => {
    const el = root.querySelector(sel);
    return el && { el, boards: [...el.querySelectorAll('[data-fit]')] };
  }).filter((a) => a && a.boards.length);
  if (areas.length < 3) return;
  for (const a of areas) { a.el.style.flexGrow = 1; for (const b of a.boards) b.style.setProperty('--fit', 1); }
  const avail = areas.reduce((t, a) => t + a.el.getBoundingClientRect().height, 0);
  for (const a of areas) a.frame = a.el.getBoundingClientRect().height - a.boards[0].getBoundingClientRect().height;
  const needAt = (f) => areas.map((a) => {
    for (const b of a.boards) b.style.setProperty('--fit', f);
    return a.frame + Math.max(...a.boards.map(contentHeight));
  });
  let lo = 0.25, hi = 1, best = needAt(0.25);
  const all = needAt(1);
  if (all.reduce((t, x) => t + x, 0) <= avail) { lo = 1; best = all; }
  else {
    for (let i = 0; i < 8; i++) {                 // the largest common scale that fits
      const mid = (lo + hi) / 2, n = needAt(mid);
      if (n.reduce((t, x) => t + x, 0) <= avail) { lo = mid; best = n; } else hi = mid;
    }
  }
  areas.forEach((a, k) => { a.el.style.flexGrow = Math.max(1, best[k]); });
}

export function fitBoards(root = document) {
  if (!root) return;
  if (document.body.classList.contains('playing')) shareHeight(root);
  for (const box of root.querySelectorAll('[data-fit]')) {
    let fit = 1;
    box.style.setProperty('--fit', fit);
    for (let i = 0; i < 16 && box.scrollHeight > box.clientHeight + 1 && fit > 0.3; i++) {
      fit *= 0.92;
      box.style.setProperty('--fit', fit.toFixed(3));
    }
  }
}
