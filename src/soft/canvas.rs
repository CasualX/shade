/*!
Render queue.
*/

use d2::ColorV;
use {ICanvas, Index, PlaceV, Primitive};
use super::Uniforms;

#[derive(Debug)]
struct Batch {
	prim: Primitive,
	numvert: Index,
	numprim: Index,
	shader_uid: u32,
}

#[derive(Debug)]
pub struct Canvas {
	vbuf: Vec<ColorV>,
	ibuf: Vec<Index>,
	batches: Vec<Batch>,
	uniforms: Vec<Uniforms>,
}

impl Canvas {
	pub fn new() -> Canvas {
		Canvas {
			vbuf: Vec::new(),
			ibuf: Vec::new(),
			batches: Vec::new(),
			uniforms: Vec::new(),
		}
	}
	pub fn prim_begin(&mut self, shader_uid: u32, prim: Primitive, nverts: usize, nprims: usize) -> (*mut ColorV, *mut Index) {
		let new_batch = {
			if let Some(last_batch) = self.batches.last_mut() {
				// If the last batch has not been specialized, claim it
				if last_batch.shader_uid == 0 {
					last_batch.prim = prim;
					last_batch.shader_uid = shader_uid;
					false
				}
				// If last batch has a different primitive or shader_uid, new batch
				else if last_batch.prim != prim || last_batch.shader_uid != shader_uid {
					true
				}
				// Appending to the last batch
				else {
					false
				}
			}
			// Create the first batch
			else {
				true
			}
		};
		if new_batch {
			self.batches.push(Batch { prim, numvert: 0, numprim: 0, shader_uid });
		}

		// Update the counters
		let last_batch = self.batches.last_mut().unwrap();
		last_batch.numvert += nverts as Index;
		last_batch.numprim += nprims as Index;

		// Allocate vertices
		let vstart = self.vbuf.len();
		if self.vbuf.capacity() < vstart + nverts {
			let reserve = vstart + nverts - self.vbuf.capacity();
			self.vbuf.reserve(reserve);
		}
		let vert_ptr = unsafe {
			self.vbuf.set_len(vstart + nverts);
			self.vbuf.as_mut_ptr().offset(vstart as isize)
		};

		// Allocate indices
		let istart = self.ibuf.len();
		let nindices = nprims * prim as u8 as usize;
		if self.ibuf.capacity() < istart + nindices {
			let reserve = istart + nindices - self.ibuf.capacity();
			self.ibuf.reserve(reserve);
		}
		let indices_ptr = unsafe {
			self.ibuf.set_len(istart + nindices);
			self.ibuf.as_mut_ptr().offset(istart as isize)
		};

		// Set indices to base of vertices
		unsafe {
			for i in 0..nindices {
				*indices_ptr.offset(i as isize) = vstart as Index;
			}
		}

		(vert_ptr, indices_ptr)
	}
	pub fn new_batch(&mut self) {
		let new_batch = {
			// Only create a new batch if necessary
			if let Some(last_batch) = self.batches.last() {
				last_batch.shader_uid != 0
			}
			else {
				true
			}
		};
		if new_batch {
			// Write a temp new batch with dummy values
			// Important is shader_uid is zero, this indicates it's an unspecialized batch
			self.batches.push(Batch {
				prim: Primitive::Points,
				numvert: 0,
				numprim: 0,
				shader_uid: 0,
			});
		}
	}
	pub fn push_uniforms(&mut self, uniforms: &Uniforms) {
		// Fixme! Check if uniforms have changed!
		self.new_batch();
		self.uniforms.push(*uniforms);
	}
}

impl ICanvas for Canvas {
	unsafe fn draw_primitive(&mut self, shader_uid: u32, prim: Primitive, nverts: usize, nprims: usize) -> (*mut PlaceV, *mut Index) {
		unimplemented!()
	}
	unsafe fn new_batch(&mut self) {
		unimplemented!()
	}
}
