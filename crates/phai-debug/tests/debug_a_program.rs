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
    build_text(dir, PROGRAM)
}

fn build_text(dir: &Path, program: &str) -> Option<PathBuf> {
    std::fs::create_dir_all(dir).ok()?;
    let src = dir.join("main.rs");
    std::fs::write(&src, program).ok()?;
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

    // A value typed over, as in Locals; the program goes on with it (it prints y at the end).
    s.send(Command::SetValue { expr: "y".into(), value: "7".into() });
    let locals = loop {
        if let Event::ValueSet { result, .. } = rx.recv_timeout(Duration::from_secs(20)).expect("no answer") {
            break result.expect("set");
        }
    };
    assert_eq!(local(&locals, "y"), "7");
    s.send(Command::StepOver);
    assert_eq!(stopped(&rx).1, 9);
    s.send(Command::StepOver);
    let (_, line, locals, _) = stopped(&rx);
    assert_eq!(line, 10, "over String::from");
    assert_eq!(local(&locals, "name"), "\"phai\"");
    assert_eq!(local(&locals, "text"), "\"ไผ่\"");

    // Words typed over: a &str gets a copy anywhere, a String only within its capacity.
    let set = |expr: &str, value: &str| {
        s.send(Command::SetValue { expr: expr.into(), value: value.into() });
        loop {
            if let Event::ValueSet { result, .. } = rx.recv_timeout(Duration::from_secs(20)).expect("no answer") {
                break result;
            }
        }
    };
    let locals = set("name", "\"ไผ่ \\\"studio\\\"\"").expect("a &str");
    assert_eq!(local(&locals, "name"), "\"ไผ่ \\\"studio\\\"\"");
    let locals = set("text", "Phai").expect("fits in 9 bytes");
    assert_eq!(local(&locals, "text"), "\"Phai\"");
    assert!(set("text", "far too long for nine bytes").is_err(), "beyond its capacity");

    // Watch: paths and comparisons, while paused.
    s.send(Command::Evaluate { id: 7, exprs: vec!["text".into(), "y == 7".into(), "x == y".into(), "nothing".into()] });
    let vars = loop {
        if let Event::Values { id: 7, vars } = rx.recv_timeout(Duration::from_secs(20)).expect("no values") {
            break vars;
        }
    };
    assert_eq!(vars[0].value, "\"Phai\"", "as typed over above");
    assert_eq!(vars[1].value, "true");
    assert_eq!(vars[2].value, "false");
    assert!(vars[3].value.contains("no `nothing`"), "{vars:?}");

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

#[test]
fn a_condition_decides_whether_a_breakpoint_stops() {
    let dir = std::env::temp_dir().join(format!("phai-debug-cond-{}", std::process::id()));
    let Some(exe) = build(&dir) else { return };
    let src = dir.join("main.rs");
    for (cond, stops) in [("x < 5", false), ("x == 40", true)] {
        let (tx, rx) = channel();
        let s = Session::launch(&exe, &dir, vec![(src.clone(), 7)], move |e| {
            let _ = tx.send(e);
        });
        s.send(Command::Conditions(vec![(src.clone(), 7, cond.into())]));
        let mut stopped = false;
        loop {
            match rx.recv_timeout(Duration::from_secs(20)).expect("quiet") {
                Event::Stopped { line, .. } => {
                    assert_eq!(line, 7);
                    stopped = true;
                    s.send(Command::Continue);
                }
                Event::Exited(_) => break,
                Event::Failed(e) => panic!("{e}"),
                _ => {}
            }
        }
        assert_eq!(stopped, stops, "condition {cond}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_hit_count_decides_which_time_a_breakpoint_stops() {
    const LOOP: &str = "fn main() {\n    let mut sum = 0;\n    for i in 0..10 {\n        sum += i;\n    }\n    println!(\"{sum}\");\n}\n";
    let dir = std::env::temp_dir().join(format!("phai-debug-hits-{}", std::process::id()));
    let Some(exe) = build_text(&dir, LOOP) else { return };
    let src = dir.join("main.rs");
    // The 3rd time, every 4th time, and from the 9th: which `i` each stop sees.
    for (hits, want) in [("= 3", vec!["2"]), ("% 4", vec!["3", "7"]), (">= 9", vec!["8", "9"])] {
        let (tx, rx) = channel();
        let s = Session::launch(&exe, &dir, vec![(src.clone(), 4)], move |e| {
            let _ = tx.send(e);
        });
        let h = phai_debug::Hits::parse(hits).unwrap();
        s.send(Command::Conditions(vec![(src.clone(), 4, phai_debug::join_condition("", h))]));
        let mut seen = vec![];
        loop {
            match rx.recv_timeout(Duration::from_secs(20)).expect("quiet") {
                Event::Stopped { locals, .. } => {
                    seen.push(local(&locals, "i").to_string());
                    s.send(Command::Continue);
                }
                Event::Exited(_) => break,
                Event::Failed(e) => panic!("{e}"),
                _ => {}
            }
        }
        assert_eq!(seen, want, "hit count {hits}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

const THREADS: &str = r#"fn work(n: u64) -> u64 {
    let mut total = 0;
    for i in 0..n {
        total += i;
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    total
}
fn main() {
    let h = std::thread::Builder::new().name("worker".into()).spawn(|| work(1000)).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(300));
    let x = 1;
    println!("{x}");
    drop(h);
}
"#;

#[test]
fn threads_are_listed_and_one_can_be_looked_at() {
    let dir = std::env::temp_dir().join(format!("phai-debug-threads-{}", std::process::id()));
    let Some(exe) = build_text(&dir, THREADS) else { return };
    let src = dir.join("main.rs");
    let (tx, rx) = channel();
    let s = Session::launch(&exe, &dir, vec![(src.clone(), 13)], move |e| {
        let _ = tx.send(e);
    });
    let threads = loop {
        match rx.recv_timeout(Duration::from_secs(20)).expect("quiet") {
            Event::Stopped { threads, .. } => break threads,
            Event::Exited(c) => panic!("ended {c}"),
            Event::Failed(e) => panic!("{e}"),
            _ => {}
        }
    };
    eprintln!("{threads:?}");
    assert_eq!(threads[0].name, "main");
    assert!(threads[0].current, "stopped on the main thread");
    let worker = threads.iter().find(|t| t.name == "worker").expect("the named thread is listed");
    s.send(Command::Thread(worker.id));
    loop {
        match rx.recv_timeout(Duration::from_secs(20)).expect("quiet") {
            Event::Stopped { reason, frames, threads, .. } => {
                assert_eq!(reason, "thread");
                assert!(threads.iter().any(|t| t.current && t.name == "worker"));
                assert!(frames.iter().any(|f| f.function.contains("work")), "its own stack: {frames:?}");
                break;
            }
            Event::Failed(e) => panic!("{e}"),
            _ => {}
        }
    }
    s.send(Command::Stop);
    let _ = std::fs::remove_dir_all(&dir);
}

