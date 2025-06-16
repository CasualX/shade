use std::{fs, mem, slice, thread, time};

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
#version 330 core
out vec4 FragColor;

in vec3 Normal;
in vec2 UV;
in vec3 FragPos;

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
	vec4 texColor = texture(baseColorTexture, uv);
	if (texColor.a < 0.1) {
		discard;
	}

	// Apply quantized diffuse lighting to texture color
	vec3 finalColor = texColor.rgb * (0.4 + diff * 0.8);

	vec3 viewDir = normalize(cameraPos - FragPos);
	float rim = 1.0 - max(dot(viewDir, norm), 0.0);
	rim = smoothstep(0.5, 0.6, rim);
	finalColor += vec3(1.0, 0.8, 0.5) * rim * 0.2;  // warm rim glow

	FragColor = vec4(finalColor, texColor.a);

	// FragColor = vec4(norm * 0.5 + 0.5, 1.0);
}
"#;

const VERTEX_SHADER: &str = r#"
#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNormal;
layout (location = 2) in vec2 aUV;

out vec3 FragPos;  // <-- Pass fragment world position to fragment shader
out vec3 Normal;
out vec2 UV;

uniform mat4 model;      // Model matrix
uniform mat4 view;       // View matrix
uniform mat4 projection; // Projection matrix

void main()
{
	// Calculate world position of the vertex
	FragPos = vec3(model * vec4(aPos, 1.0));

	// Transform the normal properly (especially for scaling)
	Normal = mat3(transpose(inverse(model))) * aNormal;

	// Pass through UV
	UV = aUV;

	// Final position for rasterization
	gl_Position = projection * view * vec4(FragPos, 1.0);
}
"#;

#[derive(Copy, Clone)]
#[repr(C)]
struct Uniform {
	model: cvmath::Mat4f,
	view: cvmath::Mat4f,
	projection: cvmath::Mat4f,
	light_pos: cvmath::Vec3f,
	camera_pos: cvmath::Vec3f,
	texture: shade::Texture2D,
}

impl Default for Uniform {
	fn default() -> Self {
		Uniform {
			model: cvmath::Mat4::IDENTITY,
			view: cvmath::Mat4::IDENTITY,
			projection: cvmath::Mat4::IDENTITY,
			light_pos: cvmath::Vec3::ZERO,
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
				ty: shade::UniformType::Mat4x4 { layout: shade::MatrixLayout::RowMajor },
				offset: dataview::offset_of!(Uniform.model) as u16,
				len: 1,
			},
			shade::UniformField {
				name: "view",
				ty: shade::UniformType::Mat4x4 { layout: shade::MatrixLayout::RowMajor },
				offset: dataview::offset_of!(Uniform.view) as u16,
				len: 1,
			},
			shade::UniformField {
				name: "projection",
				ty: shade::UniformType::Mat4x4 { layout: shade::MatrixLayout::RowMajor },
				offset: dataview::offset_of!(Uniform.projection) as u16,
				len: 1,
			},
			shade::UniformField {
				name: "lightPos",
				ty: shade::UniformType::F3,
				offset: dataview::offset_of!(Uniform.light_pos) as u16,
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

struct State {
	screen_size: cvmath::Vec2<i32>,
	camera: shade::camera::ArcballCamera,
	model_size: f32,
	model_shader: shade::Shader,
	model_vertices: shade::VertexBuffer,
	model_vertices_len: u32,
	model_texture: shade::Texture2D,
	gizmo: shade::d3::gizmos::axes::Axes,
}

impl State {
	fn draw(&mut self, g: &mut shade::Graphics) {
		// Render the frame
		g.begin().unwrap();

		// Clear the screen
		g.clear(&shade::ClearArgs {
			surface: shade::Surface::BACK_BUFFER,
			color: Some(cvmath::Vec4(0.5, 0.2, 0.2, 1.0)),
			depth: Some(1.0),
			..Default::default()
		}).unwrap();

		// Update the transformation matrices
		let model = cvmath::Mat4::IDENTITY;
		let view = self.camera.view_matrix(cvmath::RH);
		let projection = cvmath::Mat4::perspective_fov(cvmath::Deg(90.0), self.screen_size.x as f32, self.screen_size.y as f32, 0.1, 40.0, (cvmath::RH, cvmath::NO));
		// let transform = projection * view * model;
		let camera_pos = self.camera.position();
		let light_pos = cvmath::Vec3(4.0, 0.0, -230.0);

		let gizmo_transform = projection * view * cvmath::Mat4::scale(self.model_size * 0.2);

		// Update the uniform buffer with the new transformation matrix
		let uniforms = Uniform { model, view, projection, light_pos, camera_pos, texture: self.model_texture };

		// Draw the model
		g.draw(&shade::DrawArgs {
			surface: shade::Surface::BACK_BUFFER,
			viewport: cvmath::Bounds2::vec(self.screen_size),
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

		g.draw_indexed(&shade::DrawIndexedArgs {
			surface: shade::Surface::BACK_BUFFER,
			viewport: cvmath::Bounds2::vec(self.screen_size),
			scissor: None,
			blend_mode: shade::BlendMode::Solid,
			depth_test: None,
			cull_mode: Some(shade::CullMode::CCW),
			mask: shade::DrawMask::ALL,
			prim_type: shade::PrimType::Lines,
			shader: self.gizmo.shader,
			vertices: &[shade::DrawVertexBuffer {
				buffer: self.gizmo.vertices,
				divisor: shade::VertexDivisor::PerVertex,
			}],
			uniforms: &[shade::UniformRef::from(&shade::d3::gizmos::axes::Uniform { transform: gizmo_transform })],
			indices: self.gizmo.indices,
			index_start: 0,
			vertex_start: 0,
			vertex_end: self.gizmo.vertices_len,
			index_end: self.gizmo.indices_len,
			instances: -1,
		}).unwrap();

		// Finish the frame
		g.end().unwrap();
	}
}

fn main() {
	let mut size = winit::dpi::PhysicalSize::new(800, 600);

	let mut event_loop = winit::event_loop::EventLoop::new();
	let window = winit::window::WindowBuilder::new()
		.with_inner_size(size);

	let window_context = glutin::ContextBuilder::new()
		.build_windowed(window, &event_loop)
		.unwrap();

	let context = unsafe { window_context.make_current().unwrap() };

	shade::gl::capi::load_with(|s| context.get_proc_address(s) as *const _);

	// Create the graphics context
	let mut g = shade::gl::GlGraphics::new();

	let (vb, vb_len, mut mins, mut maxs); {
		let vertices = fs::read("examples/oldtree/vertices.bin").unwrap();
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

	let extent = maxs - mins;
	let target = cvmath::Vec3f(0.0, 0.0, (maxs.z + mins.z) * 0.5);
	let camera_position = target + cvmath::Vec3::<f32>::X * f32::max(extent.x, extent.y);

	let texture = shade::image::png::load_file(&mut g, None, "examples/oldtree/texture.png", &shade::image::TextureProps {
		filter_min: shade::TextureFilter::Nearest,
		filter_mag: shade::TextureFilter::Nearest,
		wrap_u: shade::TextureWrap::ClampEdge,
		wrap_v: shade::TextureWrap::ClampEdge,
	}, None).unwrap();

	// Create the shader
	let shader = g.shader_create(None, VERTEX_SHADER, FRAGMENT_SHADER).unwrap();

	let gizmo_shader = g.shader_create(None, include_str!("../src/gl/shaders/gizmo.axes.vs.glsl"), include_str!("../src/gl/shaders/gizmo.axes.fs.glsl")).unwrap();
	let gizmo = shade::d3::gizmos::axes::Axes::create(&mut g, gizmo_shader);

	let mut state = State {
		screen_size: cvmath::Vec2(size.width as i32, size.height as i32),
		camera: shade::camera::ArcballCamera::new(camera_position, target, cvmath::Vec3::Z),
		model_size: extent.x.max(extent.y).max(extent.z),
		model_shader: shader,
		model_vertices: vb,
		model_vertices_len: vb_len,
		model_texture: texture,
		gizmo,
	};

	let mut left_click = false;
	let mut right_click = false;
	let mut middle_click = false;
	let mut auto_rotate = true;
	let mut cursor_position = winit::dpi::PhysicalPosition::<f64>::new(0.0, 0.0);

	// Main loop
	let mut quit = false;
	while !quit {
		// Handle events
		use winit::platform::run_return::EventLoopExtRunReturn as _;
		event_loop.run_return(|event, _, control_flow| {
			*control_flow = winit::event_loop::ControlFlow::Wait;

			// // Print only Window events to reduce noise
			// if let winit::event::Event::WindowEvent { event, .. } = &event {
			// 	println!("{:?}", event);
			// }

			match event {
				winit::event::Event::WindowEvent { event: winit::event::WindowEvent::CloseRequested, .. } => {
					quit = true;
				}
				winit::event::Event::WindowEvent { event: winit::event::WindowEvent::Resized(new_size), .. } => {
					size = new_size;
					state.screen_size.x = new_size.width as i32;
					state.screen_size.y = new_size.height as i32;
					context.resize(new_size);
				}
				winit::event::Event::WindowEvent { event: winit::event::WindowEvent::CursorMoved { position, .. }, .. } => {
					let dx = position.x as f32 - cursor_position.x as f32;
					let dy = position.y as f32 - cursor_position.y as f32;
					if left_click {
						auto_rotate = false;
						state.camera.rotate(-dx, dy);
					}
					if right_click {
						auto_rotate = false;
						state.camera.pan(dx, dy);
					}
					if middle_click {
						state.camera.zoom(dy * 0.01);
					}
					cursor_position = position;
				}
				winit::event::Event::WindowEvent { event: winit::event::WindowEvent::MouseInput { state, button: winit::event::MouseButton::Left, .. }, .. } => {
					left_click = matches!(state, winit::event::ElementState::Pressed);
				}
				winit::event::Event::WindowEvent { event: winit::event::WindowEvent::MouseInput { state, button: winit::event::MouseButton::Right, .. }, .. } => {
					right_click = matches!(state, winit::event::ElementState::Pressed);
				}
				winit::event::Event::WindowEvent { event: winit::event::WindowEvent::MouseInput { state, button: winit::event::MouseButton::Middle, .. }, .. } => {
					middle_click = matches!(state, winit::event::ElementState::Pressed);
				}
				winit::event::Event::MainEventsCleared => {
					*control_flow = winit::event_loop::ControlFlow::Exit;
				}
				_ => (),
			}
		});

		if auto_rotate {
			state.camera.rotate(1.0, 0.0);
		}

		state.draw(&mut g);

		// Swap the buffers and wait for the next frame
		context.swap_buffers().unwrap();
		thread::sleep(time::Duration::from_millis(16));
	}
}
