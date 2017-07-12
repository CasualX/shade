
use ::{Shader, Index, Primitive};
use ::d2::ColorV;
use super::Canvas;
use super::Uniforms;

pub struct Solid<'a> {
	canvas: &'a mut Canvas,
	uniforms: Uniforms,
}

impl<'a> From<&'a mut Canvas> for Solid<'a> {
	fn from(canvas: &'a mut Canvas) -> Solid<'a> {
		Solid {
			canvas,
			uniforms: Uniforms::default(),
		}
	}
}

impl<'a> Shader<'a> for Solid<'a> {
	type Vertex = ColorV;
	type Context = Uniforms;

	fn uid() -> u32 { 0x116e33ac }

	fn context(&self) -> Self::Context {
		self.uniforms
	}
	fn set_context(&mut self, ctx: &Self::Context) {
		self.uniforms = *ctx;
	}

	fn draw_primitive(&mut self, prim: Primitive, nverts: usize, nprims: usize) -> (&mut [ColorV], &mut [Index]) {
		let (vp, ip) = self.canvas.prim_begin(Self::uid(), prim, nverts, nprims);
		use ::std::slice::from_raw_parts_mut;
		let nindices = nprims * (prim as u8) as usize;
		unsafe { (from_raw_parts_mut(vp, nverts), from_raw_parts_mut(ip, nindices)) }
	}
	fn new_batch(&mut self) {
		self.canvas.new_batch();
	}
}
