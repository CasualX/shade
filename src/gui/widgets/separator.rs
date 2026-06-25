use super::*;

/// Fixed separator height until widgets gain preferred sizes.
pub const SEPARATOR_HEIGHT: i32 = 9;

const LINE_INSET_X: i32 = 8;
const LINE_COLOR: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(83, 88, 101));

/// Decorative separator between menu items.
pub struct Separator {
	key: SlotKey,
}

impl Widget for Separator {
	fn key(&self) -> SlotKey {
		self.key
	}

	fn hittable(&self) -> bool {
		false
	}

	fn draw<'a>(&mut self, _g: &mut Graphics, im: &mut im::DrawPool<'a>, ctx: &DrawContext, resx: &'a dyn Resources, _app: &dyn AppState) {
		let bounds = cvmath::Bounds2i::vec(ctx.bounds.size());
		let y = bounds.top() + bounds.height() / 2;
		let line = cvmath::Bounds2!(bounds.left() + LINE_INSET_X, y, bounds.right() - LINE_INSET_X, y + 1);
		let shader = resx.get_shader(SystemResources::COLOR_SHADER_KEY).unwrap();
		im.fill_rect(ctx, line, LINE_COLOR, shader);
	}
}

impl dto::Separator {
	pub fn construct(self, scene: &mut Scene, ctx: &mut dto::BuildContext) -> SlotKey {
		let name = self.name;
		scene.create_widget(|key| {
			ctx.insert(name, key);
			Separator { key }
		})
	}
}
