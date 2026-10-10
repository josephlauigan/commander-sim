//! Python string methods the ported code leans on.

/// `s.isdigit()` (ASCII digits: card text has no others)
pub fn isdigit(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
}

/// `s.lstrip(chars)`
pub fn lstrip<'a>(s: &'a str, chars: &str) -> &'a str {
    s.trim_start_matches(|c| chars.contains(c))
}

/// `s.rstrip(chars)`
pub fn rstrip<'a>(s: &'a str, chars: &str) -> &'a str {
    s.trim_end_matches(|c| chars.contains(c))
}

/// `s.strip(chars)`
pub fn strip<'a>(s: &'a str, chars: &str) -> &'a str {
    s.trim_matches(|c| chars.contains(c))
}

/// `len(s)`: code points
pub fn len(s: &str) -> usize {
    s.chars().count()
}

/// `s.split(sep)[i]` (None when there are fewer parts)
pub fn part<'a>(s: &'a str, sep: &str, i: usize) -> Option<&'a str> {
    s.split(sep).nth(i)
}

/// `s.split(sep)[0]`
pub fn first<'a>(s: &'a str, sep: &str) -> &'a str {
    s.split(sep).next().unwrap_or("")
}

/// `' '.join(s.split())`-style word split (Python's `s.split()`)
pub fn words(s: &str) -> Vec<&str> {
    s.split_whitespace().collect()
}

/// `int(s)` for a tag value
pub fn int(s: &str) -> Result<i64, String> {
    s.trim().parse::<i64>().map_err(|_| format!("invalid literal for int(): {s:?}"))
}
