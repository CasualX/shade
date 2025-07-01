use super::*;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SubMesh {
	pub vertex_start: u32,
	pub vertex_end: u32,
	pub bounds: Bounds3<f32>,
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SubIndexedMesh {
	pub vertex_start: u32,
	pub vertex_end: u32,
	pub index_start: u32,
	pub index_end: u32,
	pub bounds: Bounds3<f32>,
}
