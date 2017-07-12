extern crate shade;

use shade::soft::Solid;
use shade::d2::{IPen, Pen, Point};

fn main() {
	let cg = shade::soft::Graphics::new();

	let mut cv = cg.paint();
	{
		let mut shader = Solid::from(&mut cv);
		let pen = Pen::default();
		shader.draw_line(&pen, Point::new(-0.5, 4.0), Point::new(-3.25, 8.125));
	}

	println!("{:#?}", cv);
}
