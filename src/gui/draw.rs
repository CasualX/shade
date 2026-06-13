#![allow(dead_code)]

use super::*;

/// Convenience drawing primitives for GUI rendering on [`im::DrawPool`].
pub trait DrawExt<'a> {
	/// Fills a rectangle using the GUI color shader and current scissor state.
	fn fill_rect(&mut self, ctx: &DrawContext, rc: cvmath::Bounds2i, color: cvmath::Vec4<u8>, shader: &'a dyn ShaderProgram);

	/// Draws the outline of a rectangle using the GUI color shader and current scissor state.
	fn fill_edge_rect(&mut self, ctx: &DrawContext, rc: cvmath::Bounds2i, color: cvmath::Vec4<u8>, thickness: f32, shader: &'a dyn ShaderProgram);

	/// Draws a single line of text at a position relative to the current widget offset.
	fn draw_text(&mut self, ctx: &DrawContext, font: &d2::FontResource<&'a dyn d2::IFont, &'a dyn Texture2D, &'a dyn ShaderProgram>, scribe: &mut d2::Scribe, pos: cvmath::Vec2f, text: &str);

	/// Draws text aligned within a rectangle relative to the current widget offset.
	fn draw_text_box(&mut self, ctx: &DrawContext, font: &d2::FontResource<&'a dyn d2::IFont, &'a dyn Texture2D, &'a dyn ShaderProgram>, scribe: &d2::Scribe, rc: &cvmath::Bounds2i, align: d2::TextAlign, text: &str);
}

impl<'a> DrawExt<'a> for im::DrawPool<'a> {
	fn fill_rect(&mut self, ctx: &DrawContext, rc: cvmath::Bounds2i, color: cvmath::Vec4<u8>, shader: &'a dyn ShaderProgram) {
		let buf = self.get::<d2::ColorVertex, d2::ColorUniform>();
		buf.blend_mode = BlendMode::Alpha;
		buf.cull_mode = None;
		buf.scissor = Some(ctx.scissor());
		buf.shader = Some(shader);
		buf.uniform.transform = cvmath::Transform2::ortho(ctx.viewport.cast());
		let paint = d2::Paint {
			template: d2::ColorTemplate { color1: color, color2: color },
		};
		buf.fill_rect(&paint, &(rc + ctx.bounds.mins).cast());
	}

	fn fill_edge_rect(&mut self, ctx: &DrawContext, rc: cvmath::Bounds2i, color: cvmath::Vec4<u8>, thickness: f32, shader: &'a dyn ShaderProgram) {
		let buf = self.get::<d2::ColorVertex, d2::ColorUniform>();
		buf.blend_mode = BlendMode::Alpha;
		buf.cull_mode = None;
		buf.scissor = Some(ctx.scissor());
		buf.shader = Some(shader);
		buf.uniform.transform = cvmath::Transform2::ortho(ctx.viewport.cast());
		let paint = d2::Paint {
			template: d2::ColorTemplate { color1: color, color2: color },
		};
		buf.fill_edge_rect(&paint, &(rc + ctx.bounds.mins).cast(), thickness);
	}

	fn draw_text(&mut self, ctx: &DrawContext, font: &d2::FontResource<&'a dyn d2::IFont, &'a dyn Texture2D, &'a dyn ShaderProgram>, scribe: &mut d2::Scribe, pos: cvmath::Vec2f, text: &str) {
		let buf = self.get::<d2::TextVertex, d2::TextUniform<'a>>();
		buf.blend_mode = BlendMode::Alpha;
		buf.cull_mode = None;
		buf.scissor = Some(ctx.scissor());
		buf.uniform.transform = cvmath::Transform2::ortho(ctx.viewport.cast());
		buf.uniform.outline_width_relative = 0.05;
		let offset: cvmath::Vec2f = ctx.bounds.mins.cast();
		let mut cursor = d2::Cursor(pos + offset);
		buf.text_write_ref(font, scribe, &mut cursor, text);
	}

	fn draw_text_box(&mut self, ctx: &DrawContext, font: &d2::FontResource<&'a dyn d2::IFont, &'a dyn Texture2D, &'a dyn ShaderProgram>, scribe: &d2::Scribe, rc: &cvmath::Bounds2i, align: d2::TextAlign, text: &str) {
		let buf = self.get::<d2::TextVertex, d2::TextUniform<'a>>();
		buf.blend_mode = BlendMode::Alpha;
		buf.cull_mode = None;
		buf.scissor = Some(ctx.scissor());
		buf.uniform.transform = cvmath::Transform2::ortho(ctx.viewport.cast());
		buf.uniform.outline_width_relative = 0.05;
		buf.text_box_ref(font, scribe, &(*rc + ctx.bounds.mins).cast(), align, text);
	}
}
