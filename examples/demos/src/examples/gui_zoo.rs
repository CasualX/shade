use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use crate::*;

use shade::gui;

mod drawing_window;
mod floating_window;
mod main_window;
mod shared_state;

const GUI_ZOO_SIZE: shade::cvmath::Vec2i = shade::cvmath::Vec2i::new(1100, 660);
const BACKGROUND_BOUNDS: shade::cvmath::Bounds2i = shade::cvmath::Bounds2i::new(shade::cvmath::Point2(0, 0), shade::cvmath::Point2(1100, 660));
const DRAWING_WINDOW_BOUNDS: shade::cvmath::Bounds2i = shade::cvmath::Bounds2i::new(shade::cvmath::Point2(520, 48), shade::cvmath::Point2(980, 424));
const MAIN_WINDOW_BOUNDS: shade::cvmath::Bounds2i = shade::cvmath::Bounds2i::new(shade::cvmath::Point2(24, 24), shade::cvmath::Point2(494, 664));
const FLOATING_WINDOW_BOUNDS: shade::cvmath::Bounds2i = shade::cvmath::Bounds2i::new(shade::cvmath::Point2(790, 98), shade::cvmath::Point2(1060, 288));

pub fn create(g: &mut dyn shade::IGraphics, assets: &dyn AssetLoader) -> Box<dyn DemoInterface> {
	Box::new(GuiZoo::new(g, assets))
}

struct GuiZoo {
	scene: gui::Scene,
	state: ZooState,
	ctx: ZooAppContext,
	font: d2::FontResource<shade::atlas::Font>,
	color_shader: Box<dyn shade::ShaderProgram>,
	background: gui::SlotKey,
	start: Instant,
}

struct ZooAppContext;

impl gui::AppContext for ZooAppContext {}

struct ZooState {
	drawing: drawing_window::State,
	main: main_window::State,
	floating: floating_window::State,
}

impl ZooState {
	fn bind_names(&mut self, ctx: &gui::dto::BuildContext) {
		self.main.bind_names(ctx);
		self.floating.bind_names(ctx);
	}
}

impl gui::AppState for ZooState {
	fn scope<'a>(&'a self, key: gui::SlotKey, _ctx: &dyn gui::AppContext) -> &'a dyn gui::AppState {
		if key == self.drawing.window {
			&self.drawing
		}
		else if key == self.main.window {
			&self.main
		}
		else if key == self.floating.window {
			&self.floating
		}
		else {
			self
		}
	}

	fn scope_mut<'a>(&'a mut self, key: gui::SlotKey, _ctx: &mut dyn gui::AppContext) -> &'a mut dyn gui::AppState {
		if key == self.drawing.window {
			&mut self.drawing
		}
		else if key == self.main.window {
			&mut self.main
		}
		else if key == self.floating.window {
			&mut self.floating
		}
		else {
			self
		}
	}

	fn prop(&self, _key: gui::PropKey, _ctx: &dyn gui::AppContext, _f: &mut dyn FnMut(&dyn std::any::Any)) {}
}

fn lerp_color(a: shade::cvmath::Vec4<u8>, b: shade::cvmath::Vec4<u8>, t: f32) -> shade::cvmath::Vec4<u8> {
	shade::cvmath::lerp(a.cast(), b.cast(), t).cast()
}

fn high_fill_color(value: f32) -> shade::cvmath::Vec4<u8> {
	let value = value.clamp(0.0, 1.0);
	let green = shade::cvmath::Vec4!(rgb(96, 171, 106));
	let yellow = shade::cvmath::Vec4!(rgb(232, 202, 84));
	let red = shade::cvmath::Vec4!(rgb(218, 90, 86));
	if value < 0.33 {
		red
	}
	else if value < 0.66 {
		lerp_color(red, yellow, (value - 0.33) * 3.0)
	}
	else {
		lerp_color(yellow, green, (value - 0.66) * 3.0)
	}
}

impl GuiZoo {
	fn cursor(&self, _point: shade::cvmath::Vec2i) -> Cursor {
		self.scene.get_cursor(&self.state, &self.ctx).unwrap_or(Cursor::Default)
	}

	fn new(g: &mut dyn shade::IGraphics, assets: &dyn AssetLoader) -> GuiZoo {
		let mut shader_source = shade::shader_interface! {
			files {
				"color.glsl" => shade::shaders::COLOR,
			}
		};
		let color_shader = g.shader_compile(&mut shader_source, "color.glsl", &[]);
		let font = load_font(g, assets, "font/font.json", "font/font.png", false);

		let mut scene = gui::Scene::new(GUI_ZOO_SIZE);
		let mut build_ctx = gui::dto::BuildContext::new();
		let background = gui::dto::DrawGrid {
			name: Some("background".to_owned()),
			background: None,
			line_color: None,
			spacing: None,
		}.construct(&mut scene, &mut build_ctx);
		let drawing_window = drawing_window::load(&mut scene, &mut build_ctx);
		let main_window = main_window::load(&mut scene, &mut build_ctx);
		let floating_window = floating_window::load(&mut scene, &mut build_ctx);
		scene.show(background, BACKGROUND_BOUNDS);
		scene.show(drawing_window, DRAWING_WINDOW_BOUNDS);
		scene.show(main_window, MAIN_WINDOW_BOUNDS);
		scene.show(floating_window, FLOATING_WINDOW_BOUNDS);
		let shared = Rc::new(RefCell::new(shared_state::State::new()));
		let mut state = ZooState {
			drawing: drawing_window::State::new(shared.clone(), drawing_window),
			main: main_window::State::new(shared.clone(), main_window),
			floating: floating_window::State::new(shared, floating_window),
		};
		state.bind_names(&build_ctx);
		let background = build_ctx.key("background").expect("background");

		GuiZoo {
			scene,
			state,
			ctx: ZooAppContext,
			font,
			color_shader,
			background,
			start: Instant::now(),
		}
	}

	fn send_mouse(&mut self, event: &gui::MouseEvent) {
		self.scene.mouse_event(event, self.start, &mut self.state, &mut self.ctx);
	}
}

impl DemoInterface for GuiZoo {
	fn resize(&mut self, size: shade::cvmath::Vec2i) {
		self.scene.resize(size);
		self.scene.show(self.background, shade::cvmath::Bounds2i::vec(size));
	}

	fn input(&mut self, input: Input, _g: &mut dyn shade::IGraphics, shell: &mut dyn ShellServices) {
		match input {
			Input::MouseMove { position } => {
				let pointer = position.cast();
				self.send_mouse(&gui::MouseEvent { kind: gui::MouseEventKind::Move, pointer });
				shell.set_cursor(self.cursor(pointer));
				shell.request_redraw();
			},
			Input::MouseButton { button: gui::MouseButton::LEFT, pressed, position } => {
				let kind = if pressed {
					gui::MouseEventKind::ButtonDown { button: gui::MouseButton::LEFT }
				}
				else {
					gui::MouseEventKind::ButtonUp { button: gui::MouseButton::LEFT }
				};
				let pointer = position.cast();
				self.send_mouse(&gui::MouseEvent { kind, pointer });
				shell.set_cursor(self.cursor(pointer));
				shell.request_redraw();
			},
			Input::MouseWheel { delta, position } => {
				let pointer = position.cast();
				self.send_mouse(&gui::MouseEvent { kind: gui::MouseEventKind::Wheel { delta: delta.y as i32 }, pointer });
				shell.set_cursor(self.cursor(pointer));
				shell.request_redraw();
			},
			_ => {},
		}
	}

	fn draw(&mut self, frame: Frame, g: &mut dyn shade::IGraphics) {
		let viewport = frame.viewport;
		g.begin(&shade::BeginArgs::BackBuffer { viewport });
		shade::clear!(g, color: shade::cvmath::Vec4(0.105, 0.115, 0.13, 1.0));

		self.state.main.set_pulse_value((self.start.elapsed().as_secs_f32().sin() * 0.5 + 0.5) * 0.35 + 0.45);
		let resources = gui::SystemResources {
			font: self.font.as_dyn(),
			color_shader: &*self.color_shader,
		};
		let mut draw_pool = shade::im::DrawPool::new();
		self.scene.layout(self.start, &resources, &self.state, &self.ctx);
		self.scene.draw(g, &mut draw_pool, self.start, &resources, &self.state, &self.ctx);
		draw_pool.draw(g);
		g.end();
	}
}
