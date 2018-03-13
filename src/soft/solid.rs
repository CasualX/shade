
use {Shader, Index, Primitive, ICanvas, VertexBuffer};
use d2::ColorV;
use super::Uniforms;

pub struct Solid<'a, CV: ICanvas + 'a> {
	canvas: &'a mut CV,
}

impl<'a, CV: ICanvas> From<&'a mut CV> for Solid<'a, CV> {
	fn from(canvas: &'a mut CV) -> Solid<'a, CV> {
		Solid { canvas }
	}
}

impl<'a, CV: ICanvas> Shader for Solid<'a, CV> where CV::Buffers: VertexBuffer<ColorV> {
	type Vertex = ColorV;
	type Uniform = ();

	fn uid() -> u32 { 0x116e33ac }

	fn uniforms(&self) -> () { () }
	fn set_uniforms(&mut self, _ctx: &()) {}

	fn draw_primitive(&mut self, prim: Primitive, nverts: usize, nprims: usize) -> (&mut [ColorV], &mut [Index])
	{
		self.canvas.draw_primitive::<Self>(prim, nverts, nprims)
	}
}
