
use ::graphics::IGraphics;
use super::Surface;
use super::Canvas;

pub struct Graphics {
	backbuf: Surface,
}

impl Graphics {
	pub fn new() -> Graphics {
		Graphics {
			backbuf: Surface::new(128, 128),
		}
	}
	pub fn paint(&self) -> Canvas {
		Canvas::new()
	}
	pub fn render(&self, cv: Canvas) {
		unimplemented!()
	}
}

impl IGraphics for Graphics {
	fn begin(&mut self) {}
	fn end(&mut self) {}
}
