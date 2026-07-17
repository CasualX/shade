use super::*;

const DEFAULT_FILL: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(92, 150, 218));
const DEFAULT_BACKGROUND: cvmath::Vec4<u8> = cvmath::Vec4!(rgb(26, 28, 34));

/// Progress bar widget.
pub struct ProgressBar {
	key: SlotKey,
	value: Property<f32>, // [0.0, 1.0]
	fill: Property<cvmath::Vec4<u8>>,
	background: Property<cvmath::Vec4<u8>>,
}

const TRACK_HEIGHT: i32 = 10;

impl ProgressBar {
	fn track_rect(&self, bounds: cvmath::Bounds2i) -> cvmath::Bounds2i {
		let track_left = bounds.left();
		let track_top = bounds.top() + (bounds.height() - TRACK_HEIGHT) / 2;
		let track_width = bounds.width().max(1);
		cvmath::Bounds2!(track_left, track_top, track_left + track_width, track_top + TRACK_HEIGHT)
	}
}

impl Widget for ProgressBar {
	fn key(&self) -> SlotKey {
		self.key
	}

	fn draw<'a>(&mut self, _g: &mut dyn IGraphics, im: &mut im::DrawPool<'a>, ctx: &DrawContext, resx: &'a dyn Resources, app: &dyn AppState, app_ctx: &dyn AppContext) {
		let rc = cvmath::Bounds2i::vec(ctx.bounds.size());
		let value = self.value.copied_or(app, app_ctx, 0.0).clamp(0.0, 1.0);
		let fill = self.fill.copied_or(app, app_ctx, DEFAULT_FILL);
		let background = self.background.copied_or(app, app_ctx, DEFAULT_BACKGROUND);
		let shader = resx.get_shader(SystemResources::COLOR_SHADER_KEY).unwrap();

		let track = self.track_rect(rc);
		im.fill_rect(ctx, track, background, shader);
		let fill_right = track.left() + (value * track.width() as f32) as i32;
		let fill_rc = cvmath::Bounds2!(track.left(), track.top(), fill_right, track.bottom());
		im.fill_rect(ctx, fill_rc, fill, shader);
	}
}

impl dto::ProgressBar {
	pub fn construct(self, scene: &mut Scene, ctx: &mut dto::BuildContext) -> SlotKey {
		let name = self.name;
		let value = From::from(self.value);
		let fill = From::from(self.fill);
		let background = self.background.map(From::from).unwrap_or(Property::Value(DEFAULT_BACKGROUND));
		scene.create_widget(|key| {
			ctx.insert(name, key);
			ProgressBar { key, value, fill, background }
		})
	}
}
