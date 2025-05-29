#![allow(dead_code)]

use std::{fmt, hash};

pub trait Handle: Copy + Clone + Default + fmt::Debug + Eq + PartialEq + hash::Hash {
	type Raw: Copy + Clone + fmt::Debug + Eq + PartialEq + hash::Hash;
	fn create(raw: Self::Raw) -> Self;
	fn id(&self) -> Self::Raw;
	fn next(&self) -> Self;
}

macro_rules! define_handle {
	($name:ident) => {
		#[doc = concat!(stringify!($name), " handle.")]
		#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
		#[repr(transparent)]
		pub struct $name(u32);

		unsafe impl dataview::Pod for $name {}

		impl $name {
			/// Invalid handle.
			pub const INVALID: $name = $name(0);
		}

		impl crate::handle::Handle for $name {
			type Raw = u32;

			#[inline]
			fn create(raw: u32) -> Self {
				$name(raw)
			}

			#[inline]
			fn id(&self) -> u32 {
				self.0
			}

			#[inline]
			fn next(&self) -> Self {
				$name(self.0 + 1)
			}
		}
	};
}
