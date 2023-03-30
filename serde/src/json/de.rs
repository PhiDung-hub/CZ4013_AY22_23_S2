use self::Event::*; // hoisting, being explicit
use crate::de::{Deserialize, Map, Seq, Visitor};
use crate::non_unique_box::NonuniqueBox;
use crate::{extend_lifetime, extend_lifetime_impl};
use crate::{Error, Result};

use core::ptr::NonNull;
use core::str;
use std::vec::Vec;

enum Event<'a> {
    Null,
    Bool(bool),
    Str(&'a str),
    Negative(i64),
    Nonnegative(u64),
    Float(f64),
    SeqStart,
    MapStart,
}

/// Deserialize a JSON string into any deserializable type.
/// Non recursive impls using a stack.
pub fn from_str<T: Deserialize>(j: &str) -> Result<T> {
    let mut out = None;
    internal_from_str(j, T::begin(&mut out))?;
    out.ok_or(Error)
}

struct Deserializer<'a, 'b> {
    input: &'a [u8],
    pos: usize,
    buffer: Vec<u8>,
    stack: Vec<(NonNull<dyn Visitor>, Layer<'b>)>,
}

enum Layer<'a> {
    Seq(NonuniqueBox<dyn Seq + 'a>),
    Map(NonuniqueBox<dyn Map + 'a>),
}

impl<'a, 'b> Drop for Deserializer<'a, 'b> {
    fn drop(&mut self) {
        while self.stack.pop().is_some() {}
    }
}

fn internal_from_str(s: &str, visitor: &mut dyn Visitor) -> Result<()> {
    let visitor = NonNull::from(visitor);
    let mut visitor = unsafe { extend_lifetime!(visitor as NonNull<dyn Visitor>) };
    let mut deserializer = Deserializer {
        input: s.as_bytes(),
        pos: 0,
        buffer: Vec::new(),
        stack: Vec::new(),
    };

    'outer_loop: loop {
        let visitor_mut = unsafe { &mut *visitor.as_ptr() };
        let layer = match deserializer.get_event()? {
            Null => {
                visitor_mut.null()?;
                None
            }
            Bool(b) => {
                visitor_mut.boolean(b)?;
                None
            }
            Negative(n) => {
                visitor_mut.negative(n)?;
                None
            }
            Nonnegative(n) => {
                visitor_mut.nonnegative(n)?;
                None
            }
            Float(n) => {
                visitor_mut.float(n)?;
                None
            }
            Str(s) => {
                visitor_mut.string(s)?;
                None
            }
            SeqStart => {
                let seq = visitor_mut.seq()?;
                Some(Layer::Seq(NonuniqueBox::from(seq)))
            }
            MapStart => {
                let map = visitor_mut.map()?;
                Some(Layer::Map(NonuniqueBox::from(map)))
            }
        };

        let mut accept_comma;
        let mut layer = match layer {
            Some(layer) => {
                accept_comma = false;
                layer
            }
            None => match deserializer.stack.pop() {
                Some(frame) => {
                    accept_comma = true;
                    visitor = frame.0;
                    frame.1
                }
                None => break 'outer_loop,
            },
        };

        loop {
            match deserializer.skip_whitespace().unwrap_or(b'\0') {
                b',' if accept_comma => {
                    deserializer.move_next_pos();
                    break;
                }
                close_parenthesis @ (b']' | b'}') => {
                    deserializer.move_next_pos();
                    match &mut layer {
                        Layer::Seq(seq) if close_parenthesis == b']' => seq.finish()?,
                        Layer::Map(map) if close_parenthesis == b'}' => map.finish()?,
                        _ => return Err(Error),
                    };
                    let frame = match deserializer.stack.pop() {
                        Some(frame) => frame,
                        None => break 'outer_loop,
                    };
                    accept_comma = true;
                    visitor = frame.0;
                    layer = frame.1;
                }
                _ => {
                    if accept_comma {
                        return Err(Error);
                    }
                    break;
                }
            }
        }

        let outer = visitor;
        match layer {
            Layer::Seq(mut seq) => {
                let element = seq.element()?;
                let next = NonNull::from(element);
                visitor = unsafe { extend_lifetime!(next as NonNull<dyn Visitor>) };
                deserializer.stack.push((outer, Layer::Seq(seq)));
            }
            Layer::Map(mut map) => {
                match deserializer.skip_whitespace() {
                    Some(b'"') => deserializer.move_next_pos(),
                    _ => return Err(Error),
                }
                let key = deserializer.parse_str()?;
                let entry = map.key(key)?;
                let next = NonNull::from(entry);
                visitor = unsafe { extend_lifetime!(next as NonNull<dyn Visitor>) };
                match deserializer.skip_whitespace() {
                    Some(b':') => deserializer.move_next_pos(),
                    _ => return Err(Error),
                }
                deserializer.stack.push((outer, Layer::Map(map)));
            }
        }
    }

    // Remove remaining whitespace, raise error if unusual character found
    match deserializer.skip_whitespace() {
        Some(_) => Err(Error),
        None => Ok(()),
    }
}

/// Check if `10 * a + b` is overflow for some bounded experssion `c`
/// For example
/// ```rust
/// let a = 214_748_364;
/// let b = 9;
/// // i32::MAX = 2_147_483_647;
/// overflow!(a * 10 + b, i32::MAX); // true
/// ```
macro_rules! overflow {
    ($a:ident * 10 + $b:ident, $c:expr) => {
        $a > $c / 10 || ($a >= $c / 10 && $b > $c % 10)
    };
}

impl<'a, 'b> Deserializer<'a, 'b> {
    fn next(&mut self) -> Option<u8> {
        let cur_pos = self.pos;
        if cur_pos >= self.input.len() {
            return None;
        }

        self.move_next_pos();
        Some(self.input[cur_pos])
    }

    fn next_or_null(&mut self) -> u8 {
        self.next().unwrap_or(b'\0')
    }

    fn peek(&mut self) -> Option<u8> {
        let cur_pos = self.pos;
        if cur_pos >= self.input.len() {
            return None;
        }
        Some(self.input[cur_pos])
    }

    fn peek_or_null(&mut self) -> u8 {
        self.peek().unwrap_or(b'\0')
    }

    fn move_next_pos(&mut self) {
        self.pos += 1;
    }

    fn get_event(&mut self) -> Result<Event> {
        let peek = match self.skip_whitespace() {
            Some(b) => b,
            None => return Err(Error),
        };
        self.move_next_pos();
        match peek {
            b'"' => self.parse_str().map(Str),
            digit @ b'0'..=b'9' => self.parse_integer(true, digit),
            b'-' => {
                let first_digit = self.next_or_null();
                self.parse_integer(false, first_digit)
            }
            b'{' => Ok(MapStart),
            b'[' => Ok(SeqStart),
            b'n' => {
                self.parse_ident(b"ull")?;
                Ok(Null)
            }
            b't' => {
                self.parse_ident(b"rue")?;
                Ok(Bool(true))
            }
            b'f' => {
                self.parse_ident(b"alse")?;
                Ok(Bool(false))
            }
            _ => Err(Error),
        }
    }

    fn parse_str(&mut self) -> Result<&str> {
        fn result(bytes: &[u8]) -> &str {
            unsafe { str::from_utf8_unchecked(bytes) }
        }

        // Index of the first byte not yet copied into the scratch space.
        let mut start = self.pos;
        self.buffer.clear();

        loop {
            while !ESCAPE[(self.input[self.pos]) as usize] {
                self.move_next_pos();
                if self.pos == self.input.len() {
                    return Err(Error);
                }
            }

            let borrowed_str = &self.input[start..self.pos];
            let cur_char = self.input[self.pos];

            if cur_char != b'\\' && cur_char != b'"' {
                return Err(Error);
            }
            self.move_next_pos();
            match cur_char {
                b'\\' => {
                    self.buffer.extend_from_slice(&borrowed_str);
                    self.parse_escape()?;
                    start = self.pos;
                }
                b'"' => {
                    if self.buffer.is_empty() {
                        return Ok(result(borrowed_str));
                    } else {
                        self.buffer.extend_from_slice(&borrowed_str);
                        return Ok(result(&self.buffer));
                    }
                }
                _ => unreachable!(),
            }
        }
    }

    fn next_or_eof(&mut self) -> Result<u8> {
        self.next().ok_or(Error)
    }

    fn parse_escape(&mut self) -> Result<()> {
        let ch = self.next_or_eof()?;

        match ch {
            b'"' => self.buffer.push(b'"'),
            b'\\' => self.buffer.push(b'\\'),
            b'/' => self.buffer.push(b'/'),
            b'b' => self.buffer.push(b'\x08'),
            b'f' => self.buffer.push(b'\x0c'),
            b'n' => self.buffer.push(b'\n'),
            b'r' => self.buffer.push(b'\r'),
            b't' => self.buffer.push(b'\t'),
            _ => {
                return Err(Error);
            }
        }

        Ok(())
    }

    fn skip_whitespace(&mut self) -> Option<u8> {
        let is_whitespace = |char: Option<u8>| match char {
            Some(b' ') | Some(b'\n') | Some(b'\t') | Some(b'\r') => true,
            _ => false,
        };
        while is_whitespace(self.peek()) {
            self.move_next_pos();
        }

        self.peek()
    }

    fn parse_ident(&mut self, ident: &[u8]) -> Result<()> {
        for &expected in ident {
            let next = self.next();
            if next.is_none() || next.unwrap() != expected {
                return Err(Error);
            }
        }
        Ok(())
    }

    fn parse_integer(&mut self, nonnegative: bool, first_digit: u8) -> Result<Event> {
        if !first_digit.is_ascii_digit() {
            return Err(Error);
        }
        if first_digit == b'0' {
            if self.peek_or_null().is_ascii_digit() {
                return Err(Error);
            }
            return self.parse_number(nonnegative, 0);
        }

        let mut number = (first_digit - b'0') as u64;

        while let Some(d) = self.peek() {
            if !d.is_ascii_digit() {
                break;
            }
            self.move_next_pos();

            let digit = (d - b'0') as u64;
            if overflow!(number * 10 + digit, u64::MAX) {
                // use long f64 if overflow, skip the digit, i.e. result *10
                return self.parse_long_integer(nonnegative, number).map(Float);
            }

            number = number * 10 + digit;
        }
        return self.parse_number(nonnegative, number);
    }

    fn parse_long_integer(&mut self, nonnegative: bool, mantissa: u64) -> Result<f64> {
        let mut base_10_exp = 1;
        while self.peek_or_null().is_ascii_digit() {
            self.move_next_pos();
            base_10_exp += 1;
        }

        match self.peek_or_null() {
            b'.' => {
                return self.parse_decimal(nonnegative, mantissa, base_10_exp);
            }
            b'e' | b'E' => {
                return self.parse_exponent(nonnegative, mantissa, base_10_exp);
            }
            _ => {
                return construct_f64(nonnegative, mantissa, base_10_exp);
            }
        }
    }

    fn parse_number(&mut self, nonnegative: bool, mantissa: u64) -> Result<Event> {
        match self.peek_or_null() {
            b'.' => self.parse_decimal(nonnegative, mantissa, 0).map(Float),
            b'e' | b'E' => self.parse_exponent(nonnegative, mantissa, 0).map(Float),
            _ => {
                if nonnegative {
                    return Ok(Nonnegative(mantissa));
                }

                let neg = (mantissa as i64).wrapping_neg(); // negative underflow

                return Ok(if neg > 0 { Float(-(mantissa as f64)) } else { Negative(neg) });
            }
        }
    }

    fn parse_decimal(&mut self, nonnegative: bool, mut mantissa: u64, mut base_10_exp: i32) -> Result<f64> {
        self.move_next_pos();

        let mut at_least_one_digit = false;
        while let c @ b'0'..=b'9' = self.peek_or_null() {
            self.move_next_pos();
            let digit = (c - b'0') as u64;
            at_least_one_digit = true;

            if overflow!(mantissa * 10 + digit, u64::MAX) {
                while let b'0'..=b'9' = self.peek_or_null() {
                    self.move_next_pos();
                }
                break;
            }

            mantissa = mantissa * 10 + digit;
            base_10_exp -= 1;
        }

        if !at_least_one_digit {
            return Err(Error);
        }

        match self.peek_or_null() {
            b'e' | b'E' => self.parse_exponent(nonnegative, mantissa, base_10_exp),
            _ => construct_f64(nonnegative, mantissa, base_10_exp),
        }
    }

    fn parse_exponent(&mut self, nonnegative: bool, mantissa: u64, starting_base_10_exp: i32) -> Result<f64> {
        self.move_next_pos();

        let is_positive_exponent = match self.peek_or_null() {
            b'+' => {
                self.move_next_pos();
                true
            }
            b'-' => {
                self.move_next_pos();
                false
            }
            _ => true,
        };

        let _cur = self.next_or_null();
        if !_cur.is_ascii_digit() {
            return Err(Error);
        }
        let mut exp = (_cur - b'0') as i32;

        while let c @ b'0'..=b'9' = self.peek_or_null() {
            self.move_next_pos();
            let digit = (c - b'0') as i32;

            if overflow!(exp * 10 + digit, i32::MAX) {
                // Out of bounds -> Error
                if mantissa != 0 && is_positive_exponent {
                    return Err(Error);
                }

                while let b'0'..=b'9' = self.peek_or_null() {
                    self.move_next_pos();
                }

                // negligible small -> cast to absolute 0.
                return Ok(if nonnegative { 0.0 } else { -0.0 });
            }

            exp = exp * 10 + digit;
        }

        let final_exp = if is_positive_exponent {
            starting_base_10_exp.saturating_add(exp)
        } else {
            starting_base_10_exp.saturating_sub(exp)
        };

        construct_f64(nonnegative, mantissa, final_exp)
    }
}

fn construct_f64(nonnegative: bool, significand: u64, base_10_exp: i32) -> Result<f64> {
    let mut f = significand as f64;
    const BASE: f64 = 10.0;
    let pow = BASE.powi(base_10_exp);
    f *= pow;
    if f.is_infinite() {
        return Err(Error);
    }
    Ok(if nonnegative { f } else { -f })
}

const CT: bool = true; // control character \x00..=\x1F
const QU: bool = true; // quote \x22
const BS: bool = true; // backslash \x5C
const O: bool = false; // allow unescaped

// Lookup table of bytes for escape bytes. A value of true at index i means
// that byte i requires an escape sequence in the input.
#[rustfmt::skip]
static ESCAPE: [bool; 256] = [
    //   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
    CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, // 0
    CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, // 1
     O,  O, QU,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O, // 2
     O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O, // 3
     O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O, // 4
     O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O, BS,  O,  O,  O, // 5
     O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O, // 6
     O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O, // 7
     O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O, // 8
     O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O, // 9
     O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O, // A
     O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O, // B
     O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O, // C
     O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O, // D
     O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O, // E
     O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O,  O, // F
];
