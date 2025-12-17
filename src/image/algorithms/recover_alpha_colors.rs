use super::*;

fn recover_alpha_colors(image: &Image<[u8; 4]>) -> Image<[u8; 4]> {
	let mut data = vec![[0u8; 4]; image.len()];

	// Copy the pixels but if the src is transparent (alpha == 0)
	// Then take the average of the surrounding (non transparent) pixels
	for y in 0..image.height {
		for x in 0..image.width {
			let idx = image.index(x, y).unwrap();
			let alpha = image.data[idx][3];

			if alpha == 0 {
				let mut r_sum = 0u32;
				let mut g_sum = 0u32;
				let mut b_sum = 0u32;
				let mut count = 0;

				for dy in -1..=1 {
					for dx in -1..=1 {
						if dx == 0 && dy == 0 {
							continue; // Skip the current pixel
						}

						if let Some([r, g, b, a]) = image.read(x + dx, y + dy) {
							if a == 255 {
								r_sum += r as u32;
								g_sum += g as u32;
								b_sum += b as u32;
								count += 1;
							}
						}
					}
				}

				if count > 0 {
					data[idx][0] = (r_sum / count) as u8;
					data[idx][1] = (g_sum / count) as u8;
					data[idx][2] = (b_sum / count) as u8;
					data[idx][3] = 0; // Keep the alpha channel transparent
				}
			}
			else {
				// Copy original pixel
				data[idx] = image.data[idx];
			}
		}
	}

	Image {
		width: image.width,
		height: image.height,
		data,
	}
}

impl Image<[u8; 4]> {
	/// Recovers colors for transparent pixels by averaging surrounding opaque pixels.
	#[inline]
	pub fn recover_alpha_colors(&self) -> Image<[u8; 4]> {
		recover_alpha_colors(self)
	}
}
