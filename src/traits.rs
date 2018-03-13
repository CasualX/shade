
pub trait Allocate<T> {
	fn allocate(&mut self, n: usize) -> &mut [T];
}

impl<T> Allocate<T> for Vec<T> {
	fn allocate(&mut self, n: usize) -> &mut [T] {
		if self.capacity() < n {
			let reserve = n - self.capacity();
			self.reserve(reserve);
		}
		unsafe {
			self.set_len(n);
			&mut self[..]
		}
	}
}
