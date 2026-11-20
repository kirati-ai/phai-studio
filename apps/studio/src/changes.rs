//! The bar beside lines changed since git's last commit, as Visual Studio's and Android Studio's
//! editors have: `git diff -U0 HEAD` of the file read into which lines were added, changed, or
//! had lines taken out below them.

use std::path::Path;
use std::process::Command;

use phai_editor::view::Change;

/// The changed lines (from 0) in a `git diff -U0` of one file.
pub fn parse(diff: &str) -> Vec<(usize, Change)> {
    let mut out = Vec::new();
    for l in diff.lines() {
        let Some(rest) = l.strip_prefix("@@ -") else { continue };
        let Some((old, rest)) = rest.split_once(" +") else { continue };
        let Some((new, _)) = rest.split_once(" @@") else { continue };
        let span = |s: &str| -> (usize, usize) {
            match s.split_once(',') {
                Some((a, n)) => (a.parse().unwrap_or(0), n.parse().unwrap_or(0)),
                None => (s.parse().unwrap_or(0), 1),
            }
        };
        let ((_, removed), (start, added)) = (span(old), span(new));
        if added == 0 {
            // Taken out: marked on the line above where they were (the first line when at the top).
            out.push((start.saturating_sub(1), Change::Removed));
            continue;
        }
        for k in 0..added {
            let line = start - 1 + k;
            out.push((line, if k < removed { Change::Modified } else { Change::Added }));
        }
    }
    out
}

/// The changed lines of `file` against git's last commit; none when it is not in a repository,
/// or git is not there. A file git does not know yet: every line added.
pub fn of_file(file: &Path) -> Vec<(usize, Change)> {
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    use std::os::windows::process::CommandExt;
    let Some(dir) = file.parent() else { return Vec::new() };
    let Some(name) = file.file_name() else { return Vec::new() };
    let known = Command::new("git").args(["ls-files", "--error-unmatch"]).arg(name).current_dir(dir).creation_flags(CREATE_NO_WINDOW).output();
    match known {
        Ok(o) if o.status.success() => {}
        Ok(o) if String::from_utf8_lossy(&o.stderr).contains("did not match") => {
            // Not committed yet: all of it is new.
            let n = std::fs::read_to_string(file).map(|t| t.lines().count()).unwrap_or(0);
            return (0..n).map(|i| (i, Change::Added)).collect();
        }
        _ => return Vec::new(),
    }
    match Command::new("git").args(["diff", "-U0", "--no-color", "--no-ext-diff", "HEAD", "--"]).arg(name).current_dir(dir).creation_flags(CREATE_NO_WINDOW).output() {
        Ok(o) if o.status.success() => parse(&String::from_utf8_lossy(&o.stdout)),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_diff_read_into_the_lines_it_changed() {
        let d = "diff --git a/x.rs b/x.rs\n@@ -3 +3 @@ fn a\n-old\n+new\n@@ -10,0 +11,2 @@\n+a\n+b\n@@ -20,3 +22,0 @@\n-x\n-y\n-z\n@@ -30,1 +30,3 @@\n-p\n+q\n+r\n+s\n";
        assert_eq!(
            parse(d),
            vec![(2, Change::Modified), (10, Change::Added), (11, Change::Added), (21, Change::Removed), (29, Change::Modified), (30, Change::Added), (31, Change::Added)]
        );
    }

    #[test]
    fn tests_found_with_their_attribute() {
        let src = "fn helper() {}\n\n#[test]\nfn one() {}\n\n#[test]\n#[ignore = \"slow\"]\n// why\npub fn two_slow() {}\n#[derive(Debug)]\nstruct S;\n";
        assert_eq!(phai_editor::view::tests_in(src), vec![(3, "one".to_string()), (8, "two_slow".to_string())]);
    }
}
