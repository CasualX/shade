use super::*;

const TEST_GIF: &[u8] = include_bytes!("9panel.gif");

#[test]
fn load_animated_gif_composites_subframes_to_canvas() {
	let gif = AnimatedImage::load_memory_gif(TEST_GIF).unwrap();
	assert_eq!(gif.width, 128);
	assert_eq!(gif.height, 128);
	assert!(!gif.frames.is_empty());
	for frame in &gif.frames {
		assert_eq!(frame.width, 128);
		assert_eq!(frame.height, 128);
		assert_eq!(frame.data.len(), 128 * 128);
	}
}

#[test]
fn load_single_gif_returns_full_canvas_image() {
	let animated = AnimatedImage::load_memory_gif(TEST_GIF).unwrap();
	let frame = DecodedImage::load_memory_gif(TEST_GIF).unwrap().to_rgba();
	assert_eq!(frame.width, 128);
	assert_eq!(frame.height, 128);
	assert_eq!(frame.data.len(), 128 * 128);
	let last = animated.frames.last().unwrap();
	assert_eq!(frame.width, last.width);
	assert_eq!(frame.height, last.height);
	assert_eq!(frame.data, last.data);
}

fn gif_with_background_disposal() -> Vec<u8> {
	let mut bytes = Vec::new();
	let palette = [255, 255, 255, 255, 0, 0];
	{
		let mut encoder = gif::Encoder::new(&mut bytes, 2, 2, &palette).unwrap();
		let mut frame = gif::Frame::default();
		frame.width = 2;
		frame.height = 2;
		frame.dispose = gif::DisposalMethod::Background;
		frame.buffer = std::borrow::Cow::Borrowed(&[1, 1, 1, 1]);
		encoder.write_frame(&frame).unwrap();

		let mut frame = gif::Frame::default();
		frame.width = 1;
		frame.height = 1;
		frame.buffer = std::borrow::Cow::Borrowed(&[1]);
		encoder.write_frame(&frame).unwrap();
	}
	bytes
}

#[test]
fn load_animated_gif_uses_declared_background_color_for_disposal() {
	let gif = AnimatedImage::load_memory_gif(&gif_with_background_disposal()).unwrap();
	assert_eq!(gif.frames.len(), 2);

	let frame = &gif.frames[1];
	assert_eq!(frame.data[0], [255, 0, 0, 255]);
	assert_eq!(frame.data[1], [255, 255, 255, 255]);
	assert_eq!(frame.data[2], [255, 255, 255, 255]);
	assert_eq!(frame.data[3], [255, 255, 255, 255]);
}
