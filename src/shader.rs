/*!
Shader Objects.
*/

use {Primitive, IVertex, Index, UniformData};

#[macro_export]
macro_rules! draw_primitive {
	(@count) => { 0 };
	(@count $e:expr $(, $tail:expr)*) => { 1 + draw_primitive!(@count $($tail),*) };

	($shade:expr; $prim:expr; $($index:expr),*; $($vert:expr),*,) => {
		draw_primitive!($shade; $prim; $($index),*; $($vert),*);
	};
	($shade:expr; $prim:expr; $($index:expr),*; $($vert:expr),*) => {
		const N_VERTS: usize = draw_primitive!(@count $($vert),*);
		const N_INDICES: usize = draw_primitive!(@count $($index),*);
		assert_eq!(0, N_INDICES % $prim as u8 as usize);
		let (_vp, _ip) = $shade.draw_primitive($prim, N_VERTS, N_INDICES / $prim as u8 as usize);
		debug_assert_eq!(_vp.len(), N_VERTS);
		debug_assert_eq!(_ip.len(), N_INDICES);
		let _ip = _ip.as_mut_ptr();
		let _vp = _vp.as_mut_ptr();
		let _i = -1isize;
		$(
			let _i = _i + 1;
			unsafe { *_ip.offset(_i) += $index; }
		)*
		let _v = -1isize;
		$(
			let _v = _v + 1;
			unsafe { *_vp.offset(_v) = $vert; }
		)*
	};
}

pub trait Shader {
	type Vertex: IVertex;
	type Uniform: UniformData;

	fn uid() -> u32;

	fn uniforms(&self) -> Self::Uniform;
	fn set_uniforms(&mut self, ctx: &Self::Uniform);

	fn draw_primitive(&mut self, prim: Primitive, nverts: usize, nprims: usize) -> (&mut [Self::Vertex], &mut [Index]);
}
