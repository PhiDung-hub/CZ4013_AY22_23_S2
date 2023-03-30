use crate::ser::{Fragment, Map, Seq, Serialize};
use crate::{extend_lifetime, extend_lifetime_impl};

use std::borrow::Cow;
use std::boxed::Box;
use std::string::String;
use std::vec::Vec;

/// Serialize any serializable type into a JSON string.
pub fn to_string<T: ?Sized + Serialize>(value: &T) -> String {
    to_string_impl(&value)
}

fn to_string_impl(value: &dyn Serialize) -> String {
    let mut out_str = String::new();
    let mut serializer = Serializer { stack: Vec::new() };
    let mut fragment = value.begin();

    loop {
        match fragment {
            Fragment::Null => out_str.push_str("null"),
            Fragment::Bool(b) => out_str.push_str(if b { "true" } else { "false" }),
            Fragment::Str(s) => escape_str(&s, &mut out_str),
            Fragment::U64(u) => out_str.push_str(&u.to_string()),
            Fragment::I64(i) => out_str.push_str(&i.to_string()),
            Fragment::F64(f) => out_str.push_str(&format!("{:.2}", f)),
            Fragment::Seq(mut seq) => {
                out_str.push('[');
                match unsafe { extend_lifetime!(seq.next() as Option<&dyn Serialize>) } {
                    Some(first) => {
                        serializer.stack.push(Layer::Seq(seq));
                        fragment = first.begin();
                        continue;
                    }
                    None => out_str.push(']'),
                }
            }
            Fragment::Map(mut map) => {
                out_str.push('{');
                match unsafe { extend_lifetime!(map.next() as Option<(Cow<str>, &dyn Serialize)>) } {
                    Some((key, first)) => {
                        escape_str(&key, &mut out_str);
                        out_str.push(':');
                        serializer.stack.push(Layer::Map(map));
                        fragment = first.begin();
                        continue;
                    }
                    None => out_str.push('}'),
                }
            }
        }

        loop {
            match serializer.stack.last_mut() {
                Some(Layer::Seq(seq)) => match unsafe { extend_lifetime!(seq.next() as Option<&dyn Serialize>) } {
                    Some(next) => {
                        out_str.push(',');
                        fragment = next.begin();
                        break;
                    }
                    None => out_str.push(']'),
                },
                Some(Layer::Map(map)) => match unsafe { extend_lifetime!(map.next() as Option<(Cow<str>, &dyn Serialize)>) } {
                    Some((key, next)) => {
                        out_str.push(',');
                        escape_str(&key, &mut out_str);
                        out_str.push(':');
                        fragment = next.begin();
                        break;
                    }
                    None => out_str.push('}'),
                },
                None => return out_str,
            }
            serializer.stack.pop();
        }
    }
}

struct Serializer<'a> {
    stack: Vec<Layer<'a>>,
}

enum Layer<'a> {
    Seq(Box<dyn Seq + 'a>),
    Map(Box<dyn Map + 'a>),
}

impl<'a> Drop for Serializer<'a> {
    fn drop(&mut self) {
        // Drop layers in reverse order.
        while !self.stack.is_empty() {
            self.stack.pop();
        }
    }
}

fn escape_str(value: &str, out: &mut String) {
    out.push('"');

    let bytes = value.as_bytes();
    let mut start = 0;

    for (i, &byte) in bytes.iter().enumerate() {
        let escape = ESCAPE[byte as usize];
        if escape == 0 {
            continue;
        }

        if start < i {
            out.push_str(&value[start..i]);
        }

        match escape {
            self::B_ => out.push_str("\\b"),
            self::T_ => out.push_str("\\t"),
            self::N_ => out.push_str("\\n"),
            self::F_ => out.push_str("\\f"),
            self::R_ => out.push_str("\\r"),
            self::QT => out.push_str("\\\""),
            self::BS => out.push_str("\\\\"),
            self::U => {
                static HEX_DIGITS: [u8; 16] = *b"0123456789abcdef";
                out.push_str("\\u00");
                out.push(HEX_DIGITS[(byte >> 4) as usize] as char);
                out.push(HEX_DIGITS[(byte & 0xF) as usize] as char);
            }
            _ => unreachable!(),
        }

        start = i + 1;
    }

    if start != bytes.len() {
        out.push_str(&value[start..]);
    }

    out.push('"');
}

const B_: u8 = b'b'; // \x08
const T_: u8 = b't'; // \x09
const N_: u8 = b'n'; // \x0A
const F_: u8 = b'f'; // \x0C
const R_: u8 = b'r'; // \x0D
const QT: u8 = b'"'; // \x22
const BS: u8 = b'\\'; // \x5C
const U: u8 = b'u'; // \x00...\x1F except the ones above

// Lookup table of escape sequences. A value of b'x' at index i means that byte
// i is escaped as "\x" in JSON. A value of 0 means that byte i is not escaped.
#[rustfmt::skip]
static ESCAPE: [u8; 256] = [
    //  1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
    U,  U,  U,  U,  U,  U,  U,  U, B_, T_, N_,  U, F_, R_,  U,  U, // 0
    U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U, // 1
    0,  0, QT,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 2
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 3
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 4
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, BS,  0,  0,  0, // 5
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 6
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 7
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 8
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 9
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // A
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // B
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // C
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // D
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // E
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // F
];
