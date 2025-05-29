/// Defines a type containing uniform data.
pub unsafe trait TUniform: Copy + Default + dataview::Pod {
	const LAYOUT: &'static UniformLayout;
}

static DUMMY_UNIFORM_LAYOUT: UniformLayout = UniformLayout {
	size: 0,
	alignment: 1,
	fields: &[],
};

/// Uniform reference.
///
/// This is a non-generic reference to uniform data.
#[derive(Copy, Clone)]
pub struct UniformRef<'a> {
	pub(crate) data_ptr: *const u8,
	pub(crate) layout: &'a UniformLayout,
}
impl<'a> Default for UniformRef<'a> {
	#[inline]
	fn default() -> Self {
		UniformRef {
			data_ptr: std::ptr::null(),
			layout: &DUMMY_UNIFORM_LAYOUT,
		}
	}
}
impl<'a, T: TUniform> From<&'a T> for UniformRef<'a> {
	#[inline]
	fn from(data: &'a T) -> Self {
		UniformRef {
			data_ptr: data as *const T as *const u8,
			layout: T::LAYOUT,
		}
	}
}

/// Matrix layout.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum MatrixLayout {
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
	Mat2x2 { order: MatrixLayout }, Mat2x3 { order: MatrixLayout }, Mat2x4 { order: MatrixLayout },
	Mat3x2 { order: MatrixLayout }, Mat3x3 { order: MatrixLayout }, Mat3x4 { order: MatrixLayout },
	Mat4x2 { order: MatrixLayout }, Mat4x3 { order: MatrixLayout }, Mat4x4 { order: MatrixLayout },
	Sampler2D,
}

/// Uniform field.
pub struct UniformField {
	pub name: &'static str,
	pub ty: UniformType,
	pub offset: u16,
	pub len: u16,
}

/// Uniform layout.
pub struct UniformLayout {
	pub size: u16,
	pub alignment: u16,
	pub fields: &'static [UniformField],
}
