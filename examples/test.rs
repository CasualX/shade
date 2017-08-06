extern crate shade;

use shade::soft::Solid;
use shade::d2::{IPen, Pen, Point2};

fn main() {
	let cg = shade::soft::Graphics::new();

	let mut cv = cg.paint();
	{
		let mut shader = Solid::from(&mut cv);
		let pen = Pen::default();
		shader.draw_line(&pen, Point2(-0.5, 4.0), Point2(-3.25, 8.125));
	}

	println!("{:#?}", cv);
}
