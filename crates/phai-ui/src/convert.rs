//! What C# and VB hands reach for every day, kept as short: `to_int("12")` as `int.Parse` (but
//! 0 rather than an exception for words that are no number, as VB's `Val`), `to_float`, and a
//! time of day to show - `now().format("dd/MM/yyyy HH:mm")`, as `DateTime.Now.ToString(...)`,
//! with `bbbb` for the Buddhist year. No Windows here: the web build shares this file.

/// A whole number from what was typed: `" 1,250 "` is 1250, `12.7` is 12, and 0 when there is no
/// number - never an error. As VB's `Val`, or C#'s `int.Parse` that does not fail.
///
/// Example: `let qty = to_int(&ui.qty_box.text());`
pub fn to_int(s: &str) -> i64 {
    let f = to_float(s);
    if f.is_finite() { f.trunc().clamp(i64::MIN as f64, i64::MAX as f64) as i64 } else { 0 }
}

/// A number with a fraction from what was typed: `"1,250.50"` is 1250.5, and 0 when there is no
/// number - never an error.
///
/// Example: `let price = to_float(&ui.price_box.text());`
pub fn to_float(s: &str) -> f64 {
    let clean: String = s.trim().chars().filter(|c| *c != ',' && *c != ' ' && *c != '_').collect();
    // As much of it as reads as a number: `12 kg` is 12.
    let mut end = 0;
    for (i, c) in clean.char_indices() {
        let ok = c.is_ascii_digit() || c == '.' || ((c == '-' || c == '+') && i == 0);
        if !ok {
            break;
        }
        end = i + c.len_utf8();
    }
    clean[..end].parse().unwrap_or(0.0)
}

/// A moment of the local day - C#'s DateTime, enough to show and compare.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct DateTime {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
    pub millisecond: u32,
}

impl DateTime {
    /// The date and time written as you choose, as C# writes a format: `yyyy` `yy` the year,
    /// `bbbb` the Buddhist year (พ.ศ.), `MM` `M` the month, `dd` `d` the day, `HH` `H` the hour of
    /// 24, `hh` `h` of 12, `mm` `m` the minute, `ss` `s` the second, `fff` milliseconds, `tt` AM or
    /// PM; anything else as it is.
    ///
    /// Example: `now().format("dd/MM/bbbb HH:mm")` gives 23/11/2569 21:30
    pub fn format(&self, pattern: &str) -> String {
        const TOKENS: [&str; 16] = ["yyyy", "bbbb", "fff", "yy", "MM", "dd", "HH", "hh", "mm", "ss", "tt", "M", "d", "H", "h", "m"];
        let h12 = if self.hour.is_multiple_of(12) { 12 } else { self.hour % 12 };
        let mut out = String::new();
        let mut rest = pattern;
        'outer: while !rest.is_empty() {
            for t in TOKENS.iter().chain(["s"].iter()) {
                if let Some(after) = rest.strip_prefix(t) {
                    out.push_str(&match *t {
                        "yyyy" => format!("{:04}", self.year),
                        "bbbb" => format!("{:04}", self.year + 543),
                        "yy" => format!("{:02}", self.year.rem_euclid(100)),
                        "MM" => format!("{:02}", self.month),
                        "M" => self.month.to_string(),
                        "dd" => format!("{:02}", self.day),
                        "d" => self.day.to_string(),
                        "HH" => format!("{:02}", self.hour),
                        "H" => self.hour.to_string(),
                        "hh" => format!("{h12:02}"),
                        "h" => h12.to_string(),
                        "mm" => format!("{:02}", self.minute),
                        "m" => self.minute.to_string(),
                        "ss" => format!("{:02}", self.second),
                        "s" => self.second.to_string(),
                        "fff" => format!("{:03}", self.millisecond),
                        "tt" => (if self.hour < 12 { "AM" } else { "PM" }).to_string(),
                        _ => String::new(),
                    });
                    rest = after;
                    continue 'outer;
                }
            }
            let c = rest.chars().next().unwrap_or(' ');
            out.push(c);
            rest = &rest[c.len_utf8()..];
        }
        out
    }

    /// The day as `dd/MM/yyyy`, such as 23/11/2026.
    ///
    /// Example: `ui.date_label.set_text(&now().date_text());`
    pub fn date_text(&self) -> String {
        self.format("dd/MM/yyyy")
    }

    /// The time as `HH:mm:ss`, such as 21:30:05.
    ///
    /// Example: `ui.time_label.set_text(&now().time_text());`
    pub fn time_text(&self) -> String {
        self.format("HH:mm:ss")
    }
}

impl std::fmt::Display for DateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.format("dd/MM/yyyy HH:mm:ss"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_from_words_as_vb_reads_them() {
        assert_eq!(to_int("12"), 12);
        assert_eq!(to_int(" 1,250 "), 1250);
        assert_eq!(to_int("12.7"), 12);
        assert_eq!(to_int("-3"), -3);
        assert_eq!(to_int("12 kg"), 12);
        assert_eq!(to_int("abc"), 0);
        assert_eq!(to_int(""), 0);
        assert_eq!(to_float("1,250.50"), 1250.5);
        assert_eq!(to_float("x"), 0.0);
    }

    #[test]
    fn a_time_is_shown_as_csharp_formats_it() {
        let t = DateTime { year: 2026, month: 11, day: 3, hour: 19, minute: 5, second: 9, millisecond: 42 };
        assert_eq!(t.format("dd/MM/yyyy HH:mm:ss"), "03/11/2026 19:05:09");
        assert_eq!(t.format("d/M/bbbb"), "3/11/2569", "the Buddhist year");
        assert_eq!(t.format("hh:mm tt"), "07:05 PM");
        assert_eq!(t.format("yy-MM-dd fff"), "26-11-03 042");
        assert_eq!(t.format("วันที่ d"), "วันที่ 3");
        assert_eq!(t.to_string(), "03/11/2026 19:05:09");
        assert_eq!((t.date_text(), t.time_text()), ("03/11/2026".to_string(), "19:05:09".to_string()));
    }
}
