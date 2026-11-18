// phai-studio.monrak.net: the page's motion, after monrak.net and desktopserver.monrak.net, without libraries.
// The slider and its gallery, every card's hover, cards fading in, and the strip that says how far down you are.

// slider: moves only when asked (arrows, thumbnails, a swipe); thumbnails and caption follow.
// It never moves by itself or scrolls the page: only the thumbnail strip scrolls, to show the chosen one.
(() => {
    const box = document.querySelector('.slider');
    if (!box) return;
    const track = box.querySelector('.slides');
    const thumbs = [...document.querySelectorAll('.thumbs button')];
    const cap = document.getElementById('shot-caption');
    const n = thumbs.length;
    const strip = document.querySelector('.thumbs');
    let at = 0;
    const go = (i) => {
        at = (i + n) % n;
        track.style.transform = 'translateX(' + (-100 * at) + '%)';
        thumbs.forEach((t, k) => t.classList.toggle('on', k === at));
        const t = thumbs[at];
        cap.innerHTML = '';
        const b = document.createElement('b');
        b.textContent = t.dataset.captionH;
        cap.append(b, ' — ' + t.dataset.captionP);
        if (window.thaiWrap) window.thaiWrap(cap);
        const l = t.offsetLeft - strip.offsetLeft, r = l + t.offsetWidth;
        if (l < strip.scrollLeft) strip.scrollTo({ left: l - 8, behavior: 'smooth' });
        else if (r > strip.scrollLeft + strip.clientWidth) strip.scrollTo({ left: r - strip.clientWidth + 8, behavior: 'smooth' });
    };
    box.querySelector('.sl-prev').onclick = () => go(at - 1);
    box.querySelector('.sl-next').onclick = () => go(at + 1);
    thumbs.forEach((t, k) => t.onclick = () => go(k));
    let x0 = null;
    box.addEventListener('touchstart', (e) => { x0 = e.touches[0].clientX; }, { passive: true });
    box.addEventListener('touchend', (e) => {
        const dx = e.changedTouches[0].clientX - x0;
        if (Math.abs(dx) > 40) go(at + (dx < 0 ? 1 : -1));
    });
})();

// gallery: a picture opens large; arrows or keys move, Esc or a click outside closes
(() => {
    const links = [...document.querySelectorAll('[data-gallery]')];
    const box = document.getElementById('lightbox');
    if (!box || !links.length) return;
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
    links.forEach((a, i) => a.addEventListener('click', (e) => {
        e.preventDefault();
        show(i);
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

// cards fade in once; cards and headings drift a little with the scrollbar; the strip shows how far down you are
(() => {
    const calm = matchMedia('(prefers-reduced-motion: reduce)').matches;
    const cards = [...document.querySelectorAll('.card, .slider')];
    const pct = document.getElementById('hud-pct');
    const text = document.getElementById('hud-text');
    const words = window.HUD_WORDS || [];
    const items = [];
    if (!calm) {
        cards.forEach((el, i) => { el.classList.add('reveal'); items.push({ el, depth: [0.6, 1, 1.4][i % 3], amp: 60, now: 0 }); });
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
