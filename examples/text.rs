
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

	let font = {
		// Parse the font metadata
		let font: shade::msdfgen::Font = serde_json::from_str(include_str!("font/font.json")).unwrap();

		// Load the texture
		let texture = shade::image::png::load(&mut g, Some("font"), "examples/font/font.png", &shade::image::TextureProps {
			filter_min: shade::TextureFilter::Linear,
			filter_mag: shade::TextureFilter::Linear,
			wrap_u: shade::TextureWrap::ClampEdge,
			wrap_v: shade::TextureWrap::ClampEdge,
		}, None).unwrap();

		// Compile the shader
		let shader = g.shader_create(None).unwrap();
		if let Err(_) = g.shader_compile(shader, shade::gl::MTSDF_VS, shade::gl::MTSDF_FS) {
			panic!("Failed to compile shader: {}", g.shader_compile_log(shader).unwrap());
		}

		shade::d2::FontResource { font, texture, shader }
	};

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
					color: Some(cvmath::Vec4(0.4, 0.4, 0.7, 1.0)),
					depth: Some(1.0),
					..Default::default()
				}).unwrap();

				let mut cv = shade::d2::TextBuffer::new();
				cv.shader = font.shader;
				cv.blend_mode = shade::BlendMode::Alpha;
				cv.viewport = cvmath::Rect::c(0, 0, size.width as i32, size.height as i32);
				cv.push_uniform(shade::d2::TextUniform {
					transform: cvmath::Transform2::remap(cvmath::Rect::c(0.0, 0.0, size.width as f32, size.height as f32), cvmath::Rect::c(-1.0, 1.0, 1.0, -1.0)),
					texture: font.texture,
					outline_width_relative: 0.125,
					..Default::default()
				});

				let mut pos = cvmath::Vec2(0.0, 0.0);
				let mut scribe = shade::d2::Scribe {
					font_size: 64.0,
					line_height: 64.0 * 1.5,
					x_pos: pos.x,
					top_skew: 8.0,
					..Default::default()
				};
				scribe.set_baseline_relative(0.5);

				cv.text_write(&font, &mut scribe, &mut pos, "Hello, \x1b[font_size=96.0]\x1b[font_width_scale=1.5]\x1b[top_skew=0.0]world!");

				scribe.font_size = 32.0;
				scribe.line_height = 32.0;
				scribe.font_width_scale = 1.0;
				scribe.color = cvmath::Vec4(255, 255, 0, 255);
				cv.text_box(&font, &scribe, &cvmath::Rect::c(0.0, 0.0, size.width as f32, size.height as f32), shade::d2::BoxAlign::MiddleCenter, "These\nare\nmultiple\nlines.\n");

				scribe.top_skew = 8.0;
				let rainbow = "\x1b[color=#E81416]R\x1b[color=#FFA500]A\x1b[color=#FAEB36]I\x1b[color=#79C314]N\x1b[color=#487DE7]B\x1b[color=#4B369D]O\x1b[color=#70369D]W";
				let rainbow_width = scribe.text_width(&mut {cvmath::Vec2::ZERO}, &font.font, rainbow);
				let mut pos = cvmath::Vec2f((size.width as f32 - rainbow_width) * 0.5, size.height as f32 - scribe.font_size);
				cv.text_write(&font, &mut scribe, &mut pos, rainbow);

				cv.draw(&mut g, shade::Surface::BACK_BUFFER).unwrap();

				// Finish rendering
				g.end().unwrap();

				// Swap buffers
				context.swap_buffers().unwrap();
			}
			_ => (),
		}
	});
}
