pub mod api;
pub mod globe;
pub mod oldtree;
pub mod pixelart;
pub mod text;
pub mod text3d;
pub mod textintro;
pub mod triangle;
pub mod zeldawater;

pub trait DemoContext {
	fn resize(&mut self, width: i32, height: i32);
	fn draw(&mut self, time: f64);

	fn mousemove(&mut self, _dx: f32, _dy: f32) {}
	fn mousedown(&mut self, _button: u32) {}
	fn mouseup(&mut self, _button: u32) {}
	fn wheel(&mut self, _delta_y: f32) {}
	fn keydown(&mut self, _key: u32) {}
}

pub type DemoHandle = Box<dyn DemoContext>;
