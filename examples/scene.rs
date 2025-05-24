use std::{thread, time};

//----------------------------------------------------------------
// Vertex

#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
struct MyVertex3 {
	position: cvmath::Vec3<f32>,
	tex_coord: cvmath::Vec2<f32>,
	color: [u8; 4],
}

unsafe impl shade::TVertex for MyVertex3 {
	const VERTEX_LAYOUT: &'static shade::VertexLayout = &shade::VertexLayout {
		size: std::mem::size_of::<MyVertex3>() as u16,
		alignment: std::mem::align_of::<MyVertex3>() as u16,
		attributes: &[
			shade::VertexAttribute {
				format: shade::VertexAttributeFormat::F32,
				len: 3,
				offset: dataview::offset_of!(MyVertex3.position) as u16,
			},
			shade::VertexAttribute {
				format: shade::VertexAttributeFormat::F32,
				len: 2,
				offset: dataview::offset_of!(MyVertex3.tex_coord) as u16,
			},
			shade::VertexAttribute {
				format: shade::VertexAttributeFormat::U8Norm,
				len: 4,
				offset: dataview::offset_of!(MyVertex3.color) as u16,
			},
		],
	};
}

//----------------------------------------------------------------
// Shader and uniforms

const FRAGMENT_SHADER: &str = r#"
#version 330 core
out vec4 FragColor;

in vec4 VertexColor;
in vec2 TexCoord;

uniform sampler2D tex;
uniform vec2 texSize;

void main() {
	vec4 color = texture(tex, TexCoord / texSize) * VertexColor;
	if (color.a < 0.5) {
		discard;
	}
	color.a = 1.0;
	FragColor = color;
}
"#;

const VERTEX_SHADER: &str = r#"
#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec2 aTexCoord;
layout (location = 2) in vec4 aColor;

out vec4 VertexColor;
out vec2 TexCoord;

uniform mat4x4 transform;

void main()
{
	VertexColor = aColor;
	TexCoord = aTexCoord;
	gl_Position = transform * vec4(aPos, 1.0);
}
"#;

#[derive(Copy, Clone, dataview::Pod)]
#[repr(C)]
struct MyUniform3 {
	transform: cvmath::Mat4<f32>,
	texture: shade::Texture2D,
	texture_size: [f32; 2],
}

impl Default for MyUniform3 {
	fn default() -> Self {
		MyUniform3 {
			transform: cvmath::Mat4::IDENTITY,
			texture: shade::Texture2D::INVALID,
			texture_size: [1.0, 1.0],
		}
	}
}

unsafe impl shade::TUniform for MyUniform3 {
	const UNIFORM_LAYOUT: &'static shade::UniformLayout = &shade::UniformLayout {
		size: std::mem::size_of::<MyUniform3>() as u16,
		alignment: std::mem::align_of::<MyUniform3>() as u16,
		attributes: &[
			shade::UniformAttribute {
				name: "transform",
				ty: shade::UniformType::Mat4x4 { order: shade::UniformMatOrder::RowMajor },
				offset: dataview::offset_of!(MyUniform3.transform) as u16,
				len: 1,
			},
			shade::UniformAttribute {
				name: "tex",
				ty: shade::UniformType::Sampler2D(0),
				offset: dataview::offset_of!(MyUniform3.texture) as u16,
				len: 1,
			},
			shade::UniformAttribute {
				name: "texSize",
				ty: shade::UniformType::F2,
				offset: dataview::offset_of!(MyUniform3.texture_size) as u16,
				len: 1,
			},
		],
	};
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

	// Load the texture
	let texture = shade::image::png::load(&mut g, Some("scene tiles"), "examples/textures/scene tiles.png", &shade::image::TextureProps {
		filter_min: shade::TextureFilter::Nearest,
		filter_mag: shade::TextureFilter::Nearest,
		wrap_u: shade::TextureWrap::ClampEdge,
		wrap_v: shade::TextureWrap::ClampEdge,
	}, None).unwrap();
	let tex_info = g.texture2d_get_info(texture).unwrap();
	let texture_size = [tex_info.width as f32, tex_info.height as f32];

	// Create the shader
	let shader = g.shader_create(None).unwrap();
	if let Err(_) = g.shader_compile(shader, VERTEX_SHADER, FRAGMENT_SHADER) {
		panic!("Failed to compile shader: {}", g.shader_compile_log(shader).unwrap());
	}

	let time_base = time::Instant::now();

	// Main loop
	let mut quit = false;
	while !quit {
		// Handle events
		use winit::platform::run_return::EventLoopExtRunReturn as _;
		event_loop.run_return(|event, _, control_flow| {
			*control_flow = winit::event_loop::ControlFlow::Wait;

			if let winit::event::Event::WindowEvent { event, .. } = &event {
				// Print only Window events to reduce noise
				println!("{:?}", event);
			}

			match event {
				winit::event::Event::WindowEvent { event: winit::event::WindowEvent::CloseRequested, .. } => {
					quit = true;
				}
				winit::event::Event::WindowEvent { event: winit::event::WindowEvent::Resized(new_size), .. } => {
					size = new_size;
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
			color: Some(cvmath::Vec4(0.2, 0.2, 0.5, 1.0)),
			depth: Some(1.0),
			..Default::default()
		}).unwrap();

		let curtime = time::Instant::now().duration_since(time_base).as_secs_f32();

		// Update the camera
		let projection = cvmath::Mat4::perspective_fov(cvmath::Deg(45.0), size.width as f32, size.height as f32, 0.1, 1000.0, (cvmath::RH, cvmath::NO));
		let view = {
			let eye = cvmath::Vec3(32.0 + (curtime * 2.0).sin() * 32.0, 100.0 + (curtime * 1.5).sin() * 32.0, -100.0) * 1.5;
			let target = cvmath::Vec3(96.0 * 0.5, 0.0, 32.0);
			let up = cvmath::Vec3(0.0, 1.0, 0.0);
			cvmath::Mat4::look_at(eye, target, up, cvmath::RH)
		};
		let transform = projection * view;

		let mut cv = shade::d2::CommandBuffer::<MyVertex3, MyUniform3>::new();
		cv.shader = shader;
		cv.blend_mode = shade::BlendMode::Alpha;
		cv.viewport = cvmath::Rect::c(0, 0, size.width as i32, size.height as i32);
		cv.depth_test = Some(shade::DepthTest::Less);
		cv.push_uniform(MyUniform3 { transform, texture, texture_size });
		floor_tile(&mut cv, 0, 0, &GRASS);
		floor_tile(&mut cv, 1, 0, &GRASS);
		floor_tile(&mut cv, 2, 0, &GRASS);
		floor_tile(&mut cv, 0, 1, &GRASS);
		floor_tile(&mut cv, 1, 1, &GRASS);
		floor_tile(&mut cv, 2, 1, &GRASS);
		floor_thing(&mut cv, 0, 0, &DROP);
		floor_thing(&mut cv, 1, 1, &DROP);
		floor_thing(&mut cv, 2, 0, &DROP);
		floor_thing(&mut cv, 1, 0, &BEAR);

		cv.draw(&mut g, shade::Surface::BACK_BUFFER).unwrap();

		// Finish the frame
		g.end().unwrap();

		// Swap the buffers and wait for the next frame
		context.swap_buffers().unwrap();
		thread::sleep(time::Duration::from_millis(16));
	}
}

struct Sprite {
	left: f32,
	up: f32,
	right: f32,
	down: f32,
}

const GRASS: Sprite = Sprite { left: 0.0 * 34.0 + 1.0, up: 1.0 * 34.0 + 1.0, right: 0.0 * 34.0 + 32.0, down: 1.0 * 34.0 + 32.0 };
const DROP: Sprite = Sprite { left: 1.0 * 34.0 + 1.0, up: 1.0 * 34.0 + 1.0, right: 1.0 * 34.0 + 32.0, down: 1.0 * 34.0 + 32.0 };
const BEAR: Sprite = Sprite { left: 3.0, up: 68.0, right: 49.0, down: 152.0 };

fn floor_tile(cv: &mut shade::d2::CommandBuffer<MyVertex3, MyUniform3>, x: i32, y: i32, sprite: &Sprite) {
	let mut cv = cv.begin(shade::PrimType::Triangles, 4, 2);
	cv.add_indices_quad();
	cv.add_vertex(MyVertex3 {
		position: cvmath::Vec3(x as f32 * 32.0, 0.0, y as f32 * 32.0),
		tex_coord: cvmath::Vec2(sprite.left, sprite.down),
		color: [255, 255, 255, 255],
	});
	cv.add_vertex(MyVertex3 {
		position: cvmath::Vec3(x as f32 * 32.0, 0.0, (y + 1) as f32 * 32.0),
		tex_coord: cvmath::Vec2(sprite.left, sprite.up),
		color: [255, 255, 255, 255],
	});
	cv.add_vertex(MyVertex3 {
		position: cvmath::Vec3((x + 1) as f32 * 32.0, 0.0, (y + 1) as f32 * 32.0),
		tex_coord: cvmath::Vec2(sprite.right, sprite.up),
		color: [255, 255, 255, 255],
	});
	cv.add_vertex(MyVertex3 {
		position: cvmath::Vec3((x + 1) as f32 * 32.0, 0.0, y as f32 * 32.0),
		tex_coord: cvmath::Vec2(sprite.right, sprite.down),
		color: [255, 255, 255, 255],
	});
}

fn floor_thing(cv: &mut shade::d2::CommandBuffer<MyVertex3, MyUniform3>, x: i32, y: i32, sprite: &Sprite) {
	let mut cv = cv.begin(shade::PrimType::Triangles, 4, 2);
	let yoffs = -7.0;
	let zoffs1 = 12.0;
	let zoffs2 = 24.0;
	let width = sprite.right - sprite.left;
	let height = sprite.down - sprite.up;
	cv.add_indices_quad();
	cv.add_vertex(MyVertex3 {
		position: cvmath::Vec3(x as f32 * 32.0, 0.0 + yoffs, y as f32 * 32.0 + zoffs1),
		tex_coord: cvmath::Vec2(sprite.left, sprite.down),
		color: [255, 255, 255, 255],
	});
	cv.add_vertex(MyVertex3 {
		position: cvmath::Vec3(x as f32 * 32.0, height + yoffs, y as f32 * 32.0 + zoffs2),
		tex_coord: cvmath::Vec2(sprite.left, sprite.up),
		color: [255, 255, 255, 255],
	});
	cv.add_vertex(MyVertex3 {
		position: cvmath::Vec3(x as f32 * 32.0 + width, height + yoffs, y as f32 * 32.0 + zoffs2),
		tex_coord: cvmath::Vec2(sprite.right, sprite.up),
		color: [255, 255, 255, 255],
	});
	cv.add_vertex(MyVertex3 {
		position: cvmath::Vec3(x as f32 * 32.0 + width, 0.0 + yoffs, y as f32 * 32.0 + zoffs1),
		tex_coord: cvmath::Vec2(sprite.right, sprite.down),
		color: [255, 255, 255, 255],
	});
}
