/*!
TShader Objects.
*/

use {TVertex, TUniform};

pub trait TShader: TUniform {
	type Vertex: TVertex;
	type Uniform: TUniform;

	fn shader_uid() -> u32;
}
