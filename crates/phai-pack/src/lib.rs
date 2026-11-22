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
    /// Installed for everyone on the computer - in Program Files, asking for an administrator.
    pub all_users: bool,
    /// Every file installed, relative - for the uninstaller to take away.
    pub files: Vec<String>,
    /// The kinds of file the program opens: shown with their own icon in Explorer, opened in it by
    /// a double-click, taken away again by the uninstaller.
    pub file_types: Vec<FileType>,
}

/// A kind of file a program opens - `.phf`, Phai Studio's forms.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileType {
    /// Without its dot: `phf`.
    pub ext: String,
    /// Windows' name for the kind (a ProgID): `PhaiStudio.Form`. Extensions of one kind share it.
    pub prog_id: String,
    /// What Explorer calls it: `Phai form`.
    pub description: String,
    /// Its icon, an .ico among the files installed, relative: `phf.ico`.
    pub icon: String,
}

impl Manifest {
    fn to_text(&self) -> String {
        let mut s = format!("name={}\nversion={}\npublisher={}\nexe={}\ndesktop={}\nall_users={}\n", self.name, self.version, self.publisher, self.exe, self.desktop as u8, self.all_users as u8);
        for f in &self.files {
            s.push_str(&format!("file={f}\n"));
        }
        for t in &self.file_types {
            s.push_str(&format!("file_type={}|{}|{}|{}\n", t.ext, t.prog_id, t.description, t.icon));
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
                "all_users" => m.all_users = v == "1",
                "file" => m.files.push(v.into()),
                "file_type" => {
                    let mut p = v.splitn(4, '|');
                    let mut next = || p.next().unwrap_or("").to_string();
                    m.file_types.push(FileType { ext: next(), prog_id: next(), description: next(), icon: next() });
                }
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
    let stub = unsigned(strip(stub));
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

/// A signed program's certificate table: where it starts (at the end of the file), and where the
/// PE header records it.
fn certificate_table(exe: &[u8]) -> Option<(usize, usize)> {
    let word = |at: usize| exe.get(at..at + 4).and_then(|b| b.try_into().ok()).map(u32::from_le_bytes);
    let pe = word(0x3c)? as usize;
    if exe.get(pe..pe + 4)? != b"PE\0\0" {
        return None;
    }
    let optional = pe + 24;
    let magic = u16::from_le_bytes(exe.get(optional..optional + 2)?.try_into().ok()?);
    // The data directories, PE32+ or PE32; the fifth (index 4) is the certificate table.
    let security = optional + if magic == 0x20b { 112 } else { 96 } + 4 * 8;
    let (at, size) = (word(security)? as usize, word(security + 4)? as usize);
    (at > 0 && size > 0 && at + size == exe.len()).then_some((at, security))
}

/// A program with its signature taken off, to have more packed on: what follows a signature
/// would leave it broken, and the setup made from it is signed whole afterwards.
fn unsigned(exe: &[u8]) -> Vec<u8> {
    match certificate_table(exe) {
        Some((at, security)) => {
            let mut out = exe[..at].to_vec();
            out[security..security + 8].fill(0);
            out
        }
        None => exe.to_vec(),
    }
}

/// Where what is packed ends: the end of the file - or, once the setup program is signed, where
/// Windows' certificate table begins. Signing puts that table after everything else (padded to
/// eight bytes), and the PE header says where it starts.
fn package_end(exe: &[u8]) -> usize {
    let n = exe.len();
    let mut end = certificate_table(exe).map(|(at, _)| at).unwrap_or(n);
    // The zeros signing adds to reach an eight-byte boundary.
    for _ in 0..7 {
        if end < 16 || &exe[end - 16..end - 8] == MAGIC || exe[end - 1] != 0 {
            break;
        }
        end -= 1;
    }
    end
}

fn offset_of(exe: &[u8]) -> Option<usize> {
    let n = package_end(exe);
    if n < 16 || &exe[n - 16..n - 8] != MAGIC {
        return None;
    }
    let o = u64::from_le_bytes(exe[n - 8..n].try_into().ok()?) as usize;
    (o < n - 16).then_some(o)
}

/// What a setup program carries: its manifest, and its files when `with_files`.
pub fn unpack(exe: &[u8], with_files: bool) -> Result<(Manifest, Files), String> {
    let o = offset_of(exe).ok_or("nothing is packed in this program")?;
    let end = package_end(exe) - 16;
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
        let order = FileType { ext: "shop".into(), prog_id: "Shop.Order".into(), description: "ใบสั่งซื้อ Shop order".into(), icon: "order.ico".into() };
        let m = Manifest { name: "ร้านค้า Shop".into(), version: "1.2.0".into(), publisher: "Monrak Net".into(), exe: "shop.exe".into(), desktop: true, all_users: true, files: vec![], file_types: vec![order.clone()] };
        let big: Vec<u8> = (0..200_000u32).map(|i| (i % 251) as u8).collect();
        let files = vec![("shop.exe".to_string(), big.clone()), ("data/readme.txt".to_string(), "ไผ่".as_bytes().to_vec())];
        let setup = pack(&stub, &m, &files);
        assert!(setup.len() < stub.len() + big.len(), "compressed");
        let (back, got) = unpack(&setup, true).unwrap();
        assert_eq!(back.name, "ร้านค้า Shop");
        assert!(back.all_users);
        assert_eq!(back.files, ["shop.exe", "data/readme.txt"]);
        assert_eq!(back.file_types, vec![order]);
        assert_eq!(got, files);
        assert_eq!(strip(&setup), &stub[..]);
        // Packed again onto a packed setup: the old load goes, not doubled.
        assert_eq!(strip(&pack(&setup, &m, &files)), &stub[..]);
        assert!(unpack(&stub, false).is_err());
        assert_eq!(Manifest { name: "a:b?".into(), ..Default::default() }.safe_name(), "ab");
    }

    /// A setup program signed after packing, as code signing does: a certificate table after
    /// everything (zeros first, to an eight-byte boundary), its place in the PE header.
    fn signed(setup: &[u8]) -> Vec<u8> {
        let mut out = setup.to_vec();
        while !out.len().is_multiple_of(8) {
            out.push(0);
        }
        let at = out.len() as u32;
        let certificate = vec![0x30u8; 1200];
        out.extend_from_slice(&certificate);
        // PE32+: the certificate table is data directory 4, at optional header + 112 + 32.
        let security = 0x40 + 24 + 112 + 32;
        out[security..security + 4].copy_from_slice(&at.to_le_bytes());
        out[security + 4..security + 8].copy_from_slice(&(certificate.len() as u32).to_le_bytes());
        out
    }

    #[test]
    fn a_signed_setup_still_finds_its_files() {
        // A stub with a DOS header pointing at a PE header with a PE32+ optional header.
        let mut stub = vec![0u8; 0x200];
        stub[0..2].copy_from_slice(b"MZ");
        stub[0x3c..0x40].copy_from_slice(&0x40u32.to_le_bytes());
        stub[0x40..0x44].copy_from_slice(b"PE\0\0");
        stub[0x40 + 24..0x40 + 26].copy_from_slice(&0x20bu16.to_le_bytes());
        let m = Manifest { name: "Shop".into(), version: "1.0.0".into(), exe: "shop.exe".into(), ..Default::default() };
        let files = vec![("shop.exe".to_string(), vec![7u8; 5000])];
        // Lengths of every kind: the zeros signing adds are not taken for the package.
        for name in ["Shop", "Shop!", "Shop!!", "Shop!!!"] {
            let setup = pack(&stub, &Manifest { name: name.into(), ..m.clone() }, &files);
            let signed = signed(&setup);
            let (back, got) = unpack(&signed, true).expect("the package is found past the signature");
            assert_eq!(got, files);
            assert_eq!(back.name, name);
            // The program alone: as long as it was (signing changed its header, not its size).
            assert_eq!(strip(&signed).len(), stub.len());
        }
        // A signed program packed onto: its old signature taken off first, so none is left broken.
        let signed_stub = signed(&stub);
        let setup = pack(&signed_stub, &m, &files);
        assert_eq!(&setup[..stub.len()], &stub[..], "the program as it was before signing");
        assert_eq!(unpack(&setup, true).expect("found").1, files);
    }
}
