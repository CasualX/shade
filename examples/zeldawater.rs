use std::{mem, thread, time};
use shade::cvmath::*;

#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
struct Vertex {
	position: Vec2f,
	uv: Vec2f,
}

unsafe impl shade::TVertex for Vertex {
	const LAYOUT: &'static shade::VertexLayout = &shade::VertexLayout {
		size: mem::size_of::<Vertex>() as u16,
		alignment: mem::align_of::<Vertex>() as u16,
		attributes: &[
			shade::VertexAttribute {
				name: "a_pos",
				format: shade::VertexAttributeFormat::F32v2,
				offset: dataview::offset_of!(Vertex.position) as u16,
			},
			shade::VertexAttribute {
				name: "a_uv",
				format: shade::VertexAttributeFormat::F32v2,
				offset: dataview::offset_of!(Vertex.uv) as u16,
			},
		],
	};
}

const FRAGMENT_SHADER: &str = r#"
#version 330 core
out vec4 FragColor;

in vec2 v_uv;

uniform sampler2D u_texture;
uniform sampler2D u_distortion;
uniform float u_time;

void main()
{
	vec2 uv = v_uv;

	// Layered distortion for organic feel
	float distortion1 = texture(u_distortion, fract(uv * 1.0 + vec2(u_time * 0.2, u_time * 0.25))).r;
	float distortion2 = texture(u_distortion, fract(uv * 2.5 + vec2(-u_time * 0.15, u_time * 0.1))).r;
	float distortion3 = texture(u_distortion, fract(uv * 0.75 + vec2(u_time * 0.05, -u_time * 0.2))).r;

	float distortion = (distortion1 * 0.5 + distortion2 * 0.3 + distortion3 * 0.2) - 0.5;

	uv += vec2(distortion * 0.1, distortion * 0.15);

	// Main wave layer
	float v = texture(u_texture, uv * vec2(4.0, 4.0)).r;
	vec3 mainColor = mix(vec3(0.0, 0.0, 0.5), vec3(0.5, 0.8, 1.0), v);

	// Second darker shadow layer
	float u = texture(u_texture, uv * vec2(2.0, 2.0) + vec2(0.3, 0.3)).r;
	vec3 shadowColor = mix(vec3(0.0, 0.0, 0.0), vec3(0.1, 0.2, 0.3), u);

	// Composite: shadow appears under water
	vec3 finalColor = mainColor - shadowColor * 0.5;

	FragColor = vec4(finalColor, 1.0);
}
"#;

const VERTEX_SHADER: &str = r#"
#version 330 core
in vec2 a_pos;
in vec2 a_uv;

out vec2 v_uv;

void main()
{
	v_uv = a_uv;
	gl_Position = vec4(a_pos, 0.0, 1.0);
}
"#;

#[derive(Clone, Debug)]
struct Uniform {
	time: f32,
	texture: shade::Texture2D,
	distortion: shade::Texture2D,
}

impl shade::UniformVisitor for Uniform {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_time", &self.time);
		set.value("u_texture", &self.texture);
		set.value("u_distortion", &self.distortion);
	}
}

//----------------------------------------------------------------

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

	// Create the full screen quad vertex buffer
	let vb = g.vertex_buffer(None, &[
		Vertex { position: Vec2f(-1.0, -1.0), uv: Vec2f(0.0, 0.0) },
		Vertex { position: Vec2f(1.0, -1.0), uv: Vec2f(1.0, 0.0) },
		Vertex { position: Vec2f(-1.0, 1.0), uv: Vec2f(0.0, 1.0) },
		Vertex { position: Vec2f(1.0, 1.0), uv: Vec2f(1.0, 1.0) },
	], shade::BufferUsage::Static).unwrap();

	let ib = g.index_buffer(None, &[
		0u16, 1, 2,
		1, 3, 2,
	], 4, shade::BufferUsage::Static).unwrap();

	// Load the texture
	let texture = shade::image::png::load_file(&mut g, None, "examples/zeldawater/water.png", &shade::image::TextureProps {
		filter_min: shade::TextureFilter::Linear,
		filter_mag: shade::TextureFilter::Linear,
		wrap_u: shade::TextureWrap::Repeat,
		wrap_v: shade::TextureWrap::Repeat,
	}, None).unwrap();

	let distortion = shade::image::png::load_file(&mut g, None, "examples/zeldawater/distort.png", &shade::image::TextureProps {
		filter_min: shade::TextureFilter::Linear,
		filter_mag: shade::TextureFilter::Linear,
		wrap_u: shade::TextureWrap::Repeat,
		wrap_v: shade::TextureWrap::Repeat,
	}, None).unwrap();

	// Create the water shader
	let shader = g.shader_create(None, VERTEX_SHADER, FRAGMENT_SHADER).unwrap();

	let start_time = time::Instant::now();

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
					// state.screen_size.x = new_size.width as i32;
					// state.screen_size.y = new_size.height as i32;
					context.resize(new_size);
				}
				winit::event::Event::MainEventsCleared => {
					*control_flow = winit::event_loop::ControlFlow::Exit;
				}
				_ => (),
			}
		});

		// Render the frame
		g.begin().unwrap();

		// Clear the screen
		g.clear(&shade::ClearArgs {
			surface: shade::Surface::BACK_BUFFER,
			color: Some(Vec4(0.2, 0.5, 0.2, 1.0)),
			..Default::default()
		}).unwrap();

		let time = start_time.elapsed().as_secs_f32();

		let uniform = Uniform { time, texture, distortion };

		// Draw the quad
		g.draw_indexed(&shade::DrawIndexedArgs {
			surface: shade::Surface::BACK_BUFFER,
			viewport: Bounds2::c(0, 0, size.width as i32, size.height as i32),
			scissor: None,
			blend_mode: shade::BlendMode::Solid,
			depth_test: None,
			cull_mode: None,
			mask: shade::DrawMask::COLOR,
			prim_type: shade::PrimType::Triangles,
			shader,
			vertices: &[shade::DrawVertexBuffer {
				buffer: vb,
				divisor: shade::VertexDivisor::PerVertex,
			}],
			indices: ib,
			index_start: 0,
			index_end: 6,
			uniforms: &[&uniform],
			instances: -1,
		}).unwrap();

		// Finish rendering
		g.end().unwrap();

		// Swap buffers
		context.swap_buffers().unwrap();
		thread::sleep(time::Duration::from_millis(16));
	}
}
