# Builds the Toolbox pages' examples and their pictures:
#     python docs/help-src/build_examples.py            every demo
#     python docs/help-src/build_examples.py button     only those named
# 1. examples/controls/src/main.rs is written from the *_demo.phaiform files there: `cargo run --
#    <kind>` opens that demo.
# 2. Phai Studio writes each demo's _ui.rs, as saving it in the designer does (--command 102).
# 3. cargo build - the pages quote this code, so it must compile.
# 4. Each demo is run and photographed into data/shots/ctl-<kind>.png (PHAI_SHOT).
import glob, os, re, subprocess, sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.normpath(os.path.join(HERE, '..', '..'))
PROJ = os.path.join(HERE, 'examples', 'controls')
SRC = os.path.join(PROJ, 'src')
STUDIO = os.path.join(ROOT, 'target', 'debug', 'phai-studio.exe')
ENV = dict(os.environ, RUSTUP_HOME=r'D:\Tools\Rust\rustup', CARGO_HOME=r'D:\Tools\Rust\cargo',
           PATH=r'D:\Tools\Rust\cargo\bin;' + os.environ.get('PATH', ''))


def demos():
    return sorted(os.path.basename(p)[:-len('_demo.phaiform')] for p in glob.glob(os.path.join(SRC, '*_demo.phaiform')))


def camel(s):
    return ''.join(w.capitalize() for w in s.split('_'))


def helpers():
    """Forms of the project that are not a demo window - a user control a demo puts on itself."""
    return sorted(os.path.basename(p)[:-len('.phaiform')] for p in glob.glob(os.path.join(SRC, '*.phaiform')) if not p.endswith('_demo.phaiform'))


def write_main(kinds):
    mods = '\n'.join(f'mod {k}_demo;\nmod {k}_demo_ui;' for k in kinds)
    mods += ''.join(f'\nmod {h};\nmod {h}_ui;' for h in helpers())
    arms = '\n'.join(f'        "{k}" => {k}_demo_ui::{camel(k)}DemoUi::run({k}_demo::{camel(k)}Demo::default()),' for k in kinds)
    text = f'''// Written by docs/help-src/build_examples.py from the *_demo.phaiform files: `cargo run -- <kind>`
// opens that Toolbox page's demo window.
#![windows_subsystem = "windows"]

{mods}

fn main() {{
    match std::env::args().nth(1).as_deref().unwrap_or("") {{
{arms}
        other => eprintln!("no demo called {{other:?}}"),
    }}
}}
'''
    open(os.path.join(SRC, 'main.rs'), 'w', encoding='utf-8', newline='\n').write(text)


def generate_form(name):
    """Phai Studio writes <name>_ui.rs from <name>.phaiform, as saving it in the designer does."""
    shot = os.path.join(os.environ.get('TEMP', '.'), f'phai-save-{name}.png')
    subprocess.run([STUDIO, os.path.join(SRC, f'{name}.phaiform'), '--command', '102'], env=dict(ENV, PHAI_SHOT=shot), timeout=90,
                   stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    if not os.path.exists(os.path.join(SRC, f'{name}_ui.rs')):
        sys.exit(f'Phai Studio wrote no {name}_ui.rs')


def generate(kind):
    generate_form(f'{kind}_demo')


def build():
    r = subprocess.run(['cargo', 'build'], cwd=PROJ, env=ENV, capture_output=True, text=True)
    out = r.stdout + r.stderr
    problems = [l for l in out.split('\n') if re.match(r'^(error|warning)', l)]
    if r.returncode != 0 or problems:
        print(out[-6000:])
        sys.exit('the examples do not build cleanly')


def photograph(kind):
    exe = os.path.join(PROJ, 'target', 'debug', 'control-examples.exe')
    shot = os.path.join(HERE, 'data', 'shots', f'ctl-{kind}.png')
    # From the project's folder, as cargo run is: pictures/ and other files are found from there.
    subprocess.run([exe, kind], cwd=PROJ, env=dict(ENV, PHAI_SHOT=shot, PHAI_SHOT_DELAY='2'), timeout=60)


if __name__ == '__main__':
    all_kinds = demos()
    chosen = sys.argv[1:] or all_kinds
    write_main(all_kinds)
    for h in helpers():
        generate_form(h)
    for k in chosen:
        generate(k)
    build()
    for k in chosen:
        photograph(k)
        print('ok', k)
