pub mod atlas;
pub mod conway;
pub mod dither;
pub mod globe;
pub mod gui_zoo;
pub mod mandelbrot;
pub mod oldtree;
pub mod panels;
pub mod pixelart;
pub mod polygon;
pub mod scene;
pub mod screenmelt;
pub mod shadertoy;
pub mod text;
pub mod text3d;
pub mod textintro;
pub mod textmatrix;
pub mod triangle;
pub mod zeldawater;

use crate::{AssetLoader, DemoInterface};

pub type DemoCreateFn = fn(&mut dyn shade::IGraphics, &dyn AssetLoader) -> Box<dyn DemoInterface>;

pub struct DemoEntry {
	pub id: &'static str,
	pub create: DemoCreateFn,
}

const DEMOS: &[DemoEntry] = &[
	DemoEntry { id: "triangle", create: triangle::create },
	DemoEntry { id: "text", create: text::create },
	DemoEntry { id: "zeldawater", create: zeldawater::create },
	DemoEntry { id: "pixelart", create: pixelart::create },
	DemoEntry { id: "conway", create: conway::create },
	DemoEntry { id: "dither", create: dither::create },
	DemoEntry { id: "atlas", create: atlas::create },
	DemoEntry { id: "globe", create: globe::create },
	DemoEntry { id: "gui_zoo", create: gui_zoo::create },
	DemoEntry { id: "mandelbrot", create: mandelbrot::create },
	DemoEntry { id: "oldtree", create: oldtree::create },
	DemoEntry { id: "panels", create: panels::create },
	DemoEntry { id: "polygon", create: polygon::create },
	DemoEntry { id: "scene", create: scene::create },
	DemoEntry { id: "screenmelt", create: screenmelt::create },
	DemoEntry { id: "shadertoy", create: shadertoy::create },
	DemoEntry { id: "text3d", create: text3d::create },
	DemoEntry { id: "textintro", create: textintro::create },
	DemoEntry { id: "textmatrix", create: textmatrix::create },
];

pub fn all() -> &'static [DemoEntry] {
	DEMOS
}

pub fn names() -> impl Iterator<Item = &'static str> {
	DEMOS.iter().map(|demo| demo.id)
}

pub fn names_csv() -> String {
	names().collect::<Vec<_>>().join(", ")
}

pub fn find(id: &str) -> Option<&'static DemoEntry> {
	DEMOS.iter().find(|demo| demo.id == id)
}
