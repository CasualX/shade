
pub struct Surface {
	width: u32,
	height: u32,
	data: Vec<u32>,
}

impl Surface {
	pub fn new(width: u32, height: u32) -> Surface {
		Surface {
			width, height, data: vec![0; (width as usize * height as usize)]
		}
	}
}
