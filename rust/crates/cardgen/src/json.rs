//! An ordered JSON value, read and written the way Python's `json` module does: objects keep their keys in insertion
//! order (Python dicts), and the writer reproduces `json.dump(indent=..., ensure_ascii=...)` byte for byte, so the
//! files this crate writes are the files the Python wrote.

use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum J {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Arr(Vec<J>),
    Obj(Obj),
}

/// A JSON object with its keys in insertion order. `set` replaces a key's value in place (a Python dict assignment).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Obj(pub Vec<(String, J)>);

impl Obj {
    pub fn new() -> Obj {
        Obj(Vec::new())
    }

    pub fn get(&self, k: &str) -> Option<&J> {
        self.0.iter().find(|(x, _)| x == k).map(|(_, v)| v)
    }

    pub fn get_mut(&mut self, k: &str) -> Option<&mut J> {
        self.0.iter_mut().find(|(x, _)| x == k).map(|(_, v)| v)
    }

    pub fn contains(&self, k: &str) -> bool {
        self.0.iter().any(|(x, _)| x == k)
    }

    pub fn set(&mut self, k: &str, v: impl Into<J>) {
        let v = v.into();
        match self.get_mut(k) {
            Some(slot) => *slot = v,
            None => self.0.push((k.to_string(), v)),
        }
    }

    /// Python's `dict.update`: every key of `other`, in its order (existing keys keep their place)
    pub fn update(&mut self, other: &Obj) {
        for (k, v) in &other.0 {
            self.set(k, v.clone());
        }
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(|(k, _)| k.as_str())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &J)> {
        self.0.iter().map(|(k, v)| (k.as_str(), v))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl J {
    pub fn get(&self, k: &str) -> Option<&J> {
        match self {
            J::Obj(o) => o.get(k),
            _ => None,
        }
    }

    /// Python's `d.get(k)` on a JSON record, with null read as missing (both are None in Python)
    pub fn field(&self, k: &str) -> Option<&J> {
        self.get(k).filter(|v| !v.is_null())
    }

    /// a string field, or None when it's missing, null or not a string
    pub fn s(&self, k: &str) -> Option<&str> {
        self.get(k).and_then(|v| v.as_str())
    }

    /// Python's `d.get(k) or ''` for a string field
    pub fn s_or<'a>(&'a self, k: &str) -> &'a str {
        self.s(k).unwrap_or("")
    }

    pub fn is_null(&self) -> bool {
        matches!(self, J::Null)
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            J::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            J::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_arr(&self) -> Option<&Vec<J>> {
        match self {
            J::Arr(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_obj(&self) -> Option<&Obj> {
        match self {
            J::Obj(o) => Some(o),
            _ => None,
        }
    }

    pub fn as_obj_mut(&mut self) -> Option<&mut Obj> {
        match self {
            J::Obj(o) => Some(o),
            _ => None,
        }
    }

    /// Python truthiness
    pub fn truthy(&self) -> bool {
        match self {
            J::Null => false,
            J::Bool(b) => *b,
            J::Int(i) => *i != 0,
            J::Float(f) => *f != 0.0,
            J::Str(s) => !s.is_empty(),
            J::Arr(a) => !a.is_empty(),
            J::Obj(o) => !o.is_empty(),
        }
    }

    /// the strings of an array field (a Scryfall `keywords` / `color_identity` list)
    pub fn strs(&self, k: &str) -> Vec<&str> {
        self.get(k).and_then(|v| v.as_arr()).map(|a| a.iter().filter_map(|x| x.as_str()).collect()).unwrap_or_default()
    }

    /// Python's `str(v)` for the values a tag can hold
    pub fn py_str(&self) -> String {
        match self {
            J::Null => "None".into(),
            J::Bool(true) => "True".into(),
            J::Bool(false) => "False".into(),
            J::Int(i) => i.to_string(),
            J::Float(f) => py_float(*f),
            J::Str(s) => s.clone(),
            other => dumps(other, None, false),
        }
    }
}

impl From<&str> for J {
    fn from(s: &str) -> J {
        J::Str(s.to_string())
    }
}
impl From<String> for J {
    fn from(s: String) -> J {
        J::Str(s)
    }
}
impl From<&String> for J {
    fn from(s: &String) -> J {
        J::Str(s.clone())
    }
}
impl From<i64> for J {
    fn from(i: i64) -> J {
        J::Int(i)
    }
}
impl From<i32> for J {
    fn from(i: i32) -> J {
        J::Int(i as i64)
    }
}
impl From<usize> for J {
    fn from(i: usize) -> J {
        J::Int(i as i64)
    }
}
impl From<bool> for J {
    fn from(b: bool) -> J {
        J::Bool(b)
    }
}
impl From<Vec<J>> for J {
    fn from(a: Vec<J>) -> J {
        J::Arr(a)
    }
}
impl From<Obj> for J {
    fn from(o: Obj) -> J {
        J::Obj(o)
    }
}
impl<T: Into<J>> From<Option<T>> for J {
    fn from(o: Option<T>) -> J {
        o.map(Into::into).unwrap_or(J::Null)
    }
}

/// An ordered object: `obj! {"do" => "draw", "n" => 2}`
#[macro_export]
macro_rules! obj {
    () => { $crate::json::Obj::new() };
    ($($k:expr => $v:expr),+ $(,)?) => {{
        let mut o = $crate::json::Obj::new();
        $( o.0.push(($k.to_string(), $crate::json::J::from($v))); )+
        o
    }};
}

/// An ordered object as a `J`
#[macro_export]
macro_rules! jo {
    ($($t:tt)*) => { $crate::json::J::Obj($crate::obj!($($t)*)) };
}

// ------------------------------------------------------------------ reading (keys in document order)
impl<'de> Deserialize<'de> for J {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<J, D::Error> {
        d.deserialize_any(JVisitor)
    }
}

struct JVisitor;

impl<'de> Visitor<'de> for JVisitor {
    type Value = J;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a JSON value")
    }
    fn visit_bool<E>(self, v: bool) -> Result<J, E> {
        Ok(J::Bool(v))
    }
    fn visit_i64<E>(self, v: i64) -> Result<J, E> {
        Ok(J::Int(v))
    }
    fn visit_u64<E: de::Error>(self, v: u64) -> Result<J, E> {
        i64::try_from(v).map(J::Int).map_err(|_| E::custom("integer too large"))
    }
    fn visit_f64<E>(self, v: f64) -> Result<J, E> {
        Ok(J::Float(v))
    }
    fn visit_str<E>(self, v: &str) -> Result<J, E> {
        Ok(J::Str(v.to_string()))
    }
    fn visit_string<E>(self, v: String) -> Result<J, E> {
        Ok(J::Str(v))
    }
    fn visit_unit<E>(self) -> Result<J, E> {
        Ok(J::Null)
    }
    fn visit_none<E>(self) -> Result<J, E> {
        Ok(J::Null)
    }
    fn visit_some<D: Deserializer<'de>>(self, d: D) -> Result<J, D::Error> {
        J::deserialize(d)
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<J, A::Error> {
        let mut v = Vec::new();
        while let Some(x) = a.next_element()? {
            v.push(x);
        }
        Ok(J::Arr(v))
    }
    fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<J, A::Error> {
        let mut o = Obj::new();
        while let Some((k, v)) = a.next_entry::<String, J>()? {
            o.set(&k, v);
        }
        Ok(J::Obj(o))
    }
}

pub fn parse(text: &str) -> Result<J, String> {
    serde_json::from_str(text).map_err(|e| e.to_string())
}

// ------------------------------------------------------------------ writing (Python's json.dump)
/// Python's `repr(float)`
pub fn py_float(f: f64) -> String {
    if f.is_nan() {
        return "NaN".into();
    }
    if f.is_infinite() {
        return if f > 0.0 { "Infinity".into() } else { "-Infinity".into() };
    }
    let a = f.abs();
    if a != 0.0 && !(1e-4..1e16).contains(&a) {
        // scientific: Python writes 1e+16, 1.5e-05
        let s = format!("{:e}", f);
        let (m, e) = s.split_once('e').unwrap();
        let e: i32 = e.parse().unwrap();
        return format!("{m}e{}{:02}", if e < 0 { '-' } else { '+' }, e.abs());
    }
    let s = format!("{:?}", f);
    if s.contains('.') || s.contains('e') { s } else { format!("{s}.0") }
}

fn write_str(out: &mut String, s: &str, ensure_ascii: bool) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c if ensure_ascii && (c as u32) > 0x7e => {
                let mut buf = [0u16; 2];
                for u in c.encode_utf16(&mut buf) {
                    out.push_str(&format!("\\u{:04x}", u));
                }
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

fn write(out: &mut String, v: &J, indent: Option<usize>, level: usize, ensure_ascii: bool) {
    let nl = |out: &mut String, lvl: usize| {
        if let Some(n) = indent {
            out.push('\n');
            out.push_str(&" ".repeat(n * lvl));
        }
    };
    let sep = if indent.is_some() { "," } else { ", " };
    match v {
        J::Null => out.push_str("null"),
        J::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        J::Int(i) => out.push_str(&i.to_string()),
        J::Float(f) => out.push_str(&py_float(*f)),
        J::Str(s) => write_str(out, s, ensure_ascii),
        J::Arr(a) => {
            if a.is_empty() {
                out.push_str("[]");
                return;
            }
            out.push('[');
            for (i, x) in a.iter().enumerate() {
                if i > 0 {
                    out.push_str(sep);
                }
                nl(out, level + 1);
                write(out, x, indent, level + 1, ensure_ascii);
            }
            nl(out, level);
            out.push(']');
        }
        J::Obj(o) => {
            if o.is_empty() {
                out.push_str("{}");
                return;
            }
            out.push('{');
            for (i, (k, x)) in o.0.iter().enumerate() {
                if i > 0 {
                    out.push_str(sep);
                }
                nl(out, level + 1);
                write_str(out, k, ensure_ascii);
                out.push_str(": ");
                write(out, x, indent, level + 1, ensure_ascii);
            }
            nl(out, level);
            out.push('}');
        }
    }
}

/// Python's `json.dumps(v, indent=indent, ensure_ascii=ensure_ascii)`
pub fn dumps(v: &J, indent: Option<usize>, ensure_ascii: bool) -> String {
    let mut out = String::new();
    write(&mut out, v, indent, 0, ensure_ascii);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_like_python() {
        let v = parse(r#"{"b": [1, {"x": []}, {}], "a": "Mauhúr \"q\"", "f": 1.0, "n": null}"#).unwrap();
        assert_eq!(dumps(&v, None, false), r#"{"b": [1, {"x": []}, {}], "a": "Mauhúr \"q\"", "f": 1.0, "n": null}"#);
        assert_eq!(dumps(&v, None, true), r#"{"b": [1, {"x": []}, {}], "a": "Mauhúr \"q\"", "f": 1.0, "n": null}"#);
        assert_eq!(dumps(&jo! {"a" => vec![J::Int(1)]}, Some(1), false), "{\n \"a\": [\n  1\n ]\n}");
        assert_eq!(py_float(1e16), "1e+16");
        assert_eq!(py_float(0.5), "0.5");
        assert_eq!(py_float(1000000.0), "1000000.0");
    }

    #[test]
    fn set_keeps_the_place_of_a_key() {
        let mut o = obj! {"a" => 1, "b" => 2};
        o.set("a", 3);
        o.set("c", 4);
        assert_eq!(o.keys().collect::<Vec<_>>(), ["a", "b", "c"]);
    }
}
