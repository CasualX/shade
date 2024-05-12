use super::*;

/// Text uniform.
#[derive(Copy, Clone, Debug, dataview::Pod)]
#[repr(C)]
pub struct TextUniform {
	pub transform: cvmath::Transform2<f32>,
	pub texture: Texture2D,
	pub unit_range: Vec2<f32>,
	pub threshold: f32,
	pub out_bias: f32,
	pub outline_width_absolute: f32,
	pub outline_width_relative: f32,
	pub gamma: f32,
}

impl Default for TextUniform {
	fn default() -> Self {
		TextUniform {
			transform: cvmath::Transform2::IDENTITY,
			texture: Texture2D::INVALID,
			unit_range: Vec2::dup(4.0f32) / Vec2(232.0f32, 232.0f32),
			threshold: 0.5,
			out_bias: 0.0,
			outline_width_absolute: 1.0,
			outline_width_relative: 0.125,
			gamma: 1.0,
		}
	}
}

unsafe impl TUniform for TextUniform {
	const UNIFORM_LAYOUT: &'static UniformLayout = &UniformLayout {
		size: std::mem::size_of::<TextUniform>() as u16,
		alignment: std::mem::align_of::<TextUniform>() as u16,
		attributes: &[
			UniformAttribute {
				name: "u_transform",
				ty: UniformType::Mat3x2 { order: UniformMatOrder::RowMajor },
				offset: dataview::offset_of!(TextUniform.transform) as u16,
				len: 1,
			},
			UniformAttribute {
				name: "u_texture",
				ty: UniformType::Sampler2D(0),
				offset: dataview::offset_of!(TextUniform.texture) as u16,
				len: 1,
			},
			UniformAttribute {
				name: "u_unit_range",
				ty: UniformType::F2,
				offset: dataview::offset_of!(TextUniform.unit_range) as u16,
				len: 1,
			},
			UniformAttribute {
				name: "u_threshold",
				ty: UniformType::F1,
				offset: dataview::offset_of!(TextUniform.threshold) as u16,
				len: 1,
			},
			UniformAttribute {
				name: "u_out_bias",
				ty: UniformType::F1,
				offset: dataview::offset_of!(TextUniform.out_bias) as u16,
				len: 1,
			},
			UniformAttribute {
				name: "u_outline_width_absolute",
				ty: UniformType::F1,
				offset: dataview::offset_of!(TextUniform.outline_width_absolute) as u16,
				len: 1,
			},
			UniformAttribute {
				name: "u_outline_width_relative",
				ty: UniformType::F1,
				offset: dataview::offset_of!(TextUniform.outline_width_relative) as u16,
				len: 1,
			},
			UniformAttribute {
				name: "u_gamma",
				ty: UniformType::F1,
				offset: dataview::offset_of!(TextUniform.gamma) as u16,
				len: 1,
			},
		],
	};
}
