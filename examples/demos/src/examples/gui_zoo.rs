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
const CONTEXT_MENU_HEIGHT: i32 = gui::widgets::MENU_ITEM_HEIGHT * 4 + gui::widgets::SEPARATOR_HEIGHT;

pub fn create(g: &mut shade::Graphics, assets: &dyn AssetLoader) -> Box<dyn DemoInterface> {
	Box::new(GuiZoo::new(g, assets))
}

struct GuiZoo {
	scene: gui::Scene,
	state: ZooState,
	font: d2::FontResource<shade::atlas::Font>,
	color_shader: Box<dyn shade::ShaderProgram>,
	background: gui::SlotKey,
	menu_bar: gui::SlotKey,
	context_menu: gui::SlotKey,
	start: Instant,
}

struct ZooState {
	drawing: drawing_window::State,
	main: main_window::State,
	floating: floating_window::State,
	context_copy: gui::SlotKey,
	context_reset: gui::SlotKey,
	context_submenu_first: gui::SlotKey,
	context_submenu_second: gui::SlotKey,
	context_disabled: gui::SlotKey,
	menu_file_new: gui::SlotKey,
	menu_file_recent_first: gui::SlotKey,
	menu_edit_copy: gui::SlotKey,
}

impl ZooState {
	fn bind_names(&mut self, ctx: &gui::dto::BuildContext) {
		self.main.bind_names(ctx);
		self.floating.bind_names(ctx);
	}
}

impl gui::AppState for ZooState {
	fn scope<'a>(&'a self, key: gui::SlotKey) -> &'a dyn gui::AppState {
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

	fn scope_mut<'a>(&'a mut self, key: gui::SlotKey) -> &'a mut dyn gui::AppState {
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

	fn prop(&self, _key: gui::PropKey, _f: &mut dyn FnMut(&dyn std::any::Any)) {}

	fn emit(&mut self, event: &dyn gui::UserEvent) {
		let Some(event) = event.downcast_ref::<gui::widgets::MenuItemClicked>() else {
			return;
		};
		let status = if event.key == self.context_copy {
			"Context menu: first action selected."
		}
		else if event.key == self.context_reset {
			"Context menu: second action selected."
		}
		else if event.key == self.context_submenu_first {
			"Context submenu: first nested action selected."
		}
		else if event.key == self.context_submenu_second {
			"Context submenu: second nested action selected."
		}
		else if event.key == self.context_disabled {
			"Context menu: disabled item selected."
		}
		else if event.key == self.menu_file_new {
			"Menu bar: File → New selected."
		}
		else if event.key == self.menu_file_recent_first {
			"Menu bar: File → Recent → First project selected."
		}
		else if event.key == self.menu_edit_copy {
			"Menu bar: Edit → Copy selected."
		}
		else {
			return;
		};
		self.main.set_status(status);
	}
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
		self.scene.get_cursor(&self.state).unwrap_or(Cursor::Default)
	}

	fn new(g: &mut shade::Graphics, assets: &dyn AssetLoader) -> GuiZoo {
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
		let menu_bar = gui::dto::MenuBar {
			name: Some("menu_bar".to_owned()),
			children: vec![
				gui::dto::MenuBarItem {
					name: Some("menu_file".to_owned()),
					label: gui::dto::Property::Value("File".to_owned()),
					enabled: None,
					menu: Box::new(gui::dto::Menu {
						name: Some("menu_file_popup".to_owned()),
						children: vec![
							gui::dto::Widget::MenuItem(gui::dto::MenuItem {
								name: Some("menu_file_new".to_owned()),
								label: gui::dto::Property::Value("New".to_owned()),
								enabled: None,
								submenu: None,
							}),
							gui::dto::Widget::MenuItem(gui::dto::MenuItem {
								name: Some("menu_file_recent".to_owned()),
								label: gui::dto::Property::Value("Recent".to_owned()),
								enabled: None,
								submenu: Some(Box::new(gui::dto::Menu {
									name: Some("menu_file_recent_popup".to_owned()),
									children: vec![
										gui::dto::Widget::MenuItem(gui::dto::MenuItem {
											name: Some("menu_file_recent_first".to_owned()),
											label: gui::dto::Property::Value("First project".to_owned()),
											enabled: None,
											submenu: None,
										}),
										gui::dto::Widget::MenuItem(gui::dto::MenuItem {
											name: Some("menu_file_recent_second".to_owned()),
											label: gui::dto::Property::Value("Second project".to_owned()),
											enabled: None,
											submenu: None,
										}),
									],
								})),
							}),
							gui::dto::Widget::Separator(gui::dto::Separator { name: None }),
							gui::dto::Widget::MenuItem(gui::dto::MenuItem {
								name: Some("menu_file_disabled".to_owned()),
								label: gui::dto::Property::Value("Unavailable".to_owned()),
								enabled: Some(gui::dto::Property::Value(false)),
								submenu: None,
							}),
						],
					}),
				},
				gui::dto::MenuBarItem {
					name: Some("menu_edit".to_owned()),
					label: gui::dto::Property::Value("Edit".to_owned()),
					enabled: None,
					menu: Box::new(gui::dto::Menu {
						name: Some("menu_edit_popup".to_owned()),
						children: vec![
							gui::dto::Widget::MenuItem(gui::dto::MenuItem {
								name: Some("menu_edit_copy".to_owned()),
								label: gui::dto::Property::Value("Copy".to_owned()),
								enabled: None,
								submenu: None,
							}),
							gui::dto::Widget::MenuItem(gui::dto::MenuItem {
								name: Some("menu_edit_paste".to_owned()),
								label: gui::dto::Property::Value("Paste".to_owned()),
								enabled: None,
								submenu: None,
							}),
						],
					}),
				},
			],
		}.construct(&mut scene, &mut build_ctx);
		let context_menu = gui::dto::Menu {
			name: Some("context_menu".to_owned()),
			children: vec![
				gui::dto::Widget::MenuItem(gui::dto::MenuItem {
					name: Some("context_copy".to_owned()),
					label: gui::dto::Property::Value("First context action".to_owned()),
					enabled: None,
					submenu: None,
				}),
				gui::dto::Widget::MenuItem(gui::dto::MenuItem {
					name: Some("context_reset".to_owned()),
					label: gui::dto::Property::Value("Second context action".to_owned()),
					enabled: None,
					submenu: None,
				}),
				gui::dto::Widget::MenuItem(gui::dto::MenuItem {
					name: Some("context_more".to_owned()),
					label: gui::dto::Property::Value("More actions".to_owned()),
					enabled: None,
					submenu: Some(Box::new(gui::dto::Menu {
						name: Some("context_more_menu".to_owned()),
						children: vec![
							gui::dto::Widget::MenuItem(gui::dto::MenuItem {
								name: Some("context_submenu_first".to_owned()),
								label: gui::dto::Property::Value("First nested action".to_owned()),
								enabled: None,
								submenu: None,
							}),
							gui::dto::Widget::MenuItem(gui::dto::MenuItem {
								name: Some("context_submenu_second".to_owned()),
								label: gui::dto::Property::Value("Second nested action".to_owned()),
								enabled: None,
								submenu: None,
							}),
						],
					})),
				}),
				gui::dto::Widget::Separator(gui::dto::Separator { name: None }),
				gui::dto::Widget::MenuItem(gui::dto::MenuItem {
					name: Some("context_disabled".to_owned()),
					label: gui::dto::Property::Value("Unavailable action".to_owned()),
					enabled: Some(gui::dto::Property::Value(false)),
					submenu: None,
				}),
			],
		}.construct(&mut scene, &mut build_ctx);
		scene.show(background, BACKGROUND_BOUNDS);
		scene.show(drawing_window, DRAWING_WINDOW_BOUNDS);
		scene.show(main_window, MAIN_WINDOW_BOUNDS);
		scene.show(floating_window, FLOATING_WINDOW_BOUNDS);
		scene.show(menu_bar, shade::cvmath::Bounds2!(0, 0, GUI_ZOO_SIZE.x, gui::widgets::MENU_BAR_HEIGHT));
		let context_copy = build_ctx.key("context_copy").expect("context_copy");
		let context_reset = build_ctx.key("context_reset").expect("context_reset");
		let context_submenu_first = build_ctx.key("context_submenu_first").expect("context_submenu_first");
		let context_submenu_second = build_ctx.key("context_submenu_second").expect("context_submenu_second");
		let context_disabled = build_ctx.key("context_disabled").expect("context_disabled");
		let menu_file_new = build_ctx.key("menu_file_new").expect("menu_file_new");
		let menu_file_recent_first = build_ctx.key("menu_file_recent_first").expect("menu_file_recent_first");
		let menu_edit_copy = build_ctx.key("menu_edit_copy").expect("menu_edit_copy");
		let shared = Rc::new(RefCell::new(shared_state::State::new()));
		let mut state = ZooState {
			drawing: drawing_window::State::new(shared.clone(), drawing_window),
			main: main_window::State::new(shared.clone(), main_window),
			floating: floating_window::State::new(shared, floating_window),
			context_copy,
			context_reset,
			context_submenu_first,
			context_submenu_second,
			context_disabled,
			menu_file_new,
			menu_file_recent_first,
			menu_edit_copy,
		};
		state.bind_names(&build_ctx);
		let background = build_ctx.key("background").expect("background");

		GuiZoo {
			scene,
			state,
			font,
			color_shader,
			background,
			menu_bar,
			context_menu,
			start: Instant::now(),
		}
	}

	fn send_mouse(&mut self, event: &gui::MouseEvent) {
		self.scene.mouse_event(event, self.start, &mut self.state);
	}

	fn show_context_menu(&mut self, pointer: shade::cvmath::Vec2i) {
		let size = self.scene.size();
		let left = pointer.x.clamp(0, (size.x - gui::widgets::MENU_WIDTH).max(0));
		let top = pointer.y.clamp(0, (size.y - CONTEXT_MENU_HEIGHT).max(0));
		let bounds = shade::cvmath::Bounds2!(
			left,
			top,
			left + gui::widgets::MENU_WIDTH,
			top + CONTEXT_MENU_HEIGHT,
		);
		self.scene.show_popup(self.context_menu, bounds);
	}

}

impl DemoInterface for GuiZoo {
	fn resize(&mut self, size: shade::cvmath::Vec2i) {
		self.scene.resize(size);
		self.scene.show(self.background, shade::cvmath::Bounds2i::vec(size));
		self.scene.show(self.menu_bar, shade::cvmath::Bounds2!(0, 0, size.x, gui::widgets::MENU_BAR_HEIGHT));
	}

	fn input(&mut self, input: Input, _g: &mut shade::Graphics, shell: &mut dyn ShellServices) {
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
			Input::MouseButton { button: gui::MouseButton::RIGHT, pressed, position } => {
				let pointer = position.cast();
				let kind = if pressed {
					gui::MouseEventKind::ButtonDown { button: gui::MouseButton::RIGHT }
				}
				else {
					gui::MouseEventKind::ButtonUp { button: gui::MouseButton::RIGHT }
				};
				self.send_mouse(&gui::MouseEvent { kind, pointer });
				if pressed {
					self.show_context_menu(pointer);
				}
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

	fn draw(&mut self, frame: Frame, g: &mut shade::Graphics) {
		let viewport = frame.viewport;
		g.begin(&shade::BeginArgs::BackBuffer { viewport });
		shade::clear!(g, color: shade::cvmath::Vec4(0.105, 0.115, 0.13, 1.0));

		self.state.main.set_pulse_value((self.start.elapsed().as_secs_f32().sin() * 0.5 + 0.5) * 0.35 + 0.45);
		let resources = gui::SystemResources {
			font: self.font.as_dyn(),
			color_shader: &*self.color_shader,
		};
		let mut draw_pool = shade::im::DrawPool::new();
		self.scene.layout(self.start, &resources, &mut self.state);
		self.scene.draw(g, &mut draw_pool, self.start, &resources, &self.state);
		draw_pool.draw(g);
		g.end();
	}
}
