use super::*;

const DEFAULT_BORDER: cvmath::Vec4<u8> = cvmath::Vec4!(rgba(250, 250, 252, 0.35));
const TRANSPARENT: cvmath::Vec4<u8> = cvmath::Vec4!(rgba(0, 0, 0, 0));
const EDGE_THICKNESS: f32 = 1.0;

/// Simple color swatch widget.
pub struct ColorSwatch {
	key: SlotKey,
	color: Property<cvmath::Vec4<u8>>,
	border: Property<cvmath::Vec4<u8>>,
	inset: i32,
}

const DEFAULT_INSET: i32 = 5;

impl Widget for ColorSwatch {
	fn key(&self) -> SlotKey {
		self.key
	}

	fn draw<'a>(&mut self, _g: &mut Graphics, im: &mut im::DrawPool<'a>, ctx: &DrawContext, resx: &'a dyn Resources, app: &dyn AppState) {
		let shader = resx.get_shader(SystemResources::COLOR_SHADER_KEY).unwrap();
		let bounds = cvmath::Bounds2i::vec(ctx.bounds.size()).inset(self.inset);
		let color = self.color.copied_or(app, TRANSPARENT);
		if color.w != 0 {
			im.fill_rect(ctx, bounds, color, shader);
		}
		let border = self.border.copied_or(app, TRANSPARENT);
		if border.w != 0 {
			im.fill_edge_rect(ctx, bounds, border, EDGE_THICKNESS, shader);
		}
	}
}

impl dto::ColorSwatch {
	pub fn construct(self, scene: &mut Scene, ctx: &mut dto::BuildContext) -> SlotKey {
		let name = self.name;
		let color = From::from(self.color);
		let border = self.border.map(From::from).unwrap_or(Property::Value(DEFAULT_BORDER));
		let inset = self.inset.unwrap_or(DEFAULT_INSET);
		scene.create_widget(|key| {
			ctx.insert(name, key);
			ColorSwatch { key, color, border, inset }
		})
	}
}
