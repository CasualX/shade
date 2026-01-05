use std::{borrow, fmt, hash, str};

/// Error returned by [`SmallString::new`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct InlineStringError {
	pub len: usize,
	pub max_len: usize,
}

/// A small inline UTF-8 string stored in a fixed buffer.
#[derive(Copy, Clone)]
pub struct SmallString<const N: usize> {
	buf: [u8; N],
}

impl<const N: usize> SmallString<N> {
	const MAX_LEN: usize = N - 1;

	/// Creates a new inline string.
	///
	/// Returns an error if `s.len() > N-1`.
	#[inline(always)]
	pub const fn new(s: &str) -> Result<Self, InlineStringError> {
		if N == 0 {
			panic!("SmallString<N>: N must be >= 1");
		}
		if Self::MAX_LEN > u8::MAX as usize {
			panic!("SmallString<N>: N must be <= 256 (length marker is one byte)");
		}

		let bytes = s.as_bytes();
		let len = bytes.len();

		if len > Self::MAX_LEN {
			return Err(InlineStringError {
				len,
				max_len: Self::MAX_LEN,
			});
		}

		let mut buf = [0u8; N];

		// Store payload in first `len` bytes. The rest stays 0.
		let mut i = 0;
		while i < len {
			buf[i] = bytes[i];
			i += 1;
		}

		// Store inverted length in the last byte: (N-1)-len.
		// If len==(N-1) then this becomes 0, keeping the whole buffer NUL-terminated.
		buf[N - 1] = (Self::MAX_LEN - len) as u8;

		Ok(Self { buf })
	}

	/// Returns the string length in bytes.
	#[inline]
	pub fn len(&self) -> usize {
		let inv = self.buf[N - 1] as usize;
		if inv > Self::MAX_LEN {
			unsafe { std::hint::unreachable_unchecked() }
		}
		Self::MAX_LEN - inv
	}

	/// Returns the string contents as bytes.
	#[inline]
	pub fn as_bytes(&self) -> &[u8] {
		&self.buf[..self.len()]
	}

	/// Returns the string contents as `&str`.
	#[inline]
	pub fn as_str(&self) -> &str {
		// Safe by construction: only created from valid UTF-8 `&str`.
		unsafe { str::from_utf8_unchecked(self.as_bytes()) }
	}
}

impl<const N: usize> From<&str> for SmallString<N> {
	#[inline]
	fn from(s: &str) -> Self {
		Self::new(s).unwrap()
	}
}

impl<const N: usize> PartialEq for SmallString<N> {
	#[inline]
	fn eq(&self, other: &Self) -> bool {
		self.as_bytes() == other.as_bytes()
	}
}
impl<const N: usize> Eq for SmallString<N> {}

impl<const N: usize> PartialEq<str> for SmallString<N> {
	#[inline]
	fn eq(&self, other: &str) -> bool {
		self.as_bytes() == other.as_bytes()
	}
}

impl<const N: usize> PartialEq<SmallString<N>> for str {
	#[inline]
	fn eq(&self, other: &SmallString<N>) -> bool {
		self.as_bytes() == other.as_bytes()
	}
}

impl<const N: usize> PartialEq<&str> for SmallString<N> {
	#[inline]
	fn eq(&self, other: &&str) -> bool {
		self.as_bytes() == other.as_bytes()
	}
}

impl<const N: usize> PartialEq<SmallString<N>> for &str {
	#[inline]
	fn eq(&self, other: &SmallString<N>) -> bool {
		self.as_bytes() == other.as_bytes()
	}
}

impl<const N: usize> borrow::Borrow<str> for SmallString<N> {
	#[inline]
	fn borrow(&self) -> &str {
		self.as_str()
	}
}

impl<const N: usize> hash::Hash for SmallString<N> {
	#[inline]
	fn hash<H: hash::Hasher>(&self, state: &mut H) {
		self.as_str().hash(state)
	}
}

impl<const N: usize> fmt::Debug for SmallString<N> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.as_str().fmt(f)
	}
}
impl<const N: usize> fmt::Display for SmallString<N> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::collections::HashMap;

	#[test]
	fn stores_inverted_len_and_is_nul_terminated() {
		type S = SmallString<64>;
		let s = S::new("abc").unwrap();
		assert_eq!(s.len(), 3);
		assert_eq!(s.as_bytes(), b"abc");
		assert_eq!(s.as_str(), "abc");

		// NUL terminator after the string (since buf is zero-initialized)
		assert_eq!(s.buf[3], 0);

		// Inverted length stored in last byte
		assert_eq!(s.buf[63], 60);
	}

	#[test]
	fn len_63_keeps_last_byte_nul() {
		type S = SmallString<64>;
		let input = "a".repeat(63);
		let s = S::new(&input).unwrap();
		assert_eq!(s.len(), 63);
		assert_eq!(s.buf[63], 0); // inverted len for 63 -> 0, also NUL terminator
		assert_eq!(s.as_str(), input);
	}

	#[test]
	fn too_long_errors() {
		type S = SmallString<64>;
		let input = "a".repeat(64);
		let err = S::new(&input).unwrap_err();
		assert_eq!(err.max_len, 63);
		assert_eq!(err.len, 64);
	}

	#[test]
	fn hashmap_lookup_by_str() {
		type K = SmallString<64>;
		let mut map: HashMap<K, u32> = HashMap::new();
		map.insert(K::new("a_pos").unwrap(), 123);

		// Requires Borrow<str> + Hash/Eq for str.
		assert_eq!(map.get("a_pos").copied(), Some(123));
		assert_eq!(map.get("a_normal"), None);
	}

	#[test]
	fn hashmap_trailing_nul_is_different_key() {
		type K = SmallString<64>;
		let mut map: HashMap<K, u32> = HashMap::new();
		map.insert(K::new("a_pos").unwrap(), 123);

		let with_nul = K::new("a_pos\0").unwrap();
		assert_eq!(with_nul.len(), 6);
		assert_ne!(with_nul.as_bytes(), b"a_pos");

		// This is the kind of mismatch that can look identical in logs.
		assert_eq!(map.get(&with_nul), None);
		assert_eq!(map.get(with_nul.as_str()), None);
	}
}

