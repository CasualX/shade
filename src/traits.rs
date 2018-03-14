
/// Allocator trait to allocate canvas resources.
pub trait Allocate<T: Copy + 'static> {
	/// Allocates a number of logically uninitialized `T`.
	unsafe fn allocate(&mut self, n: usize) -> &mut [T];
}

impl<T: Copy + 'static> Allocate<T> for Vec<T> {
	unsafe fn allocate(&mut self, n: usize) -> &mut [T] {
		let len = self.len();
		if self.capacity() < len + n {
			let reserve = len + n - self.capacity();
			self.reserve(reserve);
		}
		self.set_len(len + n);
		self.get_unchecked_mut(len..)
	}
}
