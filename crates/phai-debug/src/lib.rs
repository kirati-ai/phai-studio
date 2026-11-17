//! A debugger for Rust programs built for Windows (the MSVC toolchain), on the Windows debug API
//! and dbghelp - both part of Windows, so nothing to install. It does what Visual Studio's does
//! for a first day: breakpoints by file and line, Continue, Step Over, Step Into, Step Out, Break
//! All, a stop on every panic, the call stack, and the locals of the frame stopped in - numbers,
//! `bool`, `char`, `&str`, `String` and plain structs read from the program's memory through the
//! types in its PDB.
//!
//! The debuggee is driven from a thread of its own (Windows wants the thread that started it to
//! wait on it); [`Session`] sends that thread [`Command`]s and hears back [`Event`]s.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};

use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE, NTSTATUS};
use windows::Win32::System::Diagnostics::Debug::*;
use windows::Win32::System::Memory::{VirtualAllocEx, MEM_COMMIT, MEM_RESERVE, PAGE_READWRITE};
use windows::Win32::System::Threading::{
    CreateProcessW, GetProcessId, OpenProcess, QueryFullProcessImageNameW, TerminateProcess, CREATE_NEW_CONSOLE, DEBUG_ONLY_THIS_PROCESS, PROCESS_ALL_ACCESS, PROCESS_INFORMATION, PROCESS_NAME_WIN32, STARTUPINFOW,
};

/// What the debugger is asked to do.
#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    Continue,
    StepOver,
    StepInto,
    StepOut,
    /// Break All: stop the program wherever it is.
    Pause,
    /// End the program - or, attached to one already running, let it go on alone.
    Stop,
    /// Let the program go on alone, the debugger gone from it (Detach All).
    Detach,
    /// Exception Settings: whether a panic stops the program, and the Windows exceptions (by
    /// code, `0xC0000005` an access violation) that stop it as soon as they happen - before the
    /// program's own handlers see them. Others stop it only when nothing handles them.
    Exceptions { panics: bool, first_chance: Vec<u32> },
    /// While paused: `len` bytes of memory at an address (`0x7ff6...`) or where a variable
    /// (`total`, `order.items`) is - the Memory window. Answered by [`Event::Memory`].
    Memory { id: u64, at: String, len: usize },
    /// While paused: stop when the variable `expr` changes - a data breakpoint, on the
    /// processor's own debug registers (four at most). Answered by [`Event::DataBreakpoint`].
    DataBreakpoint(String),
    /// Every data breakpoint taken away.
    ClearDataBreakpoints,
    /// While paused: look at this thread instead - its stack and locals said again by a
    /// [`Event::Stopped`] (reason `thread`), and the steps taken on it.
    Thread(u32),
    /// The breakpoints, every one, as file and line from 1.
    Breakpoints(Vec<(PathBuf, u32)>),
    /// Conditions on breakpoints - file, line, `x > 5` - each stopping only when it holds.
    Conditions(Vec<(PathBuf, u32, String)>),
    /// While paused: the values of these expressions (`total`, `order.items.len`), answered by
    /// [`Event::Values`] with the same `id` - for Watch, and for a data tip.
    Evaluate { id: u64, exprs: Vec<String> },
    /// While paused: a number, `bool` or `char` given a new value, as Visual Studio lets a
    /// value in Locals be typed over. Answered by [`Event::ValueSet`].
    SetValue { expr: String, value: String },
}

/// A thread of the paused program, as the Threads window lists it.
#[derive(Clone, Debug, PartialEq)]
pub struct Thread {
    pub id: u32,
    /// Its name (`main`, or what `thread::Builder::name` gave it); empty when it has none.
    pub name: String,
    /// Where it is: its innermost function in the program's own code, else its innermost.
    pub location: String,
    /// The thread looked at - stopped in, or switched to.
    pub current: bool,
}

/// A frame of the call stack: the function, and where in the source when the PDB says.
#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    pub function: String,
    pub file: Option<PathBuf>,
    pub line: u32,
}

/// A local variable or parameter of the frame stopped in.
#[derive(Clone, Debug, PartialEq)]
pub struct Var {
    pub name: String,
    pub ty: String,
    pub value: String,
}

/// What the debugger says.
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    /// The program is paused: why (`breakpoint`, `step`, `panic`, `pause`, an exception), where,
    /// its call stack, and the locals of its top frame.
    Stopped { reason: String, file: Option<PathBuf>, line: u32, frames: Vec<Frame>, locals: Vec<Var>, threads: Vec<Thread> },
    /// It runs again.
    Running,
    /// A breakpoint asked for at a line with no code moved to this line (or went nowhere: 0).
    BreakpointAt { file: PathBuf, asked: u32, line: u32 },
    /// Words the program wrote with `OutputDebugString`.
    Output(String),
    /// It ended, with this exit code.
    Exited(u32),
    /// It could not be started.
    Failed(String),
    /// The debugger let go of it; it runs on alone.
    Detached,
    /// [`Command::Memory`] answered: the address, and each byte (`None` where it cannot be read).
    Memory { id: u64, result: Result<(u64, Vec<Option<u8>>), String> },
    /// [`Command::DataBreakpoint`] done - what it watches - or why not.
    DataBreakpoint(Result<String, String>),
    /// The values asked for by [`Command::Evaluate`].
    Values { id: u64, vars: Vec<Var> },
    /// [`Command::SetValue`] done - the locals as they are now - or why not.
    ValueSet { expr: String, result: Result<Vec<Var>, String> },
}

/// A program running under the debugger.
pub struct Session {
    tx: Sender<Command>,
    /// The program's process id once it has started; 0 before.
    pid: std::sync::Arc<std::sync::atomic::AtomicU32>,
}

impl Session {
    /// Starts `exe` (in `cwd`) under the debugger with `breakpoints` set; `on_event` hears what
    /// happens, on the debugger's own thread.
    pub fn launch(exe: &Path, cwd: &Path, breakpoints: Vec<(PathBuf, u32)>, on_event: impl Fn(Event) + Send + 'static) -> Session {
        let (tx, rx) = channel();
        let (exe, cwd) = (exe.to_path_buf(), cwd.to_path_buf());
        let pid = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let theirs = pid.clone();
        std::thread::spawn(move || {
            match Engine::start(&exe, &cwd, breakpoints, rx, Box::new(on_event)) {
                Ok(mut e) => {
                    theirs.store(unsafe { windows::Win32::System::Threading::GetProcessId(e.process) }, std::sync::atomic::Ordering::Relaxed);
                    e.run()
                }
                Err((msg, say)) => say(Event::Failed(msg)),
            }
        });
        Session { tx, pid }
    }

    /// Attaches the debugger to the program already running as process `pid`; its source files are
    /// named from `cwd` (its project), as when launched.
    pub fn attach(pid: u32, cwd: &Path, breakpoints: Vec<(PathBuf, u32)>, on_event: impl Fn(Event) + Send + 'static) -> Session {
        let (tx, rx) = channel();
        let cwd = cwd.to_path_buf();
        let shared = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(pid));
        std::thread::spawn(move || match Engine::attach(pid, &cwd, breakpoints, rx, Box::new(on_event)) {
            Ok(mut e) => e.run(),
            Err((msg, say)) => say(Event::Failed(msg)),
        });
        Session { tx, pid: shared }
    }

    /// The program's process id, once it has started (0 before).
    pub fn pid(&self) -> u32 {
        self.pid.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn send(&self, c: Command) {
        let _ = self.tx.send(c);
    }
}

// --- the engine ---------------------------------------------------------------------------------

const BREAKPOINT: u32 = 0x8000_0003;
const SINGLE_STEP: u32 = 0x8000_0004;
const DBG_CONTINUE: NTSTATUS = NTSTATUS(0x0001_0002);
const DBG_EXCEPTION_NOT_HANDLED: NTSTATUS = NTSTATUS(0x8001_0001u32 as i32);
const TRAP_FLAG: u32 = 0x100;
const SYMFLAG_REGISTER: u32 = 0x8;
const SYMFLAG_REGREL: u32 = 0x10;
const SYMFLAG_FRAMEREL: u32 = 0x20;
const SYMFLAG_PARAMETER: u32 = 0x40;
const SYMFLAG_LOCAL: u32 = 0x80;

/// A thread's registers, aligned as `GetThreadContext` wants them.
#[repr(C, align(16))]
#[derive(Clone, Copy)]
struct Ctx(CONTEXT);

#[derive(Clone, Copy, Debug, PartialEq)]
enum Kind {
    User,
    /// `rust_panic`: every panic passes through it.
    Panic,
    /// Step Over's or Step Out's: where a call returns to, and the stack it returns with.
    Temp(u64),
}

#[derive(Clone, Copy, Debug)]
struct Bp {
    orig: u8,
    kind: Kind,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum StepKind {
    Into,
    Over,
    Out,
}

#[derive(Clone, Debug)]
struct Step {
    /// The thread being stepped: another passing the same code is not the step.
    tid: u32,
    kind: StepKind,
    from: Option<(String, u32)>,
    rsp: u64,
    prev_rip: u64,
    prev_rsp: u64,
}

type Say = Box<dyn Fn(Event) + Send>;

struct Engine {
    process: HANDLE,
    /// The program, and the folder its PDB is in.
    exe: PathBuf,
    /// Where it runs - for Cargo, the project: the PDB names its files from there (`src\main.rs`).
    cwd: PathBuf,
    threads: HashMap<u32, HANDLE>,
    bps: HashMap<u64, Bp>,
    wanted: Vec<(PathBuf, u32)>,
    /// A breakpoint stepped off, to be written again after one instruction.
    rearm: Option<u64>,
    step: Option<Step>,
    loaded: bool,
    rx: Receiver<Command>,
    say: Say,
    /// Break All asked for: the next breakpoint Windows reports is ours.
    pausing: bool,
    /// Conditions by file and line, and by the address their breakpoint landed at.
    conds: Vec<(PathBuf, u32, String)>,
    cond_at: HashMap<u64, String>,
    /// How many times each breakpoint was reached with its condition holding.
    hits: HashMap<u64, u64>,
    /// Attached to a program already running: Stop lets it go rather than ending it.
    attached: bool,
    /// Detach asked for while paused: done once the event is let go.
    detaching: bool,
    /// The thread the program began with: Windows gives it no name.
    main_thread: u32,
    /// Exception Settings: see [`Command::Exceptions`].
    break_on_panic: bool,
    first_chance: Vec<u32>,
    /// Data breakpoints: address, length (1, 2, 4 or 8) and what was asked - in DR0 to DR3.
    watched: Vec<(u64, u64, String)>,
}

/// The program file of a running process, if it may be looked at.
pub fn process_image(pid: u32) -> Option<PathBuf> {
    unsafe {
        let h = OpenProcess(windows::Win32::System::Threading::PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = vec![0u16; 1024];
        let mut n = buf.len() as u32;
        let got = QueryFullProcessImageNameW(h, PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut n);
        let _ = CloseHandle(h);
        got.ok()?;
        Some(PathBuf::from(String::from_utf16_lossy(&buf[..n as usize])))
    }
}

/// When a breakpoint stops by how many times it has been reached, as Visual Studio's Hit Count.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hits {
    /// `= 3`: the third time only.
    Equal(u64),
    /// `>= 3`: the third time and every time after.
    AtLeast(u64),
    /// `% 3`: every third time.
    Multiple(u64),
}

impl Hits {
    /// `= 3`, `3`, `>= 3`, `% 3` or `multiple of 3`; `Ok(None)` for nothing written.
    pub fn parse(text: &str) -> Result<Option<Hits>, String> {
        let t = text.trim();
        if t.is_empty() {
            return Ok(None);
        }
        let num = |s: &str| s.trim().parse::<u64>().ok().filter(|n| *n > 0).ok_or_else(|| format!("`{t}`: a hit count is = N, >= N or % N, N from 1"));
        let h = if let Some(r) = t.strip_prefix(">=") {
            Hits::AtLeast(num(r)?)
        } else if let Some(r) = t.strip_prefix('%').or_else(|| t.strip_prefix("multiple of")) {
            Hits::Multiple(num(r)?)
        } else {
            Hits::Equal(num(t.trim_start_matches("==").trim_start_matches('='))?)
        };
        Ok(Some(h))
    }

    pub fn text(&self) -> String {
        match self {
            Hits::Equal(n) => format!("= {n}"),
            Hits::AtLeast(n) => format!(">= {n}"),
            Hits::Multiple(n) => format!("% {n}"),
        }
    }

    /// Whether the `count`th time reached stops.
    pub fn stops(&self, count: u64) -> bool {
        match *self {
            Hits::Equal(n) => count == n,
            Hits::AtLeast(n) => count >= n,
            Hits::Multiple(n) => count.is_multiple_of(n),
        }
    }
}

/// A breakpoint's condition and hit count in the one text `Command::Conditions` carries:
/// `#>= 3;x > 5`, or the expression alone.
pub fn join_condition(expr: &str, hits: Option<Hits>) -> String {
    match hits {
        Some(h) => format!("#{};{}", h.text(), expr.trim()),
        None => expr.trim().to_string(),
    }
}

/// The text `join_condition` made, apart again: the expression (maybe empty) and the hit count.
pub fn split_condition(cond: &str) -> (String, Option<Hits>) {
    match cond.strip_prefix('#').and_then(|r| r.split_once(';')) {
        Some((h, e)) => (e.trim().to_string(), Hits::parse(h).ok().flatten()),
        None => (cond.trim().to_string(), None),
    }
}

/// Whether two paths are one file, as Windows compares them: case and slashes aside.
fn same_file(a: &Path, b: &Path) -> bool {
    let n = |p: &Path| p.to_string_lossy().replace('/', "\\").trim_start_matches(r"\?\").to_lowercase();
    n(a) == n(b)
}

/// A thread's name, as `SetThreadDescription` gave it (Rust names its threads so).
fn thread_name(h: HANDLE) -> String {
    unsafe {
        match windows::Win32::System::Threading::GetThreadDescription(h) {
            Ok(p) if !p.is_null() => {
                let s = p.to_string().unwrap_or_default();
                let _ = windows::Win32::Foundation::LocalFree(Some(windows::Win32::Foundation::HLOCAL(p.0 as _)));
                s
            }
            _ => String::new(),
        }
    }
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

fn from_wide(p: PWSTR) -> String {
    if p.is_null() {
        return String::new();
    }
    unsafe { p.to_string().unwrap_or_default() }
}

impl Engine {
    fn start(exe: &Path, cwd: &Path, wanted: Vec<(PathBuf, u32)>, rx: Receiver<Command>, say: Say) -> Result<Engine, (String, Say)> {
        let mut si = STARTUPINFOW { cb: std::mem::size_of::<STARTUPINFOW>() as u32, ..Default::default() };
        si.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
        let mut pi = PROCESS_INFORMATION::default();
        let app = wide(&exe.to_string_lossy());
        let mut cmd = wide(&format!("\"{}\"", exe.display()));
        let dir = wide(&cwd.to_string_lossy());
        let ok = unsafe { CreateProcessW(PCWSTR(app.as_ptr()), Some(PWSTR(cmd.as_mut_ptr())), None, None, false, DEBUG_ONLY_THIS_PROCESS | CREATE_NEW_CONSOLE, None, PCWSTR(dir.as_ptr()), &si, &mut pi) };
        if let Err(e) = ok {
            return Err((format!("{}: {e}", exe.display()), say));
        }
        unsafe {
            let _ = CloseHandle(pi.hThread);
        }
        Ok(Engine::new(pi.hProcess, exe, cwd, wanted, rx, say, false))
    }

    fn new(process: HANDLE, exe: &Path, cwd: &Path, wanted: Vec<(PathBuf, u32)>, rx: Receiver<Command>, say: Say, attached: bool) -> Engine {
        Engine { process, exe: exe.to_path_buf(), cwd: cwd.to_path_buf(), threads: HashMap::new(), bps: HashMap::new(), wanted, rearm: None, step: None, loaded: false, rx, say, pausing: false, conds: vec![], cond_at: HashMap::new(), hits: HashMap::new(), attached, detaching: false, main_thread: 0, break_on_panic: true, first_chance: vec![], watched: vec![] }
    }

    /// Attach to Process: the debugger joins a program already running. Windows then tells of its
    /// threads and modules as though it had just started, and stops it once - let go at once.
    fn attach(pid: u32, cwd: &Path, wanted: Vec<(PathBuf, u32)>, rx: Receiver<Command>, say: Say) -> Result<Engine, (String, Say)> {
        let Some(exe) = process_image(pid) else { return Err((format!("process {pid}: it cannot be looked at"), say)) };
        let process = match unsafe { OpenProcess(PROCESS_ALL_ACCESS, false, pid) } {
            Ok(h) => h,
            Err(e) => return Err((format!("{}: {e}", exe.display()), say)),
        };
        if let Err(e) = unsafe { DebugActiveProcess(pid) } {
            unsafe {
                let _ = CloseHandle(process);
            }
            return Err((format!("{}: {e}", exe.display()), say));
        }
        // Phai Studio closing must not end a program it only joined.
        unsafe {
            let _ = DebugSetProcessKillOnExit(false);
        }
        Ok(Engine::new(process, &exe, cwd, wanted, rx, say, true))
    }

    /// Detach: every breakpoint taken out, no thread left stepping, and the program let go.
    fn detach(&mut self) {
        let addrs: Vec<u64> = self.bps.keys().copied().collect();
        for a in addrs {
            self.disarm(a);
        }
        self.rearm = None;
        self.step = None;
        let tids: Vec<u32> = self.threads.keys().copied().collect();
        for tid in tids {
            if let Some(mut c) = self.context(tid) {
                if c.0.EFlags & TRAP_FLAG != 0 {
                    c.0.EFlags &= !TRAP_FLAG;
                    self.set_context(tid, &c);
                }
            }
        }
        unsafe {
            let _ = DebugActiveProcessStop(GetProcessId(self.process));
            let _ = SymCleanup(self.process);
            let _ = CloseHandle(self.process);
        }
        (self.say)(Event::Detached);
    }

    fn run(&mut self) {
        loop {
            let mut ev = DEBUG_EVENT::default();
            if unsafe { WaitForDebugEvent(&mut ev, 50) }.is_err() {
                // Nothing from the program: what the person asks while it runs.
                match self.rx.try_recv() {
                    Ok(Command::Detach) => return self.detach(),
                    Ok(Command::Stop) if self.attached => return self.detach(),
                    Ok(Command::Stop) => unsafe {
                        let _ = TerminateProcess(self.process, 1);
                    },
                    Ok(Command::Pause) => unsafe {
                        self.pausing = true;
                        let _ = DebugBreakProcess(self.process);
                    },
                    Ok(Command::Breakpoints(list)) => self.set_breakpoints(list),
                    Ok(Command::ClearDataBreakpoints) => {
                        self.watched.clear();
                        self.write_debug_registers();
                    }
                    Ok(Command::Exceptions { panics, first_chance }) => {
                        self.break_on_panic = panics;
                        self.first_chance = first_chance;
                    }
                    Ok(Command::Conditions(list)) => {
                        self.conds = list;
                        let w = self.wanted.clone();
                        self.set_breakpoints(w);
                    }
                    Ok(_) => {}
                    Err(TryRecvError::Disconnected) if self.attached => return self.detach(),
                    Err(TryRecvError::Disconnected) => unsafe {
                        let _ = TerminateProcess(self.process, 1);
                    },
                    Err(TryRecvError::Empty) => {}
                }
                continue;
            }
            let tid = ev.dwThreadId;
            let mut status = DBG_CONTINUE;
            unsafe {
                match ev.dwDebugEventCode {
                    CREATE_PROCESS_DEBUG_EVENT => {
                        let info = ev.u.CreateProcessInfo;
                        self.threads.insert(tid, info.hThread);
                        self.main_thread = tid;
                        self.load_symbols(info.hFile, info.lpBaseOfImage as u64);
                        if !info.hFile.is_invalid() {
                            let _ = CloseHandle(info.hFile);
                        }
                    }
                    CREATE_THREAD_DEBUG_EVENT => {
                        self.threads.insert(tid, ev.u.CreateThread.hThread);
                        // A new thread watches what the others do.
                        if !self.watched.is_empty() {
                            self.write_debug_registers();
                        }
                    }
                    EXIT_THREAD_DEBUG_EVENT => {
                        if std::env::var_os("PHAI_DEBUG_LOG").is_some() {
                            eprintln!("thread {tid} ended: {} {}", ev.u.ExitThread.dwExitCode, self.threads.get(&tid).map(|h| thread_name(*h)).unwrap_or_default());
                        }
                        self.threads.remove(&tid);
                    }
                    LOAD_DLL_DEBUG_EVENT => {
                        let f = ev.u.LoadDll.hFile;
                        // Known to dbghelp, so a stack walks through it (a thread asleep in Windows)
                        // and its functions have names.
                        if !f.is_invalid() {
                            let _ = SymLoadModuleExW(self.process, Some(f), PCWSTR::null(), PCWSTR::null(), ev.u.LoadDll.lpBaseOfDll as u64, 0, None, None);
                        }
                        if !f.is_invalid() {
                            let _ = CloseHandle(f);
                        }
                    }
                    OUTPUT_DEBUG_STRING_EVENT => {
                        let d = ev.u.DebugString;
                        let n = d.nDebugStringLength as usize;
                        if d.fUnicode != 0 {
                            let mut buf = vec![0u16; n];
                            let _ = ReadProcessMemory(self.process, d.lpDebugStringData.0 as _, buf.as_mut_ptr() as _, n * 2, None);
                            (self.say)(Event::Output(String::from_utf16_lossy(&buf).trim_end_matches('\0').to_string()));
                        } else {
                            let mut buf = vec![0u8; n];
                            let _ = ReadProcessMemory(self.process, d.lpDebugStringData.0 as _, buf.as_mut_ptr() as _, n, None);
                            (self.say)(Event::Output(String::from_utf8_lossy(&buf).trim_end_matches('\0').to_string()));
                        }
                    }
                    EXCEPTION_DEBUG_EVENT => {
                        let info = ev.u.Exception;
                        status = self.exception(tid, &info);
                    }
                    EXIT_PROCESS_DEBUG_EVENT => {
                        let code = ev.u.ExitProcess.dwExitCode;
                        let _ = ContinueDebugEvent(ev.dwProcessId, tid, DBG_CONTINUE);
                        let _ = SymCleanup(self.process);
                        let _ = CloseHandle(self.process);
                        (self.say)(Event::Exited(code));
                        return;
                    }
                    _ => {}
                }
                let _ = ContinueDebugEvent(ev.dwProcessId, tid, status);
            }
            if self.detaching {
                return self.detach();
            }
        }
    }

    fn load_symbols(&mut self, file: HANDLE, base: u64) {
        unsafe {
            // Lines, plain names, and the PDB beside the program.
            SymSetOptions(0x0000_0010 /* LOAD_LINES */ | 0x0000_0002 /* UNDNAME */ | 0x0008_0000 /* FAIL_CRITICAL_ERRORS */);
            // Its PDB is looked for beside it first.
            let dir = wide(&self.exe.parent().map(|d| d.to_string_lossy().to_string()).unwrap_or_default());
            let _ = SymInitializeW(self.process, PCWSTR(dir.as_ptr()), false);
            let image = wide(&self.exe.to_string_lossy());
            let got = SymLoadModuleExW(self.process, Some(file), PCWSTR(image.as_ptr()), PCWSTR::null(), base, 0, None, None);
            if std::env::var_os("PHAI_DEBUG_LOG").is_some() {
                eprintln!("symbols at {base:#x}: {got:#x} {:?}", windows::Win32::Foundation::GetLastError());
                let mut m = IMAGEHLP_MODULEW64 { SizeOfStruct: std::mem::size_of::<IMAGEHLP_MODULEW64>() as u32, ..Default::default() };
                let r = SymGetModuleInfoW64(self.process, base, &mut m);
                eprintln!("module info {:?} type {:?} pdb {}", r, m.SymType, String::from_utf16_lossy(&m.LoadedPdbName).trim_end_matches('\0'));
            }
        }
        if std::env::var_os("PHAI_DEBUG_LOG").is_some() {
            for n in ["main", "prog::main", "rust_panic", "std::panicking::rust_panic"] {
                let a = self.symbol_address(n);
                eprintln!("{n}: {a:x?} {:?}", a.and_then(|a| self.line_of(a)));
            }
        }
        let wanted = std::mem::take(&mut self.wanted);
        self.set_breakpoints(wanted);
        // Every panic passes through it - named in full by today's Rust, plainly by older ones.
        if let Some(addr) = self.symbol_address("std::panicking::rust_panic").or_else(|| self.symbol_address("rust_panic")) {
            self.arm(addr, Kind::Panic);
        }
    }

    fn symbol_address(&self, name: &str) -> Option<u64> {
        let mut buf = SymBuf::new();
        let n = wide(name);
        unsafe { SymFromNameW(self.process, PCWSTR(n.as_ptr()), buf.info()) }.ok()?;
        Some(unsafe { (*buf.info()).Address })
    }

    // --- breakpoints ----------------------------------------------------------------------------

    fn read(&self, addr: u64, len: usize) -> Option<Vec<u8>> {
        let mut buf = vec![0u8; len];
        let mut got = 0usize;
        unsafe { ReadProcessMemory(self.process, addr as _, buf.as_mut_ptr() as _, len, Some(&mut got)) }.ok()?;
        buf.truncate(got);
        // What the program really holds, not the breakpoints written over it.
        for (a, bp) in &self.bps {
            if *a >= addr && *a < addr + got as u64 {
                buf[(*a - addr) as usize] = bp.orig;
            }
        }
        (got == len).then_some(buf)
    }

    fn write_byte(&self, addr: u64, b: u8) {
        unsafe {
            let _ = WriteProcessMemory(self.process, addr as _, &b as *const u8 as _, 1, None);
            let _ = FlushInstructionCache(self.process, Some(addr as _), 1);
        }
    }

    fn arm(&mut self, addr: u64, kind: Kind) {
        if let Some(bp) = self.bps.get_mut(&addr) {
            // A user's breakpoint stays one whatever else lands there.
            if bp.kind != Kind::User {
                bp.kind = kind;
            }
            return;
        }
        let mut orig = [0u8];
        if unsafe { ReadProcessMemory(self.process, addr as _, orig.as_mut_ptr() as _, 1, None) }.is_err() {
            return;
        }
        self.bps.insert(addr, Bp { orig: orig[0], kind });
        self.write_byte(addr, 0xCC);
    }

    fn disarm(&mut self, addr: u64) {
        if let Some(bp) = self.bps.remove(&addr) {
            self.write_byte(addr, bp.orig);
        }
    }

    /// The first address with code at `line` of `file`, or a little below when the line has none.
    fn line_address(&self, file: &Path, line: u32) -> Option<(u64, u32)> {
        // The PDB may name the file in full, from the project (Cargo), or otherwise: the whole
        // path first, then from the project, then ever shorter tails of it.
        let mut names: Vec<String> = vec![file.to_string_lossy().to_string()];
        if let Ok(rel) = file.strip_prefix(&self.cwd) {
            names.push(rel.to_string_lossy().to_string());
        }
        let parts: Vec<String> = file.components().map(|c| c.as_os_str().to_string_lossy().to_string()).collect();
        for k in (1..parts.len().min(4)).rev() {
            names.push(parts[parts.len() - k..].join("\\"));
        }
        for name in names {
            let f = wide(&name);
            for l in line..line + 12 {
                let mut info = IMAGEHLP_LINEW64 { SizeOfStruct: std::mem::size_of::<IMAGEHLP_LINEW64>() as u32, ..Default::default() };
                let mut disp = 0i32;
                if unsafe { SymGetLineFromNameW64(self.process, PCWSTR::null(), PCWSTR(f.as_ptr()), l, &mut disp, &mut info) }.is_ok() && info.LineNumber == l {
                    // Only a line in this very file: a tail can match another one.
                    let named = self.resolve(&from_wide(info.FileName));
                    if same_file(&named, file) {
                        return Some((info.Address, l));
                    }
                }
            }
        }
        None
    }

    /// A file as the PDB names it, made whole: a relative name is from where the program runs.
    fn resolve(&self, name: &str) -> PathBuf {
        let p = PathBuf::from(name);
        if p.is_absolute() { p } else { self.cwd.join(p) }
    }

    fn set_breakpoints(&mut self, list: Vec<(PathBuf, u32)>) {
        let old: Vec<u64> = self.bps.iter().filter(|(_, b)| b.kind == Kind::User).map(|(a, _)| *a).collect();
        for a in old {
            if self.rearm == Some(a) {
                self.rearm = None;
            }
            self.disarm(a);
        }
        // Before symbols are loaded, kept until they are.
        if !self.loaded_symbols() {
            self.wanted = list;
            return;
        }
        self.cond_at.clear();
        for (file, line) in &list {
            match self.line_address(file, *line) {
                Some((addr, at)) => {
                    self.arm(addr, Kind::User);
                    if let Some((_, _, c)) = self.conds.iter().find(|(f, l, _)| same_file(f, file) && (*l == *line || *l == at)) {
                        self.cond_at.insert(addr, c.clone());
                    }
                    if at != *line {
                        (self.say)(Event::BreakpointAt { file: file.clone(), asked: *line, line: at });
                    }
                }
                None => (self.say)(Event::BreakpointAt { file: file.clone(), asked: *line, line: 0 }),
            }
        }
        self.wanted = list;
    }

    fn loaded_symbols(&self) -> bool {
        // dbghelp is set up once the process is created; before that nothing resolves.
        !self.threads.is_empty()
    }

    // --- stopping -----------------------------------------------------------------------------

    fn context(&self, tid: u32) -> Option<Ctx> {
        let h = *self.threads.get(&tid)?;
        let mut c: Ctx = unsafe { std::mem::zeroed() };
        c.0.ContextFlags = CONTEXT_FLAGS(0x0010_001B); // CONTEXT_FULL and the debug registers (AMD64)
        unsafe { GetThreadContext(h, &mut c.0) }.ok()?;
        Some(c)
    }

    fn set_context(&self, tid: u32, c: &Ctx) {
        if let Some(h) = self.threads.get(&tid) {
            unsafe {
                let _ = SetThreadContext(*h, &c.0);
            }
        }
    }

    fn exception(&mut self, tid: u32, info: &EXCEPTION_DEBUG_INFO) -> NTSTATUS {
        let code = info.ExceptionRecord.ExceptionCode.0 as u32;
        let addr = info.ExceptionRecord.ExceptionAddress as u64;
        match code {
            BREAKPOINT => {
                let Some(bp) = self.bps.get(&addr).copied() else {
                    if !self.loaded {
                        // Windows' own breakpoint as the program starts: on we go.
                        self.loaded = true;
                        return DBG_CONTINUE;
                    }
                    let reason = if std::mem::take(&mut self.pausing) { "pause" } else { "break" };
                    self.pause(tid, reason);
                    return DBG_CONTINUE;
                };
                let Some(mut c) = self.context(tid) else { return DBG_CONTINUE };
                // Back onto the instruction the breakpoint stood on, as it was.
                c.0.Rip = addr;
                self.write_byte(addr, bp.orig);
                match bp.kind {
                    Kind::Temp(rsp) if c.0.Rsp < rsp || self.step.as_ref().is_some_and(|s| s.tid != tid) => {
                        // The same place, deeper in a recursion or on another thread: not the return
                        // waited for.
                        c.0.EFlags |= TRAP_FLAG;
                        self.rearm = Some(addr);
                        self.set_context(tid, &c);
                        DBG_CONTINUE
                    }
                    Kind::Temp(_) => {
                        self.bps.remove(&addr);
                        match self.step.as_ref().map(|s| s.kind) {
                            Some(StepKind::Out) => {
                                self.step = None;
                                self.set_context(tid, &c);
                                self.pause(tid, "step");
                            }
                            Some(_) => {
                                // Back from the call stepped over: on, a line at a time.
                                if let Some(s) = self.step.as_mut() {
                                    s.prev_rip = c.0.Rip;
                                    s.prev_rsp = c.0.Rsp;
                                }
                                c.0.EFlags |= TRAP_FLAG;
                                self.set_context(tid, &c);
                            }
                            None => self.set_context(tid, &c),
                        }
                        DBG_CONTINUE
                    }
                    Kind::User | Kind::Panic => {
                        // Written again after this one instruction.
                        c.0.EFlags |= TRAP_FLAG;
                        self.rearm = Some(addr);
                        self.set_context(tid, &c);
                        // A condition that does not hold: on, as though no breakpoint were there.
                        if let Some(cond) = self.cond_at.get(&addr).cloned() {
                            let (expr, hits) = split_condition(&cond);
                            if !expr.is_empty() && self.holds(tid, &expr) == Some(false) {
                                return DBG_CONTINUE;
                            }
                            // Counted only when the condition held; stops only on the count asked.
                            let n = self.hits.entry(addr).or_insert(0);
                            *n += 1;
                            if hits.is_some_and(|h| !h.stops(*n)) {
                                return DBG_CONTINUE;
                            }
                        }
                        // Panics left to run their course (Exception Settings).
                        if bp.kind == Kind::Panic && !self.break_on_panic {
                            return DBG_CONTINUE;
                        }
                        // A breakpoint ends any step on the way to it.
                        self.forget_temps();
                        self.step = None;
                        self.pause(tid, if bp.kind == Kind::Panic { "panic" } else { "breakpoint" });
                        DBG_CONTINUE
                    }
                }
            }
            SINGLE_STEP if self.data_hit(tid).is_some() => {
                let what = self.data_hit(tid).unwrap_or_default();
                if let Some(mut c) = self.context(tid) {
                    c.0.Dr6 = 0;
                    self.set_context(tid, &c);
                }
                self.forget_temps();
                self.step = None;
                self.pause(tid, &format!("data breakpoint: {what} changed"));
                DBG_CONTINUE
            }
            SINGLE_STEP => {
                if let Some(a) = self.rearm.take() {
                    if self.bps.contains_key(&a) {
                        self.write_byte(a, 0xCC);
                    }
                }
                if self.step.as_ref().is_some_and(|s| s.tid == tid) {
                    self.step_tick(tid);
                }
                DBG_CONTINUE
            }
            _ if info.dwFirstChance == 0 => {
                // Nobody in the program took it: show where, then let it end the program.
                self.pause(tid, &format!("exception 0x{code:08X}"));
                DBG_EXCEPTION_NOT_HANDLED
            }
            _ if self.first_chance.contains(&code) => {
                // Asked for in Exception Settings: shown where it happened, before any handler.
                self.pause(tid, &format!("first-chance exception 0x{code:08X}"));
                DBG_EXCEPTION_NOT_HANDLED
            }
            _ => DBG_EXCEPTION_NOT_HANDLED,
        }
    }

    fn forget_temps(&mut self) {
        let temps: Vec<u64> = self.bps.iter().filter(|(_, b)| matches!(b.kind, Kind::Temp(_))).map(|(a, _)| *a).collect();
        for a in temps {
            self.disarm(a);
        }
    }

    /// The source file and line an address belongs to, and how far into that line's code it is.
    fn line_of(&self, addr: u64) -> Option<(String, u32, u32)> {
        let mut info = IMAGEHLP_LINEW64 { SizeOfStruct: std::mem::size_of::<IMAGEHLP_LINEW64>() as u32, ..Default::default() };
        let mut disp = 0u32;
        unsafe { SymGetLineFromAddrW64(self.process, addr, &mut disp, &mut info) }.ok()?;
        Some((self.resolve(&from_wide(info.FileName)).to_string_lossy().to_string(), info.LineNumber, disp))
    }

    /// One instruction further in a step: stop at a new line, step over calls, give up on code
    /// with no source.
    fn step_tick(&mut self, tid: u32) {
        let Some(mut c) = self.context(tid) else { return };
        let Some(step) = self.step.clone() else { return };
        let (rip, rsp) = (c.0.Rip, c.0.Rsp);
        // A call just made: the return address is on the stack, right after the call.
        let called = rsp + 8 == step.prev_rsp && self.read(rsp, 8).map(|b| u64::from_le_bytes(b.try_into().unwrap_or([0; 8]))).is_some_and(|ret| ret > step.prev_rip && ret <= step.prev_rip + 16);
        let here = self.line_of(rip);
        if std::env::var_os("PHAI_DEBUG_LOG").is_some() {
            eprintln!("tick {rip:#x} rsp {rsp:#x} (from rsp {:#x}) called {called} {:?}", step.rsp, here.as_ref().map(|(_, l, d)| (l, d)));
        }
        let has_source = here.as_ref().is_some_and(|(f, _, _)| Path::new(f).is_file());
        if called && (step.kind == StepKind::Over || !has_source) {
            // Over the call: run to where it returns.
            let ret = self.read(rsp, 8).map(|b| u64::from_le_bytes(b.try_into().unwrap_or([0; 8]))).unwrap_or(0);
            self.arm(ret, Kind::Temp(rsp + 8));
            return;
        }
        if called {
            // Into a function of the program: past its first line - the `fn` line, where its
            // arguments are not yet in place - to the first line of its body.
            if let Some(s) = self.step.as_mut() {
                s.from = here.as_ref().map(|(f, l, _)| (f.clone(), *l));
                s.prev_rip = rip;
                s.prev_rsp = rsp;
            }
            c.0.EFlags |= TRAP_FLAG;
            self.set_context(tid, &c);
            return;
        }
        let new_line = here.as_ref().is_some_and(|(f, l, d)| *d == 0 && Some((f.clone(), *l)) != step.from);
        let stop = match step.kind {
            StepKind::Into => new_line && has_source,
            StepKind::Over => new_line && has_source && rsp >= step.rsp,
            StepKind::Out => false,
        };
        if stop {
            self.step = None;
            self.pause(tid, "step");
            return;
        }
        if rsp > step.rsp && !has_source {
            // Out of the program's own code (past the end of main): run on.
            self.step = None;
            (self.say)(Event::Running);
            return;
        }
        if let Some(s) = self.step.as_mut() {
            s.prev_rip = rip;
            s.prev_rsp = rsp;
        }
        c.0.EFlags |= TRAP_FLAG;
        self.set_context(tid, &c);
    }

    /// Paused: tell what is where, then wait for what to do next.
    /// Every thread: its name, and where it is.
    fn thread_list(&self, current: u32) -> Vec<Thread> {
        let mut ids: Vec<u32> = self.threads.keys().copied().collect();
        ids.sort_by_key(|t| (*t != self.main_thread, *t));
        ids.into_iter()
            .map(|id| {
                let frames = self.frames(id);
                let mine = frames.iter().find(|f| f.file.as_ref().is_some_and(|p| self.resolve(&p.to_string_lossy()).starts_with(&self.cwd) && !p.to_string_lossy().contains("rustc")));
                let location = mine.or(frames.first()).map(|f| f.function.clone()).unwrap_or_default();
                let mut name = self.threads.get(&id).map(|h| thread_name(*h)).unwrap_or_default();
                if name.is_empty() && id == self.main_thread {
                    name = "main".into();
                }
                Thread { id, name, location, current: id == current }
            })
            .collect()
    }

    /// The program paused on thread `tid`: where, and what it holds, said; then what is asked.
    fn pause(&mut self, tid: u32, reason: &str) {
        let mut tid = tid;
        self.say_stopped(tid, reason);
        loop {
            let got = self.rx.recv();
            if std::env::var_os("PHAI_DEBUG_LOG").is_some() {
                eprintln!("paused on {tid}: {got:?}");
            }
            let Ok(cmd) = got else {
                if self.attached {
                    self.detaching = true;
                } else {
                    unsafe {
                        let _ = TerminateProcess(self.process, 1);
                    }
                }
                return;
            };
            match cmd {
                Command::Breakpoints(list) => {
                    self.set_breakpoints(list);
                    continue;
                }
                Command::Conditions(list) => {
                    // Changed: counted afresh, as Visual Studio does.
                    if list != self.conds {
                        self.hits.clear();
                    }
                    self.conds = list;
                    let w = self.wanted.clone();
                    self.set_breakpoints(w);
                    continue;
                }
                Command::Evaluate { id, exprs } => {
                    let vars = exprs.iter().map(|e| self.evaluate(tid, e)).collect();
                    (self.say)(Event::Values { id, vars });
                    continue;
                }
                Command::SetValue { expr, value } => {
                    let result = self.set_value(tid, &expr, &value).map(|_| self.locals(tid));
                    (self.say)(Event::ValueSet { expr, result });
                    continue;
                }
                Command::Pause => continue,
                Command::Memory { id, at, len } => {
                    let result = self.memory(tid, &at, len);
                    (self.say)(Event::Memory { id, result });
                    continue;
                }
                Command::DataBreakpoint(expr) => {
                    let result = self.add_data_breakpoint(tid, &expr);
                    (self.say)(Event::DataBreakpoint(result));
                    continue;
                }
                Command::ClearDataBreakpoints => {
                    self.watched.clear();
                    self.write_debug_registers();
                    continue;
                }
                Command::Exceptions { panics, first_chance } => {
                    self.break_on_panic = panics;
                    self.first_chance = first_chance;
                    continue;
                }
                Command::Thread(t) => {
                    if self.threads.contains_key(&t) {
                        tid = t;
                        self.say_stopped(tid, "thread");
                    }
                    continue;
                }
                Command::Detach => {
                    self.detaching = true;
                    return;
                }
                Command::Stop if self.attached => {
                    self.detaching = true;
                    return;
                }
                Command::Stop => unsafe {
                    let _ = TerminateProcess(self.process, 1);
                },
                Command::Continue => {}
                Command::StepInto | Command::StepOver | Command::StepOut => {
                    let Some(mut c) = self.context(tid) else { return };
                    let kind = match cmd {
                        Command::StepInto => StepKind::Into,
                        Command::StepOver => StepKind::Over,
                        _ => StepKind::Out,
                    };
                    let from = self.line_of(c.0.Rip).map(|(f, l, _)| (f, l));
                    self.step = Some(Step { tid, kind, from, rsp: c.0.Rsp, prev_rip: c.0.Rip, prev_rsp: c.0.Rsp });
                    if kind == StepKind::Out {
                        // To where this function returns: the caller's place, a shallower stack.
                        match self.return_address(tid) {
                            Some(ret) => self.arm(ret, Kind::Temp(c.0.Rsp + 8)),
                            None => self.step = None,
                        }
                    } else {
                        c.0.EFlags |= TRAP_FLAG;
                        self.set_context(tid, &c);
                    }
                }
            }
            (self.say)(Event::Running);
            return;
        }
    }

    /// Where an address or a variable is: `0x...` as written, else the variable's place, and how
    /// long its value is.
    fn address_of(&self, tid: u32, at: &str) -> Result<(u64, u64), String> {
        let at = at.trim();
        if let Some(hex) = at.strip_prefix("0x").or_else(|| at.strip_prefix("0X")) {
            return u64::from_str_radix(&hex.replace(['_', '`'], ""), 16).map(|a| (a, 8)).map_err(|_| format!("`{at}` is not an address"));
        }
        let (base, t, addr) = self.place(tid, at)?;
        let types = Types { process: self.process, base, engine: self };
        Ok((addr, types.len(t)))
    }

    /// The Memory window's bytes: `len` of them from an address or a variable.
    fn memory(&self, tid: u32, at: &str, len: usize) -> Result<(u64, Vec<Option<u8>>), String> {
        let (addr, _) = self.address_of(tid, at)?;
        let mut out = Vec::with_capacity(len);
        for row in 0..len.div_ceil(16) {
            let a = addr + row as u64 * 16;
            let n = 16.min(len - row * 16);
            match self.read(a, n) {
                Some(b) => out.extend(b.into_iter().map(Some)),
                None => out.extend((0..n as u64).map(|i| self.read(a + i, 1).map(|b| b[0]))),
            }
        }
        Ok((addr, out))
    }

    /// A data breakpoint on `expr`: its place and length given to the next free debug register.
    fn add_data_breakpoint(&mut self, tid: u32, expr: &str) -> Result<String, String> {
        let (addr, len) = self.address_of(tid, expr)?;
        if self.watched.len() >= 4 {
            return Err("four data breakpoints are all the processor has".into());
        }
        // The processor watches 1, 2, 4 or 8 bytes, aligned to their length.
        let len = match len {
            1 => 1,
            2 => 2,
            3 | 4 => 4,
            _ => 8,
        };
        if addr % len != 0 {
            return Err(format!("`{expr}` is not aligned for the processor to watch"));
        }
        self.watched.retain(|w| w.0 != addr);
        self.watched.push((addr, len, expr.to_string()));
        self.write_debug_registers();
        Ok(format!("{expr} (0x{addr:X}, {len} bytes)"))
    }

    /// DR0-DR3 and DR7 of every thread set to the data breakpoints: each one written to.
    fn write_debug_registers(&self) {
        let mut dr7 = 0u64;
        let mut regs = [0u64; 4];
        for (i, (addr, len, _)) in self.watched.iter().enumerate().take(4) {
            regs[i] = *addr;
            let size = match len {
                1 => 0b00,
                2 => 0b01,
                8 => 0b10,
                _ => 0b11,
            };
            dr7 |= 1 << (2 * i); // enabled for this thread
            dr7 |= 0b01 << (16 + 4 * i); // on a write
            dr7 |= size << (18 + 4 * i);
        }
        let tids: Vec<u32> = self.threads.keys().copied().collect();
        for tid in tids {
            if let Some(mut c) = self.context(tid) {
                (c.0.Dr0, c.0.Dr1, c.0.Dr2, c.0.Dr3) = (regs[0], regs[1], regs[2], regs[3]);
                c.0.Dr7 = dr7;
                self.set_context(tid, &c);
            }
        }
    }

    /// The data breakpoint a single-step exception on `tid` came from, if it did.
    fn data_hit(&self, tid: u32) -> Option<String> {
        let c = self.context(tid)?;
        (0..self.watched.len().min(4)).find(|i| c.0.Dr6 & (1 << i) != 0).map(|i| self.watched[i].2.clone())
    }

    fn say_stopped(&self, tid: u32, reason: &str) {
        let frames = self.frames(tid);
        let locals = self.locals(tid);
        let threads = self.thread_list(tid);
        let (file, line) = frames.first().map(|f| (f.file.clone(), f.line)).unwrap_or((None, 0));
        (self.say)(Event::Stopped { reason: reason.to_string(), file, line, frames, locals, threads });
    }

    // --- the stack and the locals -------------------------------------------------------------

    fn walk(&self, tid: u32) -> Vec<u64> {
        let Some(mut c) = self.context(tid) else { return vec![] };
        let Some(thread) = self.threads.get(&tid).copied() else { return vec![] };
        let mut sf = STACKFRAME64::default();
        sf.AddrPC.Offset = c.0.Rip;
        sf.AddrPC.Mode = AddrModeFlat;
        sf.AddrStack.Offset = c.0.Rsp;
        sf.AddrStack.Mode = AddrModeFlat;
        sf.AddrFrame.Offset = c.0.Rbp;
        sf.AddrFrame.Mode = AddrModeFlat;
        let mut out = Vec::new();
        for _ in 0..64 {
            let ok = unsafe { StackWalk64(0x8664 /* AMD64 */, self.process, thread, &mut sf, &mut c.0 as *mut CONTEXT as _, None, Some(table_access), Some(module_base), None) };
            if !ok.as_bool() || sf.AddrPC.Offset == 0 {
                break;
            }
            out.push(sf.AddrPC.Offset);
        }
        out
    }

    fn return_address(&self, tid: u32) -> Option<u64> {
        self.walk(tid).get(1).copied()
    }

    fn frames(&self, tid: u32) -> Vec<Frame> {
        self.walk(tid)
            .into_iter()
            .map(|pc| {
                let mut buf = SymBuf::new();
                let function = if unsafe { SymFromAddrW(self.process, pc, None, buf.info()) }.is_ok() { buf.name() } else { format!("0x{pc:016X}") };
                // A return address is just after its call: the call's own line is one byte back.
                let (file, line) = self.line_of(pc).or_else(|| self.line_of(pc.saturating_sub(1))).map(|(f, l, _)| (Some(PathBuf::from(f)), l)).unwrap_or((None, 0));
                Frame { function, file, line }
            })
            .collect()
    }

    /// The locals and parameters of the top frame, each with where it is (`None` when it is in a
    /// register or has no value at this point of the code).
    fn scope(&self, tid: u32) -> (Option<Ctx>, Vec<(Found, Option<u64>)>) {
        let Some(c) = self.context(tid) else { return (None, vec![]) };
        let frame = IMAGEHLP_STACK_FRAME { InstructionOffset: c.0.Rip, StackOffset: c.0.Rsp, FrameOffset: c.0.Rbp, ..Default::default() };
        unsafe {
            let _ = SymSetContext(self.process, &frame, None);
        }
        let mut found: Vec<Found> = Vec::new();
        let star = wide("*");
        unsafe {
            let _ = SymEnumSymbolsW(self.process, 0, PCWSTR(star.as_ptr()), Some(collect), Some(&mut found as *mut Vec<Found> as _));
        }
        let out = found
            .into_iter()
            .map(|f| {
                let base = if f.flags & 0x8_0000 != 0 {
                    None
                } else if f.flags & SYMFLAG_REGREL != 0 {
                    register(&c, f.register).map(|r| r.wrapping_add(f.address))
                } else if f.flags & SYMFLAG_FRAMEREL != 0 {
                    Some(c.0.Rbp.wrapping_add(f.address))
                } else if f.flags & SYMFLAG_REGISTER != 0 {
                    None
                } else {
                    Some(f.address)
                };
                (f, base)
            })
            .collect();
        (Some(c), out)
    }

    fn locals(&self, tid: u32) -> Vec<Var> {
        let (c, found) = self.scope(tid);
        let Some(c) = c else { return vec![] };
        let mut out = Vec::new();
        for (f, base) in found {
            let types = Types { process: self.process, base: f.mod_base, engine: self };
            let ty = types.name(f.type_index);
            let value = if f.flags & 0x8_0000 != 0 {
                // Not yet given a value at this point of the code (or no longer).
                "—".into()
            } else {
                match base {
                    Some(addr) => types.value(f.type_index, addr, 0),
                    None => match register(&c, f.register) {
                        Some(v) => types.format_bytes(f.type_index, &v.to_le_bytes()),
                        None => "?".into(),
                    },
                }
            };
            // Parameters first, then locals, in the order the PDB has them.
            out.push((f.flags & SYMFLAG_PARAMETER == 0, Var { name: f.name, ty, value }));
        }
        out.sort_by_key(|(local, _)| *local);
        out.into_iter().map(|(_, v)| v).collect()
    }

    /// Where a path of the top frame is - `order.customer.name`: its type, module and address.
    /// References are followed on the way, as Rust's `.` follows them.
    fn place(&self, tid: u32, path: &str) -> Result<(u64, u32, u64), String> {
        let mut parts = path.split('.').map(str::trim);
        let first = parts.next().unwrap_or_default();
        let (_, found) = self.scope(tid);
        // The innermost of several with the name: the last the PDB lists.
        let (f, base) = found.into_iter().rev().find(|(f, _)| f.name == first).ok_or_else(|| format!("no `{first}` here"))?;
        let mut addr = base.ok_or_else(|| format!("`{first}` has no value here"))?;
        let types = Types { process: self.process, base: f.mod_base, engine: self };
        let mut t = f.type_index;
        for part in parts {
            // Through typedefs and references to the struct itself.
            loop {
                match types.tag(t) {
                    TAG_TYPEDEF => t = types.inner(t).ok_or("?")?,
                    TAG_POINTER => {
                        addr = types.read_u64(addr).ok_or("unreadable")?;
                        t = types.inner(t).ok_or("?")?;
                    }
                    _ => break,
                }
            }
            let (off, ty) = types.field(t, part).or_else(|| types.field(t, &format!("__{part}"))).ok_or_else(|| format!("no field `{part}`"))?;
            addr += off as u64;
            t = ty;
        }
        Ok((f.mod_base, t, addr))
    }

    /// An expression's value, as Watch shows it: a path, or a comparison of one.
    fn evaluate(&self, tid: u32, expr: &str) -> Var {
        let e = expr.trim();
        if let Some((l, op, r)) = split_comparison(e) {
            let value = match (self.text_of(tid, l), literal_or(self, tid, r)) {
                (Ok(a), Ok(b)) => compare(&a, op, &b).to_string(),
                (Err(x), _) | (_, Err(x)) => x,
            };
            return Var { name: e.to_string(), ty: "bool".into(), value };
        }
        match self.place(tid, e) {
            Ok((m, t, addr)) => {
                let types = Types { process: self.process, base: m, engine: self };
                Var { name: e.to_string(), ty: types.name(t), value: types.value(t, addr, 0) }
            }
            Err(x) => Var { name: e.to_string(), ty: String::new(), value: x },
        }
    }

    /// Writes `value` over a number, `bool` or `char` in the program's memory.
    fn set_value(&self, tid: u32, expr: &str, value: &str) -> Result<(), String> {
        let (m, mut t, addr) = self.place(tid, expr.trim())?;
        let types = Types { process: self.process, base: m, engine: self };
        while types.tag(t) == TAG_TYPEDEF {
            t = types.inner(t).ok_or("?")?;
        }
        if types.tag(t) == TAG_UDT {
            return self.set_text(&types, t, addr, value);
        }
        if types.tag(t) != TAG_BASE {
            return Err("only a number, bool, char, &str or String can be changed here".into());
        }
        let bt = types.info_u32(t, TI_GET_BASETYPE).unwrap_or(0);
        let len = types.len(t) as usize;
        let bytes = encode_value(bt, len, value.trim())?;
        unsafe { WriteProcessMemory(self.process, addr as _, bytes.as_ptr() as _, bytes.len(), None) }.map_err(|e| e.to_string())
    }

    /// A `&str` or `String` given new words (quoted or not). A `&str` points at a copy made in the
    /// program's memory; a `String` takes them in its own buffer, so they may be no longer than
    /// its capacity - its allocator must be able to free that buffer later.
    fn set_text(&self, types: &Types, t: u32, addr: u64, value: &str) -> Result<(), String> {
        let v = value.trim();
        let words = if v.len() >= 2 && v.starts_with('"') && v.ends_with('"') { unescape(&v[1..v.len() - 1]) } else { v.to_string() };
        let bytes = words.as_bytes();
        let name = types.sym_name(t).unwrap_or_default();
        let write = |at: u64, b: &[u8]| unsafe { WriteProcessMemory(self.process, at as _, b.as_ptr() as _, b.len(), None) }.map_err(|e| e.to_string());
        if name.starts_with("ref$<str$>") || name == "&str" {
            let (Some(p), Some(l)) = (types.path(t, &["data_ptr"]), types.path(t, &["length"])) else { return Err("this &str's parts are not in the symbols".into()) };
            let at = if bytes.is_empty() {
                1 // a dangling, well-aligned pointer, as Rust's own empty strings have
            } else {
                let mem = unsafe { VirtualAllocEx(self.process, None, bytes.len(), MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE) } as u64;
                if mem == 0 {
                    return Err("no memory could be had in the program".into());
                }
                write(mem, bytes)?;
                mem
            };
            write(addr + p, &at.to_le_bytes())?;
            return write(addr + l, &(bytes.len() as u64).to_le_bytes());
        }
        if name == "alloc::string::String" {
            let ptr = types.path(t, &["vec", "buf", "inner", "ptr"]).or_else(|| types.path(t, &["vec", "buf", "ptr"]));
            let cap = [&["vec", "buf", "inner", "cap", "__0"][..], &["vec", "buf", "inner", "cap"], &["vec", "buf", "cap", "__0"], &["vec", "buf", "cap"]].iter().find_map(|p| types.path(t, p));
            let len = types.path(t, &["vec", "len"]);
            let (Some(p), Some(c), Some(l)) = (ptr, cap, len) else { return Err("this String's parts are not in the symbols".into()) };
            let (Some(at), Some(cap)) = (types.read_u64(addr + p), types.read_u64(addr + c)) else { return Err("the String cannot be read".into()) };
            if bytes.len() as u64 > cap {
                return Err(format!("{} bytes do not fit: this String has room for {cap}", bytes.len()));
            }
            write(at, bytes)?;
            return write(addr + l, &(bytes.len() as u64).to_le_bytes());
        }
        Err(format!("a {} cannot be typed over - only its number, bool, char or text fields", rust_name(&name)))
    }

    /// A path's value as plain text: a number, `true`, a string's words without quotes.
    fn text_of(&self, tid: u32, path: &str) -> Result<String, String> {
        let (m, t, addr) = self.place(tid, path)?;
        let types = Types { process: self.process, base: m, engine: self };
        let v = types.value(t, addr, 0);
        Ok(unquote(&v))
    }

    /// Whether a breakpoint's condition holds; `None` when it cannot be worked out (then the
    /// breakpoint stops, as Visual Studio's does, so the mistake is seen).
    fn holds(&self, tid: u32, cond: &str) -> Option<bool> {
        let v = self.evaluate(tid, cond);
        match v.value.as_str() {
            "true" => Some(true),
            "false" => Some(false),
            "0" => Some(false),
            x if x.parse::<f64>().is_ok() => Some(true),
            _ => None,
        }
    }
}

/// `\"`, `\\`, `\n`, `\t` as the characters they stand for.
fn unescape(s: &str) -> String {
    let mut out = String::new();
    let mut it = s.chars();
    while let Some(c) = it.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match it.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('0') => out.push('\0'),
            Some(o) => out.push(o),
            None => out.push('\\'),
        }
    }
    out
}

/// The bytes a value typed in takes in memory, for a base type of `len` bytes.
fn encode_value(bt: u32, len: usize, v: &str) -> Result<Vec<u8>, String> {
    let bad = || format!("`{v}` is not a value of this type");
    let mut out = match bt {
        6 | 13 => {
            let n: i128 = v.replace('_', "").parse().map_err(|_| bad())?;
            let bits = len * 8;
            if bits < 128 && (n < -(1i128 << (bits - 1)) || n >= 1i128 << (bits - 1)) {
                return Err(format!("{n} does not fit in {bits} bits"));
            }
            n.to_le_bytes().to_vec()
        }
        7 | 14 => {
            let n: u128 = v.replace('_', "").parse().map_err(|_| bad())?;
            if len < 16 && n >= 1u128 << (len * 8) {
                return Err(format!("{n} does not fit in {} bits", len * 8));
            }
            n.to_le_bytes().to_vec()
        }
        8 if len == 4 => v.parse::<f32>().map_err(|_| bad())?.to_le_bytes().to_vec(),
        8 => v.parse::<f64>().map_err(|_| bad())?.to_le_bytes().to_vec(),
        10 => vec![match v {
            "true" | "1" => 1,
            "false" | "0" => 0,
            _ => return Err(bad()),
        }],
        2 | 3 | 32 => {
            let c = unquote(v).chars().next().ok_or_else(bad)?;
            (c as u32).to_le_bytes().to_vec()
        }
        _ => return Err("this type cannot be changed here".into()),
    };
    out.resize(len, 0);
    Ok(out)
}

/// `x >= 5` as (`x`, `>=`, `5`); `None` for a plain path.
fn split_comparison(e: &str) -> Option<(&str, &str, &str)> {
    for op in ["==", "!=", "<=", ">=", "<", ">"] {
        // Outside quotes only.
        let mut quoted = false;
        for (i, ch) in e.char_indices() {
            if ch == '"' {
                quoted = !quoted;
            }
            if !quoted && e[i..].starts_with(op) {
                return Some((e[..i].trim(), op, e[i + op.len()..].trim()));
            }
        }
    }
    None
}

fn unquote(v: &str) -> String {
    let v = v.trim();
    if v.len() >= 2 && (v.starts_with('"') && v.ends_with('"') || v.starts_with('\'') && v.ends_with('\'')) {
        v[1..v.len() - 1].to_string()
    } else {
        v.to_string()
    }
}

/// The right side of a comparison: a literal, or another path.
fn literal_or(e: &Engine, tid: u32, r: &str) -> Result<String, String> {
    let r = r.trim();
    if r.starts_with('"') || r.starts_with('\'') || r == "true" || r == "false" || r.parse::<f64>().is_ok() {
        return Ok(unquote(r));
    }
    e.text_of(tid, r)
}

/// Two values compared as numbers when both are, else as text.
fn compare(a: &str, op: &str, b: &str) -> bool {
    use std::cmp::Ordering::*;
    let ord = match (a.parse::<f64>(), b.parse::<f64>()) {
        (Ok(x), Ok(y)) => x.partial_cmp(&y).unwrap_or(Equal),
        _ => a.cmp(b),
    };
    match op {
        "==" => ord == Equal,
        "!=" => ord != Equal,
        "<" => ord == Less,
        "<=" => ord != Greater,
        ">" => ord == Greater,
        _ => ord != Less,
    }
}

unsafe extern "system" fn table_access(h: HANDLE, a: u64) -> *mut core::ffi::c_void {
    unsafe { SymFunctionTableAccess64(h, a) }
}

unsafe extern "system" fn module_base(h: HANDLE, a: u64) -> u64 {
    unsafe { SymGetModuleBase64(h, a) }
}

/// A register by its CodeView number, as a PDB names the register a local is kept relative to.
fn register(c: &Ctx, n: u32) -> Option<u64> {
    let c = &c.0;
    Some(match n {
        328 => c.Rax,
        329 => c.Rbx,
        330 => c.Rcx,
        331 => c.Rdx,
        332 => c.Rsi,
        333 => c.Rdi,
        334 => c.Rbp,
        335 => c.Rsp,
        336 => c.R8,
        337 => c.R9,
        338 => c.R10,
        339 => c.R11,
        340 => c.R12,
        341 => c.R13,
        342 => c.R14,
        343 => c.R15,
        _ => return None,
    })
}

struct Found {
    name: String,
    type_index: u32,
    mod_base: u64,
    flags: u32,
    register: u32,
    address: u64,
}

unsafe extern "system" fn collect(info: *const SYMBOL_INFOW, _size: u32, user: *const core::ffi::c_void) -> windows::core::BOOL {
    unsafe {
        let i = &*info;
        let out = &mut *(user as *mut Vec<Found>);
        if i.Flags.0 & (SYMFLAG_LOCAL | SYMFLAG_PARAMETER) != 0 {
            let name = String::from_utf16_lossy(std::slice::from_raw_parts(i.Name.as_ptr(), i.NameLen as usize)).trim_end_matches('\0').to_string();
            out.push(Found { name, type_index: i.TypeIndex, mod_base: i.ModBase, flags: i.Flags.0, register: i.Register, address: i.Address });
        }
    }
    true.into()
}

/// A `SYMBOL_INFOW` with room for its name after it.
struct SymBuf(Vec<u64>);

impl SymBuf {
    fn new() -> SymBuf {
        let mut b = SymBuf(vec![0u64; (std::mem::size_of::<SYMBOL_INFOW>() + 1024) / 8]);
        let i = b.info();
        unsafe {
            (*i).SizeOfStruct = std::mem::size_of::<SYMBOL_INFOW>() as u32;
            (*i).MaxNameLen = 500;
        }
        b
    }
    fn info(&mut self) -> *mut SYMBOL_INFOW {
        self.0.as_mut_ptr() as *mut SYMBOL_INFOW
    }
    fn name(&mut self) -> String {
        let i = self.info();
        unsafe { String::from_utf16_lossy(std::slice::from_raw_parts((*i).Name.as_ptr(), (*i).NameLen as usize)).trim_end_matches('\0').to_string() }
    }
}

// --- values, through the types in the PDB ----------------------------------------------------

const TAG_UDT: u32 = 11;
const TAG_ENUM: u32 = 12;
const TAG_POINTER: u32 = 14;
const TAG_ARRAY: u32 = 15;
const TAG_BASE: u32 = 16;
const TAG_TYPEDEF: u32 = 17;

struct Types<'a> {
    process: HANDLE,
    base: u64,
    engine: &'a Engine,
}

impl Types<'_> {
    fn info_u32(&self, t: u32, what: IMAGEHLP_SYMBOL_TYPE_INFO) -> Option<u32> {
        let mut v = 0u32;
        unsafe { SymGetTypeInfo(self.process, self.base, t, what, &mut v as *mut u32 as _) }.ok()?;
        Some(v)
    }
    fn info_u64(&self, t: u32, what: IMAGEHLP_SYMBOL_TYPE_INFO) -> Option<u64> {
        let mut v = 0u64;
        unsafe { SymGetTypeInfo(self.process, self.base, t, what, &mut v as *mut u64 as _) }.ok()?;
        Some(v)
    }
    fn sym_name(&self, t: u32) -> Option<String> {
        let mut p: *mut u16 = std::ptr::null_mut();
        unsafe { SymGetTypeInfo(self.process, self.base, t, TI_GET_SYMNAME, &mut p as *mut *mut u16 as _) }.ok()?;
        if p.is_null() {
            return None;
        }
        let s = from_wide(PWSTR(p));
        unsafe {
            let _ = windows::Win32::Foundation::LocalFree(Some(windows::Win32::Foundation::HLOCAL(p as _)));
        }
        Some(s)
    }
    fn tag(&self, t: u32) -> u32 {
        self.info_u32(t, TI_GET_SYMTAG).unwrap_or(0)
    }
    fn len(&self, t: u32) -> u64 {
        self.info_u64(t, TI_GET_LENGTH).unwrap_or(0)
    }
    /// What a typedef or pointer points at.
    fn inner(&self, t: u32) -> Option<u32> {
        self.info_u32(t, TI_GET_TYPE)
    }
    fn children(&self, t: u32) -> Vec<(String, u32, u32)> {
        let n = self.info_u32(t, TI_GET_CHILDRENCOUNT).unwrap_or(0) as usize;
        if n == 0 || n > 256 {
            return vec![];
        }
        let mut buf = vec![0u32; n + 2];
        buf[0] = n as u32;
        if unsafe { SymGetTypeInfo(self.process, self.base, t, TI_FINDCHILDREN, buf.as_mut_ptr() as _) }.is_err() {
            return vec![];
        }
        buf[2..]
            .iter()
            .filter_map(|&c| {
                let name = self.sym_name(c)?;
                let off = self.info_u32(c, TI_GET_OFFSET)?;
                let ty = self.info_u32(c, TI_GET_TYPEID)?;
                Some((name, off, ty))
            })
            .collect()
    }
    fn field(&self, t: u32, name: &str) -> Option<(u32, u32)> {
        self.children(t).into_iter().find(|(n, _, _)| n == name).map(|(_, o, ty)| (o, ty))
    }
    /// The offset of a field path - `vec.len` - in a struct.
    fn path(&self, t: u32, names: &[&str]) -> Option<u64> {
        let mut t = t;
        let mut off = 0u64;
        for n in names {
            let (o, ty) = self.field(t, n)?;
            off += o as u64;
            t = ty;
        }
        Some(off)
    }

    fn name(&self, t: u32) -> String {
        match self.tag(t) {
            TAG_BASE => base_name(self.info_u32(t, TI_GET_BASETYPE).unwrap_or(0), self.len(t)).to_string(),
            TAG_POINTER => format!("*{}", self.inner(t).map(|i| self.name(i)).unwrap_or_default()),
            TAG_TYPEDEF => self.inner(t).map(|i| self.name(i)).unwrap_or_default(),
            TAG_ARRAY => format!("[{}]", self.inner(t).map(|i| self.name(i)).unwrap_or_default()),
            _ => rust_name(&self.sym_name(t).unwrap_or_default()),
        }
    }

    fn format_bytes(&self, t: u32, b: &[u8]) -> String {
        match self.tag(t) {
            TAG_BASE => format_base(self.info_u32(t, TI_GET_BASETYPE).unwrap_or(0), b),
            _ => format!("0x{}", b.iter().rev().map(|x| format!("{x:02x}")).collect::<String>()),
        }
    }

    fn value(&self, t: u32, addr: u64, depth: u32) -> String {
        let len = self.len(t).min(4096) as usize;
        match self.tag(t) {
            TAG_TYPEDEF => self.inner(t).map(|i| self.value(i, addr, depth)).unwrap_or_default(),
            TAG_BASE => match self.engine.read(addr, len) {
                Some(b) => format_base(self.info_u32(t, TI_GET_BASETYPE).unwrap_or(0), &b),
                None => "?".into(),
            },
            TAG_ENUM => match self.engine.read(addr, len.max(1)) {
                Some(b) => format_base(7, &b),
                None => "?".into(),
            },
            TAG_POINTER => {
                let Some(p) = self.engine.read(addr, 8).map(|b| u64::from_le_bytes(b.try_into().unwrap_or([0; 8]))) else { return "?".into() };
                // A reference to a number shows the number too.
                match self.inner(t) {
                    Some(i) if self.tag(i) == TAG_BASE && p != 0 && depth < 2 => format!("0x{p:x} → {}", self.value(i, p, depth + 1)),
                    _ => format!("0x{p:x}"),
                }
            }
            TAG_ARRAY => format!("[{} bytes]", len),
            TAG_UDT => self.udt(t, addr, depth),
            _ => "?".into(),
        }
    }

    fn read_u64(&self, addr: u64) -> Option<u64> {
        self.engine.read(addr, 8).map(|b| u64::from_le_bytes(b.try_into().unwrap_or([0; 8])))
    }

    fn text_at(&self, ptr: u64, len: u64) -> String {
        let n = len.min(200) as usize;
        let s = self.engine.read(ptr, n).map(|b| String::from_utf8_lossy(&b).to_string()).unwrap_or_else(|| "?".into());
        // Quotes, backslashes and control characters escaped; Thai and the rest as written.
        let shown: String = s.chars().flat_map(|c| if c == '"' || c == '\\' || c.is_control() { c.escape_default().collect::<Vec<_>>() } else { vec![c] }).collect();
        format!("\"{shown}{}\"", if len > 200 { "…" } else { "" })
    }

    fn udt(&self, t: u32, addr: u64, depth: u32) -> String {
        let name = self.sym_name(t).unwrap_or_default();
        // &str: a pointer and a length.
        if name.starts_with("ref$<str$>") || name == "&str" {
            if let (Some(p), Some(l)) = (self.path(t, &["data_ptr"]), self.path(t, &["length"])) {
                if let (Some(ptr), Some(len)) = (self.read_u64(addr + p), self.read_u64(addr + l)) {
                    return self.text_at(ptr, len);
                }
            }
        }
        // String: a Vec<u8> inside.
        if name == "alloc::string::String" {
            let ptr = self.path(t, &["vec", "buf", "inner", "ptr"]).or_else(|| self.path(t, &["vec", "buf", "ptr"]));
            let len = self.path(t, &["vec", "len"]);
            if let (Some(p), Some(l)) = (ptr, len) {
                if let (Some(ptr), Some(len)) = (self.read_u64(addr + p), self.read_u64(addr + l)) {
                    return self.text_at(ptr, len);
                }
            }
        }
        if name.starts_with("alloc::vec::Vec<") {
            if let Some(l) = self.path(t, &["len"]).and_then(|l| self.read_u64(addr + l)) {
                return format!("{{ len={l} }}");
            }
        }
        if depth >= 1 {
            return "{…}".into();
        }
        let fields = self.children(t);
        let parts: Vec<String> = fields.iter().take(6).map(|(n, o, ty)| format!("{n}: {}", self.value(*ty, addr + *o as u64, depth + 1))).collect();
        if parts.is_empty() {
            return format!("{{{}}}", rust_name(&name));
        }
        format!("{{ {}{} }}", parts.join(", "), if fields.len() > 6 { ", …" } else { "" })
    }
}

/// A Rust name for a base type, by dbghelp's base type and its size.
fn base_name(bt: u32, len: u64) -> &'static str {
    match (bt, len) {
        (6, 1) | (13, 1) => "i8",
        (6, 2) | (13, 2) => "i16",
        (6, 4) | (13, 4) => "i32",
        (6, 8) | (13, 8) => "i64",
        (6, 16) => "i128",
        (7, 1) | (14, 1) => "u8",
        (7, 2) | (14, 2) => "u16",
        (7, 4) | (14, 4) => "u32",
        (7, 8) | (14, 8) => "u64",
        (7, 16) => "u128",
        (8, 4) => "f32",
        (8, 8) => "f64",
        (10, _) => "bool",
        (2, _) | (3, _) | (32, _) => "char",
        (1, _) => "()",
        _ => "?",
    }
}

fn format_base(bt: u32, b: &[u8]) -> String {
    let u = |b: &[u8]| -> u128 {
        let mut v = [0u8; 16];
        v[..b.len().min(16)].copy_from_slice(&b[..b.len().min(16)]);
        u128::from_le_bytes(v)
    };
    match bt {
        6 | 13 => {
            let bits = b.len() * 8;
            let v = u(b);
            let signed = if bits >= 128 { v as i128 } else { ((v << (128 - bits)) as i128) >> (128 - bits) };
            signed.to_string()
        }
        7 | 14 => u(b).to_string(),
        8 if b.len() == 4 => f32::from_le_bytes(b.try_into().unwrap_or([0; 4])).to_string(),
        8 if b.len() == 8 => f64::from_le_bytes(b.try_into().unwrap_or([0; 8])).to_string(),
        10 => (b.first().copied().unwrap_or(0) != 0).to_string(),
        2 | 3 | 32 => char::from_u32(u(b) as u32).map(|c| format!("'{c}'")).unwrap_or_else(|| "?".into()),
        _ => format!("0x{}", b.iter().rev().map(|x| format!("{x:02x}")).collect::<String>()),
    }
}

/// The PDB's spelling of a Rust type made readable: `ref$<str$>` is `&str`.
fn rust_name(s: &str) -> String {
    s.replace("ref$<str$>", "&str").replace("ref_mut$<", "&mut ").replace("ref$<", "&").replace("$>", ">").replace('$', "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_read_as_rust_writes_them() {
        assert_eq!(format_base(6, &(-5i32).to_le_bytes()), "-5");
        assert_eq!(format_base(7, &42u64.to_le_bytes()), "42");
        assert_eq!(format_base(8, &1.5f64.to_le_bytes()), "1.5");
        assert_eq!(format_base(10, &[1]), "true");
        assert_eq!(format_base(32, &('ไ' as u32).to_le_bytes()), "'ไ'");
        assert_eq!(base_name(6, 4), "i32");
        assert_eq!(rust_name("ref$<str$>"), "&str");
    }

    #[test]
    fn conditions_split_and_compare() {
        assert_eq!(split_comparison("x >= 5"), Some(("x", ">=", "5")));
        assert_eq!(split_comparison("name == \"a<b\""), Some(("name", "==", "\"a<b\"")));
        assert_eq!(split_comparison("order.total"), None);
        assert!(compare("42", ">", "5") && !compare("42", "<", "5"));
        assert!(compare("phai", "==", "phai") && compare("10", "==", "10.0"));
        assert_eq!(unquote("\"ไผ่\""), "ไผ่");
    }

    #[test]
    fn hit_counts_parse_and_ride_with_the_condition() {
        assert_eq!(Hits::parse("3"), Ok(Some(Hits::Equal(3))));
        assert_eq!(Hits::parse(">= 2"), Ok(Some(Hits::AtLeast(2))));
        assert_eq!(Hits::parse("multiple of 4"), Ok(Some(Hits::Multiple(4))));
        assert_eq!(Hits::parse(" "), Ok(None));
        assert!(Hits::parse("% 0").is_err() && Hits::parse("abc").is_err());
        assert!(Hits::Multiple(2).stops(4) && !Hits::Multiple(2).stops(3));
        let c = join_condition("x > 5", Some(Hits::AtLeast(3)));
        assert_eq!(split_condition(&c), ("x > 5".to_string(), Some(Hits::AtLeast(3))));
        assert_eq!(split_condition("x > 5"), ("x > 5".to_string(), None));
        assert_eq!(split_condition(&join_condition("", Some(Hits::Equal(2)))), (String::new(), Some(Hits::Equal(2))));
    }

    #[test]
    fn typed_values_become_bytes() {
        assert_eq!(encode_value(6, 4, "-2").unwrap(), (-2i32).to_le_bytes());
        assert_eq!(encode_value(7, 1, "255").unwrap(), [255]);
        assert!(encode_value(7, 1, "256").is_err());
        assert!(encode_value(6, 1, "128").is_err());
        assert_eq!(encode_value(8, 8, "1.5").unwrap(), 1.5f64.to_le_bytes());
        assert_eq!(encode_value(10, 1, "true").unwrap(), [1]);
        assert_eq!(encode_value(32, 4, "'ไ'").unwrap(), ('ไ' as u32).to_le_bytes());
    }
}
