use std::mem;
use shade::cvmath::*;

//----------------------------------------------------------------
// The triangle's vertex

#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
struct TriangleVertex {
	position: Vec2f,
	color: [u8; 4],
}

unsafe impl shade::TVertex for TriangleVertex {
	const LAYOUT: &'static shade::VertexLayout = &shade::VertexLayout {
		size: mem::size_of::<TriangleVertex>() as u16,
		alignment: mem::align_of::<TriangleVertex>() as u16,
		attributes: &[
			shade::VertexAttribute {
				name: "aPos",
				format: shade::VertexAttributeFormat::F32v2,
				offset: dataview::offset_of!(TriangleVertex.position) as u16,
			},
			shade::VertexAttribute {
				name: "aColor",
				format: shade::VertexAttributeFormat::U8Normv4,
				offset: dataview::offset_of!(TriangleVertex.color) as u16,
			},
		],
	};
}

const FRAGMENT_SHADER: &str = r#"
#version 330 core

in vec4 v_color;

out vec4 o_fragColor;

void main()
{
	float levels = 10.0;
	vec3 qColor = floor(v_color.rgb * levels) / (levels - 1.0);
	o_fragColor = vec4(pow(qColor, 1.0 / vec3(2.2, 2.2, 2.2)), v_color.a);
}
"#;

const VERTEX_SHADER: &str = r#"
#version 330 core

in vec2 aPos;
in vec4 aColor;

out vec4 v_color;

void main()
{
	v_color = pow(aColor, vec4(2.2, 2.2, 2.2, 1.0));
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
		TriangleVertex { position: Vec2( 0.0,  0.5), color: [255, 0, 0, 255] },
		TriangleVertex { position: Vec2(-0.5, -0.5), color: [0, 255, 0, 255] },
		TriangleVertex { position: Vec2( 0.5, -0.5), color: [0, 0, 255, 255] },
	], shade::BufferUsage::Static);

	// Create the triangle shader
	let shader = g.shader_create(None, VERTEX_SHADER, FRAGMENT_SHADER);

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
				g.begin();

				// Clear the screen
				g.clear(&shade::ClearArgs {
					surface: shade::Surface::BACK_BUFFER,
					color: Some(Vec4(0.2, 0.5, 0.2, 1.0)),
					..Default::default()
				});

				// Draw the triangle
				g.draw(&shade::DrawArgs {
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
					uniforms: &[],
					vertex_start: 0,
					vertex_end: 3,
					instances: -1,
				});

				// Finish rendering
				g.end();

				// Swap buffers
				context.swap_buffers().unwrap();
			}
			_ => (),
		}
	});
}
