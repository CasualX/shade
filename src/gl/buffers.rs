
use std::collections::HashMap;
use std::{mem, ops};

struct GlVertexBuffer {
	id: crate::VertexBuffer,
	buffer: gl::types::GLuint,
	vao: gl::types::GLuint,
	layout: &'static crate::VertexLayout,
	count: usize,
	init: bool,
}

pub struct GlVertexBuffers {
	vertex_buffers: HashMap<crate::VertexBuffer, GlVertexBuffer>,
	vertex_buffers_names: HashMap<String, crate::VertexBuffer>,
	vertex_buffers_next_id: u32,
}
