// Thai lines break only at the spaces between phrases. Browsers guess Thai word breaks from a
// dictionary and sometimes cut inside a word ("ต่อย|อดได้"); here each run of Thai between spaces
// is kept whole (span.nb, white-space: nowrap), and the CSS (text-wrap: pretty) keeps a lone
// word from falling onto the last line. Shared by monrak.net and the Phai family's sites.
(function () {
    var thai = /[฀-๿]/;
    var skip = { SCRIPT: 1, STYLE: 1, CODE: 1, PRE: 1, TEXTAREA: 1, INPUT: 1, SELECT: 1, OPTION: 1, BUTTON: 0 };
    function wrap(root) {
        var walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT, {
            acceptNode: function (n) {
                for (var p = n.parentNode; p && p !== root.parentNode; p = p.parentNode) {
                    if (skip[p.nodeName] || (p.classList && p.classList.contains('nb'))) return NodeFilter.FILTER_REJECT;
                }
                return thai.test(n.nodeValue) ? NodeFilter.FILTER_ACCEPT : NodeFilter.FILTER_REJECT;
            }
        });
        var nodes = [];
        while (walker.nextNode()) nodes.push(walker.currentNode);
        nodes.forEach(function (n) {
            var parts = n.nodeValue.split(/(\s+)/);
            if (parts.length < 2 && !thai.test(n.nodeValue)) return;
            var frag = document.createDocumentFragment();
            parts.forEach(function (part) {
                if (!part) return;
                if (/^\s+$/.test(part) || !thai.test(part)) {
                    frag.appendChild(document.createTextNode(part));
                } else {
                    var s = document.createElement('span');
                    s.className = 'nb';
                    s.textContent = part;
                    frag.appendChild(s);
                }
            });
            n.parentNode.replaceChild(frag, n);
        });
    }
    // A phrase wider than its line (a long one on a narrow phone) may break after all.
    function fit() {
        document.querySelectorAll('.nb').forEach(function (e) { e.style.whiteSpace = ''; });
        document.querySelectorAll('.nb').forEach(function (e) {
            var p = e.parentElement;
            while (p && getComputedStyle(p).display.indexOf('inline') === 0) p = p.parentElement;
            if (p && e.getBoundingClientRect().width > p.clientWidth) e.style.whiteSpace = 'normal';
        });
    }
    function run() { wrap(document.body); fit(); }
    var t;
    window.addEventListener('resize', function () { clearTimeout(t); t = setTimeout(fit, 150); });
    if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', run); else run();
    window.thaiWrap = wrap;
})();
