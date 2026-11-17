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
use windows::Win32::System::Threading::{
    CreateProcessW, TerminateProcess, DEBUG_ONLY_THIS_PROCESS, CREATE_NEW_CONSOLE, PROCESS_INFORMATION, STARTUPINFOW,
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
    /// End the program.
    Stop,
    /// The breakpoints, every one, as file and line from 1.
    Breakpoints(Vec<(PathBuf, u32)>),
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
    Stopped { reason: String, file: Option<PathBuf>, line: u32, frames: Vec<Frame>, locals: Vec<Var> },
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
}

/// A program running under the debugger.
pub struct Session {
    tx: Sender<Command>,
}

impl Session {
    /// Starts `exe` (in `cwd`) under the debugger with `breakpoints` set; `on_event` hears what
    /// happens, on the debugger's own thread.
    pub fn launch(exe: &Path, cwd: &Path, breakpoints: Vec<(PathBuf, u32)>, on_event: impl Fn(Event) + Send + 'static) -> Session {
        let (tx, rx) = channel();
        let (exe, cwd) = (exe.to_path_buf(), cwd.to_path_buf());
        std::thread::spawn(move || {
            match Engine::start(&exe, &cwd, breakpoints, rx, Box::new(on_event)) {
                Ok(mut e) => e.run(),
                Err((msg, say)) => say(Event::Failed(msg)),
            }
        });
        Session { tx }
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
}

/// Whether two paths are one file, as Windows compares them: case and slashes aside.
fn same_file(a: &Path, b: &Path) -> bool {
    let n = |p: &Path| p.to_string_lossy().replace('/', "\\").trim_start_matches(r"\?\").to_lowercase();
    n(a) == n(b)
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
        Ok(Engine { process: pi.hProcess, exe: exe.to_path_buf(), cwd: cwd.to_path_buf(), threads: HashMap::new(), bps: HashMap::new(), wanted, rearm: None, step: None, loaded: false, rx, say, pausing: false })
    }

    fn run(&mut self) {
        loop {
            let mut ev = DEBUG_EVENT::default();
            if unsafe { WaitForDebugEvent(&mut ev, 50) }.is_err() {
                // Nothing from the program: what the person asks while it runs.
                match self.rx.try_recv() {
                    Ok(Command::Stop) => unsafe {
                        let _ = TerminateProcess(self.process, 1);
                    },
                    Ok(Command::Pause) => unsafe {
                        self.pausing = true;
                        let _ = DebugBreakProcess(self.process);
                    },
                    Ok(Command::Breakpoints(list)) => self.set_breakpoints(list),
                    Ok(_) => {}
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
                        self.load_symbols(info.hFile, info.lpBaseOfImage as u64);
                        if !info.hFile.is_invalid() {
                            let _ = CloseHandle(info.hFile);
                        }
                    }
                    CREATE_THREAD_DEBUG_EVENT => {
                        self.threads.insert(tid, ev.u.CreateThread.hThread);
                    }
                    EXIT_THREAD_DEBUG_EVENT => {
                        self.threads.remove(&tid);
                    }
                    LOAD_DLL_DEBUG_EVENT => {
                        let f = ev.u.LoadDll.hFile;
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
            for n in ["main", "prog::main", "rust_panic"] {
                let a = self.symbol_address(n);
                eprintln!("{n}: {a:x?} {:?}", a.and_then(|a| self.line_of(a)));
            }
        }
        let wanted = std::mem::take(&mut self.wanted);
        self.set_breakpoints(wanted);
        if let Some(addr) = self.symbol_address("rust_panic") {
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
        for (file, line) in &list {
            match self.line_address(file, *line) {
                Some((addr, at)) => {
                    self.arm(addr, Kind::User);
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
        c.0.ContextFlags = CONTEXT_FLAGS(0x0010_000B); // CONTEXT_FULL (AMD64)
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
                    Kind::Temp(rsp) if c.0.Rsp < rsp => {
                        // The same place, deeper in a recursion: not the return waited for.
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
                        // A breakpoint ends any step on the way to it.
                        self.forget_temps();
                        self.step = None;
                        self.pause(tid, if bp.kind == Kind::Panic { "panic" } else { "breakpoint" });
                        DBG_CONTINUE
                    }
                }
            }
            SINGLE_STEP => {
                if let Some(a) = self.rearm.take() {
                    if self.bps.contains_key(&a) {
                        self.write_byte(a, 0xCC);
                    }
                }
                if self.step.is_some() {
                    self.step_tick(tid);
                }
                DBG_CONTINUE
            }
            _ if info.dwFirstChance == 0 => {
                // Nobody in the program took it: show where, then let it end the program.
                self.pause(tid, &format!("exception 0x{code:08X}"));
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
    fn pause(&mut self, tid: u32, reason: &str) {
        let frames = self.frames(tid);
        let locals = self.locals(tid);
        let (file, line) = frames.first().map(|f| (f.file.clone(), f.line)).unwrap_or((None, 0));
        (self.say)(Event::Stopped { reason: reason.to_string(), file, line, frames, locals });
        loop {
            let Ok(cmd) = self.rx.recv() else {
                unsafe {
                    let _ = TerminateProcess(self.process, 1);
                }
                return;
            };
            match cmd {
                Command::Breakpoints(list) => {
                    self.set_breakpoints(list);
                    continue;
                }
                Command::Pause => continue,
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
                    self.step = Some(Step { kind, from, rsp: c.0.Rsp, prev_rip: c.0.Rip, prev_rsp: c.0.Rsp });
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

    fn locals(&self, tid: u32) -> Vec<Var> {
        let Some(c) = self.context(tid) else { return vec![] };
        let frame = IMAGEHLP_STACK_FRAME { InstructionOffset: c.0.Rip, StackOffset: c.0.Rsp, FrameOffset: c.0.Rbp, ..Default::default() };
        unsafe {
            let _ = SymSetContext(self.process, &frame, None);
        }
        let mut found: Vec<Found> = Vec::new();
        let star = wide("*");
        unsafe {
            let _ = SymEnumSymbolsW(self.process, 0, PCWSTR(star.as_ptr()), Some(collect), Some(&mut found as *mut Vec<Found> as _));
        }
        let mut out = Vec::new();
        for f in found {
            let base = if f.flags & SYMFLAG_REGREL != 0 {
                register(&c, f.register).map(|r| r.wrapping_add(f.address))
            } else if f.flags & SYMFLAG_FRAMEREL != 0 {
                Some(c.0.Rbp.wrapping_add(f.address))
            } else if f.flags & SYMFLAG_REGISTER != 0 {
                None
            } else {
                Some(f.address)
            };
            let types = Types { process: self.process, base: f.mod_base, engine: self };
            let ty = types.name(f.type_index);
            // Not yet given a value at this point of the code (or no longer).
            if f.flags & 0x8_0000 != 0 {
                out.push((f.flags & SYMFLAG_PARAMETER == 0, Var { name: f.name, ty, value: "—".into() }));
                continue;
            }
            let value = match base {
                Some(addr) => types.value(f.type_index, addr, 0),
                None => match register(&c, f.register) {
                    Some(v) => types.format_bytes(f.type_index, &v.to_le_bytes()),
                    None => "?".into(),
                },
            };
            // Parameters first, then locals, in the order the PDB has them.
            out.push((f.flags & SYMFLAG_PARAMETER == 0, Var { name: f.name, ty, value }));
        }
        out.sort_by_key(|(local, _)| *local);
        out.into_iter().map(|(_, v)| v).collect()
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
}
