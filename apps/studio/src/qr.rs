//! A QR code of a short text - the address Run on Phone shows, for a phone's camera to open.
//! Byte mode, error correction level M, versions 1 to 6 (up to 106 bytes): all an address
//! needs, written here rather than taken from a crate.

/// Total codewords, error-correction codewords per block, blocks - versions 1 to 6, level M.
const VERSIONS: [(usize, usize, usize); 6] = [(26, 10, 1), (44, 16, 1), (70, 26, 1), (100, 18, 2), (134, 24, 2), (172, 16, 4)];

/// The modules, `true` dark; `size` by `size`, read `dark[y * size + x]`.
pub struct Code {
    pub size: usize,
    pub dark: Vec<bool>,
}

impl Code {
    pub fn at(&self, x: usize, y: usize) -> bool {
        self.dark[y * self.size + x]
    }
}

/// The code of `text`; `None` when it is longer than version 6 holds.
pub fn encode(text: &str) -> Option<Code> {
    let bytes = text.as_bytes();
    let (version, &(total, ec, blocks)) = VERSIONS.iter().enumerate().find(|(_, (total, ec, blocks))| total - ec * blocks >= bytes.len() + 2)?;
    let version = version + 1;
    let data_len = total - ec * blocks;
    let data = data_codewords(bytes, data_len);
    let all = with_correction(&data, total, ec, blocks);
    let mut m = Matrix::new(version);
    m.draw_function_patterns();
    m.draw_codewords(&all);
    // The mask with the lowest penalty, as the standard has it.
    let mut best: Option<(u32, Matrix)> = None;
    for mask in 0..8 {
        let mut t = m.clone();
        t.apply_mask(mask);
        t.draw_format(mask);
        let p = t.penalty();
        if best.as_ref().is_none_or(|(bp, _)| p < *bp) {
            best = Some((p, t));
        }
    }
    let (_, m) = best?;
    Some(Code { size: m.size, dark: m.dark })
}

/// Mode, count, the bytes, the end, then the pad bytes - `len` codewords.
fn data_codewords(bytes: &[u8], len: usize) -> Vec<u8> {
    let mut bits: Vec<bool> = Vec::new();
    let put = |v: u32, n: u32, bits: &mut Vec<bool>| (0..n).rev().for_each(|i| bits.push((v >> i) & 1 == 1));
    put(0b0100, 4, &mut bits);
    put(bytes.len() as u32, 8, &mut bits);
    for &b in bytes {
        put(b as u32, 8, &mut bits);
    }
    let room = len * 8;
    let end = (room - bits.len()).min(4);
    put(0, end as u32, &mut bits);
    while !bits.len().is_multiple_of(8) {
        bits.push(false);
    }
    let mut out: Vec<u8> = bits.chunks(8).map(|c| c.iter().fold(0u8, |a, &b| (a << 1) | b as u8)).collect();
    let mut pad = [0xEC, 0x11].into_iter().cycle();
    while out.len() < len {
        out.push(pad.next().unwrap_or(0xEC));
    }
    out
}

/// The data split into blocks, each with its error correction, interleaved as the code holds them.
fn with_correction(data: &[u8], total: usize, ec: usize, blocks: usize) -> Vec<u8> {
    let short = total / blocks - ec;
    let longer = total % blocks;
    let divisor = rs_divisor(ec);
    let mut split: Vec<(&[u8], Vec<u8>)> = Vec::new();
    let mut at = 0;
    for i in 0..blocks {
        let n = short + usize::from(i >= blocks - longer);
        let d = &data[at..at + n];
        at += n;
        split.push((d, rs_remainder(d, &divisor)));
    }
    let mut out = Vec::with_capacity(total);
    for i in 0..=short {
        for (d, _) in &split {
            if let Some(&b) = d.get(i) {
                out.push(b);
            }
        }
    }
    for i in 0..ec {
        for (_, e) in &split {
            out.push(e[i]);
        }
    }
    out
}

fn gf_mul(x: u8, y: u8) -> u8 {
    let mut z: u16 = 0;
    for i in (0..8).rev() {
        z = (z << 1) ^ ((z >> 7) * 0x11D);
        z ^= ((y as u16 >> i) & 1) * x as u16;
    }
    z as u8
}

fn rs_divisor(degree: usize) -> Vec<u8> {
    let mut result = vec![0u8; degree];
    result[degree - 1] = 1;
    let mut root: u8 = 1;
    for _ in 0..degree {
        for j in 0..degree {
            result[j] = gf_mul(result[j], root);
            if j + 1 < degree {
                result[j] ^= result[j + 1];
            }
        }
        root = gf_mul(root, 0x02);
    }
    result
}

fn rs_remainder(data: &[u8], divisor: &[u8]) -> Vec<u8> {
    let mut result = vec![0u8; divisor.len()];
    for &b in data {
        let factor = b ^ result[0];
        result.remove(0);
        result.push(0);
        for (r, &c) in result.iter_mut().zip(divisor) {
            *r ^= gf_mul(c, factor);
        }
    }
    result
}

/// The format bits of level M and `mask`: the 15 bits the code shows twice.
fn format_bits(mask: u32) -> u32 {
    let data = mask; // level M is 00
    let mut rem = data;
    for _ in 0..10 {
        rem = (rem << 1) ^ ((rem >> 9) * 0x537);
    }
    ((data << 10) | rem) ^ 0x5412
}

#[derive(Clone)]
struct Matrix {
    version: usize,
    size: usize,
    dark: Vec<bool>,
    function: Vec<bool>,
}

impl Matrix {
    fn new(version: usize) -> Matrix {
        let size = 17 + 4 * version;
        Matrix { version, size, dark: vec![false; size * size], function: vec![false; size * size] }
    }

    fn set_function(&mut self, x: usize, y: usize, dark: bool) {
        let i = y * self.size + x;
        self.dark[i] = dark;
        self.function[i] = true;
    }

    fn draw_function_patterns(&mut self) {
        let size = self.size;
        for i in 0..size {
            self.set_function(6, i, i % 2 == 0);
            self.set_function(i, 6, i % 2 == 0);
        }
        self.draw_finder(3, 3);
        self.draw_finder(size as i32 - 4, 3);
        self.draw_finder(3, size as i32 - 4);
        if self.version >= 2 {
            let c = size - 7;
            self.draw_alignment(c, c);
        }
        // The format areas, kept for later; their dark module.
        self.draw_format(0);
    }

    fn draw_finder(&mut self, cx: i32, cy: i32) {
        for dy in -4i32..=4 {
            for dx in -4i32..=4 {
                let (x, y) = (cx + dx, cy + dy);
                if x < 0 || y < 0 || x >= self.size as i32 || y >= self.size as i32 {
                    continue;
                }
                let d = dx.abs().max(dy.abs());
                self.set_function(x as usize, y as usize, d != 2 && d != 4);
            }
        }
    }

    fn draw_alignment(&mut self, cx: usize, cy: usize) {
        for dy in -2i32..=2 {
            for dx in -2i32..=2 {
                let d = dx.abs().max(dy.abs());
                self.set_function((cx as i32 + dx) as usize, (cy as i32 + dy) as usize, d != 1);
            }
        }
    }

    fn draw_format(&mut self, mask: u32) {
        let bits = format_bits(mask);
        let bit = |i: u32| (bits >> i) & 1 == 1;
        let size = self.size;
        for i in 0..=5 {
            self.set_function(8, i, bit(i as u32));
        }
        self.set_function(8, 7, bit(6));
        self.set_function(8, 8, bit(7));
        self.set_function(7, 8, bit(8));
        for i in 9..15 {
            self.set_function(14 - i, 8, bit(i as u32));
        }
        for i in 0..8 {
            self.set_function(size - 1 - i, 8, bit(i as u32));
        }
        for i in 8..15 {
            self.set_function(8, size - 15 + i, bit(i as u32));
        }
        self.set_function(8, size - 8, true);
    }

    /// The codewords in the zigzag from the bottom right, two columns at a time.
    fn draw_codewords(&mut self, data: &[u8]) {
        let size = self.size;
        let mut i = 0usize;
        let mut right = size as i32 - 1;
        while right >= 1 {
            if right == 6 {
                right = 5;
            }
            for vert in 0..size {
                for j in 0..2 {
                    let x = (right - j) as usize;
                    let upward = (right + 1) & 2 == 0;
                    let y = if upward { size - 1 - vert } else { vert };
                    if !self.function[y * size + x] && i < data.len() * 8 {
                        self.dark[y * size + x] = (data[i >> 3] >> (7 - (i & 7))) & 1 == 1;
                        i += 1;
                    }
                }
            }
            right -= 2;
        }
    }

    fn apply_mask(&mut self, mask: u32) {
        let size = self.size;
        for y in 0..size {
            for x in 0..size {
                let flip = match mask {
                    0 => (x + y) % 2 == 0,
                    1 => y % 2 == 0,
                    2 => x % 3 == 0,
                    3 => (x + y) % 3 == 0,
                    4 => (x / 3 + y / 2) % 2 == 0,
                    5 => x * y % 2 + x * y % 3 == 0,
                    6 => (x * y % 2 + x * y % 3) % 2 == 0,
                    _ => ((x + y) % 2 + x * y % 3) % 2 == 0,
                };
                let i = y * size + x;
                if flip && !self.function[i] {
                    self.dark[i] = !self.dark[i];
                }
            }
        }
    }

    /// The standard's penalty: runs of one colour, 2x2 boxes, finder-like rows, the dark share.
    fn penalty(&self) -> u32 {
        let size = self.size;
        let at = |x: usize, y: usize| self.dark[y * size + x];
        let mut p = 0u32;
        for horizontal in [true, false] {
            for a in 0..size {
                let line: Vec<bool> = (0..size).map(|b| if horizontal { at(b, a) } else { at(a, b) }).collect();
                let mut run = 1;
                for b in 1..size {
                    if line[b] == line[b - 1] {
                        run += 1;
                        if run == 5 {
                            p += 3;
                        } else if run > 5 {
                            p += 1;
                        }
                    } else {
                        run = 1;
                    }
                }
                for w in line.windows(11) {
                    let pat = [true, false, true, true, true, false, true];
                    let core = w[..7] == pat && w[7..].iter().all(|&d| !d);
                    let core2 = w[4..] == pat && w[..4].iter().all(|&d| !d);
                    if core || core2 {
                        p += 40;
                    }
                }
            }
        }
        for y in 0..size - 1 {
            for x in 0..size - 1 {
                let c = at(x, y);
                if c == at(x + 1, y) && c == at(x, y + 1) && c == at(x + 1, y + 1) {
                    p += 3;
                }
            }
        }
        let darks = self.dark.iter().filter(|&&d| d).count();
        let total = size * size;
        let k = ((darks * 20).abs_diff(total * 10)).div_ceil(total).saturating_sub(1);
        p + k as u32 * 10
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reed_solomon_as_the_standard_s_example() {
        // "HELLO WORLD", version 1-M: its data and its error correction, as published.
        let data = [32, 91, 11, 120, 209, 114, 220, 77, 67, 64, 236, 17, 236, 17, 236, 17];
        assert_eq!(rs_remainder(&data, &rs_divisor(10)), vec![196, 35, 39, 119, 235, 215, 231, 226, 93, 23]);
    }

    #[test]
    fn format_bits_as_the_standard_s_table() {
        assert_eq!(format_bits(0), 0b101010000010010);
        assert_eq!(format_bits(1), 0b101000100100101);
        assert_eq!(format_bits(4), 0b100010111111001);
        assert_eq!(format_bits(7), 0b100101010100000);
    }

    #[test]
    fn data_codewords_padded() {
        let d = data_codewords(b"ab", 16);
        assert_eq!(&d[..4], &[0x40, 0x26, 0x16, 0x20]);
        assert_eq!(&d[4..6], &[0xEC, 0x11]);
    }

    #[test]
    fn an_address_fits_and_has_its_finders() {
        let c = encode("http://192.168.100.200:8790/").unwrap();
        assert_eq!(c.size, 29, "version 3");
        for (x, y) in [(0, 0), (c.size - 7, 0), (0, c.size - 7)] {
            assert!(c.at(x, y) && c.at(x + 6, y + 6) && !c.at(x + 1, y + 1) && c.at(x + 3, y + 3));
        }
        assert!(c.at(8, c.size - 8), "the dark module");
        assert!(encode(&"x".repeat(107)).is_none());
        assert_eq!(encode(&"x".repeat(106)).unwrap().size, 41);
    }

    #[test]
    fn blocks_interleaved() {
        // Version 6-M: four blocks of 27.
        let data: Vec<u8> = (0..108).map(|i| i as u8).collect();
        let all = with_correction(&data, 172, 16, 4);
        assert_eq!(&all[..5], &[0, 27, 54, 81, 1]);
        assert_eq!(all.len(), 172);
    }
}
