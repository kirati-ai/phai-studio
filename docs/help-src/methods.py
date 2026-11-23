# The methods a control's code can call, read from phai-ui's source so the guide never lists one
# that is not there: the ones every control has (the `control!` macro in ctl.rs), its type's own
# (`impl <Type> { pub fn ... }` in ctl.rs, components.rs, binding.rs, print.rs), and TextEdit's for
# the text boxes. Each with the `///` lines above it.
import os, re

SRC = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', '..', 'crates', 'phai-ui', 'src')
FILES = ['ctl.rs', 'components.rs', 'binding.rs', 'print.rs']


def _read(name):
    return open(os.path.join(SRC, name), encoding='utf-8').read().split('\n')


def _block_end(lines, start):
    """The line where the block whose `{` is on line `start` closes."""
    depth = 0
    for i in range(start, len(lines)):
        code = re.sub(r'"(\\.|[^"\\])*"', '""', lines[i].split('//')[0])
        depth += code.count('{') - code.count('}')
        if depth <= 0 and i > start or (depth == 0 and '{' in code and '}' in code and i == start):
            return i
    return len(lines) - 1


def _fns(lines, start, end, public=True):
    """(name, signature, doc) of each function between the lines, one level inside the block."""
    out = []
    i = start + 1
    while i < end:
        l = lines[i].strip()
        m = re.match(r'(pub )?fn (\w+)', l)
        if m and (m.group(1) or not public):
            sig = l
            j = i
            while '{' not in sig and ';' not in sig and j + 1 < end:
                j += 1
                sig += ' ' + lines[j].strip()
            sig = sig.split('{')[0].strip().rstrip(';').strip()
            doc = []
            k = i - 1
            while k > start and (lines[k].strip().startswith('///') or lines[k].strip().startswith('#[')):
                # The `ไทย:` line is the editor's Thai guide - the Thai pages use method_th's.
                if lines[k].strip().startswith('///') and not lines[k].strip()[3:].strip().startswith('ไทย:'):
                    doc.insert(0, lines[k].strip()[3:].strip())
                k -= 1
            out.append((m.group(2), sig, ' '.join(doc)))
            i = _block_end(lines, j) + 1 if '{' in lines[j] else j + 1
            continue
        i += 1
    return out


def common():
    """What every control has: the inherent methods the `control!` macro writes."""
    lines = _read('ctl.rs')
    m = next(i for i, l in enumerate(lines) if l.startswith('macro_rules! control'))
    impl = next(i for i in range(m, len(lines)) if lines[i].strip() == 'impl $name {')
    return [(n, s.replace('$name', 'Self'), d) for n, s, d in _fns(lines, impl, _block_end(lines, impl))]


def own(type_name):
    """The type's own methods, every `impl <type_name> {` block of the files."""
    out = []
    for f in FILES:
        lines = _read(f)
        for i, l in enumerate(lines):
            if l.strip() == f'impl {type_name} {{':
                out += _fns(lines, i, _block_end(lines, i))
    return out


def text_edit():
    """TextEdit's methods (TextBox and RichTextBox): `use phai_ui::ctl::TextEdit;` to call them."""
    lines = _read('ctl.rs')
    t = next(i for i, l in enumerate(lines) if l.startswith('pub trait TextEdit'))
    return _fns(lines, t, _block_end(lines, t), public=False)


if __name__ == '__main__':
    import json, sys
    sys.stdout.reconfigure(encoding='utf-8')
    data = json.load(open(os.path.join(os.path.dirname(os.path.abspath(__file__)), 'data', 'controls.json'), encoding='utf-8'))
    print('common', [n for n, _, _ in common()])
    print('TextEdit', [n for n, _, _ in text_edit()])
    total = 0
    for t in data['tools']:
        if t.get('type'):
            o = own(t['type'])
            total += len(o)
            print(t['type'], len(o), [n for n, _, _ in o][:12])
    print('total own', total)
