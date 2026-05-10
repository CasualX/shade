use std::{mem, slice};

pub(crate) fn slice_map<T: Copy, U, R, const STACK_CAP: usize>(data: &[T], map: impl Fn(T) -> U, f: impl FnOnce(&[U]) -> R) -> R {
	if data.len() <= STACK_CAP {
		let mut stack = [const { mem::MaybeUninit::uninit() }; STACK_CAP];
		for (dst, src) in stack[..data.len()].iter_mut().zip(data.iter().copied()) {
			dst.write(map(src));
		}
		let mapped = unsafe { slice::from_raw_parts(stack.as_ptr() as *const U, data.len()) };
		f(mapped)
	}
	else {
		let mut heap = Vec::with_capacity(data.len());
		for src in data.iter().copied() {
			heap.push(map(src));
		}
		f(&heap)
	}
}
