// phai-studio.monrak.net: the page's motion, after monrak.net and desktopserver.monrak.net, without libraries.
// The slider and its gallery, every card's hover, cards fading in, and the strip that says how far down you are.

// slider: moves on by itself every few seconds, and by the arrows or a swipe; the panel beside it
// (its number, name, line and points) follows.
// It waits while the pointer or focus is on it, while it is off screen or the tab hidden, and never
// moves by itself for a reader who asked for less motion. It never scrolls the page.
document.querySelectorAll('.gallery').forEach((gal) => {
    const box = gal.querySelector('.slider');
    if (!box) return;
    const track = box.querySelector('.slides');
    const slides = [...track.querySelectorAll('.slide')];
    const cap = gal.querySelector('.shot-info');
    const n = slides.length;
    const EVERY = 5000;
    let at = 0;
    const warm = (i) => { const img = slides[(i + n) % n].querySelector('img'); if (img) img.loading = 'eager'; };
    const go = (i) => {
        at = (i + n) % n;
        warm(at); warm(at + 1);
        track.style.transform = 'translateX(' + (-100 * at) + '%)';
        const s = slides[at];
        const pad = (k) => String(k).padStart(2, '0');
        cap.querySelector('.shot-n b').textContent = pad(at + 1);
        cap.querySelector('h3').textContent = s.dataset.h;
        cap.querySelector('p').textContent = s.dataset.p;
        const ul = cap.querySelector('ul');
        ul.innerHTML = '';
        for (const m of JSON.parse(s.dataset.more || '[]')) { const li = document.createElement('li'); li.textContent = m; ul.append(li); }
        cap.classList.remove('fresh'); void cap.offsetWidth; cap.classList.add('fresh');
        if (window.thaiWrap) window.thaiWrap(cap);
    };
    warm(1);
    const still = window.matchMedia('(prefers-reduced-motion: reduce)');
    let held = false, seen = false, timer = 0;
    const start = () => {
        clearInterval(timer);
        if (!still.matches && !held && seen && !document.hidden) timer = setInterval(() => go(at + 1), EVERY);
    };
    box.querySelector('.sl-prev').onclick = () => { go(at - 1); start(); };
    box.querySelector('.sl-next').onclick = () => { go(at + 1); start(); };
    box.addEventListener('mouseenter', () => { held = true; start(); });
    box.addEventListener('mouseleave', () => { held = false; start(); });
    box.addEventListener('focusin', () => { held = true; start(); });
    box.addEventListener('focusout', () => { held = false; start(); });
    document.addEventListener('visibilitychange', start);
    new IntersectionObserver((es) => { seen = es[0].isIntersecting; start(); }, { threshold: 0.3 }).observe(box);
    let x0 = null;
    box.addEventListener('touchstart', (e) => { x0 = e.touches[0].clientX; }, { passive: true });
    box.addEventListener('touchend', (e) => {
        const dx = e.changedTouches[0].clientX - x0;
        if (Math.abs(dx) > 40) { go(at + (dx < 0 ? 1 : -1)); start(); }
    });
});

// gallery: a picture opens large; arrows or keys move, Esc or a click outside closes
(() => {
    const all = [...document.querySelectorAll('[data-gallery]')];
    const box = document.getElementById('lightbox');
    if (!box || !all.length) return;
    let links = all;
    const img = box.querySelector('img');
    const cap = box.querySelector('.lb-caption');
    const stage = box.querySelector('.lb-stage');
    let at = 0;
    const show = (i) => {
        at = (i + links.length) % links.length;
        img.src = links[at].href;
        img.alt = cap.textContent = links[at].dataset.caption + '  (' + (at + 1) + ' / ' + links.length + ')';
    };
    const close = () => { box.hidden = true; document.body.style.overflow = ''; };
    all.forEach((a) => a.addEventListener('click', (e) => {
        e.preventDefault();
        links = all.filter((x) => x.dataset.gallery === a.dataset.gallery);
        show(links.indexOf(a));
        box.hidden = false;
        document.body.style.overflow = 'hidden';
    }));
    box.querySelector('.lb-close').onclick = close;
    box.querySelector('.lb-prev').onclick = () => show(at - 1);
    box.querySelector('.lb-next').onclick = () => show(at + 1);
    box.addEventListener('click', (e) => { if (e.target === box || e.target === stage) close(); });
    document.addEventListener('keydown', (e) => {
        if (box.hidden) return;
        if (e.key === 'Escape') close();
        if (e.key === 'ArrowLeft') show(at - 1);
        if (e.key === 'ArrowRight') show(at + 1);
    });
})();

// every card: a spotlight that follows the pointer and a tilt of at most 5 degrees
(() => {
    if (!matchMedia('(hover: hover)').matches) return;
    const calm = matchMedia('(prefers-reduced-motion: reduce)').matches;
    document.querySelectorAll('.card').forEach((el) => {
        el.addEventListener('pointermove', (e) => {
            const r = el.getBoundingClientRect();
            const x = (e.clientX - r.left) / r.width, y = (e.clientY - r.top) / r.height;
            el.style.setProperty('--mx', (x * 100).toFixed(1) + '%');
            el.style.setProperty('--my', (y * 100).toFixed(1) + '%');
            if (!calm) {
                el.style.setProperty('--rx', ((0.5 - y) * 5).toFixed(2) + 'deg');
                el.style.setProperty('--ry', ((x - 0.5) * 5).toFixed(2) + 'deg');
            }
        });
        el.addEventListener('pointerleave', () => { el.style.setProperty('--rx', '0deg'); el.style.setProperty('--ry', '0deg'); });
    });
})();

// cards fade in once; headings drift a little with the scrollbar; the strip shows how far down you are
(() => {
    const calm = matchMedia('(prefers-reduced-motion: reduce)').matches;
    const cards = [...document.querySelectorAll('.card, .slider')];
    const pct = document.getElementById('hud-pct');
    const text = document.getElementById('hud-text');
    const words = window.HUD_WORDS || [];
    const items = [];
    if (!calm) {
        // cards only fade and rise in once: they never drift apart, so a row stays a straight row
        cards.forEach((el) => el.classList.add('reveal'));
        document.querySelectorAll('.section h2, .section .sub, .section .label, .hero h1, .hero .lead').forEach((el, i) => {
            el.classList.add('glide');
            items.push({ el, depth: [0.5, 0.8][i % 2], amp: 36, now: 0 });
        });
        const seen = new IntersectionObserver((list) => list.forEach((e) => {
            if (e.isIntersecting) { e.target.classList.add('shown'); seen.unobserve(e.target); }
        }), { rootMargin: '0px 0px -8% 0px' });
        cards.forEach((el) => seen.observe(el));
    }
    let running = false;
    const step = () => {
        const vh = innerHeight;
        let moving = false;
        for (const it of items) {
            const r = it.el.getBoundingClientRect();
            if (r.bottom < -200 || r.top > vh + 200) continue;
            const target = ((r.top + r.height / 2) / vh - 0.5) * it.amp * it.depth;
            it.now += (target - it.now) * 0.1;
            if (Math.abs(target - it.now) > 0.2) moving = true;
            it.el.style.setProperty('--drift', it.now.toFixed(2) + 'px');
        }
        const max = document.documentElement.scrollHeight - vh;
        const done = max > 0 ? Math.min(1, scrollY / max) : 0;
        if (pct) pct.textContent = Math.round(done * 100) + '%';
        if (text && words.length) text.textContent = words[Math.min(words.length - 1, Math.floor(done * words.length))];
        running = moving;
        if (moving) requestAnimationFrame(step);
    };
    const kick = () => { if (!running) { running = true; requestAnimationFrame(step); } };
    addEventListener('scroll', kick, { passive: true });
    addEventListener('resize', kick);
    kick();
})();
