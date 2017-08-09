
use ::{Shader, Index, Primitive, ICanvas, VertexBuffer};
use ::d2::ColorV;
use super::Uniforms;

pub struct Solid<'a, CV: ICanvas + 'a> {
	canvas: &'a mut CV,
}

impl<'a, CV: ICanvas> From<&'a mut CV> for Solid<'a, CV> {
	fn from(canvas: &'a mut CV) -> Solid<'a, CV> {
		Solid { canvas }
	}
}

impl<'a, CV: ICanvas> Shader for Solid<'a, CV> where CV::VertexBuffers: VertexBuffer<ColorV> {
	type Vertex = ColorV;
	type Context = ();

	fn uid() -> u32 { 0x116e33ac }

	fn context(&self) -> Self::Context { () }
	fn set_context(&mut self, ctx: &Self::Context) {}

	fn draw_primitive(&mut self, prim: Primitive, nverts: usize, nprims: usize) -> (&mut [ColorV], &mut [Index])
	{
		self.canvas.draw_primitive::<Self>(prim, nverts, nprims)
	}
}
