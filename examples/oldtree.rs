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
			shade::VertexAttribute::with::<cvmath::Vec3f>("a_pos", dataview::offset_of!(Vertex.position)),
			shade::VertexAttribute::with::<cvmath::Vec3f>("a_normal", dataview::offset_of!(Vertex.normal)),
			shade::VertexAttribute::with::<cvmath::Vec2f>("a_uv", dataview::offset_of!(Vertex.uv)),
		],
	};
}

const FRAGMENT_SHADER: &str = r#"\
#version 330 core

out vec4 o_fragColor;

in vec3 v_normal;
in vec2 v_uv;
in vec3 v_fragPos;

uniform sampler2D u_diffuse;
uniform vec3 u_position;
uniform vec3 u_lightPos;

void main() {
	// Define light direction (normalized)
	vec3 lightDir = normalize(u_lightPos - v_fragPos);

	// Calculate diffuse lighting
	vec3 norm = normalize(v_normal);
	float diff = max(dot(norm, lightDir), 0.0);

	// Sample texture and discard transparent fragments
	vec2 uv = vec2(v_uv.x, 1.0 - v_uv.y);
	vec4 texColor = texture(u_diffuse, uv);
	if (texColor.a < 0.1) {
		discard;
	}

	// Apply quantized diffuse lighting to texture color
	vec3 finalColor = texColor.rgb * (0.4 + diff * 0.8);

	vec3 viewDir = normalize(u_position - v_fragPos);
	float rim = 1.0 - max(dot(viewDir, norm), 0.0);
	rim = smoothstep(0.5, 0.6, rim);
	finalColor += vec3(1.0, 0.8, 0.5) * rim * 0.2;  // warm rim glow

	o_fragColor = vec4(finalColor, texColor.a);

	// o_fragColor = vec4(norm * 0.5 + 0.5, 1.0);
}
"#;

const VERTEX_SHADER: &str = r#"\
#version 330 core

in vec3 a_pos;
in vec3 a_normal;
in vec2 a_uv;

out vec3 v_fragPos;
out vec3 v_normal;
out vec2 v_uv;

uniform mat4x3 u_model;

uniform mat4 u_view_proj;

void main()
{
	// Calculate world position of the vertex
	v_fragPos = vec3(u_model * vec4(a_pos, 1.0));

	// Transform the normal properly (especially for scaling)
	v_normal = transpose(inverse(mat3(u_model))) * a_normal;

	// Pass through UV
	v_uv = a_uv;

	// Final position for rasterization
	gl_Position = u_view_proj * vec4(v_fragPos, 1.0);
}
"#;


const PARALLAX_SHADER: &str = r#"
#version 330 core
out vec4 o_fragColor;

in vec2 v_uv;
in vec3 v_normal;
in vec3 v_fragPos;

uniform sampler2D u_diffuse;
uniform sampler2D u_normalMap;
uniform sampler2D u_heightMap;
uniform vec3 u_position;
uniform vec3 u_lightPos;

uniform float u_heightScale;

// Construct TBN matrix using derivatives (quad assumed)
mat3 computeTBN(vec3 normal, vec3 pos, vec2 uv) {
	vec3 dp1 = dFdx(pos);
	vec3 dp2 = dFdy(pos);
	vec2 duv1 = dFdx(uv);
	vec2 duv2 = dFdy(uv);

	vec3 t = normalize(dp1 * duv2.y - dp2 * duv1.y);
	vec3 b = normalize(cross(normal, t));
	return mat3(t, -b, normal);
}

// Parallax Occlusion Mapping
vec2 parallaxOcclusionMap(vec2 uv, vec3 viewDirTangent) {
	const int numLayers = 32;
	const float minLayers = 8.0;
	const float maxLayers = 32.0;

	float angle = dot(vec3(0.0, 0.0, 1.0), viewDirTangent);
	float num = mix(maxLayers, minLayers, abs(angle));
	float layerDepth = 1.0 / num;
	vec2 P = viewDirTangent.xy * u_heightScale;
	vec2 deltaUV = -P / num;

	vec2 currUV = uv;
	float currDepth = 0.0;
	float heightFromMap = texture(u_heightMap, currUV).r;

	// Step until depth of map is below current layer
	while (currDepth < heightFromMap && num > 0.0) {
		currUV += deltaUV;
		currDepth += layerDepth;
		heightFromMap = texture(u_heightMap, currUV).r;
	}

	return currUV;
}

void main() {
	// Compute TBN matrix
	mat3 TBN = computeTBN(normalize(v_normal), v_fragPos, v_uv);
	vec3 viewDir = normalize(u_position - v_fragPos);
	vec3 viewDirTangent = TBN * viewDir;

	// Perform Parallax Occlusion Mapping
	vec2 displacedUV = parallaxOcclusionMap(v_uv, viewDirTangent);

	// Optional: Clamp UVs to avoid artifacts at the edges
	// if (displacedUV.x < 0.0 || displacedUV.x > 1.0 || displacedUV.y < 0.0 || displacedUV.y > 1.0)
	// 	discard;

	// Sample diffuse texture
	vec4 texColor = texture(u_diffuse, displacedUV);
	if (texColor.a < 0.1)
		discard;

	// Sample and decode the normal map (assumed in [0,1] range)
	vec3 normalTangent = texture(u_normalMap, displacedUV).rgb * 2.0 - 1.0;

	// Transform to world space
	vec3 perturbedNormal = normalize(TBN * normalTangent);

	// Lighting
	vec3 lightDir = normalize(u_lightPos - v_fragPos);
	float diffLight = max(dot(perturbedNormal, lightDir), 0.0);

	// Final color
	vec3 finalColor = texColor.rgb * (0.6 + diffLight * 0.4);
	o_fragColor = vec4(finalColor, texColor.a);
}
"#;

//----------------------------------------------------------------

struct OldTreeInstance {
	model: cvmath::Transform3f,
	light_pos: cvmath::Vec3f,
}

impl shade::UniformVisitor for OldTreeInstance {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_model", &self.model);
		set.value("u_lightPos", &self.light_pos);
	}
}

struct OldTreeModel {
	shader: shade::Shader,
	vertices: shade::VertexBuffer,
	vertices_len: u32,
	texture: shade::Texture2D,
	bounds: cvmath::Bounds3<f32>,
}

impl shade::UniformVisitor for OldTreeModel {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_diffuse", &self.texture);
	}
}

impl OldTreeModel {
	fn create(g: &mut shade::Graphics) -> OldTreeModel {
		let vertices = fs::read("examples/oldtree/vertices.bin").unwrap();
		let vertices = unsafe { slice::from_raw_parts(vertices.as_ptr() as *const Vertex, vertices.len() / mem::size_of::<Vertex>()) };
		let vertices_len = vertices.len() as u32;

		let mut mins = cvmath::Vec3::dup(f32::INFINITY);
		let mut maxs = cvmath::Vec3::dup(f32::NEG_INFINITY);
		for v in vertices {
			mins = mins.min(v.position);
			maxs = maxs.max(v.position);
			// println!("Vertex {}: {:?}", i, v);
		}
		let bounds = cvmath::Bounds3(mins, maxs);

		let vertices = g.vertex_buffer(None, &vertices, shade::BufferUsage::Static).unwrap();

		let texture = shade::image::png::load_file(g, None, "examples/oldtree/texture.png", &shade::image::TextureProps {
			filter_min: shade::TextureFilter::Nearest,
			filter_mag: shade::TextureFilter::Nearest,
			wrap_u: shade::TextureWrap::ClampEdge,
			wrap_v: shade::TextureWrap::ClampEdge,
		}, None).unwrap();

		let shader = g.shader_create(None, VERTEX_SHADER, FRAGMENT_SHADER).unwrap();

		OldTreeModel { shader, vertices, vertices_len, texture, bounds }
	}
	fn draw(&self, g: &mut shade::Graphics, camera: &shade::d3::CameraSetup, instance: &OldTreeInstance) {
		g.draw(&shade::DrawArgs {
			surface: camera.surface,
			viewport: camera.viewport,
			scissor: None,
			blend_mode: shade::BlendMode::Solid,
			depth_test: Some(shade::DepthTest::Less),
			cull_mode: Some(shade::CullMode::CW),
			mask: shade::DrawMask {
				red: true,
				green: true,
				blue: false,
				alpha: true,
				depth: true,
				stencil: 0,
			},
			prim_type: shade::PrimType::Triangles,
			shader: self.shader,
			vertices: &[shade::DrawVertexBuffer {
				buffer: self.vertices,
				divisor: shade::VertexDivisor::PerVertex,
			}],
			uniforms: &[camera, self, instance],
			vertex_start: 0,
			vertex_end: self.vertices_len,
			instances: -1,
		}).unwrap();

	}
}

//----------------------------------------------------------------

struct ParallaxInstance {
	model: cvmath::Transform3f,
	light_pos: cvmath::Vec3f,
}

impl shade::UniformVisitor for ParallaxInstance {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_model", &self.model);
		set.value("u_lightPos", &self.light_pos);
	}
}

struct ParallaxModel {
	shader: shade::Shader,
	diffuse: shade::Texture2D,
	normal_map: shade::Texture2D,
	height_map: shade::Texture2D,
	height_scale: f32,
}

impl shade::UniformVisitor for ParallaxModel {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_diffuse", &self.diffuse);
		set.value("u_normalMap", &self.normal_map);
		set.value("u_heightMap", &self.height_map);
		set.value("u_heightScale", &self.height_scale);
	}
}

impl ParallaxModel {
	fn create(g: &mut shade::Graphics) -> ParallaxModel {
		let diffuse = shade::image::png::load_file(g, None, "examples/textures/stonefloor-512.diffuse.png", &shade::image::TextureProps {
			filter_min: shade::TextureFilter::Linear,
			filter_mag: shade::TextureFilter::Linear,
			wrap_u: shade::TextureWrap::Repeat,
			wrap_v: shade::TextureWrap::Repeat,
		}, None).unwrap();

		let normal_map = shade::image::png::load_file(g, None, "examples/textures/stonefloor-512.normal.png", &shade::image::TextureProps {
			filter_min: shade::TextureFilter::Linear,
			filter_mag: shade::TextureFilter::Linear,
			wrap_u: shade::TextureWrap::Repeat,
			wrap_v: shade::TextureWrap::Repeat,
		}, None).unwrap();

		let height_map = shade::image::png::load_file(g, None, "examples/textures/stonefloor-512.height.png", &shade::image::TextureProps {
			filter_min: shade::TextureFilter::Linear,
			filter_mag: shade::TextureFilter::Linear,
			wrap_u: shade::TextureWrap::Repeat,
			wrap_v: shade::TextureWrap::Repeat,
		}, None).unwrap();

		let shader = g.shader_create(None, VERTEX_SHADER, PARALLAX_SHADER).unwrap();
		ParallaxModel { shader, diffuse, normal_map, height_map, height_scale: 0.04 }
	}
	fn draw(&self, g: &mut shade::Graphics, camera: &shade::d3::CameraSetup, instance: &ParallaxInstance) {
		let vertices = [
			Vertex { position: cvmath::Vec3f(-5.0, -5.0, 0.0), normal: cvmath::Vec3f(0.0, 0.0, 1.0), uv: cvmath::Vec2f(0.0, 2.0) },
			Vertex { position: cvmath::Vec3f(5.0, -5.0, 0.0), normal: cvmath::Vec3f(0.0, 0.0, 1.0), uv: cvmath::Vec2f(2.0, 2.0) },
			Vertex { position: cvmath::Vec3f(5.0, 5.0, 0.0), normal: cvmath::Vec3f(0.0, 0.0, 1.0), uv: cvmath::Vec2f(2.0, 0.0) },
			Vertex { position: cvmath::Vec3f(-5.0, 5.0, 0.0), normal: cvmath::Vec3f(0.0, 0.0, 1.0), uv: cvmath::Vec2f(0.0, 0.0) },
		];
		let indices = [0, 1, 2, 0, 2, 3];
		let vertices = indices.map(|i| vertices[i]);
		let vb = g.vertex_buffer(None, &vertices, shade::BufferUsage::Static).unwrap();
		g.draw(&shade::DrawArgs {
			surface: camera.surface,
			viewport: camera.viewport,
			scissor: None,
			blend_mode: shade::BlendMode::Solid,
			depth_test: Some(shade::DepthTest::Less),
			cull_mode: Some(shade::CullMode::CW),
			mask: shade::DrawMask::ALL,
			prim_type: shade::PrimType::Triangles,
			shader: self.shader,
			vertices: &[shade::DrawVertexBuffer {
				buffer: vb,
				divisor: shade::VertexDivisor::PerVertex,
			}],
			uniforms: &[camera, self, instance],
			vertex_start: 0,
			vertex_end: vertices.len() as u32,
			instances: -1,
		}).unwrap();
		g.vertex_buffer_free(vb, shade::FreeMode::Delete).unwrap();
	}
}

//----------------------------------------------------------------

#[allow(dead_code)]
enum ProjectionType {
	Perspective,
	Orthographic,
}
impl ProjectionType {
	fn toggle(&mut self) {
		*self = match *self {
			ProjectionType::Perspective => ProjectionType::Orthographic,
			ProjectionType::Orthographic => ProjectionType::Perspective,
		};
	}
}

struct Scene {
	screen_size: cvmath::Vec2<i32>,
	projection_type: ProjectionType,
	camera: shade::d3::ArcballCamera,
	tree: OldTreeModel,
	floor: ParallaxModel,
	axes: shade::d3::axes::AxesModel,
	frustum: shade::d3::frustum::FrustumModel,
	view: cvmath::Transform3f,
}

impl Scene {
	fn create(g: &mut shade::Graphics, screen_size: cvmath::Vec2<i32>) -> Scene {
		let tree = OldTreeModel::create(g);

		let floor = ParallaxModel::create(g);

		let (axes, frustum) = {
			let shader = g.shader_create(None, shade::gl::shaders::COLOR3D_VS, shade::gl::shaders::COLOR3D_FS).unwrap();
			(shade::d3::axes::AxesModel::create(g, shader), shade::d3::frustum::FrustumModel::create(g, shader))
		};

		let camera = {
			let pivot = tree.bounds.center().set_x(0.0).set_y(0.0);
			let position = pivot + cvmath::Vec3::<f32>::X * tree.bounds.size().xy().vmax();

			shade::d3::ArcballCamera::new(position, pivot, cvmath::Vec3::Z)
		};

		let view = camera.view_matrix(cvmath::Hand::RH);

		let projection_type = ProjectionType::Perspective;

		Scene { screen_size, projection_type, camera, tree, floor, axes, frustum, view }
	}
	fn draw(&mut self, g: &mut shade::Graphics, time: f32) {
		// Render the frame
		g.begin().unwrap();

		// Clear the screen
		g.clear(&shade::ClearArgs {
			surface: shade::Surface::BACK_BUFFER,
			color: Some(cvmath::Vec4(0.5, 0.2, 0.2, 1.0)),
			depth: Some(1.0),
			..Default::default()
		}).unwrap();

		let frustum_view_proj;

		// Camera setup
		let camera = {
			let surface = shade::Surface::BACK_BUFFER;
			let viewport = cvmath::Bounds2::vec(self.screen_size);
			let aspect_ratio = self.screen_size.x as f32 / self.screen_size.y as f32;
			let position = self.camera.position();
			let hand = cvmath::Hand::RH;
			let view = self.camera.view_matrix(hand);
			let clip = cvmath::Clip::NO;
			let (near, far) = (0.1, 40.0);
			let projection = match self.projection_type {
				ProjectionType::Perspective => cvmath::Mat4::perspective_fov(cvmath::Deg(90.0), self.screen_size.x as f32, self.screen_size.y as f32, near, far, (hand, clip)),
				ProjectionType::Orthographic => cvmath::Mat4::ortho(-5.0 * aspect_ratio, 5.0 * aspect_ratio, -5.0, 5.0, near, far, (hand, clip)),
			};
			let view_proj = projection * view;
			frustum_view_proj = projection * self.view;
			let inv_view_proj = view_proj.inverse();
			shade::d3::CameraSetup { surface, viewport, aspect_ratio, position, near, far, view, projection, view_proj, inv_view_proj, clip }
		};

		let radius = 5000.0;
		let angle = time * 2.0; // Adjust speed here
		let light_pos = cvmath::Vec3f::new(
			radius * angle.cos(),
			radius * angle.sin(),
			40.0, // Fixed elevation, adjust if needed
		);

		// Draw the models
		self.tree.draw(g, &camera, &OldTreeInstance {
			model: cvmath::Transform3f::IDENTITY,
			light_pos,
		});

		self.floor.draw(g, &camera, &ParallaxInstance {
			model: cvmath::Transform3f::IDENTITY,
			light_pos,
		});

		self.axes.draw(g, &camera, &shade::d3::axes::AxesInstance {
			local: cvmath::Transform3f::scale(camera.position.len() * 0.2),
			depth_test: None,
		});

		self.frustum.draw(g, &camera, &shade::d3::frustum::FrustumInstance {
			view_proj: frustum_view_proj,
			clip: camera.clip,
		});

		// Finish the frame
		g.end().unwrap();
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
	let ref mut g = shade::gl::GlGraphics::new();

	let mut scene = Scene::create(g, cvmath::Vec2(size.width as i32, size.height as i32));

	let mut left_click = false;
	let mut right_click = false;
	let mut middle_click = false;
	let mut auto_rotate = true;
	let mut cursor_position = winit::dpi::PhysicalPosition::<f64>::new(0.0, 0.0);

	let epoch = time::Instant::now();

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
					scene.screen_size.x = new_size.width as i32;
					scene.screen_size.y = new_size.height as i32;
					context.resize(new_size);
				}
				winit::event::Event::WindowEvent { event: winit::event::WindowEvent::CursorMoved { position, .. }, .. } => {
					let dx = position.x as f32 - cursor_position.x as f32;
					let dy = position.y as f32 - cursor_position.y as f32;
					if left_click {
						auto_rotate = false;
						scene.camera.rotate(-dx, -dy);
					}
					if right_click {
						auto_rotate = false;
						scene.camera.pan(-dx, dy);
					}
					if middle_click {
						scene.camera.zoom(dy * 0.01);
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
				winit::event::Event::WindowEvent { event: winit::event::WindowEvent::KeyboardInput { input, .. }, .. } => {
					if let Some(key) = input.virtual_keycode {
						match key {
							winit::event::VirtualKeyCode::P if input.state == winit::event::ElementState::Pressed => scene.projection_type.toggle(),
							_ => (),
						}
					}
				}
				winit::event::Event::MainEventsCleared => {
					*control_flow = winit::event_loop::ControlFlow::Exit;
				}
				_ => (),
			}
		});

		if auto_rotate {
			scene.camera.rotate(-1.0, 0.0);
		}

		let time = epoch.elapsed().as_secs_f32();
		scene.draw(g, time);

		// Swap the buffers and wait for the next frame
		context.swap_buffers().unwrap();
		thread::sleep(time::Duration::from_millis(16));
	}
}
