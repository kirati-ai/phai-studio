//! A setup program carries what it installs on its own end: Phai's setup exe, then the product's
//! description and its files (each compressed), then a trailer that says where they start. The
//! setup reads itself, finds the trailer, and has everything - one file to hand out.
//!
//! ```text
//! [setup exe] [manifest length u32][manifest][file]... [b"PHAIPACK"][offset u64]
//! file = [path length u16][path][size u64][compressed length u64][deflate bytes]
//! ```

const MAGIC: &[u8; 8] = b"PHAIPACK";

/// What is installed: its name, version, publisher, and the program to start.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Manifest {
    pub name: String,
    pub version: String,
    pub publisher: String,
    /// The program, relative to the install folder: `shop.exe`.
    pub exe: String,
    /// Whether the setup offers a desktop shortcut ticked.
    pub desktop: bool,
    /// Every file installed, relative - for the uninstaller to take away.
    pub files: Vec<String>,
}

impl Manifest {
    fn to_text(&self) -> String {
        let mut s = format!("name={}\nversion={}\npublisher={}\nexe={}\ndesktop={}\n", self.name, self.version, self.publisher, self.exe, self.desktop as u8);
        for f in &self.files {
            s.push_str(&format!("file={f}\n"));
        }
        s
    }

    fn from_text(t: &str) -> Manifest {
        let mut m = Manifest::default();
        for line in t.lines() {
            let Some((k, v)) = line.split_once('=') else { continue };
            match k {
                "name" => m.name = v.into(),
                "version" => m.version = v.into(),
                "publisher" => m.publisher = v.into(),
                "exe" => m.exe = v.into(),
                "desktop" => m.desktop = v == "1",
                "file" => m.files.push(v.into()),
                _ => {}
            }
        }
        m
    }

    /// A name safe for folders, shortcuts and the registry: what Windows forbids in a file name
    /// taken out.
    pub fn safe_name(&self) -> String {
        let s: String = self.name.chars().filter(|c| !"<>:\"/\\|?*".contains(*c) && !c.is_control()).collect();
        let s = s.trim().trim_end_matches('.').to_string();
        if s.is_empty() { "Program".into() } else { s }
    }
}

/// Files as packed: a path inside the program's folder, and its bytes.
pub type Files = Vec<(String, Vec<u8>)>;

/// The setup program `stub` with `files` (path, bytes) packed onto it.
pub fn pack(stub: &[u8], manifest: &Manifest, files: &[(String, Vec<u8>)]) -> Vec<u8> {
    let stub = strip(stub);
    let mut m = manifest.clone();
    // No files: the manifest as it is - an uninstaller, which only needs to know what to take away.
    if !files.is_empty() {
        m.files = files.iter().map(|(p, _)| p.clone()).collect();
    }
    let mut out = stub.to_vec();
    let offset = out.len() as u64;
    let text = m.to_text();
    out.extend_from_slice(&(text.len() as u32).to_le_bytes());
    out.extend_from_slice(text.as_bytes());
    for (path, data) in files {
        let packed = miniz_oxide::deflate::compress_to_vec(data, 8);
        out.extend_from_slice(&(path.len() as u16).to_le_bytes());
        out.extend_from_slice(path.as_bytes());
        out.extend_from_slice(&(data.len() as u64).to_le_bytes());
        out.extend_from_slice(&(packed.len() as u64).to_le_bytes());
        out.extend_from_slice(&packed);
    }
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&offset.to_le_bytes());
    out
}

/// A setup program without what is packed on it - the program alone.
pub fn strip(exe: &[u8]) -> &[u8] {
    match offset_of(exe) {
        Some(o) => &exe[..o],
        None => exe,
    }
}

fn offset_of(exe: &[u8]) -> Option<usize> {
    let n = exe.len();
    if n < 16 || &exe[n - 16..n - 8] != MAGIC {
        return None;
    }
    let o = u64::from_le_bytes(exe[n - 8..].try_into().ok()?) as usize;
    (o < n - 16).then_some(o)
}

/// What a setup program carries: its manifest, and its files when `with_files`.
pub fn unpack(exe: &[u8], with_files: bool) -> Result<(Manifest, Files), String> {
    let o = offset_of(exe).ok_or("nothing is packed in this program")?;
    let end = exe.len() - 16;
    let mut at = o;
    let take = |at: &mut usize, n: usize| -> Result<&[u8], String> {
        if *at + n > end {
            return Err("the package is cut short".into());
        }
        let s = &exe[*at..*at + n];
        *at += n;
        Ok(s)
    };
    let mlen = u32::from_le_bytes(take(&mut at, 4)?.try_into().unwrap_or_default()) as usize;
    let manifest = Manifest::from_text(&String::from_utf8_lossy(take(&mut at, mlen)?));
    let mut files = Vec::new();
    if with_files {
        while at < end {
            let plen = u16::from_le_bytes(take(&mut at, 2)?.try_into().unwrap_or_default()) as usize;
            let path = String::from_utf8_lossy(take(&mut at, plen)?).to_string();
            let size = u64::from_le_bytes(take(&mut at, 8)?.try_into().unwrap_or_default()) as usize;
            let clen = u64::from_le_bytes(take(&mut at, 8)?.try_into().unwrap_or_default()) as usize;
            let data = miniz_oxide::inflate::decompress_to_vec_with_limit(take(&mut at, clen)?, size).map_err(|e| format!("{path}: {e:?}"))?;
            // A path that climbs out of the install folder is not one a setup writes.
            if path.contains("..") || path.starts_with(['/', '\\']) || path.contains(':') {
                return Err(format!("{path}: not a path inside the program's folder"));
            }
            files.push((path, data));
        }
    }
    Ok((manifest, files))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packed_files_come_back_whole() {
        let stub = b"MZ pretend setup program".to_vec();
        let m = Manifest { name: "ร้านค้า Shop".into(), version: "1.2.0".into(), publisher: "Monrak Net".into(), exe: "shop.exe".into(), desktop: true, files: vec![] };
        let big: Vec<u8> = (0..200_000u32).map(|i| (i % 251) as u8).collect();
        let files = vec![("shop.exe".to_string(), big.clone()), ("data/readme.txt".to_string(), "ไผ่".as_bytes().to_vec())];
        let setup = pack(&stub, &m, &files);
        assert!(setup.len() < stub.len() + big.len(), "compressed");
        let (back, got) = unpack(&setup, true).unwrap();
        assert_eq!(back.name, "ร้านค้า Shop");
        assert_eq!(back.files, ["shop.exe", "data/readme.txt"]);
        assert_eq!(got, files);
        assert_eq!(strip(&setup), &stub[..]);
        // Packed again onto a packed setup: the old load goes, not doubled.
        assert_eq!(strip(&pack(&setup, &m, &files)), &stub[..]);
        assert!(unpack(&stub, false).is_err());
        assert_eq!(Manifest { name: "a:b?".into(), ..Default::default() }.safe_name(), "ab");
    }
}
