//! Help > Check for Updates: the newest Phai Studio on GitHub Releases, downloaded and checked
//! against the SHA-256 GitHub gives for it, then its setup run. Through Windows' own curl.exe,
//! as the crate search goes.

use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const LATEST: &str = "https://api.github.com/repos/kirati-ai/phai-studio/releases/latest";

/// A release of Phai Studio: its version and its setup program.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Release {
    /// Without the `v`: `0.2.8`.
    pub version: String,
    pub file: String,
    pub url: String,
    /// Lower-case hex; empty when GitHub gave none.
    pub sha256: String,
}

/// The newest release, asked of GitHub.
pub fn newest() -> Result<Release, String> {
    let out = curl(&["-s", "-f", "--max-time", "20", "-H", "Accept: application/vnd.github+json", LATEST])?;
    release_of(&String::from_utf8_lossy(&out)).ok_or_else(|| "GitHub's answer has no setup program".to_string())
}

/// GitHub's description of a release read: its tag and its `-setup.exe`.
pub fn release_of(json: &str) -> Option<Release> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    let version = v["tag_name"].as_str()?.trim_start_matches('v').to_string();
    let asset = v["assets"].as_array()?.iter().find(|a| a["name"].as_str().is_some_and(|n| n.ends_with("-setup.exe")))?;
    let url = asset["browser_download_url"].as_str()?.to_string();
    // Only from the project's own releases.
    if !url.starts_with("https://github.com/kirati-ai/phai-studio/releases/download/") {
        return None;
    }
    Some(Release {
        version,
        file: asset["name"].as_str()?.to_string(),
        url,
        sha256: asset["digest"].as_str().unwrap_or("").trim_start_matches("sha256:").to_ascii_lowercase(),
    })
}

/// Whether version `theirs` comes after `ours`: `0.2.10` after `0.2.9`.
pub fn is_newer(theirs: &str, ours: &str) -> bool {
    let parts = |s: &str| s.split(['.', '-']).map(|p| p.parse::<u64>().unwrap_or(0)).collect::<Vec<_>>();
    let (a, b) = (parts(theirs), parts(ours));
    for i in 0..a.len().max(b.len()) {
        let (x, y) = (a.get(i).copied().unwrap_or(0), b.get(i).copied().unwrap_or(0));
        if x != y {
            return x > y;
        }
    }
    false
}

/// The release's setup downloaded into the temporary folder and checked: its path.
pub fn download(r: &Release) -> Result<PathBuf, String> {
    let to = std::env::temp_dir().join(&r.file);
    let _ = std::fs::remove_file(&to);
    let path = to.to_string_lossy().to_string();
    curl(&["-s", "-f", "-L", "--max-time", "600", "-o", &path, &r.url])?;
    let data = std::fs::read(&to).map_err(|e| e.to_string())?;
    if !r.sha256.is_empty() && sha256_hex(&data) != r.sha256 {
        let _ = std::fs::remove_file(&to);
        return Err(phai_ui::text::t("The download is not the file GitHub describes - it was deleted. Try again later.").to_string());
    }
    Ok(to)
}

fn sha256_hex(data: &[u8]) -> String {
    use sha2::Digest;
    sha2::Sha256::digest(data).iter().map(|b| format!("{b:02x}")).collect()
}

fn curl(args: &[&str]) -> Result<Vec<u8>, String> {
    let out = Command::new("curl.exe")
        .arg("-A")
        .arg(concat!("phai-studio/", env!("CARGO_PKG_VERSION")))
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("curl: {e}"))?;
    if !out.status.success() {
        return Err(phai_ui::text::t("GitHub could not be reached - is the computer online?").to_string());
    }
    Ok(out.stdout)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The real thing: GitHub asked, the newest setup downloaded and checked, then deleted.
    #[test]
    #[ignore]
    fn the_newest_setup_downloads_and_checks() {
        let r = newest().unwrap();
        assert!(!r.sha256.is_empty(), "GitHub gives the setup's SHA-256");
        let p = download(&r).unwrap();
        assert!(std::fs::metadata(&p).unwrap().len() > 1_000_000);
        std::fs::remove_file(p).unwrap();
        let bad = Release { sha256: "00".into(), ..r };
        assert!(download(&bad).is_err(), "a file that is not what GitHub says is refused");
    }

    #[test]
    fn versions_compare_by_number() {
        assert!(is_newer("0.2.10", "0.2.9"));
        assert!(is_newer("0.3.0", "0.2.7"));
        assert!(is_newer("1.0", "0.9.9"));
        assert!(!is_newer("0.2.7", "0.2.7"));
        assert!(!is_newer("0.2.6", "0.2.7"));
    }

    #[test]
    fn github_s_answer_is_read() {
        let json = r#"{"tag_name":"v0.2.8","assets":[{"name":"notes.txt","browser_download_url":"https://github.com/kirati-ai/phai-studio/releases/download/v0.2.8/notes.txt"},
            {"name":"PhaiStudio-0.2.8-setup.exe","digest":"sha256:ABC123","browser_download_url":"https://github.com/kirati-ai/phai-studio/releases/download/v0.2.8/PhaiStudio-0.2.8-setup.exe"}]}"#;
        let r = release_of(json).unwrap();
        assert_eq!(r.version, "0.2.8");
        assert_eq!(r.file, "PhaiStudio-0.2.8-setup.exe");
        assert_eq!(r.sha256, "abc123");
        assert!(release_of(&json.replace("github.com/kirati-ai", "example.com/x")).is_none(), "only the project's own releases");
        assert!(release_of(r#"{"tag_name":"v1","assets":[]}"#).is_none());
        assert_eq!(sha256_hex(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    }
}
