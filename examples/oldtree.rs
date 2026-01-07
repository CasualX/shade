use std::time;
use std::ffi::CString;
use std::num::NonZeroU32;

use glutin::prelude::*;
use shade::cvmath::*;

const FRAGMENT_SHADER: &str = r#"\
#version 330 core

out vec4 o_fragColor;

in vec3 v_normal;
in vec2 v_uv;
in vec3 v_fragPos;

uniform sampler2D u_diffuse;
uniform vec3 u_cameraPosition;
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

	vec3 viewDir = normalize(u_cameraPosition - v_fragPos);
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

uniform mat4 u_viewProjMatrix;

void main()
{
	// Calculate world position of the vertex
	v_fragPos = vec3(u_model * vec4(a_pos, 1.0));

	// Transform the normal properly (especially for scaling)
	v_normal = transpose(inverse(mat3(u_model))) * a_normal;

	// Pass through UV
	v_uv = a_uv;

	// Final position for rasterization
	gl_Position = u_viewProjMatrix * vec4(v_fragPos, 1.0);
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
uniform vec3 u_cameraPosition;
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
	vec3 viewDir = normalize(u_cameraPosition - v_fragPos);
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

struct OldTreeMaterial {
	shader: shade::Shader,
	texture: shade::Texture2D,
}
impl shade::UniformVisitor for OldTreeMaterial {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_diffuse", &self.texture);
	}
}

struct OldTreeInstance {
	model: Transform3f,
}
impl shade::UniformVisitor for OldTreeInstance {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_model", &self.model);
	}
}

struct OldTreeRenderable {
	mesh: shade::d3::VertexMesh,
	material: OldTreeMaterial,
	instance: OldTreeInstance,
}
impl OldTreeRenderable {
	fn create(g: &mut shade::Graphics) -> OldTreeRenderable {
		shade::include_bin!(VERTICES: [shade::d3::TexturedVertexN] = "oldtree/vertices.bin");
		let mesh = shade::d3::VertexMesh::new(g, None, Vec3f::ZERO, &VERTICES, shade::BufferUsage::Static);

		let shader = g.shader_create(None, VERTEX_SHADER, FRAGMENT_SHADER);
		let texture = {
			let image = shade::image::DecodedImage::load_file_png("examples/oldtree/texture.png").unwrap();
			g.image(None, &image)
		};
		let material = OldTreeMaterial { shader, texture };

		let instance = OldTreeInstance {
			model: Transform3f::IDENTITY,
		};

		OldTreeRenderable { mesh, material, instance }
	}
	fn draw(&self, g: &mut shade::Graphics, camera: &shade::d3::Camera, light: &Light) {
		g.draw(&shade::DrawArgs {
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
			shader: self.material.shader,
			vertices: &[shade::DrawVertexBuffer {
				buffer: self.mesh.vertices,
				divisor: shade::VertexDivisor::PerVertex,
			}],
			uniforms: &[camera, &self.material, &self.instance, light],
			vertex_start: 0,
			vertex_end: self.mesh.vertices_len,
			instances: -1,
		});
	}
}

//----------------------------------------------------------------

struct ParallaxMaterial {
	shader: shade::Shader,
	diffuse: shade::Texture2D,
	normal_map: shade::Texture2D,
	height_map: shade::Texture2D,
	height_scale: f32,
}
impl shade::UniformVisitor for ParallaxMaterial {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_diffuse", &self.diffuse);
		set.value("u_normalMap", &self.normal_map);
		set.value("u_heightMap", &self.height_map);
		set.value("u_heightScale", &self.height_scale);
	}
}

struct ParallaxInstance {
	model: Transform3f,
}

impl shade::UniformVisitor for ParallaxInstance {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_model", &self.model);
	}
}

struct ParallaxRenderable {
	mesh: shade::d3::VertexMesh,
	material: ParallaxMaterial,
	instance: ParallaxInstance,
}

impl ParallaxRenderable {
	fn create(g: &mut shade::Graphics) -> ParallaxRenderable {
		let vertices = [
			shade::d3::TexturedVertexN { pos: Vec3f(-5.0, -5.0, 0.0), normal: Vec3f(0.0, 0.0, 1.0), uv: Vec2f(0.0, 2.0) },
			shade::d3::TexturedVertexN { pos: Vec3f(5.0, -5.0, 0.0), normal: Vec3f(0.0, 0.0, 1.0), uv: Vec2f(2.0, 2.0) },
			shade::d3::TexturedVertexN { pos: Vec3f(5.0, 5.0, 0.0), normal: Vec3f(0.0, 0.0, 1.0), uv: Vec2f(2.0, 0.0) },
			shade::d3::TexturedVertexN { pos: Vec3f(-5.0, 5.0, 0.0), normal: Vec3f(0.0, 0.0, 1.0), uv: Vec2f(0.0, 0.0) },
		];
		let indices = [0, 1, 2, 0, 2, 3];
		let vertices = indices.map(|i| vertices[i]);

		let mesh = shade::d3::VertexMesh::new(g, None, Vec3f::ZERO, &vertices, shade::BufferUsage::Static);

		let diffuse = {
			let image = shade::image::DecodedImage::load_file_png("examples/textures/stonefloor-512.diffuse.png").unwrap();
			let props = shade::TextureProps {
				mip_levels: 1,
				usage: shade::TextureUsage::TEXTURE,
				filter_min: shade::TextureFilter::Linear,
				filter_mag: shade::TextureFilter::Linear,
				wrap_u: shade::TextureWrap::Repeat,
				wrap_v: shade::TextureWrap::Repeat,
			};
			g.image(None, &(&image, &props))
		};

		let normal_map = {
			let image = shade::image::DecodedImage::load_file_png("examples/textures/stonefloor-512.normal.png").unwrap();
			let props = shade::TextureProps {
				mip_levels: 1,
				usage: shade::TextureUsage::TEXTURE,
				filter_min: shade::TextureFilter::Linear,
				filter_mag: shade::TextureFilter::Linear,
				wrap_u: shade::TextureWrap::Repeat,
				wrap_v: shade::TextureWrap::Repeat,
			};
			g.image(None, &(&image, &props))
		};

		let height_map = {
			let image = shade::image::DecodedImage::load_file_png("examples/textures/stonefloor-512.height.png").unwrap();
			let props = shade::TextureProps {
				mip_levels: 1,
				usage: shade::TextureUsage::TEXTURE,
				filter_min: shade::TextureFilter::Linear,
				filter_mag: shade::TextureFilter::Linear,
				wrap_u: shade::TextureWrap::Repeat,
				wrap_v: shade::TextureWrap::Repeat,
			};
			g.image(None, &(&image, &props))
		};

		let shader = g.shader_create(None, VERTEX_SHADER, PARALLAX_SHADER);
		let material = ParallaxMaterial {
			shader,
			diffuse,
			normal_map,
			height_map,
			height_scale: 0.04,
		};

		let instance = ParallaxInstance {
			model: Transform3f::IDENTITY,
		};

		ParallaxRenderable { mesh, material, instance }
	}
	fn draw(&self, g: &mut shade::Graphics, camera: &shade::d3::Camera, light: &Light) {
		g.draw(&shade::DrawArgs {
			scissor: None,
			blend_mode: shade::BlendMode::Solid,
			depth_test: Some(shade::DepthTest::Less),
			cull_mode: Some(shade::CullMode::CW),
			mask: shade::DrawMask::ALL,
			prim_type: shade::PrimType::Triangles,
			shader: self.material.shader,
			vertices: &[shade::DrawVertexBuffer {
				buffer: self.mesh.vertices,
				divisor: shade::VertexDivisor::PerVertex,
			}],
			uniforms: &[camera, &self.material, &self.instance, light],
			vertex_start: 0,
			vertex_end: self.mesh.vertices_len,
			instances: -1,
		});
	}
}

//----------------------------------------------------------------

struct Light {
	light_pos: Vec3f,
}
impl shade::UniformVisitor for Light {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_lightPos", &self.light_pos);
	}
}

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
	screen_size: Vec2i,
	projection_type: ProjectionType,
	camera: shade::d3::ArcballCamera,
	tree: OldTreeRenderable,
	floor: ParallaxRenderable,
	axes: shade::d3::axes::AxesModel,
	frustum: shade::d3::frustum::FrustumModel,
	view: Transform3f,
}

impl Scene {
	fn create(g: &mut shade::Graphics, screen_size: Vec2i) -> Scene {
		let tree = OldTreeRenderable::create(g);

		let floor = ParallaxRenderable::create(g);

		let (axes, frustum) = {
			let shader = g.shader_create(None, shade::gl::shaders::COLOR3D_VS, shade::gl::shaders::COLOR3D_FS);
			(shade::d3::axes::AxesModel::create(g, shader), shade::d3::frustum::FrustumModel::create(g, shader, Clip::NO))
		};

		let camera = {
			let pivot = tree.mesh.bounds.center().set_x(0.0).set_y(0.0);
			let position = pivot + Vec3::<f32>::X * tree.mesh.bounds.size().xy().vmax();
			shade::d3::ArcballCamera::new(position, pivot, Vec3::Z)
		};

		let view = camera.view_matrix(Hand::RH);

		let projection_type = ProjectionType::Perspective;

		Scene { screen_size, projection_type, camera, tree, floor, axes, frustum, view }
	}
	fn draw(&mut self, g: &mut shade::Graphics, time: f32) {
		// Render the frame
		let viewport = Bounds2::vec(self.screen_size);
		g.begin(&shade::RenderPassArgs::BackBuffer { viewport });

		// Clear the screen
		shade::clear!(g, color: Vec4(0.5, 0.2, 0.2, 1.0), depth: 1.0);

		let frustum_view_proj;

		// Camera setup
		let camera = {
			let aspect_ratio = self.screen_size.x as f32 / self.screen_size.y as f32;
			let position = self.camera.position();
			let hand = Hand::RH;
			let view = self.camera.view_matrix(hand);
			let clip = Clip::NO;
			let (near, far) = (0.1, 40.0);
			let projection = match self.projection_type {
				ProjectionType::Perspective => Mat4::perspective(Angle::deg(90.0), aspect_ratio, near, far, (hand, clip)),
				ProjectionType::Orthographic => Mat4::ortho(-5.0 * aspect_ratio, 5.0 * aspect_ratio, -5.0, 5.0, near, far, (hand, clip)),
			};
			let view_proj = projection * view;
			frustum_view_proj = projection * self.view;
			let inv_view_proj = view_proj.inverse();
			shade::d3::Camera { viewport, aspect_ratio, position, near, far, view, projection, view_proj, inv_view_proj, clip }
		};

		let radius = 5000.0;
		let angle = time * 2.0; // Adjust speed here
		let light_pos = Vec3f::new(
			radius * angle.cos(),
			radius * angle.sin(),
			40.0, // Fixed elevation, adjust if needed
		);
		let light = Light { light_pos };

		// Draw the models
		self.tree.draw(g, &camera, &light);

		self.floor.draw(g, &camera, &light);

		self.axes.draw(g, &camera, &shade::d3::axes::AxesInstance {
			local: Transform3f::scale(Vec3::dup(camera.position.len() * 0.2)),
			depth_test: None,
		});

		self.frustum.draw(g, &camera, &shade::d3::frustum::FrustumInstance {
			view_proj: frustum_view_proj,
		});

		// Finish the frame
		g.end();
	}
}

//----------------------------------------------------------------
// Application state

struct App {
	size: winit::dpi::PhysicalSize<u32>,
	window: winit::window::Window,
	surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
	context: glutin::context::PossiblyCurrentContext,
	g: shade::gl::GlGraphics,
	scene: Scene,
	left_click: bool,
	right_click: bool,
	middle_click: bool,
	auto_rotate: bool,
	cursor_position: winit::dpi::PhysicalPosition<f64>,
	epoch: time::Instant,
}

impl App {
	fn new(event_loop: &winit::event_loop::ActiveEventLoop) -> Box<App> {
		use glutin::config::ConfigTemplateBuilder;
		use glutin::context::{ContextApi, ContextAttributesBuilder, Version};
		use glutin::display::GetGlDisplay;
		use glutin::surface::{SurfaceAttributesBuilder, WindowSurface};
		use raw_window_handle::HasWindowHandle;

		let size = winit::dpi::PhysicalSize::new(800, 600);

		let template = ConfigTemplateBuilder::new()
			.with_alpha_size(8)
			.with_multisampling(4);

		let window_attributes = winit::window::WindowAttributes::default()
			.with_inner_size(size);

		let (window, gl_config) = glutin_winit::DisplayBuilder::new()
			.with_window_attributes(Some(window_attributes))
			.build(event_loop, template, |configs| configs.max_by_key(|c| c.num_samples()).unwrap())
			.expect("Failed to build window and GL config");

		let window = window.expect("DisplayBuilder did not build a Window");
		let raw_window_handle = window
			.window_handle()
			.expect("Failed to get raw window handle")
			.as_raw();

		let context_attributes = ContextAttributesBuilder::new()
			.with_context_api(ContextApi::OpenGl(Some(Version::new(3, 3))))
			.build(Some(raw_window_handle));

		let gl_display = gl_config.display();

		let not_current = unsafe {
			gl_display.create_context(&gl_config, &context_attributes)
		}.expect("Failed to create GL context");

		let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
			raw_window_handle,
			NonZeroU32::new(size.width.max(1)).unwrap(),
			NonZeroU32::new(size.height.max(1)).unwrap(),
		);

		let surface = unsafe {
			gl_display.create_window_surface(&gl_config, &attrs)
		}.expect("Failed to create GL surface");

		let context = not_current
			.make_current(&surface)
			.expect("Failed to make GL context current");

		shade::gl::capi::load_with(|s| {
			let c = CString::new(s).unwrap();
			gl_display.get_proc_address(&c)
		});

		// Create the graphics context
		let mut g = shade::gl::GlGraphics::new();

		let scene = Scene::create(&mut g, Vec2(size.width as i32, size.height as i32));

		let left_click = false;
		let right_click = false;
		let middle_click = false;
		let auto_rotate = true;
		let cursor_position = winit::dpi::PhysicalPosition::<f64>::new(0.0, 0.0);

		let epoch = time::Instant::now();

		Box::new(App { size, window, surface, context, g, scene, left_click, right_click, middle_click, auto_rotate, cursor_position, epoch })
	}

	fn draw(&mut self) {
		if self.auto_rotate {
			self.scene.camera.rotate(-1.0, 0.0);
		}

		let time = self.epoch.elapsed().as_secs_f32();
		self.scene.draw(&mut self.g, time);
	}
}

fn main() {
	let event_loop = winit::event_loop::EventLoop::new().expect("Failed to create event loop");

	let mut app: Option<Box<App>> = None;

	#[allow(deprecated)]
	let _ = event_loop.run(move |event, event_loop| {
		use winit::event::{ElementState, Event, MouseButton, WindowEvent};
		use winit::keyboard::Key;

		match event {
			Event::Resumed => {
				if app.is_none() {
					app = Some(App::new(event_loop));
				}
			}
			Event::WindowEvent { event, .. } => match event {
				WindowEvent::Resized(new_size) => {
					if let Some(app) = app.as_deref_mut() {
						let width = NonZeroU32::new(new_size.width.max(1)).unwrap();
						let height = NonZeroU32::new(new_size.height.max(1)).unwrap();
						app.size = new_size;
						app.scene.screen_size.x = new_size.width as i32;
						app.scene.screen_size.y = new_size.height as i32;
						app.surface.resize(&app.context, width, height);
					}
				}
				WindowEvent::CursorMoved { position, .. } => {
					if let Some(app) = app.as_deref_mut() {
						let dx = position.x as f32 - app.cursor_position.x as f32;
						let dy = position.y as f32 - app.cursor_position.y as f32;
						if app.left_click {
							app.auto_rotate = false;
							app.scene.camera.rotate(-dx, -dy);
						}
						if app.right_click {
							app.auto_rotate = false;
							app.scene.camera.pan(-dx, dy);
						}
						if app.middle_click {
							app.scene.camera.zoom(dy * 0.01);
						}
						app.cursor_position = position;
					}
				}
				WindowEvent::MouseInput { state, button: MouseButton::Left, .. } => {
					if let Some(app) = app.as_deref_mut() {
						app.left_click = matches!(state, ElementState::Pressed);
					}
				}
				WindowEvent::MouseInput { state, button: MouseButton::Right, .. } => {
					if let Some(app) = app.as_deref_mut() {
						app.right_click = matches!(state, ElementState::Pressed);
					}
				}
				WindowEvent::MouseInput { state, button: MouseButton::Middle, .. } => {
					if let Some(app) = app.as_deref_mut() {
						app.middle_click = matches!(state, ElementState::Pressed);
					}
				}
				WindowEvent::KeyboardInput { event, .. } => {
					if let Some(app) = app.as_deref_mut() {
						if let Key::Character(ch) = &event.logical_key {
							if (ch == "p" || ch == "P") && matches!(event.state, ElementState::Pressed) {
								app.scene.projection_type.toggle();
							}
						}
					}
				}
				WindowEvent::CloseRequested => event_loop.exit(),
				WindowEvent::RedrawRequested => {
					if let Some(app) = app.as_deref_mut() {
						app.draw();
						app.surface.swap_buffers(&app.context).unwrap();
					}
				}
				_ => {}
			},
			Event::AboutToWait => {
				if let Some(app) = app.as_deref() {
					app.window.request_redraw();
				}
			}
			_ => {}
		}
	});
}
