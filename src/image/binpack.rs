
/// Simple grid-based bin packer.
pub struct GridBinPacker {
	cell_width: i32,
	cell_height: i32,
	width_in_cells: i32,
	height_in_cells: i32,
	start_x: i32,
	start_y: i32,
	occupied: Vec<bool>,
}

impl GridBinPacker {
	/// Creates a new instance.
	#[inline]
	pub fn new(px_width: i32, px_height: i32, cell_width: i32, cell_height: i32) -> GridBinPacker {
		assert!(px_width >= 0 && px_height >= 0, "Width and height must be non-negative");
		assert!(cell_width > 0 && cell_height > 0, "Cell width and height must be positive");
		let width_in_cells = px_width / cell_width;
		let height_in_cells = px_height / cell_height;
		let occupied = vec![false; (width_in_cells * height_in_cells) as usize];
		GridBinPacker { cell_width, cell_height, width_in_cells, height_in_cells, start_x: 0, start_y: 0, occupied }
	}

	/// Returns the number of cells in the bin packer.
	#[inline]
	pub fn len(&self) -> usize {
		self.occupied.len()
	}

	/// Inserts a rectangle of pixel size `(width, height)` into the bin packer.
	///
	/// Returns the `(x, y)` position of the inserted rectangle in pixels, or `None` if there is no space.
	#[inline]
	pub fn insert(&mut self, width: i32, height: i32) -> Option<(i32, i32)> {
		assert!(width > 0 && height > 0, "Width and height must be positive");
		let wc = ceil_div(width, self.cell_width);
		let hc = ceil_div(height, self.cell_height);
		self._insert(wc, hc)
	}

	// Find the next free cell in row-major order starting from the
	// previous start position. This is independent of the requested
	// rectangle size so it stays correct across inserts of different sizes.
	#[inline(always)]
	fn _update_start(&mut self) {
		let mut idx = self.start_y * self.width_in_cells + self.start_x;
		while idx < self.len() as i32 {
			let uidx = idx as usize;
			if let Some(&occupied) = self.occupied.get(uidx) {
				if !occupied {
					break;
				}
			}
			idx += 1;
		}
		self.start_y = idx / self.width_in_cells;
		self.start_x = idx % self.width_in_cells;
	}

	#[inline(never)]
	fn _insert(&mut self, w: i32, h: i32) -> Option<(i32, i32)> {
		if w > self.width_in_cells || h > self.height_in_cells {
			return None;
		}

		self._update_start();

		let mut start_x = self.start_x;
		for y in self.start_y..(self.height_in_cells - h + 1) {
			for x in start_x..(self.width_in_cells - w + 1) {
				if self.try_mark(x, y, w, h) {
					return Some((x * self.cell_width, y * self.cell_height));
				}
			}
			start_x = 0;
		}

		// No space found
		return None;
	}

	#[inline(always)]
	fn try_mark(&mut self, x: i32, y: i32, w: i32, h: i32) -> bool {
		// Naive check for occupancy
		for yy in y..y + h {
			for xx in x..x + w {
				let index = (yy * self.width_in_cells + xx) as usize;
				if let Some(&mut occupied) = self.occupied.get_mut(index) {
					if occupied {
						// Already occupied
						return false;
					}
				}
			}
		}
		// Mark as occupied
		for yy in y..y + h {
			for xx in x..x + w {
				let index = (yy * self.width_in_cells + xx) as usize;
				if let Some(occupied) = self.occupied.get_mut(index) {
					*occupied = true;
				}
			}
		}
		return true;
	}
}

#[inline]
fn ceil_div(a: i32, b: i32) -> i32 {
	(a + b - 1) / b
}

#[test]
fn test_grid_bin_packer() {
	let mut packer = GridBinPacker::new(100, 100, 10, 10);
	assert_eq!(packer.insert(20, 20), Some((0, 0)));
	assert_eq!(packer.insert(30, 30), Some((20, 0)));
	assert_eq!(packer.insert(50, 50), Some((50, 0)));
	assert_eq!(packer.insert(10, 10), Some((0, 20)));
	assert_eq!(packer.insert(90, 90), None);
	assert_eq!(packer.insert(10, 10), Some((10, 20)));
}
