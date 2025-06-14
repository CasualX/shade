use std::{io, mem, slice};

mod api;

#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
struct Vertex {
	position: cvmath::Vec3f,
	normal: cvmath::Vec3f,
	uv: cvmath::Vec2f,
}

unsafe impl shade::TVertex for Vertex {
	const LAYOUT: &'static shade::VertexLayout = &shade::VertexLayout {
		size: mem::size_of::<Vertex>() as u16,
		alignment: mem::align_of::<Vertex>() as u16,
		attributes: &[
			shade::VertexAttribute::with::<cvmath::Vec3f>("aPos", dataview::offset_of!(Vertex.position)),
			shade::VertexAttribute::with::<cvmath::Vec3f>("aNormal", dataview::offset_of!(Vertex.normal)),
			shade::VertexAttribute::with::<cvmath::Vec2f>("aUV", dataview::offset_of!(Vertex.uv)),
		],
	};
}

const FRAGMENT_SHADER: &str = r#"
precision mediump float;

varying vec3 Normal;
varying vec2 UV;
varying vec3 FragPos;

uniform sampler2D baseColorTexture;
uniform vec3 cameraPos;

void main() {
	// Define light direction (normalized)
	vec3 lightDir = normalize(vec3(1.0, -1.0, 1.0));

	// Calculate diffuse lighting
	vec3 norm = normalize(Normal);
	float diff = max(dot(norm, lightDir), 0.0);

	// Quantize diffuse into 3 steps for toon shading
	if (diff > 0.66) diff = 1.0;
	else if (diff > 0.33) diff = 0.66;
	else diff = 0.33;

	// Sample texture and discard transparent fragments
	vec2 uv = vec2(UV.x, 1.0 - UV.y);
	vec4 texColor = texture2D(baseColorTexture, uv);
	if (texColor.a < 0.1) {
		discard;
	}

	// Apply quantized diffuse lighting to texture color
	vec3 finalColor = texColor.rgb * (0.4 + diff * 0.8);

	vec3 viewDir = normalize(cameraPos - FragPos);
	float rim = 1.0 - max(dot(viewDir, norm), 0.0);
	rim = smoothstep(0.5, 0.6, rim);
	finalColor += vec3(1.0, 0.8, 0.5) * rim * 0.2;  // warm rim glow

	gl_FragColor = vec4(finalColor, texColor.a);

	// gl_FragColor = vec4(norm * 0.5 + 0.5, 1.0);
}
"#;

const VERTEX_SHADER: &str = r#"
precision mediump float;

attribute vec3 aPos;
attribute vec3 aNormal;
attribute vec2 aUV;

varying vec3 FragPos;
varying vec3 Normal;
varying vec2 UV;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

uniform mat3 normalMatrix;

void main()
{
	// Calculate world position of the vertex
	FragPos = vec3(model * vec4(aPos, 1.0));

	// Transform the normal properly (especially for scaling)
	Normal = normalMatrix * aNormal;

	// Pass through UV
	UV = aUV;

	// Final position for rasterization
	gl_Position = projection * view * vec4(FragPos, 1.0);
}
"#;

#[derive(Copy, Clone, dataview::Pod)]
#[repr(C)]
struct Uniform {
	model: cvmath::Mat4f,
	view: cvmath::Mat4f,
	projection: cvmath::Mat4f,
	normal_matrix: cvmath::Mat3f, // Normal matrix for transforming normals
	light_pos: cvmath::Vec3f,
	view_pos: cvmath::Vec3f,
	camera_pos: cvmath::Vec3f,
	texture: shade::Texture2D,
}

impl Default for Uniform {
	fn default() -> Self {
		Uniform {
			model: cvmath::Mat4::IDENTITY,
			view: cvmath::Mat4::IDENTITY,
			projection: cvmath::Mat4::IDENTITY,
			normal_matrix: cvmath::Mat3::IDENTITY,
			light_pos: cvmath::Vec3::ZERO,
			view_pos: cvmath::Vec3::ZERO,
			camera_pos: cvmath::Vec3::ZERO,
			texture: shade::Texture2D::INVALID,
		}
	}
}

unsafe impl shade::TUniform for Uniform {
	const LAYOUT: &'static shade::UniformLayout = &shade::UniformLayout {
		size: mem::size_of::<Uniform>() as u16,
		alignment: mem::align_of::<Uniform>() as u16,
		fields: &[
			shade::UniformField {
				name: "model",
				ty: shade::UniformType::Mat4x4 { order: shade::MatrixLayout::RowMajor },
				offset: dataview::offset_of!(Uniform.model) as u16,
				len: 1,
			},
			shade::UniformField {
				name: "view",
				ty: shade::UniformType::Mat4x4 { order: shade::MatrixLayout::RowMajor },
				offset: dataview::offset_of!(Uniform.view) as u16,
				len: 1,
			},
			shade::UniformField {
				name: "projection",
				ty: shade::UniformType::Mat4x4 { order: shade::MatrixLayout::RowMajor },
				offset: dataview::offset_of!(Uniform.projection) as u16,
				len: 1,
			},
			shade::UniformField {
				name: "normalMatrix",
				ty: shade::UniformType::Mat3x3 { order: shade::MatrixLayout::RowMajor },
				offset: dataview::offset_of!(Uniform.normal_matrix) as u16,
				len: 1,
			},
			shade::UniformField {
				name: "lightPos",
				ty: shade::UniformType::F3,
				offset: dataview::offset_of!(Uniform.light_pos) as u16,
				len: 1,
			},
			shade::UniformField {
				name: "viewPos",
				ty: shade::UniformType::F3,
				offset: dataview::offset_of!(Uniform.view_pos) as u16,
				len: 1,
			},
			shade::UniformField {
				name: "cameraPos",
				ty: shade::UniformType::F3,
				offset: dataview::offset_of!(Uniform.camera_pos) as u16,
				len: 1,
			},
			shade::UniformField {
				name: "baseColorTexture",
				ty: shade::UniformType::Sampler2D,
				offset: dataview::offset_of!(Uniform.texture) as u16,
				len: 1,
			},
		],
	};
}

//----------------------------------------------------------------

pub struct Context {
	webgl: shade::webgl::WebGLGraphics,
	screen_size: cvmath::Vec2<i32>,
	model_bounds: cvmath::Bounds3<f32>,
	model_shader: shade::Shader,
	model_vertices: shade::VertexBuffer,
	model_vertices_len: u32,
	model_texture: shade::Texture2D,
}

impl Context {
	pub fn new() -> Context {
		api::setup_panic_hook();

		let mut webgl = shade::webgl::WebGLGraphics::new();

		let mut g = shade::Graphics(&mut webgl);

		let (vb, vb_len, mut mins, mut maxs); {
			let vertices = include_bytes!("../../../oldtree/vertices.bin");
			let vertices = unsafe { slice::from_raw_parts(vertices.as_ptr() as *const Vertex, vertices.len() / mem::size_of::<Vertex>()) };
			vb = g.vertex_buffer(None, &vertices, shade::BufferUsage::Static).unwrap();
			vb_len = vertices.len() as u32;
			mins = cvmath::Vec3::dup(f32::INFINITY);
			maxs = cvmath::Vec3::dup(f32::NEG_INFINITY);
			for v in vertices {
				mins = mins.min(v.position);
				maxs = maxs.max(v.position);
				// println!("Vertex {}: {:?}", i, v);
			}
		}

		let texture = include_bytes!("../../../oldtree/texture.png");

		let texture = shade::image::png::load(&mut g, None, &mut io::Cursor::new(texture), &shade::image::TextureProps {
			filter_min: shade::TextureFilter::Nearest,
			filter_mag: shade::TextureFilter::Nearest,
			wrap_u: shade::TextureWrap::ClampEdge,
			wrap_v: shade::TextureWrap::ClampEdge,
		}, None).unwrap();

		// Create the shader
		let shader = g.shader_create(None, VERTEX_SHADER, FRAGMENT_SHADER).unwrap();

		Context {
			webgl,
			screen_size: cvmath::Vec2::ZERO,
			model_bounds: cvmath::Bounds(mins, maxs),
			model_shader: shader,
			model_vertices: vb,
			model_vertices_len: vb_len,
			model_texture: texture,
		}
	}

	pub fn resize(&mut self, width: i32, height: i32) {
		self.screen_size = cvmath::Vec2(width, height);
	}

	pub fn draw(&mut self, time: f64) {
		let g = shade::Graphics(&mut self.webgl);

		// Render the frame
		g.begin().unwrap();

		// Clear the screen
		g.clear(&shade::ClearArgs {
			surface: shade::Surface::BACK_BUFFER,
			color: Some(cvmath::Vec4(0.5, 0.2, 0.2, 1.0)),
			depth: Some(1.0),
			..Default::default()
		}).unwrap();

		// Rotate the model
		let model_origin = (self.model_bounds.mins + self.model_bounds.maxs) * 0.5;
		let model_transform =
			cvmath::Mat4::rotate(cvmath::Deg(-90.0), cvmath::Vec3::X) *
			cvmath::Mat4::translate(-model_origin) *
			cvmath::Mat4::rotate(cvmath::Rad(time as f32), cvmath::Vec3::Z);

		// Update the transformation matrices
		let projection = cvmath::Mat4::perspective_fov(cvmath::Deg(45.0), self.screen_size.x as f32, self.screen_size.y as f32, 0.1, 40.0, (cvmath::RH, cvmath::NO));
		let camera_pos = cvmath::Vec3(0.0, 2.0, -10.0);
		let view = cvmath::Mat4::look_at(camera_pos, model_origin, cvmath::Vec3(0.0, 1.0, 0.0), cvmath::RH);
		// let transform = projection * view * model;
		let light_pos = cvmath::Vec3(4.0, 0.0, -230.0);
		let view_pos = cvmath::Vec3(-10.0, 0.0, -10.0);
		let normal_matrix = model_transform.mat3().inverse().transpose();

		// Update the uniforms
		let uniforms = Uniform { model: model_transform, view, projection, normal_matrix, light_pos, view_pos, camera_pos, texture: self.model_texture };

		// Draw the model
		g.draw(&shade::DrawArgs {
			surface: shade::Surface::BACK_BUFFER,
			viewport: cvmath::Bounds2::c(0, 0, self.screen_size.x, self.screen_size.y),
			scissor: None,
			blend_mode: shade::BlendMode::Solid,
			depth_test: Some(shade::DepthTest::Less),
			cull_mode: Some(shade::CullMode::CCW),
			mask: shade::DrawMask {
				red: true,
				green: true,
				blue: false,
				alpha: true,
				depth: true,
				stencil: 0,
			},
			prim_type: shade::PrimType::Triangles,
			shader: self.model_shader,
			vertices: &[shade::DrawVertexBuffer {
				buffer: self.model_vertices,
				divisor: shade::VertexDivisor::PerVertex,
			}],
			uniforms: &[shade::UniformRef::from(&uniforms)],
			vertex_start: 0,
			vertex_end: self.model_vertices_len,
			instances: -1,
		}).unwrap();

		// Finish the frame
		g.end().unwrap();
	}
}
