//! The debugger end to end: a small program built with `rustc -g`, run under the debugger,
//! stopped at a breakpoint, stepped into, out of and over, its locals read, run to its end.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

use phai_debug::{Command, Event, Session};

const PROGRAM: &str = r#"fn add(a: i32, b: i32) -> i32 {
    let s = a + b;
    s
}
fn main() {
    let x = 40;
    let y = add(x, 2);
    let name = "phai";
    let text = String::from("ไผ่");
    println!("{y} {name} {text}");
}
"#;

fn rustc() -> Option<PathBuf> {
    let cargo = PathBuf::from(std::env::var_os("CARGO")?);
    let r = cargo.with_file_name("rustc.exe");
    r.is_file().then_some(r)
}

fn build(dir: &Path) -> Option<PathBuf> {
    std::fs::create_dir_all(dir).ok()?;
    let src = dir.join("main.rs");
    std::fs::write(&src, PROGRAM).ok()?;
    let exe = dir.join("prog.exe");
    let ok = std::process::Command::new(rustc()?).arg("-g").arg("-Copt-level=0").arg(&src).arg("-o").arg(&exe).current_dir(dir).status().ok()?.success();
    ok.then_some(exe)
}

fn stopped(rx: &Receiver<Event>) -> (String, u32, Vec<phai_debug::Var>, Vec<phai_debug::Frame>) {
    loop {
        match rx.recv_timeout(Duration::from_secs(20)).expect("the debugger went quiet") {
            Event::Stopped { reason, line, locals, frames, .. } => return (reason, line, locals, frames),
            Event::Exited(c) => panic!("the program ended ({c}) before stopping"),
            Event::BreakpointAt { file, asked, line } => eprintln!("breakpoint {} {asked} -> {line}", file.display()),
            Event::Failed(e) => panic!("{e}"),
            _ => {}
        }
    }
}

fn local<'a>(vars: &'a [phai_debug::Var], name: &str) -> &'a str {
    vars.iter().find(|v| v.name == name).map(|v| v.value.as_str()).unwrap_or_else(|| panic!("no local {name} in {vars:?}"))
}

#[test]
fn break_step_and_read_locals() {
    let dir = std::env::temp_dir().join(format!("phai-debug-test-{}", std::process::id()));
    let Some(exe) = build(&dir) else {
        eprintln!("no rustc to build the test program with; skipped");
        return;
    };
    let src = dir.join("main.rs");
    let (tx, rx) = channel();
    let s = Session::launch(&exe, &dir, vec![(src.clone(), 7)], move |e| {
        let _ = tx.send(e);
    });

    let (reason, line, locals, frames) = stopped(&rx);
    assert_eq!((reason.as_str(), line), ("breakpoint", 7));
    assert_eq!(local(&locals, "x"), "40");
    assert!(frames[0].function.ends_with("main"), "{frames:?}");

    s.send(Command::StepInto);
    let (_, line, locals, frames) = stopped(&rx);
    assert_eq!(line, 2, "into add");
    assert_eq!((local(&locals, "a"), local(&locals, "b")), ("40", "2"));
    assert!(frames[0].function.ends_with("add") && frames[1].function.ends_with("main"), "{frames:?}");

    s.send(Command::StepOut);
    let (_, line, _, frames) = stopped(&rx);
    assert_eq!(line, 7, "back in main");
    assert!(frames[0].function.ends_with("main"));

    s.send(Command::StepOver);
    let (_, line, locals, _) = stopped(&rx);
    assert_eq!(line, 8);
    assert_eq!(local(&locals, "y"), "42");

    s.send(Command::StepOver);
    assert_eq!(stopped(&rx).1, 9);
    s.send(Command::StepOver);
    let (_, line, locals, _) = stopped(&rx);
    assert_eq!(line, 10, "over String::from");
    assert_eq!(local(&locals, "name"), "\"phai\"");
    assert_eq!(local(&locals, "text"), "\"ไผ่\"");

    s.send(Command::Continue);
    loop {
        match rx.recv_timeout(Duration::from_secs(20)).expect("no end") {
            Event::Exited(code) => {
                assert_eq!(code, 0);
                break;
            }
            Event::Stopped { reason, line, .. } => panic!("stopped again: {reason} at {line}"),
            _ => {}
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
}
