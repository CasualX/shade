use crate::*;

const SCREEN_MELT_COLUMNS: i32 = 160;

struct PostProcessMeltUniforms<'a> {
	scene: &'a dyn shade::Texture2D,
	delays: &'a dyn shade::Texture2D,
	time: f32,
}

impl<'a> shade::UniformVisitor for PostProcessMeltUniforms<'a> {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_scene", self.scene);
		set.value("u_delays", self.delays);
		set.value("u_time", &self.time);
	}
}

pub fn create(g: &mut shade::Graphics, assets: &dyn AssetLoader) -> Box<dyn DemoInterface> {
	Box::new(ScreenMelt::new(g, assets))
}

struct ScreenMelt {
	pp: shade::d2::PostProcessQuad,
	pp_copy_shader: Box<dyn shade::ShaderProgram>,
	pp_melt_shader: Box<dyn shade::ShaderProgram>,
	gameplay_texture: Box<dyn shade::Texture2D>,
	main_menu_texture: Box<dyn shade::Texture2D>,
	delay_texture: Box<dyn shade::Texture2D>,
}

impl ScreenMelt {
	fn new(g: &mut shade::Graphics, assets: &dyn AssetLoader) -> ScreenMelt {
		let delay_texture = {
			let mut delays = Vec::new();
			let mut offset = 128u8;
			let mut rng = urandom::new();
			for _ in 0..SCREEN_MELT_COLUMNS {
				let a = (rng.range(0u8..4) as i8 - 1) * 8;
				offset = offset.saturating_add_signed(a);
				delays.push(offset);
			}
			let info = shade::Texture2DInfo {
				format: shade::TextureFormat::R8,
				width: SCREEN_MELT_COLUMNS,
				height: 1,
				props: shade::TextureProps! {
					usage: shade::TextureUsage::TEXTURE,
					filter: shade::TextureFilter::Nearest,
					wrap: shade::TextureWrap::Edge,
				},
			};
			g.texture2d(&info, &delays)
		};
		let gameplay_texture = {
			let bytes = assets.read("screenmelt/e1m1.gif").unwrap();
			let image = shade::image::DecodedImage::load_memory(&bytes).unwrap();
			g.image(&image)
		};
		let main_menu_texture = {
			let bytes = assets.read("screenmelt/main-menu.png").unwrap();
			let image = shade::image::DecodedImage::load_memory(&bytes).unwrap();
			g.image(&image)
		};
		let pp = shade::d2::PostProcessQuad::create_flipped(g);
		let mut shader_source = shade::shader_interface! {
			files {
				"post_process.copy.glsl" => shade::shaders::POST_PROCESS_COPY,
				"post_process.melt.glsl" => shade::shaders::POST_PROCESS_MELT,
			}
		};
		let pp_copy_shader = g.shader_compile(&mut shader_source, "post_process.copy.glsl", &[]);
		let pp_melt_shader = g.shader_compile(&mut shader_source, "post_process.melt.glsl", &[]);
		ScreenMelt { gameplay_texture, main_menu_texture, delay_texture, pp, pp_copy_shader, pp_melt_shader }
	}
}

impl DemoInterface for ScreenMelt {
	fn draw(&mut self, frame: Frame, g: &mut shade::Graphics) {
		g.begin(&shade::BeginArgs::BackBuffer { viewport: frame.viewport });
		self.pp.draw(g,
			&*self.pp_copy_shader,
			shade::BlendMode::Alpha,
			&[&shade::shaders::PostProcessCopyUniforms {
				texture: &*self.gameplay_texture,
			}],
		);
		self.pp.draw(g, &*self.pp_melt_shader, shade::BlendMode::Alpha, &[&PostProcessMeltUniforms {
			scene: &*self.main_menu_texture,
			delays: &*self.delay_texture,
			time: (frame.time as f32 - 2.0) * 2.0,
		}]);
		g.end();
	}
}
