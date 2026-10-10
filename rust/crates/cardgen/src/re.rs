//! Python's `re` functions over `fancy_regex` (which has the lookarounds the Python patterns use, and the same
//! leftmost-first, backtracking preferences). Patterns are compiled once and kept.

use fancy_regex::Regex;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

static CACHE: LazyLock<Mutex<HashMap<String, &'static Regex>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

/// the compiled pattern (compiled on first use, then kept for the life of the program)
pub fn rx(pat: &str) -> &'static Regex {
    let mut c = CACHE.lock().unwrap();
    if let Some(r) = c.get(pat) {
        return r;
    }
    let r: &'static Regex = Box::leak(Box::new(Regex::new(pat).unwrap_or_else(|e| panic!("bad pattern {pat:?}: {e}"))));
    c.insert(pat.to_string(), r);
    r
}

/// A match's groups, owned (Python's `re.Match`).
#[derive(Debug, Clone)]
pub struct M {
    groups: Vec<Option<(usize, usize, String)>>,
}

impl M {
    fn from(c: &fancy_regex::Captures) -> M {
        M { groups: (0..c.len()).map(|i| c.get(i).map(|m| (m.start(), m.end(), m.as_str().to_string()))).collect() }
    }

    /// group i, or None when it didn't take part (Python's `m.group(i)`)
    pub fn g(&self, i: usize) -> Option<&str> {
        self.groups.get(i).and_then(|x| x.as_ref()).map(|x| x.2.as_str())
    }

    /// group i, or "" (Python's `m.group(i) or ''`)
    pub fn gs(&self, i: usize) -> &str {
        self.g(i).unwrap_or("")
    }

    /// the whole match
    pub fn all(&self) -> &str {
        self.gs(0)
    }

    pub fn start(&self) -> usize {
        self.groups[0].as_ref().unwrap().0
    }

    pub fn end(&self) -> usize {
        self.groups[0].as_ref().unwrap().1
    }

    /// the groups after the whole match (Python's `m.groups()`)
    pub fn groups(&self) -> Vec<Option<&str>> {
        (1..self.groups.len()).map(|i| self.g(i)).collect()
    }
}

fn captures(r: &Regex, s: &str, from: usize) -> Option<M> {
    r.captures_from_pos(s, from).unwrap_or_else(|e| panic!("regex {}: {e}", r.as_str())).map(|c| M::from(&c))
}

/// match an already compiled pattern at the start of s's text (the pattern carries its own anchor)
pub fn captures_re(r: &Regex, s: &str) -> Option<M> {
    captures(r, s, 0)
}

/// `re.search(pat, s)`
pub fn search(pat: &str, s: &str) -> Option<M> {
    captures(rx(pat), s, 0)
}

/// `re.match(pat, s)`: anchored at the start
pub fn rmatch(pat: &str, s: &str) -> Option<M> {
    captures(rx(&format!(r"\A(?:{pat})")), s, 0)
}

/// `re.fullmatch(pat, s)`
pub fn fullmatch(pat: &str, s: &str) -> Option<M> {
    captures(rx(&format!(r"\A(?:{pat})\z")), s, 0)
}

/// does `pat` occur in s? (`bool(re.search(pat, s))`)
pub fn is(pat: &str, s: &str) -> bool {
    rx(pat).is_match(s).unwrap_or_else(|e| panic!("regex {pat}: {e}"))
}

/// does s start with `pat`? (`bool(re.match(pat, s))`)
pub fn starts(pat: &str, s: &str) -> bool {
    rmatch(pat, s).is_some()
}

/// every match, left to right, not overlapping (`re.finditer`)
pub fn finditer(pat: &str, s: &str) -> Vec<M> {
    let r = rx(pat);
    let mut out = Vec::new();
    let mut pos = 0;
    while pos <= s.len() {
        let Some(m) = captures(r, s, pos) else { break };
        let (a, b) = (m.start(), m.end());
        out.push(m);
        pos = if b == a { next_char(s, b) } else { b };
    }
    out
}

fn next_char(s: &str, i: usize) -> usize {
    s[i..].chars().next().map(|c| i + c.len_utf8()).unwrap_or(s.len() + 1)
}

/// `re.findall(pat, s)` for a pattern with at most one group: the group (or the whole match)
pub fn findall(pat: &str, s: &str) -> Vec<String> {
    finditer(pat, s)
        .into_iter()
        .map(|m| if m.groups.len() > 1 { m.gs(1).to_string() } else { m.all().to_string() })
        .collect()
}

/// `re.sub(pat, repl, s)`; `repl` may name groups as `\1`
pub fn sub(pat: &str, repl: &str, s: &str) -> String {
    sub_fn(pat, s, |m| expand(repl, m))
}

/// `re.sub(pat, fn, s)`
pub fn sub_fn(pat: &str, s: &str, f: impl Fn(&M) -> String) -> String {
    let mut out = String::new();
    let mut last = 0;
    for m in finditer(pat, s) {
        out.push_str(&s[last..m.start()]);
        out.push_str(&f(&m));
        last = m.end();
    }
    out.push_str(&s[last..]);
    out
}

fn expand(repl: &str, m: &M) -> String {
    let mut out = String::new();
    let mut it = repl.chars().peekable();
    while let Some(c) = it.next() {
        if c == '\\' {
            if let Some(d) = it.peek().copied().filter(|d| d.is_ascii_digit()) {
                it.next();
                out.push_str(m.gs(d.to_digit(10).unwrap() as usize));
                continue;
            }
        }
        out.push(c);
    }
    out
}

/// `re.split(pat, s)` for a pattern without groups
pub fn split(pat: &str, s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut last = 0;
    for m in finditer(pat, s) {
        out.push(s[last..m.start()].to_string());
        last = m.end();
    }
    out.push(s[last..].to_string());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn python_semantics() {
        assert_eq!(rmatch(r"a|b", "ba").unwrap().all(), "b");
        assert!(rmatch(r"a", "ba").is_none());
        assert!(fullmatch(r"(\{[^}]+\})+", "{2}{U}").is_some());
        assert!(fullmatch(r"(\{[^}]+\})+", "{2}{U} x").is_none());
        assert_eq!(sub(r"exile (target)", r"blink \1", "exile target x"), "blink target x");
        assert_eq!(split(r"(?:,? then |\. |; )", "a, then b. c"), ["a", "b", "c"]);
        assert_eq!(findall(r"\{([WUBRG])/P\}", "{1}{B/P}{U/P}"), ["B", "U"]);
        assert_eq!(search(r"x(?! y)", "x y xz").unwrap().start(), 4);
    }
}
