//! OpenGL generation tracking

use std::sync::atomic::{AtomicU32, Ordering};

static GENERATION: AtomicU32 = AtomicU32::new(0);

pub fn next() -> u32 {
	GENERATION.fetch_add(1, Ordering::AcqRel).wrapping_add(1)
}

pub fn drop(generation: u32) {
	let _ = GENERATION.compare_exchange(generation, generation.wrapping_add(1), Ordering::AcqRel, Ordering::Acquire);
}

pub fn is_current(generation: u32) -> bool {
	GENERATION.load(Ordering::Acquire) == generation
}
