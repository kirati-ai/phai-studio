// คู่มือ Phai Studio - the sidebar: folding groups (remembered), putting it away and back,
// and a search box over its links. Works with storage blocked too (a file opened from disk,
// a private window): it then just forgets between pages.
(function () {
  'use strict';
  var KEY_CLOSED = 'phai-help-closed';   // the groups folded, by their data-key
  var KEY_HIDDEN = 'phai-help-hidden';   // the sidebar put away (wide screens)
  var nav = document.getElementById('nav');
  if (!nav) return;

  function load(key, fallback) {
    try { var v = localStorage.getItem(key); return v === null ? fallback : JSON.parse(v); } catch (e) { return fallback; }
  }
  function save(key, value) {
    try { localStorage.setItem(key, JSON.stringify(value)); } catch (e) { /* not remembered */ }
  }

  // --- folding groups ---------------------------------------------------------------------
  var groups = Array.prototype.slice.call(nav.querySelectorAll('details.grp'));
  var closed = load(KEY_CLOSED, []);
  var here = nav.querySelector('a.here');
  groups.forEach(function (d) {
    var holdsHere = here && d.contains(here);
    // The page's own group is always open; the rest as they were left.
    d.open = holdsHere || closed.indexOf(d.dataset.key) < 0;
    d.addEventListener('toggle', function () {
      if (searching) return;
      var now = load(KEY_CLOSED, []).filter(function (k) { return k !== d.dataset.key; });
      if (!d.open) now.push(d.dataset.key);
      save(KEY_CLOSED, now);
    });
  });
  // The current page's link in the middle of the sidebar - the sidebar alone scrolled, up and down
  // only (scrollIntoView also moved a phone's hidden drawer sideways).
  if (here) nav.scrollTop = here.getBoundingClientRect().top - nav.getBoundingClientRect().top - nav.clientHeight / 2;

  // --- putting the sidebar away -----------------------------------------------------------
  var narrow = window.matchMedia('(max-width: 860px)');
  function setHidden(hidden) {
    if (narrow.matches) {
      document.body.classList.toggle('nav-drawn', !hidden);
    } else {
      document.documentElement.classList.toggle('nav-hidden', hidden);
      save(KEY_HIDDEN, hidden);
    }
  }
  // (the page's head already put a remembered nav-hidden on <html>, before the first paint)
  requestAnimationFrame(function () { document.documentElement.classList.add('nav-ready'); });
  nav.querySelector('.nav-close').addEventListener('click', function () { setHidden(true); });
  document.querySelector('.nav-open').addEventListener('click', function () { setHidden(false); });
  document.querySelector('.nav-shade').addEventListener('click', function () { setHidden(true); });
  document.addEventListener('keydown', function (e) {
    if (e.key === 'Escape' && document.body.classList.contains('nav-drawn')) setHidden(true);
  });

  // --- search -----------------------------------------------------------------------------
  var searching = false;
  var box = nav.querySelector('.nav-search');
  var none = nav.querySelector('.nav-none');
  var before = null;
  box.addEventListener('input', function () {
    var q = box.value.trim().toLowerCase();
    if (q && !searching) {
      searching = true;
      before = groups.map(function (d) { return d.open; });
    }
    var any = false;
    nav.querySelectorAll('.nav-body a').forEach(function (a) {
      var hit = !q || a.textContent.toLowerCase().indexOf(q) >= 0;
      a.hidden = !hit;
      any = any || hit;
    });
    groups.forEach(function (d, i) {
      var shown = d.querySelectorAll('a:not([hidden])').length > 0;
      d.hidden = q && !shown;
      if (q) d.open = shown; else if (before) d.open = before[i];
    });
    none.hidden = any;
    if (!q) { searching = false; before = null; }
  });
  box.addEventListener('keydown', function (e) {
    if (e.key === 'Enter') {
      var first = nav.querySelector('.nav-body a:not([hidden])');
      if (first) location.href = first.getAttribute('href');
    }
    if (e.key === 'Escape') { box.value = ''; box.dispatchEvent(new Event('input')); }
  });
})();
