use super::*;

const DEFAULT_BACKGROUND: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(27, 29, 34));
const DEFAULT_LINE_COLOR: cvmath::Vec4<u8> = cvmath::Vec4!(rgba(255, 255, 255, 0.06));
const DEFAULT_SPACING: i32 = 48;
const GRID_LINE_THICKNESS: f32 = 1.0;

/// Decorative widget that draws a background grid.
pub struct DrawGrid {
	key: SlotKey,
	background: cvmath::Vec4<u8>,
	line_color: cvmath::Vec4<u8>,
	spacing: i32,
}

impl Widget for DrawGrid {
	fn key(&self) -> SlotKey {
		self.key
	}

	fn hittable(&self) -> bool {
		false
	}

	fn draw<'a>(&mut self, _g: &mut Graphics, im: &mut im::DrawPool<'a>, ctx: &DrawContext, resx: &'a dyn Resources, _app: &dyn AppState) {
		let bounds = cvmath::Bounds2i::vec(ctx.bounds.size());
		let shader = resx.get_shader(SystemResources::COLOR_SHADER_KEY).unwrap();
		im.fill_rect(ctx, bounds, self.background, shader);

		let spacing = self.spacing.max(1) as usize;
		for x in (0..bounds.width()).step_by(spacing) {
			let line = cvmath::Bounds2!(x, 0, x + 1, bounds.height());
			im.fill_edge_rect(ctx, line, self.line_color, GRID_LINE_THICKNESS, shader);
		}
		for y in (0..bounds.height()).step_by(spacing) {
			let line = cvmath::Bounds2!(0, y, bounds.width(), y + 1);
			im.fill_edge_rect(ctx, line, self.line_color, GRID_LINE_THICKNESS, shader);
		}
	}
}

impl dto::DrawGrid {
	pub fn construct(self, scene: &mut Scene, ctx: &mut dto::BuildContext) -> SlotKey {
		let name = self.name;
		let background = self.background.unwrap_or(DEFAULT_BACKGROUND);
		let line_color = self.line_color.unwrap_or(DEFAULT_LINE_COLOR);
		let spacing = self.spacing.unwrap_or(DEFAULT_SPACING);
		scene.create_widget(|key| {
			ctx.insert(name, key);
			DrawGrid { key, background, line_color, spacing }
		})
	}
}
