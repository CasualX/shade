use super::*;

#[derive(Clone, Debug)]
pub struct FrustumInstance {
	/// The view-projection matrix defining the camera frustum to visualize.
	pub view_proj: Mat4f,
}

#[derive(Debug)]
pub struct FrustumModel {
	pub shader: ShaderProgram,
	pub vertices: VertexBuffer,
	pub vertices_len: u32,
	pub indices: IndexBuffer,
	pub indices_len: u32,
}

impl FrustumModel {
	pub fn create(g: &mut Graphics, shader: ShaderProgram, clip: Clip) -> FrustumModel {
		// There's no BaseVertex support in WebGL!
		// To keep compatibility the vertices are selected when the frustum is created
		let vertices = match clip {
			Clip::NO => &VERTICES[..8],
			Clip::ZO => &VERTICES[8..],
		};
		let vertices_len = vertices.len() as u32;
		let vertices = g.vertex_buffer(vertices, BufferUsage::Static);
		let indices_len = INDICES.len() as u32;
		let indices = g.index_buffer(&INDICES, vertices_len as u8, BufferUsage::Static);

		FrustumModel { shader, vertices, vertices_len, indices, indices_len }
	}

	pub fn draw(&self, g: &mut Graphics, camera: &camera::Camera, instance: &FrustumInstance) {
		let uniforms = ColorUniform3 {
			transform: camera.view_proj * instance.view_proj.inverse(),
			colormod: Vec4f::ONE,
		};

		g.draw_indexed(&DrawIndexedArgs {
			scissor: None,
			blend_mode: BlendMode::Solid,
			depth_test: Some(Compare::Less),
			cull_mode: None,
			mask: DrawMask::COLOR,
			prim_type: PrimType::Lines,
			shader: self.shader,
			uniforms: &[camera, &uniforms],
			vertices: &[DrawVertexBuffer {
				buffer: self.vertices,
				divisor: VertexDivisor::PerVertex,
			}],
			indices: self.indices,
			index_start: LINES_INDEX_START,
			index_end: LINES_INDEX_END,
			instances: -1,
		});

		g.draw_indexed(&DrawIndexedArgs {
			scissor: None,
			blend_mode: BlendMode::Alpha,
			depth_test: Some(Compare::Less),
			cull_mode: None,
			mask: DrawMask::COLOR,
			prim_type: PrimType::Triangles,
			shader: self.shader,
			uniforms: &[&uniforms],
			vertices: &[DrawVertexBuffer {
				buffer: self.vertices,
				divisor: VertexDivisor::PerVertex,
			}],
			indices: self.indices,
			index_start: TRIANGLES_INDEX_START,
			index_end: TRIANGLES_INDEX_END,
			instances: -1,
		});
	}
}

const NEAR: [u8; 4] = [100, 149, 237, 255];
const FAR: [u8; 4] = [255, 223, 128, 0];

static VERTICES: [ColorVertex3; 16] = [
	// Clip::NO vertices
	ColorVertex3 { pos: Vec3f(-1.0, -1.0, -1.0), color: NEAR }, // Near BL
	ColorVertex3 { pos: Vec3f(-1.0,  1.0, -1.0), color: NEAR }, // Near TL
	ColorVertex3 { pos: Vec3f( 1.0,  1.0, -1.0), color: NEAR }, // Near TR
	ColorVertex3 { pos: Vec3f( 1.0, -1.0, -1.0), color: NEAR }, // Near BR
	ColorVertex3 { pos: Vec3f(-1.0, -1.0,  1.0), color: FAR  }, // Far BL
	ColorVertex3 { pos: Vec3f(-1.0,  1.0,  1.0), color: FAR  }, // Far TL
	ColorVertex3 { pos: Vec3f( 1.0,  1.0,  1.0), color: FAR  }, // Far TR
	ColorVertex3 { pos: Vec3f( 1.0, -1.0,  1.0), color: FAR  }, // Far BR

	// Clip::ZO vertices
	ColorVertex3 { pos: Vec3f(-1.0, -1.0,  0.0), color: NEAR }, // Near BL
	ColorVertex3 { pos: Vec3f(-1.0,  1.0,  0.0), color: NEAR }, // Near TL
	ColorVertex3 { pos: Vec3f( 1.0,  1.0,  0.0), color: NEAR }, // Near TR
	ColorVertex3 { pos: Vec3f( 1.0, -1.0,  0.0), color: NEAR }, // Near BR
	ColorVertex3 { pos: Vec3f(-1.0, -1.0,  1.0), color: FAR  }, // Far BL
	ColorVertex3 { pos: Vec3f(-1.0,  1.0,  1.0), color: FAR  }, // Far TL
	ColorVertex3 { pos: Vec3f( 1.0,  1.0,  1.0), color: FAR  }, // Far TR
	ColorVertex3 { pos: Vec3f( 1.0, -1.0,  1.0), color: FAR  }, // Far BR
];

static INDICES: [u8; 24 + 36] = [
	// Line indices [0]
	0, 1, 1, 2, 2, 3, 3, 0, // Near face
	4, 5, 5, 6, 6, 7, 7, 4, // Far face
	0, 4, 1, 5, 2, 6, 3, 7, // Connecting edges

	// Triangle indices [24]
	4, 6, 5, 4, 7, 6, // Far plane
	0, 4, 5, 0, 5, 1, // Left plane
	3, 2, 6, 3, 6, 7, // Right plane
	1, 5, 6, 1, 6, 2, // Top plane
	0, 3, 7, 0, 7, 4, // Bottom plane
	0, 1, 2, 0, 2, 3, // Near plane
];

const LINES_INDEX_START: u32 = 0;
const LINES_INDEX_END: u32 = 24;
const TRIANGLES_INDEX_START: u32 = 24 + 6;
const TRIANGLES_INDEX_END: u32 = 24 + 6 + 30;
