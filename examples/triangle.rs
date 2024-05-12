
//----------------------------------------------------------------
// The triangle's vertex

#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
struct TriangleVertex {
	position: cvmath::Vec2<f32>,
	color: [u8; 4],
}

unsafe impl shade::TVertex for TriangleVertex {
	const VERTEX_LAYOUT: &'static shade::VertexLayout = &shade::VertexLayout {
		size: std::mem::size_of::<TriangleVertex>() as u16,
		alignment: std::mem::align_of::<TriangleVertex>() as u16,
		attributes: &[
			shade::VertexAttribute {
				format: shade::VertexAttributeFormat::F32,
				len: 2,
				offset: dataview::offset_of!(TriangleVertex.position) as u16,
			},
			shade::VertexAttribute {
				format: shade::VertexAttributeFormat::U8Norm,
				len: 4,
				offset: dataview::offset_of!(TriangleVertex.color) as u16,
			},
		],
	};
}

//----------------------------------------------------------------
// The triangle's uniforms and shaders
// Here the shader has no uniforms, so we use an empty struct

#[derive(Copy, Clone, dataview::Pod)]
#[repr(C)]
struct TriangleUniforms {}

impl Default for TriangleUniforms {
	fn default() -> Self {
		TriangleUniforms {}
	}
}

unsafe impl shade::TUniform for TriangleUniforms {
	const UNIFORM_LAYOUT: &'static shade::UniformLayout = &shade::UniformLayout {
		size: std::mem::size_of::<TriangleUniforms>() as u16,
		alignment: std::mem::align_of::<TriangleUniforms>() as u16,
		attributes: &[],
	};
}

const FRAGMENT_SHADER: &str = r#"
#version 330 core
out vec4 FragColor;

in vec4 VertexColor;

void main()
{
	FragColor = pow(VertexColor, 1.0 / vec4(2.2, 2.2, 2.2, 1.0));
}
"#;

const VERTEX_SHADER: &str = r#"
#version 330 core
layout (location = 0) in vec2 aPos;
layout (location = 1) in vec4 aColor;

out vec4 VertexColor;

void main()
{
	VertexColor = pow(aColor, vec4(2.2, 2.2, 2.2, 1.0));
	gl_Position = vec4(aPos, 0.0, 1.0);
}
"#;

//----------------------------------------------------------------

fn main() {
	let mut size = winit::dpi::PhysicalSize::new(800, 600);

	let event_loop = winit::event_loop::EventLoop::new();
	let window = winit::window::WindowBuilder::new()
		.with_inner_size(size);

	let window_context = glutin::ContextBuilder::new()
		.build_windowed(window, &event_loop)
		.unwrap();

	let context = unsafe { window_context.make_current().unwrap() };

	shade::gl::capi::load_with(|s| context.get_proc_address(s) as *const _);

	// Create the graphics context
	let mut g = shade::gl::GlGraphics::new();

	// Create the triangle vertex buffer
	let vb = g.vertex_buffer(None, &[
		TriangleVertex { position: cvmath::Vec2( 0.0,  0.5), color: [255, 0, 0, 255] },
		TriangleVertex { position: cvmath::Vec2(-0.5, -0.5), color: [0, 255, 0, 255] },
		TriangleVertex { position: cvmath::Vec2( 0.5, -0.5), color: [0, 0, 255, 255] },
	], shade::BufferUsage::Static).unwrap();

	// Create the triangle uniform buffer
	let ub = g.uniform_buffer(None, &[
		TriangleUniforms::default(),
	]).unwrap();

	// Create the triangle shader
	let shader = g.shader_create(None).unwrap();
	if let Err(_) = g.shader_compile(shader, VERTEX_SHADER, FRAGMENT_SHADER) {
		panic!("Failed to compile shader: {}", g.shader_compile_log(shader).unwrap());
	}

	// Main loop
	event_loop.run(move |event, _, control_flow| {
		match event {
			winit::event::Event::WindowEvent { event: winit::event::WindowEvent::CloseRequested, .. } => {
				*control_flow = winit::event_loop::ControlFlow::Exit
			}
			winit::event::Event::WindowEvent { event: winit::event::WindowEvent::Resized(new_size), .. } => {
				size = new_size;
				context.resize(new_size);
			}
			winit::event::Event::RedrawRequested(_) => {
				// Render the frame
				g.begin().unwrap();

				// Clear the screen
				g.clear(&shade::ClearArgs {
					surface: shade::Surface::BACK_BUFFER,
					color: Some(cvmath::Vec4(0.2, 0.5, 0.2, 1.0)),
					..Default::default()
				}).unwrap();

				// Draw the triangle
				g.draw(&shade::DrawArgs {
					surface: shade::Surface::BACK_BUFFER,
					viewport: cvmath::Rect::c(0, 0, size.width as i32, size.height as i32),
					scissor: None,
					blend_mode: shade::BlendMode::Solid,
					depth_test: None,
					cull_mode: None,
					prim_type: shade::PrimType::Triangles,
					shader,
					vertices: vb,
					uniforms: ub,
					vertex_start: 0,
					vertex_end: 3,
					uniform_index: 0,
					instances: -1,
				}).unwrap();

				// Finish rendering
				g.end().unwrap();

				// Swap buffers
				context.swap_buffers().unwrap();
			}
			_ => (),
		}
	});
}
