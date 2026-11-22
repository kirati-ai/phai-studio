//! Made by Phai Studio. Embeds a manifest that names Common Controls 6 - without it Windows hands
//! the program the old controls: buttons as in Windows 2000, toolbar icons in black squares - and
//! the program's icon, the .ico Cargo.toml names (`[package.metadata.phai] icon`), as Explorer,
//! the taskbar and shortcuts show it. Project > Properties sets the icon.

fn main() {
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg=/MANIFESTDEPENDENCY:type='win32' name='Microsoft.Windows.Common-Controls' version='6.0.0.0' processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'"
        );
        if let Some(icon) = icon_of_package() {
            println!("cargo:rerun-if-changed={icon}");
            match std::fs::read(&icon).map_err(|e| e.to_string()).and_then(|ico| res_of_ico(&ico)) {
                Ok(res) => {
                    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap_or_default()).join("icon.res");
                    if std::fs::write(&out, res).is_ok() {
                        // The linker takes a compiled resource file as it is.
                        println!("cargo:rustc-link-arg={}", out.display());
                    }
                }
                Err(e) => println!("cargo:warning=the program's icon {icon}: {e}"),
            }
        }
    }
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");
}

/// `icon = "..."` under `[package.metadata.phai]` in Cargo.toml.
fn icon_of_package() -> Option<String> {
    let toml = std::fs::read_to_string("Cargo.toml").ok()?;
    let mut inside = false;
    for line in toml.lines().map(str::trim) {
        if line.starts_with('[') {
            inside = line == "[package.metadata.phai]";
        } else if inside {
            if let Some(v) = line.strip_prefix("icon").map(str::trim_start).and_then(|r| r.strip_prefix('=')) {
                let v = v.trim().trim_matches('"').trim();
                return (!v.is_empty()).then(|| v.to_string());
            }
        }
    }
    None
}

/// An .ico as a compiled resource file: each picture an RT_ICON, the set an RT_GROUP_ICON named 1.
fn res_of_ico(ico: &[u8]) -> Result<Vec<u8>, String> {
    let u16_at = |i: usize| ico.get(i..i + 2).map(|b| u16::from_le_bytes([b[0], b[1]])).ok_or("too short");
    let u32_at = |i: usize| ico.get(i..i + 4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]])).ok_or("too short");
    if u16_at(0)? != 0 || u16_at(2)? != 1 {
        return Err("not an .ico file".into());
    }
    let count = u16_at(4)? as usize;
    let mut res = Vec::new();
    // A resource: its header (type and name as numbers), then its data, each to 4 bytes.
    fn put(res: &mut Vec<u8>, kind: u16, name: u16, flags: u16, data: &[u8]) {
        for v in [data.len() as u32, 32] {
            res.extend_from_slice(&v.to_le_bytes());
        }
        for v in [0xFFFF, kind, 0xFFFF, name] {
            res.extend_from_slice(&(v as u16).to_le_bytes());
        }
        res.extend_from_slice(&0u32.to_le_bytes()); // data version
        res.extend_from_slice(&flags.to_le_bytes());
        res.extend_from_slice(&0x0409u16.to_le_bytes()); // language
        res.extend_from_slice(&0u32.to_le_bytes()); // version
        res.extend_from_slice(&0u32.to_le_bytes()); // characteristics
        res.extend_from_slice(data);
        while res.len() % 4 != 0 {
            res.push(0);
        }
    }
    // A resource file starts with an empty one.
    res.extend_from_slice(&[0, 0, 0, 0, 32, 0, 0, 0, 0xFF, 0xFF, 0, 0, 0xFF, 0xFF, 0, 0]);
    res.extend_from_slice(&[0u8; 16]);
    let mut group = Vec::new();
    group.extend_from_slice(&[0, 0, 1, 0]);
    group.extend_from_slice(&(count as u16).to_le_bytes());
    for k in 0..count {
        let e = 6 + k * 16;
        let (size, offset) = (u32_at(e + 8)? as usize, u32_at(e + 12)? as usize);
        let image = ico.get(offset..offset + size).ok_or("a picture past the file's end")?;
        put(&mut res, 3, (k + 1) as u16, 0x1010, image);
        group.extend_from_slice(&ico[e..e + 8]); // width, height, colours, reserved, planes, bits
        group.extend_from_slice(&(size as u32).to_le_bytes());
        group.extend_from_slice(&((k + 1) as u16).to_le_bytes());
    }
    put(&mut res, 14, 1, 0x1030, &group);
    Ok(res)
}
