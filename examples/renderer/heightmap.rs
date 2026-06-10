use cvmath::*;

struct Material {
	shader: shade::ShaderProgram,
}

struct Instance {
	model_matrix: Transform3f,
}

pub struct Renderable {
	mesh: shade::d3::VertexIndexedMesh,
	material: Material,
	instance: Instance,
}

impl Renderable {
	pub fn create(g: &mut shade::Graphics, shader: shade::ShaderProgram) -> Renderable {
		let heightmap = shade::image::DecodedImage::load_file("assets/textures/D1.png").unwrap();
		let heightmap = heightmap.to_rgb().map_colors(|[r, _g, _b]| r);
		let heightmap_palette =
			shade::image::ImageRGB::load_file("assets/textures/C1W.png").unwrap();

		let step = 1;
		let width = heightmap.width / step;
		let height = heightmap.height / step;

		let mut vertices = Vec::new();
		for y in 0..height {
			for x in 0..width {
				let v = heightmap.read(x * step, y * step).unwrap();
				let [red, green, blue] = heightmap_palette.read(x * step, y * step).unwrap();
				// let h = if v < 80 { (v as f32 / 80.0 - 1.0) * 10.0 } else { (v - 80) as f32 };
				let h = v as f32;
				vertices.push(shade::d3::ColorVertex3 {
					pos: Vec3f((x * step) as f32, (y * step) as f32, h),
					color: [red, green, blue, 255],
				});
			}
		}

		let mut indices = Vec::new();
		for y in 0..(height - 1) {
			for x in 0..(width - 1) {
				let i0 = (y * width + x) as u32;
				let i1 = (y * width + (x + 1)) as u32;
				let i2 = ((y + 1) * width + x) as u32;
				let i3 = ((y + 1) * width + (x + 1)) as u32;
				let h00 = heightmap.read(x * step, y * step).unwrap() as f32;
				let h10 = heightmap.read((x + 1) * step, y * step).unwrap() as f32;
				let h01 = heightmap.read(x * step, (y + 1) * step).unwrap() as f32;
				let h11 = heightmap.read((x + 1) * step, (y + 1) * step).unwrap() as f32;
				let d0 = (h00 - h11).abs();
				let d1 = (h10 - h01).abs();
				if d0 <= d1 {
					indices.push(i0);
					indices.push(i2);
					indices.push(i3);
					indices.push(i0);
					indices.push(i3);
					indices.push(i1);
				} else {
					indices.push(i0);
					indices.push(i2);
					indices.push(i1);
					indices.push(i1);
					indices.push(i2);
					indices.push(i3);
				}
			}
		}

		let bounds = shade::d3::compute_bounds(&vertices);

		let mesh = shade::d3::VertexIndexedMesh {
			origin: Vec3f::ZERO,
			bounds,
			vertices: g.vertex_buffer(&vertices, shade::BufferUsage::Static),
			vertices_len: vertices.len() as u32,
			indices: g.index_buffer(&indices, vertices.len() as u32, shade::BufferUsage::Static),
			indices_len: indices.len() as u32,
		};

		let material = Material { shader };

		let instance = Instance {
			model_matrix: Transform3f::translation(Vec3f(-50.0, 50.0, 0.0))
				* Transform3f::scaling(Vec3f(100.0 / 1024.0, 100.0 / 1024.0, 0.05)),
		};

		Renderable {
			mesh,
			material,
			instance,
		}
	}
}

impl crate::IRenderable for Renderable {
	fn update(&mut self, _globals: &crate::Globals) {}

	fn draw(
		&self,
		g: &mut shade::Graphics,
		globals: &crate::Globals,
		camera: &shade::d3::Camera,
		light: &crate::Light,
		_shadow: bool,
	) {
		let uniforms = shade::d3::ColorUniform3 {
			transform: camera.view_proj * self.instance.model_matrix,
			colormod: Vec4f::dup(1.0),
		};
		g.draw_indexed(&shade::DrawIndexedArgs {
			scissor: None,
			blend_mode: shade::BlendMode::Solid,
			depth_test: Some(shade::Compare::Less),
			cull_mode: Some(shade::CullMode::CCW),
			mask: shade::DrawMask::COLOR | shade::DrawMask::DEPTH,
			prim_type: shade::PrimType::Triangles,
			shader: self.material.shader,
			uniforms: &[globals, camera, light, &uniforms],
			vertices: &[shade::DrawVertexBuffer {
				buffer: self.mesh.vertices,
				divisor: shade::VertexDivisor::PerVertex,
			}],
			indices: self.mesh.indices,
			index_start: 0,
			index_end: self.mesh.indices_len,
			instances: -1,
		});
	}

	fn get_bounds(&self) -> (Bounds3f, Transform3f) {
		(self.mesh.bounds, self.instance.model_matrix)
	}
}
