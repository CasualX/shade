define_handle!(UniformBuffer);

/// Defines a type containing uniform data.
pub unsafe trait TUniform: Copy + Default + dataview::Pod {
	const UNIFORM_LAYOUT: &'static UniformLayout;
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum UniformMatOrder {
	ColumnMajor,
	RowMajor,
}

/// Uniform data type.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum UniformType {
	D1, D2, D3, D4,
	F1, F2, F3, F4,
	I1, I2, I3, I4,
	U1, U2, U3, U4,
	B1, B2, B3, B4,
	Mat2x2 { order: UniformMatOrder }, Mat2x3 { order: UniformMatOrder }, Mat2x4 { order: UniformMatOrder },
	Mat3x2 { order: UniformMatOrder }, Mat3x3 { order: UniformMatOrder }, Mat3x4 { order: UniformMatOrder },
	Mat4x2 { order: UniformMatOrder }, Mat4x3 { order: UniformMatOrder }, Mat4x4 { order: UniformMatOrder },
	Sampler2D(u8),
}

/// Uniform attribute.
pub struct UniformAttribute {
	pub name: &'static str,
	pub ty: UniformType,
	pub offset: u16,
	pub len: u16,
}

/// Uniform memory layout.
pub struct UniformLayout {
	pub size: u16,
	pub alignment: u16,
	pub attributes: &'static [UniformAttribute],
}
