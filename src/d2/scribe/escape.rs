#![allow(dead_code)]

use super::*;

#[inline]
fn parse_hexdigit(c: u8) -> Option<u8> {
	match c {
		b'0'..=b'9' => Some(c - b'0'),
		b'a'..=b'f' => Some(c - b'a' + 10),
		b'A'..=b'F' => Some(c - b'A' + 10),
		_ => None,
	}
}

fn parse_hexcolor(s: &str) -> Option<Vec4<u8>> {
	let mut s = s.as_bytes();
	if s.first() == Some(&b'#') {
		s = &s[1..];
	}
	if s.len() > 8 {
		return None;
	}
	let mut digits = [0u8; 8];
	for i in 0..s.len() {
		digits[i] = parse_hexdigit(s[i])?;
	}
	let color = match s.len() {
		1 => {
			let v = digits[0] * 17;
			Vec4(v, v, v, 255)
		}
		3 => {
			let r = digits[0] * 17;
			let g = digits[1] * 17;
			let b = digits[2] * 17;
			Vec4(r, g, b, 255)
		}
		4 => {
			let r = digits[0] * 17;
			let g = digits[1] * 17;
			let b = digits[2] * 17;
			let a = digits[3] * 17;
			Vec4(r, g, b, a)
		}
		6 => {
			let r = digits[0] * 16 + digits[1];
			let g = digits[2] * 16 + digits[3];
			let b = digits[4] * 16 + digits[5];
			Vec4(r, g, b, 255)
		}
		8 => {
			let r = digits[0] * 16 + digits[1];
			let g = digits[2] * 16 + digits[3];
			let b = digits[4] * 16 + digits[5];
			let a = digits[6] * 16 + digits[7];
			Vec4(r, g, b, a)
		}
		_ => return None,
	};
	Some(color)
}

#[inline]
fn split_cmp<'a>(seq: &'a str, key: &str) -> Option<&'a str> {
	if !seq.starts_with(key) {
		return None;
	}
	// SAFETY: key is a prefix of seq.
	Some(unsafe { seq.get_unchecked(key.len()..) })
}

#[inline(never)]
fn process2_color(seq: &str, key: &str, color: &mut Vec4<u8>) -> bool {
	let Some(tail) = split_cmp(seq, key) else {
		return false;
	};

	let Some(value) = split_cmp(tail, "=") else {
		#[cfg(debug_assertions)]
		panic!("Invalid escape sequence: {}", seq);
		#[cfg(not(debug_assertions))]
		return false;
	};

	// if value == "reset" {
	// 	*color = *init;
	// 	return true;
	// }

	let Some(value) = parse_hexcolor(value) else {
		#[cfg(debug_assertions)]
		panic!("Invalid color syntax: {}", value);
		#[cfg(not(debug_assertions))]
		return false;
	};

	*color = value;
	true
}

fn process2_f32(seq: &str, key: &str, float: &mut f32) -> bool {
	let Some(tail) = split_cmp(seq, key) else {
		return false;
	};

	// if tail == "=reset" {
	// 	*float = *init;
	// 	return true;
	// }

	let (tail, fun) = match tail.as_bytes().first() {
		Some(b'+') => (&tail[1..], (|lhs, rhs| lhs + rhs) as fn(f32, f32) -> f32),
		Some(b'-') => (&tail[1..], (|lhs, rhs| lhs - rhs) as fn(f32, f32) -> f32),
		Some(b'*') => (&tail[1..], (|lhs, rhs| lhs * rhs) as fn(f32, f32) -> f32),
		Some(b'/') => (&tail[1..], (|lhs, rhs| lhs / rhs) as fn(f32, f32) -> f32),
		_ => (tail, (|_, value| value) as fn(f32, f32) -> f32),
	};

	let Some(value) = split_cmp(tail, "=") else {
		#[cfg(debug_assertions)]
		panic!("Invalid escape sequence: {}", seq);
		#[cfg(not(debug_assertions))]
		return false;
	};

	let value = match value.parse() {
		Ok(value) => value,
		Err(_err) => {
			#[cfg(debug_assertions)]
			panic!("Invalid number syntax: {}: {}", value, _err);
			#[cfg(not(debug_assertions))]
			return false;
		},
	};

	*float = fun(*float, value);
	true
}

/// Process an escape sequence.
///
/// These allow for changing the scribe properties in the middle of a text string.
///
/// Syntax is `\x1b[key?=value]`:
///
/// * `key` is the name of the parameter to change.
/// * `?` is an optional operator (`+` `-` `*` `/`) to apply to the current value, only used when the property is a number.
/// * `value` is the new value to set.
///   * If the property is a color, it will be parsed as a hex color (syntax: `#V`, `#RGB`, `#RGBA`, `#RRGGBB`, `#RRGGBBAA`).
///   * If the property is a number, it will be parsed as a float.
///
/// # Panics
///
/// Panics if the key is unknown, or if the value is invalid.
#[inline(never)]
pub(crate) fn process(sequence: &str, scribe: &mut Scribe, _cv: Option<&mut TextBuffer>) {
	macro_rules! def_handler {
		($handler:ident, $key:ident) => {
			&mut |seq| $handler(seq, stringify!($key), &mut scribe.$key)
		}
	}
	let handlers: [&mut dyn FnMut(&str) -> bool; 9] = [
		def_handler!(process2_f32, font_size),
		def_handler!(process2_f32, font_width_scale),
		def_handler!(process2_f32, line_height),
		def_handler!(process2_f32, baseline),
		def_handler!(process2_f32, x_pos),
		def_handler!(process2_f32, letter_spacing),
		def_handler!(process2_f32, top_skew),
		def_handler!(process2_color, color),
		def_handler!(process2_color, outline),
	];
	let key_chars = [b'f', b'f', b'l', b'b', b'x', b'l', b't', b'c', b'o'];
	assert_eq!(handlers.len(), key_chars.len());

	let mut success = false;
	if let Some(&first) = sequence.as_bytes().first() {
		for i in 0..handlers.len() {
			if first == key_chars[i] {
				if handlers[i](sequence) {
					success = true;
					break;
				}
			}
		}
	}
	if !success {
		#[cfg(debug_assertions)]
		panic!("Unknown escape sequence: {}", sequence);
	}
}
